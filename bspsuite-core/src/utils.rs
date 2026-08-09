use anyhow::{Result, anyhow};
use std::path::Path;

pub(crate) fn path_extension(path: &Path) -> Result<Option<&str>>
{
	return match path.extension()
	{
		Some(os_str) => os_str.to_str().map(|str| Some(str)).ok_or_else(|| {
			anyhow!(
				"Could not convert extension for path {} to a string",
				path.display()
			)
		}),
		None => Ok(None),
	};
}
