use std::ops::Add;
use std::path::PathBuf;

use crate::vertex_generation::{MeshComponents, MeshVertexGenerator};
use anyhow::{Context, Result};
use bspcore::io::{MapGeomFileIO, VersionedIOFormat};
use bspcore::model::{MapGeomBrush, MapGeomFile};
use clap::Parser;
use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use paris::formatter::colorize_string;
use rand::random_range;

use raylib::camera::Camera3D;
use raylib::math::Vector3;
use raylib::models::WeakMaterial;
use raylib::{RaylibHandle, RaylibThread, prelude as rl};
use rl::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};

mod conv;
mod vertex_generation;

#[derive(clap::Parser)]
#[command(version, about, long_about = None, display_name = env!("CARGO_BIN_NAME"))]
pub struct Cli
{
	/// File to visualise.
	#[arg(short, long)]
	pub file: PathBuf,
}

fn main()
{
	let args: Cli = Cli::parse();

	init_logger();
	init_raylib_logs();

	if !args.file.exists()
	{
		panic!("File {} was not found", args.file.display());
	}

	let map_geometry: MapGeomFile = MapGeomFileIO::read(args.file.as_path())
		.expect(format!("Failed to load {}", args.file.display()).as_str());

	let (mut handle, thread) = raylib::init()
		.size(800, 600)
		.resizable()
		.title("Hello, World")
		.build();

	let mut meshes: Vec<rl::Mesh> = Vec::new();

	for entity in map_geometry.entities.iter()
	{
		let ent_meshes: Vec<rl::Mesh> = entity
			.brushes
			.iter()
			.map(|brush| brush_to_mesh(&thread, brush))
			.collect::<Result<Vec<rl::Mesh>>>()
			.with_context(|| "Failed to generate meshes for brushes")
			.unwrap();

		meshes.extend(ent_meshes);
	}

	let mat: WeakMaterial = handle.load_material_default(&thread);

	let mut camera: Camera3D = Camera3D::perspective(
		Vector3::new(0.0, 0.0, 0.0),
		Vector3::new(0.0, 1.0, 0.0),
		Vector3::new(0.0, 0.0, 1.0),
		60.0,
	);

	handle.set_target_fps(60);
	handle.disable_cursor();

	while !handle.window_should_close()
	{
		let camera_rot: rl::Vector3 = compute_camera_rot_delta(&handle, 0.25);
		let camera_delta: rl::Vector3 = compute_camera_movement_delta(&handle, &camera, 50.0);

		camera.update_camera_pro(camera_delta, camera_rot, 0.0);
		handle.set_mouse_position(rl::Vector2 { x: 400.0, y: 300.0 });

		let mut d = handle.begin_drawing(&thread);
		d.clear_background(rl::Color::DARKGREEN);

		d.draw_mode3D(camera, |mut d2| {
			for mesh in meshes.iter()
			{
				d2.draw_mesh(mesh, mat.clone(), rl::Matrix::identity());
			}
		});

		d.draw_rectangle(10, 10, 220, 70, rl::Color::SKYBLUE);
		d.draw_rectangle_lines(10, 10, 220, 70, rl::Color::BLUE);
		d.draw_text(
			"First person camera default controls:",
			20,
			20,
			10,
			rl::Color::BLACK,
		);
		d.draw_text(
			"- Move with keys: W, A, S, D",
			40,
			40,
			10,
			rl::Color::DARKGRAY,
		);
		d.draw_text(
			"- Mouse move to look around",
			40,
			60,
			10,
			rl::Color::DARKGRAY,
		);
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

fn brush_to_mesh(thread: &RaylibThread, brush: &MapGeomBrush) -> Result<rl::Mesh>
{
	let mut vertex_generator: MeshVertexGenerator = MeshVertexGenerator::from_brush(brush)?;
	vertex_generator.set_colour(rl::Color::color_from_hsv(
		random_range(0.0..360.0),
		0.5,
		1.0,
	));

	let components: MeshComponents = vertex_generator.into_components();

	return rl::Mesh::gen_mesh(&components.positions, &components.tex_coords)
		.normals(&components.normals)
		.indices(&components.indices)
		.colors(&components.colours)
		.build(&thread)
		.with_context(|| {
			format!(
				"Failed to generate mesh for brush {}",
				brush.global_brush_index
			)
		});
}

fn compute_camera_movement_delta(
	handle: &RaylibHandle,
	camera: &rl::Camera3D,
	scale: f32,
) -> rl::Vector3
{
	let mut delta: rl::Vector2 = rl::Vector2::zero();

	if handle.is_key_down(rl::KeyboardKey::KEY_W)
	{
		delta.x += 1.0;
	}

	if handle.is_key_down(rl::KeyboardKey::KEY_S)
	{
		delta.x -= 1.0;
	}

	if handle.is_key_down(rl::KeyboardKey::KEY_D)
	{
		delta.y += 1.0;
	}

	if handle.is_key_down(rl::KeyboardKey::KEY_A)
	{
		delta.y -= 1.0;
	}

	let right: rl::Vector3 = camera.forward().cross(camera.up());

	let direction: rl::Vector3 = camera
		.forward()
		.scale(delta.x)
		.add(right.scale(delta.y))
		.normalize();

	return rl::Vector3 {
		x: camera.forward().dot(direction),
		y: right.dot(direction),
		z: camera.up().dot(direction),
	}
	.scale(scale * handle.get_frame_time());
}

fn compute_camera_rot_delta(handle: &RaylibHandle, scale: f32) -> rl::Vector3
{
	let mouse_delta: rl::Vector2 = handle.get_mouse_delta().scale(scale);

	return rl::Vector3 {
		x: mouse_delta.x,
		y: mouse_delta.y,
		z: 0.0,
	};
}
