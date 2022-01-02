[[group(0), binding(0)]] var texture_sdf: texture_2d<f32>;
[[group(0), binding(1)]] var sampler_sdf: sampler;

struct VertexInput {
  [[location(0)]] position: vec3<f32>;
  [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>;
  [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = vec4<f32>(in.position, 1.0);
  out.tex_coords = in.tex_coords;
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
 return textureSample(texture_sdf, sampler_sdf, in.tex_coords);
}
