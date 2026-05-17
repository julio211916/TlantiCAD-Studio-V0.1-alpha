//! Screenshot capture utilities for off-screen rendering output

use serde::{Deserialize, Serialize};

/// Supported screenshot image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScreenshotFormat {
    PNG,
    JPEG,
    BMP,
}

/// Configuration for screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    pub width: u32,
    pub height: u32,
    pub format: ScreenshotFormat,
    /// RGBA background color (0..1 each channel)
    pub background_color: [f32; 4],
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            format: ScreenshotFormat::PNG,
            background_color: [0.15, 0.15, 0.15, 1.0],
        }
    }
}

/// Encode raw RGB pixels into a minimal PNG byte stream without external dependencies.
///
/// Uses PNG's uncompressed DEFLATE store block and proper CRC/Adler-32 checksums.
/// `pixels` must have exactly `width * height * 3` bytes (RGB, top-to-bottom).
pub fn encode_png(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();

    // PNG signature
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    // IHDR chunk
    let ihdr_data = {
        let mut d = Vec::new();
        d.extend_from_slice(&width.to_be_bytes());
        d.extend_from_slice(&height.to_be_bytes());
        d.push(8);  // bit depth
        d.push(2);  // color type: RGB
        d.push(0);  // compression method
        d.push(0);  // filter method
        d.push(0);  // interlace method
        d
    };
    write_chunk(&mut out, b"IHDR", &ihdr_data);

    // IDAT chunk — filter each row then store with zlib wrapper
    let row_size = (width * 3) as usize;
    let mut filtered = Vec::with_capacity((row_size + 1) * height as usize);
    for row in 0..height as usize {
        filtered.push(0); // filter type None
        let start = row * row_size;
        let end = start + row_size;
        let end = end.min(pixels.len());
        if start < pixels.len() {
            filtered.extend_from_slice(&pixels[start..end]);
        } else {
            filtered.extend(vec![0u8; row_size]);
        }
    }
    let idat_data = zlib_store(&filtered);
    write_chunk(&mut out, b"IDAT", &idat_data);

    // IEND chunk
    write_chunk(&mut out, b"IEND", b"");

    out
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

/// Wrap raw bytes in a minimal zlib stream using uncompressed DEFLATE store blocks.
fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // zlib header: CMF=0x78 (deflate, window=32KB), FLG=0x01 (no dict, check bits)
    out.push(0x78);
    out.push(0x01);

    // DEFLATE using non-compressed blocks (type 00)
    const MAX_BLOCK: usize = 65535;
    let mut pos = 0;
    while pos < data.len() {
        let end = (pos + MAX_BLOCK).min(data.len());
        let block = &data[pos..end];
        let is_last = end == data.len();
        let blen = block.len() as u16;
        out.push(if is_last { 0x01 } else { 0x00 }); // BFINAL | BTYPE=00
        out.extend_from_slice(&blen.to_le_bytes());
        out.extend_from_slice(&(!blen).to_le_bytes()); // one's complement
        out.extend_from_slice(block);
        pos = end;
    }
    if data.is_empty() {
        // Empty store block
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    }

    // Adler-32 checksum
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc ^ 0xFFFF_FFFF
}
