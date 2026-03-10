/// Raw VGA ROM CP437 8×16 bitmap font (256 glyphs × 16 rows = 4096 bytes).
/// Source: IBM VGA 8×16, public domain.
const FONT_DATA: &[u8; 4096] = include_bytes!("cp437_8x16.bin");

/// Returns `true` if pixel (px, py) within the glyph for byte `ch` is set.
/// px: 0..8, py: 0..16.
pub fn glyph_pixel(ch: u8, px: u32, py: u32) -> bool {
    if px >= 8 || py >= 16 {
        return false;
    }
    let row = FONT_DATA[usize::from(ch) * 16 + py as usize];
    row & (0x80 >> px) != 0
}
