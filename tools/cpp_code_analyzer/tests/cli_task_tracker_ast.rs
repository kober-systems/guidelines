use std::{fs, path::{Path, PathBuf}};

use cpp_code_analyzer::{
  ast::{AST, Class, Function, Kind::*, Reference::*, Variable},
  input::get_sources,
};
use pretty_assertions::assert_eq;

#[test]
fn parses_task_header_into_ast_structure() {
  let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cli_task_tracker");

  let actual = get_sources(&fixture.join("include/Task.h")).unwrap();

  assert_eq!(actual, vec![include_task_ast(&fixture)]);
}

fn include_task_ast(fixture: &Path) -> AST {
    AST {
        name: fixture.join("include/Task.h").to_string_lossy().into_owned(),
        kind: File {
            content: read_fixture(&fixture, "include/Task.h"),
        },
        children: vec![
            AST {
                name: "Task".to_string(),
                kind: Class(
                    Class {
                        is_abstract: false,
                    },
                ),
                children: vec![
                    AST {
                        name: "Task".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
                            AST {
                                name: "id".to_string(),
                                kind: Variable(
                                    Variable {
                                        is_const: false,
                                        visibility: "public".to_string(),
                                    },
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 81..87,
                                instructions: vec![],
                            },
                            AST {
                                name: "title".to_string(),
                                kind: Variable(
                                    Variable {
                                        is_const: false,
                                        visibility: "public".to_string(),
                                    },
                                ),
                                children: vec![],
                                dependencies: vec![
                                    AST {
                                        name: "std::string".to_string(),
                                        kind: Reference(
                                            TypeRead,
                                        ),
                                        children: vec![],
                                        dependencies: vec![],
                                        range: 0..0,
                                        instructions: vec![],
                                    },
                                ],
                                range: 89..106,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![],
                        range: 76..146,
                        instructions: vec![],
                    },
                    AST {
                        name: "id".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
                            AST {
                                name: "id_".to_string(),
                                kind: Reference(
                                    Read,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 174..177,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![
                            AST {
                                name: "int".to_string(),
                                kind: Reference(
                                    TypeRead,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 150..153,
                                instructions: vec![],
                            },
                        ],
                        range: 150..180,
                        instructions: vec![],
                    },
                    AST {
                        name: "title".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
                            AST {
                                name: "title_".to_string(),
                                kind: Reference(
                                    Read,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 225..231,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![
                            AST {
                                name: "std::string".to_string(),
                                kind: Reference(
                                    TypeRead,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 189..200,
                                instructions: vec![],
                            },
                        ],
                        range: 183..234,
                        instructions: vec![],
                    },
                    AST {
                        name: "completed".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
                            AST {
                                name: "completed_".to_string(),
                                kind: Reference(
                                    Read,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 269..279,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![
                            AST {
                                name: "bool".to_string(),
                                kind: Reference(
                                    TypeRead,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 237..241,
                                instructions: vec![],
                            },
                        ],
                        range: 237..282,
                        instructions: vec![],
                    },
                    AST {
                        name: "complete".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
                            AST {
                                name: "completed_".to_string(),
                                kind: Reference(
                                    Write,
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 303..313,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![],
                        range: 285..323,
                        instructions: vec![],
                    },
                    AST {
                        name: "id_".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "private".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "int".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 337..340,
                              instructions: vec![],
                          },
                        ],
                        range: 341..344,
                        instructions: vec![],
                    },
                    AST {
                        name: "title_".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "private".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "std::string".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 348..359,
                              instructions: vec![],
                          },
                        ],
                        range: 360..366,
                        instructions: vec![],
                    },
                    AST {
                        name: "completed_".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "private".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "bool".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 370..374,
                              instructions: vec![],
                          },
                        ],
                        range: 375..385,
                        instructions: vec![],
                    },
                ],
                dependencies: vec![],
                range: 52..396,
                instructions: vec![],
            },
        ],
        dependencies: vec![
            AST {
                name: "<string>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 14..32,
                instructions: vec![],
            },
            AST {
                name: "<utility>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 32..51,
                instructions: vec![],
            },
        ],
        range: 0..398,
        instructions: vec![],
    }
}

