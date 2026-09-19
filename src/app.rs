use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;
use glium::Surface;

#[derive(Default)]
pub struct App {
    display: Option<glium::Display<glium::glutin::surface::WindowSurface>>,
    window: Option<glium::winit::window::Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("Blackhole by ink ")
            .build(event_loop);
        self.window = Some(window);
        self.display = Some(display);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let display = self.display.as_ref().unwrap();
                let mut frame = display.draw();
                frame.clear_color(1.0, 0.0, 1.0, 1.0);
                frame.finish().unwrap();
            }
            _ => (),
        }
    }
}