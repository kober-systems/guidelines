use std::{fs, io};
use std::path::{Path, PathBuf};

use clap::Parser;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;
use cpp_code_analyzer::ast::{Kind, AST};
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

fn main() -> io::Result<()> {
    let args = Args::parse();

    let entries = get_sources_from_dir(&args.input)?;
    if !args.svg {
      print_all_errors(&entries);
    } else {
      to_svg(&entries);
    }
    Ok(())
}

fn print_all_errors(ast: &Vec<AST>) {
  for ast in ast.iter() {
    if let Kind::File { content } = &ast.kind {
      print_errors(&content, &ast.name);
    }
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

fn to_svg(ast: &Vec<AST>) {
  println!("{}", visualize(&ast, ""));
}

fn get_sources_from_dir(dir: &Path) -> io::Result<Vec<AST>> {
  let mut entries = vec![];
  if dir.is_dir() {
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_dir() {
          if !is_path_hidden(&path)  {
            entries.append(&mut get_sources_from_dir(&path)?);
          }
      } else {
        let filepath = path.to_string_lossy().to_string();
        if filepath.ends_with(".h") || filepath.ends_with(".cpp") {
          let input = std::fs::read_to_string(&path)?;
          entries.push(parser::parse_cpp_chunc(&filepath, &input));
        }
      }
    }
  } else {
    let filepath = dir.to_string_lossy().to_string();
    let input = std::fs::read_to_string(&dir)?;
    entries.push(parser::parse_cpp_chunc(&filepath, &input));
  }

  Ok(entries)
}

fn is_path_hidden(path: &Path) -> bool {
  path.file_name().unwrap().to_string_lossy().starts_with(".")
}
