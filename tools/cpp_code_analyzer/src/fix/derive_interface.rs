use ast_grep_core::{
  language::Language,
  matcher::{KindMatcher, PatternBuilder, PatternError},
  tree_sitter::{LanguageExt, StrDoc, TSLanguage},
  Pattern,
};
use crate::ast::AST;

#[derive(Clone)]
struct CppLang;

impl Language for CppLang {
  fn kind_to_id(&self, kind: &str) -> u16 {
    let ts_lang: TSLanguage = tree_sitter_cpp::LANGUAGE.into();
    ts_lang.id_for_node_kind(kind, true)
  }

  fn field_to_id(&self, field: &str) -> Option<u16> {
    self.get_ts_language()
      .field_id_for_name(field)
      .map(|f| f.get())
  }

  fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
    builder.build(|src| StrDoc::try_new(src, self.clone()))
  }
}

impl LanguageExt for CppLang {
  fn get_ts_language(&self) -> TSLanguage {
    tree_sitter_cpp::LANGUAGE.into()
  }
}

pub fn modify_to_derive_from_interface(class: &AST, content: &str) -> String {
  let chunk = &content[class.range.start..class.range.end];
  let root = CppLang.ast_grep(chunk);

  let mut edits: Vec<(usize, String)> = vec![];

  // Find class name end position to insert ": public AbstractXxx"
  let class_spec = root.root().find(KindMatcher::new("class_specifier", CppLang));
  if let Some(cs) = class_spec {
    if let Some(name_node) = cs.find(KindMatcher::new("type_identifier", CppLang)) {
      let insert_pos = class.range.start + name_node.range().end;
      edits.push((insert_pos, format!(": public Abstract{}", class.name)));
    }
  }

  // Insert include at start of file
  edits.push((0, format!("#include \"Abstract{}.h\"\n", class.name)));

  // Apply edits from end to start so earlier byte offsets stay valid
  let mut result = content.to_string();
  edits.sort_by_key(|e| e.0);
  for (pos, text) in edits.into_iter().rev() {
    result.insert_str(pos, &text);
  }

  result
}
