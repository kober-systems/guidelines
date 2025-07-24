use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn traverse_ifdefs() {
    let code = r#"
#ifndef AbstractMyClass_h_INCLUDED
#define AbstractMyClass_h_INCLUDED

class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;
    virtual void foo() = 0;
};

#endif // AbstractMyClass_h_INCLUDED
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn traverse_namespaces() {
    let code = r#"
namespace mynamespace {

class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;
    virtual void foo() = 0;
};

}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn traverse_templates() {
    let code = r#"
template <typename T>
class AbstractMyClass<T> {
public:
    virtual ~AbstractMyClass() = default;
    virtual T foo() = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_global_functions() {
    let code = r#"
int global_function(int param1, float param2);
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_function_definitions() {
    let code = r#"
int global_function(int param1) {
  if (true || true != false && ~1 == 2) {
    return 42 * 1;
  } else {
    return 42 | 0xff << -(1 >> 8);
  }

  // comments should be ignored
  for (int i=0; i<42; i++) {
    2+40;
  }

  return 42;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_method_definitions() {
    let code = r#"
int myClass::method() {
  return 42;
}
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_global_enums() {
    let code = r#"
enum class my_enum {
  variant_0,
  variant_1 = 1000,
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_global_structs() {
    let code = r#"
struct my_struct {
  int x=42;
};

typedef struct my_struct2 {
  int x=42;
} my_struct2;
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_preproc_args() {
    let code = r#"
#define PREPROC_PARAM 20;

#if defined(PROPROC_CONDITION)
#define PREPROC_PARAM2 42;
#elif defined(ELSE_PREPROC_CONDITION)
#define PREPROC_PARAM2 0;
#else
#define PREPROC_PARAM2 1;
#endif

#ifdef PROPROC_CONDITION2
#define PREPROC_PARAM3 42;
#endif
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parse_alias_declarations() {
    let code = r#"
using my_alias = MyClass::my_inner_enum;
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

