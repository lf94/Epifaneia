use wgpu::util::DeviceExt;
use serde_json::Value;

pub struct PipelineSDF {
  render_pipeline: wgpu::RenderPipeline,
  bind_group_layout: wgpu::BindGroupLayout,
}

impl PipelineSDF {
  pub fn new(device: &wgpu::Device, shader_text: &str) -> Self {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Shader SDF"),
      source: wgpu::ShaderSource::Wgsl(shader_text.into()),
    });

    let vertex_buffer_layout = wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3],
    };

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

    let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
      label: Some("RenderPipelineDesc SDF"),
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
          format: wgpu::TextureFormat::Rgba8Unorm,
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
    };

    PipelineSDF {
      render_pipeline: device.create_render_pipeline(&render_pipeline_desc),
      bind_group_layout
    }
  }

  pub fn json_points_to_gpu_buffer(device: &wgpu::Device, points: &Value) -> wgpu::Buffer {
    let mut buffer_gpu_contents: Vec<u8> = vec![];

    match &points {
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

    return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&buffer_gpu_contents),
      usage: wgpu::BufferUsages::UNIFORM,
    });
  }

  pub fn render_pass(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    time: std::time::Instant,
    buffer_vertices: &wgpu::Buffer,
    buffer_resolution: &wgpu::Buffer,
    buffer_points: &wgpu::Buffer
  ) -> wgpu::Texture {
    let buffer_time = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: 
        &time
        .elapsed()
        .as_secs_f32().to_bits().to_le_bytes(),
      usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &self.bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer_points.as_entire_binding(),
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

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: None,
      size: wgpu::Extent3d {
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: None,
    });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[
          wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.8, b: 0.3, a: 1.0 }),
              store: true
            }
          }
        ],
        depth_stencil_attachment: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_vertex_buffer(0, buffer_vertices.slice(..));
      render_pass.set_bind_group(0, &bind_group, &[]);
      render_pass.draw(0..4, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));

    return texture;
  }
}
