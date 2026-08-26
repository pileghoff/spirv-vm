fn test(i: i32) -> i32 {
	if(i == 0) {
		return 1;
	}

	if(i == 1) {
		return 0;
	}

	return 99;
}

@compute @workgroup_size(1)
fn main() {
    var a = vec2();
	a.x = 2;
	a.y = test(a.y);
}
