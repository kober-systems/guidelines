use std::collections::HashMap;
use core::ops::Range;
use cpp_code_analyzer::{ast::{LintError, LintErrorTypes}, fix::{apply_fixes, Fix, FixInstruction}};
use pretty_assertions::assert_eq;

fn make_fix(class_name: &str, file_path: &str) -> Fix {
  Fix {
    instruction: FixInstruction::CreateAbstractClass(class_name.to_string()),
    main_lint_err: LintError {
      kind: LintErrorTypes::DeriveFromAbstractInterface(class_name.to_string()),
      range: Range { start: 0, end: 30 },
      file_path: file_path.to_string(),
    },
    affected_lint_errors: vec![],
  }
}

#[test]
fn apply_change() {
  // Minimal class: only a constructor, no other public methods.
  // Interface contains just the virtual destructor.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("Minimal.h".to_string(), MINIMAL_CLASS.to_string());

  let sources = apply_fixes(vec![make_fix("Minimal", "Minimal.h")], sources);

  assert_eq!(sources, HashMap::from([
    ("Minimal.h".to_string(), MINIMAL_DERIVED.to_string()),
    ("AbstractMinimal.h".to_string(), MINIMAL_ABSTRACT.to_string()),
  ]));
}

#[test]
fn apply_changes_in_order() {
  // Two independent class files fixed in one call.
  // Both are created regardless of application order.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("Foo.h".to_string(), FOO_CLASS.to_string());
  sources.insert("Bar.h".to_string(), BAR_CLASS.to_string());

  let sources = apply_fixes(vec![
    make_fix("Foo", "Foo.h"),
    make_fix("Bar", "Bar.h"),
  ], sources);

  assert_eq!(sources, HashMap::from([
    ("Foo.h".to_string(), FOO_DERIVED.to_string()),
    ("AbstractFoo.h".to_string(), FOO_ABSTRACT.to_string()),
    ("Bar.h".to_string(), BAR_DERIVED.to_string()),
    ("AbstractBar.h".to_string(), BAR_ABSTRACT.to_string()),
  ]));
}

#[test]
fn apply_change_derive_class() {
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("MyClass.h".to_string(), NOT_DERIVED_CLASS.to_string());

  let sources = apply_fixes(vec![make_fix("MyClass", "MyClass.h")], sources);

  assert_eq!(sources, HashMap::from([
    ("MyClass.h".to_string(), DERIVED_CLASS.to_string()),
    ("AbstractMyClass.h".to_string(), ABSTRACT_INTERFACE.to_string()),
  ]));
}

#[test]
fn apply_change_preserves_method_parameters() {
  // Parameter signatures are reproduced verbatim in the interface.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("Sensor.h".to_string(), SENSOR_CLASS.to_string());

  let sources = apply_fixes(vec![make_fix("Sensor", "Sensor.h")], sources);

  assert_eq!(sources, HashMap::from([
    ("Sensor.h".to_string(), SENSOR_DERIVED.to_string()),
    ("AbstractSensor.h".to_string(), SENSOR_ABSTRACT.to_string()),
  ]));
}

#[test]
fn apply_change_preserves_const_qualifier() {
  // const-qualified methods keep their qualifier in the interface signature.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("Display.h".to_string(), DISPLAY_CLASS.to_string());

  let sources = apply_fixes(vec![make_fix("Display", "Display.h")], sources);

  assert_eq!(sources, HashMap::from([
    ("Display.h".to_string(), DISPLAY_DERIVED.to_string()),
    ("AbstractDisplay.h".to_string(), DISPLAY_ABSTRACT.to_string()),
  ]));
}

#[test]
fn apply_change_skips_private_methods() {
  // Private methods stay in the class but are excluded from the interface.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("Worker.h".to_string(), WORKER_CLASS.to_string());

  let sources = apply_fixes(vec![make_fix("Worker", "Worker.h")], sources);

  assert_eq!(sources, HashMap::from([
    ("Worker.h".to_string(), WORKER_DERIVED.to_string()),
    ("AbstractWorker.h".to_string(), WORKER_ABSTRACT.to_string()),
  ]));
}

fn make_remove_fix(var_name: &str, file_path: &str, range_end: usize) -> Fix {
  Fix {
    instruction: FixInstruction::RemoveGlobalVariable(var_name.to_string()),
    main_lint_err: LintError {
      kind: LintErrorTypes::GlobalVariablesDeclaration(var_name.to_string()),
      range: Range { start: 0, end: range_end },
      file_path: file_path.to_string(),
    },
    affected_lint_errors: vec![],
  }
}

#[test]
fn remove_unused_global_variable() {
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("a.cpp".to_string(), UNUSED_GLOBAL.to_string());

  let sources = apply_fixes(vec![make_remove_fix("unused", "a.cpp", 11)], sources);

  assert_eq!(sources, HashMap::from([
    ("a.cpp".to_string(), UNUSED_GLOBAL_REMOVED.to_string()),
  ]));
}

#[test]
fn remove_write_only_global_variable() {
  // Variable is only written to, never read — should still be removed.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("b.cpp".to_string(), WRITE_ONLY_GLOBAL.to_string());

  let sources = apply_fixes(vec![make_remove_fix("flag", "b.cpp", 9)], sources);

  assert_eq!(sources, HashMap::from([
    ("b.cpp".to_string(), WRITE_ONLY_GLOBAL_REMOVED.to_string()),
  ]));
}

#[test]
fn remove_global_variable_keeps_others() {
  // Only the targeted variable is removed; other globals remain untouched.
  let mut sources: HashMap<String, String> = HashMap::default();
  sources.insert("c.cpp".to_string(), TWO_GLOBALS.to_string());

  let sources = apply_fixes(vec![make_remove_fix("to_remove", "c.cpp", 14)], sources);

  assert_eq!(sources, HashMap::from([
    ("c.cpp".to_string(), TWO_GLOBALS_ONE_REMOVED.to_string()),
  ]));
}

// ── fixtures ─────────────────────────────────────────────────────────────────

const MINIMAL_CLASS: &str = r"
class Minimal {
public:
  Minimal();
};
";

const MINIMAL_DERIVED: &str = r#"#include "AbstractMinimal.h"

class Minimal: public AbstractMinimal {
public:
  Minimal();
};
"#;

const MINIMAL_ABSTRACT: &str = r"
class AbstractMinimal {
public:
  virtual ~AbstractMinimal() = default;

}
";

const FOO_CLASS: &str = r"
class Foo {
public:
  Foo();
  void act();
};
";

const FOO_DERIVED: &str = r#"#include "AbstractFoo.h"

class Foo: public AbstractFoo {
public:
  Foo();
  void act();
};
"#;

const FOO_ABSTRACT: &str = r"
class AbstractFoo {
public:
  virtual ~AbstractFoo() = default;

  virtual void act() = 0;
}
";

const BAR_CLASS: &str = r"
class Bar {
public:
  Bar();
  int compute();
};
";

const BAR_DERIVED: &str = r#"#include "AbstractBar.h"

class Bar: public AbstractBar {
public:
  Bar();
  int compute();
};
"#;

const BAR_ABSTRACT: &str = r"
class AbstractBar {
public:
  virtual ~AbstractBar() = default;

  virtual int compute() = 0;
}
";

const NOT_DERIVED_CLASS: &str = r"
class MyClass {
public:
  MyClass();

  void foo();
};
";

const DERIVED_CLASS: &str = r#"#include "AbstractMyClass.h"

class MyClass: public AbstractMyClass {
public:
  MyClass();

  void foo();
};
"#;

const ABSTRACT_INTERFACE: &str = r"
class AbstractMyClass {
public:
  virtual ~AbstractMyClass() = default;

  virtual void foo() = 0;
}
";

const SENSOR_CLASS: &str = r"
class Sensor {
public:
  Sensor();
  int read(int channel);
  void configure(bool active, int gain);
};
";

const SENSOR_DERIVED: &str = r#"#include "AbstractSensor.h"

class Sensor: public AbstractSensor {
public:
  Sensor();
  int read(int channel);
  void configure(bool active, int gain);
};
"#;

const SENSOR_ABSTRACT: &str = r"
class AbstractSensor {
public:
  virtual ~AbstractSensor() = default;

  virtual int read(int channel) = 0;
  virtual void configure(bool active, int gain) = 0;
}
";

const DISPLAY_CLASS: &str = r"
class Display {
public:
  Display();
  bool isReady() const;
  void show(int value);
};
";

const DISPLAY_DERIVED: &str = r#"#include "AbstractDisplay.h"

class Display: public AbstractDisplay {
public:
  Display();
  bool isReady() const;
  void show(int value);
};
"#;

const DISPLAY_ABSTRACT: &str = r"
class AbstractDisplay {
public:
  virtual ~AbstractDisplay() = default;

  virtual bool isReady() const = 0;
  virtual void show(int value) = 0;
}
";

const WORKER_CLASS: &str = r"
class Worker {
public:
  Worker();
  void run();
private:
  void setup();
  int compute(int x);
};
";

const WORKER_DERIVED: &str = r#"#include "AbstractWorker.h"

class Worker: public AbstractWorker {
public:
  Worker();
  void run();
private:
  void setup();
  int compute(int x);
};
"#;

const WORKER_ABSTRACT: &str = r"
class AbstractWorker {
public:
  virtual ~AbstractWorker() = default;

  virtual void run() = 0;
}
";

const UNUSED_GLOBAL: &str = "int unused;\n\nvoid foo() {}\n";
const UNUSED_GLOBAL_REMOVED: &str = "\nvoid foo() {}\n";

const WRITE_ONLY_GLOBAL: &str = "int flag;\n\nvoid set_flag() {\n  flag = 1;\n}\n";
const WRITE_ONLY_GLOBAL_REMOVED: &str = "\nvoid set_flag() {\n  flag = 1;\n}\n";

const TWO_GLOBALS: &str = "int to_remove;\nint to_keep;\n\nvoid use_kept() {\n  to_keep = 1;\n}\n";
const TWO_GLOBALS_ONE_REMOVED: &str = "int to_keep;\n\nvoid use_kept() {\n  to_keep = 1;\n}\n";
