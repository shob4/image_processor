pub enum Pixels {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
    BigRgb(u16, u16, u16),
    BigRgba(u16, u16, u16, u16),
    Gray(u8),
    GrayAlpha(u8, u8),
    BigGray(u16),
    BigGrayAlpha(u16, u16),
}
