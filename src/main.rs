#![feature(async_closure)]

/// Entry point for a standalone binary
fn main() {
    pollster::block_on(ray_tracer::run());
}
