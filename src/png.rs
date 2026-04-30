use crate::error::ImageError;
use crc::{CRC_32_ISO_HDLC, Crc};

const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

const PLTE: u32 = u32::from_be_bytes(*b"PLTE");
const IDAT: u32 = u32::from_be_bytes(*b"IDAT");

#[derive(Debug)]
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
    fn new(bytes: &[u8]) -> Result<PngChunk, ImageError> {
        let length: u32 = u32::from_be_bytes(bytes[..4].try_into()?);
        let new_chunk = PngChunk {
            length: length,
            name: u32::from_be_bytes(bytes[4..8].try_into()?),
            data: bytes[8..8 + length as usize].to_vec(),
        };
        check_crc(bytes, length)?;
        Ok(new_chunk)
    }
}

#[derive(Debug)]
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

// may be changed to hold data only, not entire chunks, may be split into different structs
#[derive(Debug)]
pub struct PngImageChunks {
    image: Vec<PngChunk>,
}

impl PngImageChunks {
    pub fn new(bytes: &[u8]) -> Result<(PngImageChunks, PngHeader), ImageError> {
        let header_length = u32::from_be_bytes(bytes[..4].try_into()?);
        let header_id = u32::from_be_bytes(bytes[4..8].try_into()?);
        if header_id != 0x49484452 {
            return Err(ImageError::CustomError(
                "Png: first chunk was not the header chunk".to_string(),
            ));
        }

        check_crc(bytes, header_length);
        let header = PngHeader::new(&bytes[8..8 + header_length as usize])?;
        let mut chunks: Vec<PngChunk> = Vec::new();
        let mut i = 8 + header_length as usize;
        while i < bytes.len() {
            let new_chunk = PngChunk::new(&bytes[i..])?;
            i = new_chunk.length as usize + i + 4;
            chunks.push(new_chunk);
        }
        Ok((PngImageChunks { image: chunks }, header))
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

// TODO figure out structure
// bit_depth
// color_type
// compression_method
// filter_method
// interlace_method
// need to read chunk name and data
fn build_image(header: PngHeader, chunks: PngImageChunks) -> Result<(), ImageError> {
    let palette_chunk = chunks.image.iter().find(|c| c.name == PLTE);
    let idat_chunks: Vec<&PngChunk> = chunks.image.iter().filter(|c| c.name == IDAT).collect();

    let pixels: Vec<[u8; 3]> = match header.color_type {
        0b000 => read_pixels(idat_chunks, header.bit_depth, 1)?,
        0b010 => read_pixels(idat_chunks, header.bit_depth, 3)?,
        0b011 => {
            let chunk = palette_chunk
                .ok_or_else(|| ImageError::CustomError("no palette found in png".to_string()))?;

            if chunk.data.len() % 3 != 0 {
                return Err(ImageError::CustomError(
                    "invalid palette length: not divisible by 3".to_string(),
                ));
            }
            let entry_count = chunk.data.len() / 3;
            if entry_count < 1 || entry_count > 256 {
                return Err(ImageError::CustomError(format!(
                    "invalid palette size: {}",
                    entry_count
                )));
            }

            let palette = chunk
                .data
                .chunks_exact(3)
                .map(|rgb| [rgb[0], rgb[1], rgb[2]])
                .collect();

            read_pixels_with_palette(idat_chunks, palette, header.bit_depth)?
        }
        0b100 => read_pixels(idat_chunks, header.bit_depth, 2)?,
        0b110 => read_pixels(idat_chunks, header.bit_depth, 4)?,
        _ => {
            return Err(ImageError::CustomError(format!(
                "invalid color type: {0}",
                header.color_type
            )));
        }
    };
    Ok(())
}

fn read_pixels(
    image_data: Vec<&PngChunk>,
    bit_depth: u8,
    num_channels: u8,
) -> Result<Vec<[u8; 3]>, ImageError> {
    let pixels: Vec<[u8; 3]> = Vec::new();
    for pixel in image_data {}
    Ok(pixels)
}

fn read_pixels_with_palette(
    image_data: Vec<&PngChunk>,
    palette: Vec<[u8; 3]>,
    bit_depth: u8,
) -> Result<Vec<[u8; 3]>, ImageError> {
    let mut image: Vec<[u8; 3]> = Vec::new();

    for chunk in image_data {
        let indices: Vec<u8> = match bit_depth {
            8 => chunk.data.to_vec(),
            4 => chunk
                .data
                .iter()
                .flat_map(|&b| [b >> 4, b & 0x0F])
                .collect(),
            2 => chunk
                .data
                .iter()
                .flat_map(|&b| [b >> 6, (b >> 4) & 0x03, (b >> 2) & 0x03, b & 0x03])
                .collect(),
            1 => chunk
                .data
                .iter()
                .flat_map(|&b| (0..8).rev().map(move |i| (b >> i) & 1))
                .collect(),
            _ => {
                return Err(ImageError::CustomError(format!(
                    "invalid bit depth: {}",
                    bit_depth
                )));
            }
        };
        let mut pixels: Vec<[u8; 3]> = indices.iter().map(|&i| palette[i as usize]).collect();
        image.append(&mut pixels);
    }
    Ok(image)
}

// let chunk_name = chunk.name.to_string();
// match chunk_name.as_str() {
//     "IDAT" => {
//         todo!("reference palette");
//     }
//     "bKGD" => {
//         todo!("default background color");
//     }
//     "cHRM" => {
//         todo!("chromacity coordinates of display primaries and white point");
//     }
//     "cICP" => {
//         todo!("defines color space");
//     }
//     "hIST" => {
//         todo!("total amount of each color in image");
//     }
//     "tRNS" => {
//         todo!("contains transparency info")
//     }
//     _ => {
//         todo!("read data as normal");
//     }
// }
