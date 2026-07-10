use std::io::{Read, Write};

use super::helpers::VersionedIOFormat;
use crate::configs::GameConfig;
use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use toml;

pub struct GameConfigIOFormatV1;

impl VersionedIOFormat for GameConfigIOFormatV1
{
	type InnerFormat = GameConfig;
	type SerializableFormat = version_1::V1File;

	fn serialize_impl<Writer, OutFmt>(mut writer: Writer, data: &OutFmt) -> Result<()>
	where
		Writer: Write,
		OutFmt: Serialize,
	{
		let toml_string: String = toml::to_string(data)?;
		writer.write_all(toml_string.as_bytes())?;
		return Ok(());
	}

	fn deserialize_impl<Reader, InFmt>(mut reader: Reader) -> Result<InFmt>
	where
		Reader: Read,
		InFmt: DeserializeOwned,
	{
		let mut toml_string: String = String::new();
		reader.read_to_string(&mut toml_string)?;
		let doc = toml::from_str(&toml_string)?;
		return Ok(doc);
	}
}

mod version_1;
