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
    let kind = infer::get(&buf).expect("file type is known");
    match kind.map(|t| t.mime_type()) {
        Some("image/png") => todo!("create png type"),
        Some("image/jpeg") => todo!("create jpeg type"),
        Some("image/bmp") => todo!("create bmp type"),
        Some(other) => Err(ImageError::CustomError("unsupported file type".to_string())),
        None => Err(ImageError::CustomError("unsupported file type".to_string())),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), ImageError> {
        read_image("kingInYellow.png")
    }
}
