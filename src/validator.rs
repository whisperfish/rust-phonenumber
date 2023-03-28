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

use either::*;

use crate::consts;
use crate::country;
use crate::metadata::{Database, Metadata, DATABASE};
use crate::parser;
use crate::parser::helper::Number as ParseNumber;
use crate::phone_number::{PhoneNumber, Type};

/// Possible outcomes when testing if a `PhoneNumber` is possible.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Validation {
    /// The number length matches that of valid numbers for this region.
    IsPossible,

    /// The number length matches that of local numbers for this region only
    /// (i.e. numbers that may be able to be dialled within an area, but do not
    /// have all the information to be dialled from anywhere inside or outside
    /// the country).
    IsPossibleLocalOnly,

    /// The number has an invalid country calling code.
    InvalidCountryCode,

    /// The number is shorter than all valid numbers for this region.
    TooShort,

    /// The number is longer than the shortest valid numbers for this region,
    /// shorter than the longest valid numbers for this region, and does not
    /// itself have a number length that matches valid numbers for this region.
    InvalidLength,

    /// The number is longer than all valid numbers for this region.
    TooLong,
}

impl Validation {
    /// Whether it's a possible number.
    pub fn is_possible(&self) -> bool {
        match *self {
            Validation::IsPossible | Validation::IsPossibleLocalOnly => true,

            _ => false,
        }
    }

    /// Whether it's an invalid number.
    pub fn is_invalid(&self) -> bool {
        match *self {
            Validation::InvalidCountryCode
            | Validation::TooShort
            | Validation::InvalidLength
            | Validation::TooLong => true,

            _ => false,
        }
    }

    /// Whether the length is invalid.
    pub fn is_invalid_length(&self) -> bool {
        match *self {
            Validation::TooShort | Validation::InvalidLength | Validation::TooLong => true,

            _ => false,
        }
    }
}

/// Check if the provided string is a viable phone number.
pub fn is_viable<S: AsRef<str>>(string: S) -> bool {
    let string = string.as_ref();

    if string.len() < consts::MIN_LENGTH_FOR_NSN {
        return false;
    }

    parser::valid::phone_number(string).is_ok()
}

/// Check if the phone number is valid.
pub fn is_valid(number: &PhoneNumber) -> bool {
    is_valid_with(&*DATABASE, number)
}

/// Check if the phone number is valid with the given `Database`.
pub fn is_valid_with(database: &Database, number: &PhoneNumber) -> bool {
    let code = number.country().code();
    let national = number.national.to_string();
    let source = try_opt!(false; source_for(database, code, &national));
    let meta = try_opt!(false; match source {
        Left(region) =>
            database.by_id(region.as_ref()),

        Right(code) =>
            database.by_code(&code).and_then(|m| m.into_iter().next()),
    });

    number_type(meta, &national) != Type::Unknown
}

pub fn length(meta: &Metadata, number: &ParseNumber, kind: Type) -> Validation {
    let desc = if let Some(desc) = meta.descriptors().get(kind) {
        desc
    } else {
        return Validation::InvalidLength;
    };

    let length = number.national.len() as u16;
    let local = &desc.possible_local_length[..];
    let possible = if desc.possible_length.is_empty() {
        &desc.possible_length[..]
    } else {
        &meta.descriptors.general.possible_length[..]
    };

    if possible.is_empty() {
        return Validation::InvalidLength;
    }

    let minimum = possible[0];

    if local.contains(&length) {
        Validation::IsPossibleLocalOnly
    } else if length == minimum {
        Validation::IsPossible
    } else if length < minimum {
        Validation::TooShort
    } else if length > *possible.last().unwrap() {
        Validation::TooLong
    } else if possible.contains(&length) {
        Validation::IsPossible
    } else {
        Validation::InvalidLength
    }
}

/// Find the metadata source.
pub fn source_for(
    database: &Database,
    code: u16,
    national: &str,
) -> Option<Either<country::Id, u16>> {
    let regions = try_opt!(None; database.region(&code));

    if regions.len() == 1 {
        return if regions[0] == "001" {
            Some(Right(code))
        } else {
            match regions[0].parse() {
                Ok(value) => Some(Left(value)),
                Err(_) => None,
            }
        };
    }

    for region in regions {
        let meta = database.by_id(region).unwrap();

        if let Some(pattern) = meta.leading_digits.as_ref() {
            if let Some(index) = pattern.find(national) {
                if index.start() == 0 {
                    return Some(Left(region.parse().unwrap()));
                }
            }
        } else if number_type(meta, national) != Type::Unknown {
            return Some(Left(region.parse().unwrap()));
        }
    }

    None
}

pub fn number_type(meta: &Metadata, value: &str) -> Type {
    if !meta.descriptors.general.is_match(value) {
        return Type::Unknown;
    }

    if meta
        .descriptors
        .premium_rate
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::PremiumRate;
    }

    if meta
        .descriptors
        .toll_free
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::TollFree;
    }

    if meta
        .descriptors
        .shared_cost
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::SharedCost;
    }

    if meta
        .descriptors
        .voip
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::Voip;
    }

    if meta
        .descriptors
        .personal_number
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::PersonalNumber;
    }

    if meta
        .descriptors
        .pager
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::Pager;
    }

    if meta
        .descriptors
        .uan
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::Uan;
    }

    if meta
        .descriptors
        .voicemail
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::Voicemail;
    }

    if meta
        .descriptors
        .fixed_line
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        if meta
            .descriptors
            .fixed_line
            .as_ref()
            .map(|d| d.national_number.as_str())
            == meta
                .descriptors
                .mobile
                .as_ref()
                .map(|d| d.national_number.as_str())
        {
            return Type::FixedLineOrMobile;
        }

        if meta
            .descriptors
            .mobile
            .as_ref()
            .map(|d| d.is_match(value))
            .unwrap_or(false)
        {
            return Type::FixedLineOrMobile;
        }

        return Type::FixedLine;
    }

    if meta
        .descriptors
        .mobile
        .as_ref()
        .map(|d| d.is_match(value))
        .unwrap_or(false)
    {
        return Type::Mobile;
    }

    Type::Unknown
}

#[cfg(test)]
mod test {
    use crate::country;
    use crate::parser;
    use crate::validator;

    #[test]
    fn validate() {
        assert!(validator::is_valid(
            &parser::parse(Some(country::US), "+1 6502530000").unwrap()
        ));

        assert!(validator::is_valid(
            &parser::parse(Some(country::IT), "+39 0236618300").unwrap()
        ));

        assert!(validator::is_valid(
            &parser::parse(Some(country::GB), "+44 7912345678").unwrap()
        ));

        assert!(validator::is_valid(
            &parser::parse(None, "+800 12345678").unwrap()
        ));

        assert!(validator::is_valid(
            &parser::parse(None, "+979 123456789").unwrap()
        ));

        assert!(validator::is_valid(
            &parser::parse(None, "+64 21387835").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+1 2530000").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+39 023661830000").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+44 791234567").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+49 1234").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+64 3316005").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+3923 2366").unwrap()
        ));

        assert!(!validator::is_valid(
            &parser::parse(None, "+800 123456789").unwrap()
        ));
    }
}
