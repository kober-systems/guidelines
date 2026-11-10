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

fn include_task_repository_ast(fixture: &Path) -> AST {
    AST {
        name: fixture.join("include/TaskRepository.h").to_string_lossy().into_owned(),
        kind: File {
            content: read_fixture(&fixture, "include/TaskRepository.h"),
        },
        children: vec![
            AST {
                name: "TaskRepository".to_string(),
                kind: Class(
                    Class {
                        is_abstract: false,
                    },
                ),
                children: vec![
                    AST {
                        name: "~TaskRepository".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: true,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 104..140,
                        instructions: vec![],
                    },
                    AST {
                        name: "Task".to_string(),
                        kind: Reference(
                            TypeRead,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 152..156,
                        instructions: vec![],
                    },
                    AST {
                        name: "add".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: true,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
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
                                range: 161..178,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![],
                        range: 144..184,
                        instructions: vec![],
                    },
                    AST {
                        name: "all".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: true,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "std::vector<Task>".to_string(),
                              kind: Reference(
                                TypeRead
                              ),
                              children: vec![],
                              dependencies: vec![
                                AST {
                                    name: "std::vector".to_string(),
                                    kind: Reference(
                                      TypeRead
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 195..206,
                                    instructions: vec![],
                                },
                                AST {
                                    name: "Task".to_string(),
                                    kind: Reference(
                                      TypeRead
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 207..211,
                                    instructions: vec![],
                                },
                              ],
                              range: 195..212,
                              instructions: vec![],
                          },
                        ],
                        range: 187..229,
                        instructions: vec![],
                    },
                    AST {
                        name: "complete".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: true,
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
                                range: 254..260,
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
                                range: 240..244,
                                instructions: vec![],
                            },
                        ],
                        range: 232..266,
                        instructions: vec![],
                    },
                ],
                dependencies: vec![],
                range: 70..268,
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
                name: "<vector>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 32..50,
                instructions: vec![],
            },
            AST {
                name: "\"Task.h\"".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 51..69,
                instructions: vec![],
            },
        ],
        range: 0..270,
        instructions: vec![],
    }
}

fn test_task_storage_main_ast(fixture: &Path) -> AST {
    AST {
        name: fixture.join("test/test_task_storage/test_main.cpp").to_string_lossy().into_owned(),
        kind: File {
            content: read_fixture(&fixture, "test/test_task_storage/test_main.cpp"),
        },
        children: vec![
            AST {
                name: "test_task_storage_adds_and_completes_tasks".to_string(),
                kind: Function(
                    Function {
                        is_virtual: false,
                        visibility: "public".to_string(),
                        in_external_namespace: None,
                    },
                ),
                children: vec![],
                dependencies: vec![],
                range: 20..70,
                instructions: vec![],
            },
            AST {
                name: "setup".to_string(),
                kind: Function(
                    Function {
                        is_virtual: false,
                        visibility: "public".to_string(),
                        in_external_namespace: None,
                    },
                ),
                children: vec![
                    AST {
                        name: "UNITY_BEGIN".to_string(),
                        kind: Reference(
                            Call,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 89..100,
                        instructions: vec![],
                    },
                    AST {
                        name: "RUN_TEST".to_string(),
                        kind: Reference(
                            Call,
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "test_task_storage_adds_and_completes_tasks".to_string(),
                              kind: Reference(
                                Read,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 115..157,
                              instructions: vec![],
                          },
                        ],
                        range: 106..114,
                        instructions: vec![],
                    },
                    AST {
                        name: "UNITY_END".to_string(),
                        kind: Reference(
                            Call,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 162..171,
                        instructions: vec![],
                    },
                ],
                dependencies: vec![],
                range: 72..176,
                instructions: vec![],
            },
            AST {
                name: "loop".to_string(),
                kind: Function(
                    Function {
                        is_virtual: false,
                        visibility: "public".to_string(),
                        in_external_namespace: None,
                    },
                ),
                children: vec![],
                dependencies: vec![],
                range: 178..192,
                instructions: vec![],
            },
        ],
        dependencies: vec![
            AST {
                name: "<unity.h>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 0..19,
                instructions: vec![],
            },
        ],
        range: 0..193,
        instructions: vec![],
    }
}

fn main_ast(fixture: &Path) -> AST {
    AST {
        name: fixture.join("src/main.cpp").to_string_lossy().into_owned(),
        kind: File {
            content: read_fixture(&fixture, "src/main.cpp"),
        },
        children: vec![
            AST {
                name: "task_storage".to_string(),
                kind: Variable(
                    Variable {
                        is_const: false,
                        visibility: "public".to_string(),
                    },
                ),
                children: vec![],
                dependencies: vec![
                  AST {
                      name: "TaskStorage".to_string(),
                      kind: Reference(
                          TypeRead,
                      ),
                      children: vec![],
                      dependencies: vec![],
                      range: 141..152,
                      instructions: vec![],
                  },
                ],
                range: 153..165,
                instructions: vec![],
            },
            AST {
                name: "main".to_string(),
                kind: Function(
                    Function {
                        is_virtual: false,
                        visibility: "public".to_string(),
                        in_external_namespace: None,
                    },
                ),
                children: vec![
                    AST {
                        name: "argc".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "public".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 177..185,
                        instructions: vec![],
                    },
                    AST {
                        name: "argv".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "public".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 187..199,
                        instructions: vec![],
                    },
                    AST {
                        name: "argc".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 209..213,
                        instructions: vec![],
                    },
                    AST {
                        name: "std::cerr".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 225..234,
                        instructions: vec![],
                    },
                    AST {
                        name: "command".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: true,
                                visibility: "public".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "std::string".to_string(),
                              kind: Reference(
                                TypeRead
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 318..329,
                              instructions: vec![],
                          },
                        ],
                        range: 330..337,
                        instructions: vec![],
                    },
                    AST {
                        name: "argv".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 340..344,
                        instructions: vec![],
                    },
                    AST {
                        name: "command".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 355..362,
                        instructions: vec![],
                    },
                    AST {
                        name: "argc".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 375..379,
                        instructions: vec![],
                    },
                    AST {
                        name: "std::cout".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 392..401,
                        instructions: vec![],
                    },
                    AST {
                        name: "task_storage".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "add".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![
                                AST {
                                    name: "id".to_string(),
                                    kind: Reference(
                                        Call,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 443..445,
                                    instructions: vec![],
                                },
                              ],
                              dependencies: vec![
                                AST {
                                    name: "argv".to_string(),
                                    kind: Reference(
                                        Read,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 434..438,
                                    instructions: vec![],
                                },
                              ],
                              range: 430..433,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 417..429,
                        instructions: vec![],
                    },
                    AST {
                        name: "command".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 481..488,
                        instructions: vec![],
                    },
                    AST {
                        name: "task".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: true,
                                visibility: "public".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "Task".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 517..521,
                              instructions: vec![],
                          },
                        ],
                        range: 521..527,
                        instructions: vec![],
                    },
                    AST {
                        name: "task_storage".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "all".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 543..546,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 530..542,
                        instructions: vec![],
                    },
                    AST {
                        name: "std::cout".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 558..567,
                        instructions: vec![],
                    },
                    AST {
                        name: "task".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "id".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 576..578,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 571..575,
                        instructions: vec![],
                    },
                    AST {
                        name: "task".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "completed".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 598..607,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 593..597,
                        instructions: vec![],
                    },
                    AST {
                        name: "task".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "title".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 639..644,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 634..638,
                        instructions: vec![],
                    },
                    AST {
                        name: "command".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 686..693,
                        instructions: vec![],
                    },
                    AST {
                        name: "argc".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 711..715,
                        instructions: vec![],
                    },
                    AST {
                        name: "task_storage".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![
                          AST {
                              name: "complete".to_string(),
                              kind: Reference(
                                  Call,
                              ),
                              children: vec![],
                              dependencies: vec![
                                AST {
                                    name: "std::stoi".to_string(),
                                    kind: Reference(
                                        Call,
                                    ),
                                    children: vec![],
                                    dependencies: vec![
                                      AST {
                                          name: "argv".to_string(),
                                          kind: Reference(
                                              Read,
                                          ),
                                          children: vec![],
                                          dependencies: vec![],
                                          range: 767..771,
                                          instructions: vec![],
                                      },
                                    ],
                                    range: 757..766,
                                    instructions: vec![],
                                },
                              ],
                              range: 748..756,
                              instructions: vec![],
                          },
                        ],
                        dependencies: vec![],
                        range: 735..747,
                        instructions: vec![],
                    },
                    AST {
                        name: "std::cerr".to_string(),
                        kind: Reference(
                            Read,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 793..802,
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
                        range: 168..171,
                        instructions: vec![],
                    },
                ],
                range: 168..840,
                instructions: vec![],
            },
        ],
        dependencies: vec![
            AST {
                name: "<iostream>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 0..20,
                instructions: vec![],
            },
            AST {
                name: "<string>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 20..38,
                instructions: vec![],
            },
            AST {
                name: "\"TaskStorage.h\"".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 39..64,
                instructions: vec![],
            },
        ],
        range: 0..841,
        instructions: vec![],
    }
}

fn task_storage_header_ast(fixture: &Path) -> AST {
    AST {
        name: fixture.join("lib/TaskStorage/include/TaskStorage.h").to_string_lossy().into_owned(),
        kind: File {
            content: read_fixture(&fixture, "lib/TaskStorage/include/TaskStorage.h"),
        },
        children: vec![
            AST {
                name: "TaskStorage".to_string(),
                kind: Class(
                    Class {
                        is_abstract: false,
                    },
                ),
                children: vec![
                    AST {
                        name: "add".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![
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
                                range: 144..161,
                                instructions: vec![],
                            },
                        ],
                        dependencies: vec![
                          AST {
                              name: "Task".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![],
                              range: 135..139,
                              instructions: vec![],
                          },
                        ],
                        range: 135..172,
                        instructions: vec![],
                    },
                    AST {
                        name: "all".to_string(),
                        kind: Function(
                            Function {
                                is_virtual: false,
                                visibility: "public".to_string(),
                                in_external_namespace: None,
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "std::vector<Task>".to_string(),
                              kind: Reference(
                                  TypeRead,
                              ),
                              children: vec![],
                              dependencies: vec![
                                AST {
                                    name: "std::vector".to_string(),
                                    kind: Reference(
                                        TypeRead,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 175..186,
                                    instructions: vec![],
                                },
                                AST {
                                    name: "Task".to_string(),
                                    kind: Reference(
                                        TypeRead,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 187..191,
                                    instructions: vec![],
                                },
                              ],
                              range: 175..192,
                              instructions: vec![],
                          },
                        ],
                        range: 175..214,
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
                                name: "id".to_string(),
                                kind: Variable(
                                    Variable {
                                        is_const: false,
                                        visibility: "public".to_string(),
                                    },
                                ),
                                children: vec![],
                                dependencies: vec![],
                                range: 231..237,
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
                                range: 217..221,
                                instructions: vec![],
                            },
                        ],
                        range: 217..248,
                        instructions: vec![],
                    },
                    AST {
                        name: "tasks_".to_string(),
                        kind: Variable(
                            Variable {
                                is_const: false,
                                visibility: "private".to_string(),
                            },
                        ),
                        children: vec![],
                        dependencies: vec![
                          AST {
                              name: "std::vector<Task>".to_string(),
                              kind: Reference(
                                TypeRead
                              ),
                              children: vec![],
                              dependencies: vec![
                                AST {
                                    name: "std::vector".to_string(),
                                    kind: Reference(
                                        TypeRead,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 262..273,
                                    instructions: vec![],
                                },
                                AST {
                                    name: "Task".to_string(),
                                    kind: Reference(
                                        TypeRead,
                                    ),
                                    children: vec![],
                                    dependencies: vec![],
                                    range: 274..278,
                                    instructions: vec![],
                                },
                              ],
                              range: 262..279,
                              instructions: vec![],
                          },
                        ],
                        range: 280..286,
                        instructions: vec![],
                    },
                ],
                dependencies: vec![
                    AST {
                        name: "TaskRepository".to_string(),
                        kind: Reference(
                            Depend,
                        ),
                        children: vec![],
                        dependencies: vec![],
                        range: 107..121,
                        instructions: vec![],
                    },
                ],
                range: 80..289,
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
                name: "<vector>".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 32..50,
                instructions: vec![],
            },
            AST {
                name: "\"TaskRepository.h\"".to_string(),
                kind: Reference(
                    Depend,
                ),
                children: vec![],
                dependencies: vec![],
                range: 51..79,
                instructions: vec![],
            },
        ],
        range: 0..291,
        instructions: vec![],
    }
}

fn read_fixture(fixture: &Path, path: &str) -> String {
  fs::read_to_string(fixture.join(path)).unwrap()
}

