#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;

extern crate regex;
extern crate fnv;
extern crate quick_xml as xml;

pub mod error;
pub use error::{Error, ErrorKind, Result};

pub mod metadata;
pub use metadata::Metadata;

mod national_number;
pub use national_number::NationalNumber;

pub mod country_code;
pub use country_code::CountryCode;

mod extension;
pub use extension::Extension;

mod phone_number;
pub use phone_number::PhoneNumber;
