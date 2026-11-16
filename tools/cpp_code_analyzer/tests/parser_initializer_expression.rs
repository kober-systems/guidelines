use cpp_code_analyzer::{
    ast::{AST, Function, Kind::*, Reference::*, Variable},
    parser::parse_cpp_chunc,
};
use pretty_assertions::assert_eq;

#[test]
fn attaches_constructor_initializer_expressions_to_the_declared_variable() {
    let source = "void inspect() { Task task(static_cast<int>(tasks_.size()) + 1, std::move(title)); }";

    let actual = parse_cpp_chunc("test.cpp", source);

    assert_eq!(
        actual,
        AST {
            name: "test.cpp".to_string(),
            kind: File {
                content: source.to_string(),
            },
            children: vec![AST {
                name: "inspect".to_string(),
                kind: Function(Function {
                    is_virtual: false,
                    visibility: "public".to_string(),
                    in_external_namespace: None,
                }),
                children: vec![AST {
                    name: "task".to_string(),
                    kind: Variable(Variable {
                        is_const: false,
                        visibility: "private".to_string(),
                    }),
                    children: vec![],
                    dependencies: vec![
                        AST {
                            name: "Task".to_string(),
                            kind: Reference(TypeRead),
                            children: vec![],
                            dependencies: vec![],
                            range: 17..21,
                            instructions: vec![],
                        },
                        AST {
                            name: "static_cast".to_string(),
                            kind: Reference(Call),
                            children: vec![],
                            dependencies: vec![
                                AST {
                                    name: "tasks_".to_string(),
                                    kind: Reference(Read),
                                    children: vec![AST {
                                        name: "size".to_string(),
                                        kind: Reference(Call),
                                        children: vec![],
                                        dependencies: vec![],
                                        range: 51..55,
                                        instructions: vec![],
                                    }],
                                    dependencies: vec![],
                                    range: 44..50,
                                    instructions: vec![],
                                },
                                AST {
                                    name: "std::move".to_string(),
                                    kind: Reference(Call),
                                    children: vec![],
                                    dependencies: vec![AST {
                                        name: "title".to_string(),
                                        kind: Reference(Read),
                                        children: vec![],
                                        dependencies: vec![],
                                        range: 74..79,
                                        instructions: vec![],
                                    }],
                                    range: 64..73,
                                    instructions: vec![],
                                },
                            ],
                            range: 27..38,
                            instructions: vec![],
                        },
                    ],
                    range: 22..26,
                    instructions: vec![],
                }],
                dependencies: vec![],
                range: 0..84,
                instructions: vec![],
            }],
            dependencies: vec![],
            range: 0..84,
            instructions: vec![],
        }
    );
}
