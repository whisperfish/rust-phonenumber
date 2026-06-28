// Regression test for https://github.com/whisperfish/rust-phonenumber/issues/103
//
// The library previously guarded a regex cache behind a mutex, which serialised
// all parsing and made concurrent use pointless. The cache has since been
// removed; these tests ensure parsing stays lock-free-friendly and that the
// public types remain `Send + Sync` so the library is usable across threads.

use std::sync::Arc;
use std::thread;

use phonenumber::{Mode, country, parse};

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn public_types_are_send_sync() {
    assert_send_sync::<phonenumber::PhoneNumber>();
    assert_send_sync::<phonenumber::Metadata>();
    assert_send_sync::<phonenumber::metadata::Database>();
}

#[test]
fn parses_concurrently_from_many_threads() {
    // A spread of inputs across different regions, with the expected E.164 form.
    let cases: Arc<Vec<(Option<country::Id>, &'static str, &'static str)>> = Arc::new(vec![
        (None, "+1 6502530000", "+16502530000"),
        (Some(country::GB), "+44 7912345678", "+447912345678"),
        (Some(country::IT), "+39 0236618300", "+390236618300"),
        (Some(country::FR), "+33142764978", "+33142764978"),
        (Some(country::US), "+64 3 331 6005", "+6433316005"),
    ]);

    let mut handles = Vec::new();
    for _ in 0..16 {
        let cases = Arc::clone(&cases);
        handles.push(thread::spawn(move || {
            for _ in 0..200 {
                for (country, input, expected) in cases.iter() {
                    let parsed = parse(*country, input).expect("should parse");
                    assert_eq!(parsed.format().mode(Mode::E164).to_string(), *expected);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }
}
