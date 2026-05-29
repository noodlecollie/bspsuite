use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use paris::formatter::colorize_string;

use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::math::{Vector2, Vector3};
use raylib::prelude as rl;
use rl::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};

fn main()
{
	init_logger();
	init_raylib_logs();

	let (mut handle, thread) = raylib::init()
		.size(640, 480)
		.resizable()
		.title("Hello, World")
		.build();

	let mut camera: Camera3D = Camera3D::perspective(
		Vector3::new(4.0, 2.0, 4.0),
		Vector3::new(0.0, 1.8, 0.0),
		Vector3::new(0.0, 1.0, 0.0),
		60.0,
	);

	handle.set_target_fps(60);

	while !handle.window_should_close()
	{
		handle.update_camera(&mut camera, rl::CameraMode::CAMERA_FIRST_PERSON);

		let mut d = handle.begin_drawing(&thread);
		d.clear_background(Color::DARKGREEN);

		d.draw_mode3D(camera, |mut d2, _| {
			d2.draw_plane(
				Vector3::new(0.0, 0.0, 0.0),
				Vector2::new(32.0, 32.0),
				Color::LIGHTGRAY,
			);
			d2.draw_cube(Vector3::new(-16.0, 2.5, 0.0), 1.0, 5.0, 32.0, Color::BLUE);
			d2.draw_cube(Vector3::new(16.0, 2.5, 0.0), 1.0, 5.0, 32.0, Color::LIME);
			d2.draw_cube(Vector3::new(0.0, 2.5, 16.0), 32.0, 5.0, 1.0, Color::GOLD);
		});

		d.draw_rectangle(10, 10, 220, 70, Color::SKYBLUE);
		d.draw_rectangle_lines(10, 10, 220, 70, Color::BLUE);
		d.draw_text(
			"First person camera default controls:",
			20,
			20,
			10,
			Color::BLACK,
		);
		d.draw_text("- Move with keys: W, A, S, D", 40, 40, 10, Color::DARKGRAY);
		d.draw_text("- Mouse move to look around", 40, 60, 10, Color::DARKGRAY);
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

	let base_config = fern::Dispatch::new().level(LevelFilter::Debug);

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

	rl::set_trace_log_callback(|log_level, msg| {
		match log_level
		{
			TraceLogLevel::LOG_FATAL => panic!("Fatal RayLib error: {msg}"),
			TraceLogLevel::LOG_ERROR => log::error!(target: "raylib", "[RL] {msg}"),
			TraceLogLevel::LOG_WARNING => log::warn!(target: "raylib", "[RL] {msg}"),
			// Treat Raylib info logs as debug, since most of the time they're not relevant to us.
			TraceLogLevel::LOG_INFO | TraceLogLevel::LOG_DEBUG =>
			{
				log::debug!(target: "raylib", "[RL] {msg}")
			}
			TraceLogLevel::LOG_TRACE => log::trace!(target: "raylib", "[RL] {msg}"),
			_ => panic!("Unexpected log level {:?}", log_level),
		}
	})
	.expect("Could not set Raylib log callback");
}
