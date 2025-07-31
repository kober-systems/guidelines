use crate::ast::{Class, Function, Kind, LintInstruction, Reference, Variable, AST};
use tree_sitter::{Node, Parser};

pub fn parse_cpp_chunc(name: &str, input: &str) -> AST {
  let mut parser = Parser::new();
  parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading Cpp grammar");

  let tree = parser.parse(input, None).unwrap();
  let root_node = tree.root_node();

  let mut base = AST {
    name: name.to_string(),
    kind: Kind::File { content: input.to_string() },
    range: root_node.byte_range(),
    ..AST::default()
  };
  parse_global_codechunk(&mut base, &root_node, input);

  base
}

fn parse_global_codechunk(base: &mut AST, cl: &Node, code: &str) {
  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => base.children.push(extract_class(&child, code)),
      "declaration" => base.children.append(&mut extract_declaration(&child, code, "public")),
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
      "enum_specifier" => base.children.append(&mut parse_enum(&child, code)),
      "type_definition" => parse_global_codechunk(base, &child, code),
      "struct_specifier" => base.children.push(parse_struct(&child, code)),
      "alias_declaration" => base.children.push(parse_alias(&child, code)),
      "type_identifier" => (),
      "function_definition" => base.children.push(extract_function(&child, code, "public")),
      _ => base.children.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
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
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Reference(Reference::Depend),
    children,
    range: node.byte_range(),
    ..AST::default()
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
              kind: Kind::LintError(format!("could not parse lint instruction in comment: {previous_comment}")) ,
              range: range.clone(),
              ..AST::default()
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
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
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
      "declaration"|"field_declaration" => children.append(&mut extract_declaration(&child, code, access_specifier)),
      "function_definition" => children.push(extract_function_definition(&child, code, access_specifier)),
      "type_definition" => children.push(parse_struct(&child, code)),
      "type_identifier"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      "alias_declaration" => children.push(parse_alias(&child, code)),
      _ => children.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range,
        ..AST::default()
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
        kind: Kind::Reference(Reference::Depend),
        range: child.byte_range(),
        ..AST::default()
      }),
      "template_type" => derived_from.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Depend),
        range: child.byte_range(),
        ..AST::default()
      }),
      "access_specifier" => if &code[range.start..range.end] != "public" {
        errors.push(AST {
          kind: Kind::LintError(format!("Class '{class_name}': Derives must always be public")),
          range: child.byte_range(),
          ..AST::default()
        });
      }
      "class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
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

fn extract_declaration(field: &Node, code: &str, access_specifier: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"array_declarator"|"field_identifier" => {
        children.push(AST {
          name: get_variable_name(&child, code),
          kind: Kind::Variable(Variable {
            visibility: access_specifier.to_string(),
            is_const: check_is_const(&field.parent().unwrap(), code),
          }),
          range,
          ..AST::default()
        });
      }
      "init_declarator" => {
        children.append(&mut extract_declaration(&child, code, access_specifier));
      },
      "pointer_declarator" => {
        let name = &code[range.start..range.end];
        children.push(AST {
          name: name.to_string(),
          kind: if name.contains("(") {
            Kind::Function(Function {
              visibility: access_specifier.to_string(),
              is_virtual: check_pure_virtual(&field, code),
              in_external_namespace: None,
            })
          } else {
            Kind::Variable(Variable {
              visibility: access_specifier.to_string(),
              is_const: check_is_const(&field.parent().unwrap(), code),
            })
          },
          range,
          ..AST::default()
        });
      }
      "function_declarator" => {
        children.push(extract_function_definition(&field, code, access_specifier));
      }
      "enum_specifier" => {
        children.append(&mut parse_enum(&child, code));
      }
      ";"|"{"|"}"|"("|")"|":"|"="|"," => (),
      "primitive_type"|"type_identifier"
        |"type_qualifier"|"storage_class_specifier"|"attribute_specifier"
        |"sizeof_expression"|"sized_type_specifier"|"virtual" => (),
      x if is_literal(x) => (),
      "initializer_list" => (),
      x if is_statement(x) => children.append(&mut extract_statement(&child, code)),
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_declaration: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_function_definition(field: &Node, code: &str, access_specifier: &str) -> AST {
  let mut errors = vec![];

  let mut name = "".to_string();
  let mut kind = Kind::Unhandled(format!("extract_function_definition: {}", field.to_sexp()));
  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "function_declarator" => {
        name = code[range.start..range.end].to_string();
        kind = Kind::Function(Function {
          visibility: access_specifier.to_string(),
          is_virtual: check_pure_virtual(&field, code),
          in_external_namespace: None,
        });
      }
      ";"|"{"|"}"|"("|")"|":"|"=" => (),
      "primitive_type"
        |"type_qualifier" => (),
      _ => errors.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name,
    kind,
    children: errors,
    range: field.byte_range(),
    ..AST::default()
  }
}

fn extract_function(field: &Node, code: &str, access_specifier: &str) -> AST {
  let (name, namespace) = get_function_name(field, code);
  let mut dependencies = vec![];
  let mut children = vec![];

  for idx in 0..field.child_count() {
    let child = field.child(idx).unwrap();
    match child.kind() {
      "primitive_type" => dependencies.push(AST {
        kind: Kind::Reference(Reference::TypeRead),
        range: child.byte_range(),
        ..AST::default()
      }),
      "function_declarator" => children.append(&mut extract_parameters(&child, code)),
      "type_identifier"|";"|"comment" => (),
      "compound_statement" => children.append(&mut extract_statement(&child, code)),
      "template_type" => (),
      "pointer_declarator" => (),
      "type_qualifier"|"storage_class_specifier" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_function: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name,
    kind: Kind::Function(Function {
      is_virtual: false,
      visibility: access_specifier.to_string(),
      in_external_namespace: namespace,
    }),
    children,
    dependencies,
    range: field.byte_range(),
    instructions: vec![],
  }
}

fn extract_statement(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      x if is_statement(x)  => children.append(&mut extract_statement(&child, code)),
      "identifier"|"qualified_identifier" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Read),
        range,
        ..AST::default()
      } ),
      x if is_update_expression(x) => children.append(&mut extract_update_expression(&child, code)),
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      "declaration" => children.append(&mut extract_declaration(&child, code, "public")),
      "("|")"|"{"|"}"|";"|"<"|">"|"!="|"<="|">="|"+"|"-"|"||"|"|"
        |"<<"|">>"|"&&"|"&"|"~"|"*"|"=="|"["|"]"|"!"|"/"|"%"|":" => (),
      "return"|"if"|"for"|"do"
        |"comment"|"else"|"while"|"switch"
        |"case"|"break_statement"|"default" => (),
      "sizeof_expression" => (),
      x if is_literal(x) => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_statement: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_update_expression(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"qualified_identifier" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Write),
        range,
        ..AST::default()
      } ),
      x if is_statement(x) => children.append(&mut extract_statement(&child, code)),
      x if is_update_expression(x) => children.append(&mut extract_update_expression(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
      x if is_literal(x) => (),
      "sizeof_expression"|"delete" => (),
      "("|")"|"{"|"}"|";"|"++"|"--"|"="|"+="|"*="|"-="|"^="
        |">>="|"|="|"&=" => (),
      "new" => (),
      "argument_list" => children.append(&mut extract_arguments(&child, code)),
      "primitive_type"|"type_identifier"|"struct_specifier"
        |"function_declarator" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::TypeRead),
        ..AST::default()
      }),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_update_expression: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_call_expression(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Call),
        range,
        ..AST::default()
      } ),
      "argument_list" => children.append(&mut extract_arguments(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      "("|")"|"{"|"}"|";" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_field_expression(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Read),
        range,
        ..AST::default()
      }),
      x if is_statement(x) => children.append(&mut extract_statement(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      "field_identifier" => (),
      "this" => (),
      "("|")"|"{"|"}"|";"|"."|"->" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_field_expression: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_parameters(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "parameter_declaration"|"optional_parameter_declaration" => children.push(extract_param(&child, code)),
      "("|")"|"," => (),
      "identifier" => (),
      "qualified_identifier" => (),
      "parameter_list" => children.append(&mut extract_parameters(&child, code)),
      _ => children.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_arguments(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "identifier" => {
        children.push(extract_argument(&child, code))
      }
      "pointer_expression" => children.append(&mut extract_arguments(&child, code) ),
      x if is_statement(x) => children.append(&mut extract_statement(&child, code)),
      "field_expression" => children.append(&mut extract_field_expression(&child, code)),
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
      "update_expression" => children.append(&mut extract_update_expression(&child, code)),
      "sizeof_expression" => (),
      "("|")"|","|"&"|"*" => (),
      "this" => (),
      x if is_literal(x) => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_arguments {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_argument(node: &Node, code: &str) -> AST {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier" => {
        name = &code[range.start..range.end];
      }
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_argument: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Variable(Variable {
      is_const: false,
      visibility: "public".to_string(),
    }),
    children,
    range: node.byte_range(),
    ..AST::default()
  }
}

fn extract_param(node: &Node, code: &str) -> AST {
  let mut dependencies = vec![];
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"pointer_declarator"|"reference_declarator" => {
        name = &code[range.start..range.end];
      }
      "primitive_type"|"type_identifier"|"struct_specifier"
        |"function_declarator" => dependencies.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::TypeRead),
        ..AST::default()
      }),
      "type_qualifier" => (),
      "=" => (),
      x if is_literal(x) => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_param: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: if name != "" {
      Kind::Variable(Variable {
        is_const: false,
        visibility: "public".to_string(),
      })
    } else {
      Kind::Unhandled(format!("parameter not parsable: {}", node.to_sexp()))
    },
    children,
    dependencies,
    range: node.byte_range(),
    ..AST::default()
  }
}

fn parse_enum(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_identifier" => {
        name = &code[range.start..range.end];
        children.push(AST {
          name: name.to_string(),
          kind: Kind::Type,
          range: node.byte_range(),
          ..AST::default()
        })
      },
      "enum"|"class"|";" => (),
      "enumerator_list" => children.append(&mut parse_enum_variant(&child, code, name)),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("parse_enum: {}", child.to_sexp())),
        range,
        ..AST::default()
      }),
    }
  }

  children
}

fn parse_enum_variant(node: &Node, code: &str, namespace: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "enumerator" => {
        let variant_name = get_variable_name(&child, code);
        let qualified_name = format!("{namespace}::{variant_name}");
        children.push(AST {
          name: variant_name,
          kind: Kind::Variable(Variable {
            is_const: true,
            visibility: "public".to_string(),
          }),
          range: range.clone(),
          ..AST::default()
        });
        children.push(AST {
          name: qualified_name,
          kind: Kind::Variable(Variable {
            is_const: true,
            visibility: "public".to_string(),
          }),
          range: range,
          ..AST::default()
        });
      },
      "{"|"}"|"," => (),
      "comment" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(child.to_sexp()),
        range,
        ..AST::default()
      }),
    }
  }

  children
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
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Type,
    children,
    range: node.byte_range(),
    ..AST::default()
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
        kind: Kind::Unhandled(child.to_sexp()),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  AST {
    name: name.to_string(),
    kind: Kind::Type,
    children,
    range: node.byte_range(),
    ..AST::default()
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

fn get_variable_name(node: &Node, code: &str) -> String {
  match node.kind() {
    "identifier"|"field_identifier" => {
      let range = node.byte_range();
      return code[range.start..range.end].to_string()
    },
    "array_declarator"|"enumerator" => {
      for idx in 0..node.child_count() {
        let child = node.child(idx).unwrap();
        match child.kind() {
          "identifier"|"field_identifier"|"array_declarator" => {
            return get_variable_name(&child, code)
          },
          _ => (),
        }
      }
    },
    _ => (),
  }

  panic!("each variable must have a name!")
}

fn get_function_name(cl: &Node, code: &str) -> (String, Option<String>) {
  let mut namespace = None;

  for idx in 0..cl.child_count() {
    let child = cl.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier" => {
        return (code[range.start..range.end].to_string(), namespace);
      },
      "namespace_identifier" => {
        namespace = Some(code[range.start..range.end].to_string());
      },
      "template_type"|"function_declarator"|"qualified_identifier"|"pointer_declarator" => {
        return get_function_name(&child, code)
      },
      _ => (),
    }
  }
  panic!("each function must have a name!")
}

fn check_is_const(node: &Node, code: &str) -> bool {
  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_qualifier" => {
        return &code[range.start..range.end] == "constexpr";
      },
      "declaration" => {
        return check_is_const(&child, code);
      }
      _ => (),
    }
  }
  false
}

fn is_literal(kind: &str) -> bool {
  match kind {
    "number_literal"|"string_literal"|"true"|"false"
      |"null"|"char_literal" => true,
    _ => false
  }
}

fn is_statement(kind: &str) -> bool {
  match kind {
    "return_statement"|"if_statement"|"condition_clause"
      |"compound_statement"|"expression_statement"
      |"for_statement"|"binary_expression"|"else_clause"
      |"unary_expression"|"parenthesized_expression"
      |"subscript_expression"|"subscript_argument_list"
      |"cast_expression"|"while_statement"|"pointer_expression"
      |"switch_statement"|"case_statement"
      |"do_statement"|"new_declarator" => true,
    _ => false
  }
}

fn is_update_expression(kind: &str) -> bool {
  match kind {
    "update_expression"|"assignment_expression"
      |"delete_expression"|"new_expression" => true,
    _ => false
  }
}
