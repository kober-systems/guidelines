use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn abstract_class_happy_path() {
    let code = r#"
class AbstractMyClass {
public:
    // provides foo service to the class
    virtual void foo() = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert!(errors.is_empty());
}

#[test]
fn prevent_attributes_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    int x;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, vec![
      "Abstract class `AbstractMyClass` must not have attributes ('x')".to_string(),
    ]);
}


#[test]
fn prevent_private_members_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    virtual void foo() = 0;
private:
    int x;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, vec![
      "Abstract class `AbstractMyClass` should ONLY define 'public' methods (not allowed private)".to_string(),
      "Abstract class `AbstractMyClass` must not have attributes ('x')".to_string(),
    ]);
}

#[test]
fn make_sure_all_methods_are_virtual_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    virtual void foo() = 0;
    void bar() = 0;
    virtual void baz();
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, vec![
      "method 'void bar() = 0;' in abstract class 'AbstractMyClass' must be virtual".to_string(),
      "Abstract class 'AbstractMyClass': missing `= 0;` for method 'virtual void baz();'".to_string(),
    ]);
}

