use std::fs::File;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::Path;

use anyhow::{Context, Result};
use log::info;
use serde::de::{DeserializeOwned, Deserializer, Error as DeError, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

pub trait IOFormat
{
	fn format_name() -> &'static str;
	fn format_version() -> u64;
	fn type_desc() -> &'static str;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IOFmtSignature<Parent>
where
	Parent: IOFormat,
{
	#[serde(skip)]
	parent: PhantomData<Parent>,

	#[serde(deserialize_with = "IOFmtSignature::<Parent>::deserialize_format")]
	format: String,

	#[serde(deserialize_with = "IOFmtSignature::<Parent>::deserialize_version")]
	version: u64,
}

impl<Parent> IOFmtSignature<Parent>
where
	Parent: IOFormat,
{
	pub fn new() -> Self
	{
		return Self {
			parent: PhantomData,
			format: Parent::format_name().to_owned(),
			version: Parent::format_version(),
		};
	}

	fn deserialize_version<'de, D>(deserializer: D) -> Result<u64, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct VersionVisitor<Parent>
		{
			parent: PhantomData<Parent>,
		}

		impl<'de, Parent> Visitor<'de> for VersionVisitor<Parent>
		where
			Parent: IOFormat,
		{
			type Value = u64;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
			{
				let expected_version: u64 = Parent::format_version();
				write!(formatter, "format version `{expected_version}`")
			}

			fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
			where
				E: DeError,
			{
				let expected_version: u64 = Parent::format_version();

				if v != expected_version
				{
					return Err(DeError::invalid_value(Unexpected::Unsigned(v), &self));
				}

				return Ok(v);
			}

			// Some formats only support signed integers, so cater for this too.
			fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
			where
				E: DeError,
			{
				let expected_version: u64 = Parent::format_version();

				if v < 0 || (v as u64) != expected_version
				{
					return Err(DeError::invalid_value(Unexpected::Signed(v), &self));
				}

				return Ok(v as u64);
			}
		}

		let visitor: VersionVisitor<Parent> = VersionVisitor {
			parent: PhantomData,
		};

		return deserializer.deserialize_u64(visitor);
	}

	fn deserialize_format<'de, D>(deserializer: D) -> Result<String, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct FormatVisitor<Parent>
		{
			parent: PhantomData<Parent>,
		}

		impl<'de, Parent> Visitor<'de> for FormatVisitor<Parent>
		where
			Parent: IOFormat,
		{
			type Value = String;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
			{
				let expected_format: &'static str = Parent::format_name();
				write!(formatter, "format name \"{expected_format}\"")
			}

			fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
			where
				E: DeError,
			{
				let expected_format: &'static str = Parent::format_name();

				if v != expected_format
				{
					return Err(DeError::invalid_value(Unexpected::Str(v), &self));
				}

				return Ok(v.to_owned());
			}
		}

		let visitor: FormatVisitor<Parent> = FormatVisitor {
			parent: PhantomData,
		};

		return deserializer.deserialize_string(visitor);
	}
}

pub trait VersionedIOFormat
{
	type SerializableFormat;
	type InnerFormat;

	fn serialize<'l, Writer>(writer: Writer, inner: &'l Self::InnerFormat) -> Result<()>
	where
		Writer: Write,
		Self::SerializableFormat: From<&'l Self::InnerFormat> + Serialize + IOFormat,
	{
		let result: Result<()> = <Self as VersionedIOFormat>::serialize_impl(
			writer,
			&Self::SerializableFormat::from(inner),
		);

		return result.with_context(|| {
			format!(
				"Failed to serialise version {} {}",
				Self::SerializableFormat::format_version(),
				Self::SerializableFormat::type_desc()
			)
		});
	}

	fn deserialize<Reader>(reader: Reader) -> Result<Self::InnerFormat>
	where
		Reader: Read,
		Self::SerializableFormat: Into<Self::InnerFormat> + DeserializeOwned + IOFormat,
	{
		let wrapper: Self::SerializableFormat =
			<Self as VersionedIOFormat>::deserialize_impl(reader).with_context(|| {
				format!(
					"Failed to deserialise version {} {}",
					Self::SerializableFormat::format_version(),
					Self::SerializableFormat::type_desc()
				)
			})?;

		return Ok(wrapper.into());
	}

	fn write<'l>(path: &Path, inner: &'l Self::InnerFormat) -> Result<()>
	where
		Self::SerializableFormat: From<&'l Self::InnerFormat> + Serialize + IOFormat,
	{
		info!(
			"Dumping {} to {}",
			Self::SerializableFormat::type_desc(),
			path.display()
		);

		let out_file: File = File::create(path)
			.with_context(|| format!("Failed to open file {} for writing", path.display()))?;

		return Self::serialize(out_file, inner);
	}

	fn read(path: &Path) -> Result<Self::InnerFormat>
	where
		Self::SerializableFormat: Into<Self::InnerFormat> + DeserializeOwned + IOFormat,
	{
		info!(
			"Reading {} from {}",
			Self::SerializableFormat::type_desc(),
			path.display()
		);

		let in_file: File = File::open(path)
			.with_context(|| format!("Failed to open file {} for reading", path.display()))?;

		return Self::deserialize(in_file);
	}

	// JSON by default, but implementers can override this.
	fn serialize_impl<Writer>(writer: Writer, data: &Self::SerializableFormat) -> Result<()>
	where
		Writer: Write,
		Self::SerializableFormat: Serialize,
	{
		return Ok(serde_json::to_writer(writer, data)?);
	}

	// JSON by default, but implementers can override this.
	fn deserialize_impl<Reader>(reader: Reader) -> Result<Self::SerializableFormat>
	where
		Reader: Read,
		Self::SerializableFormat: DeserializeOwned,
	{
		return Ok(serde_json::from_reader(reader)?);
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use serde_json;

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct ParentStruct
	{
		signature: IOFmtSignature<ParentStruct>,
		value: String,
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	struct FlatParentStruct
	{
		#[serde(flatten)]
		signature: IOFmtSignature<ParentStruct>,
		value: String,
	}

	impl IOFormat for ParentStruct
	{
		fn format_name() -> &'static str
		{
			return "dummy_format";
		}

		fn format_version() -> u64
		{
			return 1234;
		}

		fn type_desc() -> &'static str
		{
			return "dummy file format";
		}
	}

	impl IOFormat for FlatParentStruct
	{
		fn format_name() -> &'static str
		{
			return "dummy_format";
		}

		fn format_version() -> u64
		{
			return 1234;
		}

		fn type_desc() -> &'static str
		{
			return "dummy file format";
		}
	}

	#[test]
	fn serialize_and_deserialize_signature()
	{
		let data: ParentStruct = ParentStruct {
			signature: IOFmtSignature::new(),
			value: "test value".to_owned(),
		};

		let json_string: String = serde_json::to_string(&data).unwrap();

		let deserialized_data: ParentStruct =
			serde_json::from_str::<ParentStruct>(&json_string).unwrap();

		assert_eq!(data, deserialized_data);
	}

	#[test]
	fn deserialize_incorrect_format_name()
	{
		let raw_json: &str =
			r##"{"signature":{"format":"wrong name", "version":1234}, "value":"hello"}"##;

		let deserialized_data = serde_json::from_str::<ParentStruct>(&raw_json);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let prefix: &str =
			"invalid value: string \"wrong name\", expected format name \"dummy_format\"";

		assert!(
			error_string.starts_with(prefix),
			"Error string:\n  \"{error_string}\"\nshould start with prefix\n  \"{prefix}\""
		);
	}

	#[test]
	fn deserialize_incorrect_format_version()
	{
		let raw_json: &str =
			r##"{"signature":{"format":"dummy_format", "version":99}, "value":"hello"}"##;

		let deserialized_data = serde_json::from_str::<ParentStruct>(&raw_json);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let prefix: &str = "invalid value: integer `99`, expected format version `1234`";

		assert!(
			error_string.starts_with(prefix),
			"Error string:\n  \"{error_string}\"\nshould start with prefix\n  \"{prefix}\""
		);
	}

	#[test]
	fn deserialize_with_no_signature()
	{
		let raw_json: &str = r##"{"value":"hello"}"##;
		let deserialized_data = serde_json::from_str::<ParentStruct>(&raw_json);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let prefix: &str = "missing field `signature`";

		assert!(
			error_string.starts_with(prefix),
			"Error string:\n  \"{error_string}\"\nshould start with prefix\n  \"{prefix}\""
		);
	}

	#[test]
	fn serialize_flat_signature()
	{
		let data: FlatParentStruct = FlatParentStruct {
			signature: IOFmtSignature::new(),
			value: "test value".to_owned(),
		};

		let json_string: String = serde_json::to_string(&data).unwrap();

		assert_eq!(
			json_string,
			r##"{"format":"dummy_format","version":1234,"value":"test value"}"##
		);
	}

	#[test]
	fn deserialize_incorrect_flat_format_name()
	{
		let raw_json: &str = r##"{"format":"wrong name", "version":1234, "value":"hello"}"##;

		let deserialized_data = serde_json::from_str::<FlatParentStruct>(&raw_json);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let prefix: &str =
			"invalid value: string \"wrong name\", expected format name \"dummy_format\"";

		assert!(
			error_string.starts_with(prefix),
			"Error string:\n  \"{error_string}\"\nshould start with prefix\n  \"{prefix}\""
		);
	}

	#[test]
	fn deserialize_incorrect_flat_format_version()
	{
		let raw_json: &str = r##"{"format":"dummy_format", "version":99, "value":"hello"}"##;

		let deserialized_data = serde_json::from_str::<FlatParentStruct>(&raw_json);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let prefix: &str = "invalid value: integer `99`, expected format version `1234`";

		assert!(
			error_string.starts_with(prefix),
			"Error string:\n  \"{error_string}\"\nshould start with prefix\n  \"{prefix}\""
		);
	}
}
