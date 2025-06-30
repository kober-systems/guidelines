pub mod parser;
pub mod checker;
pub mod ast;

pub fn analyze_cpp(input: &str) -> Vec<String> {
  let ast = parser::parse_cpp_chunc(input);

  checker::check_global_codechunk(ast)
}

