mod version_1;

use crate::model::MapGeomFile;

use super::helpers::VersionedIOFormat;
use version_1::VERSION;

pub struct MapGeomFileIOFormatV1;

impl VersionedIOFormat<VERSION> for MapGeomFileIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = MapGeomFile;

	fn type_desc() -> &'static str
	{
		return "map geometry file";
	}
}
