use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};


use phonenumber::metadata::{Database};
use phonenumber::metadata::loader;
use phonenumber::PhoneNumber;
use phonenumber::parser;
use bincode::Options;

const DATABASE_BINFILE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/database.bin"));

fn bincode_deserialize_database(db_raw: &[u8]) -> Vec<loader::Metadata> {
	bincode::options()
		.with_varint_encoding().deserialize(db_raw).unwrap()
}

fn parse_database(db_decoded: Vec<loader::Metadata>) -> Database {
	Database::from(db_decoded, false).unwrap()
}

fn first_query(db: &Database) -> PhoneNumber {
	parser::parse_with(db, None, "+16137827274").unwrap()
}


fn criterion_benchmark(c: &mut Criterion) {
	let mut group = c.benchmark_group("init");
	group.sampling_mode(SamplingMode::Auto);
	group.sample_size(50);

	group.bench_function("deserialize binencoded database", |b| b.iter(|| bincode_deserialize_database(black_box(DATABASE_BINFILE))));

	let db_decoded = bincode_deserialize_database(DATABASE_BINFILE);
	group.bench_function("parse database (unchecked)", |b| b.iter(|| parse_database(black_box(db_decoded.to_vec()))));

	let db = parse_database(db_decoded.to_vec());
	group.bench_function("first query", |b| { db.cache().lock().unwrap().clear(); b.iter(|| first_query(black_box(&db))) });
	group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);