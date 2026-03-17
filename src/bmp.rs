use crate::error::ImageError;
use std::fs::File;
use std::io::Read;

pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

struct BmpFileHeader {
    signature: u16,
    size: u32,
    reserve: u32,
    start: u32,
}

struct BitMapImageHeader {
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
    header: BmpFileHeader,
    dib_header: BitMapImageHeader,
    pixels: Vec<Vec<Rgb>>,
}

impl BmpImage {
    fn new(
        header: BmpFileHeader,
        dib_header: BitMapImageHeader,
        file: &str,
    ) -> Result<BmpImage, ImageError> {
        let mut buffer = Vec::new();
        let file = File::open(file)?;
        file.read(&mut buffer)?;
        let image = &buffer[37..];
        let pixels = image;
        Ok(BmpImage {
            header: header,
            dib_header: dib_header,
            pixels: pixels,
        })
    }
}
