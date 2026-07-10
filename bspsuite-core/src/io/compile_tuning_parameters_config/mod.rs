use super::helpers::VersionedIOFormat;
use crate::configs::CompileTuningParametersConfig;

pub struct CompileTuningParametersConfigIOFormatV1;

impl VersionedIOFormat for CompileTuningParametersConfigIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = CompileTuningParametersConfig;
}

mod version_1;
