@group(0) @binding(0)
var<uniform> camera : Camera;

@group(1) @binding(0)
var<storage, read> spheres: Spheres;

const SAMPLE_COUNT = 4;
const SAMPLES = array<vec2<f32>, SAMPLE_COUNT>(
    vec2<f32>(-0.25, -0.25),
    vec2<f32>(-0.25, 0.25),
    vec2<f32>(0.25, -0.25),
    vec2<f32>(0.25, 0.25),
);

struct Camera {
    dimensions: vec2<f32>,
    focal: f32,
    viewport_height: f32,
    pos: vec3<f32>,
    max_depth: i32,
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct RayHit {
    hit: bool,
    distance: f32,
    pos: vec3<f32>,
    normal: vec3<f32>,
    colour: vec3<f32>,
    reflection: f32,
}

struct Spheres {
    @align(16)
    spheres: array<Sphere>,
};

struct Sphere {
    pos: vec3<f32>,
    radius: f32,
    colour: vec3<f32>,
    reflection: f32,
}

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

var<private> seed: f32 = 0.0;

fn base_hash(p: vec2<u32>) -> u32 {
    var p_shifted = vec2<u32>(p.x >> u32(1), p.y >> u32(1));
    var q = u32(1103515245) * ((p_shifted) ^ (p.yx));
    var h32 = u32(1103515245) * ((q.x) ^ (q.y >> u32(3)));
    return h32 ^ (h32 >> u32(16));
}

fn hash3(seed: ptr<private, f32>) -> vec3<f32> {
    var l = *seed;
    *seed += 0.1;
    var r = *seed;
    *seed += 0.1;

    var n = base_hash(vec2<u32>(vec2<f32>(l, r)));
    var rz = vec3<u32>(
        n & u32(0x7fffffff),
        (n * u32(16807)) & u32(0x7fffffff),
        (n * u32(48271)) & u32(0x7fffffff)
    );
    return vec3<f32>(rz) / f32(0x7fffffff);
}

fn random_in_unit_sphere(seed: ptr<private, f32>) -> vec3<f32> {
    var h = hash3(seed) * vec3<f32>(2., 6.28318530718, 1.) - vec3<f32>(1.0, 0.0, 0.0);
    var phi = h.y;
    var r = pow(h.z, 1. / 3.);
    return r * vec3(sqrt(1. - h.x * h.x) * vec2(sin(phi), cos(phi)), h.x);
}

fn hit_sphere(sphere: Sphere, ray: Ray) -> RayHit {
    var dif: vec3<f32> = ray.pos - sphere.pos;
    var x: f32 = dot(dif, ray.dir);
    var y: f32 = dot(dif, dif) - (sphere.radius * sphere.radius);

    var d: f32 = x * x - y;

    if d > 0.0 {
        var xy = sqrt(d);
        var root1 = -x - xy;
        if root1 >= 0.0 {
            var ray_hit: RayHit;
            ray_hit.hit = true;
            ray_hit.distance = root1;
            ray_hit.pos = ray.pos + root1 * ray.dir;
            ray_hit.normal = normalize(ray_hit.pos - sphere.pos);

            ray_hit.colour = sphere.colour;
            ray_hit.reflection = sphere.reflection;

            return ray_hit;
        }
        var root2 = -x + xy;
        if root2 >= 0.0 {
            var ray_hit: RayHit;
            ray_hit.hit = true;
            ray_hit.distance = root2;
            ray_hit.pos = ray.pos + root2 * ray.dir;
            ray_hit.normal = normalize(ray_hit.pos - sphere.pos);

            ray_hit.colour = sphere.colour;
            ray_hit.reflection = sphere.reflection;

            return ray_hit;
        }
    }
    var ray_hit: RayHit;
    return ray_hit;
}

fn sky_colour(ray: Ray) -> vec3<f32> {
    var a = 0.5 * (normalize(ray.dir).y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}

fn iterative_ray_colour(ray: Ray) -> vec3<f32> {
    var cumulative_colour: vec3<f32>;
    var colour_multiplier: f32 = 1.0;

    var current_ray: Ray = ray;

    for (var depth = 0; depth < camera.max_depth; depth += 1) {
        var hit_out = cast_ray(current_ray);
        if hit_out.hit {
            cumulative_colour += colour_multiplier * hit_out.colour;
            colour_multiplier *= hit_out.reflection;

            var direction = hit_out.normal * 1.001 + random_in_unit_sphere(&seed);
            current_ray.dir = direction;
            current_ray.pos = hit_out.pos;
        } else {
            cumulative_colour += colour_multiplier * sky_colour(ray);
            break;
        }
    }
    return cumulative_colour;
}

fn cast_ray(ray: Ray) -> RayHit {
    var hit: bool = false;
    var closest: RayHit;
    for (var i = 0; i < i32(arrayLength(&spheres.spheres)); i += 1) {
        var sphere = spheres.spheres[i];
        var ray_hit = hit_sphere(spheres.spheres[i], ray);

        if ray_hit.hit {
            if !hit || closest.distance >= ray_hit.distance {
                closest = ray_hit;
                hit = true;
            }
        }
    }
    return closest;
}

fn calc_ray(screen_pos: vec2<f32>) -> Ray {
    var focal_length = 1.0;
    var viewport_width = camera.viewport_height * (camera.dimensions.x / camera.dimensions.y);

    var viewport_u: vec3<f32> = vec3<f32>(viewport_width, 0.0, 0.0);
    var viewport_v = vec3<f32>(0.0, -camera.viewport_height, 0.0);

    var pixel_delta_u = viewport_u / camera.dimensions.x;
    var pixel_delta_v = viewport_v / camera.dimensions.y;

    var viewport_upper_left = camera.pos - vec3<f32>(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    var pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    var pixel_center = pixel00_loc + (screen_pos.x * pixel_delta_u) + (screen_pos.y * pixel_delta_v);
    var ray_direction = pixel_center - camera.pos;

    var ray: Ray;
    ray.dir = ray_direction;
    ray.pos = pixel_center;
    return ray;
}

fn gamma_correction(in: f32) -> f32 {
    if in > 0.0 {
        return sqrt(in);
    } else {
        return 0.0;
    }
}

fn gamma_correction_vec(in: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        gamma_correction(in.x),
        gamma_correction(in.y),
        gamma_correction(in.z),
    );
}

fn cast_multiple_rays(origin: vec2<f32>) -> vec3<f32> {
    var pixel_colour: vec3<f32>;
    pixel_colour += iterative_ray_colour(calc_ray(origin + SAMPLES[0]));
    pixel_colour += iterative_ray_colour(calc_ray(origin + SAMPLES[1]));
    pixel_colour += iterative_ray_colour(calc_ray(origin + SAMPLES[2]));
    pixel_colour += iterative_ray_colour(calc_ray(origin + SAMPLES[3]));
    return gamma_correction_vec(pixel_colour / 4.0);
}


// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    seed = f32(base_hash(vec2<u32>(in.clip_position.xy)));
    return vec4<f32>(cast_multiple_rays(in.clip_position.xy), 1.0);
}