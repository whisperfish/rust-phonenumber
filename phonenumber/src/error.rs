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

pub use phonenumber_metadata::error::MetadataParseError;

/// Parsing errors.
#[derive(Error, Clone, Debug)]
// This module is used in build.rs, and only some public items are used there.
#[allow(dead_code)]
pub enum Parse {
    /// This generally indicates the string passed in had less than 3 digits in
    /// it.
    #[error("not a number")]
    NoNumber,

    /// The country code supplied did not belong to a supported country or
    /// non-geographical entity.
    #[error("invalid country code")]
    InvalidCountryCode,

    /// This indicates the string started with an international dialing prefix,
    /// but after this was stripped from the number, had less digits than any
    /// valid phone number (including country code) could have.
    #[error("the number is too short after IDD")]
    TooShortAfterIdd,

    /// This indicates the string, after any country code has been stripped, had
    /// less digits than any valid phone number could have.
    #[error("the number is too short after the country code")]
    TooShortNsn,

    /// This indicates the string had more digits than any valid phone number
    /// could have.
    #[error("the number is too long")]
    TooLong,

    /// A integer parts of a number is malformed, normally this should be caught by the parsing regexes.
    #[error("malformed integer part in phone number: {0}")]
    MalformedInteger(#[from] std::num::ParseIntError),
}
