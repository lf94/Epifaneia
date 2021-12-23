use winit::{
  event::*,
  event_loop::{EventLoop},
};

use async_std::prelude::*;

#[async_std::main]
async fn main() ->() {
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

  let surface_config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: surface.get_preferred_format(&adapter).unwrap(),
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Fifo,
  };
  surface.configure(&device, &surface_config);

  let frame = surface.get_current_texture().unwrap();
  let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
  
  let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: None,
  });

  {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[
        wgpu::RenderPassColorAttachment {
          view: &frame_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1, g: 0.2, b: 0.3, a: 1.0
            }),
            store: true
          }
        }
      ],
      depth_stencil_attachment: None,
    });
  }

  queue.submit(std::iter::once(encoder.finish()));
  frame.present();

  event_loop.run(move |event, _, _| {
    match event {
      Event::MainEventsCleared => window.request_redraw(),
      _ => {},
    }
  });
}
