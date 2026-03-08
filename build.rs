use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

#[path = "src/metadata/loader.rs"]
mod loader;

#[path = "src/error.rs"]
mod error;

fn main() {
    build_metadata_database();
    build_carrier_data();
}

fn build_metadata_database() {
    let pnm_path = "assets/PhoneNumberMetadata.xml";
    let metadata = loader::load(BufReader::new(
        File::open(pnm_path).expect("could not open metadata file"),
    ))
    .expect("failed to load metadata");
    println!("cargo:rerun-if-changed={pnm_path}");

    let mut out = BufWriter::new(
        File::create(
            Path::new(&env::var("OUT_DIR").expect("OUT_DIR not set")).join("database.bin"),
        )
        .expect("could not create database file"),
    );

    postcard::to_io(&metadata, &mut out).expect("failed to serialize database");
}

fn build_carrier_data() {
    let carrier_dir = Path::new("assets/carrier");
    // Watch the carrier directory for structural changes.
    println!("cargo:rerun-if-changed=assets/carrier");

    // prefix → { lang → name }
    let mut entries: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut max_prefix_len: usize = 0;

    assert!(
        carrier_dir.is_dir(),
        "assets/carrier directory is missing — this is a repository bug"
    );

    // Walk each language directory: assets/carrier/en/, assets/carrier/zh/, etc.
    let mut lang_dirs: Vec<_> = fs::read_dir(carrier_dir)
        .expect("could not read carrier directory")
        .map(|e| e.expect("could not read carrier directory entry"))
        .filter(|e| e.path().is_dir())
        .collect();
    lang_dirs.sort_by_key(|e| e.file_name());

    for lang_entry in lang_dirs {
        let lang = lang_entry
            .file_name()
            .to_str()
            .expect("unicode carrier directory")
            .to_string();
        let lang_path = lang_entry.path();

        let mut txt_files: Vec<_> = fs::read_dir(&lang_path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", lang_path.display()))
            .map(|e| {
                e.unwrap_or_else(|err| {
                    panic!("could not read entry in {}: {err}", lang_path.display())
                })
            })
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
            .map(|e| e.path())
            .collect();
        txt_files.sort();

        for path in txt_files {
            println!("cargo:rerun-if-changed={}", path.display());
            let file = BufReader::new(
                File::open(&path)
                    .unwrap_or_else(|e| panic!("could not open {}: {e}", path.display())),
            );
            for line in file.lines() {
                let line = line.expect("could not read line");
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let (prefix, name) = line.split_once('|').expect("line format");
                let prefix = prefix.trim();
                let name = name.trim();
                if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) {
                    max_prefix_len = max_prefix_len.max(prefix.len());
                    entries
                        .entry(prefix.to_string())
                        .or_default()
                        .insert(lang.clone(), name.to_string());
                }
            }
        }
    }

    // Serialize as Vec<(prefix, Vec<(lang, name)>)> for postcard.
    let serializable: Vec<(String, Vec<(String, String)>)> = entries
        .into_iter()
        .map(|(prefix, langs)| {
            let lang_vec: Vec<(String, String)> = langs.into_iter().collect();
            (prefix, lang_vec)
        })
        .collect();

    let out_path =
        Path::new(&env::var("OUT_DIR").expect("OUT_DIR not set")).join("carrier_data.bin");
    let mut out =
        BufWriter::new(File::create(&out_path).expect("could not create carrier data file"));
    postcard::to_io(&(&serializable, max_prefix_len), &mut out)
        .expect("failed to serialize carrier data");
}
