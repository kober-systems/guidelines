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
struct GraphData {
  nodes: BTreeMap<String, Entity>,
  connections: Vec<Connection>,
}

#[derive(Debug, PartialEq)]
struct Connection {
  kind: ConnectionType, // Dependency, Inheritance, Composition, Usage,
  from: String,
  to: String,
  problematic: Option<String>,
}

#[derive(Debug, PartialEq)]
enum ConnectionType {
  Usage,
  Inheritance,
  Composition,
}

#[derive(Debug, PartialEq)]
struct Entity {
  kind: String, // Extern, Interface, Class, Variable, Function, Type (Struct|Enum)
  name: String,
  problematic: Option<String>,
}

pub fn visualize(ast: Vec<AST>, code: &str) -> String {
  let ast = crate::checker::add_lint_erros(ast);

  let mut vg = VisualGraph::new(Orientation::LeftToRight);

  let mut g = GraphData { nodes: BTreeMap::default(), connections: vec![] };
  for node in ast.iter() {
    g = extract_node(node, code, g)
  }
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
      get_style(node.problematic.is_some()),
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
        look: get_style(con.problematic.is_some()),
        ..Arrow::default()
      },
      from,
      to);
  }
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
          problematic: None,
        });
      }
      base
    },
    Kind::Type|Kind::Reference|Kind::Variable(_) => {
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

/// Some nodes make the output only less readable. Keep them only when
/// they create problems in the code and need attention
fn remove_visual_noise(graph: GraphData) -> GraphData {
  let GraphData { nodes, connections } = graph;
  let mut nodes_to_be_removed: HashSet<_> = nodes.iter().filter_map(|(name, node)| match node.kind.as_str() {
    "T"|"V"|"F" => {
      if node.problematic.is_none() {
        Some(name.to_string())
      } else {
        None
      }
    }
    _ => None,
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
    Kind::Reference => "Ref",
    //Kind::Unhandled(_element) => (),
    _ => todo!()
  }
}

fn get_text_width(text: &str) -> f64 {
  (text.len() as f64 * 8.) + 20.
}

fn is_problematic(node: &AST) -> Option<String> {
  if node.children.iter().filter(|n| match n.kind {
    Kind::LintError(_) => true,
    Kind::Unhandled(_) => true,
    _ => false,
  }).count() > 0 {
    Some("TODO".to_string())
  } else {
    None
  }
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
          problematic: None,
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: None,
        }),
        ("MyGlobalVar".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: Some("Global variables create hidden dependencies".to_string()),
        }),
        ("MyGlobalConstant".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalConstant".to_string(),
          problematic: None,
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: None,
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalVar".to_string(),
          problematic: Some("Global variables create hidden dependencies".to_string()),
        },
      ],
    };
    assert_eq!(remove_visual_noise(graph), GraphData {
      nodes: BTreeMap::from([
        ("Interface".to_string(), Entity {
          kind: "I".to_string(),
          name: "Interface".to_string(),
          problematic: None,
        }),
        ("MyClass".to_string(), Entity {
          kind: "C".to_string(),
          name: "MyClass".to_string(),
          problematic: None,
        }),
        ("MyGlobalVar".to_string(), Entity {
          kind: "V".to_string(),
          name: "MyGlobalVar".to_string(),
          problematic: Some("Global variables create hidden dependencies".to_string()),
        }),
      ]),
      connections: vec![
        Connection {
          kind: ConnectionType::Inheritance,
          from: "MyClass".to_string(),
          to: "Interface".to_string(),
          problematic: None,
        },
        Connection {
          kind: ConnectionType::Usage,
          from: "MyClass".to_string(),
          to: "MyGlobalVar".to_string(),
          problematic: Some("Global variables create hidden dependencies".to_string()),
        },
      ],
    });
  }
}
