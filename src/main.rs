use std::env;
use std::path::PathBuf;
use std::boxed::Box;
use std::fs::File;
use std::io::Read;

fn load_bin(path: &PathBuf) -> Box<[u8]> {
	let mut bytes = Vec::new();
	let mut file = File::open(path).unwrap();
	file.read_to_end(&mut bytes).unwrap();
	bytes.into_boxed_slice()
}

fn main() {
	
	let rom_path = PathBuf::from(env::args().nth(1).unwrap());
	let rom_binary = load_bin(&rom_path);

	println!("ROM file name: {:?}", rom_path.file_name().unwrap());
	println!("ROM size: {:?}", rom_binary.len());


}
