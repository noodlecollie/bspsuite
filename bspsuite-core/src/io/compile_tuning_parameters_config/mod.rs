use std::io::{Read, Write};

use super::helpers::VersionedIOFormat;
use crate::configs::CompileTuningParametersConfig;
use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use toml;

pub struct CompileTuningParametersConfigIOFormatV1;

impl VersionedIOFormat for CompileTuningParametersConfigIOFormatV1
{
	type InnerFormat = CompileTuningParametersConfig;
	type SerializableFormat = version_1::V1File;

	fn serialize_impl<Writer>(mut writer: Writer, data: &Self::SerializableFormat) -> Result<()>
	where
		Writer: Write,
		Self::SerializableFormat: Serialize,
	{
		let toml_string: String = toml::to_string(data)?;
		writer.write_all(toml_string.as_bytes())?;
		return Ok(());
	}

	fn deserialize_impl<Reader>(mut reader: Reader) -> Result<Self::SerializableFormat>
	where
		Reader: Read,
		Self::SerializableFormat: DeserializeOwned,
	{
		let mut toml_string: String = String::new();
		reader.read_to_string(&mut toml_string)?;
		let doc = toml::from_str(&toml_string)?;
		return Ok(doc);
	}
}

mod version_1;
