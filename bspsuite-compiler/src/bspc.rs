mod cli;

use bspcore::{BUILD_IDENTIFIER, commands as cmds};

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
	let result_code: cmds::ResultCode = match subcommand
	{
		cli::Subcommand::Extinfo(args) => run_extinfo_command(&parsed_args, &args),
		cli::Subcommand::Compile(args) => run_compile_command(&parsed_args, &args),
		cli::Subcommand::Resinfo(args) => run_resinfo_command(&parsed_args, &args),
	};

	match result_code
	{
		cmds::ResultCode::Ok => (),
		_ =>
		{
			error!("{subcommand} command failed.");
		}
	}

	std::process::exit(result_code as i32);
}

fn run_extinfo_command(base_args: &cli::Cli, args: &cli::ExtinfoCommandArgs) -> cmds::ResultCode
{
	let args: cmds::ExtinfoArgs = cmds::ExtinfoArgs {
		base: cmds::BaseArgs {
			toolchain_root: base_args.toolchain_root.clone(),
		},
		extension_name: args.extension.clone(),
	};

	return cmds::bspcore_run_extinfo(&args);
}

fn run_compile_command(base_args: &cli::Cli, args: &cli::CompileCommandArgs) -> cmds::ResultCode
{
	let args: cmds::CompileArgs = cmds::CompileArgs {
		base: cmds::BaseArgs {
			toolchain_root: base_args.toolchain_root.clone(),
		},
		input_file: args.input_file.clone(),
		game: args.game.clone(),
		game_dir: args.game_dir.clone(),
		map_format_override: args.map_format.clone(),
		parameters_file: args.parameters_file.clone(),
		dump_source_file: args.dump_source_file,
	};

	return cmds::bspcore_run_compile(&args);
}

fn run_resinfo_command(base_args: &cli::Cli, args: &cli::ResinfoCommandArgs) -> cmds::ResultCode
{
	let args: cmds::ResinfoArgs = cmds::ResinfoArgs {
		base: cmds::BaseArgs {
			toolchain_root: base_args.toolchain_root.clone(),
		},
		game: args.game.clone(),
		game_dir: args.game_dir.clone(),
		resource_path: args.resource_path.clone(),
	};

	return cmds::bspcore_run_resinfo(&args);
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
	let bin_name: String = colorize_string(format!("<b>{}</b>", env!("CARGO_BIN_NAME")));

	info!(
		"\
		================================================================================\n\
		{bin_name} version {} ({})\n\
		================================================================================",
		env!("CARGO_PKG_VERSION"),
		BUILD_IDENTIFIER
	);
}
