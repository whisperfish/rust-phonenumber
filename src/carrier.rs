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

use serde_derive::{Deserialize, Serialize};
use std::fmt;

use crate::ParseError;

/// A phone number carrier.
///
/// See: https://en.wikipedia.org/wiki/Mobile_country_code#National_operators
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Debug)]
pub struct Carrier {
    pub mcc: u16, // always 3 digits
    pub mnc: u16, // 2 or 3 digits
    pub mnc_3: bool,
}

impl TryFrom<&str> for Carrier {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            mcc: value
                .get(0..3)
                .and_then(|c| c.parse().ok())
                .ok_or(ParseError::InvalidCountryCode)?,
            mnc: value
                .get(3..)
                .and_then(|c| c.parse().ok())
                .ok_or(ParseError::InvalidNetworkCode)?,
            mnc_3: value.len() == 6,
        })
    }
}

impl fmt::Display for Carrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mnc_3 {
            write!(f, "{:03}{:03}", self.mcc, self.mnc)
        } else {
            write!(f, "{:03}{:02}", self.mcc, self.mnc)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Carrier;
    use std::convert::TryInto;

    #[test]
    fn test_mobile_network_codes() {
        assert_eq!(
            Carrier {
                mcc: 336,
                mnc: 1,
                mnc_3: true
            },
            "336001".try_into().unwrap()
        );
        assert_eq!(
            Carrier {
                mcc: 336,
                mnc: 35,
                mnc_3: false
            },
            "33635".try_into().unwrap()
        );
    }
}
