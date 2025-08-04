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

fn parse_to_graph(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let ast = checker::add_lint_erros(ast);
  visualize::ast_to_graph(ast, code)
}
