use crate::ast::{AST, Kind, Class, Variable, Function};
use tree_sitter::{Node, Parser};

pub fn parse_cpp_chunc(name: &str, input: &str) -> AST {
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

  base
}

fn parse_global_codechunk(base: &mut AST, cl: &Node, code: &str) {
  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => base.children.push(extract_class(&child, code)),
      "declaration" => base.children.push(extract_field_or_function(&child, code, "public")),
      "preproc_ifdef"|"preproc_def"|"namespace_definition"|"declaration_list"|"preproc_if" => parse_global_codechunk(base, &child, code),
      "preproc_include" => base.dependencies.push(parse_include(&child, code)),
      "identifier"|"namespace_identifier" => (), // ignoring identifiers on global level
      "template_declaration" => parse_global_codechunk(base, &child, code),
      "template_parameter_list" => (),
      "comment"|"#ifndef"|"#define"|"#endif"|"preproc_arg"|"namespace"|"#if"|"preproc_defined"|"template"|"typedef" => (),
      ";"|"{"|"}"|"\n" => (),
      "enum_specifier" => base.children.push(parse_enum(&child, code)),
      "type_definition" => parse_global_codechunk(base, &child, code),
      "struct_specifier" => base.children.push(parse_struct(&child, code)),
      "type_identifier" => (),
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

fn parse_include(node: &Node, code: &str) -> AST {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "string_literal"|"system_lib_string" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
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
    name: name.to_string(),
    kind: Kind::Reference,
    children,
    dependencies: vec![],
    range: node.byte_range(),
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
      "template_type" => (),
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
      "declaration"|"field_declaration" => children.push(extract_field_or_function(&child, code, access_specifier)),
      "function_definition" => children.push(extract_field_or_function(&child, code, access_specifier)),
      "type_identifier"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      "type_definition" => children.push(parse_struct(&child, code)),
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
      "template_type" => derived_from.push(code[range.start..range.end].to_string()),
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

fn check_pure_virtual_ending(code: &str) -> bool {
  code.replace(" ", "").ends_with("=0;")
}

fn check_pure_virtual(field: &Node, code: &str) -> bool {
  let range = field.byte_range();
  let code = &code[range.start..range.end];
  if !code.starts_with("virtual") {
    return false;
  }

  check_pure_virtual_ending(code) || is_default_destructor(field)
}

fn is_default_destructor(node: &Node) -> bool {
  let mut is_destructor = false;
  let mut is_default = false;

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "function_declarator" => is_destructor = check_is_destructor(&child),
      "default_method_clause" => is_default = true,
      _ => (),
    }
  }

  return is_destructor && is_default;
}

fn check_is_destructor(node: &Node) -> bool {
  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "destructor_name" => return true,
      _ => (),
    }
  }

  return false;
}

fn extract_field_or_function(field: &Node, code: &str, access_specifier: &str) -> AST {
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
      "pointer_declarator" => {
        name = code[range.start..range.end].to_string();
        if name.contains("(") {
          kind = Kind::Function(Function {
            visibility: access_specifier.to_string(),
            is_virtual: check_pure_virtual(&field, code),
          });
        } else {
          kind = Kind::Variable(Variable {
            visibility: access_specifier.to_string(),
            is_const: false,
          });
        }
      }
      "function_declarator" => {
        name = code[range.start..range.end].to_string();
        kind = Kind::Function(Function {
          visibility: access_specifier.to_string(),
          is_virtual: check_pure_virtual(&field, code),
        });
      }
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "virtual"|"primitive_type"|"number_literal" => (),
      "enum_specifier" => {
        return parse_enum(&child, code);
      }
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

fn parse_enum(node: &Node, code: &str) -> AST {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "type_identifier" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
      }
      "enumerator_list"|"class"|";" => (),
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
    name: name.to_string(),
    kind: Kind::Type,
    children,
    dependencies: vec![],
    range: node.byte_range(),
  }
}

fn parse_struct(node: &Node, code: &str) -> AST {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "type_identifier" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
      }
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
    name: name.to_string(),
    kind: Kind::Type,
    children,
    dependencies: vec![],
    range: node.byte_range(),
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
      "template_type" => {
        return get_class_name(&child, code)
      },
      _ => (),
    }
  }
  panic!("each class must have a name!")
}

