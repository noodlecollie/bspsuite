use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use paris::formatter::colorize_string;

use raylib::prelude as rl;
use rl::RaylibDraw;

fn main()
{
	init_logger();
	init_raylib_logs();

	let (mut handle, thread) = raylib::init().size(640, 480).title("Hello, World").build();

	while !handle.window_should_close()
	{
		let mut d: rl::RaylibDrawHandle = handle.begin_drawing(&thread);

		d.clear_background(rl::Color::WHITE);
		d.draw_text("Hello, world!", 12, 12, 20, rl::Color::BLACK);
	}
}

fn init_logger()
{
	lazy_static! {
		pub static ref RESET: String = colorize_string("</>");
		pub static ref TRACE_PREFIX: String = colorize_string("<d>");
		pub static ref WARNING_PREFIX: String = colorize_string("<b><yellow>");
		pub static ref ERROR_PREFIX: String = colorize_string("<b><red>");
	}

	let base_config = fern::Dispatch::new().level(LevelFilter::Info);

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
			return md.level() == Level::Info
				|| md.level() == Level::Debug
				|| md.level() == Level::Trace;
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

fn init_raylib_logs()
{
	use raylib::consts::TraceLogLevel;

	rl::set_trace_log_callback(|log_level, msg|  {
		match log_level {
			TraceLogLevel::LOG_FATAL => panic!("Fatal RayLib error: {msg}"),
			TraceLogLevel::LOG_ERROR => log::error!(target: "raylib", "[RL] {msg}"),
			TraceLogLevel::LOG_WARNING => log::warn!(target: "raylib", "[RL] {msg}"),
			// Treat Raylib info logs as debug, since most of the time they're not relevant to us.
			TraceLogLevel::LOG_INFO | TraceLogLevel::LOG_DEBUG => log::debug!(target: "raylib", "[RL] {msg}"),
			TraceLogLevel::LOG_TRACE => log::trace!(target: "raylib", "[RL] {msg}"),
			_ => panic!("Unexpected log level {:?}", log_level),
		}
	}).expect("Could not set Raylib log callback");
}
