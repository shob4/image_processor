use crate::error::ImageError;
use std::fs::File;
use std::io::Read;

pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct BitMapImageHeader {
    size: u32,
    width: i32,
    height: i32,
    number_of_color_planes: u16,
    bits_per_pixel: u16,
    compression_method_raw: u32,
    image_size: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    color_count: u32,
    important_color_count: u32,
}

impl BitMapImageHeader {
    pub fn new(header_bytes: &[u8]) -> Result<BitMapImageHeader, ImageError> {
        Ok(BitMapImageHeader {
            size: u32::from_le_bytes(header_bytes[..4].try_into()?),
            width: i32::from_le_bytes(header_bytes[4..8].try_into()?),
            height: i32::from_le_bytes(header_bytes[8..12].try_into()?),
            number_of_color_planes: u16::from_le_bytes(header_bytes[12..14].try_into()?),
            bits_per_pixel: u16::from_le_bytes(header_bytes[14..16].try_into()?),
            compression_method_raw: u32::from_le_bytes(header_bytes[16..20].try_into()?),
            image_size: u32::from_le_bytes(header_bytes[20..24].try_into()?),
            horizontal_resolution: u32::from_le_bytes(header_bytes[24..28].try_into()?),
            vertical_resolution: u32::from_le_bytes(header_bytes[28..32].try_into()?),
            color_count: u32::from_le_bytes(header_bytes[32..36].try_into()?),
            important_color_count: u32::from_le_bytes(header_bytes[36..40].try_into()?),
        })
    }
}

enum BmpCompressionMethods {
    BiRgb,
    BiRle8,
    BiRle4,
    BiBitfields,
    BiJpeg,
    BiPng,
    BiAlphabitfields,
    BiCmyk,
    BiCmykrle8,
    BiCmykrle4,
}

pub struct BmpImage {
    dib_header: BitMapImageHeader,
    pixels: Vec<Rgb>,
}

impl BmpImage {
    pub fn new(dib_header: BitMapImageHeader, mut file: File) -> Result<BmpImage, ImageError> {
        let mut buffer = Vec::new();
        file.read(&mut buffer)?;
        let image = &buffer[dib_header.size as usize..];
        let num_colors = if dib_header.color_count == 0 {
            1 << dib_header.bits_per_pixel
        } else {
            dib_header.color_count as usize
        };
        let mut palette = Vec::new();
        for i in 0..num_colors {
            let b = image[i * 4];
            let g = image[i * 4 + 1];
            let r = image[i * 4 + 2];
            let color = Rgb {
                red: r,
                green: g,
                blue: b,
            };
            palette.push(color)
        }
        Ok(BmpImage {
            dib_header: dib_header,
            pixels: palette,
        })
    }
}
