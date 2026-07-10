use super::helpers::VersionedIOFormat;
use crate::model::MapSourceFile;

pub struct MapSourceFileIOFormatV1;

impl VersionedIOFormat for MapSourceFileIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = MapSourceFile;
}

mod version_1;
