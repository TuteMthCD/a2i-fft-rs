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

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(sample_rate);

    let mut buff = vec![Complex::default(); fft.get_inplace_scratch_len()];

    let mut seconds: Vec<Vec<f32>> = Vec::with_capacity(samples.len() / sample_rate);

    for second in samples.chunks(sample_rate) {
        if second.len() < sample_rate {
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

    let downsample = 128; // reduce horizontal resolution for easier viewing

    let width = ((seconds[0].len() + (downsample - 1)) / downsample) as u32;
    let height = seconds.len() as u32;

    let pixels: Vec<u8> = {
        let mut pix_buff = Vec::with_capacity(width as usize * height as usize * 3);

        for feqs in seconds.iter() {
            for chunk in feqs.chunks(downsample) {
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

    let image_path = Path::new("./test.png");
    _ = save_rgb_image(image_path, width, height, pixels.as_slice()).unwrap();
    Ok(())
}
