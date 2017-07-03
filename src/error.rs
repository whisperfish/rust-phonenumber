error_chain! {
	errors {
		Metadata(error: Metadata) {
			description("An error occurred while parsing the metadata.")
			display("Metadata parsing error: `{:?}`", error)
		}
	}

	foreign_links {
		Io(::std::io::Error);
		Xml(::xml::errors::Error);
		Utf8(::std::str::Utf8Error);
		ParseInt(::std::num::ParseIntError);
		ParseBool(::std::str::ParseBoolError);
		Regex(::regex::Error);
	}
}

impl From<Metadata> for Error {
	fn from(error: Metadata) -> Self {
		ErrorKind::Metadata(error).into()
	}
}

impl From<Metadata> for ErrorKind {
	fn from(error: Metadata) -> Self {
		ErrorKind::Metadata(error)
	}
}

#[derive(Clone, Debug)]
pub enum Metadata {
	UnexpectedEof,
	MismatchedTag(String),
	MissingValue(String),

	UnhandledElement {
		phase: String,
		name:  String
	},

	UnhandledAttribute {
		phase: String,
		name:  String,
		value: String,
	},
}
