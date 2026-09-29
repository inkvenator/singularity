use glium::implement_vertex;
use glium::winit::application::ApplicationHandler;
use glium::winit::event::WindowEvent;
use glium::winit::event_loop::ActiveEventLoop;
use glium::winit::window::WindowId;
use glium::Surface;
use crate::camera::Camera;
use glium::winit::event::{ElementState, MouseButton, MouseScrollDelta, KeyEvent};
use glium::winit::keyboard::{PhysicalKey, KeyCode};

#[derive(Copy, Clone)]
struct Vertex2D {
    position: [f32; 2],
}
implement_vertex!(Vertex2D, position);

#[derive(Copy, Clone)]
struct Vertex3D {
    position: [f32; 3],
}
implement_vertex!(Vertex3D, position);

pub struct App {
    display: Option<glium::Display<glium::glutin::surface::WindowSurface>>,
    window: Option<glium::winit::window::Window>,
    rs: f64,
    camera: Camera,
    last_cursor: (f64, f64),

    quad_program: Option<glium::Program>,
    quad_vertex_buffer:Option<glium::VertexBuffer<Vertex2D>>,
    
    // Grid rendering
    grid_program: Option<glium::Program>,
    grid_vertex_buffer: Option<glium::VertexBuffer<Vertex3D>>,
    grid_index_buffer: Option<glium::IndexBuffer<u32>>,

    show_grid: bool,
 
}

impl Default for App {
    fn default() -> Self {
        let mut camera = Camera::default();
        camera.radius = 6.0;
        let rs = 0.5_f64;

        App {
            display: None, 
            window: None, 
            rs,
            camera,
            last_cursor: (0.0, 0.0),
            grid_program: None,
            grid_vertex_buffer: None,
            grid_index_buffer: None,
            quad_program: None,
            quad_vertex_buffer: None,

            show_grid: true,
            
        }
    }
}

impl App {
    
    /// Generate Flamm's paraboloid grid mesh
    fn generate_grid(&self, display: &glium::Display<glium::glutin::surface::WindowSurface>)
        -> (glium::VertexBuffer<Vertex3D>, glium::IndexBuffer<u32>)
    {
        const GRID_SIZE: usize = 25;
        const SPACING: f32 = 0.4;
        const NAN_MARKER: f32 = f32::NAN;

        let rs = self.rs as f32;

        // Calibrate so the grid's far edges sit near y = 0, matching the
        // real (flat) plane the disk/rays are computed in. Not derived from
        // physics — purely a visual offset, tuned to this grid's own extent.
        let max_r = (GRID_SIZE as f32 * SPACING / 2.0) * std::f32::consts::SQRT_2;
        let offset = 2.0 * (rs * (max_r - rs)).sqrt();

        let mut vertices = Vec::new();

        for z in 0..=GRID_SIZE {
            for x in 0..=GRID_SIZE {
                let world_x = (x as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
                let world_z = (z as f32 - GRID_SIZE as f32 / 2.0) * SPACING;

                let r = (world_x * world_x + world_z * world_z).sqrt();

                if r > rs {
                    let y = 2.0 * (rs * (r - rs)).sqrt() - offset;
                    vertices.push(Vertex3D { position: [world_x, y, world_z] });
                } else {
                    // Inside the horizon: mark as unusable rather than
                    // clamping to a flat plateau. Filtered out below.
                    vertices.push(Vertex3D { position: [NAN_MARKER, NAN_MARKER, NAN_MARKER] });
                }
            }
        }

        // Only emit a line if both endpoints are valid (non-NaN).
        let mut indices = Vec::new();
        let valid = |i: usize| !vertices[i].position[0].is_nan();

        for z in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let i = z * (GRID_SIZE + 1) + x;

                // Horizontal segment: i -> i+1
                if valid(i) && valid(i + 1) {
                    indices.push(i as u32);
                    indices.push((i + 1) as u32);
                }

                // Vertical segment: i -> i + (GRID_SIZE+1)
                if valid(i) && valid(i + GRID_SIZE + 1) {
                    indices.push(i as u32);
                    indices.push((i + GRID_SIZE + 1) as u32);
                }
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

        let quad_vertex_shader =include_str!("../shaders/quad.vert");

        let quad_fragment_shader = include_str!("../shaders/quad.frag");

        let quad_program = glium::Program::from_source(
            &display, quad_vertex_shader, quad_fragment_shader, None,
        ).unwrap();

        let quad_verts = vec![
            Vertex2D { position: [-1.0, -1.0] },
            Vertex2D { position: [ 1.0, -1.0] },
            Vertex2D { position: [-1.0,  1.0] },
            Vertex2D { position: [ 1.0,  1.0] },
        ];
        let quad_vb = glium::VertexBuffer::new(&display, &quad_verts).unwrap();


        // Shaders for the 3D Grid
        let grid_vertex_shader = include_str!("../shaders/grid.vert");

        let grid_fragment_shader = include_str!("../shaders/grid.frag");

        let grid_program = glium::Program::from_source(
            &display,
            grid_vertex_shader,
            grid_fragment_shader,
            None,
        ).unwrap();

        // Generate Flamm's paraboloid grid
        let (grid_vb, grid_ib) = self.generate_grid(&display);

        self.window = Some(window);
        self.display = Some(display);
        self.grid_program = Some(grid_program);
        self.grid_vertex_buffer = Some(grid_vb);
        self.grid_index_buffer = Some(grid_ib);
        self.quad_program = Some(quad_program);
        self.quad_vertex_buffer = Some(quad_vb);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::KeyG),
                    state: ElementState::Pressed,
                    repeat: false,
                    .. },
            .. } => {
                self.show_grid = !self.show_grid;
            }

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

                if let (Some(quad_prog), Some(quad_vb)) = (&self.quad_program, &self.quad_vertex_buffer) {
                let (forward, right, up) = self.camera.basis();
                let cam_pos = self.camera.position();

                let quad_params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

                frame.draw(
                    quad_vb,
                    &indices,
                    quad_prog,
                    &glium::uniform! {
                        cam_pos: cam_pos.to_array(),
                        cam_forward: forward.to_array(),
                        cam_right: right.to_array(),
                        cam_up: up.to_array(),
                        fov: self.camera.fov_degrees.to_radians(),
                        aspect: aspect,
                        rs: self.rs as f32,
                        disk_inner: (self.rs as f32) * 3.0,  
                        disk_outer: (self.rs as f32) * 6.0,
                        view_proj: view_proj.to_cols_array_2d(),
                    },
                    &quad_params,
                ).unwrap();
            }


                // Draw 3D Flamm's Paraboloid Grid
                if self.show_grid{
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