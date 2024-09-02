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

#![allow(unused)]

use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

/// The minimum length of the National Significant Number.
pub const MIN_LENGTH_FOR_NSN: usize = 2;

/// The maximum length of the National Significant Number.
pub const MAX_LENGTH_FOR_NSN: usize = 17;

/// The maximum length of the country calling code.
pub const MAX_LENGTH_FOR_COUNTRY_CODE: usize = 3;

/// Region-code for the unknown region.
pub const UNKNOWN_REGION: &str = "ZZ";

pub const NANPA_COUNTRY_CODE: u32 = 1;

/// The prefix that needs to be inserted in front of a Colombian landline
/// number when dialed from a mobile phone in Colombia.
pub const COLOMBIA_MOBILE_TO_FIXED_LINE_PREFIX: &str = "3";

pub const PLUS_SIGN: char = '+';
pub const STAR_SIGN: char = '*';
pub const SHARP_SIGN: char = '#';

pub const RFC3966_EXTN_PREFIX: &str = ";ext=";
pub const RFC3966_PREFIX: &str = "tel:";
pub const RFC3966_PHONE_CONTEXT: &str = ";phone-context=";
pub const RFC3966_ISDN_SUBADDRESS: &str = ";isub=";

pub const REGION_CODE_FOR_NON_GEO_ENTITY: &str = "001";

/// Map of country calling codes that use a mobile token before the area code. One example of when
/// this is relevant is when determining the length of the national destination code, which should
/// be the length of the area code plus the length of the mobile token.
pub static MOBILE_TOKEN_MAPPINGS: Lazy<FnvHashMap<u16, &'static str>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();
    map.insert(52, "1");
    map.insert(54, "9");
    map
});

/// Set of country codes that have geographically assigned mobile numbers
/// (see GEO_MOBILE_COUNTRIES below) which are not based on *area codes*. For
/// example, in China mobile numbers start with a carrier indicator, and
/// beyond that are geographically assigned: this carrier indicator is not
/// considered to be an area code.
pub static GEO_MOBILE_COUNTRIES_WITHOUT_MOBILE_AREA_CODES: Lazy<FnvHashSet<u16>> =
    Lazy::new(|| {
        let mut set = FnvHashSet::default();
        set.insert(86); // China
        set
    });

/// Set of country calling codes that have geographically assigned mobile
/// numbers. This may not be complete; we add calling codes case by case, as
/// we find geographical mobile numbers or hear from user reports. Note that
/// countries like the US, where we can't distinguish between fixed-line or
/// mobile numbers, are not listed here, since we consider
/// FIXED_LINE_OR_MOBILE to be a possibly geographically-related type anyway
/// (like FIXED_LINE).
pub static GEO_MOBILE_COUNTRIES: Lazy<FnvHashSet<u16>> = Lazy::new(|| {
    let mut set = FnvHashSet::default();
    set.insert(52); // Mexico
    set.insert(54); // Argentina
    set.insert(55); // Brazil
    set.insert(62); // Indonesia: some prefixes only (fixed CMDA wireless)
    set.extend(GEO_MOBILE_COUNTRIES_WITHOUT_MOBILE_AREA_CODES.iter());
    set
});

/// Helper ASCII mappings.
pub static ASCII_MAPPINGS: Lazy<FnvHashMap<char, char>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();
    map.insert('0', '0');
    map.insert('1', '1');
    map.insert('2', '2');
    map.insert('3', '3');
    map.insert('4', '4');
    map.insert('5', '5');
    map.insert('6', '6');
    map.insert('7', '7');
    map.insert('8', '8');
    map.insert('9', '9');
    map
});

/// A map that contains characters that are essential when dialling. That
/// means any of the characters in this map must not be removed from a number
/// when dialling, otherwise the call will not reach the intended
/// destination.
pub static DIALLABLE_CHAR_MAPPINGS: Lazy<FnvHashMap<char, char>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();
    map.extend(ASCII_MAPPINGS.iter());
    map.insert(PLUS_SIGN, PLUS_SIGN);
    map.insert(STAR_SIGN, STAR_SIGN);
    map.insert(SHARP_SIGN, SHARP_SIGN);
    map
});

/// Only upper-case variants of alpha characters are stored.
pub static ALPHA_MAPPINGS: Lazy<FnvHashMap<char, char>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();
    map.insert('A', '2');
    map.insert('B', '2');
    map.insert('C', '2');
    map.insert('D', '3');
    map.insert('E', '3');
    map.insert('F', '3');
    map.insert('G', '4');
    map.insert('H', '4');
    map.insert('I', '4');
    map.insert('J', '5');
    map.insert('K', '5');
    map.insert('L', '5');
    map.insert('M', '6');
    map.insert('N', '6');
    map.insert('O', '6');
    map.insert('P', '7');
    map.insert('Q', '7');
    map.insert('R', '7');
    map.insert('S', '7');
    map.insert('T', '8');
    map.insert('U', '8');
    map.insert('V', '8');
    map.insert('W', '9');
    map.insert('X', '9');
    map.insert('Y', '9');
    map.insert('Z', '9');

    map.insert('a', '2');
    map.insert('b', '2');
    map.insert('c', '2');
    map.insert('d', '3');
    map.insert('e', '3');
    map.insert('f', '3');
    map.insert('g', '4');
    map.insert('h', '4');
    map.insert('i', '4');
    map.insert('j', '5');
    map.insert('k', '5');
    map.insert('l', '5');
    map.insert('m', '6');
    map.insert('n', '6');
    map.insert('o', '6');
    map.insert('p', '7');
    map.insert('q', '7');
    map.insert('r', '7');
    map.insert('s', '7');
    map.insert('t', '8');
    map.insert('u', '8');
    map.insert('v', '8');
    map.insert('w', '9');
    map.insert('x', '9');
    map.insert('y', '9');
    map.insert('z', '9');

    map
});

/// For performance reasons, amalgamate both into one map.
pub static ALPHA_PHONE_MAPPINGS: Lazy<FnvHashMap<char, char>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();
    map.extend(ASCII_MAPPINGS.iter());
    map.extend(ALPHA_MAPPINGS.iter());
    map
});

/// Separate map of all symbols that we wish to retain when formatting alpha
/// numbers. This includes digits, ASCII letters and number grouping symbols
/// such as "-" and " ".
pub static ALL_PLUS_NUMBER_GROUPING_SYMBOLS: Lazy<FnvHashMap<char, char>> = Lazy::new(|| {
    let mut map = FnvHashMap::default();

    for &c in ALPHA_MAPPINGS.keys() {
        map.insert(c, c);
        map.insert(c.to_lowercase().next().unwrap(), c);
    }

    map.extend(ASCII_MAPPINGS.iter());

    map.insert('-', '-');
    map.insert('\u{FF0D}', '-');
    map.insert('\u{2010}', '-');
    map.insert('\u{2011}', '-');
    map.insert('\u{2012}', '-');
    map.insert('\u{2013}', '-');
    map.insert('\u{2014}', '-');
    map.insert('\u{2015}', '-');
    map.insert('\u{2212}', '-');
    map.insert('/', '/');
    map.insert('\u{FF0F}', '/');
    map.insert(' ', ' ');
    map.insert('\u{3000}', ' ');
    map.insert('\u{2060}', ' ');
    map.insert('.', '.');
    map.insert('\u{FF0E}', '.');

    map
});

/// Pattern that makes it easy to distinguish whether a region has a unique
/// international dialing prefix or not. If a region has a unique
/// international prefix (e.g. 011 in USA), it will be represented as a
/// string that contains a sequence of ASCII digits. If there are multiple
/// available international prefixes in a region, they will be represented as
/// a regex string that always contains character(s) other than ASCII digits.
///
/// Note this regex also includes tilde, which signals waiting for the tone.
pub static UNIQUE_INTERNATIONAL_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\d]+(?:[~\x{2053}\x{223C}\x{FF5E}][\d]+)?").unwrap());

/// Regular expression of acceptable punctuation found in phone numbers. This
/// excludes punctuation found as a leading character only.
///
/// This consists of dash characters, white space characters, full stops,
/// slashes, square brackets, parentheses and tildes. It also includes the
/// letter 'x' as that is found as a placeholder for carrier information in
/// some phone numbers. Full-width variants are also present.
pub const VALID_PUNCTUATION: &str = r"-x\x{2010}-\x{2015}\x{2212}\x{30FC}\x{FF0D}-\x{FF0F} \x{00A0}\x{00AD}\x{200B}\x{2060}\x{3000}()\x{FF08}\x{FF09}\x{FF3B}\x{FF3D}.\[\]/~\x{2053}\x{223C}\x{FF5E}";

/// Pattern for digits.
pub const DIGITS: &str = r"\p{Nd}";

/// Plus characters.
pub const PLUS_CHARS: &str = r"\+\x{FF0B}";

/// We accept alpha characters in phone numbers, ASCII only, upper and lower
/// case.
pub static VALID_ALPHA: Lazy<String> = Lazy::new(|| {
    let mut string = String::new();
    let clean = Regex::new(r"[, \[\]]").unwrap();
    let alpha = ALPHA_MAPPINGS.keys().collect::<String>();

    string.push_str(&clean.replace(&alpha, ""));
    string.push_str(&clean.replace(&alpha.to_lowercase(), ""));

    string
});

pub static PLUS_CHARS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("[{}]+", PLUS_CHARS)).unwrap());

pub static SEPARATOR_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("[{}]+", VALID_PUNCTUATION)).unwrap());

pub static CAPTURING_DIGIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("({})", DIGITS)).unwrap());

/// Regular expression of acceptable characters that may start a phone number
/// for the purposes of parsing. This allows us to strip away meaningless
/// prefixes to phone numbers that may be mistakenly given to us. This
/// consists of digits, the plus symbol and arabic-indic digits. This does
/// not contain alpha characters, although they may be used later in the
/// number. It also does not include other punctuation, as this will be
/// stripped later during parsing and is of no information value when parsing
/// a number.
pub static VALID_START_CHAR: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("[{}{}]", PLUS_CHARS, DIGITS)).unwrap());

/// Regular expression of characters typically used to start a second phone
/// number for the purposes of parsing. This allows us to strip off parts of
/// the number that are actually the start of another number, such as for:
/// (530) 583-6985 x302/x2303 -> the second extension here makes this
/// actually two phone numbers, (530) 583-6985 x302 and (530) 583-6985 x2303.
/// We remove the second extension so that the first number is parsed
/// correctly.
pub static SECOND_NUMBER_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\\/] *x").unwrap());

/// Regular expression of trailing characters that we want to remove. We
/// remove all characters that are not alpha or numerical characters. The
/// hash character is retained here, as it may signify the previous block was
/// an extension.
pub static UNWANTED_END_CHARS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[[\P{N}&&\P{L}]&&[^#]]+$").unwrap());

/// We use this pattern to check if the phone number has at least three
/// letters in it - if so, then we treat it as a number where some
/// phone-number digits are represented by letters.
pub static VALID_ALPHA_PHONE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:.*?[A-Za-z]){3}.*").unwrap());

/// Default extension prefix to use when formatting. This will be put in
/// front of any extension component of the number, after the main national
/// number is formatted. For example, if you wish the default extension
/// formatting to be " extn: 3456", then you should specify " extn: " here as
/// the default extension prefix. This can be overridden by region-specific
/// preferences.
pub const DEFAULT_EXTN_PREFIX: &str = " ext. ";

/// Pattern to capture digits used in an extension. Places a maximum length
/// of "7" for an extension.
pub static CAPTURING_EXTN_DIGITS: Lazy<String> = Lazy::new(|| format!("({}{{0,7}})", DIGITS));

/// Regexp of all possible ways to write extensions, for use when parsing.
/// This will be run as a case-insensitive regexp match. Wide character
/// versions are also provided after each ASCII version.
///
/// For parsing, we are slightly more lenient in our interpretation than for
/// matching. Here we allow "comma" and "semicolon" as possible extension
/// indicators. When matching, these are hardly ever used to indicate this.
pub static EXTN_PATTERNS_FOR_PARSING: Lazy<String> = Lazy::new(|| {
    format!(
        r"{rfc3966_extn_prefix}{capturing_extn_digits}|[ \x{{00A0}}\t,]*(?:e?xt(?:ensi(?:o\x{{0301}}?|\x{{00F3}}))?n?|\x{{FF45}}?\x{{FF58}}\x{{FF54}}\x{{FF4E}}?|[{symbols}]|int|anexo|\x{{FF49}}\x{{FF4E}}\x{{FF54}})[:\.\x{{FF0E}}]?[ \x{{00A0}}\t,-]*{capturing_extn_digits}#?|[- ]+({digits}{{1,5}})#",
        rfc3966_extn_prefix = RFC3966_EXTN_PREFIX,
        capturing_extn_digits = *CAPTURING_EXTN_DIGITS,
        symbols = r",;x\x{FF58}#\x{FF03}~\x{FF5E}",
        digits = DIGITS
    )
});

/// Regexp of all possible ways to write extensions, for use when parsing.
/// This will be run as a case-insensitive regexp match. Wide character
/// versions are also provided after each ASCII version.
///
/// One-character symbols that can be used to indicate an extension.
pub static EXTN_PATTERNS_FOR_MATCHING: Lazy<String> = Lazy::new(|| {
    format!(
        r"{rfc3966_extn_prefix}{capturing_extn_digits}|[ \x{{00A0}}\t,]*(?:e?xt(?:ensi(?:o\x{{0301}}?|\x{{00F3}}))?n?|\x{{FF45}}?\x{{FF58}}\x{{FF54}}\x{{FF4E}}?|[{symbols}]|int|anexo|\x{{FF49}}\x{{FF4E}}\x{{FF54}})[:\.\x{{FF0E}}]?[ \x{{00A0}}\t,-]*{capturing_extn_digits}#?|[- ]+({digits}{{1,5}})#",
        rfc3966_extn_prefix = RFC3966_EXTN_PREFIX,
        capturing_extn_digits = *CAPTURING_EXTN_DIGITS,
        symbols = r"x\x{FF58}#\x{FF03}~\x{FF5E}",
        digits = DIGITS
    )
});

/// Regexp of all known extension prefixes used by different regions followed
/// by 1 or more valid digits, for use when parsing.
pub static EXTN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(&format!(r"(?:{})$", *EXTN_PATTERNS_FOR_PARSING))
        .case_insensitive(true)
        .build()
        .unwrap()
});

/// We append optionally the extension pattern to the end here, as a valid
/// phone number may have an extension prefix appended, followed by 1 or more
/// digits.
pub static VALID_PHONE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(&format!(r"(?:{})?", *EXTN_PATTERNS_FOR_PARSING))
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub static NON_DIGITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\D+)").unwrap());

/// The FIRST_GROUP_PATTERN was originally set to $1 but there are some
/// countries for which the first group is not used in the national pattern
/// (e.g. Argentina) so the $1 group does not match correctly.  Therefore, we
/// use \d, so that the first group actually used in the pattern will be
/// matched.
pub static FIRST_GROUP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\$\d)").unwrap());

pub const NP: &str = "$NP";
pub const FG: &str = "$FG";
pub const CC: &str = "$CC";

/// A pattern that is used to determine if the national prefix formatting
/// rule has the first group only, i.e., does not start with the national
/// prefix. Note that the pattern explicitly allows for unbalanced
/// parentheses.
pub static FIRST_GROUP_ONLY_PREFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(?\$1\)?").unwrap());
