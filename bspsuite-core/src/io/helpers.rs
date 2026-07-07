use serde::de::{Deserializer, Error, Visitor};

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
