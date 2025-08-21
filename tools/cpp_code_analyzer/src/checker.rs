use std::collections::{HashMap, HashSet};

use crate::ast::{AST, Kind, Function, LintError, Reference};

pub fn check_global_codechunk(ast: Vec<AST>) -> Vec<LintError> {
  let vars = get_scope(&ast);
  let source = TextFile {
    content: "".to_string(),
    file_path: "".to_string(),
  };
  let ast = add_lint_errors_to_codechunk(ast, &vars);
  error_message_from_global_codechunk(ast, &source, &vars)
}

fn error_message_from_global_codechunk(ast: Vec<AST>, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let mut errors = vec![];
  for node in ast.into_iter() {
    errors.append(&mut error_message_from_ast(node, code, &vars));
  }

  errors
}

pub fn add_lint_errors(ast: Vec<AST>) -> Vec<AST> {
  let vars = get_scope(&ast);

  ast.into_iter().map(|mut node| {
    match &node.kind {
      Kind::File { content } => {
        let source = TextFile {
          content: content.clone(),
          file_path: node.name.clone(),
        };
        let has_main_entrypoint = check_if_has_main_entrypoint(&node);
        node.children = node.children.into_iter().map(|node| {
          add_lint_errors_for_node(node, &source, &vars, has_main_entrypoint)
        }).collect();
      },
      _ => todo!("{:?}", node.kind),
    }
    node
  }).collect()
}

fn add_lint_errors_to_codechunk(ast: Vec<AST>, vars: &InScope) -> Vec<AST> {
  ast.into_iter().map(|mut node| {
    match &node.kind {
      Kind::File { content } => {
        let source = TextFile {
          content: content.clone(),
          file_path: node.name.clone(),
        };
        let has_main_entrypoint = check_if_has_main_entrypoint(&node);
        node.children = node.children.into_iter().map(|node| {
          add_lint_errors_for_node(node, &source, &vars, has_main_entrypoint)
        }).collect();
      },
      _ => (),
    }
    node
  }).collect()
}

pub fn filter_references_in_scope(ast: Vec<AST>) -> Vec<AST> {
  let vars = get_scope(&ast);

  ast.into_iter().map(|mut node| {
    match &node.kind {
      Kind::File { content: _ } => {
        node.children = node.children.into_iter().map(|mut node| {
          match node.kind.clone() {
            Kind::Function(fun) => {
              match &fun.in_external_namespace {
                None => filter_references_in_function(node, |name| { vars.constants.contains(name)  }),
                Some(namespace) => filter_references_in_function(node, |name| {
                  let empty = HashSet::default();
                  let class_vars = vars.namespaces.get(namespace).unwrap_or(&empty);
                  class_vars.contains(name) || vars.constants.contains(name)
                }),
              }
            },
            Kind::Class(_) => {
              let class_name = &node.name;
              node.children = node.children.into_iter().map(|node| {
                match node.kind.clone() {
                  Kind::Function(_) => {
                    filter_references_in_function(node, |name| {
                      let empty = HashSet::default();
                      let class_vars = vars.namespaces.get(class_name).unwrap_or(&empty);
                      class_vars.contains(name) || vars.constants.contains(name)
                    })
                  },
                  _ => node,
                }
              }).collect();
              node
            },
            _ => node,
          }
        }).collect();
      },
      _ => (),
    }
    node
  }).collect()
}

fn get_scope(ast: &Vec<AST>) -> InScope {
  let vars = get_variables_from_all_classes(ast);
  let constants = get_constants(ast);
  InScope {
    constants,
    namespaces: vars,
  }
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
        if fun.is_virtual && child.name == format!("~{class_name}") {
          has_default_destructor = true;
        }
        errors.append(&mut check_function_is_virtual(&child, &fun, class_name, code));
      },
      Kind::Type|Kind::Reference(_)|Kind::LintError(_) => (),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: child.range.clone(),
        file_path: code.file_path.clone(),
      }),
      _ => todo!("{:?}", child.kind),
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

fn check_derived_class(node: AST, class_name: &str, code: &TextFile, vars: &InScope) -> AST {
  let mut node = node;
  let mut errors = vec![];

  node.children = node.children.into_iter().map(|child| {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "private" && node.instructions.iter().find(|inst| inst.ident == "E_MOD_01").iter().count() == 0 {
          errors.push(LintError {
            message: format!("Derived class '{class_name}' must not have non private attributes ('{}')", child.name),
            range: child.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
        child
      }
      Kind::Function(fun) => {
        errors.append(&mut check_function_is_not_virtual(&child, &fun, class_name, code));
        add_lint_errors_for_function(child, |name| {
          let empty = HashSet::default();
          let class_vars = vars.namespaces.get(&node.name).unwrap_or(&empty);
          class_vars.contains(name) || vars.constants.contains(name)
        }, code)
      },
      Kind::LintError(_) => child,
      Kind::Unhandled(element) => {
        errors.push(LintError {
          message: element.clone(),
          range: child.range.clone(),
          file_path: code.file_path.clone(),
        });
        child
      },
      Kind::Type|Kind::Reference(_) => child,
      _ => unreachable!(),
    }
  }).collect();

  for err in errors.into_iter() {
    node.children.push(AST {
      kind: Kind::LintError(err.message),
      range: err.range,
      ..AST::default()
    });
  }
  node
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

fn error_message_from_ast(input: AST, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let mut errors = vec![];

  match &input.kind {
    Kind::File { content } => {
      let source = TextFile {
        content: content.clone(),
        file_path: input.name.clone(),
      };
      errors.append(&mut error_message_from_global_codechunk(input.children, &source, vars));
    }
    _ => errors.append(&mut get_lint_errors_for_node(&input, code, vars)),
  }

  errors
}

fn get_lint_errors_for_node(input: &AST, code: &TextFile, vars: &InScope) -> Vec<LintError> {
  let mut errors = vec![];
  match &input.kind {
    Kind::Unhandled(element)|Kind::LintError(element) => errors.push(LintError {
      message: element.clone(),
      range: input.range.clone(),
      file_path: code.file_path.clone(),
    }),
    _ => {
      for child in input.children.iter() {
        errors.append(&mut get_lint_errors_for_node(child, code, vars));
      }
    }
  };
  errors
}

fn add_lint_errors_for_node(node: AST, code: &TextFile, vars: &InScope, has_main_entrypoint: bool) -> AST {
  let mut node = node;
  let mut errors = vec![];
  let name = &node.name.clone();
  match &node.kind.clone() {
    Kind::Class(ref cl) => {
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&node, &name, code));
      } else {
        node = check_derived_class(node, &name, code, vars);
        if node.dependencies.len() == 0 {
          errors.push(LintError {
            message: format!("Class '{name}' must be derived from abstract interface"),
            range: node.range.clone(),
            file_path: code.file_path.clone(),
          });
        }
      }
      errors.append(&mut check_derives(&node, code));
    }
    Kind::Function(fun) => {
      node = match &fun.in_external_namespace {
        None => add_lint_errors_for_function(node, |name| { vars.constants.contains(name) }, code),
        Some(namespace) => add_lint_errors_for_function(node, |name| {
          let empty = HashSet::default();
          let class_vars = vars.namespaces.get(namespace).unwrap_or(&empty);
          class_vars.contains(name) || vars.constants.contains(name)
        }, code),
      };
    },
    Kind::Type|Kind::Reference(_) => (),
    Kind::Variable(var) => {
      if !var.is_const && !has_main_entrypoint {
        errors.push(LintError {
          message: format!("It's not allowed to create global variables ('{}'). Global variables create invisible coupling.", node.name),
          range: node.range.clone(),
          file_path: code.file_path.clone(),
        });
      };
    }
    Kind::Unhandled(element) => errors.push(LintError {
      message: element.clone(),
      range: node.range.clone(),
      file_path: code.file_path.clone(),
    }),
    _ => todo!("{:?}", node.kind)
  };

  for err in errors.into_iter() {
    node.children.push(AST {
      kind: Kind::LintError(err.message),
      range: err.range,
      ..AST::default()
    });
  }
  node
}

fn add_lint_errors_for_function<F>(input: AST, in_scope: F, code: &TextFile) -> AST
where
  F: Fn(&str) -> bool,
{
  let mut input = input;
  let mut errors = vec![];
  let vars_in_scope = get_vars_in_scope(&input);

  input.children = input.children.into_iter().map(|mut node| {
    match &node.kind {
      Kind::Reference(ref_kind) => {
        use Reference::*;
        match ref_kind {
          Read|Write => if !vars_in_scope.contains(&node.name) && !in_scope(&node.name) {
            node.children.push(AST {
              kind: Kind::LintError(format!("It's not allowed to use global variables ('{}'). Global variables create invisible coupling.", node.name)),
              range: node.range.clone(),
              ..AST::default()
            });
          }
          Call|TypeRead|Depend => (),
        }
        node
      }
      Kind::Variable(_var) => node,
      Kind::Unhandled(element) => {
        errors.push(LintError {
          message: element.clone(),
          range: node.range.clone(),
          file_path: code.file_path.clone(),
        });
        node
      },
      _ => todo!("node {:?} not yet implemented", node.kind)
    }
  }).collect();

  for err in errors.into_iter() {
    input.children.push(AST {
      kind: Kind::LintError(err.message),
      range: err.range,
      ..AST::default()
    });
  }
  input
}

fn filter_references_in_function<F>(input: AST, in_scope: F) -> AST
where
  F: Fn(&str) -> bool,
{
  let vars_in_scope = get_vars_in_scope(&input);

  let AST { name, kind, children, dependencies, range, instructions } = input;
  let children = children.into_iter().filter(|node| {
    match &node.kind {
      Kind::Reference(ref_kind) => {
        use Reference::*;
        match ref_kind {
          Read|Write => !vars_in_scope.contains(&node.name) && !in_scope(&node.name),
          Call|TypeRead|Depend => true,
        }
      }
      _ => true,
    }
  }).collect();
  AST {
    name,
    kind,
    children,
    dependencies,
    range,
    instructions
  }
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

fn check_if_has_main_entrypoint(input: &AST) -> bool {
  let mut has_setup = false;
  let mut has_loop = false;
  for node in input.children.iter() {
    match node.kind {
      Kind::Function(_) => match node.name.trim() {
        "main" => {
          return true;
        }
        "setup" => {
          if has_loop { return true; }
          has_setup = true;
        }
        "loop" => {
          if has_setup { return true; }
          has_loop = true;
        }
        _ => (),
      }
      _ => (),
    }
  }

  false
}

