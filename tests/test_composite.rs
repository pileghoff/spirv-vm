use spirvemu::id_types::*;
use spirvemu::types::*;
mod common;
use common::*;

#[test]
fn test_vec2_init() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var a = vec2();
                a.x = 2;
            }
        ",
    );

    assert_eq!(
        get_var(&program, "a"),
        RuntimeValue::Vec {
            lenght: 2,
            contents: vec![2.into(), 0.into()]
        }
    );
}

#[test]
fn test_vec2_math() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var a = vec2();
                a.x = 2;
                a.y = 1;
                var b = vec2();
                b.y = 4;
                b.x = a.x + a.y + b.y; // 2 + 1 + 4 = 7
            }
        ",
    );

    assert_eq!(
        get_var(&program, "a"),
        RuntimeValue::Vec {
            lenght: 2,
            contents: vec![2.into(), 1.into()]
        }
    );

    assert_eq!(
        get_var(&program, "b"),
        RuntimeValue::Vec {
            lenght: 2,
            contents: vec![7.into(), 4.into()]
        }
    );
}

#[test]
fn test_struct() {
    let program = run_source(
        "
            struct Foo{
              bar: u32,
              baz: u32,
            }

            @compute @workgroup_size(1)
            fn main() {
                var a = Foo(2, 4);
            }
        ",
    );

    assert_eq!(
        get_var(&program, "a"),
        RuntimeValue::Struct {
            members: vec![2u32.into(), 4u32.into()]
        }
    );
}

#[test]
fn test_nested_struct() {
    let program = run_source(
        "
            struct Foo{
              bar: u32,
              baz: u32,
            }

            struct FooFoo{
              bar: Foo,
              baz: u32,
            }

            @compute @workgroup_size(1)
            fn main() {
                var a = FooFoo(Foo(2, 4), 6);
                var b = FooFoo(Foo(4, 4), 6);
                var c = b.baz + a.bar.baz; // 4 + 6 = 10
            }
        ",
    );

    assert_eq!(
        get_var(&program, "a"),
        RuntimeValue::Struct {
            members: vec![
                RuntimeValue::Struct {
                    members: vec![2u32.into(), 4u32.into()]
                },
                6u32.into()
            ]
        }
    );

    assert_eq!(get_var(&program, "c"), 10u32.into());
}

#[test]
fn test_nested_struct_reassign_member() {
    let program = run_source(
        "
            struct Foo{
              bar: u32,
              baz: u32,
            }

            struct FooFoo{
              bar: Foo,
              baz: u32,
            }

            @compute @workgroup_size(1)
            fn main() {
                var a = FooFoo(Foo(2, 4), 6);
                a.baz = 2;
                a.bar.baz = 3;

                var b = FooFoo(Foo(2, 4), 6);
                b.bar = Foo(999, 31);

                var c = FooFoo(Foo(2, 4), 6);
                c = FooFoo(Foo(1,2),3);
            }
        ",
    );

    assert_eq!(
        get_var(&program, "a"),
        RuntimeValue::Struct {
            members: vec![
                RuntimeValue::Struct {
                    members: vec![2u32.into(), 3u32.into()]
                },
                2u32.into()
            ]
        }
    );

    assert_eq!(
        get_var(&program, "b"),
        RuntimeValue::Struct {
            members: vec![
                RuntimeValue::Struct {
                    members: vec![999u32.into(), 31u32.into()]
                },
                6u32.into()
            ]
        }
    );

    assert_eq!(
        get_var(&program, "c"),
        RuntimeValue::Struct {
            members: vec![
                RuntimeValue::Struct {
                    members: vec![1u32.into(), 2u32.into()]
                },
                3u32.into()
            ]
        }
    );
}
