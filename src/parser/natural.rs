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

use crate::consts;
use crate::parser::helper::*;
use nom::error::ErrorKind;
use nom::IResult;

pub fn phone_number(i: &str) -> IResult<&str, Number<'_>> {
    let (_, i) = extract(i)?;
    let extension = consts::EXTN_PATTERN.captures(i);

    if let Some(c) = extension.as_ref() {
        if c.get(0).is_none() || c.get(2).is_none() {
            return Err(nom::Err::Failure(nom::error::Error::new(i, ErrorKind::Eof)));
        }
    }

    Ok((
        "",
        Number {
            national: extension
                .as_ref()
                .map(|c| &i[..c.get(0).unwrap().start()])
                .unwrap_or(i)
                .into(),

            extension: extension
                .as_ref()
                .map(|c| c.get(2).unwrap().as_str())
                .map(Into::into),

            ..Default::default()
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::parser::helper::*;
    use crate::parser::natural;

    #[test]
    fn phone_number() {
        assert_eq!(
            natural::phone_number("650 253 0000 extn. 4567").unwrap().1,
            Number {
                national: "650 253 0000".into(),
                extension: Some("4567".into()),

                ..Default::default()
            }
        );
    }
}
