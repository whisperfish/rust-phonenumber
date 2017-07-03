use metadata::{Format, Descriptor};
use country_code::CountryCode;

#[derive(Clone, Debug)]
pub struct Metadata {
	pub(crate) general:          Option<Descriptor>,
	pub(crate) fixed_line:       Option<Descriptor>,
	pub(crate) mobile:           Option<Descriptor>,
	pub(crate) toll_free:        Option<Descriptor>,
	pub(crate) premium_rate:     Option<Descriptor>,
	pub(crate) shared_cost:      Option<Descriptor>,
	pub(crate) personal:         Option<Descriptor>,
	pub(crate) voip:             Option<Descriptor>,
	pub(crate) pager:            Option<Descriptor>,
	pub(crate) uan:              Option<Descriptor>,
	pub(crate) emergency:        Option<Descriptor>,
	pub(crate) voicemail:        Option<Descriptor>,
	pub(crate) short_code:       Option<Descriptor>,
	pub(crate) standard_rate:    Option<Descriptor>,
	pub(crate) carrier:          Option<Descriptor>,
	pub(crate) no_international: Option<Descriptor>,

	/// The CLDR 2-letter representation of a country/region, with the exception
	/// of "country calling codes" used for non-geographical entities, such as
	/// Universal International Toll Free Number (+800). These are all given the
	/// ID "001", since this is the numeric region code for the world according
	/// to UN M.49: http://en.wikipedia.org/wiki/UN_M.49
	pub(crate) id: String,

	/// The country calling code that one would dial from overseas when trying to
	/// dial a phone number in this country. For example, this would be "64" for
	/// New Zealand.
	pub(crate) country_code: Option<u32>,

	/// The international_prefix of country A is the number that needs to be
	/// dialled from country A to another country (country B). This is followed
	/// by the country code for country B. Note that some countries may have more
	/// than one international prefix, and for those cases, a regular expression
	/// matching the international prefixes will be stored in this field.
	pub(crate) international_prefix: Option<String>,

	/// If more than one international prefix is present, a preferred prefix can
	/// be specified here for out-of-country formatting purposes. If this field
	/// is not present, and multiple international prefixes are present, then "+"
	/// will be used instead.
	pub(crate) preferred_international_prefix: Option<String>,

	/// The national prefix of country A is the number that needs to be dialled
	/// before the national significant number when dialling internally. This
	/// would not be dialled when dialling internationally. For example, in New
	/// Zealand, the number that would be locally dialled as 09 345 3456 would be
	/// dialled from overseas as +64 9 345 3456. In this case, 0 is the national
	/// prefix.
	pub(crate) national_prefix: Option<String>,

	/// The preferred prefix when specifying an extension in this country. This
	/// is used for formatting only, and if this is not specified, a suitable
	/// default should be used instead. For example, if you wanted extensions to
	/// be formatted in the following way:
	///
	/// 1 (365) 345 445 ext. 2345
	/// " ext. "  should be the preferred extension prefix.
	pub(crate) preferred_extension_prefix: Option<String>,

	/// This field is used for cases where the national prefix of a country
	/// contains a carrier selection code, and is written in the form of a
	/// regular expression. For example, to dial the number 2222-2222 in
	/// Fortaleza, Brazil (area code 85) using the long distance carrier Oi
	/// (selection code 31), one would dial 0 31 85 2222 2222. Assuming the only
	/// other possible carrier selection code is 32, the field will contain
	/// "03[12]".
	///
	/// When it is missing from the XML file, this field inherits the value of
	/// national_prefix, if that is present.
	pub(crate) national_prefix_for_parsing: Option<String>,

	/// This field is only populated and used under very rare situations.  For
	/// example, mobile numbers in Argentina are written in two completely
	/// different ways when dialed in-country and out-of-country (e.g. 0343 15
	/// 555 1212 is exactly the same number as +54 9 343 555 1212).
	///
	/// This field is used together with national_prefix_for_parsing to transform
	/// the number into a particular representation for storing in the
	/// phonenumber proto buffer in those rare cases.
	pub(crate) national_prefix_transform_rule: Option<String>,

	/// Note that the number format here is used for formatting only, not
	/// parsing.  Hence all the varied ways a user *may* write a number need not
	/// be recorded - just the ideal way we would like to format it for them.
	///
	/// When this element is absent, the national significant number will be
	/// formatted as a whole without any formatting applied.
	pub(crate) format: Vec<Format>,

	/// This field is populated only when the national significant number is
	/// formatted differently when it forms part of the INTERNATIONAL format and
	/// NATIONAL format. A case in point is mobile numbers in Argentina: The
	/// number, which would be written in INTERNATIONAL format as +54 9 343 555
	/// 1212, will be written as 0343 15 555 1212 for NATIONAL format. In this
	/// case, the prefix 9 is inserted when dialling from overseas, but otherwise
	/// the prefix 0 and the carrier selection code
	/// 15 (inserted after the area code of 343) is used.
	///
	/// Note: this field is populated by setting a value for <intlFormat> inside
	/// the <numberFormat> tag in the XML file. If <intlFormat> is not set then
	/// it defaults to the same value as the <format> tag.
	///
	/// Examples:
	///   To set the <intlFormat> to a different value than the <format>:
	///     <numberFormat pattern=....>
	///       <format>$1 $2 $3</format>
	///       <intlFormat>$1-$2-$3</intlFormat>
	///     </numberFormat>
	///
	///   To have a format only used for national formatting, set <intlFormat> to
	///   "NA":
	///     <numberFormat pattern=....>
	///       <format>$1 $2 $3</format>
	///       <intlFormat>NA</intlFormat>
	///     </numberFormat>
	pub(crate) international_format: Vec<Format>,

	/// This field is set when this country is considered to be the main country
	/// for a calling code. It may not be set by more than one country with the
	/// same calling code, and it should not be set by countries with a unique
	/// calling code. This can be used to indicate that "GB" is the main country
	/// for the calling code "44" for example, rather than Jersey or the Isle of
	/// Man.
	pub(crate) main_country_for_code: bool,

	/// This field is populated only for countries or regions that share a
	/// country calling code. If a number matches this pattern, it could belong
	/// to this region. This is not intended as a replacement for
	/// IsValidForRegion since a matching prefix is insufficient for a number to
	/// be valid. Furthermore, it does not contain all the prefixes valid for a
	/// region - for example, 800 numbers are valid for all NANPA countries and
	/// are hence not listed here.
	///
	/// This field should be a regular expression of the expected prefix match.
	///
	/// It is used merely as a short-cut for working out which region a number
	/// comes from in the case that there is only one, so leading_digit prefixes
	/// should not overlap.
	pub(crate) leading_digits: Option<String>,

	/// This field is set when this country has implemented mobile number
	/// portability. This means that transferring mobile numbers between carriers
	/// is allowed. A consequence of this is that phone prefix to carrier mapping
	/// is less reliable.
	pub(crate) mobile_number_portable: bool,
}
