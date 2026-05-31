use std::error::Error;
use std::fmt;

use crate::commands::ResultCode;
use {anyhow, strum};

#[derive(Debug, Copy, Clone, PartialEq, strum::Display)]
pub enum CompilerErrorCode
{
	InternalError,
	ArgumentError,
	ConfigError,
	IoError,
}

#[derive(Debug)]
pub struct CompilerError
{
	pub code: CompilerErrorCode,
	wrapped_err: Box<dyn Error + Send + Sync + 'static>,
}

impl CompilerError
{
	pub fn new<E>(code: CompilerErrorCode, orig_err: E) -> Self
	where
		E: Error + Send + Sync + 'static,
	{
		return Self {
			code: code,
			wrapped_err: Box::new(orig_err),
		};
	}

	pub fn from_anyhow(code: CompilerErrorCode, orig_err: anyhow::Error) -> Self
	{
		return Self {
			code: code,
			wrapped_err: orig_err.into_boxed_dyn_error(),
		};
	}

	pub fn new_anyhow(code: CompilerErrorCode, orig_err: anyhow::Error) -> anyhow::Error
	{
		return CompilerError::from_anyhow(code, orig_err).into_anyhow();
	}

	pub fn into_anyhow(self) -> anyhow::Error
	{
		return anyhow::Error::new(self);
	}

	pub fn first_error_in_chain(err: &anyhow::Error) -> Option<&CompilerError>
	{
		for item in err.chain()
		{
			if let Some(err) = item.downcast_ref::<CompilerError>()
			{
				return Some(err);
			}
		}

		return None;
	}
}

impl fmt::Display for CompilerError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		// Keep an eye on this implementation. This line was enough for everything to
		// work properly on Windows with the ":#" alternate format selector, but printed
		// the code and nothing else on Linux. The latter code was added following the
		// anyhow approach: https://github.com/dtolnay/anyhow/blob/master/src/fmt.rs#L10
		// Things still might not be perfect, though.
		write!(f, "{}", self.code)?;

		if f.alternate()
		{
			write!(f, ": {:?}", self.wrapped_err)?;
		}

		return Ok(());
	}
}

impl Error for CompilerError
{
	fn source(&self) -> Option<&(dyn Error + 'static)>
	{
		return Some(self.wrapped_err.as_ref());
	}
}

impl From<CompilerErrorCode> for ResultCode
{
	fn from(value: CompilerErrorCode) -> Self
	{
		return match value
		{
			CompilerErrorCode::InternalError => ResultCode::InternalError,
			CompilerErrorCode::ArgumentError => ResultCode::ArgumentError,
			CompilerErrorCode::ConfigError => ResultCode::ConfigError,
			CompilerErrorCode::IoError => ResultCode::IoError,
		};
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[derive(Debug, strum::Display)]
	enum DummyError
	{
		ErrorVal,
	}

	impl Error for DummyError
	{
	}

	#[test]
	fn attach_compiler_error_code_to_anyhow_error()
	{
		let orig_error: DummyError = DummyError::ErrorVal;
		let wrapped_error: anyhow::Error =
			CompilerError::new(CompilerErrorCode::InternalError, orig_error).into_anyhow();
		let identified_error = CompilerError::first_error_in_chain(&wrapped_error);

		assert!(identified_error.is_some());
		assert_eq!(
			identified_error.unwrap().code,
			CompilerErrorCode::InternalError
		)
	}
}
