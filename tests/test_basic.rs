use naga::{
    back::spv::{self, Options, WriterFlags},
    valid::ShaderStages,
};
use spirvemu::parse::parse_words;
use spirvemu::run::run;
use spirvemu::types::*;
use spirvemu::{id_types::*, program::Program};

fn compile(source: &str) -> Vec<u32> {
    let module: naga::Module = naga::front::wgsl::parse_str(source).unwrap();
    let module_info: naga::valid::ModuleInfo = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .subgroup_stages(naga::valid::ShaderStages::all())
    .subgroup_operations(naga::valid::SubgroupOperationSet::all())
    .validate(&module)
    .unwrap();
    let options = Options::default();

    spv::write_vec(&module, &module_info, &options, None).unwrap()
}

fn run_source(source: &str) -> Program {
    let spirv = compile(source);
    let mut program = parse_words(spirv);
    run(&mut program);
    program
}

#[test]
fn test_add_vars() {
    let source = "
    @compute @workgroup_size(1)
    fn main() {
        var a: i32 = 0;
        var b: i32 = a + 1;
    }";
    let program = run_source(source);

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(program.mem_read(&a).unwrap(), 0i32.into());

    let b: ValueId = program.find_valueid_for_name("b").unwrap();
    assert_eq!(program.mem_read(&b).unwrap(), 1i32.into());
}

#[test]
fn test_add_vars_u32() {
    let source = "
    @compute @workgroup_size(1)
    fn main() {
        var a: u32 = 0;
        var b: u32 = a + 1;
    }";
    let program = run_source(source);

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(program.mem_read(&a).unwrap(), 0u32.into());

    let b: ValueId = program.find_valueid_for_name("b").unwrap();
    assert_eq!(program.mem_read(&b).unwrap(), 1u32.into());
}

#[test]
fn test_function_ret_into_var() {
    let program = run_source(
        "
    fn foo() -> i32 {
        return 1;
    }

    @compute @workgroup_size(1)
    fn main() {
        var a: i32 = foo();
    }",
    );

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(program.mem_read(&a).unwrap(), 1i32.into());
}

#[test]
fn test_add_in_func() {
    let program = run_source(
        "
    fn foo(i: i32) -> i32 {
        return i + i;
    }

    @compute @workgroup_size(1)
    fn main() {
        var a: i32 = 1;
        var b: i32 = foo(a); // b = 2
        a = foo(b); // a  = 4
    }",
    );

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(program.mem_read(&a).unwrap(), 4i32.into());

    let b: ValueId = program.find_valueid_for_name("b").unwrap();
    assert_eq!(program.mem_read(&b).unwrap(), 2i32.into());
}

#[test]
fn test_if_else() {
    let program = run_source(
        "
    fn foo(i: i32) -> i32 {
        if( i == 0) {
            return 999;
        }
        return i + 1;
    }

    @compute @workgroup_size(1)
    fn main() {
        var a: i32 = foo(0); // a = 999
        var b: i32 = foo(1); // b = 2
    }",
    );

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(program.mem_read(&a).unwrap(), 999i32.into());

    let b: ValueId = program.find_valueid_for_name("b").unwrap();
    assert_eq!(program.mem_read(&b).unwrap(), 2i32.into());
}

#[test]
fn test_eq() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                let a = 10;
                let b = 20;
                var x = a == b;
            }
        ",
    );

    let x: ValueId = program.find_valueid_for_name("x").unwrap();
    assert_eq!(program.mem_read(&x).unwrap(), false.into());
}

#[test]
fn test_lt() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                let a = 10;
                let b = 20;
                var x = a < b;
            }
        ",
    );

    let x: ValueId = program.find_valueid_for_name("x").unwrap();
    assert_eq!(program.mem_read(&x).unwrap(), true.into());
}

#[test]
fn test_gt() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                let a = 10;
                let b = 20;
                var x = a > b;
            }
        ",
    );

    let x: ValueId = program.find_valueid_for_name("x").unwrap();
    assert_eq!(program.mem_read(&x).unwrap(), false.into());
}

#[test]
fn test_lte() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                let a = 10;
                let b = 20;
                var x = a <= b;
            }
        ",
    );

    let x: ValueId = program.find_valueid_for_name("x").unwrap();
    assert_eq!(program.mem_read(&x).unwrap(), true.into());
}

#[test]
fn test_gte() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                let a = 10;
                let b = 20;
                let c = 10;
                var x = a >= b;
                var y = a >= c;
            }
        ",
    );

    let x: ValueId = program.find_valueid_for_name("x").unwrap();
    assert_eq!(program.mem_read(&x).unwrap(), false.into());

    let y: ValueId = program.find_valueid_for_name("y").unwrap();
    assert_eq!(program.mem_read(&y).unwrap(), true.into());
}

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

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(
        program.mem_read(&a).unwrap(),
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

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(
        program.mem_read(&a).unwrap(),
        RuntimeValue::Vec {
            lenght: 2,
            contents: vec![2.into(), 1.into()]
        }
    );

    let b: ValueId = program.find_valueid_for_name("b").unwrap();
    assert_eq!(
        program.mem_read(&b).unwrap(),
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

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(
        program.mem_read(&a).unwrap(),
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

    let a: ValueId = program.find_valueid_for_name("a").unwrap();
    assert_eq!(
        program.mem_read(&a).unwrap(),
        RuntimeValue::Struct {
            members: vec![
                RuntimeValue::Struct {
                    members: vec![2u32.into(), 4u32.into()]
                },
                6u32.into()
            ]
        }
    );

    let c: ValueId = program.find_valueid_for_name("c").unwrap();
    assert_eq!(program.mem_read(&c).unwrap(), 10u32.into());
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
        program
            .mem_read(&program.find_valueid_for_name("a").unwrap())
            .unwrap(),
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
        program
            .mem_read(&program.find_valueid_for_name("b").unwrap())
            .unwrap(),
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
        program
            .mem_read(&program.find_valueid_for_name("c").unwrap())
            .unwrap(),
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
