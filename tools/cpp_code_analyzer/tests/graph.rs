use cpp_code_analyzer::{parser, visualize};
use cpp_code_analyzer::visualize::{Connection, ConnectionType, Entity, GraphData};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn basic_derives() {
  let code = r#"
class AbstractInterface {
public:
  virtual ~AbstractHandle() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  void foo() {}
};
"#;
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let g = visualize::ast_to_graph(ast, code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      ("AbstractInterface".to_string(), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: None,
      }),
      ("Derived".to_string(), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: None,
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: "Derived".to_string(),
        to: "AbstractInterface".to_string(),
        problematic: None,
      },
    ],
  });
}
