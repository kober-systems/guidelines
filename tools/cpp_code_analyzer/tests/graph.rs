use cpp_code_analyzer::{parser, visualize};
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
  void foo() { my_global = 42; }
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

fn parse_to_graph(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  visualize::ast_to_graph(ast, code)
}
