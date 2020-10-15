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

use std::path::Path;
use std::fs::File;
use std::io::{Cursor, BufReader};
use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use bincode::Options;
use fnv::FnvHashMap;
use regex_cache::{RegexCache, CachedRegex, CachedRegexBuilder};
use bincode;

use crate::error;
use crate::metadata::loader;

const DATABASE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/database.bin"));

lazy_static! {
	/// The Google provided metadata database, used as default.
	pub static ref DEFAULT: Database =
		Database::from(bincode::options()
		.with_varint_encoding().deserialize(DATABASE).unwrap(), false).unwrap();
}

/// Representation of a database of metadata for phone number.
#[derive(Clone, Debug)]
pub struct Database {
	cache:   Arc<Mutex<RegexCache>>,
	by_id:   FnvHashMap<String, Arc<super::Metadata>>,
	by_code: FnvHashMap<u16, Vec<Arc<super::Metadata>>>,
	regions: FnvHashMap<u16, Vec<String>>,
}

impl Database {
	/// Load a database from the given file.
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, error::LoadMetadata> {
		Database::from(loader::load(BufReader::new(File::open(path)?))?, false)
	}

	/// Parse a database from the given string.
	pub fn parse<S: AsRef<str>>(content: S) -> Result<Self, error::LoadMetadata> {
		Database::from(loader::load(Cursor::new(content.as_ref()))?, false)
	}

	/// Create a database from a loaded database.
	pub fn from(meta: Vec<loader::Metadata>, check_regex: bool) -> Result<Self, error::LoadMetadata> {
		fn tranpose<T, E>(value: Option<Result<T, E>>) -> Result<Option<T>, E> {
			match value {
				None =>
					Ok(None),

				Some(Ok(value)) =>
					Ok(Some(value)),

				Some(Err(err)) =>
					Err(err),
			}
		}

		let cache = Arc::new(Mutex::new(RegexCache::new(100)));
		let regex = |value: String| -> Result<CachedRegex, error::LoadMetadata> {
			if check_regex {
				Ok(CachedRegexBuilder::new(cache.clone(), &value)
					.ignore_whitespace(true).build()?)
			} else {
				// the regex can be added to the cache without a syntax check as the syntax 
				// has already been checked by the metadata loader at build time
				Ok(CachedRegexBuilder::new(cache.clone(), &value)
					.ignore_whitespace(true).build_unchecked())
			}
		};

		let descriptor = |desc: loader::Descriptor| -> Result<super::Descriptor, error::LoadMetadata> {
			desc.national_number.as_ref().unwrap();
			desc.national_number.as_ref().unwrap();

			Ok(super::Descriptor {
				national_number: desc.national_number.ok_or_else(||
					error::LoadMetadata::from(error::Metadata::MissingValue {
						phase: "descriptor".into(),
						name:  "national_number".into(),
					})).and_then(&regex)?,

				possible_length: desc.possible_length,
				possible_local_length: desc.possible_local_length,
				example: desc.example,
			})
		};

		let format = |format: loader::Format| -> Result<super::Format, error::LoadMetadata> {
			Ok(super::Format {
				pattern: format.pattern.ok_or_else(||
					error::LoadMetadata::from(error::Metadata::MissingValue {
						phase: "format".into(),
						name:  "pattern".into(),
					})).and_then(&regex)?,

				format: format.format.ok_or_else(||
					error::LoadMetadata::from(error::Metadata::MissingValue {
						phase: "format".into(),
						name:  "format".into()
					}))?,

				leading_digits: format.leading_digits.into_iter()
					.map(&regex).collect::<Result<_, _>>()?,

				national_prefix:          format.national_prefix_formatting_rule,
				national_prefix_optional: format.national_prefix_optional_when_formatting,

				domestic_carrier: format.domestic_carrier,
			})
		};

		let metadata = |meta: loader::Metadata| -> Result<super::Metadata, error::LoadMetadata> {
			Ok(super::Metadata {
				descriptors: super::Descriptors {
					general: descriptor(meta.general.ok_or_else(||
						error::LoadMetadata::from(error::Metadata::MissingValue {
							phase: "metadata".into(),
							name:  "generalDesc".into(),
						}))?)?,

					fixed_line:       tranpose(meta.fixed_line.map(&descriptor))?,
					mobile:           tranpose(meta.mobile.map(&descriptor))?,
					toll_free:        tranpose(meta.toll_free.map(&descriptor))?,
					premium_rate:     tranpose(meta.premium_rate.map(&descriptor))?,
					shared_cost:      tranpose(meta.shared_cost.map(&descriptor))?,
					personal_number:  tranpose(meta.personal_number.map(&descriptor))?,
					voip:             tranpose(meta.voip.map(&descriptor))?,
					pager:            tranpose(meta.pager.map(&descriptor))?,
					uan:              tranpose(meta.uan.map(&descriptor))?,
					emergency:        tranpose(meta.emergency.map(&descriptor))?,
					voicemail:        tranpose(meta.voicemail.map(&descriptor))?,
					short_code:       tranpose(meta.short_code.map(&descriptor))?,
					standard_rate:    tranpose(meta.standard_rate.map(&descriptor))?,
					carrier:          tranpose(meta.carrier.map(&descriptor))?,
					no_international: tranpose(meta.no_international.map(&descriptor))?,
				},

				id: meta.id.ok_or_else(||
					error::LoadMetadata::from(error::Metadata::MissingValue {
						phase: "metadata".into(),
						name:  "id".into()
					}))?,

				country_code: meta.country_code.ok_or_else(||
					error::LoadMetadata::from(error::Metadata::MissingValue {
						phase: "metadata".into(),
						name: "countryCode".into(),
					}))?,

				international_prefix: tranpose(meta.international_prefix.map(&regex))?,
				preferred_international_prefix: meta.preferred_international_prefix,
				national_prefix: meta.national_prefix,
				preferred_extension_prefix: meta.preferred_extension_prefix,
				national_prefix_for_parsing: tranpose(meta.national_prefix_for_parsing.map(&regex))?,
				national_prefix_transform_rule: meta.national_prefix_transform_rule,

				formats: meta.formats.into_iter().map(&format).collect::<Result<_, _>>()?,
				international_formats: meta.international_formats.into_iter().map(&format).collect::<Result<_, _>>()?,

				main_country_for_code: meta.main_country_for_code,
				leading_digits: tranpose(meta.leading_digits.map(&regex))?,
				mobile_number_portable: meta.mobile_number_portable,
			})
		};

		let mut by_id   = FnvHashMap::default();
		let mut by_code = FnvHashMap::default();
		let mut regions = FnvHashMap::default();

		for meta in meta {
			let meta = Arc::new(metadata(meta)?);

			by_id.insert(meta.id.clone(), meta.clone());

			let by_code = by_code.entry(meta.country_code)
				.or_insert_with(Vec::new);

			let regions = regions.entry(meta.country_code)
				.or_insert_with(Vec::new);

			if meta.main_country_for_code {
				by_code.insert(0, meta.clone());
				regions.insert(0, meta.id.clone())
			}
			else {
				by_code.push(meta.clone());
				regions.push(meta.id.clone());
			}
		}

		Ok(Database {
			cache:   cache.clone(),
			by_id:   by_id,
			by_code: by_code,
			regions: regions,
		})
	}

	/// Get the regular expression cache.
	pub fn cache(&self) -> Arc<Mutex<RegexCache>> {
		self.cache.clone()
	}

	/// Get a metadata entry by country ID.
	pub fn by_id<Q>(&self, key: &Q) -> Option<&super::Metadata>
		where Q:      ?Sized + Hash + Eq,
		      String: Borrow<Q>,
	{
		self.by_id.get(key).map(AsRef::as_ref)
	}

	/// Get metadata entries by country code.
	pub fn by_code<Q>(&self, key: &Q) -> Option<Vec<&super::Metadata>>
		where Q:   ?Sized + Hash + Eq,
		      u16: Borrow<Q>,
	{
		self.by_code.get(key).map(|m| m.iter().map(AsRef::as_ref).collect())
	}

	/// Get all country IDs corresponding to the given country code.
	pub fn region<Q>(&self, code: &Q) -> Option<Vec<&str>>
		where Q:   ?Sized + Hash + Eq,
		      u16: Borrow<Q>
	{
		self.regions.get(code).map(|m| m.iter().map(AsRef::as_ref).collect())
	}
}
