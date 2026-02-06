mod app;
mod state;

use app::App;
use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use paris::formatter::colorize_string;
use winit::event_loop::EventLoop;

pub fn main()
{
	run().unwrap_or_default();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue>
{
	console_error_panic_hook::set_once();
	run().unwrap_throw();
	return Ok(());
}

fn run() -> anyhow::Result<()>
{
	#[cfg(not(target_arch = "wasm32"))]
	{
		init_logger();
	}

	#[cfg(target_arch = "wasm32")]
	{
		let log_level: log::Level = if cfg!(debug_assertions)
		{
			log::Level::Debug
		}
		else
		{
			log::Level::Info
		};

		console_log::init_with_level(log::Level::Info).unwrap_throw();
	}

	let event_loop = EventLoop::with_user_event().build()?;
	let mut app = App::new(
		#[cfg(target_arch = "wasm32")]
		&event_loop,
	);

	event_loop.run_app(&mut app)?;
	return Ok(());
}

fn init_logger()
{
	lazy_static! {
		pub static ref RESET: String = colorize_string("</>");
		pub static ref TRACE_PREFIX: String = colorize_string("<d>");
		pub static ref WARNING_PREFIX: String = colorize_string("<b><yellow>");
		pub static ref ERROR_PREFIX: String = colorize_string("<b><red>");
	}

	let log_filter: LevelFilter = if cfg!(debug_assertions)
	{
		LevelFilter::Debug
	}
	else
	{
		LevelFilter::Info
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
