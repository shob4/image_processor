pub enum ImageTypes {
    Bmp,
}

struct BmpFileHeader {
    signature: [u8; 2],
    size: [u8; 4],
    reserve: [u8; 2],
    reserve2: [u8; 2],
    start: [u8; 4],
}

struct BmpImage {
    dib_header: [u8; 7],
}
