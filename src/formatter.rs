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

use std::io::{self, Write};

use metadata::Database;
use phone_number::PhoneNumber;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Type {
	E164,
	International,
	National,
	Rfc3966,
}

pub fn format<W: Write>(database: &Database, kind: Type, number: &PhoneNumber, mut out: W) -> io::Result<()> {
	write!(&mut out, "+{} ", number.country_code.value)?;

	if let Some(zeroes) = number.national_number.zeroes {
		for _ in 0 .. zeroes {
			write!(&mut out, "0")?;
		}
	}
	
	write!(&mut out, "{}", number.national_number.value)?;

	if let Some(extension) = number.extension.as_ref() {
		write!(&mut out, " extn. {}", extension.0)?;
	}

	Ok(())
}
