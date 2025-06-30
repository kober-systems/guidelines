pub struct AST {
  pub name: String,
  pub kind: Kind,
  pub children: Vec<AST>,
  pub range: core::ops::Range<usize>,
}

pub enum Kind {
  File{ content: String },
  Class(Class),
  Function(Function),
  Variable(Variable),
  Unhandled(String),
  LintError(String),
}

pub struct Class {
  pub derived_from: Vec<String>,
  pub is_abstract: bool,
}

pub struct Variable {
  pub is_const: bool,
  pub visibility: String,
}

pub struct Function {
  pub is_virtual: bool,
  pub visibility: String,
}

pub struct LintError {
  pub message: String,
  pub range: core::ops::Range<usize>,
}
