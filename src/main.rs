use serde_json::{Value};
use std::fs::OpenOptions;
use std::time::Instant;
use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
};
use wgpu::util::DeviceExt;

#[async_std::main]
async fn main() -> () {
  std::fs::create_dir_all("/tmp/epifaneia").unwrap();

  loop {
    let _file_ctl = OpenOptions::new().truncate(true).read(true).write(true)
      .create(true).open("/tmp/epifaneia/ctl").unwrap();
    let file_json = OpenOptions::new().truncate(false).read(true).write(true)
      .create(true).open("/tmp/epifaneia/json").unwrap();
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

  let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(shader_text.into()),
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

  let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: None,
    entries: &[
      wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      },
      wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      },
      wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
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

  let mut buffer_resolution = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&[surface_config.width as f32, surface_config.height as f32]),
    usage: wgpu::BufferUsages::UNIFORM,
  });

  let start = Instant::now();

  let mut buffer_gpu_contents: Vec<u8> = vec![];

  match &shader_data {
    Value::Array(o) => for l in o.iter() {
      if let Value::Array(v) = l {
        for p in v.iter() {
          if let Value::Array(a) = p {
            for b in a.iter() {
              if let Value::Number(n) = b {
                buffer_gpu_contents.append(
                  &mut Vec::<u8>::from((n.as_f64().unwrap() as f32)
                  .to_bits().to_le_bytes())
                );
              }
            }
          }
        }

        let padding_needed = buffer_gpu_contents.len() % 8;
        // Must ensure each member is aligned.
        if padding_needed > 0 {
          buffer_gpu_contents.append(&mut vec![0u8; padding_needed]);
        }
      }
    },
    _ => { buffer_gpu_contents = vec![0u8;64]; }
  }

  let padding_needed = buffer_gpu_contents.len() % 8;
  // Must ensure the struct is aligned.
  if padding_needed > 0 {
    buffer_gpu_contents.append(&mut vec![0u8; padding_needed]);
  }

  if buffer_gpu_contents.len() == 0 {
    buffer_gpu_contents = vec![0u8;64]; 
  }

  let buffer_gpu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: bytemuck::cast_slice(&buffer_gpu_contents),
    usage: wgpu::BufferUsages::UNIFORM,
  });

  event_loop.run(move |event, _, control_flow| {
    match event {
      Event::WindowEvent { ref event, .. } => {
        match event {
          WindowEvent::Resized(size) => {
            surface_config.width = size.width;
            surface_config.height = size.height;

            buffer_resolution = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
              label: None,
              contents: bytemuck::cast_slice(&[surface_config.width as f32, surface_config.height as f32]),
              usage: wgpu::BufferUsages::UNIFORM,
            });

            surface.configure(&device, &surface_config);
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
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: None,
        });

        let buffer_time = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: None,
          contents: 
              &start
              .elapsed()
              .as_secs_f32().to_bits().to_le_bytes()
            ,
          usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: None,
          layout: &bind_group_layout,
          entries: &[
            wgpu::BindGroupEntry {
              binding: 0,
              resource: buffer_gpu.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
              binding: 1,
              resource: buffer_resolution.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
              binding: 2,
              resource: buffer_time.as_entire_binding(),
            }
          ]
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
      Event::MainEventsCleared => window.request_redraw(),
      _ => {},
    }
  });
}
