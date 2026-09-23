use glam::{Mat4, Vec3};
use glam::camera::rh::{view::look_at_mat4, proj::directx::perspective};
use std::f32::consts::PI;

/// Orbit camera: always looks at `target`, position derived from
/// spherical coordinates (radius, azimuth, elevation) around it.
/// Same model as the C++ reference's `Camera` struct.
pub struct Camera {
    pub target: Vec3,
    pub radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,

    pub azimuth: f32,   // rotation around the y axis
    pub elevation: f32, // angle from the y axis (poles at 0 / PI)

    pub orbit_speed: f32,
    pub zoom_speed: f32,

    dragging: bool,
    last_x: f64,
    last_y: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            target: Vec3::ZERO,
            radius: 10.0,
            min_radius: 2.0,
            max_radius: 50.0,
            azimuth: 0.0,
            elevation: PI / 2.0,
            orbit_speed: 0.005,
            zoom_speed: 0.5,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
        }
    }
}

impl Camera {
    /// World-space position, from spherical coordinates around `target`.
    pub fn position(&self) -> Vec3 {
        let e = self.elevation.clamp(0.01, PI - 0.01);
        self.target
            + Vec3::new(
                self.radius * e.sin() * self.azimuth.cos(),
                self.radius * e.cos(),
                self.radius * e.sin() * self.azimuth.sin(),
            )
    }

    /// Orthonormal basis: forward, right, up — needed for both the
    /// view matrix and, later, per-pixel ray directions in the shader.
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.target - self.position()).normalize();
        let world_up = Vec3::Y;
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward);
        (forward, right, up)
    }

    pub fn view_matrix(&self) -> Mat4 {
        look_at_mat4(self.position(), self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32, fov_y_radians: f32) -> Mat4 {
        perspective(fov_y_radians, aspect, 0.1, 1000.0)
    }

    // --- input handling --- //

    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.dragging = true;
        self.last_x = x;
        self.last_y = y;
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn mouse_moved(&mut self, x: f64, y: f64) {
        if self.dragging {
            let dx = (x - self.last_x) as f32;
            let dy = (y - self.last_y) as f32;

            self.azimuth += dx * self.orbit_speed;
            self.elevation -= dy * self.orbit_speed;
            self.elevation = self.elevation.clamp(0.01, PI - 0.01);
        }
        self.last_x = x;
        self.last_y = y;
    }

    pub fn scroll(&mut self, y_offset: f32) {
        self.radius -= y_offset * self.zoom_speed;
        self.radius = self.radius.clamp(self.min_radius, self.max_radius);
    }
}