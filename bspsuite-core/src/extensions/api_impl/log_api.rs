use std::fmt::Arguments;

use bspextifc::log_api;
use bspffi::types::XCOption;
use log::{Record, RecordBuilder};

pub fn create_api() -> log_api::LogApi
{
	return log_api::LogApi {
		get_log_level_filter_fn: get_log_level_filter,
		log_fn: log_message,
	};
}

extern "C" fn get_log_level_filter() -> XCOption<log_api::LogLevel>
{
	return ext2int_log_filter(log::max_level()).into();
}

extern "C" fn log_message(args: &log_api::LogMessageArgs)
{
	let msg_string: String = args.msg.to_string();
	let msg_args: Arguments = format_args!("{}", msg_string);

	let mut builder: RecordBuilder = RecordBuilder::new();

	let record: Record = builder
		.file(Some(args.file.as_str()))
		.line(Some(args.line))
		.target(args.target.as_str())
		.module_path(Some(args.module.as_str()))
		.level(int2ext_log_level(args.level))
		.args(msg_args)
		.build();

	log::logger().log(&record);
}

fn int2ext_log_level(value: log_api::LogLevel) -> log::Level
{
	return match value
	{
		log_api::LogLevel::Error => log::Level::Error,
		log_api::LogLevel::Warn => log::Level::Warn,
		log_api::LogLevel::Info => log::Level::Info,
		log_api::LogLevel::Debug => log::Level::Debug,
		log_api::LogLevel::Trace => log::Level::Trace,
	};
}

fn ext2int_log_filter(filter: log::LevelFilter) -> Option<log_api::LogLevel>
{
	return match filter
	{
		log::LevelFilter::Off => None,
		log::LevelFilter::Error => Some(log_api::LogLevel::Error),
		log::LevelFilter::Warn => Some(log_api::LogLevel::Warn),
		log::LevelFilter::Info => Some(log_api::LogLevel::Info),
		log::LevelFilter::Debug => Some(log_api::LogLevel::Debug),
		log::LevelFilter::Trace => Some(log_api::LogLevel::Trace),
	};
}
