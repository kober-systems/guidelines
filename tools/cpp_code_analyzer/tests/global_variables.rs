use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn prevent_definition_of_global_variables() {
    let code = r#"
int my_global = 42;

int my_other_global;
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "It's not allowed to create global variables ('my_global'). Global variables create invisible coupling.",
      "It's not allowed to create global variables ('my_other_global'). Global variables create invisible coupling.",
    ]);
}

