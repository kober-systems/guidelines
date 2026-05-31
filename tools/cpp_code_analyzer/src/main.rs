use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;
use cpp_code_analyzer::ast::{Kind, AST};
use cpp_code_analyzer::visualize::{to_graphml, to_graphviz, visualize};
use cpp_code_analyzer::{checker, input};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to check
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,
    /// Output format
    #[arg(short, long, value_enum, default_value_t=OutputType::Terminal)]
    format: OutputType,
    /// Fix problems interactivly
    #[arg(long, default_value_t=false)]
    interactive: bool,
}

#[derive(ValueEnum, Clone, Copy)]
enum OutputType {
  /// Print out on the terminal
  Terminal,
  Svg,
  /// Graphviz dot format
  Dot,
  Graphml,
}

fn main() -> io::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let entries = input::get_sources(&args.input)?;

    use  OutputType::*;
    match args.format {
      Terminal => print_all_errors(entries, args.interactive),
      Svg => to_svg(entries),
      Dot => to_dot(entries),
      Graphml => {
        println!("{}", to_graphml(entries, ""));
      }
    }
    Ok(())
}

fn print_all_errors(ast: Vec<AST>, fix_interactive: bool) {
  let mut files = SimpleFiles::new();
  let mut mapping = HashMap::<String, usize>::default();

  for ast in ast.iter() {
    if let Kind::File { content } = &ast.kind {
      let file_id = files.add(ast.name.to_string(), content.to_string());
      mapping.insert(ast.name.to_string(), file_id);
    }
  }

  let errors = checker::check_global_codechunk(ast);

  let writer = StandardStream::stderr(ColorChoice::Always);
  let config = codespan_reporting::term::Config::default();

  let mut user_input = String::new();
  for error in errors.iter() {
    let file_id = mapping.get(&error.file_path).unwrap_or(&0);
    let diagnostic = Diagnostic::error()
        .with_message(&format!("{}", error.kind))
        .with_labels(vec![
            Label::primary(*file_id, error.range.start..error.range.end),
        ]);
    let diagnostic = if fix_interactive {
      diagnostic.with_note("no fix available. Hit enter to continue")
    } else {
      diagnostic
    };

    term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    if fix_interactive {
      std::io::stdin().read_line(&mut user_input).unwrap();
    }
  }

  println!("found {} errors", errors.len());
}

fn to_svg(ast: Vec<AST>) {
  println!("{}", visualize(ast, ""));
}

fn to_dot(ast: Vec<AST>) {
  println!("{}", to_graphviz(ast, ""));
}

