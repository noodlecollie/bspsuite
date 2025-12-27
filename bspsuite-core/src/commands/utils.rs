use super::types::ResultCode;
use crate::compiler_error::CompilerError;
use anyhow;
use log::{error, warn};
use paris::formatter::colorize_string;
use std::any::Any;
use std::panic::{UnwindSafe, catch_unwind};

pub fn wrap_residual_errors<F>(func: F) -> ResultCode
where
	F: FnOnce() -> anyhow::Result<()> + UnwindSafe,
{
	return wrap_panics(|| match func()
	{
		Err(err) =>
		{
			error!("{:#}", err);

			let compiler_error: Option<&CompilerError> = CompilerError::first_error_in_chain(&err);

			// TODO: We should just make the function return a CompilerError and not have to
			// worry about this.
			debug_assert!(
				compiler_error.is_some(),
				"Encountered a compile error which was not of type CompilerError"
			);

			if compiler_error.is_none()
			{
				warn!(
					"The above error was not of the expected type CompilerError. This is a programmer oversight!"
				);
			}

			compiler_error
				.map(|err| err.code.into())
				.unwrap_or(ResultCode::InternalError)
		}
		Ok(_) => ResultCode::Ok,
	});
}

// Ensures that if a panic occurs, we log a fatal error and exit.
pub fn wrap_panics<F>(func: F) -> ResultCode
where
	F: FnOnce() -> ResultCode + UnwindSafe,
{
	type UnwindError = Box<dyn Any + Send + 'static>;
	let result: Result<ResultCode, UnwindError> = catch_unwind(func);

	return match result
	{
		Ok(result_code) => result_code,
		Err(_) =>
		{
			let banner: String = colorize_string(
				"<u><b><red>\
					**************************************\n\
					********** CRITICAL FAILURE **********\n\
					**************************************\n\
					</>",
			);

			// We don't know the type of the error, and the functions we have available to
			// check it are frustratingly limited, so there's not much we can actually log
			// here.
			//
			// TODO: Does the project configuration support setting a repo? Can we use an
			// environment variable to fetch the URL?
			error!(
				"\n\
				{banner}\n\
				The compiler has encountered an unrecoverable error and halted.\n\
				Please create an issue report at https://github.com/noodlecollie/bspsuite/issues/\n\
				and include the full log from this run."
			);

			ResultCode::InternalError
		}
	};
}
