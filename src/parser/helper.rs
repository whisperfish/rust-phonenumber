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

use crate::consts;
use crate::country;
use crate::error;
use crate::metadata::{Database, Metadata};
use crate::phone_number::Type;
use crate::validator;
use fnv::FnvHashMap;
use nom::{
    character::complete::*,
    combinator::*,
    error::{make_error, ErrorKind},
    multi::*,
    AsChar, IResult,
};
use regex_cache::CachedRegex;
use std::borrow::Cow;

macro_rules! parse {
	($input:ident => ) => ();

	($input:ident => $combinator:expr) => (
		$combinator($input)
	);

	($input:ident => let $name:ident = $combinator:expr; $($rest:tt)*) => (
		parse!(@ $input => let $name = $combinator);
		parse!($input => $($rest)*)
	);

	($input:ident => $combinator:expr; $($rest:tt)*) => (
		parse!(@ $input => $combinator);
		parse!($input => $($rest)*)
	);

	($input:ident => $combinator:expr) => (
		$combinator($input)
	);

	(@ $input:ident => let $name:ident = $combinator:expr) => (
		let ($input, $name) = $combinator($input)?;
	);

	(@ $input:ident => $combinator:expr) => (
		let ($input, _) = $combinator($input)?;
	);
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Number<'a> {
    pub country: country::Source,
    pub national: Cow<'a, str>,
    pub prefix: Option<Cow<'a, str>>,
    pub extension: Option<Cow<'a, str>>,
    pub carrier: Option<Cow<'a, str>>,
}

pub fn ieof(i: &str) -> IResult<&str, ()> {
    if i.is_empty() {
        Ok((i, ()))
    } else {
        Err(nom::Err::Error(make_error(i, ErrorKind::LengthValue)))
    }
}

pub fn punctuation(i: &str) -> IResult<&str, char> {
    one_of("-x\u{2010}\u{2011}\u{2012}\u{2013}\u{2014}\u{2015}\u{2212}\u{30FC}\u{FF0D}-\u{FF0F} \u{00A0}\u{00AD}\u{200B}\u{2060}\u{3000}()\u{FF08}\u{FF09}\u{FF3B}\u{FF3D}.[]/~\u{2053}\u{223C}\u{FF5E}")(i)
}

pub fn alpha(i: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(i)
}

// TODO: Extend with Unicode digits.
pub fn digit(i: &str) -> IResult<&str, char> {
    one_of("0123456789")(i)
}

pub fn plus(i: &str) -> IResult<&str, char> {
    one_of("+\u{FF0B}")(i)
}

pub fn star(i: &str) -> IResult<&str, char> {
    one_of("*")(i)
}

pub fn ignore_plus(i: &str) -> IResult<&str, &str> {
    recognize(many1(plus))(i)
}

/// Attempts to extract a possible number from the string passed in. This
/// currently strips all leading characters that cannot be used to start a
/// phone number. Characters that can be used to start a phone number are
/// defined in the `VALID_START_CHAR` regex. If none of these characters are
/// found in the number passed in, an empty string is returned. This function
/// also attempts to strip off any alternative extensions or endings if two or
/// more are present, such as in the case of: (530) 583-6985 x302/x2303. The
/// second extension here makes this actually two phone numbers, (530) 583-6985
/// x302 and (530) 583-6985 x2303. We remove the second extension so that the
/// first number is parsed correctly.
pub fn extract(value: &str) -> IResult<&str, &str> {
    let (mut result, start) = if let Some(index) = consts::VALID_START_CHAR.find(value) {
        (&value[index.start()..], index.start())
    } else {
        return Err(nom::Err::Error(make_error(value, ErrorKind::RegexpMatch)));
    };

    if let Some(trailing) = consts::UNWANTED_END_CHARS.find(result) {
        result = &result[..trailing.start()];
    }

    if let Some(extra) = consts::SECOND_NUMBER_START.find(result) {
        result = &result[..extra.start()];
    }

    if result.is_empty() {
        Err(nom::Err::Error(make_error(value, ErrorKind::RegexpMatch)))
    } else {
        Ok((&value[start + result.len()..], result))
    }
}

/// Parse and insert the proper country code.
pub fn country_code<'a>(
    database: &Database,
    country: Option<country::Id>,
    mut number: Number<'a>,
) -> Result<Number<'a>, error::Parse> {
    let idd = country
        .and_then(|c| database.by_id(c.as_ref()))
        .and_then(|m| m.international_prefix.as_ref());

    number = international_prefix(idd, number);

    match number.country {
        // The country source was found from the initial PLUS or it was extract
        // from the number already.
        country::Source::Plus | country::Source::Idd | country::Source::Number => {
            if number.national.len() <= consts::MIN_LENGTH_FOR_NSN {
                return Err(error::Parse::TooShortNsn);
            }

            // If the prefix was already extracted, check it is valid.
            if number.prefix.is_some() {
                let prefix = number.prefix.as_ref().unwrap().parse()?;

                if database.by_code(&prefix).is_none() {
                    return Err(error::Parse::InvalidCountryCode);
                } else {
                    return Ok(number);
                }
            } else {
                // Check the possible country code does not start with a 0 since those
                // are invalid.
                if number.national.starts_with('0') {
                    return Err(error::Parse::InvalidCountryCode);
                }

                // Try to find the first available country code.
                for len in 1..consts::MAX_LENGTH_FOR_COUNTRY_CODE + 1 {
                    let code = number.national[..len].parse().unwrap();

                    if database.by_code(&code).is_some() {
                        number.national = trim(number.national, len);
                        number.prefix = Some(code.to_string().into());

                        return Ok(number);
                    }
                }
            }
        }

        country::Source::Default => {
            if let Some(country) = country {
                let meta = database.by_id(country.as_ref()).unwrap();
                let code = meta.country_code.to_string();

                if number.national.starts_with(&code)
                    && (!meta.descriptors().general().is_match(&number.national)
                        || !validator::length(meta, &number, Type::Unknown).is_possible())
                {
                    number.country = country::Source::Number;
                    number.national = trim(number.national, code.len());
                }

                number.prefix = Some(code.into());

                return Ok(number);
            }
        }
    }

    Err(error::Parse::InvalidCountryCode)
}

/// Strip the IDD from a `Number`, update the country code source, and
/// normalize it.
///
/// Note that since the IDD comes from a passed default region, we can find the
/// country code from the given default if the country source is from the IDD.
pub fn international_prefix<'a>(idd: Option<&CachedRegex>, mut number: Number<'a>) -> Number<'a> {
    // If there's a prefix already, i.e. RFC3966, just change the country source.
    if number.prefix.is_some() {
        number.country = country::Source::Plus;
        return normalize(number, &consts::ALPHA_PHONE_MAPPINGS);
    }

    // Ignore any leading PLUS characters.
    let start = ignore_plus(&number.national)
        .map(|(_, s)| s.len()) ////YANN: CHECK
        .unwrap_or(0);

    // If there are any pluses, strip them and change the country source.
    if start != 0 {
        number.country = country::Source::Plus;
        number.national = trim(number.national, start);
        number = normalize(number, &consts::ALPHA_PHONE_MAPPINGS);

        if !idd
            .and_then(|re| re.find(&number.national))
            .map(|m| m.start() == 0)
            .unwrap_or(false)
        {
            return number;
        }
    } else {
        number.country = country::Source::Default;
        number = normalize(number, &consts::ALPHA_PHONE_MAPPINGS);
    }

    // Check if the IDD pattern matches.
    let index = idd
        .and_then(|re| re.find(&number.national))
        .map(|m| (m.start(), m.end()));

    // If it does.
    if let Some((start, end)) = index {
        // Check it starts at the beginning and the next digit after the IDD is not
        // a 0, since that's invalid.
        if start == 0 && !number.national[end..].starts_with('0') {
            if number.country != country::Source::Plus {
                number.country = country::Source::Idd;
            }

            number.national = trim(number.national, end);
        }
    }

    number
}

/// Strip national prefix and extract carrier.
pub fn national_number<'a>(meta: &Metadata, mut number: Number<'a>) -> Number<'a> {
    let transform = meta.national_prefix_transform_rule.as_ref();
    let parsing = if let Some(re) = meta.national_prefix_for_parsing.as_ref() {
        re
    } else {
        if let Some(prefix) = meta.national_prefix.as_ref() {
            if number.national.starts_with(prefix) {
                number.national = trim(number.national, prefix.len());
            }
        }

        return number;
    };

    let index = parsing.find(&number.national).map(|m| (m.start(), m.end()));

    if index.is_none() {
        return number;
    }

    let (start, end) = index.unwrap();
    if start != 0 {
        return number;
    }

    let viable = meta.descriptors.general.is_match(&number.national);
    let groups = parsing.captures_len();

    let (first, last) = parsing
        .captures(&number.national)
        .map(|c| {
            (
                c.get(1).map(|m| m.as_str().to_owned()),
                c.get(c.len() - 1).map(|m| m.as_str().to_owned()),
            )
        })
        .unwrap();

    if transform.is_none() || last.is_none() {
        if viable && !meta.descriptors.general.is_match(&number.national[start..]) {
            return number;
        }

        number.carrier = last.filter(|_| groups > 0).map(Into::into);

        number.national = trim(number.national, end);
    } else if let Some(transform) = transform {
        let transformed = parsing.replace(&number.national, transform).into_owned();

        if viable && !meta.descriptors.general.is_match(&transformed) {
            return number;
        }

        number.carrier = Some(first.unwrap().into());
        number.national = transformed.into();
    }

    number
}

/// Normalize a given `Number`, replacing the characters matching the mappings
/// and converting any Unicode non-decimal digits into their decimal
/// counterpart.
///
/// Note if the `Number` is already normalized it does not get modified.
pub fn normalize<'a>(mut number: Number<'a>, mappings: &FnvHashMap<char, char>) -> Number<'a> {
    fn act<'a>(value: Cow<'a, str>, mappings: &FnvHashMap<char, char>) -> Cow<'a, str> {
        let mut owned = None;
        {
            let mut chars = value.char_indices();

            while let Some((start, ch)) = chars.next() {
                if !ch.is_dec_digit() {
                    let mut string = String::from(&value[..start]);

                    if let Some(ch) = ch.as_dec_digit() {
                        string.push(ch);
                    } else if let Some(&ch) = mappings.get(&ch) {
                        string.push(ch);
                    }

                    for (_, ch) in chars.by_ref() {
                        if let Some(ch) = ch.as_dec_digit() {
                            string.push(ch);
                        } else if let Some(&ch) = mappings.get(&ch) {
                            string.push(ch);
                        }
                    }

                    owned = Some(string);
                }
            }
        }

        owned.map(Cow::Owned).unwrap_or(value)
    }

    number.national = act(number.national, mappings);
    number.prefix = number.prefix.map(|p| act(p, mappings));
    number.extension = number.extension.map(|e| act(e, mappings));

    number
}

pub fn trim(value: Cow<'_, str>, start: usize) -> Cow<'_, str> {
    match value {
        Cow::Borrowed(value) => Cow::Borrowed(&value[start..]),

        Cow::Owned(mut value) => {
            value.drain(..start);
            Cow::Owned(value)
        }
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait AsCharExt {
    fn is_wide_digit(self) -> bool;

    fn as_dec_digit(self) -> Option<char>;
}

impl<T: AsChar> AsCharExt for T {
    fn is_wide_digit(self) -> bool {
        self.as_char().is_ascii_digit()
    }

    fn as_dec_digit(self) -> Option<char> {
        let ch = self.as_char();

        if ch.is_dec_digit() {
            return Some(ch);
        }

        match ch {
            '٠' | '۰' | '߀' | '०' | '০' | '੦' | '૦' | '୦' | '௦' | '౦' | '೦' | '൦' | '๐' | '໐'
            | '０' => Some('0'),

            '١' | '۱' | '߁' | '१' | '১' | '੧' | '૧' | '୧' | '௧' | '౧' | '೧' | '൧' | '๑' | '໑'
            | '１' => Some('1'),

            '٢' | '۲' | '߂' | '२' | '২' | '੨' | '૨' | '୨' | '௨' | '౨' | '೨' | '൨' | '๒' | '໒'
            | '２' => Some('2'),

            '٣' | '۳' | '߃' | '३' | '৩' | '੩' | '૩' | '୩' | '௩' | '౩' | '೩' | '൩' | '๓' | '໓'
            | '３' => Some('3'),

            '٤' | '۴' | '߄' | '४' | '৪' | '੪' | '૪' | '୪' | '௪' | '౪' | '೪' | '൪' | '๔' | '໔'
            | '４' => Some('4'),

            '٥' | '۵' | '߅' | '५' | '৫' | '੫' | '૫' | '୫' | '௫' | '౫' | '೫' | '൫' | '๕' | '໕'
            | '５' => Some('5'),

            '٦' | '۶' | '߆' | '६' | '৬' | '੬' | '૬' | '୬' | '௬' | '౬' | '೬' | '൬' | '๖' | '໖'
            | '６' => Some('6'),

            '٧' | '۷' | '߇' | '७' | '৭' | '੭' | '૭' | '୭' | '௭' | '౭' | '೭' | '൭' | '๗' | '໗'
            | '７' => Some('7'),

            '٨' | '۸' | '߈' | '८' | '৮' | '੮' | '૮' | '୮' | '௮' | '౮' | '೮' | '൮' | '๘' | '໘'
            | '８' => Some('8'),

            '٩' | '۹' | '߉' | '९' | '৯' | '੯' | '૯' | '୯' | '௯' | '౯' | '೯' | '൯' | '๙' | '໙'
            | '９' => Some('9'),

            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::metadata::DATABASE;
    use crate::parser::helper;
    use crate::parser::helper::*;

    #[test]
    fn punctuation() {
        assert!(helper::punctuation("-").is_ok());
        assert!(helper::punctuation("x").is_ok());
        assert!(helper::punctuation("\u{2015}").is_ok());

        assert!(helper::punctuation("a").is_err());
    }

    #[test]
    fn alpha() {
        assert!(helper::alpha("a").is_ok());
        assert!(helper::alpha("x").is_ok());
        assert!(helper::alpha("Z").is_ok());

        assert!(helper::alpha("2").is_err());
    }

    #[test]
    fn plus() {
        assert!(helper::plus("+").is_ok());
        assert!(helper::plus("\u{FF0B}").is_ok());
        assert!(helper::plus("a").is_err());
    }

    #[test]
    fn extract() {
        // Removes preceding funky punctuation and letters but leaves the rest untouched.
        assert_eq!(
            "0800-345-600",
            helper::extract("Tel:0800-345-600").unwrap().1
        );
        assert_eq!(
            "0800 FOR PIZZA",
            helper::extract("Tel:0800 FOR PIZZA").unwrap().1
        );
        // Should not remove plus sign
        assert_eq!(
            "+800-345-600",
            helper::extract("Tel:+800-345-600").unwrap().1
        );
        // Should recognise wide digits as possible start values.
        assert_eq!(
            "\u{FF10}\u{FF12}\u{FF13}",
            helper::extract("\u{FF10}\u{FF12}\u{FF13}").unwrap().1
        );
        // Dashes are not possible start values and should be removed.
        assert_eq!(
            "\u{FF11}\u{FF12}\u{FF13}",
            helper::extract("Num-\u{FF11}\u{FF12}\u{FF13}").unwrap().1
        );
        // If not possible number present, return empty string.
        assert!(helper::extract("Num-....").is_err());
        // Leading brackets are stripped - these are not used when parsing.
        assert_eq!(
            "650) 253-0000",
            helper::extract("(650) 253-0000").unwrap().1
        );

        // Trailing non-alpha-numeric characters should be removed.
        assert_eq!(
            "650) 253-0000",
            helper::extract("(650) 253-0000..- ..").unwrap().1
        );
        assert_eq!(
            "650) 253-0000",
            helper::extract("(650) 253-0000.").unwrap().1
        );
        // This case has a trailing RTL char.
        assert_eq!(
            "650) 253-0000",
            helper::extract("(650) 253-0000\u{200F}").unwrap().1
        );
    }

    #[test]
    fn country_code() {
        assert_eq!(
            Number {
                country: country::Source::Idd,
                national: "123456789".into(),
                prefix: Some("1".into()),

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::US),
                Number {
                    national: "011112-3456789".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );

        assert_eq!(
            Number {
                country: country::Source::Plus,
                national: "23456789".into(),
                prefix: Some("64".into()),

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::US),
                Number {
                    national: "+6423456789".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );

        assert_eq!(
            Number {
                country: country::Source::Plus,
                national: "12345678".into(),
                prefix: Some("800".into()),

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::US),
                Number {
                    national: "+80012345678".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );

        assert_eq!(
            Number {
                country: country::Source::Default,
                national: "23456789".into(),
                prefix: Some("1".into()),

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::US),
                Number {
                    national: "2345-6789".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );

        assert!(helper::country_code(
            &DATABASE,
            Some(country::US),
            Number {
                national: "0119991123456789".into(),

                ..Default::default()
            }
        )
        .is_err());

        assert_eq!(
            Number {
                national: "6106194466".into(),
                prefix: Some("1".into()),
                country: country::Source::Number,

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::US),
                Number {
                    national: "(1 610) 619 4466".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );

        assert_eq!(
            Number {
                national: "3298888888".into(),
                prefix: Some("39".into()),
                country: country::Source::Number,

                ..Default::default()
            },
            helper::country_code(
                &DATABASE,
                Some(country::IT),
                Number {
                    national: "393298888888".into(),

                    ..Default::default()
                }
            )
            .unwrap()
        );
    }

    #[test]
    fn normalize() {
        // Strips symbols.
        assert_eq!(
            "034562",
            helper::normalize(
                Number {
                    national: "034-56&+#2".into(),
                    ..Default::default()
                },
                &consts::ALPHA_PHONE_MAPPINGS
            )
            .national
        );

        // Converts letters to numbers.
        assert_eq!(
            "034426486479",
            helper::normalize(
                Number {
                    national: "034-I-am-HUNGRY".into(),
                    ..Default::default()
                },
                &consts::ALPHA_PHONE_MAPPINGS
            )
            .national
        );

        // Handles wide numbers.
        assert_eq!(
            "420",
            helper::normalize(
                Number {
                    national: "４2０".into(),
                    ..Default::default()
                },
                &consts::ALPHA_PHONE_MAPPINGS
            )
            .national
        );
    }

    #[test]
    fn international_prefix() {
        assert_eq!(
            Number {
                country: country::Source::Idd,
                national: "45677003898003".into(),

                ..Default::default()
            },
            helper::international_prefix(
                Some(&CachedRegex::new(DATABASE.cache(), "00[39]").unwrap()),
                Number {
                    national: "0034567700-3898003".into(),

                    ..Default::default()
                }
            )
        );

        assert_eq!(
            Number {
                country: country::Source::Idd,
                national: "45677003898003".into(),

                ..Default::default()
            },
            helper::international_prefix(
                Some(&CachedRegex::new(DATABASE.cache(), "00[39]").unwrap()),
                Number {
                    national: "00945677003898003".into(),

                    ..Default::default()
                }
            )
        );

        assert_eq!(
            Number {
                country: country::Source::Idd,
                national: "45677003898003".into(),

                ..Default::default()
            },
            helper::international_prefix(
                Some(&CachedRegex::new(DATABASE.cache(), "00[39]").unwrap()),
                Number {
                    national: "00 9 45677003898003".into(),

                    ..Default::default()
                }
            )
        );

        assert_eq!(
            Number {
                national: "45677003898003".into(),

                ..Default::default()
            },
            helper::international_prefix(
                Some(&CachedRegex::new(DATABASE.cache(), "00[39]").unwrap()),
                Number {
                    national: "45677003898003".into(),

                    ..Default::default()
                }
            )
        );

        assert_eq!(
            Number {
                country: country::Source::Plus,
                national: "45677003898003".into(),

                ..Default::default()
            },
            helper::international_prefix(
                Some(&CachedRegex::new(DATABASE.cache(), "00[39]").unwrap()),
                Number {
                    national: "+45677003898003".into(),

                    ..Default::default()
                }
            )
        );
    }
}
