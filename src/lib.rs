mod error;
mod image_types;

use crate::error::ImageError;
use std::fs::File;
use std::io::Read;

pub fn read_image(image: &str) -> Result<(), ImageError> {
    let mut file = File::open(image)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
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
