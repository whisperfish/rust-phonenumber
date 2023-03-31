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

use fnv::FnvHashMap;
use nom::{
    self,
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::{make_error, ErrorKind},
    multi::*,
    AsChar, IResult,
};

use crate::parser::helper::*;

pub fn phone_number(i: &str) -> IResult<&str, Number> {
    parse! { i =>
        opt(tag_no_case("Tel:"));
        let prefix = opt(prefix);
        let national = take_while1(number);
        check;
        let params = opt(parameters);
    };

    Ok((
        i,
        Number {
            national: (*national).into(),

            prefix: prefix
                .or_else(|| {
                    params
                        .as_ref()
                        .and_then(|m| m.get("phone-context"))
                        .map(|&s| if s.as_bytes()[0] == b'+' { &s[1..] } else { s })
                })
                .map(|cs| cs.into()),

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
}
