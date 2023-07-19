use crossbeam_channel::{unbounded, Receiver, Sender};
use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgb};
use indicatif::{MultiProgress, ProgressBar};
use nalgebra::{Complex, DMatrix, Normed};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const MAX_ITER: i32 = 1000;

const WIDTH: u32 = 1920 / 4;
const HEIGHT: u32 = 1080 / 4;
const ASPECT: f64 = WIDTH as f64 / HEIGHT as f64;
const SCALE: f64 = 3.0;

const C: Complex<f64> = Complex::new(-0.8, 0.156);

type Julia = DMatrix<i32>;

const TIMINGS_FILE: &str = "timings.csv";
type Timings = Vec<(i32, f32)>;

fn julia(c: Complex<f64>, x: u32, y: u32) -> i32 {
    let scaled_x = (x as f64 / WIDTH as f64 - 0.5) * SCALE;
    let scaled_y = (y as f64 / HEIGHT as f64 - 0.5) * SCALE / ASPECT;
    let mut z = Complex::new(scaled_x, scaled_y);

    for i in 0..MAX_ITER {
        if z.norm() > 2.0 {
            return i;
        }

        z = z * z + c;
    }

    -1
}

fn generate_julias(m: &MultiProgress) -> Julia {
    let pb_x = m.add(ProgressBar::new(WIDTH as u64));
    let pb_y = m.add(ProgressBar::new(HEIGHT as u64));

    let mut data: Julia = DMatrix::<i32>::from_element(WIDTH as usize, HEIGHT as usize, -1);

    for x in 0..WIDTH {
        pb_y.set_position(0);
        for y in 0..HEIGHT {
            let value = julia(C, x, y);
            data[(x as usize, y as usize)] = value;
            pb_y.inc(1);
        }
        pb_x.inc(1);
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

    img.save(format!("julia\\{}.png", max_iter)).unwrap();
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

    let m = MultiProgress::new();

    let frames = 0..MAX_ITER;

    let data: Arc<Julia> = Arc::new(generate_julias(&m));

    let num_threads = thread::available_parallelism().unwrap().get() as i32 - 1;
    let mut threads: Vec<thread::JoinHandle<Timings>> = vec![];
    let pb = m.add(ProgressBar::new(MAX_ITER as u64));

    let (frame_sender, frame_receiver): (Sender<i32>, Receiver<i32>) = unbounded();

    for i in frames {
        frame_sender.send(i).unwrap();
    }

    for _ in 0..num_threads {
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

    println!("Elapsed: {:.2?}", now.elapsed());
}
