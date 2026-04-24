use crate::error::ImageError;
use crc::{CRC_32_ISO_HLDC, Crc};

const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HLDC);

struct PngChunk {
    length: u32,
    /*
     * case of first letter determines criticality, second publicity, third should be upper,
     * fourth safe to copy if lower case, if upper only if modifications have not touched
     * other critical chunks
     */
    name: u32,
    data: Vec<u8>,
}

impl PngChunk {
    fn new(self, bytes: &[u8]) -> Result<(PngChunk, u64), ImageError> {
        let length: u32 = u32::from_be_bytes(bytes[..4].try_into()?);
        let new_chunk = PngChunk {
            length: length,
            name: u32::from_be_bytes(bytes[4..8].try_into()?),
            data: bytes[8..8 + length as usize].to_vec(),
        };
        let bytes_read: u64 = 12 + length as u64;
        check_crc(bytes, length)?;
        Ok((new_chunk, bytes_read))
    }
}

struct PngHeader {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

impl PngHeader {
    fn new(bytes: &[u8]) -> Result<PngHeader, ImageError> {
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
                "Png: first chunk was not the header chunk".to_string(),
            ));
        }

        check_crc(bytes, header_length);
        let header = PngHeader::new(&bytes[8..8 + header_length as usize]);
        let mut chunks: Vec<PngChunk> = Vec::new();
        let mut i = 8 + header_length as usize;
        while i < bytes.len() {}
        Ok(PngImageChunks { image: chunks })
    }
}

fn check_crc(bytes: &[u8], length: u32) -> Result<(), ImageError> {
    let crc_data = &bytes[4..8 + length as usize];
    let computed = PNG_CRC.checksum(crc_data);

    let stored_crc_start = 8 + length as usize;
    let stored = u32::from_be_bytes(
        bytes[stored_crc_start..stored_crc_start + 4]
            .try_into()
            .map_err(|_| ImageError::CustomError("CRC slice error".to_string()))?,
    );

    if computed != stored {
        return Err(ImageError::CustomError(format!(
            "PNG header CRC mismatch: computed {:#010x}, stored {:#010x}",
            computed, stored
        )));
    }
    Ok(())
}
