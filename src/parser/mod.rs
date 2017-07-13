// Copyright (C) 2017 1aim GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use metadata::{DATABASE, Database};
use phone_number::PhoneNumber;
use national_number::NationalNumber;
use country::{Code, Country};
use extension::Extension;
use carrier::Carrier;
use consts;
use validator::{self, Validation};
use phone_number::Type;
use error::{self, Result};

pub mod helper;
pub mod valid;
pub mod rfc3966;
pub mod natural;

/// Parse a phone number.
pub fn parse<S: AsRef<str>>(country: Option<Country>, string: S) -> Result<PhoneNumber> {
	parse_with(&*DATABASE, country, string)
}

/// Parse a phone number using a specific `Database`.
pub fn parse_with<S: AsRef<str>>(database: &Database, country: Option<Country>, string: S) -> Result<PhoneNumber> {
	named!(phone_number(&str) -> helper::Number,
		alt_complete!(call!(rfc3966::phone_number) | call!(natural::phone_number)));

	// Try to parse the number as RFC3966 or natural language.
	let mut number = phone_number(string.as_ref()).to_full_result()
		.or(Err(error::Parse::NoNumber))?;

	// Normalize the number and extract country code.
	number = helper::country_code(database, country, number)?;

	// Extract carrier and strip national prefix if present.
	if let Some(meta) = country.and_then(|c| database.by_id(c.as_ref())) {
		let mut potential = helper::national_number(meta, number.clone());

		// Strip national prefix if present.
		if let Some(prefix) = meta.national_prefix.as_ref() {
			if potential.national.starts_with(prefix) {
				potential.national = helper::trim(potential.national, prefix.len());
			}
		}

		if validator::length(meta, &potential, Type::Unknown) != Validation::TooShort {
			number = potential;
		}
	}

	if number.national.len() < consts::MIN_LENGTH_FOR_NSN {
		return Err(error::Parse::TooShortNsn.into());
	}

	if number.national.len() > consts::MAX_LENGTH_FOR_NSN {
		return Err(error::Parse::TooLong.into());
	}

	Ok(PhoneNumber {
		country: Code {
			code:   number.prefix.map(|p| p.parse()).unwrap_or(Ok(0))?,
			source: number.country,
		},

		national: NationalNumber {
			value: number.national.parse()?,
			zeros: number.national.chars().take_while(|&c| c == '0').count() as u8,
		},

		extension: number.extension.map(|s| Extension(s.into_owned())),
		carrier:   number.carrier.map(|s| Carrier(s.into_owned())),
	})
}

#[cfg(test)]
mod test {
	use parser;
	use phone_number::PhoneNumber;
	use national_number::NationalNumber;
	use country::{Code, Country, Source};

	#[test]
	fn parse() {
		let mut number = PhoneNumber {
			country: Code {
				code:   64,
				source: Source::Default,
			},

			national: NationalNumber {
				value: 33316005,
				zeros: 0,
			},

			extension: None,
			carrier:   None,
		};

		number.country.source = Source::Default;
		assert_eq!(number, parser::parse(Some(Country::NZ), "033316005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "33316005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "03-331 6005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "03 331 6005").unwrap());

		number.country.source = Source::Plus;
		assert_eq!(number, parser::parse(Some(Country::NZ), "tel:03-331-6005;phone-context=+64").unwrap());
		// FIXME: What the fuck is this.
		// assert_eq!(number, parser::parse(Some(Country::NZ), "tel:331-6005;phone-context=+64-3").unwrap());
		// assert_eq!(number, parser::parse(Some(Country::NZ), "tel:331-6005;phone-context=+64-3").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "tel:03-331-6005;phone-context=+64;a=%A1").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "tel:03-331-6005;isub=12345;phone-context=+64").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "tel:+64-3-331-6005;isub=12345").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "03-331-6005;phone-context=+64").unwrap());

		number.country.source = Source::Idd;
		assert_eq!(number, parser::parse(Some(Country::NZ), "0064 3 331 6005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::US), "01164 3 331 6005").unwrap());

		number.country.source = Source::Plus;
		assert_eq!(number, parser::parse(Some(Country::US), "+64 3 331 6005").unwrap());

		assert_eq!(number, parser::parse(Some(Country::US), "+01164 3 331 6005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "+0064 3 331 6005").unwrap());
		assert_eq!(number, parser::parse(Some(Country::NZ), "+ 00 64 3 331 6005").unwrap());

		let number = PhoneNumber {
			country: Code {
				code:   64,
				source: Source::Number,
			},

			national: NationalNumber {
				value: 64123456,
				zeros: 0,
			},

			extension: None,
			carrier:   None,
		};

		assert_eq!(number, parser::parse(Some(Country::NZ), "64(0)64123456").unwrap());

		assert_eq!(PhoneNumber {
			country: Code {
				code:   49,
				source: Source::Default,
			},

			national: NationalNumber {
				value: 30123456,
				zeros: 0,
			},

			extension: None,
			carrier:   None,
		}, parser::parse(Some(Country::DE), "301/23456").unwrap());

		assert_eq!(PhoneNumber {
			country: Code {
				code:   81,
				source: Source::Plus,
			},

			national: NationalNumber {
				value: 2345,
				zeros: 0,
			},

			extension: None,
			carrier:   None,
		}, parser::parse(Some(Country::JP), "+81 *2345").unwrap());

		assert_eq!(PhoneNumber {
			country: Code {
				code:   64,
				source: Source::Default,
			},

			national: NationalNumber {
				value: 12,
				zeros: 0,
			},

			extension: None,
			carrier:   None,
		}, parser::parse(Some(Country::NZ), "12").unwrap());

		assert_eq!(PhoneNumber {
			country: Code {
				code:   55,
				source: Source::Default,
			},

			national: NationalNumber {
				value: 3121286979,
				zeros: 0,
			},

			extension: None,
			carrier:   Some("12".into()),
		}, parser::parse(Some(Country::BR), "012 3121286979").unwrap());
	}
}
