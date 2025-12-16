use super::api_info::ApiInfo;
use crate::types::StringRef;
use log;

// TODO: Can we build this without a dependency on the log crate?
// We don't want to depend on external crates here, as it would mean
// that bumping a dependency might necessitate bumping the extension
// API version.
pub const API_INFO: ApiInfo = ApiInfo::new("LogApi", 1);

#[repr(C)]
#[derive(Clone)]
pub struct Api
{
	pub get_log_level_filter_fn: extern "C" fn() -> log::LevelFilter,
	pub log_fn: extern "C" fn(&LogMessageArgs),
}

#[repr(C)]
pub struct LogMessageArgs<'l>
{
	pub level: log::Level,
	pub target: &'l str,
	pub module: &'l str,
	pub file: &'l str,
	pub line: u32,
	pub msg: StringRef<'l>,
}

pub struct ExtensionLogger
{
	log_api: Api,
}

impl ExtensionLogger
{
	pub fn assign_static_logger(log_api: Api) -> Result<(), log::SetLoggerError>
	{
		let filter: log::LevelFilter = (log_api.get_log_level_filter_fn)();

		log::set_boxed_logger(Box::new(Self { log_api: log_api }))?;
		log::set_max_level(filter);

		return Ok(());
	}
}

impl log::Log for ExtensionLogger
{
	fn enabled(&self, metadata: &log::Metadata) -> bool
	{
		return metadata.level() <= (self.log_api.get_log_level_filter_fn)();
	}

	fn log(&self, record: &log::Record)
	{
		if self.enabled(record.metadata())
		{
			let message: String = format!("{}", record.args());

			let args: LogMessageArgs = LogMessageArgs {
				level: record.metadata().level(),
				target: record.metadata().target(),
				module: record.module_path().unwrap_or("<unknown>"),
				file: record.file().unwrap_or("<unknown>"),
				line: record.line().unwrap_or(0),
				msg: StringRef::from(message.as_ref()),
			};

			(self.log_api.log_fn)(&args);
		}
	}

	fn flush(&self)
	{
	}
}
