use std::collections::{BTreeMap, HashSet};

use crate::ast::{Kind, Reference, AST};

use ascii_dag::{Direction, Graph, LayoutConfig, RenderOptions};
use layout::adt::dag::NodeHandle;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::color::Color;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;
//use layout::topo::placer::Placer;

/// Identifies a graph declaration independently of its display name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(String);

impl NodeId {
  pub fn new(value: impl Into<String>) -> Self {
    Self(value.into())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Display for NodeId {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(self.as_str())
  }
}

#[derive(Debug, PartialEq)]
pub struct GraphData {
  pub nodes: BTreeMap<NodeId, Entity>,
  pub connections: Vec<Connection>,
}

#[derive(Debug, PartialEq)]
pub struct Connection {
  pub kind: ConnectionType, // Dependency, Inheritance, Composition, Usage,
  pub from: NodeId,
  pub to: NodeId,
  pub problematic: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionType {
  Usage,
  Inheritance,
  Composition,
}

#[derive(Debug, PartialEq)]
pub struct Entity {
  pub kind: String, // Extern, Interface, Class, Variable, Function, Type (Struct|Enum)
  pub name: String,
  pub problematic: Vec<String>,
}

pub fn visualize(ast: Vec<AST>, code: &str) -> String {
  let ast = crate::checker::filter_references_in_scope(ast);
  let ast = crate::checker::add_lint_errors(ast);

  let mut vg = VisualGraph::new(Orientation::LeftToRight);

  let g = ast_to_graph(ast, code);
  let g = remove_visual_noise(g);
  visualize_graph_data(g, &mut vg);

  let mut svg = SVGWriter::new();
  vg.do_it(false, false, true, &mut svg);

  svg.finalize()
}

pub fn to_ascii(ast: Vec<AST>, code: &str) -> String {
  let ast = crate::checker::filter_references_in_scope(ast);
  let ast = crate::checker::add_lint_errors(ast);
  let g = remove_visual_noise(ast_to_graph(ast, code));

  if g.nodes.is_empty() {
    return String::new();
  }

  let labels: Vec<_> = g.nodes.values()
      .map(|node| format!("({}) {}", node.kind, node.name))
      .collect();
  let node_ids: BTreeMap<_, _> = g.nodes.keys()
      .enumerate()
      .map(|(index, id)| (id.clone(), index + 1))
      .collect();

  let mut graph = Graph::new();
  for (id, label) in labels.iter().enumerate() {
    graph.add_node(id + 1, label.as_str());
  }
  for connection in &g.connections {
    graph.add_edge(node_ids[&connection.from], node_ids[&connection.to], None);
  }

  let mut config = LayoutConfig::standard();
  config.direction = Direction::LeftRight;
  graph.compute_layout_with_config(&config).render_string(&RenderOptions::plain())
}

pub fn to_graphviz(ast: Vec<AST>, code: &str) -> String {
  let ast = crate::checker::filter_references_in_scope(ast);
  let ast = crate::checker::add_lint_errors(ast);
  let g = ast_to_graph(ast, code);
  let g = remove_visual_noise(g);

  let mut out = "digraph Code {\n".to_string();
  out.push_str(" concentrate=True;\n");
  out.push_str(" rankdir=BT;\n\n");

  for (key, node) in g.nodes.iter() {
    out.push_str(&key.as_str().replace(":", "_"));
    out.push_str(" [");
    out.push_str("label=\"");
    out.push_str(key.as_str());
    out.push_str("\";");
    if !node.problematic.is_empty() {
      out.push_str("color=red;");
    } else {
      out.push_str("color=black;");
    }
    match node.kind.as_str() {
      "Ref" => {
        out.push_str("style=dotted;");
      }
      "V" => {
        out.push_str("shape=ellipse;");
      }
      "F" => {
        out.push_str("shape=parallelogram;");
      }
      _ => {
        out.push_str("shape=box;");
      }
    }
    out.push_str("]\n");
  }

  for con in g.connections.iter() {
    out.push_str(&con.from.as_str().replace(":", "_"));
    out.push_str(" -> ");
    out.push_str(&con.to.as_str().replace(":", "_"));
    out.push_str(" [");
    if !con.problematic.is_empty() {
      out.push_str("color=red;");
    }
    match con.kind {
      ConnectionType::Composition => {
        out.push_str("arrowhead=diamond;");
      }
      ConnectionType::Usage => {
        out.push_str("style=dashed;");
      }
      ConnectionType::Inheritance => ()
    }
    out.push_str("]\n");
  }

  out.push_str("}");
  out
}

pub fn to_graphml(ast: Vec<AST>, code: &str) -> String {
  let ast = crate::checker::filter_references_in_scope(ast);
  let ast = crate::checker::add_lint_errors(ast);
  let g = ast_to_graph(ast, code);
  let g = remove_visual_noise(g);

  let mut out = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_string();
  out.push_str(
    r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns
     http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">\n"#);

  out.push_str(r#" <key id="label" for="node" attr.name="label" attr.type="string">\n"#);
  out.push_str(r#" <key id="kind" for="node" attr.name="kind" attr.type="string">\n"#);
  out.push_str(r#" <key id="is_problematic" for="all" attr.name="is_problematic" attr.type="boolean">\n"#);
  out.push_str(r#"   <default>false</default>\n"#);
  out.push_str(r#" </key>\n"#);

  out.push_str(r#" <graph id="G" edgedefault="directed">\n"#);

  for (key, node) in g.nodes.iter() {
    out.push_str(&format!("    <node id=\"{}\">\n", key.as_str()));
    out.push_str(&format!("      <data key=\"label\">{}</data>\n", key.as_str()));
    out.push_str(&format!("      <data key=\"kind\">{}</data>\n", &node.kind));
    out.push_str(&format!("      <data key=\"is_problematic\">{}</data>\n", !node.problematic.is_empty()));
    out.push_str("    </node>\n");
  }

  for con in g.connections.iter() {
    out.push_str(&format!("    <edge source=\"{}\" target=\"{}\">\n", con.from.as_str(), con.to.as_str()));
    out.push_str(&format!("      <data key=\"is_problematic\">{}</data>\n", !con.problematic.is_empty()));
    //match con.kind {
    //  ConnectionType::Composition => {
    //    out.push_str("arrowhead=diamond;");
    //  }
    //  ConnectionType::Usage => {
    //    out.push_str("style=dashed;");
    //  }
    //  ConnectionType::Inheritance => ()
    //}
    //out.push_str("]\n");
    out.push_str("    </edge>\n");
  }

  out.push_str("  </graph>\n</graphml>");
  out
}

fn visualize_graph_data(g: GraphData, vg: &mut VisualGraph) {
  let mut handles: BTreeMap<NodeId, NodeHandle> = BTreeMap::default();

  for (key, node) in g.nodes.iter() {
    let vis_name = format!("({}) {}", node.kind, node.name);
    let sz = Point::new(get_text_width(&vis_name), 100.);

    let handle = vg.add_node(Element::create(
      ShapeKind::new_box(&vis_name),
      get_style(!node.problematic.is_empty()),
      Orientation::LeftToRight,
      sz)
    );

    handles.insert(key.clone(), handle);
  }

  for con in g.connections.iter() {
    let from = *handles.get(&con.from).unwrap();
    let to = *handles.get(&con.to).unwrap();
    vg.add_edge(
      Arrow{
        look: get_style(!con.problematic.is_empty()),
        ..Arrow::default()
      },
      from,
      to);
  }
}

#[derive(Debug, Default)]
struct DeclarationIndex {
  by_scope: BTreeMap<Vec<String>, BTreeMap<String, Vec<NodeId>>>,
}

impl DeclarationIndex {
  fn insert(&mut self, scope: &[String], name: &str, id: NodeId) {
    self.by_scope
      .entry(scope.to_vec())
      .or_default()
      .entry(name.to_string())
      .or_default()
      .push(id);
  }

  fn lookup_in_scope(&self, scope: &[String], name: &str) -> &[NodeId] {
    self.by_scope
      .get(scope)
      .and_then(|symbols| symbols.get(name))
      .map(Vec::as_slice)
      .unwrap_or(&[])
  }

  fn resolve(&self, scope: &[String], name: &str) -> Option<NodeId> {
    let mut current_scope = scope;
    loop {
      let matches = self.lookup_in_scope(current_scope, name);
      if matches.len() == 1 {
        return Some(matches[0].clone());
      }
      if current_scope.is_empty() {
        return None;
      }
      current_scope = &current_scope[..current_scope.len() - 1];
    }
  }
}

fn build_declaration_index(ast: &[AST]) -> DeclarationIndex {
  let mut index = DeclarationIndex::default();
  for node in ast {
    index_declarations(node, &node.name, &[], &mut index);
  }
  index
}

fn index_declarations(node: &AST, file_path: &str, scope: &[String], index: &mut DeclarationIndex) {
  match &node.kind {
    Kind::File { .. } => {
      for child in &node.children {
        index_declarations(child, &node.name, scope, index);
      }
    }
    Kind::Class(_) | Kind::Function(_) | Kind::Variable(_) | Kind::Type => {
      let mut declaration_scope = scope.to_vec();
      if let Kind::Function(function) = &node.kind {
        if let Some(class_name) = &function.in_external_namespace {
          declaration_scope.push(class_name.clone());
        }
      }

      let id = declaration_id(file_path, &declaration_scope, node);
      index.insert(&declaration_scope, &node.name, id);

      let mut child_scope = declaration_scope;
      child_scope.push(node.name.clone());
      for child in &node.children {
        index_declarations(child, file_path, &child_scope, index);
      }
    }
    _ => {}
  }
}

fn declaration_id(file_path: &str, scope: &[String], node: &AST) -> NodeId {
  let qualified_name = scope.iter()
    .chain(std::iter::once(&node.name))
    .cloned()
    .collect::<Vec<_>>()
    .join("::");
  NodeId::new(format!(
    "{file_path}::{qualified_name}:{}@{}..{}",
    declaration_kind(node),
    node.range.start,
    node.range.end,
  ))
}

fn declaration_kind(node: &AST) -> &'static str {
  match node.kind {
    Kind::Class(_) => "class",
    Kind::Function(_) => "function",
    Kind::Variable(_) => "variable",
    Kind::Type => "type",
    _ => unreachable!("declaration_kind called for a non-declaration"),
  }
}

fn external_node_id(name: &str) -> NodeId {
  NodeId::new(format!("external::{name}"))
}

pub fn ast_to_graph(ast: Vec<AST>, code: &str) -> GraphData {
  let declarations = build_declaration_index(&ast);
  let mut g = GraphData { nodes: BTreeMap::default(), connections: vec![] };
  for node in ast.iter() {
    g = extract_node(node, code, g, &declarations, &node.name)
  }
  g
}

fn get_style(problematic: bool) -> StyleAttr {
  StyleAttr::new(
    if problematic {
      Color::fast("red")
    } else {
      Color::fast("black")
    },
    2,
    Some(Color::new(0xf2f2f2ff)), // gray95
    0,
    15
  )
}

fn extract_node(
  input: &AST,
  code: &str,
  base: GraphData,
  declarations: &DeclarationIndex,
  file_path: &str,
) -> GraphData {
  let mut base = base;
  match &input.kind {
    Kind::File { content } => {
      for child in input.children.iter() {
        base = extract_node(child, &content, base, declarations, &input.name)
      }
      base
    },
    Kind::Class(ref cl) => {
      let input_id = declaration_id(file_path, &[], input);
      base.nodes.insert(input_id.clone(), Entity {
        kind: if cl.is_abstract { "A".to_string() } else { "C".to_string() },
        name: input.name.clone(),
        problematic: is_problematic(input) } );
      for dependecy in input.dependencies.iter() {
        let dep_id = declarations.resolve(&[], &dependecy.name)
          .unwrap_or_else(|| external_node_id(&dependecy.name));
        if !base.nodes.contains_key(&dep_id) {
          base.nodes.insert(dep_id.clone(), Entity {
            kind: "Ref".to_string(),
            name: dependecy.name.clone(),
            problematic: is_problematic(dependecy),
          });
        }
        base.connections.push(Connection {
          kind: ConnectionType::Inheritance,
          from: input_id.clone(),
          to: dep_id,
          problematic: vec![],
        });
      }
      for child in input.children.iter() {
        base = extract_references(child, &input_id, code, base, declarations);
      }
      base
    },
    Kind::Type|Kind::Variable(_) => {
      base.nodes.insert(declaration_id(file_path, &[], input), Entity {
        kind: get_entity_type(&input).to_string(),
        name: input.name.clone(),
        problematic: is_problematic(input)} );
      base
    },
    Kind::Function(fun) => {
      match fun.in_external_namespace {
        Some(ref class_name) => {
          let class_id = declarations.resolve(&[], class_name)
            .unwrap_or_else(|| external_node_id(class_name));
          if !base.nodes.contains_key(&class_id) {
            base.nodes.insert(class_id.clone(), Entity {
              kind: "C".to_string(),
              name: input.name.clone(),
              problematic: is_problematic(input)} );
          }
          base = extract_references(input, &class_id, code, base, declarations);
        }
        None => {
          let input_id = declaration_id(file_path, &[], input);
          base.nodes.insert(input_id.clone(), Entity {
            kind: "F".to_string(),
            name: input.name.clone(),
            problematic: is_problematic(input)} );
          base = extract_references(input, &input_id, code, base, declarations);
        }
      }
      base
    },
    Kind::Unhandled(_element) => base,
    _ => todo!()
  }
}

fn extract_references(
  input: &AST,
  from: &NodeId,
  code: &str,
  base: GraphData,
  declarations: &DeclarationIndex,
) -> GraphData {
  let mut base = base;
  match &input.kind {
    Kind::Reference(r) => {
      let dep_id = declarations.resolve(&[], &input.name)
        .unwrap_or_else(|| external_node_id(&input.name));
      if !base.nodes.contains_key(&dep_id) {
        base.nodes.insert(dep_id.clone(), Entity {
          kind: "Ref".to_string(),
          name: input.name.clone(),
          problematic: is_problematic(input),
        });
      }
      base.connections.push(Connection {
        kind: match r {
          Reference::Read|Reference::Write|Reference::Call => ConnectionType::Usage,
          Reference::TypeRead => ConnectionType::Composition,
          r => todo!("{:?}", r),
        },
        from: from.clone(),
        to: dep_id,
        problematic: is_problematic(input),
      });
      base
    },
    Kind::Function(_) => {
      //eprintln!("children of function: {:?}", input.children);
      for child in input.children.iter() {
        base = extract_references(child, from, code, base, declarations);
      }
      base
    },
    Kind::Unhandled(_element) => base,
    Kind::Variable(_) => {
      for dependency in &input.dependencies {
        base = extract_references(dependency, from, code, base, declarations);
      }
      base
    }
    Kind::LintError(_) => base, // TODO change is_problematic
    Kind::Type => base,
    _ => todo!("{:?}", input.kind)
  }
}

/// Some nodes make the output only less readable. Keep them only when
/// they create problems in the code and need attention
pub fn remove_visual_noise(graph: GraphData) -> GraphData {
  let GraphData { nodes, connections } = graph;
  let mut nodes_to_be_removed: HashSet<_> = nodes.iter().filter_map(|(name, node)| match node.kind.as_str() {
    "T"|"V"|"F"|"Ref" => {
      if node.problematic.is_empty() {
        Some(name.clone())
      } else {
        None
      }
    }
    _ => None,
  }).collect();

  let connections = connections.into_iter().filter(|con| {
    if con.problematic.len() > 0 {
      nodes_to_be_removed.remove(&con.to);
      nodes_to_be_removed.remove(&con.from);
      true
    } else {
      !nodes_to_be_removed.contains(&con.to)
    }
  }).collect();

  let nodes = nodes.into_iter().filter(|(name, _node)| !nodes_to_be_removed.contains(name) ).collect();

  GraphData { nodes, connections }
}

fn get_entity_type(input: &AST) -> &str {
  match &input.kind {
    Kind::Class(ref cl) => if cl.is_abstract {
      "A"
    } else {
      "C"
    }
    Kind::Function(_fun) => "F",
    Kind::Type => "T",
    Kind::Variable(_var) => "V",
    Kind::Reference(_reftype) => "Ref",
    //Kind::Unhandled(_element) => (),
    _ => todo!()
  }
}

fn get_text_width(text: &str) -> f64 {
  (text.len() as f64 * 8.) + 20.
}

fn is_problematic(node: &AST) -> Vec<String> {
  node.children.iter().filter_map(|n| match &n.kind {
    Kind::LintError(err) => Some(format!("{err}")),
    //Kind::Unhandled(_) => true,
    _ => None,
  }).collect()
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use super::*;

  #[test]
  fn declaration_index_uses_source_location_to_distinguish_equal_names() {
    let function = AST {
      name: "helper".to_string(),
      kind: Kind::Function(crate::ast::Function {
        is_virtual: false,
        visibility: "public".to_string(),
        in_external_namespace: None,
      }),
      range: 10..20,
      ..AST::default()
    };
    let ast = vec![AST {
      name: "sample.cpp".to_string(),
      kind: Kind::File { content: String::new() },
      children: vec![
        function,
        AST {
          name: "helper".to_string(),
          kind: Kind::Function(crate::ast::Function {
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

    let index = build_declaration_index(&ast);
    let helpers = index.lookup_in_scope(&[], "helper");

    assert_eq!(helpers.len(), 2);
    assert_ne!(helpers[0], helpers[1]);
    assert_eq!(helpers[0].as_str(), "sample.cpp::helper:function@10..20");
    assert_eq!(helpers[1].as_str(), "sample.cpp::helper:function@30..40");
  }

  #[test]
  fn non_problematic_variables_should_be_filtered() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyGlobalVar"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        }),
        (NodeId::new("MyGlobalConstant"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyGlobalVar"),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyGlobalVar"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyGlobalVar"),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        },
      ],
    });
  }

  #[test]
  fn non_problematic_nodes_with_problematic_connections_should_stay() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyGlobalConstant"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyGlobalConstant"),
          problematic: vec!["Something is wrong".to_string()],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyGlobalConstant"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyGlobalConstant"),
          problematic: vec!["Something is wrong".to_string()],
        },
      ],
    });
  }

  #[test]
  fn remove_non_problematic_connections_to_uninteresting_nodes() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyGlobalConstant"), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyStruct"), Entity {
          kind: "T".to_string(),
          name: "MyStruct".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyExtStruct"), Entity {
          kind: "Ref".to_string(),
          name: "MyExtStruct".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyGlobalConstant"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyClass"),
          to: NodeId::new("MyExtStruct"),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: NodeId::new("MyGlobalConstant"),
          to: NodeId::new("MyStruct"),
          problematic: vec![],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        (NodeId::new("Interface"), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        (NodeId::new("MyClass"), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: NodeId::new("MyClass"),
          to: NodeId::new("Interface"),
          problematic: vec![],
        },
      ],
    });
  }
}
