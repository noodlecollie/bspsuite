use serde_valid::ValidateMinLength;
use serde_valid::validation::Error as ValidationError;
use serde_valid::validation::error::FormatDefault;

pub(in crate::io) fn validate_items_non_empty<T: ValidateMinLength>(
	vec: &Vec<T>,
) -> Result<(), ValidationError>
{
	for (index, item) in vec.iter().enumerate()
	{
		item.validate_min_length(1).map_err(|err| {
			ValidationError::Custom(format!("Index {index}: {}", err.format_default()))
		})?;
	}

	return Ok(());
}
