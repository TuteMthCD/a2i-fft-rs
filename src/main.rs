mod audio_utils;
mod image_utils;

use anyhow::*;

use rustfft::{FftPlanner, num_complex::Complex};
use std::{env, path::PathBuf, vec};

use audio_utils::samples_from_file;
use image_utils::save_rgb_image;

struct Config {
    input_path: Option<PathBuf>,
    output_path: PathBuf,
    sample_rate: usize,
    downsample: usize,
    jobs: usize,
}

fn parse_args() -> Config {
    let mut cfg = Config {
        input_path: None,
        output_path: PathBuf::from("./a.png"),
        sample_rate: 44_100,
        downsample: 128,
        jobs: 16,
    };

    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--input" => {
                let value = args.next().unwrap();
                cfg.input_path = Some(PathBuf::from(value));
            }
            "-o" | "--output" => {
                let value = args.next().unwrap();
                cfg.output_path = PathBuf::from(value);
            }
            "-r" | "--sample-rate" => {
                let value = args.next().unwrap();
                cfg.sample_rate = value.parse().unwrap();
            }
            "-d" | "--downsample" => {
                let value = args.next().unwrap();
                cfg.downsample = value.parse().unwrap();
            }
            "-j" | "--jobs" => {
                let value = args.next().unwrap();
                cfg.jobs = value.parse().unwrap();
            }
            _other => {}
        }
    }

    if cfg.input_path.is_some() {
        return cfg;
    } else {
        println!("Need input path");
        std::process::exit(1);
    }
}

fn main() -> anyhow::Result<()> {
    let cfg = parse_args();

    let samples = samples_from_file(&cfg.input_path.unwrap(), cfg.sample_rate, cfg.jobs)?;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(cfg.sample_rate);

    let mut buff = vec![Complex::default(); fft.get_inplace_scratch_len()];

    let mut seconds: Vec<Vec<f32>> = Vec::with_capacity(samples.len() / cfg.sample_rate);

    for second in samples.chunks(cfg.sample_rate) {
        if second.len() < cfg.sample_rate {
            break;
        }

        for (i, &x) in second.iter().enumerate() {
            buff[i].re = x;
            buff[i].im = 0.0;
        }

        fft.process(&mut buff);

        // let mags: Vec<f32> = buff.iter().map(|c| c.norm() / sample_rate as f32).collect();
        let mags: Vec<f32> = buff.iter().map(|c| c.norm() as f32).collect();

        seconds.push(mags);
    }

    //normalize
    let max: f32 = {
        let mut max: f32 = 0.0;
        for freqs in seconds.iter() {
            for values in freqs.iter() {
                if *values > max {
                    max = *values;
                }
            }
        }
        max
    };

    for freqs in seconds.iter_mut() {
        freqs.iter_mut().for_each(|c| *c /= max);
    }

    let width = ((seconds[0].len() + (cfg.downsample - 1)) / cfg.downsample) as u32;
    let height = seconds.len() as u32;

    let pixels: Vec<u8> = {
        let mut pix_buff = Vec::with_capacity(width as usize * height as usize * 3);

        for feqs in seconds.iter() {
            for chunk in feqs.chunks(cfg.downsample) {
                let avg = chunk.iter().copied().sum::<f32>() / chunk.len() as f32;
                let mapped = avg
                    .clamp(0.0, 1.0)
                    .powf(0.2) // gamma curve to boost quieter bins
                    * 255.0;
                pix_buff.extend_from_slice(&[mapped as u8, mapped as u8, 0]);
            }
        }

        pix_buff
    };

    save_rgb_image(&cfg.output_path, width, height, pixels.as_slice())?;
    Ok(())
}
