use crate::error::ImageError;

#[derive(Debug)]
struct Component {
    id: u8,
    sampling_factors: u8,
    quantization_id: u8,
}

impl Component {
    fn new(bytes: &[u8]) -> Component {
        Component {
            id: bytes[0],
            sampling_factors: bytes[1],
            quantization_id: bytes[2],
        }
    }

    fn new_components(bytes: &[u8], count: u8) -> Result<Vec<Component>, ImageError> {
        if bytes.len() / count as usize != 3 {
            return Err(ImageError::CustomError(format!(
                "{0} / {1} does not equal 3",
                bytes.len(),
                count
            )));
        }
        let mut components: Vec<Component> = Vec::new();
        for i in 0..count {
            let new_component = Component::new(&bytes[(i * 3) as usize..(i * 3 + 3) as usize]);
            components.push(new_component);
        }
        Ok(components)
    }
}

#[derive(Debug)]
struct JpegHeader {
    precision: u8,
    height: u16,
    width: u16,
    component_count: u8,
    components: Vec<Component>,
}

impl JpegHeader {
    fn new(chunk: JpegChunk) -> Result<JpegHeader, ImageError> {
        let data = match chunk.data {
            Some(data) => data,
            None => {
                return Err(ImageError::CustomError(
                    "no data in jpeg header segment".to_string(),
                ));
            }
        };
        Ok(JpegHeader {
            precision: data[0],
            height: u16::from_be_bytes(data[1..2].try_into()?),
            width: u16::from_be_bytes(data[2..4].try_into()?),
            component_count: data[4],
            components: Component::new_components(&data[5..], data[4])?,
        })
    }
}

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
            0xC0 | 0xC2 | 0xC4 | 0xDB | 0xDD | 0xDA | 0xFE | 0xE0..=0xEF => {
                u16::from_be_bytes(bytes[2..4].try_into()?)
            }
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

fn ycrcb_to_rgb(y: f32, cr: f32, cb: f32) -> Result<[u16; 4], ImageError> {
    let r = (y + 1.40200 * (cr - 128.0)).clamp(0.0, 255.0) as u16;
    let g = (y - 0.34414 * (cb - 128.0) - 0.71414 * (cr - 128.0)).clamp(0.0, 255.0) as u16;
    let b = (y + 1.77200 * (cb - 128.0)).clamp(0.0, 255.0) as u16;
    Ok([r, g, b, u16::MAX])
}
