[[group(0), binding(0)]] var texture_sdf: texture_2d<f32>;
[[group(0), binding(1)]] var sampler_sdf: sampler;

[[stage(vertex)]]
fn vs_main([[location(0)]] in: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
  return vec4<f32>(in, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
 return textureSample(texture_sdf, sampler_sdf, vec2<f32>(0.0, 1.0));
}
