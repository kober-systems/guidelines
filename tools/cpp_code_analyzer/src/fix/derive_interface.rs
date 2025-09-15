use tree_sitter::{Node, Parser};
use crate::ast::AST;

pub fn modify_to_derive_from_interface(class: &AST, content: &str) -> String {
  let mut parser = Parser::new();
  parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading Cpp grammar");

  let offset = class.range.start;
  let tree = parser.parse(&content[offset..class.range.end], None).unwrap();
  let node = tree.root_node().child(0).unwrap();
  if node.kind() != "class_specifier" {
    return format!("Something is wrong {}", node.kind());
  }

  let pos = find_derive_position(&node);

  let mut content = content.to_string();
  content.insert_str(pos + offset, &format!(": public Abstract{}", class.name));

  let pos = find_include_position(&tree.root_node());
  content.insert_str(pos, &(format!(r#"#include "Abstract{}.h""#, class.name) + "\n"));
  content
}

fn find_derive_position(node: &Node) -> usize {
  let mut pos = 0;

  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();

    match child.kind() {
      "type_identifier" => {
        pos = child.byte_range().end;
      }
      "body" => {
        break;
      }
      _ => ()
    }
  }

  pos
}

fn find_include_position(node: &Node) -> usize {
  for idx in 0..node.child_count() {
    let child = node.child(idx).unwrap();

    match child.kind() {
      "preproc_ifdef"|"preproc_def" => {
        // header guard
        return find_include_position(&child);
      }
      _ => {
        return child.byte_range().start;
      }
    }
  }
  return 0;
}
