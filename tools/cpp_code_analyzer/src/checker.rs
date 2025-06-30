use crate::ast::{AST, Kind, Function};

pub fn check_global_codechunk(ast: Vec<AST>) -> Vec<String> {
  let mut errors = vec![];
  for node in ast.into_iter() {
    errors.append(&mut error_message_from_ast(&node));
  }

  errors
}

fn check_abstract_class(node: &AST, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "public" {
          errors.push(format!("Abstract class `{class_name}` should ONLY define 'public' methods (not allowed {})", vl.visibility));
        }
        errors.push(format!("Abstract class `{class_name}` must not have attributes ('{}')", child.name));
      }
      Kind::Function(fun) => errors.append(&mut check_function_is_virtual(&child, &fun, class_name)),
      Kind::Unhandled(element) => errors.push(element.clone()),
      _ => todo!(),
    }
  }

  errors
}

fn check_derived_class(node: &AST, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "private" {
          errors.push(format!("Derived class '{class_name}' must not have non private attributes ('{}')", child.name));
        }
      }
      Kind::Function(fun) => errors.append(&mut check_function_is_not_virtual(&child, &fun, class_name)),
      Kind::LintError(msg) => errors.push(msg.clone()),
      Kind::Unhandled(element) => errors.push(element.clone()),
      _ => unreachable!(),
    }
  }

  errors
}

fn check_derives(derived_from: &Vec<String>, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for derived_from in derived_from.iter() {
    if !derived_from.starts_with("Abstract") {
      errors.push(format!("Class '{class_name}': Derives must always be from abstract interfaces"));
    }
  }

  errors
}

fn check_function_is_virtual(field: &AST, fun: &Function, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  errors.append(&mut prohibit_init_function(field, class_name));

  if !fun.is_virtual {
    if !field.name.starts_with("virtual") {
      errors.push(format!("method '{}' in abstract class '{class_name}' must be virtual", field.name));
    }

    if !field.name.replace(" ", "").ends_with("=0;") {
      errors.push(format!("Abstract class '{class_name}': missing `= 0;` for method '{}'", field.name));
    }
  }

  errors
}

fn check_function_is_not_virtual(field: &AST, fun: &Function, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  if !fun.is_virtual {
    if field.name.starts_with("virtual") {
      errors.push(format!("Derived class `{class_name}` must not define virtual functions ('{}')", field.name));
    }

    if field.name.replace(" ", "").ends_with("=0;") {
      errors.push(format!("Derived class '{class_name}' method '{}' should not be pure virtual", field.name));
    }
  }

  errors.append(&mut prohibit_init_function(field, class_name));

  errors
}

fn prohibit_init_function(field: &AST, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  if field.name.contains("init") {
    errors.push(format!("Abstract class '{class_name}' should not provide an init function. Initialisation should be done in constructor."));
  }

  errors
}

fn error_message_from_ast(input: &AST) -> Vec<String> {
  let mut errors = vec![];

  let name = &input.name;
  match &input.kind {
    Kind::Class(ref cl) => {
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&input, &name));
      } else {
        errors.append(&mut check_derived_class(&input, &name));
        if cl.derived_from.len() == 0 {
          errors.push(format!("Class '{name}' must be derived from abstract interface"));
        }
      }
      errors.append(&mut check_derives(&cl.derived_from, &name));
    }
    Kind::Unhandled(element) => errors.push(element.clone()),
    _ => todo!()
  }

  errors
}
