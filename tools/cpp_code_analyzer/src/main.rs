use std::path::PathBuf;

use clap::Parser;
use cpp_code_analyzer::analyze_cpp;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to check
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let input = std::fs::read_to_string(args.input).expect("Could not read input");
    let errors = analyze_cpp(&input);

    for error in errors.iter() {
      eprintln!("\t{error}");
    }
}
