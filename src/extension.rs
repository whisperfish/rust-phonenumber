use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Extension(String);

impl Deref for Extension {
	type Target = str;

	fn deref(&self) -> &str {
		&self.0
	}
}
