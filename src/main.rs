use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{Complex, Normed};
use std::thread;
use std::time::Instant;

const MAX_ITER: i32 = 36;
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCALE: f64 = 3.0;

const ASPECT: f64 = WIDTH as f64 / HEIGHT as f64;

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
    let mut img = ImageBuffer::new(WIDTH, HEIGHT);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = (x as f64 / WIDTH as f64 - 0.5) * SCALE;
        let cy = (y as f64 / HEIGHT as f64 - 0.5) * SCALE / ASPECT;

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

fn generate_frames(frames: Vec<i32>, pb: &ProgressBar) {
    for max_iter in frames {
        generate_frame(max_iter);
        pb.inc(1);
    }
}

fn main() {
    let now = Instant::now();
    let m = MultiProgress::new();
    let frames = 0..MAX_ITER;
    let num_threads = thread::available_parallelism().unwrap().get() as i32 - 1;
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];
    for ti in 0..num_threads {
        let group: Vec<i32> = frames
            .clone()
            .filter(move |i| i % num_threads == ti)
            .collect();
        let pb = m.add(ProgressBar::new(group.len() as u64));
        let t = thread::spawn(move || generate_frames(group, &pb));
        threads.push(t);
    }
    for t in threads {
        t.join().unwrap();
    }
    println!("Elapsed: {:.2?}", now.elapsed());
}
