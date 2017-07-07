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

use metadata::{DATABASE, Database, Metadata};
use country_code::Country;
use phone_number::{Type, PhoneNumber};

pub fn validate(number: &PhoneNumber) -> bool {
	validate_with(&*DATABASE, number)
}

pub fn validate_with(database: &Database, number: &PhoneNumber) -> bool {
	let code     = number.country_code().value();
	let national = number.national_number().to_string();
	let source   = try_opt!(bool; source_for(database, code, &national));
	let meta     = try_opt!(bool; match source {
		Left(region) =>
			database.by_id(region.as_ref()),

		Right(code) =>
			database.by_code(&code).and_then(|m| m.into_iter().next()),
	});

	number_type(meta, &national) != Type::Unknown
}

/// Find the metadata source.
pub fn source_for(database: &Database, code: u16, national: &str) -> Option<Either<Country, u16>> {
	let regions = try_opt!(database.region(&code));

	if regions.len() == 1 {
		if regions[0] == "001" {
			return Some(Right(code));
		}
		else {
			return Some(Left(regions[0].parse().unwrap()));
		}
	}

	for region in regions {
		let meta = database.by_id(region).unwrap();

		if let Some(pattern) = meta.leading_digits.as_ref() {
			if let Some(index) = pattern.find(&national) {
				if index.start() == 0 {
					return Some(Left(region.parse().unwrap()));
				}
			}
		}
		else if number_type(meta, &national) != Type::Unknown {
			return Some(Left(region.parse().unwrap()));
		}
	}

	None
}

pub fn number_type(meta: &Metadata, value: &str) -> Type {
	if !meta.general.is_match(value) {
		return Type::Unknown;
	}

	if meta.premium_rate.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::PremiumRate;
	}

	if meta.toll_free.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::TollFree;
	}

	if meta.shared_cost.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::SharedCost;
	}

	if meta.voip.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::Voip;
	}

	if meta.personal.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::PersonalNumber;
	}

	if meta.pager.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::Pager;
	}

	if meta.uan.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::Uan;
	}

	if meta.voicemail.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::Voicemail;
	}

	if meta.fixed_line.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		if meta.fixed_line.as_ref().map(|d| d.national_number.as_str()) ==
		   meta.mobile.as_ref().map(|d| d.national_number.as_str())
		{
			return Type::FixedLineOrMobile;
		}

		if meta.mobile.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
			return Type::FixedLineOrMobile;
		}

		return Type::FixedLine;
	}

	if meta.mobile.as_ref().map(|d| d.is_match(value)).unwrap_or(false) {
		return Type::Mobile;
	}

	Type::Unknown
}

#[cfg(test)]
mod test {
	use validator;
	use parser;
	use country_code::Country;

	#[test]
	fn validate() {
		assert!(validator::validate(&parser::parse(
			Some(Country::US), "+1 6502530000").unwrap()));

		assert!(validator::validate(&parser::parse(
			Some(Country::IT), "+39 0236618300").unwrap()));

		assert!(validator::validate(&parser::parse(
			Some(Country::GB), "+44 7912345678").unwrap()));

		assert!(validator::validate(&parser::parse(
			None, "+800 12345678").unwrap()));

		assert!(validator::validate(&parser::parse(
			None, "+979 123456789").unwrap()));

		assert!(validator::validate(&parser::parse(
			None, "+64 21387835").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+1 2530000").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+39 023661830000").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+44 791234567").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+49 1234").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+64 3316005").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+3923 2366").unwrap()));

		assert!(!validator::validate(&parser::parse(
			None, "+800 123456789").unwrap()));
	}
}
