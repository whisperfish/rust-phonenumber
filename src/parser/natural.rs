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
use nom::IResult;
use nom::error::ErrorKind;

pub fn phone_number(i: &str) -> IResult<&str, Number<'_>> {
    let (_, i) = extract(i)?;
    let extension = consts::EXTN_PATTERN.captures(i);

    let (national, extension) = match extension {
        Some(c) => match (c.get(0), c.get(2)) {
            (Some(full), Some(ext)) => (&i[..full.start()], Some(ext.as_str().into())),
            // A malformed extension is a hard error.
            _ => return Err(nom::Err::Failure(nom::error::Error::new(i, ErrorKind::Eof))),
        },
        None => (i, None),
    };

    Ok((
        "",
        Number {
            national: national.into(),
            extension,
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
