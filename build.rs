use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::env;

extern crate thiserror;
extern crate regex;
extern crate regex_syntax;
extern crate quick_xml as xml;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::Options;

#[path = "src/metadata/loader.rs"]
mod loader;

#[path = "src/error.rs"]
mod error;

fn main() {
	println!("cargo:rerun-if-changed=assets/PhoneNumberMetadata.xml");

	let metadata = loader::load(BufReader::new(
		File::open("assets/PhoneNumberMetadata.xml")
			.expect("could not open metadata file")))
				.expect("failed to load metadata");

	let mut out = BufWriter::new(File::create(
		&Path::new(&env::var("OUT_DIR").unwrap()).join("database.bin"))
			.expect("could not create database file"));

	bincode::options().with_varint_encoding().serialize_into(&mut out, &metadata)
		.expect("failed to serialize database");
}
