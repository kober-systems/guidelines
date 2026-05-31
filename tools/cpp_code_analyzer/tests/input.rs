use std::fs;
use std::path::PathBuf;

use cpp_code_analyzer::input::{file_paths_from_compile_commands, get_sources};
use pretty_assertions::assert_eq;

// ── file_paths_from_compile_commands ─────────────────────────────────────────

#[test]
fn parses_absolute_file_paths() {
  let json = r#"[
    {"directory": "/build", "file": "/src/foo.cpp", "command": "cc -c foo.cpp"}
  ]"#;
  let paths = file_paths_from_compile_commands(json).unwrap();
  assert_eq!(paths, vec![PathBuf::from("/src/foo.cpp")]);
}

#[test]
fn resolves_relative_paths_against_directory() {
  let json = r#"[
    {"directory": "/build", "file": "foo.cpp", "command": "cc -c foo.cpp"}
  ]"#;
  let paths = file_paths_from_compile_commands(json).unwrap();
  assert_eq!(paths, vec![PathBuf::from("/build/foo.cpp")]);
}

#[test]
fn deduplicates_duplicate_entries() {
  // The same file may appear multiple times (e.g. compiled with different flags).
  let json = r#"[
    {"directory": "/build", "file": "/src/foo.cpp", "command": "cc foo.cpp"},
    {"directory": "/build", "file": "/src/foo.cpp", "command": "cc -DDEBUG foo.cpp"}
  ]"#;
  let paths = file_paths_from_compile_commands(json).unwrap();
  assert_eq!(paths, vec![PathBuf::from("/src/foo.cpp")]);
}

#[test]
fn returns_multiple_files_in_stable_order() {
  let json = r#"[
    {"directory": "/build", "file": "/src/bar.cpp", "command": "cc bar.cpp"},
    {"directory": "/build", "file": "/src/foo.cpp", "command": "cc foo.cpp"}
  ]"#;
  let mut paths = file_paths_from_compile_commands(json).unwrap();
  paths.sort();
  assert_eq!(paths, vec![
    PathBuf::from("/src/bar.cpp"),
    PathBuf::from("/src/foo.cpp"),
  ]);
}

// ── get_sources ──────────────────────────────────────────────────────────────

#[test]
fn get_sources_reads_files_from_compile_commands() {
  let dir = tempdir();
  let cpp_path = dir.join("sensor.cpp");
  fs::write(&cpp_path, "int x;\n").unwrap();

  let json = format!(
    r#"[{{"directory": "{}", "file": "sensor.cpp", "command": "cc sensor.cpp"}}]"#,
    dir.to_string_lossy()
  );
  let json_path = dir.join("compile_commands.json");
  fs::write(&json_path, &json).unwrap();

  let sources = get_sources(&json_path).unwrap();

  let names: Vec<_> = sources.iter().map(|a| a.name.as_str()).collect();
  assert_eq!(names, vec![cpp_path.to_string_lossy().as_ref()]);
}

#[test]
fn get_sources_falls_back_to_directory_traversal() {
  let dir = tempdir();
  fs::write(dir.join("a.cpp"), "int x;\n").unwrap();
  fs::write(dir.join("b.h"), "int y;\n").unwrap();
  fs::write(dir.join("readme.txt"), "ignored\n").unwrap();

  let mut sources = get_sources(&dir).unwrap();
  sources.sort_by(|a, b| a.name.cmp(&b.name));
  let names: Vec<_> = sources.iter().map(|a| a.name.as_str()).collect();
  assert_eq!(names, vec![
    dir.join("a.cpp").to_string_lossy().into_owned(),
    dir.join("b.h").to_string_lossy().into_owned(),
  ]);
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn tempdir() -> PathBuf {
  let dir = std::env::temp_dir().join(format!(
    "cpp_analyzer_test_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .subsec_nanos()
  ));
  fs::create_dir_all(&dir).unwrap();
  dir
}
