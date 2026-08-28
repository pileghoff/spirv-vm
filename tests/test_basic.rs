mod common;
use common::*;

#[test]
fn test_add_vars() {
    let source = "
    @compute @workgroup_size(1)
    fn main() {
        var a: i32 = 0;
        var b: i32 = a + 1;
    }";
    let program = run_source(source);

    assert_eq!(get_var(&program, "a"), 0i32.into());
    assert_eq!(get_var(&program, "b"), 1i32.into());
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

    assert_eq!(get_var(&program, "a"), 0u32.into());
    assert_eq!(get_var(&program, "b"), 1u32.into());
}

#[test]
fn test_sub() {
    let program = run_source(
        "
    @compute @workgroup_size(1)
    fn main() {
        var a: u32 = 10;
        a--;
    }",
    );

    assert_eq!(get_var(&program, "a"), 9u32.into());
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

    assert_eq!(get_var(&program, "a"), 1i32.into());
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

    assert_eq!(get_var(&program, "a"), 4i32.into());
    assert_eq!(get_var(&program, "b"), 2i32.into());
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

    assert_eq!(get_var(&program, "a"), 999i32.into());
    assert_eq!(get_var(&program, "b"), 2i32.into());
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

    assert_eq!(get_var(&program, "x"), false.into());
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

    assert_eq!(get_var(&program, "x"), true.into());
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

    assert_eq!(get_var(&program, "x"), false.into());
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

    assert_eq!(get_var(&program, "x"), true.into());
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

    assert_eq!(get_var(&program, "x"), false.into());
    assert_eq!(get_var(&program, "y"), true.into());
}

#[test]
fn test_mult() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var a = 10 * 5;
                var b = -10 * 5;
            }
        ",
    );

    assert_eq!(get_var(&program, "a"), 50.into());
    assert_eq!(get_var(&program, "b"), (-50).into());
}

#[test]
fn test_remainder() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var x = 10 % 2;
                var y = 20 % 3;
            }
        ",
    );

    assert_eq!(get_var(&program, "x"), 0.into());
    assert_eq!(get_var(&program, "y"), 2.into());
}

#[test]
fn test_umod() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var x:u32 = 10;
                x = x % 2;
            }
        ",
    );

    assert_eq!(get_var(&program, "x"), 0u32.into());
}
