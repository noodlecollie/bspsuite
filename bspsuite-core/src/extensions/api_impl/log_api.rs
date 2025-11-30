use std::fmt::Arguments;

use bspextifc::log_api;
use log::{Record, RecordBuilder};

pub fn create_api() -> log_api::Api
{
	return log_api::Api {
		get_log_level_filter_fn: get_log_level_filter,
		log_fn: log_message,
	};
}

extern "C" fn get_log_level_filter() -> log::LevelFilter
{
	return log::max_level();
}

extern "C" fn log_message(args: &log_api::LogMessageArgs)
{
	let msg_string: String = args.msg.to_string();
	let msg_args: Arguments = format_args!("{}", msg_string);

	let mut builder: RecordBuilder = RecordBuilder::new();

	let record: Record = builder
		.file(Some(args.file))
		.line(Some(args.line))
		.target(args.target)
		.module_path(Some(args.module))
		.level(args.level)
		.args(msg_args)
		.build();

	log::logger().log(&record);
}
