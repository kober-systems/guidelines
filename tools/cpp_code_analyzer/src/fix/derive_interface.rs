use ast_grep_core::{matcher::KindMatcher, tree_sitter::LanguageExt};
use super::cpp_lang::CppLang;
use crate::ast::AST;

pub fn modify_to_derive_from_interface(class: &AST, content: &str) -> String {
  let chunk = &content[class.range.start..class.range.end];
  let root = CppLang.ast_grep(chunk);

  let mut edits: Vec<(usize, String)> = vec![];

  let class_spec = root.root().find(KindMatcher::new("class_specifier", CppLang));
  if let Some(cs) = class_spec {
    if let Some(name_node) = cs.find(KindMatcher::new("type_identifier", CppLang)) {
      let insert_pos = class.range.start + name_node.range().end;
      edits.push((insert_pos, format!(": public Abstract{}", class.name)));
    }
  }

  edits.push((0, format!("#include \"Abstract{}.h\"\n", class.name)));

  let mut result = content.to_string();
  edits.sort_by_key(|e| e.0);
  for (pos, text) in edits.into_iter().rev() {
    result.insert_str(pos, &text);
  }

  result
}
