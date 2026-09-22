use glium::implement_vertex;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;
use glium::Surface;
use std::f32::consts::PI;
use crate::physics::Ray;
use glam::DVec2;

// 1. Define the vertex format
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

pub struct App {
    display: Option<glium::Display<glium::glutin::surface::WindowSurface>>,
    window: Option<glium::winit::window::Window>,
    program: Option<glium::Program>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
    rays: Vec<Ray>,
    rs: f64,
}

const WORLD_TO_NDC: f32 = 1.0 / 6.0;

impl Default for App {
    fn default() -> Self {
        let rs = 1.0_f64;
        let rays_vec = vec![Ray::new(DVec2::new(-5.0, 1.5), DVec2::new(1.0, 0.0), rs),
                                    Ray::new(DVec2::new(-5.0, 3.5), DVec2::new(1.0, 0.0), rs),
                                    Ray::new(DVec2::new(-5.0, 5.5), DVec2::new(1.0, 0.0), rs),
                                    Ray::new(DVec2::new(-5.0, -3.5), DVec2::new(1.0, 0.0), rs),
                                    Ray::new(DVec2::new(-5.0, -2.5), DVec2::new(1.0, 0.0), rs)

        ];
        App {
            display: None, window: None, program: None, vertex_buffer: None,
            rays: rays_vec,
            rs,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("singularity")
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
        let radius = self.rs as f32; // Radius in NDC (Normalized Device Coordinates)
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
                    

                    // CREATE AN ASPECT RATIO MATRIX
                    // We scale the X axis by the aspect ratio so it squishes X properly.

                    let matrix = [
                        [aspect_ratio * WORLD_TO_NDC, 0.0, 0.0, 0.0],
                        [0.0, WORLD_TO_NDC, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
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

                    for ray in &mut self.rays {
                        if ray.alive{
                            ray.step(0.01, self.rs, 20.0);

                        }
                    }
                    
                    
                    for ray in &self.rays{
                        if ray.trail.len() >= 2 {
                        let trail_verts: Vec<Vertex> = ray.trail.iter()
                            .map(|p| Vertex { position: *p })
                            .collect();
                        let trail_vb = glium::VertexBuffer::new(display, &trail_verts).unwrap();
                        let line_indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);

                        frame.draw(
                            &trail_vb,
                            &line_indices,
                            program,
                            &glium::uniform! { matrix: matrix, color: [1.0f32, 1.0, 1.0] },
                            &Default::default(),
                        ).unwrap();
                    }
                    }
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(window) = &self.window {
        window.request_redraw();
    }
}
}