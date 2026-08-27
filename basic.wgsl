@compute @workgroup_size(1)
fn main() {
	var a = 10;
	var b = 0;
	while(a > 0) {
		a--;
		b++;
	}
}
