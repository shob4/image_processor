use crate::error::ImageError;

struct PngChunk {
    length: u32,
    /*
     * case of first letter determines criticality, second publicity, third should be upper,
     * fourth safe to copy if lower case, if upper only if modifications have not touched
     * other critical chunks
     */
    name: u32,
    data: Vec<u8>,
    crc: u32,
}

pub struct PngHeader {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

impl PngHeader {
    pub fn new(bytes: &[u8]) -> Result<PngHeader, ImageError> {
        Ok(PngHeader {
            width: u32::from_be_bytes(bytes[..4].try_into()?),
            height: u32::from_be_bytes(bytes[4..8].try_into()?),
            bit_depth: bytes[9],
            color_type: bytes[10],
            compression_method: bytes[11],
            filter_method: bytes[12],
            interlace_method: bytes[13],
        })
    }
}

/*
* necessary for color type 3 (indexed color), optional for 2 and 6 (truecolor, truecolor with alpha)
* should not appear for 0 and 4 (grayscale, grayscale with alpha)
*/
pub struct PngPalette {
    palette: Vec<(u8, u8, u8)>,
}

pub struct PngPaletteAlpha {
    palette: Vec<(u8, u8, u8, u8)>,
}

// may be changed to hold data only, not entire chunks, may be split into different structs
pub struct PngImageChunks {
    image: Vec<PngChunk>,
}

impl PngImageChunks {
    pub fn new(bytes: &[u8]) -> Result<PngImageChunks, ImageError> {
        let header_length = u32::from_be_bytes(bytes[..4].try_into()?);
        let header_id = u32::from_be_bytes(bytes[4..8].try_into()?);
        if header_id != 0x49484452 {
            return Err(ImageError::CustomError(
                "first chunk was not the header chunk".to_string(),
            ));
        }
        let header = PngHeader::new(&bytes[8..8 + header_length as usize]);
        let mut chunks: Vec<PngChunk> = Vec::new();
        Ok(PngImageChunks { image: chunks })
    }
}
