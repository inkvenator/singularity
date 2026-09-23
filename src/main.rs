use glium::winit::event_loop::{ControlFlow, EventLoop};
mod app;
mod physics;
mod camera;


fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::default();
    event_loop.run_app(&mut app).unwrap();
}