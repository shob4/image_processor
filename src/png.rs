pub struct PngChunk {
    pub length: u32,
    /*
     * case of first letter determines criticality, second publicity, third should be upper,
     * fourth safe to copy if lower case, if upper only if modifications have not touched
     * other critical chunks
     */
    pub name: u32,
    pub data: Vec<u8>,
    pub crc: u32,
}
