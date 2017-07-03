use country_code::CountryCode;
use national_number::NationalNumber;
use extension::Extension;

#[derive(Clone, Debug)]
pub struct PhoneNumber {
	/// This field is used to store the raw input string containing phone numbers
	/// before it was canonicalized by the library. For example, it could be used
	/// to store alphanumerical numbers such as "1-800-GOOG-411".
	original: Option<String>,

	/// The country calling code for this number, as defined by the International
	/// Telecommunication Union (ITU). For example, this would be 1 for NANPA
	/// countries, and 33 for France.
	country_code: CountryCode,

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
	national_number: NationalNumber,

	/// Extension is not standardized in ITU recommendations, except for being
	/// defined as a series of numbers with a maximum length of 40 digits. It is
	/// defined as a string here to accommodate for the possible use of a leading
	/// zero in the extension (organizations have complete freedom to do so, as
	/// there is no standard defined). Other than digits, some other dialling
	/// characters such as "," (indicating a wait) may be stored here.
	extension: Option<Extension>,

	/// The carrier selection code that is preferred when calling this phone
	/// number domestically. This also includes codes that need to be dialed in
	/// some countries when calling from landlines to mobiles or vice versa. For
	/// example, in Columbia, a "3" needs to be dialed before the phone number
	/// itself when calling from a mobile phone to a domestic landline phone and
	/// vice versa.
	///
	/// Note this is the "preferred" code, which means other codes may work as
	/// well.
	domestic_carrier: Option<String>,
}
