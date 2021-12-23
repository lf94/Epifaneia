let bindingId = 0;

function compile(wgslString) {
  return {
    wgsl: [
      constants.emit(),
      vertexShader.emit(),
      polygon.emit(),
      wgslString
    ].join(''),
    rust: [].join('')
  };
}

function constants() {}
constants.emit = function() {
  return `  let X = 0; let Y = 1; let Z = 2;
  let nothing = 0.0;

  `;
};

function vertexShader() {}
vertexShader.emit = function() {
  return `  [[stage(vertex)]]
  fn vs_main([[location(0)]] in: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(in, 1.0);
  }`;
};

polygon.invocations = 0;
polygon.lists = [];
function polygon(l) {
  polygon.lists.push(l);
  return `polygon(p, ${polygon.invocations}u, ${l.length})`;
}
polygon.emit = function() {
  return `${polygon.lists.map((l,i) =>
    `[[group(0), binding(${bindingId})]] var<uniform> pnts${i}: array<vec2<f32>, ${l.length}>;`)
    .join('\n')}

  fn polygon(p: vec3<f32>, n: u32, l: i32) -> f32 {
    switch (n) {
      ${polygon.lists.map((_, i) => `case ${i}: { var v = pnts${i}; }`).join('\n')}
      default: { return 0.0; }
    }
    let p = vec2<f32>(p[X], p[Y]);
    let num = l;
    var d = dot(p-v[0], p-v[0]);
    var s = 1.0;
    var j = num - 1;
    for (var i = 0; i < num; i = i + 1) {
      let e = v[j] - v[i];
      let w = p - v[i];
      let b = w - e*clamp(dot(w,e)/dot(e,e), 0.0, 1.0);
      d = min(d, dot(b,b));
      let cond = vec3<bool>(p[Y] >= v[i][Y], p[Y] < v[j][Y], e[X]*w[Y] > e[Y]*w[X]);
      if (all(cond) || !all(cond)) { s = -s; }
      j = i;
    }
    return s * sqrt(d);
  }
  `;
};

const { wgsl } = compile(`
  fn extrude(p: vec3<f32>, d: f32, dv: f32) -> f32 {
    let h = d / 2.0;
    return max(abs(p[Z]) - h, dv);
  }

  fn sdf(p: vec3<f32>) -> f32 {
    var a: f32 = min(nothing, extrude(p, 4.0, ${polygon([[0.0, 0,0], [0.0, 1.0], [1.0, 0,0]])}));
    return a;
  }

  [[stage(fragment)]]
  fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }
`);

console.log(wgsl);
