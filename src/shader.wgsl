
    struct Resolution {
      xy: vec2<f32>;
    };

    struct Time {
      secs: f32;
    };

    [[group(0), binding(1)]] var<uniform> iResolution: Resolution;
    [[group(0), binding(2)]] var<uniform> iTime: Time;
    let X = 0; let Y = 1; let Z = 2;
  let nothing = 0.0;

    [[stage(vertex)]]
  fn vs_main([[location(0)]] in: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(in, 1.0);
  }
  struct Points {
    p0: array<vec2<f32>, 3>;

  };

  [[group(0), binding(0)]] var<uniform> points: Points;

  fn polygon(p: vec2<f32>, n: u32, l: i32) -> f32 {
    switch (n) {
      case 0: { var v = points.p0; }
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

  
let EPSILON = 0.0001;
let MAX_STEPS = 500;
let MIN_DIST = 0.0;
let MAX_DIST = 100.0;
let AMBIENT = 0.1;
let EDGE_THICKNESS = 0.015;
let SHADES = 4.0;
  
fn SceneSDF(p: vec3<f32>) -> f32
{
  
  return torus(p, vec2<f32>(1.3, 0.45));
  //return extrude(p, 1.0, polygon(p.xy, 0u, 3));
  //return min(
  //  extrude(p, 4.0, circle(p.xy + vec2<f32>(1.0, 1.0), 2.0)),
  //  extrude(p, 4.0, circle(p.xy, 2.0))
  //);

}

fn March(origin: vec3<f32>, direction: vec3<f32>, start: f32, stop: f32) -> f32
{
    var depth: f32 = start;
    var edgeLength: f32 = MAX_DIST;
    
    for (var i = 0; i < MAX_STEPS; i = i + 1)
    {
        let dist = SceneSDF(origin + (depth * direction));
        edgeLength = min(dist, edgeLength);
        
        if (dist < EPSILON) { // Hit
            return depth;
        }
        
        if (dist > edgeLength && edgeLength <= EDGE_THICKNESS ) { // Edge hit
            return 0.0;
        }
        
        depth = depth + dist; // Step
        
        if (depth >= stop) { // Reached max
            break;
        }
    }
    
    return stop;
}

fn radians(a: f32) -> f32 {
  return a * 3.141 / 180.0;
}

fn RayDirection(fov: f32, size: vec2<f32>, fragCoord: vec2<f32>) -> vec3<f32>
{
    let xy: vec2<f32> = fragCoord - (size / 2.0);
    let z: f32= size.y / tan(radians(fov) / 2.0);
    return normalize(vec3<f32>(xy, -z));
}

fn EstimateNormal(point: vec3<f32>) -> vec3<f32>
{
    return normalize(vec3<f32>(SceneSDF(vec3<f32>(point.x + EPSILON, point.y, point.z)) - SceneSDF(vec3<f32>(point.x - EPSILON, point.y, point.z)),
                          SceneSDF(vec3<f32>(point.x, point.y + EPSILON, point.z)) - SceneSDF(vec3<f32>(point.x, point.y - EPSILON, point.z)),
                          SceneSDF(vec3<f32>(point.x, point.y, point.z + EPSILON)) - SceneSDF(vec3<f32>(point.x, point.y, point.z - EPSILON))));
}

fn LookAt(camera: vec3<f32>, target: vec3<f32>, up: vec3<f32>) -> mat4x4<f32>
{
    let f: vec3<f32> = normalize(target - camera);
    let s: vec3<f32> = cross(f, up);
    let u: vec3<f32> = cross(s, f);
    
    return mat4x4<f32>(vec4<f32>(s, 0.0),
        		vec4<f32>(u, 0.0),
        		vec4<f32>(f * -1.0, 0.0),
        		vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn ComputeLighting(point: vec3<f32>, lightDir: vec3<f32>, lightColor: vec3<f32>) -> vec3<f32>
{
    var color: vec3<f32>= vec3<f32>(AMBIENT);
    var intensity: f32 = dot(EstimateNormal(point), normalize(lightDir));
    intensity = ceil(intensity * SHADES) / SHADES;
    intensity = max(intensity, AMBIENT);
    color = lightColor * intensity;
    return color;
}

  [[stage(fragment)]]
  fn fs_main([[builtin(position)]] in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var fragCoord = in;
    var fragColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    var viewDir: vec3<f32> = RayDirection(45.0, iResolution.xy, fragCoord.xy);
    let origin = vec3<f32>(sin(iTime.secs) * 9.0, (sin(iTime.secs * 2.0) * 4.0) + 6.0, cos(iTime.secs) * 9.0);
    let viewTransform = LookAt(origin, vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
    viewDir = (viewTransform * vec4<f32>(viewDir, 0.0)).xyz;
    
    let dist: f32 = March(origin, viewDir, MIN_DIST, MAX_DIST);
    
    if (dist > MAX_DIST - EPSILON) // No hit
    {
        fragColor = vec4<f32>(0.6, 0.6, 0.6, 0.6);
        return fragColor;
    }
    
    if (dist < EPSILON) // Edge hit
    {
        fragColor = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return fragColor;
    }
    
    let hitPoint: vec3<f32> = origin + (dist * viewDir);
    let lightDir: vec3<f32> = vec3<f32>(sin(iTime.secs * 2.0) * 6.0, 4.0, sin(iTime.secs * 1.25) * 5.0);
    var color: vec3<f32> = vec3<f32>(1.0, 0.5, 0.1);
    
    color = ComputeLighting(hitPoint, lightDir, color);
    
    fragColor = vec4<f32>(color, 1.0);
    return fragColor;
  }

