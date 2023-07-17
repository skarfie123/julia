use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{Complex, Normed};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

const MAX_ITER: i32 = 2400;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const ASPECT: f64 = WIDTH as f64 / HEIGHT as f64;
const SCALE: f64 = 3.0;

const C: Complex<f64> = Complex::new(-0.8, 0.156);

type Cache = HashMap<(u32, u32), i32>;

const TIMINGS_FILE: &str = "timings.csv";
type Timings = Vec<(i32, f32)>;

fn julia(c: Complex<f64>, x: u32, y: u32, max_iter: i32, cache: &mut Cache) -> f64 {
    if let Some(&i) = cache.get(&(x, y)) {
        // make sure it's not from a future frame
        if i < max_iter {
            return i as f64 / max_iter as f64;
        }
    }
    let scaled_x = (x as f64 / WIDTH as f64 - 0.5) * SCALE;
    let scaled_y = (y as f64 / HEIGHT as f64 - 0.5) * SCALE / ASPECT;
    let mut z = Complex::new(scaled_x, scaled_y);

    for i in 0..max_iter {
        if z.norm() > 2.0 {
            cache.insert((x, y), i);
            return i as f64 / max_iter as f64;
        }

        z = z * z + c;
    }

    -1.0
}

fn generate_frame(max_iter: i32, cache: &mut Cache) {
    let mut img = ImageBuffer::new(WIDTH, HEIGHT);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let value = julia(C, x, y, max_iter, cache);

        if value == -1.0 {
            *pixel = Rgb([0, 0, 0]);
            continue;
        }

        let (r, g, b) = hsv_to_rgb(360.0 * value, 1.0, value.powf(0.25));

        *pixel = Rgb([r, g, b]);
    }

    img.save(format!("julia\\{}.png", max_iter)).unwrap();
}

fn generate_frames(frames: Vec<i32>, pb: &ProgressBar) -> Timings {
    let mut cache: Cache = HashMap::new();
    let mut timings: Timings = vec![];
    for max_iter in frames {
        let now = Instant::now();
        generate_frame(max_iter, &mut cache);
        timings.push((max_iter, now.elapsed().as_secs_f32()));
        pb.inc(1);
    }
    timings
}

fn main() {
    let now = Instant::now();

    let m = MultiProgress::new();

    let frames = 0..MAX_ITER;

    let num_threads = thread::available_parallelism().unwrap().get() as i32 - 1;
    let mut threads: Vec<thread::JoinHandle<Timings>> = vec![];

    for ti in 0..num_threads {
        let group: Vec<i32> = frames
            .clone()
            .filter(move |i| i % num_threads == ti)
            .collect();

        let pb = m.add(ProgressBar::new(group.len() as u64));

        let t = thread::spawn(move || generate_frames(group, &pb));
        threads.push(t);
    }

    let mut file = File::create(TIMINGS_FILE).unwrap();
    match file.write(b"frame, time") {
        Ok(_) => (),
        Err(e) => eprint!("Error Occurred: {}", e),
    }
    for t in threads {
        let timings = t.join().unwrap();
        for (frame, timing) in timings {
            match file.write(format!("\n{}, {}", frame, timing).as_bytes()) {
                Ok(_) => (),
                Err(e) => eprint!("Error Occurred: {}", e),
            }
        }
    }

    println!("Elapsed: {:.2?}", now.elapsed());
}
