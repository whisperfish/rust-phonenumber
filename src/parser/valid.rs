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
use nom::types::CompleteStr;

named!(pub phone_number(CompleteStr) -> CompleteStr,
	alt!(short | long));

named!(short(CompleteStr) -> CompleteStr,
	recognize!(do_parse!(
		count_fixed!(char, digit, 2) >>
		eof!() >>
		(true))));

named!(long(CompleteStr) -> CompleteStr,
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
	use nom::types::CompleteStr;

	#[test]
	fn phone() {
	    assert!(!valid::phone_number(CompleteStr("1")).is_ok());
	    // Only one or two digits before strange non-possible punctuation.
	    assert!(!valid::phone_number(CompleteStr("1+1+1")).is_ok());
	    assert!(!valid::phone_number(CompleteStr("80+0")).is_ok());
	    // Two digits is viable.
	    assert!(valid::phone_number(CompleteStr("00")).is_ok());
	    assert!(valid::phone_number(CompleteStr("111")).is_ok());
	    // Alpha numbers.
	    assert!(valid::phone_number(CompleteStr("0800-4-pizza")).is_ok());
	    assert!(valid::phone_number(CompleteStr("0800-4-PIZZA")).is_ok());
	    // We need at least three digits before any alpha characters.
	    assert!(!valid::phone_number(CompleteStr("08-PIZZA")).is_ok());
	    assert!(!valid::phone_number(CompleteStr("8-PIZZA")).is_ok());
	    assert!(!valid::phone_number(CompleteStr("12. March")).is_ok());
	}
}
