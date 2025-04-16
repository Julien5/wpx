extern crate chrono;
extern crate glob;

pub mod speed;
pub mod worker;

use std::io::{BufReader, Read};
use std::fs::File;

fn main() {
	let filename = "data/blackforest.gpx";
	let file = File::open(filename).unwrap();
	let mut reader_file = BufReader::new(file);
	let mut content: Vec<u8> = Vec::new();
	let _ = reader_file.read_to_end(&mut content);
	let _=worker::worker(&content);
}
