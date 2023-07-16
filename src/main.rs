use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use indicatif::ProgressIterator;
use nalgebra::{Complex, Normed};
use std::time::Instant;

const MAX_ITER: i32 = 16;

fn julia(c: Complex<f64>, x: f64, y: f64, max_iter: i32) -> f64 {
    let mut z = Complex::new(x, y);

    for i in 0..max_iter {
        if z.norm() > 2.0 {
            return i as f64 / max_iter as f64;
        }

        z = z * z + c;
    }

    -1.0
}

fn generate_frame(max_iter: i32) {
    let width = 1920 / 2;
    let height = 1080 / 2;
    let scale = 3.0;

    let aspect = width as f64 / height as f64;

    let mut img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = (x as f64 / width as f64 - 0.5) * scale;
        let cy = (y as f64 / height as f64 - 0.5) * scale / aspect;

        let c = Complex::new(-0.8, 0.156);
        let value = julia(c, cx, cy, max_iter);

        if value == -1.0 {
            *pixel = Rgb([0, 0, 0]);
            continue;
        }

        let (r, g, b) = hsv_to_rgb(360.0 * value, 1.0, value.powf(0.25));

        *pixel = Rgb([r, g, b]);
    }

    img.save(format!("julia\\{}.png", max_iter)).unwrap();
}

fn main() {
    let now = Instant::now();
    for i in (0..MAX_ITER).progress() {
        generate_frame(i);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
