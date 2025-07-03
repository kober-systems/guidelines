use crate::ast::{AST, Kind, Function, LintError};

pub fn check_global_codechunk(ast: &Vec<AST>, code: &str) -> Vec<LintError> {
  let mut errors = vec![];
  for node in ast.into_iter() {
    errors.append(&mut error_message_from_ast(&node, code));
  }

  errors
}

fn check_abstract_class(node: &AST, class_name: &str, code: &str) -> Vec<LintError> {
  let mut errors = vec![];

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
      Kind::Function(fun) => errors.append(&mut check_function_is_virtual(&child, &fun, class_name, code)),
      Kind::Unhandled(element) => errors.push(LintError {
        message: element.clone(),
        range: child.range.clone(),
      }),
      _ => todo!(),
    }
  }

  errors
}

fn check_derived_class(node: &AST, class_name: &str) -> Vec<LintError> {
  let mut errors = vec![];

  for child in node.children.iter() {
    match &child.kind {
      Kind::Variable(vl) => {
        if vl.visibility != "private" {
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

fn check_derives(derived_from: &Vec<String>, class: &AST) -> Vec<LintError> {
  let mut errors = vec![];

  let class_name = &class.name;
  for derived_from in derived_from.iter() {
    if !derived_from.starts_with("Abstract") {
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

fn error_message_from_ast(input: &AST, code: &str) -> Vec<LintError> {
  let mut errors = vec![];

  let name = &input.name;
  match &input.kind {
    Kind::File { content } => errors.append(&mut check_global_codechunk(&input.children, &content)),
    Kind::Class(ref cl) => {
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&input, &name, code));
      } else {
        errors.append(&mut check_derived_class(&input, &name));
        if cl.derived_from.len() == 0 {
          errors.push(LintError {
            message: format!("Class '{name}' must be derived from abstract interface"),
            range: input.range.clone(),
          });
        }
      }
      errors.append(&mut check_derives(&cl.derived_from, input));
    }
    Kind::Unhandled(element) => errors.push(LintError {
      message: element.clone(),
      range: input.range.clone(),
    }),
    _ => todo!()
  }

  errors
}
