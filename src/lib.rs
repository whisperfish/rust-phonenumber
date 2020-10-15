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

#![recursion_limit = "1024"]

#[macro_use]
extern crate lazy_static;

extern crate thiserror;
#[macro_use]
extern crate nom;

extern crate regex;
extern crate regex_cache;
extern crate regex_syntax;
extern crate fnv;
extern crate quick_xml as xml;
extern crate itertools;
extern crate either;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

#[cfg(test)]
#[macro_use]
extern crate doc_comment;

#[cfg(test)]
doctest!("../README.md");

#[macro_use]
mod helper;

/// Errors for various parts of the crate.
mod error;
pub use crate::error::{Metadata as MetadataError, Parse as ParseError};

/// Phone number metadata, containing patterns, formatting and other useful
/// data about countries and phone numbers.
pub mod metadata;
pub use crate::metadata::Metadata;

/// Country related types.
pub mod country;

mod consts;

mod national_number;
pub use crate::national_number::NationalNumber;


mod extension;
pub use crate::extension::Extension;

mod carrier;
pub use crate::carrier::Carrier;

mod phone_number;
pub use crate::phone_number::{PhoneNumber, Type};

pub mod parser;
pub use crate::parser::{parse, parse_with};

mod formatter;
pub use crate::formatter::{Mode, Formatter, format, format_with};

mod validator;
pub use crate::validator::{Validation, is_viable, is_valid, is_valid_with};
