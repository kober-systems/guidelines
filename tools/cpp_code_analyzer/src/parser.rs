use ravioli_core::ast::{Class, Function, Kind, LintInstruction, LintErrorTypes, Reference, Variable, AST};
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
  for idx in 0..cl.child_count() as u32 {
    let child = cl.child(idx).unwrap();
    match child.kind() {
      "class_specifier" => base.children.push(extract_class(&child, code)),
      "declaration" => base.children.append(&mut extract_declaration(&child, code, "public")),
      "preproc_ifdef"|"preproc_def"|"namespace_definition"
        |"declaration_list"|"preproc_if"|"preproc_elif"
        |"preproc_else"|"binary_expression"|"unary_expression" => parse_global_codechunk(base, &child, code),
      "preproc_call" => (),
      "preproc_include" => base.dependencies.push(parse_include(&child, code)),
      "identifier"|"namespace_identifier" => (), // ignoring identifiers on global level
      "template_declaration" => parse_global_codechunk(base, &child, code),
      "template_parameter_list" => (),
      "comment"|"#ifdef"|"#ifndef"|"#define"|"#endif"
        |"preproc_arg"|"namespace"|"#if"|"#elif"|"#else"
        |"preproc_defined"|"template"|"typedef" => (),
      ";"|"{"|"}"|"\n" => (),
      x if is_read_operator(x) => (),
      x if is_literal(x) => (),
      "enum_specifier" => base.children.append(&mut parse_enum(&child, code)),
      "type_definition" => parse_global_codechunk(base, &child, code),
      "struct_specifier" => base.children.push(parse_struct(&child, code)),
      "alias_declaration" => base.children.push(parse_alias(&child, code)),
      "type_identifier" => (),
      "function_definition" => base.children.push(extract_function(&child, code, "public")),
      _ => base.children.push(AST {
        kind: Kind::Unhandled(format!("parse_global_codechunk: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }
}

fn parse_include(node: &Node, code: &str) -> AST {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "string_literal"|"system_lib_string" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
      }
      "type_identifier"|"class"|";"|"#include" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("parse_include: {}", child.to_sexp())),
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

  find_lint_instructions(cl, code, &mut instructions, &mut children);

  for idx in 0..cl.child_count() as u32 {
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
        kind: Kind::Unhandled(format!("extract_class: {}", child.to_sexp())),
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

fn find_lint_instructions(node: &Node, code: &str, instructions: &mut Vec<LintInstruction>, children: &mut Vec<AST>) {
  if let Some(before) = node.prev_sibling() {
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
              kind: Kind::LintError(LintErrorTypes::LintInstructionNotParseble(previous_comment.to_string())),
              range: range.clone(),
              ..AST::default()
            })
          }
        }
        next_is_instruction = instruction.ends_with(LINT_PATTERN);
      }
    }
  }
}

fn extract_class_fields(fields: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  let mut access_specifier = "public";
  for idx in 0..fields.child_count() as u32 {
    let child = fields.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "access_specifier" => {
        access_specifier = &code[range.start..range.end];
      }
      "declaration"|"field_declaration" => children.append(&mut extract_declaration(&child, code, access_specifier)),
      "function_definition" => children.push(extract_function(&child, code, access_specifier)),
      "type_definition" => children.push(parse_struct(&child, code)),
      "type_identifier"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      "alias_declaration" => children.push(parse_alias(&child, code)),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_class_fields: {}", child.to_sexp())),
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

  for idx in 0..fields.child_count() as u32 {
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
          kind: Kind::LintError(LintErrorTypes::CppDerivesAlwaysPublic(class_name.to_string())),
          range: child.byte_range(),
          ..AST::default()
        });
      }
      "class"|"comment"|";"|"{"|"}"|"("|")"|":" => (),
      _ => errors.push(AST {
        kind: Kind::Unhandled(format!("extract_derives: {}", child.to_sexp())),
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

  for idx in 0..node.child_count() as u32 {
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
  for idx in 0..node.child_count() as u32 {
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
  for idx in 0..field.child_count() as u32 {
    let child = field.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"array_declarator"|"field_identifier"
        |"reference_declarator" => {
        children.push(AST {
          name: get_variable_name(&child, code),
          kind: Kind::Variable(Variable {
            visibility: access_specifier.to_string(),
            is_const: check_is_const(&field.parent().unwrap(), code),
          }),
          dependencies: declaration_type_dependencies(field, code),
          range,
          ..AST::default()
        });
      }
      "qualified_identifier" => (),
      "init_declarator" => {
        children.append(&mut extract_declaration(&child, code, access_specifier));
      },
      "pointer_declarator" => {
        let is_function = code[range.start..range.end].contains("(");
        let name = if is_function {
          let (name, _namespace) = get_function_name(&child, code);
          name
        } else {
          get_variable_name(&child, code)
        };
        children.push(AST {
          name: name.to_string(),
          kind: if is_function {
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
          dependencies: declaration_type_dependencies(field, code),
          range,
          ..AST::default()
        });
      }
      "function_declarator" => {
        children.push(extract_function(&field, code, access_specifier));
      }
      "enum_specifier" => {
        children.append(&mut parse_enum(&child, code));
      }
      ";"|"{"|"}"|"("|")"|":"|"="|","|"*" => (),
      x if is_primitive_type(x) => (),
      "placeholder_type_specifier"
        |"type_qualifier"|"storage_class_specifier"|"attribute_specifier"
        |"sizeof_expression"|"virtual" => (),
      "type_identifier" => {
        children.push(AST {
          name: code[range.start..range.end].to_string(),
          kind: Kind::Reference(Reference::TypeRead),
          range,
          ..AST::default()
        });
      }
      x if is_literal(x) => (),
      "initializer_list" => (),
      x if is_statement(x) => children.append(&mut extract_statement(&child, code)),
      x if is_update_expression(x) => children.append(&mut extract_update_expression(&child, code)),
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
      "argument_list" => children.append(&mut extract_arguments(&child, code)),
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

fn declaration_type_dependencies(field: &Node, code: &str) -> Vec<AST> {
  (0..field.child_count() as u32)
    .filter_map(|idx| field.child(idx))
    .filter_map(|child| match child.kind() {
      kind if is_primitive_type(kind)
        && &code[child.byte_range().start..child.byte_range().end] != "void" => {
          Some(type_reference(&child, code))
        }
      "qualified_identifier"|"template_type" => Some(type_reference(&child, code)),
      _ => None,
    })
    .collect()
}

fn type_reference(node: &Node, code: &str) -> AST {
  let range = node.byte_range();
  let text = &code[range.start..range.end];
  let dependencies = text.find('<').map_or_else(Vec::new, |open| {
    let base = &text[..open];
    let arguments = &text[open + 1..text.len() - 1];
    let mut dependencies = vec![AST {
      name: base.to_string(),
      kind: Kind::Reference(Reference::TypeRead),
      range: range.start..range.start + open,
      ..AST::default()
    }];

    let mut argument_start = open + 1;
    for raw_argument in arguments.split(',') {
      let argument = raw_argument.trim();
      if !argument.is_empty() {
        let start = range.start + argument_start + raw_argument.find(argument).unwrap();
        dependencies.push(AST {
          name: argument.to_string(),
          kind: Kind::Reference(Reference::TypeRead),
          range: start..start + argument.len(),
          ..AST::default()
        });
      }
      argument_start += raw_argument.len() + 1;
    }
    dependencies
  });

  AST {
    name: code[range.start..range.end].to_string(),
    kind: Kind::Reference(Reference::TypeRead),
    dependencies,
    range,
    ..AST::default()
  }
}

fn function_name(node: &Node, code: &str) -> Option<(String, Option<String>)> {
  if node.kind() == "function_declarator" {
    return Some(get_function_name(node, code));
  }

  (0..node.child_count() as u32)
    .filter_map(|idx| node.child(idx))
    .find_map(|child| function_name(&child, code))
}

fn extract_function(field: &Node, code: &str, access_specifier: &str) -> AST {
  let (name, namespace) = function_name(field, code)
    .unwrap_or_else(|| get_function_name(field, code));
  let mut dependencies = vec![];
  let mut children = vec![];

  for idx in 0..field.child_count() as u32 {
    let child = field.child(idx).unwrap();
    match child.kind() {
      x if is_primitive_type(x) => {
        if &code[child.byte_range().start..child.byte_range().end] != "void" {
          dependencies.push(type_reference(&child, code));
        }
      }
      "qualified_identifier"|"template_type" => dependencies.push(type_reference(&child, code)),
      "function_declarator" => children.append(&mut extract_parameters(&child, code)),
      "type_identifier"|";"|"comment" => (),
      "compound_statement" => children.append(&mut extract_statement(&child, code)),
      "field_initializer_list" => (),
      "pointer_declarator"|"reference_declarator" => children.append(&mut extract_parameters(&child, code)),
      x if is_literal(x) => (),
      "=" => (),
      "initializer_list" => (),
      "type_qualifier"|"storage_class_specifier" => (),
      "virtual"|"default_method_clause" => (),
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
      is_virtual: check_pure_virtual(&field, code),
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

  for idx in 0..node.child_count() as u32 {
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
      "reference_declarator" => children.push(AST {
        name: get_variable_name(&child, code),
        kind: Kind::Variable(Variable {
          is_const: false,
          visibility: "public".to_string(),
        }),
        range,
        ..AST::default()
      }),
      "("|")"|"{"|"}"|";"|"["|"]"|":" => (),
      x if is_read_operator(x) => (),
      "return"|"if"|"for"|"do"|"continue_statement"
        |"comment"|"else"|"while"|"switch"
        |"case"|"break_statement"|"default"|"type_qualifier"
        |"type_identifier" => (),
      "sizeof_expression" => (),
      x if is_literal(x) => (),
      "type_descriptor" => (),
      "field_identifier" => (),
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

  for idx in 0..node.child_count() as u32 {
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
      "("|")"|"{"|"}"|";" => (),
      x if is_modify_operator(x) => (),
      x if is_primitive_type(x) => (),
      "new" => (),
      "argument_list" => children.append(&mut extract_arguments(&child, code)),
      "type_identifier"|"struct_specifier"
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
  let mut name = None;
  let mut range = node.byte_range();
  let mut dependencies = vec![];
  let mut member_target = None;

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let child_range = child.byte_range();
    match child.kind() {
      "identifier"|"qualified_identifier" => {
        name = Some(code[child_range.start..child_range.end].to_string());
        range = child_range;
      }
      "template_function" => children.append(&mut extract_call_expression(&child, code)),
      "template_argument_list" => dependencies.append(&mut extract_template_arguments(&child, code)),
      "argument_list" => dependencies.append(&mut extract_arguments(&child, code)),
      "field_expression" => member_target = member_call_target(&child, code),
      "("|")"|"{"|"}"|";" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_call_expression: {}", child.to_sexp())),
        range: child_range,
        ..AST::default()
      }),
    }
  }

  if let Some((mut target, member_name, member_range)) = member_target {
    append_member_call(&mut target, AST {
      name: member_name,
      kind: Kind::Reference(Reference::Call),
      dependencies,
      range: member_range,
      ..AST::default()
    });
    children.push(target);
  } else if let Some(name) = name {
    children.insert(0, AST {
      name,
      kind: Kind::Reference(Reference::Call),
      dependencies,
      range,
      ..AST::default()
    });
  } else {
    children.append(&mut dependencies);
  }

  children
}

fn member_call_target(node: &Node, code: &str) -> Option<(AST, String, std::ops::Range<usize>)> {
  let mut target = None;
  let mut member = None;

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx)?;
    let range = child.byte_range();
    match child.kind() {
      "identifier" => target = Some(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Read),
        range,
        ..AST::default()
      }),
      // A chained call is the receiver of the next member call.
      "call_expression" => target = extract_call_expression(&child, code).into_iter().next(),
      "field_identifier" => member = Some((code[range.start..range.end].to_string(), range)),
      _ => (),
    }
  }

  target.zip(member).map(|(target, (name, range))| (target, name, range))
}

fn append_member_call(target: &mut AST, call: AST) {
  if let Some(previous_call) = target.children.last_mut()
    .filter(|child| matches!(&child.kind, Kind::Reference(Reference::Call))) {
    append_member_call(previous_call, call);
  } else {
    target.children.push(call);
  }
}

fn extract_template_type(node: &Node, code: &str) -> Vec<AST> {
  let mut dependencies = vec![];

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "type_identifier" => dependencies.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::TypeRead),
        ..AST::default()
      }),
      "template_argument_list" => dependencies.append(&mut extract_template_arguments(&child, code)),
      "type_qualifier" => (),
      _ => dependencies.push(AST {
        kind: Kind::Unhandled(format!("extract_template_type: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  dependencies
}

fn extract_template_arguments(node: &Node, code: &str) -> Vec<AST> {
  let mut dependencies = vec![];

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      x if is_primitive_type(x) => (),
      "type_descriptor" => dependencies.append(&mut extract_template_arguments(&child, code)),
      "type_identifier"|"struct_specifier"
        |"function_declarator" => dependencies.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::TypeRead),
        ..AST::default()
      }),
      "type_qualifier"|"abstract_pointer_declarator" => (),
      "<"|">"|"("|")"|"{"|"}"|"," => (),
      _ => dependencies.push(AST {
        kind: Kind::Unhandled(format!("extract_template_arguments: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  dependencies
}

fn extract_field_expression(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() as u32 {
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
      "call_expression" => children.append(&mut extract_call_expression(&child, code)),
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

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "parameter_declaration"|"optional_parameter_declaration" => children.append(&mut extract_param(&child, code)),
      "("|")"|"," => (),
      "field_identifier"|"identifier"|"destructor_name" => (),
      "qualified_identifier" => (),
      "operator_name"|"type_qualifier"|"virtual_specifier"|"&" => (),
      x if is_primitive_type(x) => (),
      "parameter_list"|"function_declarator"|"reference_declarator"|"pointer_declarator" => children.append(&mut extract_parameters(&child, code)),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("extract_parameters: {}", child.to_sexp())),
        range: child.byte_range(),
        ..AST::default()
      }),
    }
  }

  children
}

fn extract_arguments(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"qualified_identifier" => children.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::Read),
        range,
        ..AST::default()
      }),
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

fn extract_param(node: &Node, code: &str) -> Vec<AST> {
  let mut dependencies = vec![];
  let mut children = vec![];
  let mut name = "".to_string();

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"pointer_declarator"|"reference_declarator"
        |"array_declarator" => {
        name = get_variable_name(&child, code);
      }
      x if is_primitive_type(x) => (),
      "type_identifier"|"struct_specifier"|"qualified_identifier"
        |"function_declarator" => dependencies.push(AST {
        name: code[range.start..range.end].to_string(),
        kind: Kind::Reference(Reference::TypeRead),
        ..AST::default()
      }),
      "template_type" => dependencies.append(&mut extract_template_type(&child, code)),
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

  if name != "" {
    vec![AST {
      name: name.to_string(),
      kind: Kind::Variable(Variable {
        is_const: false,
        visibility: "public".to_string(),
      }),
      children,
      dependencies,
      range: node.byte_range(),
      ..AST::default()
    }]
  } else { vec![] }
}

fn parse_enum(node: &Node, code: &str) -> Vec<AST> {
  let mut children = vec![];
  let mut name = "";

  for idx in 0..node.child_count() as u32 {
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

  for idx in 0..node.child_count() as u32 {
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
        kind: Kind::Unhandled(format!("parse_enum_variant: {}", child.to_sexp())),
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

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "type_identifier" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
      }
      "typedef"|"struct"|"field_declaration_list"
        |"struct_specifier"|";" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("parse_struct: {}", child.to_sexp())),
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

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    match child.kind() {
      "type_identifier" => {
        let range = child.byte_range();
        name = &code[range.start..range.end];
      }
      "using"|"="|"type_descriptor"|";" => (),
      _ => children.push(AST {
        kind: Kind::Unhandled(format!("parse_alias: {}", child.to_sexp())),
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

fn get_class_name(node: &Node, code: &str) -> String {
  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
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
  let range = node.byte_range();
  let name = code[range.start..range.end].to_string();
  log::error!("each class must have a name: {name}: {}", node.to_sexp());
  name
}

fn get_variable_name(node: &Node, code: &str) -> String {
  match node.kind() {
    "identifier"|"field_identifier"|"qualified_identifier" => {
      let range = node.byte_range();
      return code[range.start..range.end].to_string()
    },
    "array_declarator"|"enumerator"|"pointer_declarator"
      |"reference_declarator" => {
      for idx in 0..node.child_count() as u32 {
        let child = node.child(idx).unwrap();
        match child.kind() {
          "identifier"|"field_identifier"|"qualified_identifier"
            |"array_declarator"|"pointer_declarator" => {
            return get_variable_name(&child, code)
          },
          _ => (),
        }
      }
    },
    _ => (),
  }

  let range = node.byte_range();
  let name = code[range.start..range.end].to_string();
  log::error!("could not find variable name: {name}: {}", node.to_sexp());
  name
}

fn get_function_name(node: &Node, code: &str) -> (String, Option<String>) {
  let mut namespace = None;

  for idx in 0..node.child_count() as u32 {
    let child = node.child(idx).unwrap();
    let range = child.byte_range();
    match child.kind() {
      "identifier"|"field_identifier"|"destructor_name"
        |"operator_name" => {
        return (code[range.start..range.end].to_string(), namespace);
      },
      "namespace_identifier" => {
        namespace = Some(code[range.start..range.end].to_string());
      },
      "template_type"|"function_declarator"|"qualified_identifier"
        |"pointer_declarator"|"reference_declarator" => {
        return get_function_name(&child, code)
      },
      _ => (),
    }
  }
  let range = node.byte_range();
  let name = code[range.start..range.end].to_string();
  log::error!("could not find function name: {name} {}", node.to_sexp());
  (name, namespace)
}

fn check_is_const(node: &Node, code: &str) -> bool {
  for idx in 0..node.child_count() as u32 {
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
      |"null"|"char_literal"|"concatenated_string" => true,
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
      |"switch_statement"|"case_statement"|"conditional_expression"
      |"do_statement"|"new_declarator"|"for_range_loop" => true,
    _ => false
  }
}

fn is_read_operator(kind: &str) -> bool {
  match kind {
    "<"|">"|"!="|"<="|">="|"+"|"-"|"||"|"|"|"?"|"not"
      |"<<"|">>"|"&&"|"&"|"~"|"*"|"=="|"!"|"/"|"%"|"^" => true,
    _ => false
  }
}

fn is_modify_operator(kind: &str) -> bool {
  match kind {
    "++"|"--"|"="|"+="|"*="|"-="|"^="|"/="|"%="
      |">>="|"|="|"&=" => true,
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

fn is_primitive_type(kind: &str) -> bool {
  match kind {
    "primitive_type"|"sized_type_specifier" => true,
    _ => false
  }
}
