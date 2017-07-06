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

use parser::helper::*;

named!(pub phone_number(&str) -> &str,
	alt!(short | long));

named!(short(&str) -> &str,
	recognize!(do_parse!(
		count_fixed!(char, digit, 2) >>
		eof!() >>
		(true))));

named!(long(&str) -> &str,
	recognize!(do_parse!(
		many0!(plus) >>
		many0!(alt!(punctuation | star)) >>
		count_fixed!(char, digit, 3) >>
		many0!(digit) >>
		many0!(alt!(punctuation | star | digit | alpha)) >>
		(true))));

#[cfg(test)]
mod test {
	use parser::valid;

	#[test]
	fn phone() {
    assert!(!valid::phone_number("1").is_done());
    // Only one or two digits before strange non-possible punctuation.
    assert!(!valid::phone_number("1+1+1").is_done());
    assert!(!valid::phone_number("80+0").is_done());
    // Two digits is viable.
    assert!(valid::phone_number("00").is_done());
    assert!(valid::phone_number("111").is_done());
    // Alpha numbers.
    assert!(valid::phone_number("0800-4-pizza").is_done());
    assert!(valid::phone_number("0800-4-PIZZA").is_done());
    // We need at least three digits before any alpha characters.
    assert!(!valid::phone_number("08-PIZZA").is_done());
    assert!(!valid::phone_number("8-PIZZA").is_done());
    assert!(!valid::phone_number("12. March").is_done());
	}
}
