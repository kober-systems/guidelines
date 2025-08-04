use std::collections::{BTreeMap, HashSet};

use crate::ast::{AST, Kind};

use layout::adt::dag::NodeHandle;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::color::Color;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;
//use layout::topo::placer::Placer;

#[derive(Debug, PartialEq)]
pub struct GraphData {
  pub nodes: BTreeMap<String, Entity>,
  pub connections: Vec<Connection>,
}

#[derive(Debug, PartialEq)]
pub struct Connection {
  pub kind: ConnectionType, // Dependency, Inheritance, Composition, Usage,
  pub from: String,
  pub to: String,
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
  vg.do_it(false, false, false, &mut svg);

  svg.finalize()
}

fn visualize_graph_data(g: GraphData, vg: &mut VisualGraph) {
  let mut handles: BTreeMap<String, NodeHandle> = BTreeMap::default();

  for (key, node) in g.nodes.iter() {
    let vis_name = format!("({}) {}", node.kind, node.name);
    let sz = Point::new(get_text_width(&vis_name), 100.);

    let handle = vg.add_node(Element::create(
      ShapeKind::new_box(&vis_name),
      get_style(!node.problematic.is_empty()),
      Orientation::LeftToRight,
      sz)
    );

    handles.insert(key.to_string(), handle);
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

pub fn ast_to_graph(ast: Vec<AST>, code: &str) -> GraphData {
  let mut g = GraphData { nodes: BTreeMap::default(), connections: vec![] };
  for node in ast.iter() {
    g = extract_node(node, code, g)
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

fn extract_node(input: &AST, code: &str, base: GraphData) -> GraphData {
  let mut base = base;
  match &input.kind {
    Kind::File { content } => {
      for child in input.children.iter() {
        base = extract_node(child, &content, base)
      }
      base
    },
    Kind::Class(ref cl) => {
      base.nodes.insert(input.name.clone(), Entity {
        kind: if cl.is_abstract { "A".to_string() } else { "C".to_string() },
        name: input.name.clone(),
        problematic: is_problematic(input) } );
      for dependecy in input.dependencies.iter() {
        let dep_name = dependecy.name.to_string();
        if !base.nodes.contains_key(&dep_name) {
          base = extract_node(dependecy, code, base);
        }
        base.connections.push(Connection {
          kind: ConnectionType::Inheritance,
          from: input.name.clone(),
          to: dep_name,
          problematic: vec![],
        });
      }
      for child in input.children.iter() {
        base = extract_references(child, &input.name, code, base);
      }
      base
    },
    Kind::Type|Kind::Reference(_)|Kind::Variable(_) => {
      base.nodes.insert(input.name.clone(), Entity {
        kind: get_entity_type(&input).to_string(),
        name: input.name.clone(),
        problematic: is_problematic(input)} );
      base
    },
    Kind::Function(_) => {
      let name = match input.name.split_once("(") {
        Some((name, _params)) => name,
        None => &input.name,
      };
      match name.split_once("::") {
        Some((class_name, _f_name)) => {
          if !base.nodes.contains_key(class_name) {
            base.nodes.insert(class_name.to_string(), Entity {
              kind: "C".to_string(),
              name: input.name.clone(),
              problematic: is_problematic(input)} );
          }
        }
        None => {
        }
      }
      base
    },
    Kind::Unhandled(_element) => base,
    _ => todo!()
  }
}

fn extract_references(input: &AST, from: &str, code: &str, base: GraphData) -> GraphData {
  let mut base = base;
  match &input.kind {
    Kind::Reference(_) => {
      let dep_name = input.name.to_string();
      if !base.nodes.contains_key(&dep_name) {
        base = extract_node(input, code, base);
      }
      base.connections.push(Connection {
        kind: ConnectionType::Usage,
        from: from.to_string(),
        to: dep_name,
        problematic: vec![],
      });
      base
    },
    Kind::Function(_) => {
      for child in input.children.iter() {
        base = extract_references(child, from, code, base);
      }
      base
    },
    Kind::Unhandled(_element) => base,
    Kind::Variable(_) => base,
    Kind::LintError(_) => base,
    _ => todo!("{:?}", input.kind)
  }
}

/// Some nodes make the output only less readable. Keep them only when
/// they create problems in the code and need attention
pub fn remove_visual_noise(graph: GraphData) -> GraphData {
  let GraphData { nodes, connections } = graph;
  let mut nodes_to_be_removed: HashSet<_> = nodes.iter().filter_map(|(name, node)| match node.kind.as_str() {
    "T"|"V"|"F" => {
      if node.problematic.is_empty() {
        Some(name.to_string())
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
    Kind::LintError(msg) => Some(msg.clone()),
    _ => None,
  }).collect()
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use super::*;

  #[test]
  fn non_problematic_variables_should_be_filtered() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        ("MyGlobalVar".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        }),
        ("MyGlobalConstant".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        ("MyGlobalVar".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalVar".to_string(),
          problematic: vec!["Global variables create hidden dependencies".to_string()],
        },
      ],
    });
  }

  #[test]
  fn non_problematic_nodes_with_problematic_connections_should_stay() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        ("MyGlobalConstant".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalConstant".to_string(),
          problematic: vec!["Something is wrong".to_string()],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        ("MyGlobalConstant".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalConstant".to_string(),
          problematic: vec!["Something is wrong".to_string()],
        },
      ],
    });
  }

  #[test]
  fn remove_non_problematic_connections_to_uninteresting_nodes() {
    let graph = GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
        ("MyGlobalConstant".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: vec![],
        }),
        ("MyStruct".to_string(), Entity {
          kind: "T".to_string(),
          name: "MyStruct".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalConstant".to_string(),
          problematic: vec![],
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyGlobalConstant".to_string(),
          to: "MyStruct".to_string(),
          problematic: vec![],
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: vec![],
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: vec![],
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: vec![],
        },
      ],
    });
  }
}
