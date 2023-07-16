use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use nalgebra::{Complex, Normed};
use std::time::Instant;

const MAX_ITER: i32 = 360;

fn julia(c: Complex<f64>, x: f64, y: f64) -> f64 {
    let mut z = Complex::new(x, y);

    for i in 0..MAX_ITER {
        if z.norm() > 2.0 {
            return i as f64 / MAX_ITER as f64;
        }

        z = z * z + c;
    }

    -1.0
}

fn main() {
    let now = Instant::now();
    let width = 1920;
    let height = 1080;
    let scale = 3.0;

    let aspect = width as f64 / height as f64;

    let mut img = ImageBuffer::new(width, height);

    let mut max = 0.0;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = (x as f64 / width as f64 - 0.5) * scale;
        let cy = (y as f64 / height as f64 - 0.5) * scale / aspect;

        let c = Complex::new(-0.8, 0.156);
        let value = julia(c, cx, cy);

        if value == -1.0 {
            *pixel = Rgb([0, 0, 0]);
            continue;
        }

        if value > max {
            max = value;
        }

        let (r, g, b) = hsv_to_rgb(360.0 * value, 1.0, value.powf(0.25));

        *pixel = Rgb([r, g, b]);
    }
    println!("Max: {}", max * MAX_ITER as f64);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let result = img.save("julia.png");
    match result {
        Ok(()) => (),
        Err(e) => eprintln!("{}", e),
    }
    let elapsed2 = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed2);
}
