mod audio_utils;
mod image_utils;

use anyhow::*;

use rustfft::{FftPlanner, num_complex::Complex};
use std::{path::Path, vec};

use audio_utils::samples_from_file;
use image_utils::save_rgb_image;

fn main() -> anyhow::Result<()> {
    let path = Path::new("./audio/Mil_Horas.mp3");

    let sample_rate = 44100;

    let samples = samples_from_file(&path, sample_rate, 16).unwrap();

    //dbg!(samples.len() as f32 / sample_rate as f32);

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(sample_rate);

    let mut buff = vec![Complex::default(); fft.get_inplace_scratch_len()];

    let mut fft_vec: Vec<Vec<f32>> = Vec::new();

    for second in samples.chunks(sample_rate) {
        if second.len() < sample_rate {
            break;
        }

        for (i, &x) in second.iter().enumerate() {
            buff[i].re = x;
            buff[i].im = 0.0;
        }

        fft.process(&mut buff);

        // Magnitudes normalizadas
        let magnitudes: Vec<f32> = buff.iter().map(|c| c.norm() / sample_rate as f32).collect();

        fft_vec.push(magnitudes.clone());

        // dbg!(magnitudes.len());

        // dbg!(buff.len());

        // break;
    }

    dbg!(fft_vec.len());

    // let image_path = Path::new("./test.png");
    // _ = save_rgb_image(image_path, fft_vec.len(), sample_rate, &f.unwrap();

    Ok(())
}
