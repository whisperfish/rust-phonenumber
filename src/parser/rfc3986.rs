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

//! Percent-encoding as defined by RFC 3986 section 2.1.
//!
//! `tel:` URIs (RFC 3966) inherit the generic URI escaping rules, so a digit
//! or visual separator may appear pct-encoded (`%2D` for `-`, `%2B` for `+`).
//! These helpers turn such octets back into their literal characters before
//! the number is interpreted.

use std::borrow::Cow;

fn unhex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// True if `value` contains at least one `%`, i.e. it may carry pct-encoded
/// octets.
pub fn is_encoded(value: &str) -> bool {
    value.as_bytes().contains(&b'%')
}

/// Decode the pct-encoded octets in `value` per RFC 3986 section 2.1.
///
/// `pct-encoded = "%" HEXDIG HEXDIG`. Returns `None` if a `%` is not followed
/// by two hexadecimal digits, or if the decoded octets are not valid UTF-8.
/// When `value` carries no `%`, the input is borrowed unchanged.
pub fn decode(value: &str) -> Option<Cow<'_, str>> {
    if !is_encoded(value) {
        return Some(Cow::Borrowed(value));
    }

    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hi = bytes.get(i + 1).copied().and_then(unhex)?;
            let lo = bytes.get(i + 2).copied().and_then(unhex)?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(out).ok().map(Cow::Owned)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn passthrough_is_borrowed() {
        match decode("2034567890") {
            Some(Cow::Borrowed("2034567890")) => {}
            other => panic!("expected borrowed passthrough, got {other:?}"),
        }
    }

    #[test]
    fn decodes_separators() {
        assert_eq!(decode("%2D3-331-6005").unwrap(), "-3-331-6005");
        assert_eq!(decode("%2B64").unwrap(), "+64");
        assert_eq!(decode("650%2E253%2E0000").unwrap(), "650.253.0000");
    }

    #[test]
    fn decodes_lowercase_and_uppercase_hex() {
        assert_eq!(decode("%2d").unwrap(), "-");
        assert_eq!(decode("%2D").unwrap(), "-");
    }

    #[test]
    fn decodes_multibyte_utf8() {
        assert_eq!(decode("%C3%A9").unwrap(), "é");
    }

    #[test]
    fn rejects_truncated_escape() {
        assert!(decode("%2").is_none());
        assert!(decode("%").is_none());
        assert!(decode("12%").is_none());
    }

    #[test]
    fn rejects_non_hex_escape() {
        assert!(decode("%2G").is_none());
        assert!(decode("%XY").is_none());
    }

    #[test]
    fn rejects_invalid_utf8() {
        // 0xA1 is not a valid standalone UTF-8 byte.
        assert!(decode("%A1").is_none());
    }
}
