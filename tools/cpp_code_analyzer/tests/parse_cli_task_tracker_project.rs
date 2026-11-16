use std::path::PathBuf;

use cpp_code_analyzer::input::get_sources;

#[path = "fixtures/cli_task_tracker_ast.rs"]
mod cli_task_tracker_ast;
use cli_task_tracker_ast::*;
use pretty_assertions::assert_eq;

fn get_sources_sorted(path: &std::path::Path) -> Vec<cpp_code_analyzer::ast::AST> {
  let mut sources = get_sources(path).unwrap();
  sources.sort_by(|left, right| left.name.cmp(&right.name));
  sources
}

#[test]
fn parses_cli_task_tracker_into_full_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources_sorted(&fixture);
  let mut expected = vec![
    include_task_ast(&fixture),
    include_task_repository_ast(&fixture),
    test_task_storage_main_ast(&fixture),
    main_ast(&fixture),
    task_storage_header_ast(&fixture),
    task_storage_source_ast(&fixture),
  ];
  expected.sort_by(|left, right| left.name.cmp(&right.name));

  assert_eq!(actual, expected);
}

#[test]
fn parses_task_header_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("include/Task.h")).unwrap();

  assert_eq!(actual, vec![include_task_ast(&fixture)]);
}

#[test]
fn parses_task_repository_header_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("include/TaskRepository.h")).unwrap();

  assert_eq!(actual, vec![include_task_repository_ast(&fixture)]);
}

#[test]
fn parses_task_storage_test_main_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("test/test_task_storage/test_main.cpp")).unwrap();

  assert_eq!(actual, vec![test_task_storage_main_ast(&fixture)]);
}

#[test]
fn parses_main_source_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("src/main.cpp")).unwrap();

  assert_eq!(actual, vec![main_ast(&fixture)]);
}

#[test]
fn parses_task_storage_header_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("lib/TaskStorage/include/TaskStorage.h")).unwrap();

  assert_eq!(actual, vec![task_storage_header_ast(&fixture)]);
}

#[test]
fn parses_task_storage_source_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("lib/TaskStorage/src/TaskStorage.cpp")).unwrap();

  assert_eq!(actual, vec![task_storage_source_ast(&fixture)]);
}
