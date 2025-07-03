use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn abstract_class_happy_path() {
    let code = r#"
// Provides some service
class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;
    // provides foo service to the class
    virtual void foo() = 0;
    // provide some other interface
    virtual AbstractHandle* get_handle() = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn prevent_attributes_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;
    int x;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Abstract class `AbstractMyClass` must not have attributes ('x')",
    ]);
}


#[test]
fn prevent_private_members_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;

    virtual void foo() = 0;
private:
    int x;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Abstract class `AbstractMyClass` should ONLY define 'public' methods (not allowed private)",
      "Abstract class `AbstractMyClass` must not have attributes ('x')",
    ]);
}

#[test]
fn make_sure_all_methods_are_virtual_in_abstract_classes() {
    let code = r#"
class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;

    virtual void foo() = 0;
    void bar() = 0;
    virtual void baz();
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "method 'void bar() = 0;' in abstract class 'AbstractMyClass' must be virtual",
      "Abstract class 'AbstractMyClass': missing `= 0;` for method 'virtual void baz();'",
    ]);
}

#[test]
fn should_not_permit_init_function() {
    let code = r#"
class AbstractMyClass {
public:
    virtual ~AbstractMyClass() = default;
    virtual void init() = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Abstract class 'AbstractMyClass' should not provide an init function. Initialisation should be done in constructor."
    ]);
}

#[test]
fn warn_if_default_destructor_does_not_exist() {
    let code = r#"
class AbstractMyClass {
public:
    virtual void foo() = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Abstract class 'AbstractMyClass' should provide a default destructor."
    ]);
}
