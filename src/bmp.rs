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
            let base: usize = 2;
            base.pow(dib_header.bits_per_pixel as u32)
        } else {
            dib_header.color_count as usize
        };

        if dib_header.bits_per_pixel < 24 {
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
            let palette_offset = num_colors * 4;
            let pixels = match choose_decoding_method(
                dib_header.compression_method_raw,
                &image[palette_offset..],
                dib_header.bits_per_pixel,
                Some(palette),
            ) {
                Ok(pixel_array) => pixel_array,
                Err(e) => return Err(e),
            };
            return Ok(BmpImage {
                dib_header: dib_header,
                pixels: pixels,
            });
        }

        let pixels = match choose_decoding_method(
            dib_header.compression_method_raw,
            image,
            dib_header.bits_per_pixel,
            None,
        ) {
            Ok(pixel_array) => pixel_array,
            Err(e) => return Err(e),
        };

        Ok(BmpImage {
            dib_header: dib_header,
            pixels: pixels,
        })
    }
}

fn choose_decoding_method(
    method: u32,
    image: &[u8],
    bits_per_pixel: u16,
    palette: Option<Vec<Rgb>>,
) -> Result<Vec<Rgb>, ImageError> {
    // TODO not sure the encoded pixels need a palette to be decoded
    match method {
        0 => read_unencoded_pixels(image, bits_per_pixel),
        1 => {
            let palette = match palette {
                Some(colors) => colors,
                None => {
                    return Err(ImageError::CustomError(
                        "no palette to decode RLE encoded bmp with".to_string(),
                    ));
                }
            };
            decode_RLE8(image, bits_per_pixel, palette)
        }
        2 => {
            let palette = match palette {
                Some(colors) => colors,
                None => {
                    return Err(ImageError::CustomError(
                        "no palette to decode RLE encoded bmp with".to_string(),
                    ));
                }
            };
            decode_RLE4(image, bits_per_pixel, palette)
        }
        12 => {
            let palette = match palette {
                Some(colors) => colors,
                None => {
                    return Err(ImageError::CustomError(
                        "no palette to decode RLE encoded bmp with".to_string(),
                    ));
                }
            };
            decode_RLE8(image, bits_per_pixel, palette)
        }
        13 => {
            let palette = match palette {
                Some(colors) => colors,
                None => {
                    return Err(ImageError::CustomError(
                        "no palette to decode RLE encoded bmp with".to_string(),
                    ));
                }
            };
            decode_RLE4(image, bits_per_pixel, palette)
        }
        _ => panic!("dib_header read an invalid compression method"),
    }
}

fn decode_RLE8(
    image: &[u8],
    bits_per_pixel: u16,
    palette: Vec<Rgb>,
) -> Result<Vec<Rgb>, ImageError> {
    let decoded_image: Vec<Rgb> = Vec::new();
    Ok(decoded_image)
}

fn decode_RLE4(
    image: &[u8],
    bits_per_pixel: u16,
    palette: Vec<Rgb>,
) -> Result<Vec<Rgb>, ImageError> {
    let decoded_image: Vec<Rgb> = Vec::new();
    Ok(decoded_image)
}

fn decode_RLE24(
    image: &[u8],
    bits_per_pixel: u16,
    palette: Vec<Rgb>,
) -> Result<Vec<Rgb>, ImageError> {
    let decoded_image: Vec<Rgb> = Vec::new();
    Ok(decoded_image)
}

fn read_unencoded_pixels(image: &[u8], bits_per_pixel: u16) -> Result<Vec<Rgb>, ImageError> {
    let pixel_size = bits_per_pixel as usize;
    let mut decoded_image: Vec<Rgb> = Vec::new();
    // this stuff is confusing
    for i in (0..(image.len() / pixel_size)).rev() {
        let blue = image[i * pixel_size..i * pixel_size + pixel_size];
        let green = image[i * pixel_size + pixel_size + 1..i * pixel_size + 2 * pixel_size];
        let red = image[i * pixel_size + 2 * pixel_size + 1];
        decoded_image.push(Rgb {
            red: red,
            green: green,
            blue: blue,
        });
    }
    Ok(decoded_image)
}
