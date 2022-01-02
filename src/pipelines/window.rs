pub struct PipelineWindow {
  render_pipeline: wgpu::RenderPipeline,
  texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl PipelineWindow {
  pub fn new(device: &wgpu::Device, surface: &wgpu::Surface, adapter: &wgpu::Adapter) -> Self {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Shader Window"),
      source: wgpu::ShaderSource::Wgsl(include_str!("./window.wgsl").into()),
    });

    let vertex_buffer_layout = wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<[f32;5]>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
    };

    let texture_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              view_dimension: wgpu::TextureViewDimension::D2,
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ],
        label: Some("TextureBindGroupLayout Window"),
      }
    );
 
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&texture_bind_group_layout],
      push_constant_ranges: &[],
    });

    let format = surface.get_preferred_format(adapter).unwrap().clone();

    let targets = &[wgpu::ColorTargetState {
      format,
      blend: Some(wgpu::BlendState::REPLACE),
      write_mask: wgpu::ColorWrites::ALL,
    }];

    let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
      label: Some("RenderPipelineDesc Window"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[vertex_buffer_layout],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets,
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

    PipelineWindow {
      render_pipeline: device.create_render_pipeline(&render_pipeline_desc),
      texture_bind_group_layout
    }
  }

  pub fn render_pass(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame_view: &wgpu::TextureView,
    buffer_vertices: &wgpu::Buffer,
    texture_sdf: &wgpu::Texture,
    texture_sampler: &wgpu::Sampler,
  ) -> () {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("window_encoder"),
    });

    let bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &self.texture_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
              &texture_sdf.create_view(&wgpu::TextureViewDescriptor::default())
            ),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(texture_sampler),
          }
        ],
        label: Some("BindGroup Window"),
      }
    );

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[
          wgpu::RenderPassColorAttachment {
            view: frame_view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }),
              store: true
            }
          }
        ],
        depth_stencil_attachment: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &bind_group, &[]);
      render_pass.set_vertex_buffer(0, buffer_vertices.slice(..));
      render_pass.draw(0..4, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
  }
}
