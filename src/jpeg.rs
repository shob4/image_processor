use crate::error::ImageError;

#[derive(Debug)]
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            byte_pos: 0,
            bit_pos: 8,
        }
    }

    fn read_bit(&mut self) -> Result<u8, ImageError> {
        if self.bit_pos == 8 {
            self.byte_pos += 1;
            if self.data[self.byte_pos] == 0xFF {
                if self.data[self.byte_pos + 1] != 0x00 {
                    return Err(ImageError::CustomError("unexpected marker".to_string()));
                }
                self.byte_pos += 1;
            }
            self.bit_pos = 0;
        }
        let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;
        Ok(bit)
    }
}

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
struct QuantizationTable {
    length: u16,
    precision_table_id: u8,
    table: [u16; 64],
}

impl QuantizationTable {
    fn new(chunk: JpegChunk) -> Result<QuantizationTable, ImageError> {
        if chunk.indicator != 0xDB {
            return Err(ImageError::CustomError(format!(
                "{:#X} is not the quantization table",
                chunk.indicator
            )));
        }
        let data = chunk.get_data()?;
        let precision = data[0];
        let table: [u16; 64] = match precision {
            0 => {
                if data[1..].len() <= 64 {
                    return Err(ImageError::CustomError(
                        "need at least 64 bytes of data".to_string(),
                    ));
                }
                let mut arr = [0u16; 64];
                for (i, &byte) in data[1..].iter().enumerate() {
                    arr[i] = byte as u16;
                }
                arr
            }
            1 => {
                if data[3..].len() <= 128 {
                    return Err(ImageError::CustomError(
                        "need at least 128 bytes of data".to_string(),
                    ));
                }
                let mut arr = [0u16; 64];
                for (i, chunk) in data[1..].chunks_exact(2).take(64).enumerate() {
                    arr[i] = u16::from_be_bytes([chunk[0], chunk[1]]);
                }
                arr
            }
            _ => {
                return Err(ImageError::CustomError(format!(
                    "invalid quantization precision: {}",
                    precision
                )));
            }
        };
        Ok(QuantizationTable {
            length: u16::from_be_bytes(data[0..2].try_into()?),
            precision_table_id: precision,
            table: table,
        })
    }
}

#[derive(Debug)]
struct HuffmanTable {
    count: [u8; 16],
    class: u8,
    id: u8,
    min_code: [i32; 16],
    val_offset: [i32; 16],
    symbols: Vec<u8>,
}

impl HuffmanTable {
    fn new(chunk: JpegChunk) -> Result<HuffmanTable, ImageError> {
        if chunk.indicator != 0xC4 {
            return Err(ImageError::CustomError(format!(
                "{:#X} is not the huffman table",
                chunk.indicator
            )));
        }
        let length = chunk.length;
        let data = chunk.get_data()?;
        let class_and_id = data[0];
        let class = class_and_id >> 4;
        let id = class_and_id & 0x0F;
        let mut count = [0u8; 16];
        for (i, byte) in data[1..16].iter().enumerate() {
            count[i] = *byte;
        }
        let values: Vec<u8> = data[16..].iter().map(|&b| b).collect();

        let mut min_code = [i32::MAX; 16];
        let mut val_offset = [0i32; 16];

        let mut code: i32 = 0;
        let mut val_index: i32 = 0;

        for i in 0..16 {
            if count[i] > 0 {
                min_code[i] = code;
                val_offset[i] = val_index - code;
            }
            val_index += count[i] as i32;
            code = (code + count[i] as i32) << 1;
        }

        Ok(HuffmanTable {
            count: count,
            class: class,
            id: id,
            min_code: min_code,
            val_offset: val_offset,
            symbols: symbols,
        })
    }

    fn decode_symbol(&self, bits: &mut BitReader) -> Result<u8, ImageError> {
        let mut code: i32 = 0;
        for i in 0..16 {
            code = (code << 1) | bits.read_bit()? as i32;
            if self.min_code[i] == i32::MAX {
                continue;
            }
            if code >= self.min_code[i] && code < self.min_code[i] + self.count[i] as i32 {
                let index = (self.val_offset[i] + code) as usize;
                return Ok(self.symbols[index]);
            }
        }
        Err(ImageError::CustomError("invalid huffman code".to_string()))
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

    pub fn get_data(self) -> Result<Vec<u8>, ImageError> {
        match self.data {
            Some(data) => Ok(data),
            None => Err(ImageError::CustomError(format!(
                "{:#X} held no data",
                self.indicator
            ))),
        }
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
