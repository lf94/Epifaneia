use serde_json::{Value};
use std::fs::OpenOptions;
use std::time::Instant;
use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
};
use wgpu::util::DeviceExt;
use bytemuck::{Pod,Zeroable};

mod pipelines;
use crate::pipelines::{
  sdf::PipelineSDF,
  window::PipelineWindow,
};

#[derive(Clone,Copy,Pod,Zeroable)]
#[repr(C)]
struct Vertex {
  point: [f32; 3],
  tex_coord: [f32; 2],
}

#[async_std::main]
async fn main() -> () {
  loop {
    let args: Vec<String> = std::env::args().collect();
    let file_json = OpenOptions::new().truncate(false).read(true).write(true)
      .create(true).open(&args[1]).unwrap();
    let json_shader: Value = serde_json::from_reader(file_json).unwrap();
    let text = match json_shader.get("text").unwrap() {
      Value::String(s) => s,
      _ => "",
    };
    let data = json_shader.get("data").unwrap();
    create_and_run_webgpu_context(text, data).await;
  }
}

async fn create_and_run_webgpu_context(shader_text: &str, shader_data: &Value) -> () {
  let event_loop = EventLoop::new();
  let mut builder = winit::window::WindowBuilder::new();
  builder = builder.with_title("Epifaneia");
  let window = builder.build(&event_loop).unwrap();
  let size = window.inner_size();
  let instance = wgpu::Instance::new(wgpu::Backends::all());
  let surface = unsafe { instance.create_surface(&window) };
  let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    compatible_surface: Some(&surface),
    force_fallback_adapter: false,
  })
  .await
  .expect("No suitable GPU adapters found on the system!");

  let (device, queue) = adapter
  .request_device(&wgpu::DeviceDescriptor {
      label: None,
      features: wgpu::Features::empty(),
      limits: wgpu::Limits::default(),
    },
    None
  )
  .await
  .expect("Unable to find a suitable GPU adapter!");

  let mut surface_config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: surface.get_preferred_format(&adapter).unwrap(),
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Fifo,
  };

  let mut pipeline_window = PipelineWindow::new(&device, &surface, &adapter);
  let mut pipeline_sdf = PipelineSDF::new(&device, shader_text);
  let buffer_points = PipelineSDF::json_points_to_gpu_buffer(&device, shader_data);

  let start = Instant::now();

  let buffer_vertices_sdf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&[
      [-1.0f32, 1.0f32, 0.0f32],
      [-1.0f32, -1.0f32, 0.0f32],
      [1.0f32, 1.0f32, 0.0f32],
      [1.0f32, -1.0f32, 0.0f32],
    ]),
    usage: wgpu::BufferUsages::VERTEX,
  });

  let buffer_vertices_window = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&[
      Vertex { point: [-1.0f32, 1.0f32, 0.0f32], tex_coord: [0.0, 0.0] },
      Vertex { point: [-1.0f32, -1.0f32, 0.0f32], tex_coord: [0.0, 1.0] },
      Vertex { point: [1.0f32, 1.0f32, 0.0f32], tex_coord: [1.0, 0.0] },
      Vertex { point: [1.0f32, -1.0f32, 0.0f32], tex_coord: [1.0, 1.0] },
    ]),
    usage: wgpu::BufferUsages::VERTEX,
  });

  let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
  });

  surface.configure(&device, &surface_config);

  // Initial resolution is 32x32
  let min_resolution = 32;
  let max_resolution = 1024;
  let mut resolution = min_resolution;
  let mut mouse_point = PhysicalPosition<f64>::new(0.0, 0.0);
  let mut mouse_point_when_pressed = mouse_point;
  let mut texture_sdf = pipeline_sdf.render_pass(
    &device, &queue,
    start, resolution, mouse_delta,
    &buffer_vertices_sdf, &buffer_points);

  event_loop.run(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { ref event, .. } => {
        match event {
          WindowEvent::Resized(size) => {
            surface_config.width = size.width;
            surface_config.height = size.height;
            surface.configure(&device, &surface_config);
          },
          WindowEvent::CursorMoved { position, .. } => {
            mouse_point = position;
          },
          WindowEvent::MouseInput { state, button, .. } => {
            match button {
              MouseButton::Left => match state {
                ElementState::Pressed => {
                  mouse_point_when_pressed = mouse_point;
                  mouse_pressing = true;
                },
                ElementState::Released => { mouse_pressing = false;  },
              },
              _ => {},
            }
          },
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          _ => {},
        }
      },
      Event::RedrawRequested(_) => {
        let frame_maybe = surface.get_current_texture();
        if frame_maybe.is_err() {
          return;
        }

        let frame = frame_maybe.unwrap();
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        resolution = std::cmp::min(resolution, max_resolution);

        // Don't re-render if reached max resolution or no movement
        if resolution != max_resolution {
          texture_sdf = pipeline_sdf.render_pass(
            &device, &queue,
            start, resolution, mouse_delta,
            &buffer_vertices_sdf, &buffer_points);
        }

        resolution = resolution*2;

        pipeline_window.render_pass(&device, &queue, &frame_view, &buffer_vertices_window, &texture_sdf, &texture_sampler);

        frame.present();
      },
      Event::MainEventsCleared => {
        window.request_redraw();
      },
      _ => {},
    }
  });
}
