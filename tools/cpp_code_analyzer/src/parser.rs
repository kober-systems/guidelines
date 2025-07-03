use crate::ast::{AST, Kind, Class, Variable, Function};
use tree_sitter::{Node, Parser};

pub fn parse_cpp_chunc(name: &str, input: &str) -> Vec<AST> {
  let mut parser = Parser::new();
  parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading Cpp grammar");

  let tree = parser.parse(input, None).unwrap();
  let root_node = tree.root_node();

  let mut base = AST {
    name: name.to_string(),
    kind: Kind::File { content: input.to_string() },
    children: vec![],
    dependencies: vec![],
    range: root_node.byte_range(),
  };
  parse_global_codechunk(&mut base, &root_node, input);
  base.children
}

fn parse_global_codechunk(base: &mut AST, cl: &Node, code: &str) {
  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => base.children.push(extract_class(&child, code)),
      "preproc_ifdef"|"preproc_def" => { parse_global_codechunk(base, &child, code); },
      "identifier" => (), // ignoring identifiers on global level
      "comment"|"#ifndef"|"#define"|"#endif" => (),
      ";" => (),
      _ => base.children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        range: child.byte_range(),
      }),
    }
  }
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
        dependencies: vec![],
        range: child.byte_range(),
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
    dependencies: vec![],
    range: cl.byte_range(),
  }
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
        dependencies: vec![],
        range,
      }),
    }
  }

  children
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
          dependencies: vec![],
          range: child.byte_range(),
        });
      }
      "class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        range: child.byte_range(),
      }),
    }
  }

  (derived_from, errors)
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

fn extract_class_field(field: &Node, code: &str, access_specifier: &str) -> AST {
  let mut errors = vec![];

  let mut name = "".to_string();
  let mut kind = Kind::Unhandled(field.to_sexp());
  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier"|"pointer_declarator" => {
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
        dependencies: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name,
    kind,
    children: errors,
    dependencies: vec![],
    range: field.byte_range(),
  }
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

