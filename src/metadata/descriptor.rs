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

use regex::Regex;

#[derive(Clone, Debug)]
pub struct Descriptor {
	/// The national_number_pattern is the pattern that a valid national
	/// significant number would match. This specifies information such as its
	/// total length and leading digits.
	pub(crate) national_number: Option<Regex>,

	/// The possible_number_pattern represents what a potentially valid phone
	/// number for this region may be written as. This is a superset of the
	/// national_number_pattern above and includes numbers that have the area code
	/// omitted. Typically the only restrictions here are in the number of digits.
	///
	/// This could be used to highlight tokens in a text that may be a phone
	/// number, or to quickly prune numbers that could not possibly be a phone
	/// number for this locale.
	pub(crate) possible_number: Option<Regex>,

	/// These represent the lengths a phone number from this region can be. They
	/// will be sorted from smallest to biggest. Note that these lengths are for
	/// the full number, without country calling code or national prefix. For
	/// example, for the Swiss number +41789270000, in local format 0789270000,
	/// this would be 9.
	///
	/// This could be used to highlight tokens in a text that may be a phone
	/// number, or to quickly prune numbers that could not possibly be a phone
	/// number for this locale.
	pub(crate) possible_length: Vec<u16>,

	/// These represent the lengths that only local phone numbers (without an
	/// area code) from this region can be. They will be sorted from smallest to
	/// biggest. For example, since the American number 456-1234 may be locally
	/// diallable, although not diallable from outside the area, 7 could be a
	/// possible value.  This could be used to highlight tokens in a text that
	/// may be a phone number.
	///
	/// To our knowledge, area codes are usually only relevant for some
	/// fixed-line and mobile numbers, so this field should only be set for those
	/// types of numbers (and the general description) - however there are
	/// exceptions for NANPA countries.
	///
	/// This data is used to calculate whether a number could be a possible
	/// number for a particular type.
	pub(crate) possible_local_length: Vec<u16>,

	/// An example national significant number for the specific type. It should
	/// not contain any formatting information.
	pub(crate) example: Option<String>,
}
