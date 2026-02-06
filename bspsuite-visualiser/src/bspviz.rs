use std::sync::Arc;

use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use paris::formatter::colorize_string;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct State
{
	window: Arc<Window>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	size: winit::dpi::PhysicalSize<u32>,
	surface: wgpu::Surface<'static>,
	surface_format: wgpu::TextureFormat,
}

impl State
{
	async fn new(window: Arc<Window>) -> State
	{
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await
			.unwrap();
		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor::default())
			.await
			.unwrap();

		let size = window.inner_size();

		let surface = instance.create_surface(window.clone()).unwrap();
		let cap = surface.get_capabilities(&adapter);
		let surface_format = cap.formats[0];

		let state = State {
			window,
			device,
			queue,
			size,
			surface,
			surface_format,
		};

		// Configure surface for the first time
		state.configure_surface();

		state
	}

	fn get_window(&self) -> &Window
	{
		&self.window
	}

	fn configure_surface(&self)
	{
		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: self.surface_format,
			// Request compatibility with the sRGB-format texture view we‘re going to create later.
			view_formats: vec![self.surface_format.add_srgb_suffix()],
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			width: self.size.width,
			height: self.size.height,
			desired_maximum_frame_latency: 2,
			present_mode: wgpu::PresentMode::AutoVsync,
		};
		self.surface.configure(&self.device, &surface_config);
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
	{
		self.size = new_size;

		// reconfigure the surface
		self.configure_surface();
	}

	fn render(&mut self)
	{
		// Create texture view
		let surface_texture = self
			.surface
			.get_current_texture()
			.expect("failed to acquire next swapchain texture");
		let texture_view = surface_texture
			.texture
			.create_view(&wgpu::TextureViewDescriptor {
				// Without add_srgb_suffix() the image we will be working with
				// might not be "gamma correct".
				format: Some(self.surface_format.add_srgb_suffix()),
				..Default::default()
			});

		// Renders a GREEN screen
		let mut encoder = self.device.create_command_encoder(&Default::default());
		// Create the renderpass which will clear the screen.
		let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &texture_view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		// If you wanted to call any drawing commands, they would go here.

		// End the renderpass.
		drop(renderpass);

		// Submit the command in the queue to execute
		self.queue.submit([encoder.finish()]);
		self.window.pre_present_notify();
		surface_texture.present();
	}
}

#[derive(Default)]
struct App
{
	state: Option<State>,
}

impl ApplicationHandler for App
{
	fn resumed(&mut self, event_loop: &ActiveEventLoop)
	{
		// Create window object
		let window = Arc::new(
			event_loop
				.create_window(Window::default_attributes())
				.unwrap(),
		);

		let state = pollster::block_on(State::new(window.clone()));
		self.state = Some(state);

		window.request_redraw();
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent)
	{
		let state = self.state.as_mut().unwrap();
		match event
		{
			WindowEvent::CloseRequested =>
			{
				println!("The close button was pressed; stopping");
				event_loop.exit();
			}
			WindowEvent::RedrawRequested =>
			{
				state.render();
				// Emits a new redraw requested event.
				state.get_window().request_redraw();
			}
			WindowEvent::Resized(size) =>
			{
				// Reconfigures the size of the surface. We do not re-render
				// here as this event is always followed up by redraw request.
				state.resize(size);
			}
			_ => (),
		}
	}
}

fn main()
{
	init_logger();

	let event_loop = EventLoop::new().unwrap();

	// When the current loop iteration finishes, suspend the thread until
	// another event arrives. Helps keeping CPU utilization low if nothing
	// is happening, which is preferred if the application might be idling in
	// the background.
	event_loop.set_control_flow(ControlFlow::Wait);

	let mut app = App::default();
	event_loop.run_app(&mut app).unwrap();
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
			// There's quite a lot of wgpu spam that comes through as info logs, so treat
			// these as debug.
			if md.level() == Level::Info
				&& md.target().starts_with("wgpu_hal")
				&& log::max_level() < Level::Debug
			{
				return false;
			}

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
