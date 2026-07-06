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
use crate::is_viable;
use crate::metadata::{DATABASE, Database};
use crate::phone_number::{PhoneNumber, Type};
use crate::validator::{self, Validation};
use nom::{IResult, branch::alt};

#[macro_use]
pub mod helper;
pub mod natural;
pub mod rfc3601;
pub mod rfc3966;
pub mod rfc3986;
pub mod rfc4715;
pub mod rfc4904;
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
    fn phone_number(i: &str) -> IResult<&str, helper::Number<'_>> {
        parse! { i => alt((rfc3966::phone_number, natural::phone_number)) }
    }

    // Try to parse the number as RFC3966 or natural language.
    let (_, mut number) = phone_number(string.as_ref()).or(Err(error::Parse::NoNumber))?;

    if !is_viable(&number.national) {
        return Err(error::Parse::NoNumber);
    }

    // Normalize the number and extract country code.
    number = helper::country_code(database, country, number)?;

    // If no country was supplied, try to determine the metadata from the number.
    // We need to determine the country to be able to classify the national prefix if present.
    // Only trust the supplied reference country when the country code did not
    // come from the number itself. If the number carried its own country code
    // (a leading `+`, IDD, or an extracted code), the metadata must come from
    // that code, otherwise we strip the wrong national prefix.
    use either::{Left, Right};
    let meta = if let Some(country) = &country
        && number.country == country::Source::Default
    {
        database.by_id(country.as_ref())
    } else {
        let code = country::Code {
            value: number.prefix.clone().map(|p| p.parse()).unwrap_or(Ok(0))?,
            source: number.country,
        };

        let without_zeros = number.national.trim_start_matches('0');
        match validator::source_for(database, code.value(), without_zeros) {
            Some(Left(region)) => database.by_id(region.as_ref()),
            Some(Right(code)) => database.by_code(&code).and_then(|m| m.into_iter().next()),
            None => None,
        }
    };

    // Extract carrier and strip national prefix if present. national_number
    // already performs viability-checked national-prefix stripping, so it must
    // not be repeated here.
    if let Some(meta) = meta {
        let potential = helper::national_number(meta, number.clone());

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

        national: number.national.parse()?,

        extension: number.extension.map(|s| Extension(s.into_owned())),
        carrier: number.carrier.map(|s| Carrier(s.into_owned())),
    })
}

#[cfg(test)]
mod test {
    use crate::country::{self, Source};
    use crate::national_number::NationalNumber;
    use crate::parser;
    use crate::phone_number::PhoneNumber;
    use rstest::*;

    #[rstest]
    #[case(Source::Default, country::NZ, "033316005")]
    #[case(Source::Default, country::NZ, "33316005")]
    #[case(Source::Default, country::NZ, "03-331 6005")]
    #[case(Source::Default, country::NZ, "03 331 6005")]
    #[case(Source::Plus, country::NZ, "tel:03-331-6005;phone-context=+64")]
    // FIXME: What the fuck is this.
    // #[case(Source::Plus, country::NZ, "tel:331-6005;phone-context=+64-3")]
    #[case(Source::Plus, country::NZ, "tel:03-331-6005;phone-context=+64;a=%A1")]
    #[case(
        Source::Plus,
        country::NZ,
        "tel:03-331-6005;isub=12345;phone-context=+64"
    )]
    #[case(Source::Plus, country::NZ, "tel:+64-3-331-6005;isub=12345")]
    #[case(Source::Plus, country::NZ, "03-331-6005;phone-context=+64")]
    // Idd
    #[case(Source::Idd, country::NZ, "0064 3 331 6005")]
    #[case(Source::Idd, country::US, "01164 3 331 6005")]
    // Plus
    #[case(Source::Plus, country::US, "+64 3 331 6005")]
    #[case(Source::Plus, country::US, "+01164 3 331 6005")]
    #[case(Source::Plus, country::NZ, "+0064 3 331 6005")]
    #[case(Source::Plus, country::NZ, "+ 00 64 3 331 6005")]
    fn parse_1(
        #[case] source: Source,
        #[case] country: impl Into<Option<country::Id>>,
        #[case] number: &'static str,
    ) {
        let reference = PhoneNumber {
            code: country::Code { value: 64, source },

            national: NationalNumber::new(33316005, 0).unwrap(),

            extension: None,
            carrier: None,
        };

        let country = country.into();
        println!("parsing {} with country {:?}", number, country);
        let parsed = parser::parse(country, number).unwrap();
        println!("number type: {:?}", parsed.number_type());
        println!("parsed: {:?}", parsed);

        assert_eq!(reference, parsed);
    }

    #[test]
    fn parse_2() {
        // "64064123456" stripped of NZ's country code and trunk prefix is not a
        // valid NZ national number, so the leading 64 must not be treated as a
        // country code (see issue #68). The number is kept intact against the
        // default region.
        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 64,
                    source: Source::Default,
                },

                national: NationalNumber::new(64064123456, 0).unwrap(),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::NZ), "64(0)64123456").unwrap()
        );
    }

    #[test]
    fn parse_3() {
        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 49,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(30123456, 0).unwrap(),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::DE), "301/23456").unwrap()
        );
    }

    #[test]
    fn parse_4() {
        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 81,
                    source: country::Source::Plus,
                },

                national: NationalNumber::new(2345, 0,).unwrap(),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::JP), "+81 *2345").unwrap()
        );
    }

    #[test]
    fn parse_5() {
        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 64,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(12, 0,).unwrap(),

                extension: None,
                carrier: None,
            },
            parser::parse(Some(country::NZ), "12").unwrap()
        );
    }

    #[test]
    fn parse_6() {
        assert_eq!(
            PhoneNumber {
                code: country::Code {
                    value: 55,
                    source: country::Source::Default,
                },

                national: NationalNumber::new(3121286979, 0).unwrap(),

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

    #[rstest]
    #[case(country::RU, "+78005553535", 7, 8005553535)]
    #[case(country::RU, "88005553535", 7, 8005553535)]
    #[case(country::BY, "+375800111111", 375, 800111111)]
    fn issue_76(
        #[case] country: country::Id,
        #[case] number: &'static str,
        #[case] code: u16,
        #[case] national: u64,
    ) {
        // A national number that happens to start with the trunk prefix must
        // not have that prefix stripped when the result would be invalid.
        let parsed = parser::parse(Some(country), number).unwrap();
        assert_eq!(parsed.code().value(), code, "{number}");
        assert_eq!(parsed.national().value(), national, "{number}");
    }

    #[test]
    fn issue_29() {
        // A leading "00" is treated as an international dialling prefix even
        // when no reference region is supplied.
        let parsed = parser::parse(None, "0032474123456").unwrap();
        assert_eq!(parsed.code().value(), 32);
        assert_eq!(parsed.national().value(), 474123456);
        assert_eq!(parsed.code().source(), Source::Idd);
    }

    #[test]
    fn issue_30() {
        // A fully-qualified international number must round-trip regardless of
        // the reference country: the country code came from the number, so the
        // reference country's national prefix must not be stripped.
        let parsed = parser::parse(Some(country::US), "+33142764978").unwrap();
        assert_eq!(parsed.code().value(), 33);
        assert_eq!(parsed.national().value(), 142764978);
        assert_eq!(
            parser::parse(Some(country::US), "+33142764978").unwrap(),
            parser::parse(Some(country::FR), "+33142764978").unwrap()
        );
    }

    #[test]
    fn issue_100() {
        // A number with fewer than three letters is not a vanity number, so
        // stray letters are dropped rather than mapped to digits.
        let parsed = parser::parse(None, "+3367a829916").unwrap();
        assert_eq!(parsed.code().value(), 33);
        assert_eq!(parsed.national().value(), 67829916);

        // A genuine vanity number (>= 3 letters) still maps letters to digits.
        let vanity = parser::parse(Some(country::US), "1800FLOWERS").unwrap();
        assert_eq!(vanity.national().value(), 8003569377);
    }

    #[test]
    fn issue_68() {
        // A national number that begins with the same digits as the reference
        // country's calling code must not have those digits stripped.
        let parsed = parser::parse(Some(country::IT), "3912312312").unwrap();
        assert_eq!(parsed.code().value(), 39);
        assert_eq!(parsed.national().value(), 3912312312);
    }

    #[test]
    fn advisory_1() {
        let res = parser::parse(None, ".;phone-context=");
        assert!(res.is_err(), "{res:?}");
    }

    #[test]
    fn advisory_2() {
        let res = parser::parse(None, "+dwPAA;phone-context=AA");
        assert!(res.is_err(), "{res:?}");
    }

    #[test]
    fn email() {
        let res = parser::parse(Some(country::US), "someletters1110@gmail.com");
        assert!(res.is_err(), "{res:?}");
    }
}
