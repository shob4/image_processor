mod bmp;
mod error;

use crate::error::ImageError;
use std::fs::File;
use std::io::Read;

pub fn read_image(image: &str) -> Result<(), ImageError> {
    let mut file = File::open(image)?;
    let mut buffer = [0u8; 36];
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read < 36 {
        return Err(ImageError::CustomError("file too small".to_string()));
    }
    let buf = &buffer[..bytes_read];
    match infer::get(&buf).map(|t| t.mime_type()) {
        Some("image/png") => Ok(()),
        Some("image/jpeg") => Ok(()),
        Some("image/bmg") => {
            let mut buffer = Vec::new();
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read < 54 {
                return Err(ImageError::CustomError("no image".to_string()));
            }
            let image = bmp::BmpImage::new();
            Ok(())
        }
        Some(other) => Err(ImageError::CustomError(format!(
            "unsupported file type: {0}",
            other
        ))),
        None => Err(ImageError::CustomError("unsupported file type".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), ImageError> {
        read_image("kingInYellow.png")
    }
}
