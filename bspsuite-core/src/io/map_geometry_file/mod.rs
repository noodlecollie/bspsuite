use super::helpers::VersionedIOFormat;
use crate::model::MapGeomFile;

pub struct MapGeomFileIOFormatV1;

impl VersionedIOFormat for MapGeomFileIOFormatV1
{
	type InnerFormat = MapGeomFile;
	type SerializableFormat = version_1::V1File;
}

mod version_1;
