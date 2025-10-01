mod audio_utils;
mod image_utils;

use anyhow::{Context, Result, bail};

use rustfft::{FftPlanner, num_complex::Complex};
use std::{env, path::PathBuf};

use audio_utils::samples_from_file;
use image_utils::save_rgb_image;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Config {
    input_path: PathBuf,
    output_path: PathBuf,
    sample_rate: usize,
    downsample: usize,
    jobs: usize,
}

fn parse_args_from<I>(args: I) -> Result<Config>
where
    I: IntoIterator<Item = String>,
{
    let mut input_path: Option<PathBuf> = None;
    let mut output_path = PathBuf::from("./a.png");
    let mut sample_rate = 44_100usize;
    let mut downsample = 128usize;
    let mut jobs = 16usize;

    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--input" => {
                let value = args
                    .next()
                    .context("missing value for --input / -i option")?;
                input_path = Some(PathBuf::from(value));
            }
            "-o" | "--output" => {
                let value = args
                    .next()
                    .context("missing value for --output / -o option")?;
                output_path = PathBuf::from(value);
            }
            "-r" | "--sample-rate" => {
                let value = args
                    .next()
                    .context("missing value for --sample-rate / -r option")?;
                sample_rate = value
                    .parse()
                    .context("--sample-rate / -r expects a positive integer")?;
            }
            "-d" | "--downsample" => {
                let value = args
                    .next()
                    .context("missing value for --downsample / -d option")?;
                downsample = value
                    .parse()
                    .context("--downsample / -d expects a positive integer")?;
            }
            "-j" | "--jobs" => {
                let value = args
                    .next()
                    .context("missing value for --jobs / -j option")?;
                jobs = value
                    .parse()
                    .context("--jobs / -j expects a positive integer")?;
            }
            _other => {}
        }
    }

    let input_path = input_path.context("input path is required (pass with --input)")?;

    if sample_rate == 0 {
        bail!("--sample-rate must be greater than zero");
    }

    if downsample == 0 {
        bail!("--downsample must be greater than zero");
    }

    if jobs == 0 {
        bail!("--jobs must be greater than zero");
    }

    Ok(Config {
        input_path,
        output_path,
        sample_rate,
        downsample,
        jobs,
    })
}

fn main() -> Result<()> {
    let cfg = parse_args_from(env::args().skip(1)).unwrap();

    let samples = samples_from_file(&cfg.input_path, cfg.sample_rate, cfg.jobs)?;

    if samples.len() < cfg.sample_rate {
        bail!(
            "input does not contain enough samples ({}) for {} Hz",
            samples.len(),
            cfg.sample_rate
        );
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(cfg.sample_rate);

    let mut freq_buffer = vec![Complex::default(); cfg.sample_rate];

    let mut seconds: Vec<Vec<f32>> = Vec::with_capacity(samples.len() / cfg.sample_rate);

    for second in samples.chunks(cfg.sample_rate) {
        if second.len() < cfg.sample_rate {
            break;
        }

        freq_buffer
            .iter_mut()
            .zip(second.iter())
            .for_each(|(slot, &sample)| {
                slot.re = sample;
                slot.im = 0.0;
            });

        fft.process(&mut freq_buffer);

        let mags: Vec<f32> = freq_buffer.iter().map(|c| c.norm()).collect();

        seconds.push(mags);
    }

    if seconds.is_empty() {
        bail!("no complete seconds of audio could be processed");
    }

    let max = seconds.iter().flatten().copied().fold(0.0f32, f32::max);

    if max <= f32::EPSILON {
        bail!("audio signal has insufficient energy to create an image");
    }

    for freqs in seconds.iter_mut() {
        freqs.iter_mut().for_each(|value| *value /= max);
    }

    let width = ((seconds[0].len() + (cfg.downsample - 1)) / cfg.downsample) as u32;
    let height = seconds.len() as u32;

    let pixels: Vec<u8> = {
        let mut pix_buff = Vec::with_capacity(width as usize * height as usize * 3);

        for freqs in seconds.iter() {
            for chunk in freqs.chunks(cfg.downsample) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_input() {
        let result = parse_args_from(Vec::<String>::new());
        assert!(result.is_err());
    }

    #[test]
    fn parse_args_rejects_zero_downsample() {
        let args = vec![
            "--input".into(),
            "input.wav".into(),
            "--downsample".into(),
            "0".into(),
        ];
        let err = parse_args_from(args).expect_err("expected failure for zero downsample");
        assert!(err.to_string().contains("--downsample"));
    }

    #[test]
    fn parse_args_accepts_minimal_valid_input() {
        let args = vec!["--input".into(), "input.wav".into()];
        let cfg = parse_args_from(args).expect("valid arguments");
        assert_eq!(cfg.input_path, PathBuf::from("input.wav"));
        assert_eq!(cfg.downsample, 128);
    }
}
