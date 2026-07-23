use std::ops::Deref;

use serde::de::Deserialize;
use serde::ser::Serialize;
use serde_valid::ValidateMinLength;

#[derive(Debug, Clone)]
pub struct TrimmedString
{
	inner: String,
}

impl From<&str> for TrimmedString
{
	fn from(value: &str) -> Self
	{
		return Self {
			inner: value.trim().to_owned(),
		};
	}
}

impl From<String> for TrimmedString
{
	fn from(value: String) -> Self
	{
		return Self::from(value.as_str());
	}
}

impl From<TrimmedString> for String
{
	fn from(value: TrimmedString) -> Self
	{
		return value.inner;
	}
}

impl Deref for TrimmedString
{
	type Target = str;

	fn deref(&self) -> &Self::Target
	{
		return self.inner.as_str();
	}
}

impl Serialize for TrimmedString
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		return self.inner.serialize(serializer);
	}
}

impl<'de> Deserialize<'de> for TrimmedString
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		return Ok(Self::from(String::deserialize(deserializer)?));
	}
}

impl ValidateMinLength for TrimmedString
{
	fn validate_min_length(&self, min_length: usize) -> Result<(), serde_valid::MinLengthError>
	{
		return self.inner.validate_min_length(min_length);
	}
}
