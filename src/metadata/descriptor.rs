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

use regex_cache::CachedRegex;

/// Description of a phone number to parse.
#[derive(Clone, Debug)]
pub struct Descriptor {
    pub(crate) national_number: CachedRegex,

    pub(crate) possible_length: Vec<u16>,
    pub(crate) possible_local_length: Vec<u16>,

    pub(crate) example: Option<String>,
}

impl Descriptor {
    /// The national number is the pattern that a valid national significant
    /// number would match. This specifies information such as its total length
    /// and leading digits.
    pub fn national_number(&self) -> &CachedRegex {
        &self.national_number
    }

    /// These represent the lengths a phone number from this region can be. They
    /// will be sorted from smallest to biggest. Note that these lengths are for
    /// the full number, without country calling code or national prefix. For
    /// example, for the Swiss number +41789270000, in local format 0789270000,
    /// this would be 9.
    ///
    /// This could be used to highlight tokens in a text that may be a phone
    /// number, or to quickly prune numbers that could not possibly be a phone
    /// number for this locale.
    pub fn possible_length(&self) -> &[u16] {
        &self.possible_length
    }

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
    pub fn possible_local_length(&self) -> &[u16] {
        &self.possible_local_length
    }

    /// An example national significant number for the specific type. It should
    /// not contain any formatting information.
    pub fn example(&self) -> Option<&str> {
        self.example.as_ref().map(AsRef::as_ref)
    }

    /// Check if the descriptor matches the given national number.
    pub fn is_match(&self, value: &str) -> bool {
        if !self.possible_length.is_empty() && !self.possible_length.contains(&(value.len() as u16))
        {
            return false;
        }

        self.national_number
            .find(value)
            .map(|m| m.start() == 0)
            .unwrap_or(false)
    }
}
