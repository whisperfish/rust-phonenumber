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

/// Metadata loading errors.
#[derive(Fail, Clone, Debug)]
pub enum Metadata {
	/// EOF was reached before the parsing was complete.
	#[fail(display = "unexpected end of file")]
	UnexpectedEof,

	/// A mismatched tag was met.
	#[fail(display = "mismatched tag: {}", _0)]
	MismatchedTag(String),

	/// A required value was missing.
	#[fail(display = "{}: missing value: {}", phase, name)]
	MissingValue {
		phase: String,
		name:  String,
	},

	/// An element was not handled.
	#[fail(display = "{}: unhandled element: {}", phase, name)]
	UnhandledElement {
		phase: String,
		name:  String
	},

	/// An attribute was not handled.
	#[fail(display = "{}: unhandled attribute: {}={}", phase, name, value)]
	UnhandledAttribute {
		phase: String,
		name:  String,
		value: String,
	},

	/// An event was not handled.
	#[fail(display = "{}: unhandled event: {}", phase, event)]
	UnhandledEvent {
		phase: String,
		event: String,
	}
}

/// Parsing errors.
#[derive(Fail, Clone, Debug)]
pub enum Parse {
	/// This generally indicates the string passed in had less than 3 digits in
	/// it.
	#[fail(display = "not a number")]
	NoNumber,

	/// The country code supplied did not belong to a supported country or
	/// non-geographical entity.
	#[fail(display = "invalid country code")]
	InvalidCountryCode,

	/// This indicates the string started with an international dialing prefix,
	/// but after this was stripped from the number, had less digits than any
	/// valid phone number (including country code) could have.
	#[fail(display = "the number is too short after IDD")]
	TooShortAfterIdd,

	/// This indicates the string, after any country code has been stripped, had
	/// less digits than any valid phone number could have.
	#[fail(display = "the number is too short after the country code")]
	TooShortNsn,

	/// This indicates the string had more digits than any valid phone number
	/// could have.
	#[fail(display = "the number is too long")]
	TooLong,
}
