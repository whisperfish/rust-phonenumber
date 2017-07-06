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

use nom::{self, IResult, AsChar};
use fnv::FnvHashMap;

use parser::helper::*;

named!(pub phone_number(&str) -> Number,
	do_parse!(
		opt!(tag_no_case_s!("Tel:")) >>
		prefix: opt!(prefix) >>
		value:  take_while1_s!(number) >>
		call!(check) >>
		params: opt!(parameters) >>

		(Number {
			value: value.into(),

			prefix: prefix.or_else(||
				params.as_ref()
					.and_then(|m| m.get("phone-context"))
					.map(|&s| if s.as_bytes()[0] == b'+' { &s[1 ..] } else { s }))
				.map(Into::into),

			extension: params.as_ref()
				.and_then(|m| m.get("ext"))
				.map(|&s| s.into()),

			.. Default::default()
		})));

named!(prefix(&str) -> &str,
	do_parse!(
		char!('+') >>
		prefix: take_till1_s!(separator) >>

		(prefix)));

named!(parameters(&str) -> FnvHashMap<&str, &str>,
	do_parse!(
		params: many1!(parameter) >>

		({
			let mut map = FnvHashMap::default();

			for (key, value) in params {
				map.insert(key, value);
			}

			map
		})));

named!(parameter(&str) -> (&str, &str),
	do_parse!(
		char!(';') >>
		key: take_while_s!(pname) >>
		char!('=') >>
		value: take_while_s!(pchar) >>

		(key, value)));

fn check(i: &str) -> IResult<&str, ()> {
	if i.is_empty() || i.as_bytes()[0] == b';' {
		IResult::Done(i, ())
	}
	else {
		IResult::Error(nom::ErrorKind::Tag)
	}
}

fn pname(c: char) -> bool {
	c.is_alphanum() || c == '-'
}

fn pchar(c: char) -> bool {
	parameter_unreserved(c) || unreserved(c)
}

fn number(c: char) -> bool {
	digit(c) || separator(c)
}

fn digit(c: char) -> bool {
	c.is_wide_digit() || c.is_hex_digit()
}

fn separator(c: char) -> bool {
	c == '-' || c == '.' || c == '(' || c == ')'
}

fn unreserved(c: char) -> bool {
	c.is_alphanum() || mark(c)
}

fn parameter_unreserved(c: char) -> bool {
	c == '[' || c == ']' || c == '/' || c == ':' || c == '&' || c == '+' || c == '$'
}

fn mark(c: char) -> bool {
	c == '-' ||  c == '_' || c == '.' || c == '!' || c == '~' || c == '*' || c == '\'' || c == '(' || c == ')'
}

#[cfg(test)]
mod test {
	use parser::rfc3966;
	use parser::helper::*;
	use country_code::Source;

	#[test]
	fn phone_number() {
		assert_eq!(rfc3966::phone_number("tel:2034567890;ext=456;phone-context=+44").unwrap().1,
			Number {
				value:     "2034567890".into(),
				prefix:    Some("44".into()),
				extension: Some("456".into()),

				.. Default::default()
			});

		assert_eq!(rfc3966::phone_number("tel:+64-3-331-6005;ext=1235").unwrap().1,
			Number {
				value:     "-3-331-6005".into(),
				prefix:    Some("64".into()),
				extension: Some("1235".into()),

				.. Default::default()
			});
	}
}
