use glium::implement_vertex;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;
use glium::Surface;
use std::f32::consts::PI;
use crate::physics::Ray;
use glam::DVec2;
use crate::camera::Camera;
use glium::winit::event::{ElementState, MouseButton, MouseScrollDelta};

#[derive(Copy, Clone)]
struct Vertex3D {
    position: [f32; 3],
}
implement_vertex!(Vertex3D, position);

pub struct App {
    display: Option<glium::Display<glium::glutin::surface::WindowSurface>>,
    window: Option<glium::winit::window::Window>,
    program: Option<glium::Program>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex3D>>,
    index_buffer: Option<glium::IndexBuffer<u32>>,
    rays: Vec<Ray>,
    rs: f64,
    camera: Camera,
    last_cursor: (f64, f64),

    // Grid rendering
    grid_program: Option<glium::Program>,
    grid_vertex_buffer: Option<glium::VertexBuffer<Vertex3D>>,
    grid_index_buffer: Option<glium::IndexBuffer<u32>>,
}

impl Default for App {
    fn default() -> Self {
        let mut camera = Camera::default();
        camera.radius = 6.0;
        let rs = 1.0_f64;

        App {
            display: None, 
            window: None, 
            program: None, 
            vertex_buffer: None,
            index_buffer: None,
            rays: Vec::new(),
            rs,
            camera,
            last_cursor: (0.0, 0.0),
            grid_program: None,
            grid_vertex_buffer: None,
            grid_index_buffer: None,
        }
    }
}

impl App {
    /// Generate a 3D UV Sphere mesh for the Event Horizon
    fn generate_sphere(
        &self, 
        display: &glium::Display<glium::glutin::surface::WindowSurface>,
        rings: u32,
        sectors: u32,
    ) -> (glium::VertexBuffer<Vertex3D>, glium::IndexBuffer<u32>) {
        let radius = self.rs as f32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 1. Generate sphere vertices using spherical coordinates
        for i in 0..=rings {
            let theta = i as f32 * PI / rings as f32; // 0 to PI (north pole to south pole)
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for j in 0..=sectors {
                let phi = j as f32 * 2.0 * PI / sectors as f32; // 0 to 2*PI (around the equator)
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = radius * sin_theta * cos_phi;
                let y = radius * cos_theta; // Y is up
                let z = radius * sin_theta * sin_phi;

                vertices.push(Vertex3D { position: [x, y, z] });
            }
        }

        // 2. Connect vertices into triangles
        for i in 0..rings {
            for j in 0..sectors {
                let first = i * (sectors + 1) + j;
                let second = first + sectors + 1;

                // Triangle 1
                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                // Triangle 2
                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        let vb = glium::VertexBuffer::new(display, &vertices).unwrap();
        let ib = glium::IndexBuffer::new(
            display, 
            glium::index::PrimitiveType::TrianglesList, 
            &indices
        ).unwrap();

        (vb, ib)
    }

    /// Generate Flamm's paraboloid grid mesh
    fn generate_grid(&self, display: &glium::Display<glium::glutin::surface::WindowSurface>)
        -> (glium::VertexBuffer<Vertex3D>, glium::IndexBuffer<u32>)
    {
        const GRID_SIZE: usize = 25;
        const SPACING: f32 = 0.4;

        let mut vertices = Vec::new();
        let rs = self.rs as f32;

        // Generate grid vertices
        for z in 0..=GRID_SIZE {
            for x in 0..=GRID_SIZE {
                let world_x = (x as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
                let world_z = (z as f32 - GRID_SIZE as f32 / 2.0) * SPACING;

                let r = (world_x * world_x + world_z * world_z).sqrt();

                // Flamm's paraboloid formula: y = 2·√(rs·(r − rs))
                let y = if r > rs {
                    2.0 * (rs * (r - rs)).sqrt()
                } else {
                    // Inside event horizon: create a pit
                    -2.0 * (rs * rs).sqrt()
                };

                vertices.push(Vertex3D {
                    position: [world_x, y, world_z],
                });
            }
        }

        // Generate line indices (connect adjacent points)
        let mut indices = Vec::new();
        for z in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let i = (z * (GRID_SIZE + 1) + x) as u32;

                // Horizontal line
                indices.push(i);
                indices.push(i + 1);

                // Vertical line
                indices.push(i);
                indices.push(i + (GRID_SIZE + 1) as u32);
            }
        }

        let vb = glium::VertexBuffer::new(display, &vertices).unwrap();
        let ib = glium::IndexBuffer::new(display, glium::index::PrimitiveType::LinesList, &indices).unwrap();

        (vb, ib)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("singularity")
            .build(event_loop);

        // Shaders for the 3D Sphere
        let vertex_shader_src = r#"
            #version 140
            in vec3 position;
            uniform mat4 view_proj;
            void main() {
                gl_Position = view_proj * vec4(position, 1.0);
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

        // Shaders for the 3D Grid
        let grid_vertex_shader = r#"
            #version 140
            in vec3 position;
            uniform mat4 view_proj;
            void main() {
                gl_Position = view_proj * vec4(position, 1.0);
            }
        "#;

        let grid_fragment_shader = r#"
            #version 140
            out vec4 out_color;
            void main() {
                out_color = vec4(1.0, 1.0, 1.0, 0.3); // Semi-transparent white
            }
        "#;

        let grid_program = glium::Program::from_source(
            &display,
            grid_vertex_shader,
            grid_fragment_shader,
            None,
        ).unwrap();

        // Generate Flamm's paraboloid grid
        let (grid_vb, grid_ib) = self.generate_grid(&display);

        // Generate 3D UV Sphere for Event Horizon (32 rings, 32 sectors)
        let (sphere_vb, sphere_ib) = self.generate_sphere(&display, 32, 32);

        self.window = Some(window);
        self.display = Some(display);
        self.program = Some(program);
        self.vertex_buffer = Some(sphere_vb);
        self.index_buffer = Some(sphere_ib);
        self.grid_program = Some(grid_program);
        self.grid_vertex_buffer = Some(grid_vb);
        self.grid_index_buffer = Some(grid_ib);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.camera.start_drag(self.last_cursor.0, self.last_cursor.1);
                        }
                        ElementState::Released => self.camera.end_drag(),
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor = (position.x, position.y);
                self.camera.mouse_moved(position.x, position.y);
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 20.0) as f32,
                };
                self.camera.scroll(y);
            }

            WindowEvent::RedrawRequested => {
                let display = self.display.as_ref().unwrap();
                let mut frame = display.draw();

                // Clear both color buffer AND depth buffer
                frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                let (width, height) = frame.get_dimensions();
                let aspect = width as f32 / height as f32;

                let view = self.camera.view_matrix();
                let proj = self.camera.projection_matrix(aspect, 60.0_f32.to_radians());
                let view_proj = proj * view;

                // Shared depth parameter configuration for 3D drawing
                let depth_params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                // 1. Draw 3D Event Horizon Sphere
                if let (Some(program), Some(vertex_buffer), Some(index_buffer)) = 
                    (&self.program, &self.vertex_buffer, &self.index_buffer) 
                {
                    let uniforms = glium::uniform! {
                        view_proj: view_proj.to_cols_array_2d(),
                        color: [1.0f32, 0.0, 0.0],
                    };

                    frame.draw(vertex_buffer, index_buffer, program, &uniforms, &depth_params).unwrap();
                }

                // 2. Draw 3D Flamm's Paraboloid Grid
                if let (Some(grid_prog), Some(grid_vb), Some(grid_ib)) =
                    (&self.grid_program, &self.grid_vertex_buffer, &self.grid_index_buffer)
                {
                    let grid_params = glium::DrawParameters {
                        blend: glium::Blend::alpha_blending(),
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    frame.draw(
                        grid_vb,
                        grid_ib,
                        grid_prog,
                        &glium::uniform! {
                            view_proj: view_proj.to_cols_array_2d(),
                        },
                        &grid_params,
                    ).unwrap();
                }

                frame.finish().unwrap();
            }

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