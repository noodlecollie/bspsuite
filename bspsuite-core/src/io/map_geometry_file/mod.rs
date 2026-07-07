use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use log::info;
use serde_json;

use crate::model::MapGeomFile;

mod version_1;

pub fn serialize<Writer>(writer: Writer, map: &MapGeomFile) -> Result<()>
where
	Writer: Write,
{
	use version_1::V1File;

	return serde_json::to_writer(writer, &V1File::from(map)).with_context(|| {
		format!(
			"Failed to serialise version {} map geometry file",
			version_1::VERSION
		)
	});
}

pub fn deserialize<Reader>(reader: Reader) -> Result<MapGeomFile>
where
	Reader: Read,
{
	// If we support more than one version, we'll need to change this.
	// We could try versions from the latest one and working backwards.
	use version_1::V1File;

	let wrapper: V1File = serde_json::from_reader(reader).with_context(|| {
		format!(
			"Failed to deserialise version {} map geometry file",
			version_1::VERSION
		)
	})?;

	return Ok(wrapper.into());
}

pub fn write(path: &Path, map: &MapGeomFile) -> Result<()>
{
	info!("Dumping parsed map source to {}", path.display());

	let out_file: File = File::create(path)
		.with_context(|| format!("Failed to open file {} for writing", path.display()))?;

	return serialize(out_file, map);
}

pub fn read(path: &Path) -> Result<MapGeomFile>
{
	info!("Reading map geometry file from {}", path.display());

	let in_file: File = File::open(path)
		.with_context(|| format!("Failed to open file {} for reading", path.display()))?;

	return deserialize(in_file);
}
