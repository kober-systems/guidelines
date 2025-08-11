use cpp_code_analyzer::{checker, parser, visualize};
use cpp_code_analyzer::visualize::{Connection, ConnectionType, Entity, GraphData};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn basic_derives() {
  let code = r#"
class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  void foo() {}
};
"#;
  let g = parse_to_graph(code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      ("AbstractInterface".to_string(), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      ("Derived".to_string(), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: "Derived".to_string(),
        to: "AbstractInterface".to_string(),
        problematic: vec![],
      },
    ],
  });
}

#[test]
fn show_dependencies_on_global_variables() {
  let code = r#"
int my_global = 0;

class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  void foo(int param) {
    int var = param;
    my_global = 42 + param;
  }
};
"#;
  let g = parse_to_graph(code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      ("AbstractInterface".to_string(), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      ("Derived".to_string(), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      ("my_global".to_string(), Entity {
        kind: "V".to_string(),
        name: "my_global".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: "Derived".to_string(),
        to: "AbstractInterface".to_string(),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: "Derived".to_string(),
        to: "my_global".to_string(),
        problematic: vec![],
      },
    ],
  });
}

#[test]
fn show_dependencies_on_global_variables_in_functions() {
  let code = r#"
int my_global = 0;

class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  void foo(int param);
};

void Derived::foo(int param) {
  int var = param;
  my_global = 42 + param;
}
"#;
  let g = parse_to_graph_with_errors(code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      ("AbstractInterface".to_string(), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      ("Derived".to_string(), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      ("my_global".to_string(), Entity {
        kind: "V".to_string(),
        name: "my_global".to_string(),
        problematic: vec!["It's not allowed to create global variables ('my_global'). Global variables create invisible coupling.".to_string()],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: "Derived".to_string(),
        to: "AbstractInterface".to_string(),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: "Derived".to_string(),
        to: "my_global".to_string(),
        problematic: vec!["It's not allowed to use global variables ('my_global'). Global variables create invisible coupling.".to_string()],
      },
    ],
  });
}

#[test]
fn show_structure_dependencies_on_composition() {
  let code = r#"
class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  void foo(int param);

private:
  AbstractHandler *handler = nullptr;
};

"#;
  let g = parse_to_graph(code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      ("AbstractInterface".to_string(), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      ("Derived".to_string(), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      ("AbstractHandler".to_string(), Entity {
        kind: "Ref".to_string(),
        name: "AbstractHandler".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: "Derived".to_string(),
        to: "AbstractInterface".to_string(),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: "Derived".to_string(),
        to: "AbstractHandler".to_string(),
        problematic: vec![],
      },
    ],
  });
}

fn parse_to_graph(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let ast = checker::filter_references_in_scope(ast);
  visualize::ast_to_graph(ast, code)
}

fn parse_to_graph_with_errors(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let ast = checker::add_lint_errors(ast);
  let ast = checker::filter_references_in_scope(ast);
  visualize::ast_to_graph(ast, code)
}
