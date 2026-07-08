use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use log::info;
use serde::Serialize;
use serde::de::{DeserializeOwned, Deserializer, Error, Visitor};

pub(super) struct DeserializeVersionHelper<const VER: u64> {}

impl<const VER: u64> DeserializeVersionHelper<VER>
{
	pub fn deserialize_version<'de, D>(deserializer: D) -> Result<u64, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct VersionVisitor<const EXPECTED_VERSION: u64>;

		impl<'de, const EXPECTED_VERSION: u64> Visitor<'de> for VersionVisitor<EXPECTED_VERSION>
		{
			type Value = u64;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
			{
				write!(formatter, "an unsigned version number")
			}

			fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				if v != EXPECTED_VERSION
				{
					return Err(Error::custom(format!(
						"Expected version {EXPECTED_VERSION} but got version {v}"
					)));
				}

				return Ok(v);
			}
		}

		let visitor: VersionVisitor<VER> = VersionVisitor;
		return deserializer.deserialize_u64(visitor);
	}
}

pub trait VersionedIOFormat<const VER: u64>
{
	type InnerFormat;
	type FileFormat;
	const VERSION: u64 = VER;

	fn type_desc() -> &'static str;

	fn serialize<'l, Writer>(writer: Writer, inner: &'l Self::InnerFormat) -> Result<()>
	where
		Writer: Write,
		Self::FileFormat: From<&'l Self::InnerFormat> + Serialize,
	{
		return serde_json::to_writer(writer, &Self::FileFormat::from(inner))
			.with_context(|| format!("Failed to serialise version {VER} {}", Self::type_desc()));
	}

	fn deserialize<Reader>(reader: Reader) -> Result<Self::InnerFormat>
	where
		Reader: Read,
		Self::FileFormat: Into<Self::InnerFormat> + DeserializeOwned,
	{
		let wrapper: Self::FileFormat = serde_json::from_reader(reader).with_context(|| {
			format!("Failed to deserialise version {VER} {}", Self::type_desc())
		})?;

		return Ok(wrapper.into());
	}

	fn write<'l>(path: &Path, inner: &'l Self::InnerFormat) -> Result<()>
	where
		Self::FileFormat: From<&'l Self::InnerFormat> + Serialize,
	{
		info!("Dumping {} to {}", Self::type_desc(), path.display());

		let out_file: File = File::create(path)
			.with_context(|| format!("Failed to open file {} for writing", path.display()))?;

		return Self::serialize(out_file, inner);
	}

	fn read(path: &Path) -> Result<Self::InnerFormat>
	where
		Self::FileFormat: Into<Self::InnerFormat> + DeserializeOwned,
	{
		info!("Reading {} from {}", Self::type_desc(), path.display());

		let in_file: File = File::open(path)
			.with_context(|| format!("Failed to open file {} for reading", path.display()))?;

		return Self::deserialize(in_file);
	}
}
