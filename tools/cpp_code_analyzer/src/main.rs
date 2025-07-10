use std::path::PathBuf;

use clap::Parser;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;
use cpp_code_analyzer::visualize::visualize;
use cpp_code_analyzer::{analyze_cpp_errors, parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to check
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,
    #[arg(long)]
    svg: bool,
}

fn main() {
    let args = Args::parse();

    let input = std::fs::read_to_string(&args.input).expect("Could not read input");
    let filepath = args.input.to_string_lossy();
    if !args.svg {
      print_errors(&input, &filepath);
    } else {
      to_svg(&input, &filepath);
    }
}

fn print_errors(input: &str, filepath: &str) {
    let errors = analyze_cpp_errors(&filepath, &input);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    let mut files = SimpleFiles::new();
    let file_id = files.add(&filepath, input);

    for error in errors.iter() {
      let diagnostic = Diagnostic::error()
          .with_message(&error.message)
          .with_labels(vec![
              Label::primary(file_id, error.range.start..error.range.end),
          ]);

      term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }
}

fn to_svg(input: &str, filepath: &str) {
  let ast = vec![parser::parse_cpp_chunc(filepath, input)];

  println!("{}", visualize(&ast, input));
}

