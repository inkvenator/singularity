use glium::implement_vertex;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;
use glium::Surface;
use std::f32::consts::PI;

// 1. Define the vertex format
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

#[derive(Default)]
pub struct App {
    display: Option<glium::Display<glium::glutin::surface::WindowSurface>>,
    window: Option<glium::winit::window::Window>,
    program: Option<glium::Program>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("Blackhole by ink ")
            .build(event_loop);

        // 2. Write the vertex and fragment shaders
        let vertex_shader_src = r#"
            #version 140
            in vec2 position;
            uniform mat4 matrix;
            void main() {
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            uniform vec3 color;
            out vec4 out_color;
            void main() {
                out_color = vec4(color, 1.0);
            }
        "#;

        let program = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            None,
        ).unwrap();

        // 3. Generate the circle vertices
        let num_segments = 100;
        let radius = 0.5; // Radius in NDC (Normalized Device Coordinates)
        let mut shape = Vec::new();

        // Push the center vertex for the TriangleFan
        shape.push(Vertex { position: [0.0, 0.0] });

        for i in 0..=num_segments {
            let angle = 2.0 * PI * (i as f32 / num_segments as f32);
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            shape.push(Vertex { position: [x, y] });
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

        self.window = Some(window);
        self.display = Some(display);
        self.program = Some(program);
        self.vertex_buffer = Some(vertex_buffer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let display = self.display.as_ref().unwrap();
                let mut frame = display.draw();

                frame.clear_color(0.0, 0.0, 0.0, 1.0);

                if let (Some(program), Some(vertex_buffer)) = (&self.program, &self.vertex_buffer) {

                    // GET THE CURRENT WINDOW DIMENSIONS
                    let (width, height) = frame.get_dimensions();
                    let aspect_ratio = height as f32 / width as f32;
                    let radius_scale = 1.0; // scale up or down as needed

                    // CREATE AN ASPECT RATIO MATRIX
                    // We scale the X axis by the aspect ratio so it squishes X properly.
                    let matrix = [
                        [aspect_ratio * radius_scale, 0.0, 0.0, 0.0],
                        [0.0,           1.0 * radius_scale, 0.0, 0.0],
                        [0.0,           0.0,                1.0, 0.0],
                        [0.0,           0.0,                0.0, 1.0f32],
                    ];

                    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

                    // PASS THE MATRIX IN THE UNIFORMS
                    let uniforms = glium::uniform! {
                        matrix: matrix,
                        color: [1.0f32, 0.0, 0.0],
                    };

                    frame
                        .draw(
                            vertex_buffer,
                            &indices,
                            program,
                            &uniforms,
                            &Default::default(),
                        )
                        .unwrap();
                }

                frame.finish().unwrap();
            }
            // Request redraw when resized so the circle doesn't vanish
            WindowEvent::Resized(size) => {
                if let Some(display) = &self.display {
                    display.resize(size.into());
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}