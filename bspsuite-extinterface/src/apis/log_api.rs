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

pub struct ExtensionLogger
{
	log_api: LogApi,
}

pub mod log_internal
{
	use super::{ExtensionLogger, LogApi, LogLevel, LogMessageArgs};
	use bspffi::types::XCStr;
	use log;

	pub fn ext2int_log_level(value: log::Level) -> LogLevel
	{
		return match value
		{
			log::Level::Error => LogLevel::Error,
			log::Level::Warn => LogLevel::Warn,
			log::Level::Info => LogLevel::Info,
			log::Level::Debug => LogLevel::Debug,
			log::Level::Trace => LogLevel::Trace,
		};
	}

	pub fn int2ext_log_level(value: LogLevel) -> log::Level
	{
		return match value
		{
			LogLevel::Error => log::Level::Error,
			LogLevel::Warn => log::Level::Warn,
			LogLevel::Info => log::Level::Info,
			LogLevel::Debug => log::Level::Debug,
			LogLevel::Trace => log::Level::Trace,
		};
	}

	pub fn ext2int_log_filter(filter: log::LevelFilter) -> Option<LogLevel>
	{
		return match filter
		{
			log::LevelFilter::Off => None,
			log::LevelFilter::Error => Some(LogLevel::Error),
			log::LevelFilter::Warn => Some(LogLevel::Warn),
			log::LevelFilter::Info => Some(LogLevel::Info),
			log::LevelFilter::Debug => Some(LogLevel::Debug),
			log::LevelFilter::Trace => Some(LogLevel::Trace),
		};
	}

	pub fn int2ext_log_filter(level: Option<LogLevel>) -> log::LevelFilter
	{
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

	impl ExtensionLogger
	{
		pub fn assign_static_logger(log_api: LogApi) -> Result<(), log::SetLoggerError>
		{
			let filter: log::LevelFilter =
				int2ext_log_filter((log_api.get_log_level_filter_fn)().into_option());

			log::set_boxed_logger(Box::new(Self { log_api: log_api }))?;
			log::set_max_level(filter);

			return Ok(());
		}
	}

	impl log::Log for ExtensionLogger
	{
		fn enabled(&self, metadata: &log::Metadata) -> bool
		{
			let filter: log::LevelFilter =
				int2ext_log_filter((self.log_api.get_log_level_filter_fn)().into_option());

			return metadata.level() <= filter;
		}

		fn log(&self, record: &log::Record)
		{
			if self.enabled(record.metadata())
			{
				let message: String = format!("{}", record.args());

				let args: LogMessageArgs = LogMessageArgs {
					level: ext2int_log_level(record.metadata().level()),
					target: record.metadata().target().into(),
					module: record.module_path().unwrap_or("<unknown>").into(),
					file: record.file().unwrap_or("<unknown>").into(),
					line: record.line().unwrap_or(0),
					msg: XCStr::from(message.as_ref()),
				};

				(self.log_api.log_fn)(&args);
			}
		}

		fn flush(&self)
		{
		}
	}
}
