use ast_grep_core::{
  language::Language,
  matcher::{PatternBuilder, PatternError},
  tree_sitter::{LanguageExt, StrDoc, TSLanguage},
  Pattern,
};

#[derive(Clone)]
pub(super) struct CppLang;

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
