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

use crate::carrier::Carrier;
use crate::consts;
use crate::country;
use crate::error;
use crate::extension::Extension;
use crate::metadata::{Database, DATABASE};
use crate::national_number::NationalNumber;
use crate::phone_number::{PhoneNumber, Type};
use crate::validator::{self, Validation};

use nom::{branch::alt, IResult};

#[macro_use]
pub mod helper;
pub mod natural;
pub mod rfc3966;
pub mod valid;

/// Parse a phone number.
pub fn parse<S: AsRef<str>>(
    country: Option<country::Id>,
    string: S,
) -> Result<PhoneNumber, error::Parse> {
    parse_with(&DATABASE, country, string)
}

/// Parse a phone number using a specific `Database`.
pub fn parse_with<S: AsRef<str>>(
    database: &Database,
    country: Option<country::Id>,
    string: S,
) -> Result<PhoneNumber, error::Parse> {
    fn phone_number(i: &str) -> IResult<&str, helper::Number> {
        parse! { i => alt((rfc3966::phone_number, natural::phone_number)) }
    }

    // Try to parse the number as RFC3966 or natural language.
    let (_, mut number) = phone_number(string.as_ref()).or(Err(error::Parse::NoNumber))?;

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
        return Err(error::Parse::TooShortNsn);
    }

    if number.national.len() > consts::MAX_LENGTH_FOR_NSN {
        return Err(error::Parse::TooLong);
    }

    Ok(PhoneNumber {
        code: country::Code {
            value: number.prefix.map(|p| p.parse()).unwrap_or(Ok(0))?,
            source: number.country,
        },

        national: NationalNumber::new(
            number.national.parse()?,
            number.national.chars().take_while(|&c| c == '0').count() as u8,
        ),

        extension: number.extension.map(|s| Extension(s.into_owned())),
        carrier: number.carrier.map(|s| Carrier(s.into_owned())),
    })
}

#[cfg(test)]
mod test {
    use crate::country;
    use crate::national_number::NationalNumber;
    use crate::parser;
    use crate::phone_number::PhoneNumber;

    #[test]
    fn parse() {
        let mut number = PhoneNumber {
            code: country::Code {
                value: 64,
                source: country::Source::Default,
            },

            national: NationalNumber::new(33316005, 0),

            extension: None,
            carrier: None,
        };

        number.code.source = country::Source::Default;
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "033316005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "33316005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "03-331 6005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "03 331 6005").unwrap()
        );

        number.code.source = country::Source::Plus;
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "tel:03-331-6005;phone-context=+64").unwrap()
        );
        // FIXME: What the fuck is this.
        // assert_eq!(number, parser::parse(Some(country::NZ), "tel:331-6005;phone-context=+64-3").unwrap());
        // assert_eq!(number, parser::parse(Some(country::NZ), "tel:331-6005;phone-context=+64-3").unwrap());
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "tel:03-331-6005;phone-context=+64;a=%A1").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(
                Some(country::NZ),
                "tel:03-331-6005;isub=12345;phone-context=+64"
            )
            .unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "tel:+64-3-331-6005;isub=12345").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "03-331-6005;phone-context=+64").unwrap()
        );

        number.code.source = country::Source::Idd;
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "0064 3 331 6005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::US), "01164 3 331 6005").unwrap()
        );

        number.code.source = country::Source::Plus;
        assert_eq!(
            number,
            parser::parse(Some(country::US), "+64 3 331 6005").unwrap()
        );

        assert_eq!(
            number,
            parser::parse(Some(country::US), "+01164 3 331 6005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "+0064 3 331 6005").unwrap()
        );
        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "+ 00 64 3 331 6005").unwrap()
        );

        let number = PhoneNumber {
            code: country::Code {
                value: 64,
                source: country::Source::Number,
            },

            national: NationalNumber::new(64123456, 0),

            extension: None,
            carrier: None,
        };

        assert_eq!(
            number,
            parser::parse(Some(country::NZ), "64(0)64123456").unwrap()
        );

        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 49,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(30123456, 0),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::DE), "301/23456").unwrap()
        );

        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 81,
                    source: country::Source::Plus,
                },

                national: NationalNumber::new(2345, 0,),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::JP), "+81 *2345").unwrap()
        );

        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 64,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(12, 0,),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::NZ), "12").unwrap()
        );

        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 55,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(3121286979, 0),

                extension: None,
                carrier: Some("12".into()),
            },
            parser::parse(Some(country::BR), "012 3121286979").unwrap()
        );
    }

    #[test]
    fn issue_43() {
        let res = parser::parse(None, " 2 22#:");
        assert!(res.is_err());
    }

    #[test]
    fn advisory_1() {
        let res = parser::parse(None, ".;phone-context=");
        assert!(res.is_err(), "{res:?}");
    }
}
