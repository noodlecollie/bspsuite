use super::api_info::ApiInfo;
use bspffi::types::{XCOption, XCStr};

pub const API_INFO: ApiInfo = ApiInfo::new("LogApi", 1);

#[repr(C)]
#[derive(Copy, Clone)]
pub enum LogLevel
{
	Error = 1,
	Warn,
	Info,
	Debug,
	Trace,
}

#[repr(C)]
#[derive(Clone)]
pub struct LogApi
{
	pub get_log_level_filter_fn: extern "C" fn() -> XCOption<LogLevel>,
	pub log_fn: extern "C" fn(&LogMessageArgs),
}

#[repr(C)]
pub struct LogMessageArgs<'l>
{
	pub level: LogLevel,
	pub target: XCStr<'l>,
	pub module: XCStr<'l>,
	pub file: XCStr<'l>,
	pub line: u32,
	pub msg: XCStr<'l>,
}

/// Macro to implement a logger class which can be used with the Rust `log`
/// crate. This is a macro so that the log API doesn't have to depend on a
/// specific version of the log crate - the version is up to the extension using
/// the logger.
///
/// The argument given to the macro is the name of the type alias that refers to
/// the logger. Extensions should call `assign_static_logger()` on this type to
/// assign the logger to the `log` crate.
#[macro_export]
macro_rules! implement_extension_logger {
	($extlogger:ident) => {
		pub mod bspsuite_log_internal
		{
			use bspextifc::log_api::{LogApi, LogLevel, LogMessageArgs};
			use log;

			pub struct ExtensionLogger
			{
				log_api: bspextifc::log_api::LogApi,
			}

			impl ExtensionLogger
			{
				pub fn assign_static_logger(
					log_api: bspextifc::log_api::LogApi,
				) -> Result<(), log::SetLoggerError>
				{
					let filter: log::LevelFilter = ExtensionLogger::int2ext_log_filter(
						(log_api.get_log_level_filter_fn)().into_option(),
					);

					log::set_boxed_logger(Box::new(Self { log_api: log_api }))?;
					log::set_max_level(filter);

					return Ok(());
				}

				fn int2ext_log_filter(
					level: Option<bspextifc::log_api::LogLevel>,
				) -> log::LevelFilter
				{
					type LogLevel = bspextifc::log_api::LogLevel;

					return level
						.map(|val| match val
						{
							LogLevel::Error => log::LevelFilter::Error,
							LogLevel::Warn => log::LevelFilter::Warn,
							LogLevel::Info => log::LevelFilter::Info,
							LogLevel::Debug => log::LevelFilter::Debug,
							LogLevel::Trace => log::LevelFilter::Trace,
						})
						.unwrap_or(log::LevelFilter::Off);
				}

				fn ext2int_log_level(value: log::Level) -> bspextifc::log_api::LogLevel
				{
					type LogLevel = bspextifc::log_api::LogLevel;

					return match value
					{
						log::Level::Error => LogLevel::Error,
						log::Level::Warn => LogLevel::Warn,
						log::Level::Info => LogLevel::Info,
						log::Level::Debug => LogLevel::Debug,
						log::Level::Trace => LogLevel::Trace,
					};
				}
			}

			impl log::Log for ExtensionLogger
			{
				fn enabled(&self, metadata: &log::Metadata) -> bool
				{
					let filter: log::LevelFilter = ExtensionLogger::int2ext_log_filter(
						(self.log_api.get_log_level_filter_fn)().into_option(),
					);

					return metadata.level() <= filter;
				}

				fn log(&self, record: &log::Record)
				{
					if self.enabled(record.metadata())
					{
						let message: String = format!("{}", record.args());

						let args: bspextifc::log_api::LogMessageArgs =
							bspextifc::log_api::LogMessageArgs {
								level: ExtensionLogger::ext2int_log_level(
									record.metadata().level(),
								),
								target: record.metadata().target().into(),
								module: record.module_path().unwrap_or("<unknown>").into(),
								file: record.file().unwrap_or("<unknown>").into(),
								line: record.line().unwrap_or(0),
								msg: message.as_str().into(),
							};

						(self.log_api.log_fn)(&args);
					}
				}

				fn flush(&self)
				{
				}
			}
		}

		type $extlogger = bspsuite_log_internal::ExtensionLogger;
	};
}
