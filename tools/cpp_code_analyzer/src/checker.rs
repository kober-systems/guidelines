use std::collections::{HashMap, HashSet};

use crate::ast::{AST, Kind, Function, LintError, Reference};

pub fn check_global_codechunk(ast: &Vec<AST>, code: &str) -> Vec<LintError> {
  let vars = get_variables_from_all_classes(ast);
  error_message_from_global_codechunk(ast, code, &vars)
}

fn error_message_from_global_codechunk(ast: &Vec<AST>, code: &str, vars: &HashMap<String, HashSet<String>>) -> Vec<LintError> {

  let mut errors = vec![];
  for node in ast.into_iter() {
    errors.append(&mut error_message_from_ast(&node, code, &vars));
  }

  errors
}

pub fn add_lint_erros(ast: Vec<AST>) -> Vec<AST> {
  let vars = get_variables_from_all_classes(&ast);

  ast.into_iter().map(|mut node| {
    match &node.kind {
      Kind::File { content } => {
        node.children = node.children.into_iter().map(|mut node| {
          let errors = get_lint_errors_for_node(&node, &content, &vars);
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

fn check_abstract_class(node: &AST, class_name: &str, code: &str) -> Vec<LintError> {
  let mut errors = vec![];
  let mut has_default_destructor = false;

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "public" {
          errors.push(LintError {
            message: format!("Abstract class `{class_name}` should ONLY define 'public' methods (not allowed {})", vl.visibility),
            range: child.range.clone(),
          });
        }
        errors.push(LintError {
          message: format!("Abstract class `{class_name}` must not have attributes ('{}')", child.name),
          range: child.range.clone(),
        });
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
      }),
      _ => todo!(),
    }
  }

  if !has_default_destructor {
    errors.push(LintError {
      message: format!("Abstract class '{class_name}' should provide a default destructor."),
      range: node.range.clone(),
    });
  }

  errors
}

fn check_derived_class(node: &AST, class_name: &str) -> Vec<LintError> {
  let mut errors = vec![];

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "private" && node.instructions.iter().find(|inst| inst.ident == "E_MOD_01").iter().count() == 0 {
          errors.push(LintError {
            message: format!("Derived class '{class_name}' must not have non private attributes ('{}')", child.name),
            range: child.range.clone(),
          });
        }
      }
      Kind::Function(fun) => errors.append(&mut check_function_is_not_virtual(&child, &fun, class_name)),
      Kind::LintError(msg) => errors.push(LintError {
        message: msg.clone(),
        range: child.range.clone(),
      }),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: child.range.clone(),
      }),
      _ => unreachable!(),
    }
  }

  errors
}

fn check_derives(class: &AST) -> Vec<LintError> {
  let mut errors = vec![];

  let class_name = &class.name;
  for derived_from in class.dependencies.iter() {
    if !derived_from.name.starts_with("Abstract") {
      errors.push(LintError {
        message: format!("Class '{class_name}': Derives must always be from abstract interfaces"),
        range: class.range.clone(),
      });
    }
  }

  errors
}

fn check_function_is_virtual(field: &AST, fun: &Function, class_name: &str, code: &str) -> Vec<LintError> {
  let mut errors = vec![];

  errors.append(&mut prohibit_init_function(field, class_name));

  let function_code = code[field.range.start..field.range.end].to_string();
  if !fun.is_virtual {
    if !function_code.starts_with("virtual") {
      errors.push(LintError {
        message: format!("method '{function_code}' in abstract class '{class_name}' must be virtual"),
        range: field.range.clone(),
      });
    }

    if !function_code.replace(" ", "").ends_with("=0;") {
      errors.push(LintError {
        message: format!("Abstract class '{class_name}': missing `= 0;` for method '{function_code}'"),
        range: field.range.clone(),
      });
    }
  }

  errors
}

fn check_function_is_not_virtual(field: &AST, fun: &Function, class_name: &str) -> Vec<LintError> {
  let mut errors = vec![];

  if !fun.is_virtual {
    if field.name.starts_with("virtual") {
      errors.push(LintError {
        message: format!("Derived class `{class_name}` must not define virtual functions ('{}')", field.name),
        range: field.range.clone(),
      });
    }

    if field.name.replace(" ", "").ends_with("=0;") {
      errors.push(LintError {
        message: format!("Derived class '{class_name}' method '{}' should not be pure virtual", field.name),
        range: field.range.clone(),
      });
    }
  }

  errors.append(&mut prohibit_init_function(field, class_name));

  errors
}

fn prohibit_init_function(field: &AST, class_name: &str) -> Vec<LintError> {
  let mut errors = vec![];

  if field.name.contains("init") {
    errors.push(LintError {
      message: format!("Abstract class '{class_name}' should not provide an init function. Initialisation should be done in constructor."),
      range: field.range.clone(),
    });
  }

  errors
}

fn error_message_from_ast(input: &AST, code: &str, vars: &HashMap<String, HashSet<String>>) -> Vec<LintError> {
  let mut errors = vec![];

  match &input.kind {
    Kind::File { content } => errors.append(&mut error_message_from_global_codechunk(&input.children, &content, vars)),
    _ => errors.append(&mut get_lint_errors_for_node(input, code, vars)),
  }

  errors
}

fn get_lint_errors_for_node(input: &AST, code: &str, vars: &HashMap<String, HashSet<String>>) -> Vec<LintError> {
  let name = &input.name;
  match &input.kind {
    Kind::Class(ref cl) => {
      let mut errors = vec![];
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&input, &name, code));
      } else {
        errors.append(&mut check_derived_class(&input, &name));
        if input.dependencies.len() == 0 {
          errors.push(LintError {
            message: format!("Class '{name}' must be derived from abstract interface"),
            range: input.range.clone(),
          });
        }
      }
      errors.append(&mut check_derives(input));
      errors
    }
    Kind::Function(fun) => {
      match &fun.in_external_namespace {
        None => get_lint_errors_for_function(input, &HashSet::default()),
        Some(namespace) => get_lint_errors_for_function(input, vars.get(namespace).unwrap_or(&HashSet::default())),
      }
    },
    Kind::Type => vec![],
    Kind::Variable(var) => {
      let mut errors = vec![];
      if !var.is_const {
        errors.push(LintError {
          message: format!("It's not allowed to create global variables ('{}'). Global variables create invisible coupling.", input.name),
          range: input.range.clone(),
        });
      };
      errors
    }
    Kind::Unhandled(element) => vec![LintError {
      message: element.clone(),
      range: input.range.clone(),
    }],
    _ => todo!()
  }
}

fn get_lint_errors_for_function(input: &AST, class_vars: &HashSet<String>) -> Vec<LintError> {
  let mut errors = vec![];
  let vars_in_scope: HashSet<_> = input.children.iter().filter_map(|node| match node.kind {
    Kind::Variable(_) => Some(node.name.clone()),
    _ => None,
  }).collect();

  for node in input.children.iter() {
    match &node.kind {
      Kind::Reference(ref_kind) => {
        use Reference::*;
        match ref_kind {
          Read|Write => if !vars_in_scope.contains(&node.name) && !class_vars.contains(&node.name) {
            errors.push(LintError {
              message: format!("It's not allowed to use global variables ('{}'). Global variables create invisible coupling.", node.name),
              range: node.range.clone(),
            });
          }
          Call|TypeRead|Depend => (),
        }
      }
      Kind::Variable(_var) => (),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: node.range.clone(),
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
        let mut class_vars = HashSet::default();
        for child in node.children.iter() {
          match &child.kind {
            Kind::Variable(_) => { class_vars.insert(child.name.strip_prefix("*").unwrap_or(&child.name).trim().to_string()); },
            _ => (),
          }
        }
        vars.insert(node.name.clone(), class_vars);
      }
      _ => (),
    }
  }

  vars
}
