mod common;
use common::*;

#[test]
fn test_while() {
    let program = run_source(
        "
            @compute @workgroup_size(1)
            fn main() {
                var a = 10;
                var b = 0;
                while(a > 0) {
                    a--;
                    b++;
                }
            }
        ",
    );

    assert_eq!(get_var(&program, "a"), 0.into());

    assert_eq!(get_var(&program, "b"), 10.into());
}
