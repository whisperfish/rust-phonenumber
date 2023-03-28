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
use nom::{branch::*, combinator::*, multi::*, IResult};

pub fn phone_number(i: &str) -> IResult<&str, &str> {
    parse! { i => recognize(alt((short, long))) }
}

fn short(i: &str) -> IResult<&str, ()> {
    parse! { i =>
        count(digit, 2);
        eof;
    };

    Ok((i, ()))
}

fn long(i: &str) -> IResult<&str, ()> {
    parse! { i =>
        many0(plus);
        many0(alt((punctuation, star)));
        count(digit, 3);
        many0(digit);
        many0(alt((punctuation, star, digit, alpha)));
        eof;
    };

    Ok((i, ()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn phone() {
        assert!(!phone_number("1").is_ok());
        // Only one or two digits before strange non-possible punctuation.
        assert!(!phone_number("1+1+1").is_ok());
        assert!(!phone_number("80+0").is_ok());
        // Two digits is viable.
        assert!(phone_number("00").is_ok());
        assert!(phone_number("111").is_ok());
        // Alpha numbers.
        assert!(phone_number("0800-4-pizza").is_ok());
        assert!(phone_number("0800-4-PIZZA").is_ok());
        // We need at least three digits before any alpha characters.
        assert!(!phone_number("08-PIZZA").is_ok());
        assert!(!phone_number("8-PIZZA").is_ok());
        assert!(!phone_number("12. March").is_ok());
    }
}
