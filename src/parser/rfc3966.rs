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

use crate::parser::helper::*;
use crate::parser::rfc3986;
use fnv::FnvHashMap;
use nom::{
    AsChar, IResult,
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::{ErrorKind, make_error},
    multi::*,
};
use std::borrow::Cow;

pub fn phone_number(i: &str) -> IResult<&str, Number<'_>> {
    // Per RFC3966, spaces are not allowed as separators.
    if i.contains(' ') {
        return Err(nom::Err::Error(make_error(i, ErrorKind::Tag)));
    }

    parse! { i =>
        opt(tag_no_case("Tel:"));
        let prefix = opt(prefix);
        let national = take_while1(number);
        check;
        let params = opt(parameters);
    };

    // Malformed pct-encoding makes the URI invalid; surface it as a parse
    // failure rather than silently keeping the raw octets. `at` and the
    // decoded slice share the input's lifetime.
    fn decode<'a>(
        s: &'a str,
        at: &'a str,
    ) -> Result<Cow<'a, str>, nom::Err<nom::error::Error<&'a str>>> {
        rfc3986::decode(s).ok_or_else(|| nom::Err::Failure(make_error(at, ErrorKind::Tag)))
    }

    let national = decode(national, i)?;

    let prefix = match prefix {
        Some(p) => Some(decode(p, i)?),
        None => params
            .as_ref()
            .and_then(|m| m.get("phone-context"))
            .map(|&s| decode(s, i))
            .transpose()?
            .map(|cs| match cs {
                Cow::Borrowed(s) => Cow::Borrowed(s.strip_prefix('+').unwrap_or(s)),
                Cow::Owned(s) => Cow::Owned(s.strip_prefix('+').unwrap_or(&s).to_owned()),
            }),
    };

    Ok((
        i,
        Number {
            national,
            prefix,

            extension: params
                .as_ref()
                .and_then(|m| m.get("ext"))
                .map(|&cs| cs.into()),

            ..Default::default()
        },
    ))
}

fn prefix(i: &str) -> IResult<&str, &str> {
    parse! { i =>
        char('+');
        take_till1(separator)
    }
}

fn parameters(i: &str) -> IResult<&str, FnvHashMap<&str, &str>> {
    parse! { i =>
        let params = many1(parameter);
    };

    Ok((i, params.into_iter().collect()))
}

fn parameter(i: &str) -> IResult<&str, (&str, &str)> {
    parse! { i =>
        char(';');
        let key = take_while(pname);
        char('=');
        let value = take_while(pchar);
    };

    Ok((i, (key, value)))
}

fn check(i: &str) -> IResult<&str, ()> {
    if i.is_empty() || i.as_bytes()[0] == b';' {
        Ok((i, ()))
    } else {
        Err(nom::Err::Error(make_error(i, ErrorKind::Tag)))
    }
}

fn pname(c: char) -> bool {
    c.is_alphanum() || c == '-'
}

fn pchar(c: char) -> bool {
    parameter_unreserved(c) || unreserved(c) || c == '%'
}

fn number(c: char) -> bool {
    digit(c) || separator(c) || c == '%'
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
    c == '-'
        || c == '_'
        || c == '.'
        || c == '!'
        || c == '~'
        || c == '*'
        || c == '\''
        || c == '('
        || c == ')'
}

#[cfg(test)]
mod test {
    use crate::parser::helper::*;
    use crate::parser::rfc3966;

    #[test]
    fn phone_number() {
        assert_eq!(
            rfc3966::phone_number("tel:2034567890;ext=456;phone-context=+44")
                .unwrap()
                .1,
            Number {
                national: "2034567890".into(),
                prefix: Some("44".into()),
                extension: Some("456".into()),

                ..Default::default()
            }
        );

        assert_eq!(
            rfc3966::phone_number("tel:+64-3-331-6005;ext=1235")
                .unwrap()
                .1,
            Number {
                national: "-3-331-6005".into(),
                prefix: Some("64".into()),
                extension: Some("1235".into()),

                ..Default::default()
            }
        );
    }

    #[test]
    fn pct_encoded_national_decodes() {
        // %2D is '-'; the decoded national must match the literal form.
        assert_eq!(
            rfc3966::phone_number("tel:03%2D331%2D6005;phone-context=+64")
                .unwrap()
                .1,
            Number {
                national: "03-331-6005".into(),
                prefix: Some("64".into()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn pct_encoded_phone_context_decodes() {
        assert_eq!(
            rfc3966::phone_number("tel:03-331-6005;phone-context=%2B64")
                .unwrap()
                .1,
            Number {
                national: "03-331-6005".into(),
                prefix: Some("64".into()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn malformed_pct_encoding_is_error() {
        assert!(rfc3966::phone_number("tel:03-331%2-6005").is_err());
    }

    #[test]
    fn advisory_1() {
        // Just make sure this does not panic.
        drop(rfc3966::phone_number(".;phone-context="));
    }
}
