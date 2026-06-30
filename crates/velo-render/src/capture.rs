//! RGBA framebuffer utilities (platform-agnostic).

/// Convert BGRA8 pixels (Metal/wgpu default) to RGBA8.
pub fn bgra_to_rgba(bgra: &[u8], rgba: &mut [u8]) {
    assert_eq!(bgra.len(), rgba.len());
    assert_eq!(bgra.len() % 4, 0);
    for (src, dst) in bgra.chunks_exact(4).zip(rgba.chunks_exact_mut(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = src[3];
    }
}

/// PNG magic header bytes.
pub const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramebufferRgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl FramebufferRgba {
    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }

    pub fn expected_len(width: u32, height: u32) -> usize {
        width as usize * height as usize * 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_to_rgba_swaps_channels() {
        let bgra = [0x11, 0x22, 0x33, 0xFF];
        let mut rgba = [0u8; 4];
        bgra_to_rgba(&bgra, &mut rgba);
        assert_eq!(rgba, [0x33, 0x22, 0x11, 0xFF]);
    }

    #[test]
    fn framebuffer_dimensions() {
        let fb = FramebufferRgba {
            width: 2,
            height: 2,
            pixels: vec![0; 16],
        };
        assert_eq!(fb.byte_len(), FramebufferRgba::expected_len(2, 2));
    }
}
