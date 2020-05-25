use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[macro_use]
extern crate failure;
extern crate quick_xml as xml;
extern crate regex;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[path = "src/metadata/loader.rs"]
mod loader;

#[path = "src/error.rs"]
mod error;

fn main() {
    let metadata = loader::load(BufReader::new(
        File::open("assets/PhoneNumberMetadata.xml").expect("could not open metadata file"),
    ))
    .expect("failed to load metadata");

    let mut out = BufWriter::new(
        File::create(&Path::new(&env::var("OUT_DIR").unwrap()).join("database.bin"))
            .expect("could not create database file"),
    );

    bincode::serialize_into(&mut out, &metadata).expect("failed to serialize database");
}
