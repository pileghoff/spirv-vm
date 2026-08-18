fn foo(a:i32) -> i32 {
	return a + 1;
}

@compute @workgroup_size(1)
fn main() {
    let a: i32 = 10;
    let b: i32 = 20;
    let c: i32 = a + b;
	let d: i32 = foo(a);
}
