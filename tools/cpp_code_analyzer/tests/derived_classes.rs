use cpp_code_analyzer::analyze_cpp;
use pretty_assertions::assert_eq;

#[test]
fn derived_class_happy_path() {
    let code = r#"
class MyClass: public AbstractMyInterface {
public:
    MyClass(int x, AbstractUsedClass *used);
    void foo();

private:
    int my_private_variable = 0;
    AbstractUsedClass *handle = nullptr;
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

#[test]
fn all_attributes_must_be_private() {
    let code = r#"
class MyClass: public AbstractMyInterface {
public:
    void foo();

    int my_variable = 0;
    AbstractUsedClass *handle = nullptr;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, [
      "Derived class 'MyClass' must not have non private attributes ('my_variable')",
      "Derived class 'MyClass' must not have non private attributes ('*handle')",
    ]);
}

#[test]
fn can_explicitly_allow_public_attributes() {
    let code = r#"
// lint: ignore E_MOD_01 reason: only used in testing scenarios
class MyClass: public AbstractMyInterface {
public:
    void foo();

    int my_variable = 0;
    AbstractUsedClass *handle = nullptr;
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn can_derive_from_template_class() {
    let code = r#"
class MyClass: public AbstractMyInterface<int> {
public:
    MyClass();
    void foo();
};
"#;
    let errors = analyze_cpp(code);
    assert_eq!(errors, Vec::<String>::new());
}

