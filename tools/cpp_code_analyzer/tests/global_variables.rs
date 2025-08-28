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
      "It's not allowed to create global variables ('my_global_array'). Global variables create invisible coupling.",
    ]);
}

#[test]
fn allow_definition_of_global_variables_with_main_function_present() {
    let code = r#"
int my_global = 42;

int my_other_global;

char my_global_array[42];

int main(void) {
  return my_global;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn allow_definition_of_global_variables_with_setup_and_loop_function_present() {
    let code = r#"
int my_global = 42;

int my_other_global;

char my_global_array[42];

void setup() {
  return my_global;
}
void loop() {}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
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
int function_using_param(int var, float *var2, float var3[]) {
  var2 = 42.;
  var3[0] = 42;
  return var;
}

class MyClass: public AbstractMyInterface {
public:
    MyClass();
    int foo(int foo_var) {
      return foo_var + 1;
    }
    int bar(int bar_var);
};

int MyClass::bar(int bar_var) {
  return bar_var + 1;
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

#[test]
fn allow_usage_of_internal_parameters() {
    let code = r#"
int function_defining_var() {
  int var;
  float v2, v3, v4;

  var = false;
  var += 1;
  var -= v2 * v3 / v4;
  return var;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn prevent_usage_of_global_class_variables() {
    let code = r#"
int function_using_global_var() {
  return global_class.method();
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "It's not allowed to use global variables ('global_class'). Global variables create invisible coupling.",
    ]);
}

#[test]
fn allow_usage_of_class_attributes() {
    let code = r#"
class MyClass: public AbstractMyInterface {
public:
    MyClass(int x, AbstractUsedClass *used);
    void foo() {
      external_var2 += 1 + my_private_variable;
      handle->method(my_private_variable);
    }
    int bar();

private:
    int my_private_variable = 0;
    AbstractUsedClass *handle = nullptr;
};

int MyClass::bar() {
  external_var += 1 + my_private_variable;
  handle->method(my_private_variable);
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "It's not allowed to use global variables ('external_var2'). Global variables create invisible coupling.",
      "It's not allowed to use global variables ('external_var'). Global variables create invisible coupling.",
    ]);
}

#[test]
fn allow_reading_constants() {
    let code = r#"
constexpr int i = 42;
enum myenum {
  variant1,
  variant2,
};

int function_call_other(int var) {
  return i + variant1 + myenum::variant2;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn allow_reading_constants_from_methods() {
    let code = r#"
constexpr int i = 42;

int MyClass::foo() {
  return i;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}
