use crate::error::ImageError;

#[derive(Debug)]
struct JpegChunk {
    indicator: u8,
    length: u16,
    data: Option<Vec<u8>>,
}

impl JpegChunk {
    fn new(bytes: &[u8]) -> Result<JpegChunk, ImageError> {
        if bytes.len() < 4 {
            return Err(ImageError::CustomError(format!(
                "invalid chunk length: {}",
                bytes.len()
            )));
        }
        let indicator: u8 = bytes[1];
        let length = match indicator {
            0xD0..=0xD9 => {
                return Ok(JpegChunk {
                    indicator: indicator,
                    length: 0 as u16,
                    data: None,
                });
            }
            0xC0 | 0xC2 | 0xC4 | 0xDB | 0xDA | 0xFE | 0xE0..=0xEF => {
                u16::from_be_bytes(bytes[2..4].try_into()?)
            }
            0xDD => 4 as u16,
            _ => {
                return Err(ImageError::CustomError(format!(
                    "invalid indicator: {:#X}",
                    indicator
                )));
            }
        };
        Ok(JpegChunk {
            indicator: indicator,
            length: length,
            data: Some(bytes[4..2 + length as usize].to_vec()),
        })
    }
}

#[derive(Debug)]
struct JpegImageChunks {
    image: Vec<JpegChunk>,
}

impl JpegImageChunks {
    pub fn new(bytes: &[u8]) -> Result<JpegImageChunks, ImageError> {
        let mut chunks: Vec<JpegChunk> = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let chunk = JpegChunk::new(&bytes[i..])?;
            i += 2 + chunk.length as usize;
            if chunk.indicator == 0xDA {
                while i < bytes.len() - 1 {
                    if bytes[i] == 0xFF
                        && bytes[i + 1] != 0x00
                        && !(0xD0..=0xD7).contains(&bytes[i + 1])
                    {
                        break;
                    }
                    i += 1;
                }
            }
            chunks.push(chunk);
        }
        Ok(JpegImageChunks { image: chunks })
    }
}

pub fn build_jpeg(chunks: JpegImageChunks) -> Result<Vec<[u16; 4]>, ImageError> {
    let pixels: Vec<[u16; 4]> = Vec::new();
    let frame_start1 = chunks.image.iter().find(|c| c.indicator == 0xC00);
    let frame_start2 = chunks.image.iter().find(|c| c.indicator == 0xC02);

    Ok(pixels)
}
