use tree_sitter::{Node, Parser};

pub fn analyze_cpp(input: &str) -> Vec<String> {
  let mut parser = Parser::new();
  parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading Cpp grammar");

  let tree = parser.parse(input, None).unwrap();
  let root_node = tree.root_node();

  let mut errors = vec![];

  for idx in 0..root_node.child_count() {
    let child = root_node.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => errors.append(&mut check_class(&child, input)),
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

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "field_declaration_list" => errors.append(&mut check_abstract_class(&child, code, &name)),
      "type_identifier"|"class"|";" => (),
      _ => errors.push(child.to_sexp()),
    }
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
      "type_identifier"|"class"|";"|"{"|"}"|"("|")"|":" => (),
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
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual"|"primitive_type"|"function_declarator" => (),
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

