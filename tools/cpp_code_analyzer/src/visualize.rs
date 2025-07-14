use std::collections::HashMap;

use crate::ast::{AST, Class, Kind};

use layout::adt::dag::NodeHandle;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;
//use layout::topo::placer::Placer;

pub fn visualize(ast: &Vec<AST>, code: &str) -> String {
  let mut vg = VisualGraph::new(Orientation::LeftToRight);
  let mut handles = HashMap::default();

  for node in ast.into_iter() {
    visualize_node(node, code, &mut vg, &mut handles);
  }

  let mut svg = SVGWriter::new();
  vg.do_it(false, false, false, &mut svg);

  svg.finalize()
}

fn visualize_node(input: &AST, code: &str, vg: &mut VisualGraph, handles: &mut HashMap<String, NodeHandle>) {
  let name = &input.name;
  match &input.kind {
    Kind::File { content } => {
      for child in input.children.iter() {
        visualize_node(child, &content, vg, handles)
      }
    },
    Kind::Class(ref cl) => visualize_class(cl, &input, &name, code, vg, handles),
    Kind::Function(_fun) => (),
    Kind::Type => (),
    Kind::Variable(_var) => (),
    Kind::Unhandled(_element) => (),
    _ => todo!()
  }
}

fn visualize_class(cl: &Class, _input: &AST, name: &str, _code: &str, vg: &mut VisualGraph, handles: &mut HashMap<String, NodeHandle>) {
  let vis_name = if cl.is_abstract {
    &format!("(A) {name}")
  } else {
    name
  };
  let abstract_style = StyleAttr::simple();
  let derived_style = StyleAttr::simple();
  let sz = Point::new(get_text_width(vis_name), 100.);

  // Add the nodes to the graph, and save a handle to each node.
  let handle = vg.add_node(Element::create(
    ShapeKind::new_box(vis_name),
    if cl.is_abstract { abstract_style } else { derived_style },
    Orientation::LeftToRight,
    sz)
  );

  handles.insert(name.to_string(), handle);
}

fn get_text_width(text: &str) -> f64 {
  (text.len() as f64 * 8.) + 20.
}
