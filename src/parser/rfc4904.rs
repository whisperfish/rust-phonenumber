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

//! Trunk-group parameters, defined by RFC 4904.
//!
//! A `tel:` URI may identify the originating or terminating trunk group with
//! the `tgrp` parameter, qualified by a `trunk-context` that names the domain
//! in which the trunk-group label is meaningful, e.g.
//! `tel:+1-555-0100;tgrp=TG-1;trunk-context=example.com`. This is routing
//! metadata rather than part of the E.164 number, so it is exposed through its
//! own parser rather than on [`crate::PhoneNumber`].

/// A trunk group carried by a `tel:` URI.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TrunkGroup {
    /// The trunk-group label (`tgrp`).
    pub group: String,
    /// The `trunk-context` qualifying the label, if present.
    pub context: Option<String>,
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

/// Extract the trunk group from a `tel:` URI, if one is present.
pub fn trunk_group(uri: &str) -> Option<TrunkGroup> {
    let group = param(uri, "tgrp")?;

    Some(TrunkGroup {
        group: group.to_owned(),
        context: param(uri, "trunk-context").map(str::to_owned),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn absent_when_no_tgrp() {
        assert_eq!(trunk_group("tel:+1-555-0100"), None);
        assert_eq!(trunk_group("tel:+1-555-0100;trunk-context=example.com"), None);
    }

    #[test]
    fn extracts_group() {
        assert_eq!(
            trunk_group("tel:+1-555-0100;tgrp=TG-1"),
            Some(TrunkGroup {
                group: "TG-1".into(),
                context: None,
            })
        );
    }

    #[test]
    fn extracts_group_and_context() {
        assert_eq!(
            trunk_group("tel:+1-555-0100;tgrp=TG-1;trunk-context=example.com"),
            Some(TrunkGroup {
                group: "TG-1".into(),
                context: Some("example.com".into()),
            })
        );
    }

    #[test]
    fn order_independent_and_case_insensitive() {
        assert_eq!(
            trunk_group("tel:+64-3-331-6005;trunk-context=example.com;TGRP=cust1;ext=11"),
            Some(TrunkGroup {
                group: "cust1".into(),
                context: Some("example.com".into()),
            })
        );
    }
}
