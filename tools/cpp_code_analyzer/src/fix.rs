use std::collections::HashMap;

use crate::{ast::{LintError, AST}, parser::parse_cpp_chunc};

pub struct Fix {
  pub instruction: FixInstruction,
  pub main_lint_err: LintError,
  pub affected_lint_errors: Vec<LintError>,
}

pub enum FixInstruction {
  CreateAbstractClass(String),
}

pub fn apply_fixes(fixes: Vec<Fix>, files: SourceFiles) -> SourceFiles {
  let mut files = SourceAstTree::from_sources(files);
  for fix in fixes.into_iter() {
    use FixInstruction::*;

    match fix.instruction {
      CreateAbstractClass(class_name) => {
        let path = fix.main_lint_err.file_path.clone();
        let ast = files.tree.remove(&path).expect(&format!("{path} not found"));
        let content = ast.get_file_content().expect("needs to be a file");

        let idx = ast.children.iter().position(|element| element.name == class_name).expect("Not found");
        let class = &ast.children[idx];
        let interface_content = create_interface_content(class, &content);
        let interface_path = path.replace(&class_name, &format!("Abstract{class_name}"));
        let interface_ast = AST::default().set_file_content(interface_content);
        files.tree.insert(interface_path, interface_ast);

        let content = modify_to_derive_from_interface(class, &content);
        files.tree.insert(path, ast.set_file_content(content));
      }
    }
  }

  files.tree.into_iter()
    .map(|(path, ast)| (path, ast.get_file_content().expect("needs to be a file")))
    .collect()
}

fn create_interface_content(class: &AST, context_content: &str) -> String {
  let mut content = "\nclass Abstract".to_string() + &class.name;
  content += " {\npublic:\n";
  content += &format!("  virtual ~Abstract{}() = default;\n\n", class.name);
  for child in class.children.iter() {
    match &child.kind {
      crate::ast::Kind::Function(fun) => {
        if fun.visibility == "public" && child.name != class.name {
          let function_sig = &context_content[child.range.start..child.range.end];
          let function_sig = match function_sig.rsplit_once(";") {
            Some((function_sig, _)) => function_sig,
            None => function_sig,
          };
          content += &format!("  virtual {function_sig} = 0;\n");
        }
      }
      _ => (),
    }
  }
  if !content.ends_with("\n") {
    content += "\n";
  }
  content += "}\n";
  content
}

fn modify_to_derive_from_interface(class: &AST, content: &str) -> String {
  let (before, after) = content.split_once("class ").expect("pattern `class` not found");
  let (_, after) = get_classname(after);
  let content = before.to_string() + "class "
    + &class.name + ": public Abstract" + &class.name
    + " " + after;
  content
}

fn get_classname(input: &str) -> (&str, &str) {
  input.split_once(" ").expect("should work")
}

type SourceFiles = HashMap<String, String>;

struct SourceAstTree {
 tree: HashMap<String, AST>,
}

impl SourceAstTree {
  fn from_sources(input: SourceFiles) -> Self {
    let mut output = HashMap::default();

    for (path, content) in input.into_iter() {
      let content = parse_cpp_chunc(&path, &content);
      output.insert(path, content);
    }

    Self {
      tree: output,
    }
  }
}

