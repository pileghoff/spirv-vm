mod common;
use common::*;

#[test]
fn test_signed_add_at_maximum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = 2147483646i + 1i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), 2147483647i32.into());
}

#[test]
fn test_signed_add_at_minimum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = -2147483647i + -1i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (-2147483648i32).into());
}

#[test]
fn test_signed_sub_at_minimum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = -2147483647i - 1i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (-2147483648i32).into());
}

#[test]
fn test_signed_sub_below_maximum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = 2147483647i - 1i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), 2147483646i32.into());
}

#[test]
fn test_signed_mul_at_maximum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = 1073741823i * 2i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), 2147483646i32.into());
}

#[test]
fn test_signed_mul_at_minimum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: i32 = -1073741824i * 2i;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (-2147483648i32).into());
}

#[test]
fn test_signed_division_at_minimum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var minimum: i32 = -2147483647i - 1i;
            var divisor: i32 = 2i;
            var result: i32 = minimum / divisor;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (-1073741824i32).into());
}

#[test]
fn test_signed_division_at_maximum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var dividend: i32 = 2147483647i;
            var divisor: i32 = -1i;
            var result: i32 = dividend / divisor;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (-2147483647i32).into());
}

#[test]
fn test_unsigned_add_overflow_wraps() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: u32 = 4294967295u + 1u;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), 0u32.into());
}

#[test]
fn test_unsigned_sub_underflow_wraps() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: u32 = 0u - 1u;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), u32::MAX.into());
}

#[test]
fn test_unsigned_mul_overflow_wraps() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var result: u32 = 4294967295u * 2u;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), (u32::MAX - 1u32).into());
}

#[test]
fn test_unsigned_division_at_maximum() {
    let program = run_source(
        "
        @compute @workgroup_size(1)
        fn main() {
            var dividend: u32 = 4294967295u;
            var divisor: u32 = 2u;
            var result: u32 = dividend / divisor;
        }
        ",
    );

    assert_eq!(get_var(&program, "result"), 2147483647u32.into());
}
