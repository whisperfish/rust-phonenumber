use std::ops::Deref;
use std::sync::Arc;
use std::str;
use std::path::Path;
use std::fs::File;
use std::io::{self, Cursor, BufRead, BufReader, BufWriter, Write};
use std::env;

use fnv::FnvHashMap;
use regex::{RegexBuilder, Regex};
use xml::reader::Reader;
use xml::events::{self, Event};
use xml::events::attributes::Attribute;

use error::{self, ErrorKind, Result};

const METADATA:  &str = include_str!("../../assets/PhoneNumberMetadata.xml");
const ALTERNATE: &str = include_str!("../../assets/PhoneNumberAlternateFormats.xml");

lazy_static! {
	pub static ref DEFAULT: Database = Database::parse(METADATA).unwrap();
}

#[derive(Clone, Debug)]
pub struct Database {
	map: FnvHashMap<String, super::Metadata>,
}

impl Database {
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
		Database::from(metadata(&mut Reader::from_reader(
			BufReader::new(File::open(path)?)))?)
	}

	pub fn parse<S: AsRef<str>>(content: S) -> Result<Self> {
		Database::from(metadata(&mut Reader::from_reader(
			Cursor::new(content.as_ref())))?)
	}

	fn from(meta: Vec<Metadata>) -> Result<Self> {
		#[inline(always)]
		fn switch<T>(value: Option<Result<T>>) -> Result<Option<T>> {
			match value {
				None =>
					Ok(None),

				Some(Ok(value)) =>
					Ok(Some(value)),

				Some(Err(err)) =>
					Err(err),
			}
		}

		#[inline(always)]
		fn regex(value: String) -> Result<Regex> {
			Ok(RegexBuilder::new(&value).ignore_whitespace(true).build()?)
		}

		fn metadata(meta: Metadata) -> Result<super::Metadata> {
			Ok(super::Metadata {
				general:          switch(meta.general.map(descriptor))?,
				fixed_line:       switch(meta.fixed_line.map(descriptor))?,
				mobile:           switch(meta.mobile.map(descriptor))?,
				toll_free:        switch(meta.toll_free.map(descriptor))?,
				premium_rate:     switch(meta.premium_rate.map(descriptor))?,
				shared_cost:      switch(meta.shared_cost.map(descriptor))?,
				personal:         switch(meta.personal.map(descriptor))?,
				voip:             switch(meta.voip.map(descriptor))?,
				pager:            switch(meta.pager.map(descriptor))?,
				uan:              switch(meta.uan.map(descriptor))?,
				emergency:        switch(meta.emergency.map(descriptor))?,
				voicemail:        switch(meta.voicemail.map(descriptor))?,
				short_code:       switch(meta.short_code.map(descriptor))?,
				standard_rate:    switch(meta.standard_rate.map(descriptor))?,
				carrier:          switch(meta.carrier.map(descriptor))?,
				no_international: switch(meta.no_international.map(descriptor))?,

				id: meta.id.ok_or(error::Metadata::MissingValue("id".into()))?,
				country_code: meta.country_code,

				international_prefix: meta.international_prefix,
				preferred_international_prefix: meta.preferred_international_prefix,
				national_prefix: meta.national_prefix,
				preferred_extension_prefix: meta.preferred_extension_prefix,
				national_prefix_for_parsing: meta.national_prefix_for_parsing,
				national_prefix_transform_rule: meta.national_prefix_transform_rule,

				format: meta.format.into_iter().map(format).collect::<Result<_>>()?,
				international_format: meta.international_format.into_iter().map(format).collect::<Result<_>>()?,

				main_country_for_code: meta.main_country_for_code,
				leading_digits: meta.leading_digits,
				mobile_number_portable: meta.mobile_number_portable,
			})
		}

		fn descriptor(desc: Descriptor) -> Result<super::Descriptor> {
			Ok(super::Descriptor {
				national_number: switch(desc.national_number.map(regex))?,
				possible_number: switch(desc.possible_number.map(regex))?,
				possible_length: desc.possible_length,
				possible_local_length: desc.possible_local_length,
				example: desc.example,
			})
		}

		fn format(format: Format) -> Result<super::Format> {
			Ok(super::Format {
				pattern: format.pattern.ok_or(error::Metadata::MissingValue("format".into()).into()).and_then(regex)?,
				format: format.format.ok_or(error::Metadata::MissingValue("format".into()))?,
				leading_digits: format.leading_digits.into_iter().map(regex).collect::<Result<_>>()?,
				national_prefix: format.national_prefix,
				domestic_carrier: format.domestic_carrier,
			})
		}

		let mut map = FnvHashMap::default();

		for meta in meta {
			let new = metadata(meta)?;
			map.insert(new.id.clone(), new);
		}

		Ok(Database {
			map: map,
		})
	}
}

impl Deref for Database {
	type Target = FnvHashMap<String, super::Metadata>;

	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

#[derive(Clone, Default, Debug)]
struct Defaults {
	format:     Format,
	descriptor: Descriptor,
}

#[derive(Clone, Default, Debug)]
struct Metadata {
	general:          Option<Descriptor>,
	fixed_line:       Option<Descriptor>,
	mobile:           Option<Descriptor>,
	toll_free:        Option<Descriptor>,
	premium_rate:     Option<Descriptor>,
	shared_cost:      Option<Descriptor>,
	personal:         Option<Descriptor>,
	voip:             Option<Descriptor>,
	pager:            Option<Descriptor>,
	uan:              Option<Descriptor>,
	emergency:        Option<Descriptor>,
	voicemail:        Option<Descriptor>,
	short_code:       Option<Descriptor>,
	standard_rate:    Option<Descriptor>,
	carrier:          Option<Descriptor>,
	no_international: Option<Descriptor>,

	id:           Option<String>,
	country_code: Option<u32>,

	international_prefix:           Option<String>,
	preferred_international_prefix: Option<String>,
	national_prefix:                Option<String>,
	preferred_extension_prefix:     Option<String>,

	national_prefix_for_parsing: Option<String>,
	national_prefix_transform_rule: Option<String>,

	format:               Vec<Format>,
	international_format: Vec<Format>,

	main_country_for_code: bool,
	leading_digits: Option<String>,
	mobile_number_portable: bool,

	defaults: Defaults,
}

#[derive(Clone, Default, Debug)]
struct Format {
	pattern: Option<String>,
	format: Option<String>,
	leading_digits: Vec<String>,
	national_prefix: Option<String>,
	national_prefix_formatting_rule: Option<String>,
	national_prefix_optional_when_formatting: bool,
	domestic_carrier: Option<String>,
}

#[derive(Clone, Default, Debug)]
struct Descriptor {
	national_number: Option<String>,
	possible_number: Option<String>,
	possible_length: Vec<u16>,
	possible_local_length: Vec<u16>,
	example: Option<String>,
}

fn metadata<'a, R: BufRead>(reader: &mut Reader<R>) -> Result<Vec<Metadata>> {
	let mut buffer = Vec::new();
	let mut result = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Eof =>
				return Ok(result),

			Event::Start(ref e) => {
				match e.name() {
					b"phoneNumberMetadata" =>
						continue,

					b"territories" =>
						result.extend(territories(reader)?),

					name =>
						ignore(reader, name)?,
				}
			}

			Event::End(ref e) if e.name() != b"phoneNumberMetadata" =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			_ => ()
		}
	}
}

fn territories<'a, R: BufRead>(reader: &mut Reader<R>) -> Result<Vec<Metadata>> {
	let mut buffer = Vec::new();
	let mut result = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					b"territory" =>
						result.push(territory(reader, e)?),

					name =>
						ignore(reader, e.name())?,
				}
			}

			Event::End(ref e) if e.name() == b"territories" =>
				return Ok(result),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn territory<'a, R: BufRead>(reader: &mut Reader<R>, e: &events::BytesStart<'a>) -> Result<Metadata> {
	let mut buffer = Vec::new();
	let mut meta   = Metadata::default();

	for attr in e.attributes() {
		let Attribute { key, value } = attr?;

		match (str::from_utf8(key)?, str::from_utf8(value)?) {
			("id", value) =>
				meta.id = Some(value.into()),

			("countryCode", value) =>
				meta.country_code = Some(value.parse()?),

			("internationalPrefix", value) =>
				meta.international_prefix = Some(value.into()),

			("preferredInternationalPrefix", value) =>
				meta.preferred_international_prefix = Some(value.into()),

			("nationalPrefix", value) =>
				meta.national_prefix = Some(value.into()),

			("preferredExtnPrefix", value) =>
				meta.preferred_extension_prefix = Some(value.into()),

			("nationalPrefixForParsing", value) =>
				meta.national_prefix_for_parsing = Some(value.into()),

			("nationalPrefixTransformRule", value) =>
				meta.national_prefix_transform_rule = Some(value.into()),

			("mainCountryForCode", value) =>
				meta.main_country_for_code = value.parse()?,

			("leadingDigits", value) =>
				meta.leading_digits = Some(value.into()),

			("mobileNumberPortableRegion", value) =>
				meta.mobile_number_portable = value.parse()?,

			("nationalPrefixFormattingRule", value) =>
				meta.defaults.format.national_prefix_formatting_rule = Some(value.into()),

			("nationalPrefixOptionalWhenFormatting", value) =>
				meta.defaults.format.national_prefix_optional_when_formatting = value.parse()?,

			("carrierCodeFormattingRule", value) =>
				meta.defaults.format.domestic_carrier = Some(value.into()),

			(name, value) =>
				return Err(error::Metadata::UnhandledAttribute {
					phase: "format".into(),
					name:  name.into(),
					value: value.into()
				}.into())
		}
	}

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"references" =>
						ignore(reader, name)?,

					name @ b"generalDesc" =>
						meta.general = Some(descriptor(reader, &meta, name)?),

					name @ b"fixedLine" =>
						meta.fixed_line = Some(descriptor(reader, &meta, name)?),

					name @ b"mobile" =>
						meta.mobile = Some(descriptor(reader, &meta, name)?),

					name @ b"tollFree" =>
						meta.toll_free = Some(descriptor(reader, &meta, name)?),

					name @ b"premiumRate" =>
						meta.premium_rate = Some(descriptor(reader, &meta, name)?),

					name @ b"sharedCost" =>
						meta.shared_cost = Some(descriptor(reader, &meta, name)?),

					name @ b"personalNumber" =>
						meta.personal = Some(descriptor(reader, &meta, name)?),

					name @ b"voip" =>
						meta.voip = Some(descriptor(reader, &meta, name)?),

					name @ b"pager" =>
						meta.pager = Some(descriptor(reader, &meta, name)?),

					name @ b"uan" =>
						meta.uan = Some(descriptor(reader, &meta, name)?),

					name @ b"emergency" =>
						meta.emergency = Some(descriptor(reader, &meta, name)?),

					name @ b"voicemail" =>
						meta.voicemail = Some(descriptor(reader, &meta, name)?),

					name @ b"noInternationalDialling" =>
						meta.no_international = Some(descriptor(reader, &meta, name)?),

					name @ b"availableFormats" => {
						let (national, international) = formats(reader, &meta, name)?;

						meta.format               = national;
						meta.international_format = international;
					}

					name @ b"areaCodeOptional" =>
						ignore(reader, name)?,

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "territory".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == b"territory" =>
				return Ok(meta),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn descriptor<R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8]) -> Result<Descriptor> {
	let mut buffer     = Vec::new();
	let mut descriptor = meta.defaults.descriptor.clone();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"nationalNumberPattern" =>
						descriptor.national_number = Some(text(reader, name)?),

					name @ b"exampleNumber" =>
						descriptor.example = Some(text(reader, name)?),

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "descriptor".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok(descriptor),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn formats<R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8]) -> Result<(Vec<Format>, Vec<Format>)> {
	let mut buffer        = Vec::new();
	let mut national      = Vec::new();
	let mut international = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"numberFormat" => {
						let (natl, intl) = format(reader, meta, name, e)?;

						national.push(natl);

						if let Some(intl) = intl {
							international.push(intl);
						}
					}

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "formats".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok((national, international)),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn format<'a, R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8], e: &events::BytesStart<'a>) -> Result<(Format, Option<Format>)> {
	let mut buffer = Vec::new();

	let mut format        = meta.defaults.format.clone();
	let mut international = None;

	for attr in e.attributes() {
		let Attribute { key, value } = attr?;

		match (str::from_utf8(key)?, str::from_utf8(value)?) {
			("pattern", value) =>
				format.pattern = Some(value.into()),

			("nationalPrefixFormattingRule", value) =>
				format.national_prefix_formatting_rule = Some(value.into()),

			("nationalPrefixOptionalWhenFormatting", value) =>
				format.national_prefix_optional_when_formatting = value.parse()?,

			("carrierCodeFormattingRule", value) =>
				format.domestic_carrier = Some(value.into()),

			(name, value) =>
				return Err(error::Metadata::UnhandledAttribute {
					phase: "format".into(),
					name:  name.into(),
					value: value.into()
				}.into())
		}
	}

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"leadingDigits" =>
						format.leading_digits.push(text(reader, name)?),

					name @ b"format" => {
						let text = text(reader, name)?;

						format.format = Some(text.clone());
						international = Some(text);
					}

					name @ b"intlFormat" => {
						let text = text(reader, name)?;

						if text == "NA" {
							international = None;
						}
						else {
							international = Some(text);
						}
					}

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "format".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name => {
				let international = international.map(|v| {
					let mut format = format.clone();
					format.format = Some(v);
					format
				});

				return Ok((format, international));
			}

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn ignore<'a, R: BufRead>(reader: &mut Reader<R>, name: &[u8]) -> Result<()> {
	let mut buffer = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name =>
						ignore(reader, name)?,
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok(()),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn text<'a, R: BufRead>(reader: &mut Reader<R>, name: &[u8]) -> Result<String> {
	let mut buffer = Vec::new();
	let mut result = String::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Text(ref e) =>
				result.push_str(str::from_utf8(e)?),

			Event::End(ref e) if e.name() == name =>
				return Ok(result),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}
