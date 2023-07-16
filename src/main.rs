use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use nalgebra::{Complex, Normed};

const MAX_ITER: i32 = 360;

fn julia(c: Complex<f64>, x: f64, y: f64) -> i32 {
    let mut z = Complex::new(x, y);

    for i in 0..MAX_ITER {
        if z.norm() > 2.0 {
            return i;
        }

        z = z * z + c;
    }

    -1
}

fn main() {
    let width = 1980;
    let height = 1080;

    let scale_x = 3.0 / width as f64;
    let scale_y = 2.0 / height as f64;

    let mut img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = x as f64 * scale_x - 1.5;
        let cy = y as f64 * scale_y - 1.0;

        let c = Complex::new(-0.8, 0.156);
        let value = julia(c, cx, cy);

        if value == -1 {
            *pixel = Rgb([255, 255, 255]);
            continue;
        }

        let (r, g, b) = hsv_to_rgb(
            360.0 * value as f64 / MAX_ITER as f64,
            1.0,
            (value as f64 / MAX_ITER as f64).sqrt(),
        );

        *pixel = Rgb([r, g, b]);
    }

    let result = img.save("julia.png");
    match result {
        Ok(()) => (),
        Err(e) => eprintln!("{}", e),
    }
}
