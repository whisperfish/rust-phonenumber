use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[path = "src/metadata/loader.rs"]
mod loader;

#[path = "src/error.rs"]
mod error;

fn main() {
    let pnm_path = "assets/PhoneNumberMetadata.xml";
    let metadata = loader::load(BufReader::new(
        File::open(pnm_path).expect("could not open metadata file"),
    ))
    .expect("failed to load metadata");
    println!("cargo:rerun-if-changed={pnm_path}");

    let mut out = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("database.bin"))
            .expect("could not create database file"),
    );

    postcard::to_io(&metadata, &mut out).expect("failed to serialize database");
}
