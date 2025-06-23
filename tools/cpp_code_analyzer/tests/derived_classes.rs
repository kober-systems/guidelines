use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn derived_class_happy_path() {
    let code = r#"
class MyClass: public AbstractMyInterface {
public:
    void foo();

private:
    int my_private_variable = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn must_derive_from_interface() {
    let code = r#"
class MyClass {
public:
    void foo();

private:
    int my_private_variable = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Class 'MyClass' must be derived from abstract interface",
    ]);
}

#[test]
fn derives_must_use_public() {
    let code = r#"
class MyClass: private AbstractMyInterface {
public:
    void foo();

private:
    int my_private_variable = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Class 'MyClass': Derives must always be public",
    ]);
}

#[test]
fn derives_must_use_abstract_interfaces() {
    let code = r#"
class MyClass: public MyOtherClass {
public:
    void foo();

private:
    int my_private_variable = 0;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Class 'MyClass': Derives must always be from abstract interfaces",
    ]);
}

