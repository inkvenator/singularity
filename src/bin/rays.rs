use std::fs::File;
use std::io::{BufWriter, Write};

fn derivative(u: f64, v: f64, rs: f64) -> (f64, f64) {
    (v, 1.5 * rs * u * u - u)
}

fn rk4_step(u: f64, v: f64, h: f64, rs: f64) -> (f64, f64) {
    let (k1u, k1v) = derivative(u, v, rs);
    let (k2u, k2v) = derivative(u + 0.5 * h * k1u, v + 0.5 * h * k1v, rs);
    let (k3u, k3v) = derivative(u + 0.5 * h * k2u, v + 0.5 * h * k2v, rs);
    let (k4u, k4v) = derivative(u + h * k3u, v + h * k3v, rs);

    let u_next = u + (h / 6.0) * (k1u + 2.0 * k2u + 2.0 * k3u + k4u);
    let v_next = v + (h / 6.0) * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
    (u_next, v_next)
}

fn main() {
    let rs = 1.0_f64;      // set to 0.0 to run the flat-space test
    let b = 5.0_f64;       // impact parameter
    let r0 = 50.0_f64;     // starting distance
    let dphi = 0.001_f64;  // step size in phi
    let max_steps = 200_000;

    let mut u = 1.0 / r0;
    let mut phi = (b * u).asin();
    let mut v = phi.cos() / b;

    let mut points: Vec<(f64, f64)> = Vec::new();

    for _ in 0..max_steps {
        let r = 1.0 / u;
        points.push((r * phi.cos(), r * phi.sin()));

        if r <= rs {
            println!("captured at r = {:.4}, phi = {:.4}", r, phi);
            break;
        }
        if r > r0 * 2.0 {
            println!("escaped at r = {:.4}, phi = {:.4}", r, phi);
            break;
        }

        let (u_next, v_next) = rk4_step(u, v, dphi, rs);
        u = u_next;
        v = v_next;
        phi += dphi;
    }

    let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    println!("y ranged {:.6} to {:.6} (should ≈ b = {} when rs = 0.0)", y_min, y_max, b);

    let file = File::create("rays.csv").expect("could not create file");
    let mut writer = BufWriter::new(file);
    for (x, y) in &points {
        writeln!(writer, "{},{}", x, y).unwrap();
    }
    println!("wrote {} points to rays.csv", points.len());
}