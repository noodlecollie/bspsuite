use super::helpers::VersionedIOFormat;
use crate::model::MapGeomFile;

pub struct MapGeomFileIOFormatV1;

impl VersionedIOFormat for MapGeomFileIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = MapGeomFile;
}

mod version_1;
