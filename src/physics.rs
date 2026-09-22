use glam::DVec2;

fn geodesic_rhs(r: f64, dr: f64, dphi: f64, e: f64, rs: f64) -> (f64, f64, f64, f64) {
    let f = 1.0 - rs / r;
    let dt = e / f;
    let d2r = -(rs / (2.0 * r * r)) * f * dt * dt
        + (rs / (2.0 * r * r * f)) * dr * dr
        + (r - rs) * dphi * dphi;
    let d2phi = -2.0 * dr * dphi / r;
    (dr, dphi, d2r, d2phi)
}

fn rk4_step(r: f64, phi: f64, dr: f64, dphi: f64, dl: f64, e: f64, rs: f64) -> (f64, f64, f64, f64) {
    let (k1r, k1p, k1dr, k1dp) = geodesic_rhs(r, dr, dphi, e, rs);
    let (k2r, k2p, k2dr, k2dp) = geodesic_rhs(
        r + 0.5 * dl * k1r, dr + 0.5 * dl * k1dr, dphi + 0.5 * dl * k1dp, e, rs,
    );
    let (k3r, k3p, k3dr, k3dp) = geodesic_rhs(
        r + 0.5 * dl * k2r, dr + 0.5 * dl * k2dr, dphi + 0.5 * dl * k2dp, e, rs,
    );
    let (k4r, k4p, k4dr, k4dp) = geodesic_rhs(
        r + dl * k3r, dr + dl * k3dr, dphi + dl * k3dp, e, rs,
    );
    (
        r + (dl / 6.0) * (k1r + 2.0 * k2r + 2.0 * k3r + k4r),
        phi + (dl / 6.0) * (k1p + 2.0 * k2p + 2.0 * k3p + k4p),
        dr + (dl / 6.0) * (k1dr + 2.0 * k2dr + 2.0 * k3dr + k4dr),
        dphi + (dl / 6.0) * (k1dp + 2.0 * k2dp + 2.0 * k3dp + k4dp),
    )
}

pub struct Ray {
    r: f64,
    phi: f64,
    dr: f64,
    dphi: f64,
    e: f64,
    pub trail: Vec<[f32; 2]>,
    pub alive: bool,
}

impl Ray {
    /// `pos`: starting position. `dir`: starting direction (need not be unit length).
    /// `rs`: Schwarzschild radius of the black hole this ray is near.
    pub fn new(pos: DVec2, dir: DVec2, rs: f64) -> Self {
        let r = pos.length();
        let phi = pos.y.atan2(pos.x);

        let dr = dir.x * phi.cos() + dir.y * phi.sin();
        let dphi = (-dir.x * phi.sin() + dir.y * phi.cos()) / r;

        let f0 = 1.0 - rs / r;
        let dt0 = ((dr * dr) / (f0 * f0) + (r * r * dphi * dphi) / f0).sqrt();
        let e = f0 * dt0;

        Ray {
            r,
            phi,
            dr,
            dphi,
            e,
            trail: vec![[pos.x as f32, pos.y as f32]],
            alive: true,
        }
    }

    /// Advance one RK4 step of size `dl` in the affine parameter.
    /// `r_max`: stop and mark not-alive once the ray has clearly escaped.
    pub fn step(&mut self, dl: f64, rs: f64, r_max: f64) {
        if !self.alive || self.r <= rs || self.r > r_max {
            self.alive = false;
            return;
        }
        let (r, phi, dr, dphi) = rk4_step(self.r, self.phi, self.dr, self.dphi, dl, self.e, rs);
        self.r = r;
        self.phi = phi;
        self.dr = dr;
        self.dphi = dphi;

        let x = self.r * self.phi.cos();
        let y = self.r * self.phi.sin();
        self.trail.push([x as f32, y as f32]);
    }
}