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

error_chain! {
	errors {
		Metadata(error: Metadata) {
			description("An error occurred while parsing the metadata.")
			display("Metadata parsing error: `{:?}`", error)
		}

		Parse(error: Parse) {
			description("An error occurred while parsing the phone number.")
			display("PhoneNumber parsing error: `{:?}`", error)
		}
	}

	foreign_links {
		Io(::std::io::Error);
		Xml(::xml::errors::Error);
		Utf8(::std::str::Utf8Error);
		ParseInt(::std::num::ParseIntError);
		ParseBool(::std::str::ParseBoolError);
		Regex(::regex::Error);
	}
}

#[derive(Clone, Debug)]
pub enum Metadata {
	UnexpectedEof,
	MismatchedTag(String),
	MissingValue(String),

	UnhandledElement {
		phase: String,
		name:  String
	},

	UnhandledAttribute {
		phase: String,
		name:  String,
		value: String,
	},
}

impl From<Metadata> for Error {
	fn from(error: Metadata) -> Self {
		ErrorKind::Metadata(error).into()
	}
}

impl From<Metadata> for ErrorKind {
	fn from(error: Metadata) -> Self {
		ErrorKind::Metadata(error)
	}
}

#[derive(Clone, Debug)]
pub enum Parse {
	/// This generally indicates the string passed in had less than 3 digits in
	/// it.
	NoNumber,

	/// The country code supplied did not belong to a supported country or
	/// non-geographical entity.
	InvalidCountryCode,

	/// This indicates the string started with an international dialing prefix,
	/// but after this was stripped from the number, had less digits than any
	/// valid phone number (including country code) could have.
	TooShortAfterIdd,

	/// This indicates the string, after any country code has been stripped, had
	/// less digits than any valid phone number could have.
	TooShortNsn,

	/// This indicates the string had more digits than any valid phone number
	/// could have.
	TooLong,
}

impl From<Parse> for Error {
	fn from(error: Parse) -> Self {
		ErrorKind::Parse(error).into()
	}
}

impl From<Parse> for ErrorKind {
	fn from(error: Parse) -> Self {
		ErrorKind::Parse(error)
	}
}
