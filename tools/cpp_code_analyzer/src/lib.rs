pub mod parser;
pub mod checker;
pub mod ast;

pub fn analyze_cpp(input: &str) -> Vec<String> {
  let ast = parser::parse_cpp_chunc(input);

  lints_to_strings(checker::check_global_codechunk(ast))
}

fn lints_to_strings(input: Vec<ast::LintError>) -> Vec<String> {
  input.into_iter().map(|err| err.message).collect()
}

