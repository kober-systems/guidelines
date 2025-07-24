use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn prevent_definition_of_global_variables() {
    let code = r#"
int my_global = 42;

int my_other_global;

char my_global_array[42];
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "It's not allowed to create global variables ('my_global'). Global variables create invisible coupling.",
      "It's not allowed to create global variables ('my_other_global'). Global variables create invisible coupling.",
      "It's not allowed to create global variables ('my_global_array[42]'). Global variables create invisible coupling.",
    ]);
}

#[test]
fn allow_definition_of_constant_global_variables() {
    let code = r#"
constexpr int my_constant_global = 42;

typedef struct {
  int var = 42;
} my_struct;

constexpr my_struct my_constant_global_struct;
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn prevent_usage_of_global_variables() {
    let code = r#"
int function_using_global_var() {
  return global_var;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "It's not allowed to use global variables ('global_var'). Global variables create invisible coupling.",
    ]);
}

#[test]
fn allow_usage_of_parameters() {
    let code = r#"
int function_using_param(int var, float var2) {
  return var;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn allow_calling_external_functions() {
    let code = r#"
int function_call_other(int var) {
  external_fn_with_param(var, &var, 42);

  return some_external_function();
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

