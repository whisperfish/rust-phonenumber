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

/// Metadata loading errors from XML parsing.
#[derive(Error, Clone, Debug)]
pub enum MetadataParseError {
    /// EOF was reached before the parsing was complete.
    #[error("unexpected end of file")]
    UnexpectedEof,

    /// A mismatched tag was met.
    #[error("mismatched tag: {0:?}")]
    MismatchedTag(String),

    /// A required value was missing.
    #[error("{phase}: missing value: {name:?}")]
    MissingValue { phase: String, name: String },

    /// An element was not handled.
    #[error("{phase}: unhandled element: {name:?}")]
    UnhandledElement { phase: String, name: String },

    /// An attribute was not handled.
    #[error("{phase}: unhandled attribute: {name:?}={value:?}")]
    UnhandledAttribute {
        phase: String,
        name: String,
        value: String,
    },

    /// An event was not handled.
    #[error("{phase}: unhandled event: {event:?}")]
    UnhandledEvent { phase: String, event: String },
}

/// Loading of Database error — wraps all errors that can occur during metadata loading.
/// **Note: This enum has NO Parse variant.** Parse errors are separate and belong to
/// the phone number parsing logic in the main phonenumber crate.
#[derive(Error, Debug)]
pub enum MetadataLoadError {
    /// Parsing XML failed, the XML is malformed.
    #[error("Malformed Metadata XML: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Parsing UTF-8 string from XML failed.
    #[error("Non UTF-8 string in Metadata XML: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// Metadata parsing error (from Metadata enum).
    #[error("{0}")]
    ParseError(#[from] MetadataParseError),

    /// Malformed integer in Metadata XML database.
    #[error("Malformed integer in Metadata XML: {0}")]
    Integer(#[from] std::num::ParseIntError),

    /// Malformed boolean in Metadata XML database.
    #[error("Malformed boolean in Metadata XML: {0}")]
    Bool(#[from] std::str::ParseBoolError),

    /// I/O error while reading Metadata XML database.
    #[error("I/O-Error in Metadata XML: {0}")]
    Io(#[from] std::io::Error),

    /// Malformed Regex in Metadata XML database.
    #[error("Malformed Regex: {0}")]
    Regex(#[from] regex::Error),
}
