use cpp_code_analyzer::parser;
use ravioli_core::{ast::{AST, Kind}, checker, visualize};
use ravioli_core::visualize::{to_ascii, Connection, ConnectionType, Entity, GraphData, NodeId};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn node_id_preserves_a_qualified_declaration_identity() {
  let id = NodeId::new("sample.cpp:300..336:function");

  assert_eq!(id.as_str(), "sample.cpp:300..336:function");
}

#[test]
fn ast_graph_uses_distinct_declaration_ids_for_equal_names() {
  let ast = vec![AST {
    name: "sample.cpp".to_string(),
    kind: Kind::File { content: String::new() },
    children: vec![
      AST {
        name: "helper".to_string(),
        kind: Kind::Function(ravioli_core::ast::Function {
          is_virtual: false,
          visibility: "public".to_string(),
          in_external_namespace: None,
        }),
        range: 10..20,
        ..AST::default()
      },
      AST {
        name: "helper".to_string(),
        kind: Kind::Function(ravioli_core::ast::Function {
          is_virtual: false,
          visibility: "public".to_string(),
          in_external_namespace: None,
        }),
        range: 30..40,
        ..AST::default()
      },
    ],
    ..AST::default()
  }];

  let graph = visualize::ast_to_graph(ast, "");

  assert_eq!(graph.nodes.len(), 2);
  assert!(graph.nodes.contains_key(&NodeId::new("sample.cpp::helper:function@10..20")));
  assert!(graph.nodes.contains_key(&NodeId::new("sample.cpp::helper:function@30..40")));
}

#[test]
fn graph_data_distinguishes_equal_display_names_with_qualified_ids() {
  let first = NodeId::new("first.cpp:10..20:function");
  let second = NodeId::new("second.cpp:10..20:function");
  let graph = GraphData {
    nodes: BTreeMap::from([
      (first.clone(), Entity { kind: "F".to_string(), name: "helper".to_string(), problematic: vec![] }),
      (second.clone(), Entity { kind: "F".to_string(), name: "helper".to_string(), problematic: vec![] }),
    ]),
    connections: vec![Connection {
      kind: ConnectionType::Usage,
      from: first.clone(),
      to: second.clone(),
      problematic: vec![],
    }],
  };

  assert_eq!(graph.nodes.len(), 2);
  assert_eq!(graph.connections[0].from, first);
  assert_eq!(graph.connections[0].to, second);
}

#[test]
fn renders_graph_as_terminal_text() {
  let code = r#"
class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  void foo() {}
};
"#;

  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let output = to_ascii(ast, code);

  assert!(output.contains("(C) Derived"));
  assert!(output.contains("(A) AbstractInterface"));
  assert!(output.contains('→'));
}

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
      (NodeId::new("AbstractInterface"), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("Derived"), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractInterface"),
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
      (NodeId::new("AbstractInterface"), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("Derived"), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("my_global"), Entity {
        kind: "V".to_string(),
        name: "my_global".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("int"), Entity {
        kind: "Ref".to_string(),
        name: "int".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractInterface"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: NodeId::new("Derived"),
        to: NodeId::new("int"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: NodeId::new("Derived"),
        to: NodeId::new("int"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: NodeId::new("Derived"),
        to: NodeId::new("my_global"),
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
      (NodeId::new("AbstractInterface"), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("Derived"), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("my_global"), Entity {
        kind: "V".to_string(),
        name: "my_global".to_string(),
        problematic: vec!["It's not allowed to create global variables ('my_global'). Global variables create invisible coupling.".to_string()],
      }),
      (NodeId::new("int"), Entity {
        kind: "Ref".to_string(),
        name: "int".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractInterface"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: NodeId::new("Derived"),
        to: NodeId::new("int"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: NodeId::new("Derived"),
        to: NodeId::new("int"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: NodeId::new("Derived"),
        to: NodeId::new("my_global"),
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
      (NodeId::new("AbstractInterface"), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("Derived"), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("AbstractHandler"), Entity {
        kind: "Ref".to_string(),
        name: "AbstractHandler".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Inheritance,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractInterface"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Composition,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractHandler"),
        problematic: vec![],
      },
    ],
  });
}

#[test]
fn show_call_dependencies_on_functions() {
  let code = r#"
static int helper_function() {
  return 42;
}

void public_function() {
  helper_function();
}

class AbstractInterface {
public:
  virtual ~AbstractInterface() = default;
  virtual void foo() = 0;
};

class Derived: public AbstractInterface {
  Derived() {}
  int foo(int param) {
    return param + helper_function();
  }
  int bar(int p1) {
    public_function();
    return p1;
  }
};

"#;
  let g = parse_to_graph(code);
  assert_eq!(g, GraphData {
    nodes: BTreeMap::from([
      (NodeId::new("AbstractInterface"), Entity {
        kind: "A".to_string(),
        name: "AbstractInterface".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("Derived"), Entity {
        kind: "C".to_string(),
        name: "Derived".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("public_function"), Entity {
        kind: "F".to_string(),
        name: "public_function".to_string(),
        problematic: vec![],
      }),
      (NodeId::new("helper_function"), Entity {
        kind: "F".to_string(),
        name: "helper_function".to_string(),
        problematic: vec![],
      }),
    ]),
    connections: vec![
      Connection {
        kind: ConnectionType::Usage,
        from: NodeId::new("public_function"),
        to: NodeId::new("helper_function"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Inheritance,
        from: NodeId::new("Derived"),
        to: NodeId::new("AbstractInterface"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: NodeId::new("Derived"),
        to: NodeId::new("helper_function"),
        problematic: vec![],
      },
      Connection {
        kind: ConnectionType::Usage,
        from: NodeId::new("Derived"),
        to: NodeId::new("public_function"),
        problematic: vec![],
      },
    ],
  });
}

fn parse_to_graph(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let ast = checker::filter_references_in_scope(ast);
  normalize_graph_ids(visualize::ast_to_graph(ast, code))
}

fn parse_to_graph_with_errors(code: &str) -> GraphData {
  let ast = vec![parser::parse_cpp_chunc("sample.cpp", code)];
  let ast = checker::add_lint_errors(ast);
  let ast = checker::filter_references_in_scope(ast);
  normalize_graph_ids(visualize::ast_to_graph(ast, code))
}

fn normalize_graph_ids(graph: GraphData) -> GraphData {
  let ids_to_names: BTreeMap<_, _> = graph.nodes.iter()
    .map(|(id, entity)| (id.clone(), NodeId::new(&entity.name)))
    .collect();
  let nodes = graph.nodes.into_iter()
    .map(|(id, entity)| (ids_to_names[&id].clone(), entity))
    .collect();
  let connections = graph.connections.into_iter().map(|connection| Connection {
    from: ids_to_names[&connection.from].clone(),
    to: ids_to_names[&connection.to].clone(),
    ..connection
  }).collect();

  GraphData { nodes, connections }
}
