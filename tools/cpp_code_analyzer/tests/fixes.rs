use std::collections::HashMap;
use core::ops::Range;
use cpp_code_analyzer::{ast::{LintError, LintErrorTypes}, fix::{apply_fixes, Fix, FixInstruction}};
use pretty_assertions::assert_eq;

#[test]
fn apply_change_derive_class() {
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("MyClass.h".to_string(), NOT_DERIVED_CLASS.to_string());

  let sources = apply_fixes(vec![Fix {
    instruction: FixInstruction::CreateAbstractClass("MyClass".to_string()),
    main_lint_err: LintError {
      kind: LintErrorTypes::DeriveFromAbstractInterface("MyClass".to_string()),
      range: Range { start: 0, end: 30 },
      file_path: "MyClass.h".to_string(),
    },
    affected_lint_errors: vec![],
  }], sources);

  assert_eq!(sources, HashMap::from([
    ("MyClass.h".to_string(), DERIVED_CLASS.to_string()),
    ("AbstractMyClass.h".to_string(), ABSTRACT_INTERFACE.to_string()),
  ]));
}

const NOT_DERIVED_CLASS: &str = r"
class MyClass {
public:
  MyClass();

  void foo();
};
";

const DERIVED_CLASS: &str = r"
class MyClass: public AbstractMyClass {
public:
  MyClass();

  void foo();
};
";

const ABSTRACT_INTERFACE: &str = r"
class AbstractMyClass {
public:
  virtual ~AbstractMyClass() = default;

  virtual void foo() = 0;
}
";
