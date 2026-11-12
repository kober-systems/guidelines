use cpp_code_analyzer::{
    ast::{AST, Kind, Reference},
    parser::parse_cpp_chunc,
};
use pretty_assertions::assert_eq;

#[test]
fn represents_a_method_call_on_a_variable_with_its_argument() {
    let ast = parse_cpp_chunc("sample.cpp", "void f() { storage.add(value); }");

    assert_eq!(
        ast.children[0].children,
        vec![read(
            "storage",
            11..18,
            vec![call("add", 19..22, vec![read("value", 23..28, vec![])], vec![])],
        )],
    );
}

#[test]
fn represents_a_call_chain_on_the_previous_call_result() {
    let ast = parse_cpp_chunc("sample.cpp", "void f() { storage.add(value).id(); }");

    assert_eq!(
        ast.children[0].children,
        vec![read(
            "storage",
            11..18,
            vec![call(
                "add",
                19..22,
                vec![read("value", 23..28, vec![])],
                vec![call("id", 30..32, vec![], vec![])],
            )],
        )],
    );
}

fn read(name: &str, range: std::ops::Range<usize>, children: Vec<AST>) -> AST {
    AST {
        name: name.to_string(),
        kind: Kind::Reference(Reference::Read),
        children,
        range,
        ..AST::default()
    }
}

fn call(
    name: &str,
    range: std::ops::Range<usize>,
    dependencies: Vec<AST>,
    children: Vec<AST>,
) -> AST {
    AST {
        name: name.to_string(),
        kind: Kind::Reference(Reference::Call),
        children,
        dependencies,
        range,
        ..AST::default()
    }
}
