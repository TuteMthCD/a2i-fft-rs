use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Result, bail};

/// Extracts mono 32-bit float samples from an audio file using ffmpeg.
///
/// * `path` - audio source file.
/// * `sample_rate` - target sample rate in Hz.
/// * `threads` - ffmpeg thread count.
pub fn samples_from_file(path: &Path, sample_rate: usize, threads: usize) -> Result<Vec<f32>> {
    let mut buff = Vec::new();

    if !path.exists() {
        bail!("error path not exists {}", path.display());
    }

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
            "-", // raw output via stdout
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let buff_len = child.stdout.take().unwrap().read_to_end(&mut buff)?;
    let status = child.wait()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_from_file_returns_error_for_missing_source() {
        let missing_path = Path::new("./audio/non_existent_file.mp3");
        let result = samples_from_file(missing_path, 8_000, 1);
        assert!(result.is_err());
    }
}
