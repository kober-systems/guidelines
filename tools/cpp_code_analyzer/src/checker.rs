use tree_sitter::{Node, Parser};

pub fn analyze_cpp(input: &str) -> Vec<String> {
  let mut parser = Parser::new();
  parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading Cpp grammar");

  let tree = parser.parse(input, None).unwrap();
  let root_node = tree.root_node();

  check_global_codechunk(&root_node, input)
}

fn check_global_codechunk(cl: &Node, code: &str) -> Vec<String> {
  let mut errors = vec![];

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => errors.append(&mut check_class(&child, code)),
      "preproc_ifdef"|"preproc_def" => errors.append(&mut check_global_codechunk(&child, code)),
      "identifier" => (), // ignoring identifiers on global level
      "comment"|"#ifndef"|"#define"|"#endif" => (),
      ";" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  errors
}

fn check_class(cl: &Node, code: &str) -> Vec<String> {
  let mut errors = vec![];

  let name = get_class_name(cl, code);
  let is_abstact = name.starts_with("Abstract");
  let mut derived_from_interface = false;

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "field_declaration_list" => if is_abstact {
        errors.append(&mut check_abstract_class(&child, code, &name));
      } else {
        errors.append(&mut check_derived_class(&child, code, &name));
      }
      "base_class_clause" => {
        derived_from_interface = true;
      }
      "type_identifier"|"class"|";" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  if !is_abstact && !derived_from_interface {
    errors.push(format!("Class {name} must be derived from abstract interface"));
  }

  errors
}

fn check_abstract_class(fields: &Node, code: &str, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for idx in 0..fields.child_count() {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "access_specifier" => {
        let specifier = &code[range.start..range.end];
        if specifier != "public" {
          errors.push(format!("Abstract class `{class_name}` should ONLY define 'public' methods (not allowed {specifier})"));
        }
      }
      "field_declaration" => errors.append(&mut check_function_is_virtual(&child, code, class_name)),
      "type_identifier"|"class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  errors
}

fn check_derived_class(fields: &Node, code: &str, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  let mut access_specifier = "public";
  for idx in 0..fields.child_count() {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "access_specifier" => {
        access_specifier = &code[range.start..range.end];
      }
      "field_declaration" => errors.append(&mut check_function_is_not_virtual(&child, code, class_name, access_specifier)),
      "type_identifier"|"class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  errors
}

fn check_function_is_virtual(field: &Node, code: &str, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier" => {
        let field = &code[range.start..range.end];
        errors.push(format!("Abstract class `{class_name}` must not have attributes ('{}')", field));
        return errors;
      }
      "function_declarator" => errors.append(&mut prohibit_init_function(&child, code, &class_name)),
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual"|"primitive_type" => (),
      "number_literal" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  let child = field.child(0).unwrap();
  let range = child.byte_range();
  if code[range.start..range.end] != *"virtual" {
    let range = field.byte_range();
    errors.push(format!("method '{}' in abstract class '{class_name}' must be virtual", code[range.start..range.end].to_string()));
  }
  let cnt = field.child_count();
  if cnt >= 3 {
    let child = field.child(cnt-1).unwrap();
    let range = child.byte_range();
    let mut missing_pure_virtual = false;
    if code[range.start..range.end] != *";" {
      missing_pure_virtual = true;
    }
    let child = field.child(cnt-2).unwrap();
    let range = child.byte_range();
    if code[range.start..range.end] != *"0" {
      missing_pure_virtual = true;
    }
    let child = field.child(cnt-3).unwrap();
    let range = child.byte_range();
    if code[range.start..range.end] != *"=" {
      missing_pure_virtual = true;
    }
    if missing_pure_virtual {
      let range = field.byte_range();
      errors.push(format!("Abstract class '{class_name}': missing `= 0;` for method '{}'", code[range.start..range.end].to_string()));
    }
  }

  errors
}

fn check_function_is_not_virtual(field: &Node, code: &str, class_name: &str, access_specifier: &str) -> Vec<String> {
  let mut errors = vec![];

  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier" => {
        if access_specifier != "private" {
          let field = &code[range.start..range.end];
          errors.push(format!("Derived class `{class_name}` must not have non private attributes ('{}')", field));
        }
      }
      "function_declarator" => errors.append(&mut prohibit_init_function(&child, code, &class_name)),
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual" => {
        errors.push(format!("Derived class `{class_name}` must not define virtual functions ('{}')", field));
      }
      "primitive_type" => (),
      "number_literal" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  errors
}

fn prohibit_init_function(field: &Node, code: &str, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier" => {
        let field = &code[range.start..range.end];
        if field == "init" {
          errors.push(format!("Abstract class '{class_name}' should not provide an init function. Initialisation should be done in constructor."));
        }
      }
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual"|"primitive_type"|"parameter_list" => (),
      "number_literal" => (),
      _ => errors.push(child.to_sexp()),
    }
  }

  errors
}

fn get_class_name(cl: &Node, code: &str) -> String {
  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_identifier" => {
        return code[range.start..range.end].to_string()
      },
      _ => (),
    }
  }
  panic!("each class must have a name!")
}

