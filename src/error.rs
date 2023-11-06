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

use core::fmt;
use std::error::Error;

/// Metadata loading errors.
#[derive(Debug)]
pub enum Metadata {
    /// EOF was reached before the parsing was complete.
    UnexpectedEof,

    /// A mismatched tag was met.
    MismatchedTag(String),

    /// A required value was missing.
    #[allow(unused)] // This is unused in the build script
    MissingValue { phase: String, name: String },

    /// An element was not handled.
    UnhandledElement { phase: String, name: String },

    /// An attribute was not handled.
    UnhandledAttribute {
        phase: String,
        name: String,
        value: String,
    },

    /// An event was not handled.
    UnhandledEvent { phase: String, event: String },
}

impl Error for Metadata {}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Metadata::UnexpectedEof => f.write_str("unexpected end of file"),
            Metadata::MismatchedTag(s) => write!(f, "mismatched tag: {s:?}"),
            Metadata::MissingValue { phase, name } => write!(f, "{phase}: missing value: {name:?}"),
            Metadata::UnhandledElement { phase, name } => {
                write!(f, "{phase}: unhandled element: {name:?}")
            }
            Metadata::UnhandledAttribute { phase, name, value } => {
                write!(f, "{phase}: unhandled attribute: {name:?}={value:?}")
            }
            Metadata::UnhandledEvent { phase, event } => {
                write!(f, "{phase}: unhandled event: {event:?}")
            }
        }
    }
}

/// Parsing errors.
#[derive(Debug)]
pub enum Parse {
    /// This generally indicates the string passed in had less than 3 digits in
    /// it.
    #[allow(unused)] // This is unused in the build script
    NoNumber,

    /// The country code supplied did not belong to a supported country or
    /// non-geographical entity.
    #[allow(unused)] // This is unused in the build script
    InvalidCountryCode,

    /// This indicates the string started with an international dialing prefix,
    /// but after this was stripped from the number, had less digits than any
    /// valid phone number (including country code) could have.
    #[allow(unused)] // This is unused in the build script
    TooShortAfterIdd,

    /// This indicates the string, after any country code has been stripped, had
    /// less digits than any valid phone number could have.
    #[allow(unused)] // This is unused in the build script
    TooShortNsn,

    /// This indicates the string had more digits than any valid phone number
    /// could have.
    #[allow(unused)] // This is unused in the build script
    TooLong,

    /// A integer parts of a number is malformed, normally this should be caught by the parsing regexes.
    MalformedInteger(std::num::ParseIntError),
}

impl Error for Parse {}

impl fmt::Display for Parse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parse::NoNumber => f.write_str("not a number"),
            Parse::InvalidCountryCode => f.write_str("invalid country code"),
            Parse::TooShortAfterIdd => f.write_str("the number is too short after IDD"),
            Parse::TooShortNsn => f.write_str("the number is too short after the country code"),
            Parse::TooLong => f.write_str("the number is too long"),
            Parse::MalformedInteger(e) => write!(f, "malformed integer part in phone number: {e}"),
        }
    }
}

impl From<std::num::ParseIntError> for Parse {
    fn from(e: std::num::ParseIntError) -> Self {
        Parse::MalformedInteger(e)
    }
}

/// Loading of Database) Error
#[derive(Debug)]
pub enum LoadMetadata {
    /// Parsing XML failed, the XML is malformed.
    Xml(xml::Error),

    /// Parsing UTF-8 string from XML failed.
    Utf8(std::str::Utf8Error),

    /// Metadata Error
    Metadata(Metadata),

    /// Malformed integer in Metadata XML database
    Integer(std::num::ParseIntError),

    /// Malformed boolean in Metadata XML database
    Bool(std::str::ParseBoolError),

    /// I/O-Error while reading Metadata XML database
    Io(std::io::Error),

    /// Malformed Regex in Metadata XML database
    Regex(regex::Error),
}

impl Error for LoadMetadata {}

impl fmt::Display for LoadMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadMetadata::Xml(e) => write!(f, "Malformed Metadata XML: {e}"),
            LoadMetadata::Utf8(e) => write!(f, "Non UTF-8 string in Metadata XML: {e}"),
            LoadMetadata::Metadata(e) => write!(f, "{e}"),
            LoadMetadata::Integer(e) => write!(f, "Malformed integer in Metadata XML: {e}"),
            LoadMetadata::Bool(e) => write!(f, "Malformed boolean in Metadata XML: {e}"),
            LoadMetadata::Io(e) => write!(f, "I/O-Error in Metadata XML: {e}"),
            LoadMetadata::Regex(e) => write!(f, "Malformed Regex: {e}"),
        }
    }
}

impl From<xml::Error> for LoadMetadata {
    fn from(e: xml::Error) -> Self {
        LoadMetadata::Xml(e)
    }
}

impl From<std::str::Utf8Error> for LoadMetadata {
    fn from(e: std::str::Utf8Error) -> Self {
        LoadMetadata::Utf8(e)
    }
}

impl From<Metadata> for LoadMetadata {
    fn from(e: Metadata) -> Self {
        LoadMetadata::Metadata(e)
    }
}

impl From<std::num::ParseIntError> for LoadMetadata {
    fn from(e: std::num::ParseIntError) -> Self {
        LoadMetadata::Integer(e)
    }
}

impl From<std::str::ParseBoolError> for LoadMetadata {
    fn from(e: std::str::ParseBoolError) -> Self {
        LoadMetadata::Bool(e)
    }
}

impl From<std::io::Error> for LoadMetadata {
    fn from(e: std::io::Error) -> Self {
        LoadMetadata::Io(e)
    }
}

impl From<regex::Error> for LoadMetadata {
    fn from(e: regex::Error) -> Self {
        LoadMetadata::Regex(e)
    }
}
