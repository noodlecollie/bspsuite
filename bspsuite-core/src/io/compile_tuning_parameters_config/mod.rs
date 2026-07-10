mod version_1;

use crate::configs::CompileTuningParametersConfig;

use super::helpers::VersionedIOFormat;
use version_1::VERSION;

pub struct CompileTuningParametersConfigIOFormatV1;

impl VersionedIOFormat<VERSION> for CompileTuningParametersConfigIOFormatV1
{
	type FileFormat = version_1::V1File;
	type InnerFormat = CompileTuningParametersConfig;

	fn format_name() -> &'static str
	{
		return "compiletuningparams";
	}

	fn type_desc() -> &'static str
	{
		return "compile tuning params";
	}
}
