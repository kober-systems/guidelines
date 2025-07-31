use std::collections::{HashMap, HashSet};

use crate::ast::{AST, Kind, Function, LintError, Reference};

pub fn check_global_codechunk(ast: &Vec<AST>) -> Vec<LintError> {
  let vars = get_variables_from_all_classes(ast);
  let constants = get_constants(&ast);
  let vars = InScope {
    constants,
    namespaces: vars,
  };
  let source = TextFile {
    content: "".to_string(),
    file_path: "".to_string(),
  };
  error_message_from_global_codechunk(ast, &source, &vars)
}

fn error_message_from_global_codechunk(ast: &Vec<AST>, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let mut errors = vec![];
  for node in ast.into_iter() {
    errors.append(&mut error_message_from_ast(&node, code, &vars));
  }

  errors
}

pub fn add_lint_erros(ast: Vec<AST>) -> Vec<AST> {
  let vars = get_variables_from_all_classes(&ast);
  let constants = get_constants(&ast);
  let vars = InScope {
    constants,
    namespaces: vars,
  };

  ast.into_iter().map(|mut node| {
    match &node.kind {
      Kind::File { content } => {
        let source = TextFile {
          content: content.clone(),
          file_path: node.name.clone(),
        };
        node.children = node.children.into_iter().map(|mut node| {
          let errors = get_lint_errors_for_node(&node, &source, &vars);
          for err in errors.into_iter() {
            node.children.push(AST {
              kind: Kind::LintError(err.message),
              range: err.range,
              ..AST::default()
            });
          }
          node
        }).collect();
      },
      _ => todo!(),
    }
    node
  }).collect()
}

fn check_abstract_class(node: &AST, class_name: &str, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];
  let mut has_default_destructor = false;

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "public" {
          errors.push(LintError {
            message: format!("Abstract class `{class_name}` should ONLY define 'public' methods (not allowed {})", vl.visibility),
            range: child.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
        if !vl.is_const {
          errors.push(LintError {
            message: format!("Abstract class `{class_name}` must not have attributes ('{}')", child.name),
            range: child.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
      }
      Kind::Function(fun) => {
        if fun.is_virtual && child.name == format!("~{class_name}()") {
          has_default_destructor = true;
        }
        errors.append(&mut check_function_is_virtual(&child, &fun, class_name, code));
      },
      Kind::Type => (),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: child.range.clone(),
        file_path: code.file_path.clone(),
      }),
      _ => todo!(),
    }
  }

  if !has_default_destructor {
    errors.push(LintError {
      message: format!("Abstract class '{class_name}' should provide a default destructor."),
      range: node.range.clone(),
      file_path: code.file_path.clone(),
    });
  }

  errors
}

fn check_derived_class(node: &AST, class_name: &str, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "private" && node.instructions.iter().find(|inst| inst.ident == "E_MOD_01").iter().count() == 0 {
          errors.push(LintError {
            message: format!("Derived class '{class_name}' must not have non private attributes ('{}')", child.name),
            range: child.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
      }
      Kind::Function(fun) => errors.append(&mut check_function_is_not_virtual(&child, &fun, class_name, code)),
      Kind::LintError(msg) => errors.push(LintError {
        message: msg.clone(),
        range: child.range.clone(),
        file_path: code.file_path.clone(),
      }),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: child.range.clone(),
        file_path: code.file_path.clone(),
      }),
      Kind::Type|Kind::Reference(_)  => (),
      _ => unreachable!(),
    }
  }

  errors
}

fn check_derives(class: &AST, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];

  let class_name = &class.name;
  for derived_from in class.dependencies.iter() {
    if !derived_from.name.starts_with("Abstract") {
      errors.push(LintError {
        message: format!("Class '{class_name}': Derives must always be from abstract interfaces"),
        range: class.range.clone(),
        file_path: code.file_path.clone(),
      });
    }
  }

  errors
}

fn check_function_is_virtual(field: &AST, fun: &Function, class_name: &str, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];

  errors.append(&mut prohibit_init_function(field, class_name, code));

  let function_code = code.content[field.range.start..field.range.end].to_string();
  if !fun.is_virtual {
    if !function_code.starts_with("virtual") {
      errors.push(LintError {
        message: format!("method '{function_code}' in abstract class '{class_name}' must be virtual"),
        range: field.range.clone(),
        file_path: code.file_path.clone(),
      });
    }

    if !function_code.replace(" ", "").ends_with("=0;") {
      errors.push(LintError {
        message: format!("Abstract class '{class_name}': missing `= 0;` for method '{function_code}'"),
        range: field.range.clone(),
        file_path: code.file_path.clone(),
      });
    }
  }

  errors
}

fn check_function_is_not_virtual(field: &AST, fun: &Function, class_name: &str, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];

  if !fun.is_virtual {
    if field.name.starts_with("virtual") {
      errors.push(LintError {
        message: format!("Derived class `{class_name}` must not define virtual functions ('{}')", field.name),
        range: field.range.clone(),
        file_path: code.file_path.clone(),
      });
    }

    if field.name.replace(" ", "").ends_with("=0;") {
      errors.push(LintError {
        message: format!("Derived class '{class_name}' method '{}' should not be pure virtual", field.name),
        range: field.range.clone(),
        file_path: code.file_path.clone(),
      });
    }
  }

  errors.append(&mut prohibit_init_function(field, class_name, code));

  errors
}

fn prohibit_init_function(field: &AST, class_name: &str, code: &TextFile) -> Vec<LintError> {
  let mut errors = vec![];

  if field.name.contains("init") {
    errors.push(LintError {
      message: format!("Abstract class '{class_name}' should not provide an init function. Initialisation should be done in constructor."),
      range: field.range.clone(),
      file_path: code.file_path.clone(),
    });
  }

  errors
}

fn error_message_from_ast(input: &AST, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let mut errors = vec![];

  match &input.kind {
    Kind::File { content } => {
      let source = TextFile {
        content: content.clone(),
        file_path: input.name.clone(),
      };
      errors.append(&mut error_message_from_global_codechunk(&input.children, &source, vars));
    }
    _ => errors.append(&mut get_lint_errors_for_node(input, code, vars)),
  }

  errors
}

fn get_lint_errors_for_node(input: &AST, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let name = &input.name;
  match &input.kind {
    Kind::Class(ref cl) => {
      let mut errors = vec![];
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&input, &name, code));
      } else {
        errors.append(&mut check_derived_class(&input, &name, code));
        if input.dependencies.len() == 0 {
          errors.push(LintError {
            message: format!("Class '{name}' must be derived from abstract interface"),
            range: input.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
      }
      errors.append(&mut check_derives(input, code));
      errors
    }
    Kind::Function(fun) => {
      match &fun.in_external_namespace {
        None => get_lint_errors_for_function(input, |name| { vars.constants.contains(name)  }, code),
        Some(namespace) => get_lint_errors_for_function(input, |name| {
          let empty = HashSet::default();
          let class_vars = vars.namespaces.get(namespace).unwrap_or(&empty);
          class_vars.contains(name) || vars.constants.contains(name)
        }, code),
      }
    },
    Kind::Type|Kind::Reference(_)  => vec![],
    Kind::Variable(var) => {
      let mut errors = vec![];
      if !var.is_const {
        errors.push(LintError {
          message: format!("It's not allowed to create global variables ('{}'). Global variables create invisible coupling.", input.name),
          range: input.range.clone(),
          file_path: code.file_path.clone(),
        });
      };
      errors
    }
    Kind::Unhandled(element) => vec![LintError {
      message: element.clone(),
      range: input.range.clone(),
      file_path: code.file_path.clone(),
    }],
    _ => todo!("{:?}", input.kind)
  }
}

fn get_lint_errors_for_function<F>(input: &AST, in_scope: F, code: &TextFile) -> Vec<LintError>
where
  F: Fn(&str) -> bool,
{
  let mut errors = vec![];
  let vars_in_scope = get_vars_in_scope(input);

  for node in input.children.iter() {
    match &node.kind {
      Kind::Reference(ref_kind) => {
        use Reference::*;
        match ref_kind {
          Read|Write => if !vars_in_scope.contains(&node.name) && !in_scope(&node.name) {
            errors.push(LintError {
              message: format!("It's not allowed to use global variables ('{}'). Global variables create invisible coupling.", node.name),
              range: node.range.clone(),
              file_path: code.file_path.clone(),
            });
          }
          Call|TypeRead|Depend => (),
        }
      }
      Kind::Variable(_var) => (),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: node.range.clone(),
        file_path: code.file_path.clone(),
      }),
      _ => todo!("node {:?} not yet implemented", node.kind)
    }
  }
  errors
}

fn get_variables_from_all_classes(ast: &Vec<AST>) -> HashMap<String, HashSet<String>> {
  let mut vars = HashMap::default();

  for node in ast.iter() {
    match &node.kind {
      Kind::File { content: _ } => vars.extend(get_variables_from_all_classes(&node.children)),
      Kind::Class(_) => {
        let class_vars = get_vars_in_scope(node);
        vars.insert(node.name.clone(), class_vars);
      }
      _ => (),
    }
  }

  vars
}

fn get_constants(ast: &Vec<AST>) -> HashSet<String> {
  let mut constants = HashSet::default();

  for node in ast.iter() {
    match &node.kind {
      Kind::File { content: _ } => constants.extend(get_constants_in_scope(node)),
      _ => (),
    }
  }

  constants
}

fn get_vars_in_scope(input: &AST) -> HashSet<String> {
  input.children.iter().filter_map(|node| match node.kind {
    Kind::Variable(_) => Some(node.name.trim().to_string()),
    _ => None,
  }).collect()
}

fn get_constants_in_scope(input: &AST) -> HashSet<String> {
  input.children.iter().filter_map(|node| match node.kind {
    Kind::Variable(ref v) => if v.is_const {
      Some(node.name.trim().to_string())
    } else {
      None
    }
    _ => None,
  }).collect()
}

struct TextFile {
  pub content: String,
  pub file_path: String,
}

struct InScope {
  pub constants: HashSet<String>,
  pub namespaces: HashMap<String, HashSet<String>>
}
