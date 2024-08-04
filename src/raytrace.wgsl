@group(0) @binding(0)
var<uniform> camera : Camera;

struct Camera {
    dimensions: vec2<f32>,
    focal: f32,
    viewport_height: f32,
    pos: f32,
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
}

struct RayHit {
    hit: bool,
    incoming: vec3<f32>,
    distance: f32,
    normal: vec3<f32>,
    colour: vec3<f32>,
}

@group(1) @binding(0)
var<storage, read> spheres: Spheres;

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

fn hit_sphere(sphere: Sphere, ray: Ray) -> RayHit {
    var dif: vec3<f32> = ray.pos - sphere.pos;
    var x: f32 = dot(dif, ray.dir);
    var y: f32 = dot(dif, dif) - (sphere.radius * sphere.radius);

    var d: f32 = x * x - y;

    if (d > 0.0) {
        var xy = sqrt(d);
        var root1 = -x - xy;
        if (root1 >= 0.0) {
            var ray_hit : RayHit;
            ray_hit.hit = true;
            ray_hit.incoming = ray.dir;
            ray_hit.distance = root1;
            ray_hit.normal = normalize((ray.pos + root1 * ray.dir) - sphere.pos);
            ray_hit.colour = sphere.colour;
            return ray_hit;
        }
        var root2 = -x + xy;
        if (root2 >= 0.0) {
            var ray_hit : RayHit;
            ray_hit.hit = true;
            ray_hit.incoming = ray.dir;
            ray_hit.distance = root2;
            ray_hit.normal = normalize((ray.pos + root2 * ray.dir) - sphere.pos);
            ray_hit.colour = sphere.colour;
            return ray_hit;
        }
    }
    var ray_hit : RayHit;
    return ray_hit;
}

fn ray_colour(ray: Ray) -> vec3<f32> {
    var hit : bool = false;
    var closest : RayHit;
    for (var i = 0; i < i32(arrayLength(&spheres.spheres)); i += 1) {
        var sphere = spheres.spheres[i];
        var ray_hit = hit_sphere(spheres.spheres[i], ray);

        if ray_hit.hit {
            if (!hit || closest.distance >= ray_hit.distance) {
                closest = ray_hit;
                hit = true;
            }
        }
    }
    if hit {
        var n = closest.normal;
        return closest.colour;
        // return 0.5 * vec3<f32>(n.x + 1.0, n.y + 1.0, n.z + 1.0);
    }

    var a = 0.5 * (normalize(ray.dir).y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
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

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ray: Ray = calc_ray(
        vec2<f32>(
            in.clip_position.x, // camera.dimensions.x,
            in.clip_position.y // camera.dimensions.y
        )
    );

    var colour = ray_colour(ray);
    return vec4<f32>(colour, 1.0);
}