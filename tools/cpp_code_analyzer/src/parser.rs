use crate::ast::{Class, Function, Kind, LintInstruction, Variable, AST};
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
    instructions: vec![],
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
      "preproc_ifdef"|"preproc_def"|"namespace_definition"
        |"declaration_list"|"preproc_if"|"preproc_elif"
        |"preproc_else" => parse_global_codechunk(base, &child, code),
      "preproc_include" => base.dependencies.push(parse_include(&child, code)),
      "identifier"|"namespace_identifier" => (), // ignoring identifiers on global level
      "template_declaration" => parse_global_codechunk(base, &child, code),
      "template_parameter_list" => (),
      "comment"|"#ifdef"|"#ifndef"|"#define"|"#endif"
        |"preproc_arg"|"namespace"|"#if"|"#elif"|"#else"
        |"preproc_defined"|"template"|"typedef" => (),
      ";"|"{"|"}"|"\n" => (),
      "enum_specifier" => base.children.push(parse_enum(&child, code)),
      "type_definition" => parse_global_codechunk(base, &child, code),
      "struct_specifier" => base.children.push(parse_struct(&child, code)),
      "alias_declaration" => base.children.push(parse_alias(&child, code)),
      "type_identifier" => (),
      "function_definition" => base.children.push(extract_function(&child, code, "public")),
      _ => base.children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
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
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Reference,
    children,
    dependencies: vec![],
    instructions: vec![],
    range: node.byte_range(),
  }
}

fn extract_class(cl: &Node, code: &str) -> AST {
  let name = get_class_name(cl, code);
  let is_abstract = name.starts_with("Abstract");
  let mut dependencies = vec![];
  let mut children = vec![];
  let mut instructions = vec![];

  if let Some(before) = cl.prev_sibling() {
    if before.kind() == "comment" {
      let range = before.byte_range();
      let previous_comment = &code[range.start..range.end];
      let mut next_is_instruction = false;
      const LINT_PATTERN: &str = "lint: ignore ";
      for instruction in previous_comment.split_inclusive(LINT_PATTERN) {
        if next_is_instruction {
          match instruction.split_once(" ") {
            Some((number, reason)) => instructions.push(LintInstruction {
              ident: number.to_string(),
              reason: reason.to_string(),
            }),
            None => children.push(AST {
              name: "".to_string(),
              kind: Kind::LintError(format!("could not parse lint instruction in comment: {previous_comment}")) ,
              children: vec![],
              dependencies: vec![],
              instructions: vec![],
              range: range.clone(),
            })
          }
        }
        next_is_instruction = instruction.ends_with(LINT_PATTERN);
      }
    }
  }

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "field_declaration_list" => {
        children.append(&mut extract_class_fields(&child, code));
      }
      "base_class_clause" => {
        let (mut derived, mut errors) = extract_derives(&child, code, &name);
        dependencies.append(&mut derived);
        children.append(&mut errors);
      }
      "type_identifier"|"class"|";" => (),
      "template_type" => (),
      _ => children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name,
    kind: Kind::Class(Class {
      is_abstract,
    }),
    children,
    dependencies,
    range: cl.byte_range(),
    instructions,
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
      "type_definition" => children.push(parse_struct(&child, code)),
      "type_identifier"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      "alias_declaration" => children.push(parse_alias(&child, code)),
      _ => children.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
        range,
      }),
    }
  }

  children
}

fn extract_derives(fields: &Node, code: &str, class_name: &str) -> (Vec<AST>, Vec<AST>) {
  let mut derived_from = vec![];
  let mut errors = vec![];

  for idx in 0..fields.child_count() {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_identifier" => derived_from.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference,
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
        range: child.byte_range(),
      }),
      "template_type" => derived_from.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference,
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
        range: child.byte_range(),
      }),
      "access_specifier" => if &code[range.start..range.end] != "public" {
        errors.push(AST {
          name: "".to_string(),
          kind: Kind::LintError(format!("Class '{class_name}': Derives must always be public")),
          children: vec![],
          dependencies: vec![],
          instructions: vec![],
          range: child.byte_range(),
        });
      }
      "class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
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

  let mut parsed_element: Option<AST> = None;
  let mut name = "".to_string();
  let mut kind = Kind::Unhandled(format!("extract_field_or_function: {}", field.to_sexp()));
  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "field_identifier"|"identifier"|"array_declarator" => {
        name = code[range.start..range.end].to_string();
        kind = Kind::Variable(Variable {
          visibility: access_specifier.to_string(),
          is_const: check_is_const(&field.parent().unwrap(), code),
        });
      }
      "init_declarator" => {
        parsed_element = Some(extract_field_or_function(&child, code, access_specifier))
      },
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
            is_const: check_is_const(&field.parent().unwrap(), code),
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
      "virtual"|"primitive_type"|"number_literal"
        |"type_qualifier" => (),
      "enum_specifier" => {
        parsed_element = Some(parse_enum(&child, code));
      }
      _ => errors.push(AST {
        name: "".to_string(),
        kind: Kind::Unhandled(child.to_sexp()),
        children: vec![],
        dependencies: vec![],
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  if let Some(mut ast) = parsed_element {
    errors.append(&mut ast.children);
    name = ast.name;
    kind = ast.kind;
  }
  AST {
    name,
    kind,
    children: errors,
    dependencies: vec![],
    instructions: vec![],
    range: field.byte_range(),
  }
}

fn extract_function(field: &Node, code: &str, access_specifier: &str) -> AST {
  extract_field_or_function(field, code, access_specifier)
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
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Type,
    children,
    dependencies: vec![],
    instructions: vec![],
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
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Type,
    children,
    dependencies: vec![],
    instructions: vec![],
    range: node.byte_range(),
  }
}

fn parse_alias(node: &Node, code: &str) -> AST {
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
        instructions: vec![],
        range: child.byte_range(),
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Type,
    children,
    dependencies: vec![],
    instructions: vec![],
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

fn check_is_const(node: &Node, code: &str) -> bool {
  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_qualifier" => {
        return &code[range.start..range.end] == "constexpr";
      },
      _ => (),
    }
  }
  false
}

