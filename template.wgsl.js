const toonRayCaster = require('./raycasters/toon.js');

function compile(wgslString) {
  return {
    wgsl: [
      bindings.emit(),
      constants.emit(),
      vertexShader.emit(),
      polygon.emit(),
      shapes.emit(),
      wgslString
    ].join(''),
    data: [
      ...polygon.data()
    ],
  };
}

function constants() {}
constants.emit = function() {
  return `  let X = 0; let Y = 1; let Z = 2;
  let nothing = 0.0;

  `;
};

function bindings() {}
bindings.emit = function() {
  return `
    struct Resolution {
      xy: vec2<f32>;
    };

    struct Time {
      secs: f32;
    };

    [[group(0), binding(1)]] var<uniform> iResolution: Resolution;
    [[group(0), binding(2)]] var<uniform> iTime: Time;
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
function polygon(samplePointName, l) {
  polygon.lists.push(l);
  const i = polygon.invocations;
  polygon.invocations += 1;
  return `polygon(${samplePointName}.xy, ${i}u, ${l.length})`;
}
polygon.data = function() { return polygon.lists; };
polygon.emit = function() {
  if (polygon.lists.length == 0) { return ''; }

  return `
  struct Points {
  ${polygon.lists.map((l,i) => `    p${i}: array<vec2<f32>, ${l.length}>;`)
    .join('\n')}
  };

  [[group(0), binding(0)]] var<uniform> points: Points;

  fn polygon(p: vec2<f32>, n: u32, l: i32) -> f32 {
    switch (n) {
      ${polygon.lists.map((_, i) => `case ${i}: { var v = points.p${i}; }`).join('\n')}
      default: { return 0.0; }
    }
    var d: f32 = dot(p-v[0],p-v[0]);
    var s: f32 = 1.0;
    var j = l - 1; 
    for(var i=0; i<l; i = i + 1)
    {
        let e: vec2<f32> = v[j] - v[i];
        let w: vec2<f32> =    p - v[i];
        let b: vec2<f32> = w - e*clamp( dot(w,e)/dot(e,e), 0.0, 1.0 );
        d = min( d, dot(b,b) );
        let c = vec3<bool>(p.y>=v[i].y,p.y<v[j].y,e.x*w.y>e.y*w.x);
        if( all(c) || all(!(c)) ) { s = s * -1.0; }
        j=i;
    }
    return s*sqrt(d);
  }

  `;
};

function shapes() {}
shapes.emit = function() {
  return `
    fn torus(samplePoint: vec3<f32>, dimensions: vec2<f32>) -> f32 {
    	return length( vec2<f32>(length(samplePoint.xz)-dimensions.x,samplePoint.y) )-dimensions.y;
    }
    fn sphere(p: vec3<f32>, diameter: f32) -> f32 {
      return length(p) - (diameter / 2.0);
    }
    fn circle(p: vec2<f32>, diameter: f32) -> f32 {
      return length(p) - (diameter / 2.0);
    }
    fn extrude(p: vec3<f32>, d: f32, dv: f32) -> f32 {
      let h = d / 2.0;
      return max(abs(p[Z]) - h, dv);
    }

  `;
};

// p is the name for the sample point which is a vec3<f32>.
// iTime is available.
// iResolution is available.
const p = 'p';
const { wgsl, data } = compile(toonRayCaster(p, `
  var d: f32 = sphere(${p}, 0.1);
  var j: f32 = 0.0;
  var np = vec3<f32>(0.0, 0.0, 0.0);
  for(var i = 0; i < 1000; i = i + 1) {
    np = ${p} + vec3<f32>(j, sin(j) * 1.0, 0.0);
    d = min(d, sphere(np, 0.1));
    j = j + 0.005;
  }
  return d;
`));

console.log(JSON.stringify({ text: wgsl, data }));
