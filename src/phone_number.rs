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
use crate::country;
use crate::error;
use crate::extension::Extension;
use crate::formatter;
use crate::metadata::{Database, Metadata, DATABASE};
use crate::national_number::NationalNumber;
use crate::parser;
use crate::validator;
use either::*;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// A phone number.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Debug)]
pub struct PhoneNumber {
    /// The country calling code for this number, as defined by the International
    /// Telecommunication Union (ITU). For example, this would be 1 for NANPA
    /// countries, and 33 for France.
    pub(crate) code: country::Code,

    /// The National (significant) Number, as defined in International
    /// Telecommunication Union (ITU) Recommendation E.164, without any leading
    /// zero. The leading-zero is stored separately if required, since this is an
    /// uint64 and hence cannot store such information. Do not use this field
    /// directly: if you want the national significant number, call the
    /// getNationalSignificantNumber method of PhoneNumberUtil.
    ///
    /// For countries which have the concept of an "area code" or "national
    /// destination code", this is included in the National (significant) Number.
    /// Although the ITU says the maximum length should be 15, we have found
    /// longer numbers in some countries e.g. Germany.  Note that the National
    /// (significant) Number does not contain the National (trunk) prefix.
    /// Obviously, as a uint64, it will never contain any formatting (hyphens,
    /// spaces, parentheses), nor any alphanumeric spellings.
    pub(crate) national: NationalNumber,

    /// Extension is not standardized in ITU recommendations, except for being
    /// defined as a series of numbers with a maximum length of 40 digits. It is
    /// defined as a string here to accommodate for the possible use of a leading
    /// zero in the extension (organizations have complete freedom to do so, as
    /// there is no standard defined). Other than digits, some other dialling
    /// characters such as "," (indicating a wait) may be stored here.
    pub(crate) extension: Option<Extension>,

    /// The carrier selection code that is preferred when calling this phone
    /// number domestically. This also includes codes that need to be dialed in
    /// some countries when calling from landlines to mobiles or vice versa. For
    /// example, in Columbia, a "3" needs to be dialed before the phone number
    /// itself when calling from a mobile phone to a domestic landline phone and
    /// vice versa.
    ///
    /// Note this is the "preferred" code, which means other codes may work as
    /// well.
    pub(crate) carrier: Option<Carrier>,
}

/// Wrapper to make it easier to access information about the country of a
/// phone number.
pub struct Country<'a>(&'a PhoneNumber);

/// The phone number type.
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    /// Fixed line numbers.
    FixedLine,

    /// Mobile numbers.
    Mobile,

    /// In some regions (e.g. the USA), it is impossible to distinguish between
    /// fixed-line and mobile numbers by looking at the phone number itself.
    FixedLineOrMobile,

    /// Freephone lines.
    TollFree,

    /// Premium rate lines.
    PremiumRate,

    /// The cost of this call is shared between the caller and the recipient, and
    /// is hence typically less than [`PremiumRate`](Self::PremiumRate) calls. See
    /// [Shared-cost Service](http://en.wikipedia.org/wiki/Shared-cost_service)
    /// for more information.
    SharedCost,

    /// A personal number is associated with a particular person, and may be
    /// routed to either a [`Mobile`](Self::Mobile) or
    /// [`FixedLine`](Self::FixedLine) number. See
    /// [Personal Numbers](http://en.wikipedia.org/wiki/Personal_Numbers) for more
    /// information.
    PersonalNumber,

    /// Voice over IP numbers. This includes TSoIP (Telephony Service over IP).
    Voip,

    /// A pager number.
    Pager,

    /// Used for "Universal Access Numbers" or "Company Numbers". They may be
    /// further routed to specific offices, but allow one number to be used for a
    /// company.
    Uan,

    /// Emergency numbers.
    Emergency,

    /// Used for "Voice Mail Access Numbers".
    Voicemail,

    /// An abbreviated number, such as short codes like "10000".
    ShortCode,

    StandardRate,

    Carrier,

    NoInternational,

    /// A phone number is of type UNKNOWN when it does not fit any of the known
    /// patterns for a specific region.
    Unknown,
}

impl FromStr for PhoneNumber {
    type Err = error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse(None, s)
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl PhoneNumber {
    /// Get information about the country for the phone number.
    pub fn country(&self) -> Country<'_> {
        Country(self)
    }

    /// Get the country code.
    pub fn code(&self) -> &country::Code {
        &self.code
    }

    /// Get the national number.
    pub fn national(&self) -> &NationalNumber {
        &self.national
    }

    /// Get the extension.
    pub fn extension(&self) -> Option<&Extension> {
        self.extension.as_ref()
    }

    /// Get the carrier.
    pub fn carrier(&self) -> Option<&Carrier> {
        self.carrier.as_ref()
    }

    /// Prepare a formatter for this `PhoneNumber`.
    ///
    /// # Example
    ///
    /// ```
    /// use phonenumber::{self, country, Mode};
    ///
    /// let number = phonenumber::parse(Some(country::DE), "301/23456").unwrap()
    ///     .format().mode(Mode::National).to_string();
    ///
    /// assert_eq!("030 123456", number);
    /// ```
    pub fn format(&self) -> formatter::Formatter<'_, 'static, 'static> {
        formatter::format(self)
    }

    /// Prepare a formatter for this `PhoneNumber` with the given `Database`.
    pub fn format_with<'n, 'd>(
        &'n self,
        database: &'d Database,
    ) -> formatter::Formatter<'n, 'd, 'static> {
        formatter::format_with(database, self)
    }

    /// Get the metadata that applies to this phone number from the given
    /// database.
    pub fn metadata<'a>(&self, database: &'a Database) -> Option<&'a Metadata> {
        match validator::source_for(database, self.code.value(), &self.national.to_string())? {
            Left(region) => database.by_id(region.as_ref()),
            Right(code) => database.by_code(&code).and_then(|m| m.into_iter().next()),
        }
    }

    /// Check if the phone number is valid.
    pub fn is_valid(&self) -> bool {
        validator::is_valid(self)
    }

    /// Check if the phone number is valid with the given `Database`.
    pub fn is_valid_with(&self, database: &Database) -> bool {
        validator::is_valid_with(database, self)
    }

    /// Determine the [`Type`] of the phone number.
    pub fn number_type(&self, database: &Database) -> Type {
        match self.metadata(database) {
            Some(metadata) => validator::number_type(metadata, &self.national.value.to_string()),
            None => Type::Unknown,
        }
    }
}

impl<'a> Country<'a> {
    pub fn code(&self) -> u16 {
        self.0.code.value()
    }

    pub fn id(&self) -> Option<country::Id> {
        self.0.metadata(&DATABASE).and_then(|m| m.id().parse().ok())
    }
}

impl<'a> Deref for Country<'a> {
    type Target = country::Code;

    fn deref(&self) -> &Self::Target {
        self.0.code()
    }
}

#[cfg(test)]
mod test {
    use crate::country::{self, Id::*};
    use crate::metadata::DATABASE;
    use crate::Type;
    use crate::{parser, Mode, PhoneNumber};
    use anyhow::Context;
    use rstest::rstest;
    use rstest_reuse::*;

    fn parsed(number: &str) -> PhoneNumber {
        parser::parse(None, number)
            .with_context(|| format!("parsing {number}"))
            .unwrap()
    }

    #[template]
    #[rstest]
    #[case(parsed("+80012340000"), None, Type::TollFree)]
    #[case(parsed("+61406823897"), Some(AU), Type::Mobile)]
    #[case(parsed("+611900123456"), Some(AU), Type::PremiumRate)]
    #[case(parsed("+32474091150"), Some(BE), Type::Mobile)]
    #[case(parsed("+34666777888"), Some(ES), Type::Mobile)]
    #[case(parsed("+34612345678"), Some(ES), Type::Mobile)]
    #[case(parsed("+441212345678"), Some(GB), Type::FixedLine)]
    #[case(parsed("+13459492311"), Some(KY), Type::FixedLine)]
    #[case(parsed("+16137827274"), Some(CA), Type::FixedLineOrMobile)]
    #[case(parsed("+1 520 878 2491"), Some(US), Type::FixedLineOrMobile)]
    #[case(parsed("+1-520-878-2491"), Some(US), Type::FixedLineOrMobile)]
    // Case for issues
    // https://github.com/whisperfish/rust-phonenumber/issues/46 and
    // https://github.com/whisperfish/rust-phonenumber/issues/47
    // #[case(parsed("+1 520-878-2491"), US)]
    fn phone_numbers(
        #[case] number: PhoneNumber,
        #[case] country: Option<country::Id>,
        #[case] r#type: Type,
    ) {
    }

    #[apply(phone_numbers)]
    fn country_id(
        #[case] number: PhoneNumber,
        #[case] country: Option<country::Id>,
        #[case] _type: Type,
    ) -> anyhow::Result<()> {
        assert_eq!(country, number.country().id());

        Ok(())
    }

    #[apply(phone_numbers)]
    #[ignore]
    // Format-parse roundtrip
    fn round_trip_parsing(
        #[case] number: PhoneNumber,
        #[case] country: Option<country::Id>,
        #[case] _type: Type,
        #[values(Mode::International, Mode::E164, Mode::Rfc3966, Mode::National)] mode: Mode,
    ) -> anyhow::Result<()> {
        let country_hint = if mode == Mode::National {
            country
        } else {
            None
        };

        let formatted = number.format().mode(mode).to_string();
        let parsed = parser::parse(country_hint, &formatted).with_context(|| {
            format!("parsing {number} after formatting in {mode:?} mode as {formatted}")
        })?;

        // impl Eq for PhoneNumber does not consider differently parsed phone numbers to be equal.
        // E.g., parsing 047409110 with BE country hint is the same phone number as +32474091150,
        // but Eq considers them different.
        assert_eq!(number, parsed);

        Ok(())
    }

    #[apply(phone_numbers)]
    fn number_type(
        #[case] number: PhoneNumber,
        #[case] _country: Option<country::Id>,
        #[case] r#type: Type,
    ) {
        assert_eq!(r#type, number.number_type(&DATABASE));
    }
}
