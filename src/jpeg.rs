use crate::error::ImageError;

#[derive(Debug)]
struct JpegChunk {
    indicater: u8,
    length: Option<u16>,
    data: Vec<u8>,
}

impl JpegChunk {
    fn new(bytes: &[u8]) -> Result<JpegChunk, ImageError> {
        let indicator: u8 = bytes[1];
        let new_chunk = JpegChunk {
            length: Some(length),
        }
    }
}

#[derive(Debug)]
struct JpegImageChunks {
    image: Vec<JpegChunk>,
}
