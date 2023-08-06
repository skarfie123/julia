use crossbeam_channel::{unbounded, Receiver, Sender};
use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{Complex, DMatrix, Normed};
use std::f64::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const MAX_ITER: i32 = 2000;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const ASPECT: f64 = WIDTH as f64 / HEIGHT as f64;
const SCALE: f64 = PI;

const FOLDER: &str = "julia";
const EXTENSION: &str = "bmp";
const FINAL_FRAME_ONLY: bool = true;

const C: Complex<f64> = Complex::new(-0.8, 0.156);
const THRESHOLD: f64 = 2.0;

type Julia = DMatrix<i32>;

const TIMINGS_FILE: &str = "timings.csv";
type Timings = Vec<(i32, f32)>;

fn num_threads() -> usize {
    thread::available_parallelism().unwrap().get() - 1
}

fn julia(x: u32, y: u32) -> i32 {
    let scaled_x = (x as f64 / WIDTH as f64 - 0.5) * SCALE;
    let scaled_y = (y as f64 / HEIGHT as f64 - 0.5) * SCALE / ASPECT;
    let mut z = Complex::new(scaled_x, scaled_y);

    for i in 0..MAX_ITER {
        if z.norm() > THRESHOLD {
            return i;
        }

        z = z * z + C;
    }

    -1
}

fn mandelbrot(x: u32, y: u32) -> i32 {
    let scaled_x = (x as f64 / WIDTH as f64 - 0.6) * SCALE;
    let scaled_y = (y as f64 / HEIGHT as f64 - 0.5) * SCALE / ASPECT;
    let c = Complex::new(scaled_x, scaled_y);
    let mut z = Complex::new(0.0, 0.0);

    for i in 0..MAX_ITER {
        if z.norm() > THRESHOLD {
            return i;
        }

        z = z * z + c;
    }

    -1
}

fn generate_julia(m: &MultiProgress) -> Julia {
    type Pixels = (u32, u32);
    type PixelResults = Vec<(u32, u32, i32)>;

    let mut threads: Vec<thread::JoinHandle<PixelResults>> = vec![];
    let pb = m.add(ProgressBar::new((WIDTH * HEIGHT) as u64)); // # TODO

    let (pixel_sender, pixel_receiver): (Sender<Pixels>, Receiver<Pixels>) = unbounded();

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            pixel_sender.send((x, y)).unwrap();
        }
    }

    let mut data: Julia = DMatrix::<i32>::from_element(WIDTH as usize, HEIGHT as usize, -1);

    for _ in 0..num_threads() {
        let pixel_receiver = pixel_receiver.clone();
        let pb = pb.clone();

        threads.push(thread::spawn(move || {
            let mut pixels: PixelResults = vec![];
            for (x, y) in pixel_receiver {
                pixels.push((x, y, julia(x, y)));
                pb.inc(1);
            }
            pixels
        }));
    }
    drop(pixel_sender);

    for t in threads {
        let pixels: PixelResults = t.join().unwrap();
        for (x, y, value) in pixels {
            data[(x as usize, y as usize)] = value;
        }
    }

    data
}

fn generate_frame(max_iter: i32, data: &Julia) {
    let mut img = ImageBuffer::new(WIDTH, HEIGHT);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let value = data[(x as usize, y as usize)];

        if value == -1 || value > max_iter {
            *pixel = Rgb([0, 0, 0]);
            continue;
        }

        let normalised_value: f64 = value as f64 / max_iter as f64;
        let (r, g, b) = hsv_to_rgb(360.0 * normalised_value, 1.0, normalised_value.powf(0.25));

        *pixel = Rgb([r, g, b]);
    }

    img.save(format!("{FOLDER}\\{}.{EXTENSION}", max_iter))
        .unwrap();
}

fn generate_frames(frames: Receiver<i32>, pb: &ProgressBar, data: &Julia) -> Timings {
    let mut timings: Timings = vec![];
    for max_iter in frames {
        let now = Instant::now();
        generate_frame(max_iter, data);
        timings.push((max_iter, now.elapsed().as_secs_f32()));
        pb.inc(1);
    }
    timings
}

fn main() {
    let now = Instant::now();

    create_dir_all(FOLDER).unwrap();

    let m = MultiProgress::new();

    let data: Arc<Julia> = Arc::new(generate_julia(&m));

    println!("Elapsed: {:.2?}", now.elapsed());

    let frames = if FINAL_FRAME_ONLY {
        data.max()..data.max() + 1
    } else {
        0..data.max() + 1
    };

    let mut threads: Vec<thread::JoinHandle<Timings>> = vec![];
    let pb = m.add(ProgressBar::new(data.max() as u64));

    let (frame_sender, frame_receiver): (Sender<i32>, Receiver<i32>) = unbounded();

    for i in frames {
        frame_sender.send(i).unwrap();
    }

    for _ in 0..num_threads() {
        let frame_receiver = frame_receiver.clone();
        let pb = pb.clone();
        let data = data.clone();

        threads.push(thread::spawn(move || {
            generate_frames(frame_receiver, &pb, &data)
        }));
    }
    drop(frame_sender);

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
    pb.finish_and_clear();

    println!("Total Elapsed: {:.2?}", now.elapsed());
}
