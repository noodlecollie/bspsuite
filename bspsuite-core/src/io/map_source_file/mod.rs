use super::helpers::VersionedIOFormat;
use crate::model::MapSourceFile;

pub struct MapSourceFileIOFormatV1;

impl VersionedIOFormat for MapSourceFileIOFormatV1
{
	type InnerFormat = MapSourceFile;
	type SerializableFormat = version_1::V1File;
}

mod version_1;
