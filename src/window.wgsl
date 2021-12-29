[[stage(vertex)]]
fn vs_main([[location(0)]] in: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
  return vec4<f32>(in, 1.0);
}
