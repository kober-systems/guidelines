use std::collections::BTreeMap;

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

struct GraphData {
  nodes: BTreeMap<String, Entity>,
  connections: Vec<Connection>,
}

struct Connection {
  kind: ConnectionType, // Dependency, Inheritance, Composition, Usage,
  from: String,
  to: String,
  problematic: Option<String>,
}

enum ConnectionType {
  Usage,
  Inheritance,
  Composition,
}

struct Entity {
  kind: String, // Extern, Interface, Class, Variable, Function, Type (Struct|Enum)
  name: String,
  problematic: Option<String>,
}

pub fn visualize(ast: &Vec<AST>, code: &str) -> String {
  let mut vg = VisualGraph::new(Orientation::LeftToRight);

  let mut g = GraphData { nodes: BTreeMap::default(), connections: vec![] };
  for node in ast.into_iter() {
    g = extract_node(node, code, g)
  }
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
        problematic: None } );
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
    Kind::Function(_)|Kind::Type|Kind::Reference|Kind::Variable(_) => {
      base.nodes.insert(input.name.clone(), Entity {
        kind: get_entity_type(&input).to_string(),
        name: input.name.clone(),
        problematic: None } );
      base
    },
    Kind::Unhandled(_element) => base,
    _ => todo!()
  }
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
