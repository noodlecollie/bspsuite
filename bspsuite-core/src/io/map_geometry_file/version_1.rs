use std::result::Result;

use serde::Deserialize;
use serde::de::Error as DeError;
use serde::de::Visitor;
use serde::{Deserializer, Serialize};

use crate::model::MapGeomFile;

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Debug)]
pub(super) struct MapGeomFileSer<'l, const VER: u64>
{
	version: u64,
	map: &'l MapGeomFile,
}

impl<'l, const VER: u64> MapGeomFileSer<'l, VER>
{
	pub fn new(map: &'l MapGeomFile) -> Self
	{
		return Self { version: VER, map };
	}
}

#[derive(Deserialize, Debug)]
pub(super) struct MapGeomFileDe<const VER: u64>
{
	#[serde(deserialize_with = "MapGeomFileDe::<VER>::deserialize_version")]
	version: u64,

	pub map: MapGeomFile,
}

impl<const VER: u64> MapGeomFileDe<VER>
{
	fn deserialize_version<'de, D>(deserializer: D) -> Result<u64, D::Error>
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
					return Err(DeError::custom(format!(
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
