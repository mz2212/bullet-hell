use std::fs;
use std::env;

fn main() {
	// env OUT_DIR maps to the actual package build directory.
	// Seems like a bit of a strange choice to me, but considering that this
	// feature was made for libraries, I'm not going to complain.
	let out_dir = format!("{}/{}", "target", env::var("PROFILE").unwrap());
	let in_dir = format!("{}/{}", env::var("CARGO_MANIFEST_DIR").unwrap(), "assets");
	let files = fs::read_dir(in_dir).unwrap();
	println!("{:?}", files);
	for f in files {
		let f_c = f.unwrap();
		fs::copy(f_c.path(), format!("{}/{}", out_dir, f_c.file_name().into_string().unwrap().as_str())).unwrap();
	}
}