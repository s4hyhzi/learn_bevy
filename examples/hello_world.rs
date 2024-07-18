use legion::*;
use wgsl_study::Application;


// a component is any type that is 'static, sized, send and sync
#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

struct SimpleApp;

impl Application for SimpleApp {
    fn update(&self) {}
}

fn main() {
    let mut world = World::default();
    let _entity: Entity = world.push((Position { x: 0.0, y: 0.0 }, Velocity { dx: 0.0, dy: 0.0 }));
    let app = SimpleApp {};
    pollster::block_on(app.start());
}
