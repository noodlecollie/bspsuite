mod cli;

use std::ffi::{CStr, c_char};

use bspcore::commands as Cmds;
use bspextifc::types::StringRef;
use bspsuite_ffim::types::{XCOption, XCStr};

use clap::Parser;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, error, info};
use paris::formatter::colorize_string;

use crate::cli::DebugLevel;

fn main()
{
	let parsed_args: cli::Cli = cli::Cli::parse();

	init_logger(&parsed_args);
	print_banner();

	let subcommand: &cli::Subcommand = &parsed_args.command;
	let result_code: Cmds::ResultCode = match subcommand
	{
		cli::Subcommand::Extinfo(args) => run_info_command(&args),
		cli::Subcommand::Compile(args) => run_compile_command(&parsed_args, &args),
	};

	match result_code
	{
		Cmds::ResultCode::Ok => (),
		_ =>
		{
			error!("[{subcommand}] failed.");
		}
	}

	std::process::exit(result_code as i32);
}

fn run_info_command(args: &cli::ExtinfoCommandArgs) -> Cmds::ResultCode
{
	let args: Cmds::ExtinfoArgs = Cmds::ExtinfoArgs {
		base: Cmds::BaseArgs::default(),
		extension_name: args
			.extension
			.as_ref()
			.map(|val| StringRef::from(val.as_ref())),
	};

	return Cmds::bspcore_run_extinfo(&args);
}

fn run_compile_command(base_args: &cli::Cli, args: &cli::CompileCommandArgs) -> Cmds::ResultCode
{
	let input_path_str: Option<&str> = args.input_file.to_str();

	if input_path_str.is_none()
	{
		error!("Could not convert input path to string");
		return Cmds::ResultCode::InternalError;
	}

	let toolchain_path_str: Option<Option<&str>> =
		base_args.toolchain_root.as_ref().map(|path| path.to_str());

	if let Some(conv_result) = toolchain_path_str
		&& conv_result.is_none()
	{
		error!("Could not convert toolchain root path to string");
		return Cmds::ResultCode::InternalError;
	}

	let toolchain_path_str: Option<&str> = match toolchain_path_str
	{
		Some(conv_result) => Some(conv_result.unwrap()),
		None => None,
	};

	let args: Cmds::CompileArgs = Cmds::CompileArgs {
		base: Cmds::BaseArgs {
			toolchain_root: XCOption::from(toolchain_path_str.map(|val| XCStr::from(val))),
		},
		input_file: XCStr::from(input_path_str.unwrap()),
		game: XCStr::from(args.game.as_str()),
		map_format_override: XCOption::from(
			args.map_format
				.as_ref()
				.map(|val| XCStr::from(val.as_str())),
		),
	};

	return Cmds::bspcore_run_compile(&args);
}

fn init_logger(parsed_args: &cli::Cli)
{
	lazy_static! {
		pub static ref RESET: String = colorize_string("</>");
		pub static ref TRACE_PREFIX: String = colorize_string("<d>");
		pub static ref WARNING_PREFIX: String = colorize_string("<b><yellow>");
		pub static ref ERROR_PREFIX: String = colorize_string("<b><red>");
	}

	let log_filter: LevelFilter = match parsed_args.debug
	{
		Some(DebugLevel::Off) => LevelFilter::Info,
		Some(DebugLevel::On) => LevelFilter::Debug,
		Some(DebugLevel::Trace) => LevelFilter::Trace,
		None =>
		{
			if cfg!(debug_assertions)
			{
				LevelFilter::Debug
			}
			else
			{
				LevelFilter::Info
			}
		}
	};

	let base_config = fern::Dispatch::new().level(log_filter);

	let stderr_logger = fern::Dispatch::new()
		.filter(|md| md.level() == Level::Error || md.level() == Level::Warn)
		.format(|out, message, record| {
			match record.level()
			{
				Level::Error => out.finish(format_args!(
					"{}Error:{} {message}",
					ERROR_PREFIX.as_str(),
					RESET.as_str()
				)),
				Level::Warn => out.finish(format_args!(
					"{}Warning:{} {message}",
					WARNING_PREFIX.as_str(),
					RESET.as_str()
				)),
				_ => (),
			};
		})
		.chain(std::io::stderr());

	let stdout_logger = fern::Dispatch::new()
		.filter(|md| {
			md.level() == Level::Info || md.level() == Level::Debug || md.level() == Level::Trace
		})
		.format(|out, message, record| {
			match record.level()
			{
				Level::Info => out.finish(format_args!("{}", message)),
				Level::Debug => out.finish(format_args!("({}) {}", record.target(), message)),
				Level::Trace =>
				{
					let file: &str = record.file().unwrap_or("<unknown>");
					let line: u32 = record.line().unwrap_or(0);

					out.finish(format_args!(
						"{}({file}:{line}) {message}{}",
						TRACE_PREFIX.as_str(),
						RESET.as_str(),
					))
				}
				_ => (),
			};
		})
		.chain(std::io::stdout());

	base_config
		.chain(stderr_logger)
		.chain(stdout_logger)
		.apply()
		.expect("Could not initialise logger");
}

fn print_banner()
{
	let build_id_ptr: *const c_char = Cmds::bspcore_get_build_identifier_string();
	let build_id: &'static CStr = unsafe { CStr::from_ptr(build_id_ptr) };
	let bin_name: String = colorize_string(format!("<b>{}</b>", env!("CARGO_BIN_NAME")));

	info!(
		"\
		================================================================================\n\
		{bin_name} version {} ({})\n\
		================================================================================",
		env!("CARGO_PKG_VERSION"),
		build_id.to_str().unwrap()
	);
}
