use ast_grep_core::{matcher::KindMatcher, tree_sitter::LanguageExt};
use super::cpp_lang::CppLang;

pub fn remove_global_variable(var_name: &str, content: &str) -> String {
  let root = CppLang.ast_grep(content);

  for node in root.root().find_all(KindMatcher::new("declaration", CppLang)) {
    if let Some(id_node) = node.find(KindMatcher::new("identifier", CppLang)) {
      if &content[id_node.range().start..id_node.range().end] == var_name {
        let range = node.range();
        let remove_end = if content.as_bytes().get(range.end) == Some(&b'\n') {
          range.end + 1
        } else {
          range.end
        };
        let mut result = content.to_string();
        result.replace_range(range.start..remove_end, "");
        return result;
      }
    }
  }

  content.to_string()
}
