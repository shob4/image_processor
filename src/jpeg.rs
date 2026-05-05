use crate::error::ImageError;

#[derive(Debug)]
struct JpegChunk {
    indicator: u8,
    length: u16,
    data: Vec<u8>,
}

impl JpegChunk {
    fn new(bytes: &[u8]) -> Result<JpegChunk, ImageError> {
        let indicator: u8 = bytes[1];
        let length = match indicator {
            0xD0..=0xD9 => 0 as u16,
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
            data: bytes[4..2 + length as usize].to_vec(),
        })
    }
}

#[derive(Debug)]
struct JpegImageChunks {
    image: Vec<JpegChunk>,
}

impl JpegImageChunks {
    fn new(bytes: &[u8]) -> Result<JpegImageChunks, ImageError> {
        let mut chunks: Vec<JpegChunk> = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let chunk = JpegChunk::new(&bytes[i..])?;
            i += 2 + chunk.length as usize;
            chunks.push(chunk);
        }
        Ok(JpegImageChunks { image: chunks })
    }
}
