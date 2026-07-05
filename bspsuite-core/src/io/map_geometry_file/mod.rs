use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use log::info;

use crate::model::MapGeomFile;

mod version_1;

pub fn serialize<Writer>(writer: Writer, map: &MapGeomFile) -> Result<()>
where
	Writer: Write,
{
	use version_1 as current_version;

	return current_version::serialize(writer, map).with_context(|| {
		format!(
			"Failed to serialise version {} map geometry file",
			current_version::VERSION
		)
	});
}

pub fn deserialize<Reader>(reader: Reader) -> Result<MapGeomFile>
where
	Reader: Read,
{
	// If we support more than one version, we'll need to change this.
	use version_1 as current_version;

	return current_version::deserialize(reader).with_context(|| {
		format!(
			"Failed to deserialise version {} map geometry file",
			current_version::VERSION
		)
	});
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
