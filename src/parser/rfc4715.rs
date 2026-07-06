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

//! ISDN sub-address (`isub`) parameter, defined by RFC 4715.
//!
//! A `tel:` URI may carry an ISDN sub-address alongside the dialled number,
//! e.g. `tel:+1-212-555-0100;isub=12345`. RFC 4715 adds the optional
//! `isub-encoding` parameter (`nsap`, `nsap-ia5`, ...) describing how the
//! sub-address octets are represented. This is auxiliary routing data, not
//! part of the E.164 number, so it is exposed through its own parser rather
//! than on [`crate::PhoneNumber`].

/// An ISDN sub-address carried by a `tel:` URI.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Subaddress {
    /// The raw sub-address value, as it appeared in the URI.
    pub value: String,
    /// The `isub-encoding` parameter, if present.
    pub encoding: Option<String>,
}

/// Return the value of the `;name=value` parameter in a `tel:` URI.
///
/// Parameter names are matched case-insensitively. The value runs to the next
/// `;` or the end of the string.
fn param<'a>(uri: &'a str, name: &str) -> Option<&'a str> {
    // The first `;`-delimited segment is the number itself, never a parameter.
    uri.split(';').skip(1).find_map(|seg| {
        let (key, value) = seg.split_once('=')?;
        key.trim().eq_ignore_ascii_case(name).then_some(value)
    })
}

/// Extract the ISDN sub-address from a `tel:` URI, if one is present.
pub fn subaddress(uri: &str) -> Option<Subaddress> {
    let value = param(uri, "isub")?;

    Some(Subaddress {
        value: value.to_owned(),
        encoding: param(uri, "isub-encoding").map(str::to_owned),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn absent_when_no_isub() {
        assert_eq!(subaddress("tel:+1-212-555-0100"), None);
        assert_eq!(subaddress("tel:+1-212-555-0100;ext=42"), None);
    }

    #[test]
    fn extracts_value() {
        assert_eq!(
            subaddress("tel:+1-212-555-0100;isub=12345"),
            Some(Subaddress {
                value: "12345".into(),
                encoding: None,
            })
        );
    }

    #[test]
    fn extracts_encoding() {
        assert_eq!(
            subaddress("tel:+1-212-555-0100;isub=12345;isub-encoding=nsap"),
            Some(Subaddress {
                value: "12345".into(),
                encoding: Some("nsap".into()),
            })
        );
    }

    #[test]
    fn order_independent_and_case_insensitive() {
        assert_eq!(
            subaddress("tel:+64-3-331-6005;phone-context=+64;ISUB=98765;ext=11"),
            Some(Subaddress {
                value: "98765".into(),
                encoding: None,
            })
        );
    }

    #[test]
    fn isub_encoding_alone_is_not_a_subaddress() {
        // Without an `isub` there is no sub-address, even if an encoding is set.
        assert_eq!(subaddress("tel:+1-212-555-0100;isub-encoding=nsap"), None);
    }
}
