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

use thiserror::Error;

/// Metadata loading errors.
#[derive(Error, Clone, Debug)]
pub enum Metadata {
	/// EOF was reached before the parsing was complete.
	#[error("unexpected end of file")]
	UnexpectedEof,

	/// A mismatched tag was met.
	#[error("mismatched tag: {0:?}")]
	MismatchedTag(String),

	/// A required value was missing.
	#[error("{phase}: missing value: {name:?}")]
	#[allow(unused)] // This is unused in the build script
	MissingValue {
		phase: String,
		name:  String,
	},

	/// An element was not handled.
	#[error("{phase}: unhandled element: {name:?}")]
	UnhandledElement {
		phase: String,
		name:  String
	},

	/// An attribute was not handled.
	#[error("{phase}: unhandled attribute: {name:?}={value:?}")]
	UnhandledAttribute {
		phase: String,
		name:  String,
		value: String,
	},

	/// An event was not handled.
	#[error("{phase}: unhandled event: {event:?}")]
	UnhandledEvent {
		phase: String,
		event: String,
	}
}

/// Parsing errors.
#[derive(Error, Clone, Debug)]
pub enum Parse {
	/// This generally indicates the string passed in had less than 3 digits in
	/// it.
	#[error("not a number")]
	#[allow(unused)] // This is unused in the build script
	NoNumber,

	/// The country code supplied did not belong to a supported country or
	/// non-geographical entity.
	#[error("invalid country code")]
	#[allow(unused)] // This is unused in the build script
	InvalidCountryCode,

	/// This indicates the string started with an international dialing prefix,
	/// but after this was stripped from the number, had less digits than any
	/// valid phone number (including country code) could have.
	#[error("the number is too short after IDD")]
	#[allow(unused)] // This is unused in the build script
	TooShortAfterIdd,

	/// This indicates the string, after any country code has been stripped, had
	/// less digits than any valid phone number could have.
	#[error("the number is too short after the country code")]
	#[allow(unused)] // This is unused in the build script
	TooShortNsn,

	/// This indicates the string had more digits than any valid phone number
	/// could have.
	#[error("the number is too long")]
	#[allow(unused)] // This is unused in the build script
	TooLong,

	/// A integer parts of a number is malformed, normally this should be caught by the parsing regexes.
	#[error("malformed integer part in phone number: {0}")]
	MalformedInteger(#[from] std::num::ParseIntError),
}


/// Loading of Database) Error
#[derive(Error, Debug)]
pub enum LoadMetadata {

	/// Parsing XML failed, the XML is malformed.
	#[error("Malformed Metadata XML: {0}")]
	Xml(#[from] xml::Error),

	/// Parsing UTF-8 string from XML failed.
	#[error("Non UTF-8 string in Metadata XML: {0}")]
	Utf8(#[from] std::str::Utf8Error),

	/// Metadata Error
	#[error("{0}")]
	Metadata(#[from] Metadata),

	/// Malformed integer in Metadata XML database
	#[error("Malformed integer in Metadata XML: {0}")]
	Integer(#[from] std::num::ParseIntError),

	/// Malformed boolean in Metadata XML database
	#[error("Malformed boolean in Metadata XML: {0}")]
	Bool(#[from] std::str::ParseBoolError),

	/// I/O-Error while reading Metadata XML database
	#[error("I/O-Error in Metadata XML: {0}")]
	Io(#[from] std::io::Error),

	/// Malformed Regex in Metadata XML database
	#[error("Malformed Regex: {0}")]
	Regex(#[from] regex::Error),
	
	#[error("Malformed Regex: {0}")]
	RegexSyntax(#[from] regex_syntax::Error),

}