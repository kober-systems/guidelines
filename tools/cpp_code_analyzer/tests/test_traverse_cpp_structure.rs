use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn traverse_ifdefs() {
    let code = r#"
#ifndef AbstractMyClass_h_INCLUDED
#define AbstractMyClass_h_INCLUDED

class AbstractMyClass {
public:
    virtual void foo() = 0;
};

#endif // AbstractMyClass_h_INCLUDED
"#;
    let errors = analyze_cpp(code);
    assert!(errors.is_empty());
}
