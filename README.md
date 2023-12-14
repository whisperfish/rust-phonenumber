# phonenumber

![CI Build](https://github.com/whisperfish/rust-phonenumber/workflows/Build/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/phonenumber.svg)](https://crates.io/crates/phonenumber)
[![phonenumber](https://docs.rs/phonenumber/badge.svg)](https://docs.rs/phonenumber)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Rust version of [libphonenumber](https://github.com/googlei18n/libphonenumber).
We currently require 1.58.0 as minimum supported Rust version (MSRV).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
phonenumber = "0.3"
```
## Example

The following example parses, validates and formats the given phone number.

```rust,no_run
use phonenumber::Mode;
use std::env;

fn main() {
	let mut args = env::args().skip(1).collect::<Vec<_>>();

	if args.len() < 1 {
		panic!("not enough arguments");
	}

	let number  = args.pop().unwrap();
	let country = args.pop().map(|c| c.parse().unwrap());

	let number = phonenumber::parse(country, number).unwrap();
	let valid  = phonenumber::is_valid(&number);

	if valid {
		println!("\x1b[32m{:#?}\x1b[0m", number);
		println!();
		println!("International: {}", number.format().mode(Mode::International));
		println!("     National: {}", number.format().mode(Mode::National));
		println!("      RFC3966: {}", number.format().mode(Mode::Rfc3966));
		println!("        E.164: {}", number.format().mode(Mode::E164));
	}
	else {
		println!("\x1b[31m{:#?}\x1b[0m", number);
	}
}
```
