mod version_1;

use crate::model::MapSourceFile;

use super::helpers::VersionedIOFormat;
use version_1::VERSION;

pub struct MapSourceFileIOFormatV1;

impl VersionedIOFormat<VERSION> for MapSourceFileIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = MapSourceFile;

	fn type_desc() -> &'static str
	{
		return "map source file";
	}
}
