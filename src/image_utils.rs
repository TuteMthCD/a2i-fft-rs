use std::path::Path;

use anyhow::{Context, Result, ensure};
use image::RgbImage;

/// Saves an RGB image to disk from a flat pixel buffer (RGB, row-major).
///
/// * `path` - destination file path.
/// * `width`, `height` - image dimensions in pixels.
/// * `pixels` - flat buffer in RGB order, expected length = width * height * 3.
pub fn save_rgb_image<P>(path: P, width: u32, height: u32, pixels: &[u8]) -> Result<()>
where
    P: AsRef<Path>,
{
    let expected_len = width as usize * height as usize * 3;
    ensure!(
        pixels.len() == expected_len,
        "pixel buffer length mismatch: expected {} bytes ({}x{}x3), got {}",
        expected_len,
        width,
        height,
        pixels.len()
    );

    // The image crate takes ownership of the buffer, so clone is required when starting from a slice.
    let image = RgbImage::from_vec(width, height, pixels.to_vec())
        .context("failed to build image from RGB pixels")?;

    image
        .save(path)
        .context("failed to persist RGB image to disk")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use tempfile::tempdir;

    #[test]
    fn save_rgb_image_creates_png_with_expected_pixels() {
        let dir = tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.png");

        // 2x2 image: red, green, blue, white
        let pixels = [
            255, 0, 0, // (0,0)
            0, 255, 0, // (1,0)
            0, 0, 255, // (0,1)
            255, 255, 255, // (1,1)
        ];

        save_rgb_image(&file_path, 2, 2, &pixels).expect("save image");

        let image = image::open(&file_path).expect("reopen image");
        assert_eq!(image.dimensions(), (2, 2));

        let rgb_image = image.to_rgb8();
        assert_eq!(rgb_image.into_vec(), pixels.to_vec());
    }

    #[test]
    fn save_rgb_image_rejects_wrong_buffer_length() {
        let dir = tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.png");

        let pixels = [255, 0, 0];
        let err = save_rgb_image(&file_path, 2, 2, &pixels).expect_err("expected length mismatch");
        assert!(err.to_string().contains("pixel buffer length mismatch"));
    }
}
