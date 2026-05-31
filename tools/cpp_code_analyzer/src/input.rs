use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::Deserialize;

use crate::ast::AST;
use crate::parser::parse_cpp_chunc;

#[derive(Deserialize)]
struct CompileEntry {
  directory: String,
  file: String,
}

/// Extract unique, resolved file paths from a compile_commands.json string.
pub fn file_paths_from_compile_commands(json: &str) -> Result<Vec<PathBuf>, serde_json::Error> {
  let entries: Vec<CompileEntry> = serde_json::from_str(json)?;
  let mut seen = HashSet::new();
  let mut paths = Vec::new();
  for entry in entries {
    let file = PathBuf::from(&entry.file);
    let resolved = if file.is_absolute() {
      file
    } else {
      PathBuf::from(&entry.directory).join(file)
    };
    if seen.insert(resolved.clone()) {
      paths.push(resolved);
    }
  }
  Ok(paths)
}

/// Read and parse C++ source files.
///
/// If `path` points to a file named `compile_commands.json` the listed source
/// files are loaded.  Otherwise the path is treated as a directory (or single
/// file) and traversed as before.
pub fn get_sources(path: &Path) -> io::Result<Vec<AST>> {
  if path.file_name().and_then(|n| n.to_str()) == Some("compile_commands.json") {
    let json = fs::read_to_string(path)?;
    let file_paths = file_paths_from_compile_commands(&json)
      .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut entries = Vec::new();
    for file_path in file_paths {
      let content = fs::read_to_string(&file_path)?;
      let filepath = file_path.to_string_lossy().to_string();
      entries.push(parse_cpp_chunc(&filepath, &content));
    }
    Ok(entries)
  } else {
    get_sources_from_dir(path)
  }
}

fn get_sources_from_dir(dir: &Path) -> io::Result<Vec<AST>> {
  let mut entries = vec![];
  if dir.is_dir() {
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_dir() {
        if !is_hidden(&path) {
          entries.append(&mut get_sources_from_dir(&path)?);
        }
      } else {
        let filepath = path.to_string_lossy().to_string();
        if filepath.ends_with(".h") || filepath.ends_with(".cpp") {
          let content = fs::read_to_string(&path)?;
          entries.push(parse_cpp_chunc(&filepath, &content));
        }
      }
    }
  } else {
    let filepath = dir.to_string_lossy().to_string();
    let content = fs::read_to_string(dir)?;
    entries.push(parse_cpp_chunc(&filepath, &content));
  }
  Ok(entries)
}

fn is_hidden(path: &Path) -> bool {
  path.file_name().unwrap().to_string_lossy().starts_with('.')
}
