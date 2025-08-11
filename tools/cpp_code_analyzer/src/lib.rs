pub mod parser;
pub mod checker;
pub mod ast;
pub mod visualize;

pub fn analyze_cpp(input: &str) -> Vec<String> {
  lints_to_strings(analyze_cpp_errors("", input))
}

pub fn analyze_cpp_errors(name: &str, input: &str) -> Vec<ast::LintError> {
  let ast = parser::parse_cpp_chunc(name, input);

  checker::check_global_codechunk(vec![ast])
}

fn lints_to_strings(input: Vec<ast::LintError>) -> Vec<String> {
  input.into_iter().map(|err| err.message).collect()
}

