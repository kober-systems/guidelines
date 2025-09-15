use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct AST {
  pub name: String,
  pub kind: Kind,
  pub children: Vec<AST>,
  pub dependencies: Vec<AST>,
  pub range: core::ops::Range<usize>,
  pub instructions: Vec<LintInstruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
  File{ content: String },
  Class(Class),
  Function(Function),
  Variable(Variable),
  Reference(Reference),
  Type,
  Unhandled(String),
  LintError(LintErrorTypes),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Class {
  pub is_abstract: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
  pub is_const: bool,
  pub visibility: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
  pub is_virtual: bool,
  pub visibility: String,
  pub in_external_namespace: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum  Reference {
  TypeRead,
  Read,
  Write,
  Call,
  Depend,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LintError {
  pub kind: LintErrorTypes,
  pub range: core::ops::Range<usize>,
  pub file_path: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LintErrorTypes {
  InterfaceOnlyPublicMethods(String, String),
  InterfaceShouldNotDefineAttrs(String, String),
  DerivedClassesAllAttrsPrivate(String, String),
  GlobalVariablesUsage(String),
  GlobalVariablesDeclaration(String),
  DeriveFromAbstractInterface(String),
  AvoidInitMethods(String),
  ParserUnhandled(String),
  LintInstructionNotParseble(String),
  // c++ specific errors without broader meaning
  // for other languages
  CppAbstractClassMissingDefaultDestructor(String),
  CppAbstractClassMethodNotVirtual(String, String),
  CppAbstractClassMethodMissingVirtualEnding(String, String),
  CppDerivedClassMethodIsVirtual(String, String),
  CppDerivedClassMethodHasVirtualEnding(String, String),
  CppDerivesAlwaysPublic(String),
  CppDerivesAlwaysFromAbstractInterfaces(String),
}

impl Display for LintErrorTypes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LintErrorTypes::*;

    match &self {
      InterfaceOnlyPublicMethods(class_name, visibility) => {
        write!(f, "Abstract class `{class_name}` should ONLY define 'public' methods (not allowed {visibility})")
      },
      InterfaceShouldNotDefineAttrs(class_name, attr_name) => {
        write!(f, "Abstract class `{class_name}` must not have attributes ('{attr_name}')")
      },
      DerivedClassesAllAttrsPrivate(class_name, attr_name) => {
        write!(f, "Derived class '{class_name}' must not have non private attributes ('{attr_name}')")
      },
      GlobalVariablesDeclaration(name) => {
        write!(f, "It's not allowed to create global variables ('{name}'). Global variables create invisible coupling.")
      },
      GlobalVariablesUsage(name) => {
        write!(f, "It's not allowed to use global variables ('{name}'). Global variables create invisible coupling.")
      },
      DeriveFromAbstractInterface(name) => {
        write!(f, "Class '{name}' should be derived from abstract interface")
      },
      AvoidInitMethods(name) => {
        write!(f, "Class '{name}' should not provide an init function. Initialisation should be done in constructor.")
      },
      CppAbstractClassMissingDefaultDestructor(class_name) => {
        write!(f, "Abstract class '{class_name}' should provide a default destructor.")
      },
      CppAbstractClassMethodNotVirtual(class_name, function_code) => {
        write!(f, "method '{function_code}' in abstract class '{class_name}' must be virtual")
      },
      CppAbstractClassMethodMissingVirtualEnding(class_name, function_code) => {
        write!(f, "Abstract class '{class_name}': missing `= 0;` for method '{function_code}'")
      },
      CppDerivedClassMethodIsVirtual(class_name, function_name) => {
        write!(f, "Derived class `{class_name}` must not define virtual functions ('{function_name}')")
      },
      CppDerivedClassMethodHasVirtualEnding(class_name, function_name) => {
        write!(f, "Derived class '{class_name}' method '{function_name}' should not be pure virtual")
      },
      CppDerivesAlwaysPublic(class_name) => {
        write!(f, "Class '{class_name}': Derives must always be public")
      },
      CppDerivesAlwaysFromAbstractInterfaces(class_name) => {
        write!(f, "Class '{class_name}': Derives must always be from abstract interfaces")
      },
      LintInstructionNotParseble(comment) => {
        write!(f, "could not parse lint instruction in comment: {comment}")
      },
      ParserUnhandled(message) => {
        write!(f, "{message}")
      },
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct LintInstruction {
  pub ident: String,
  pub reason: String,
}

impl AST {
  pub fn get_file_content(&self) -> Result<String, String> {
    match &self.kind {
      Kind::File { content } => Ok(content.clone()),
      _ => Err(format!("{:?} is not a file", self)),
    }
  }

  pub fn set_file_content(self, content: String) -> Self {
    Self {
      name: self.name,
      kind: Kind::File { content },
      dependencies: self.dependencies,
      children: self.children,
      instructions: self.instructions,
      range: self.range,
    }
  }
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
