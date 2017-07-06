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

use std::io::Write;
use std::fmt;
use std::str::FromStr;

use error::{Error, Result};
use country_code::CountryCode;
use national_number::NationalNumber;
use extension::Extension;
use metadata::DATABASE;
use parser;
use formatter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PhoneNumber {
	/// The country calling code for this number, as defined by the International
	/// Telecommunication Union (ITU). For example, this would be 1 for NANPA
	/// countries, and 33 for France.
	pub(crate) country_code: CountryCode,

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
	pub(crate) national_number: NationalNumber,

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
	pub(crate) carrier: Option<String>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Type {
	///
	FixedLine,

	///
	Mobile,

	/// In some regions (e.g. the USA), it is impossible to distinguish between
	/// fixed-line and mobile numbers by looking at the phone number itself.
	FixedLineOrMobile,

	/// Freephone lines.
	TollFree,

	///
	PremiumRate,

	/// The cost of this call is shared between the caller and the recipient, and
	/// is hence typically less than PREMIUM_RATE calls. See //
	/// http://en.wikipedia.org/wiki/Shared_Cost_Service for more information.
	SharedCost,

	/// Voice over IP numbers. This includes TSoIP (Telephony Service over IP).
	Voip,

	/// A personal number is associated with a particular person, and may be
	/// routed to either a MOBILE or FIXED_LINE number. Some more information can
	/// be found here: http://en.wikipedia.org/wiki/Personal_Numbers
	PersonalNumber,

	///
	Pager,

	/// Used for "Universal Access Numbers" or "Company Numbers". They may be
	/// further routed to specific offices, but allow one number to be used for a
	/// company.
	Uan,

	/// Used for "Voice Mail Access Numbers".
	Voicemail,

	/// A phone number is of type UNKNOWN when it does not fit any of the known
	/// patterns for a specific region.
	Unknown,
}
