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

/// Description of a phone number format.
#[derive(Clone, Debug)]
pub struct Format {
    pub(crate) pattern: CachedRegex,
    pub(crate) format: String,

    pub(crate) leading_digits: Vec<CachedRegex>,
    pub(crate) national_prefix: Option<String>,
    pub(crate) national_prefix_optional: bool,
    pub(crate) domestic_carrier: Option<String>,
}

impl Format {
    /// A regex that is used to match the national (significant) number. For
    /// example, the pattern "(20)(\d{4})(\d{4})" will match number "2070313000",
    /// which is the national (significant) number for Google London.
    ///
    /// Note the presence of the parentheses, which are capturing groups what
    /// specifies the grouping of numbers.
    pub fn pattern(&self) -> &CachedRegex {
        &self.pattern
    }

    /// Specifies how the national (significant) number matched by pattern should
    /// be formatted.
    ///
    /// Using the same example as above, format could contain "$1 $2 $3", meaning
    /// that the number should be formatted as "20 7031 3000".
    ///
    /// Each $x are replaced by the numbers captured by group x in the regex
    /// specified by pattern.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// A regex that is used to match a certain number of digits at the beginning
    /// of the national (significant) number. When the match is successful, the
    /// accompanying pattern and format should be used to format this number. For
    /// example, if leading_digits="[1-3]|44", then all the national numbers
    /// starting with 1, 2, 3 or 44 should be formatted using the
    /// accompanying pattern and format.
    ///
    /// The first leadingDigitsPattern matches up to the first three digits of the
    /// national (significant) number; the next one matches the first four digits,
    /// then the first five and so on, until the leadingDigitsPattern can uniquely
    /// identify one pattern and format to be used to format the number.
    ///
    /// In the case when only one formatting pattern exists, no
    /// leading_digits_pattern is needed.
    pub fn leading_digits(&self) -> &[CachedRegex] {
        &self.leading_digits
    }

    /// Specifies how the national prefix ($NP) together with the first group
    /// ($FG) in the national significant number should be formatted in the
    /// NATIONAL format when a national prefix exists for a certain country.
    ///
    /// For example, when this field contains "($NP$FG)", a number from Beijing,
    /// China (whose $NP = 0), which would by default be formatted without
    /// national prefix as 10 1234 5678 in NATIONAL format, will instead be
    /// formatted as (010) 1234 5678; to format it as (0)10 1234 5678, the field
    /// would contain "($NP)$FG". Note $FG should always be present in this field,
    /// but $NP can be omitted. For example, having "$FG" could indicate the
    /// number should be formatted in NATIONAL format without the national prefix.
    ///
    /// This is commonly used to override the rule specified for the territory in
    /// the XML file.
    ///
    /// When this field is missing, a number will be formatted without national
    /// prefix in NATIONAL format. This field does not affect how a number is
    /// formatted in other formats, such as INTERNATIONAL.
    pub fn national_prefix(&self) -> Option<&str> {
        self.national_prefix.as_ref().map(AsRef::as_ref)
    }

    /// Whether the national prefix is optional when formatting.
    pub fn is_national_prefix_optional(&self) -> bool {
        self.national_prefix_optional
    }

    /// Specifies how any carrier code ($CC) together with the first group ($FG)
    /// in the national significant number should be formatted when
    /// formatWithCarrierCode is called, if carrier codes are used for a certain
    /// country.
    pub fn domestic_carrier(&self) -> Option<&str> {
        self.domestic_carrier.as_ref().map(AsRef::as_ref)
    }
}
