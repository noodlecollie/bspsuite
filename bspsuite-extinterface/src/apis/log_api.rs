use super::api_info::ApiInfo;
use bspffi::types::{XCOption, XCStr};
use log;

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

impl From<log::Level> for LogLevel
{
	fn from(value: log::Level) -> Self
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
}

impl From<LogLevel> for log::Level
{
	fn from(value: LogLevel) -> Self
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
}

impl LogLevel
{
	pub fn from_filter(filter: log::LevelFilter) -> Option<LogLevel>
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

	pub fn to_filter(level: Option<LogLevel>) -> log::LevelFilter
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

	pub fn into_filter(self) -> log::LevelFilter
	{
		return LogLevel::to_filter(Some(self));
	}
}

impl ExtensionLogger
{
	pub fn assign_static_logger(log_api: LogApi) -> Result<(), log::SetLoggerError>
	{
		let filter: log::LevelFilter =
			LogLevel::to_filter((log_api.get_log_level_filter_fn)().into_option());

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
			LogLevel::to_filter((self.log_api.get_log_level_filter_fn)().into_option());

		return metadata.level() <= filter;
	}

	fn log(&self, record: &log::Record)
	{
		if self.enabled(record.metadata())
		{
			let message: String = format!("{}", record.args());

			let args: LogMessageArgs = LogMessageArgs {
				level: record.metadata().level().into(),
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
