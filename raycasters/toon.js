//
// Shader for SDF cel-shading. Copyright Dalton of Shadertoy.
// https://www.shadertoy.com/view/ll33Wn
//
module.exports = function(samplePointName, sdfFunctionBlockText) { return `
let EPSILON = 0.0001;
let MAX_STEPS = 500;
let MIN_DIST = 0.0;
let MAX_DIST = 25.0;
let AMBIENT = 0.1;
let EDGE_THICKNESS = 0.015;
let SHADES = 1.0;
  
fn SceneSDF(${samplePointName}: vec3<f32>) -> f32
{
  ${sdfFunctionBlockText}
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
`;
};
