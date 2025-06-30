use tree_sitter::{Node, Parser};
use crate::ast::{AST, Kind, Class, Variable, Function};

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
  let class = extract_class(cl, code);

  let mut errors = vec![];

  let name = &class.name;

  match class.kind {
    Kind::Class(ref cl) => {
      if cl.is_abstract {
        errors.append(&mut check_abstract_class(&class, &name));
      } else {
        errors.append(&mut check_derived_class(&class, &name));
        if cl.derived_from.len() == 0 {
          errors.push(format!("Class '{name}' must be derived from abstract interface"));
        }
      }
      errors.append(&mut check_derives(&cl.derived_from, &name));
    }
    Kind::Unhandled(element) => errors.push(element),
    _ => todo!()
  }

  errors
}

fn extract_class(cl: &Node, code: &str) -> AST {
  let name = get_class_name(cl, code);
  let is_abstract = name.starts_with("Abstract");
  let mut derived_from = vec![];
  let mut children = vec![];

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "field_declaration_list" => {
        children.append(&mut extract_class_fields(&child, code));
      }
      "base_class_clause" => {
        let (mut derived, mut errors) = extract_derives(&child, code, &name);
        derived_from.append(&mut derived);
        children.append(&mut errors);
      }
      "type_identifier"|"class"|";" => (),
      _ => children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
      }),
    }
  }

  AST {
    name,
    kind: Kind::Class(Class {
      derived_from,
      is_abstract,
    }),
    children,
  }
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
      _ => todo!(),
    }
  }

  errors
}

fn extract_class_fields(fields: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  let mut access_specifier = "public";
  for idx in 0..fields.child_count() {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "access_specifier" => {
        access_specifier = &code[range.start..range.end];
      }
      "field_declaration" => children.push(extract_class_field(&child, code, access_specifier)),
      "type_identifier"|"class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
      }),
    }
  }

  children
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

fn extract_derives(fields: &Node, code: &str, class_name: &str) -> (Vec<String>, Vec<AST>) {
  let mut derived_from = vec![];
  let mut errors = vec![];

  for idx in 0..fields.child_count() {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_identifier" => derived_from.push(code[range.start..range.end].to_string()),
      "access_specifier" => if &code[range.start..range.end] != "public" {
        errors.push(AST {
          name: "".to_string(),
          kind: Kind::LintError(format!("Class '{class_name}': Derives must always be public")),
          children: vec![],
        });
      }
      "class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
      }),
    }
  }

  (derived_from, errors)
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

fn check_pure_virtual_ending(field: &Node, code: &str) -> bool {
  let mut missing_pure_virtual = false;

  let cnt = field.child_count();
  if cnt >= 3 {
    let child = field.child(cnt-1).unwrap();
    let range = child.byte_range();
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
  }

  !missing_pure_virtual
}

fn check_pure_virtual(field: &Node, code: &str) -> bool {
  let range = field.byte_range();
  if !code[range.start..range.end].starts_with("virtual") {
    return false;
  }

  check_pure_virtual_ending(field, code)
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

fn extract_class_field(field: &Node, code: &str, access_specifier: &str) -> AST {
  let mut errors = vec![];

  let mut name = "".to_string();
  let mut kind = Kind::Unhandled(field.to_sexp());
  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier" => {
        name = code[range.start..range.end].to_string();
        kind = Kind::Variable(Variable {
          visibility: access_specifier.to_string(),
          is_const: false,
        });
      }
      "function_declarator" => {
        let range = field.byte_range();
        name = code[range.start..range.end].to_string();
        kind = Kind::Function(Function {
          visibility: access_specifier.to_string(),
          is_virtual: check_pure_virtual(&field, code),
        });
      }
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual"|"primitive_type"|"number_literal" => (),
      _ => errors.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
      }),
    }
  }

  AST {
    name,
    kind,
    children: errors,
  }
}

fn prohibit_init_function(field: &AST, class_name: &str) -> Vec<String> {
  let mut errors = vec![];

  if field.name.contains("init") {
    errors.push(format!("Abstract class '{class_name}' should not provide an init function. Initialisation should be done in constructor."));
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

