use anyhow::*;

use core::f32;
use std::{
    io::Read,
    path::{self, Path},
    process::{Command, Stdio},
};

fn main() -> anyhow::Result<()> {
    let path = Path::new("./audio/Mil_Horas.mp3");
    let _samples = samples_from_file(&path, 44100, 16);

    Ok(())
}

fn samples_from_file(path: &Path, sample_rate: u32, threads: u32) -> Result<Vec<f32>> {
    dbg!(path, sample_rate, threads);

    let mut buff = Vec::new();

    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-i",
            path.to_str().unwrap(),
            "-ac",
            "1",
            "-ar",
            &sample_rate.to_string(),
            "-f",
            "f32le",
            "-threads",
            &threads.to_string(),
            "-", // salida cruda por stdout
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let buff_len = child.stdout.take().unwrap().read_to_end(&mut buff)?;
    let status = child.wait()?;

    dbg!(buff_len);

    if !status.success() {
        bail!("ffmpeg fail {}", status.to_string());
    }

    if buff_len % 4 != 0 {
        bail!("bytes len error {}", buff_len);
    }

    let mut output = Vec::with_capacity(buff_len / 4);

    for chunk in buff.chunks_exact(4) {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        output.push(value);
    }

    Ok(output)
}
