use winit::{
  event::*,
  event_loop::{EventLoop},
};

use async_std::prelude::*;

use wgpu::util::DeviceExt;

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

  let mut surface_config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: surface.get_preferred_format(&adapter).unwrap(),
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Fifo,
  };

  let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
  });

  let vertex_buffer_layout = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
  };

  let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&[
      [-1.0f32, 1.0f32, 0.0f32],
      [-1.0f32, -1.0f32, 0.0f32],
      [1.0f32, 1.0f32, 0.0f32],
      [1.0f32, -1.0f32, 0.0f32],
    ]),
    usage: wgpu::BufferUsages::VERTEX,
  });

  let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&[
      [-1.0f32, 1.0f32, 0.0f32],
      [-1.0f32, -1.0f32, 0.0f32],
      [1.0f32, 1.0f32, 0.0f32],
    ]),
    usage: wgpu::BufferUsages::UNIFORM,
  });

  let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: None,
    entries: &[
      wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None, // TODO: Calculate?
        },
        count: None,
      }
    ]
  });

  let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: None,
    layout: &bind_group_layout,
    entries: &[
      wgpu::BindGroupEntry {
        binding: 0,
        resource: point_buffer.as_entire_binding(),
      }
    ]
  });

  let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: None,
    bind_group_layouts: &[&bind_group_layout],
    push_constant_ranges: &[],
  });

  let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: None,
    layout: Some(&render_pipeline_layout),
    vertex: wgpu::VertexState {
      module: &shader,
      entry_point: "vs_main",
      buffers: &[vertex_buffer_layout],
    },
    fragment: Some(wgpu::FragmentState {
      module: &shader,
      entry_point: "fs_main",
      targets: &[wgpu::ColorTargetState {
        format: surface_config.format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
      }]
    }),
    primitive: wgpu::PrimitiveState {
      topology: wgpu::PrimitiveTopology::TriangleStrip,
      strip_index_format: None,
      front_face: wgpu::FrontFace::Ccw,
      cull_mode: Some(wgpu::Face::Back),
      polygon_mode: wgpu::PolygonMode::Fill,
      unclipped_depth: false,
      conservative: false,
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState {
      count: 1,
      mask: !0,
      alpha_to_coverage_enabled: false,
    },
    multiview: None,
  });

  event_loop.run(move |event, _, _| {
    match event {
      Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
        surface_config.width = size.width;
        surface_config.height = size.height;
        surface.configure(&device, &surface_config);
      },
      Event::RedrawRequested(_) => {
        let frame = surface.get_current_texture().unwrap();
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: None,
        });

        {
          let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[
              wgpu::RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                  load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1, g: 0.8, b: 0.3, a: 1.0
                  }),
                  store: true
                }
              }
            ],
            depth_stencil_attachment: None,
          });

          render_pass.set_pipeline(&render_pipeline);
          render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
          render_pass.set_bind_group(0, &bind_group, &[]);
          render_pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
      },
      _ => {},
    }
  });
}
