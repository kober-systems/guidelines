#[derive(Debug)]
pub struct AST {
  pub name: String,
  pub kind: Kind,
  pub children: Vec<AST>,
  pub dependencies: Vec<AST>,
  pub range: core::ops::Range<usize>,
  pub instructions: Vec<LintInstruction>,
}

#[derive(Debug)]
pub enum Kind {
  File{ content: String },
  Class(Class),
  Function(Function),
  Variable(Variable),
  Reference,
  Type,
  Unhandled(String),
  LintError(String),
}

#[derive(Debug)]
pub struct Class {
  pub is_abstract: bool,
}

#[derive(Debug)]
pub struct Variable {
  pub is_const: bool,
  pub visibility: String,
}

#[derive(Debug)]
pub struct Function {
  pub is_virtual: bool,
  pub visibility: String,
}

pub struct LintError {
  pub message: String,
  pub range: core::ops::Range<usize>,
}

#[derive(Debug)]
pub struct LintInstruction {
  pub ident: String,
  pub reason: String,
}

impl Default for AST {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      kind: Kind::Unhandled("not existant".to_string()),
      children: vec![],
      dependencies: vec![],
      range: core::ops::Range::default(),
      instructions: vec![],
    }
  }
}
