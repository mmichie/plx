use image::{ImageBuffer, ImageEncoder, Rgba, RgbaImage};
use std::io::{self, Write};

use super::font;
use super::sysinfo::SystemInfo;

const COLS: u32 = 80;
const ROWS: u32 = 25;
const GLYPH_W: u32 = 8;
const GLYPH_H: u32 = 16;

// CP437 box-drawing / shade characters
const BOX_HZ: u8 = 0xC4; // ─
const BOX_TL: u8 = 0xC9; // ╔
const BOX_TR: u8 = 0xBB; // ╗
const BOX_BL: u8 = 0xC8; // ╚
const BOX_BR: u8 = 0xBC; // ╝
const BOX_VT: u8 = 0xBA; // ║
const BOX_DB: u8 = 0xCD; // ═
const BOX_RT: u8 = 0xB5; // ╡
const BOX_LT: u8 = 0xC6; // ╞
const BULLET: u8 = 0xF9; // ∙
const DIAMOND: u8 = 0x04; // ♦

struct Palette {
    bg: Rgba<u8>,
    header_fg: Rgba<u8>,
    border_fg: Rgba<u8>,
    label_fg: Rgba<u8>,
    value_fg: Rgba<u8>,
    gradient: [Rgba<u8>; 4],
    title_face: Rgba<u8>,
    title_hi: Rgba<u8>,
    title_shadow: Rgba<u8>,
}

fn rgba(r: u8, g: u8, b: u8) -> Rgba<u8> {
    Rgba([r, g, b, 255])
}

fn palette_by_name(name: &str) -> Palette {
    match name {
        "fire" => Palette {
            bg: rgba(20, 5, 0),
            header_fg: rgba(255, 100, 20),
            border_fg: rgba(180, 60, 10),
            label_fg: rgba(200, 80, 15),
            value_fg: rgba(255, 180, 60),
            gradient: [
                rgba(80, 20, 0),
                rgba(160, 50, 5),
                rgba(220, 80, 10),
                rgba(255, 140, 30),
            ],
            title_face: rgba(210, 70, 8),
            title_hi: rgba(255, 230, 120),
            title_shadow: rgba(30, 5, 0),
        },
        "matrix" => Palette {
            bg: rgba(0, 10, 0),
            header_fg: rgba(0, 255, 65),
            border_fg: rgba(0, 140, 35),
            label_fg: rgba(0, 180, 45),
            value_fg: rgba(80, 255, 120),
            gradient: [
                rgba(0, 40, 10),
                rgba(0, 100, 25),
                rgba(0, 180, 45),
                rgba(0, 255, 65),
            ],
            title_face: rgba(0, 170, 42),
            title_hi: rgba(180, 255, 200),
            title_shadow: rgba(0, 18, 4),
        },
        "steel" => Palette {
            bg: rgba(15, 18, 22),
            header_fg: rgba(200, 210, 220),
            border_fg: rgba(100, 110, 120),
            label_fg: rgba(140, 150, 160),
            value_fg: rgba(220, 225, 230),
            gradient: [
                rgba(50, 55, 65),
                rgba(90, 95, 105),
                rgba(140, 150, 160),
                rgba(200, 210, 220),
            ],
            title_face: rgba(150, 160, 175),
            title_hi: rgba(235, 240, 250),
            title_shadow: rgba(18, 20, 30),
        },
        // "cyber" and default
        _ => Palette {
            bg: rgba(8, 4, 20),
            header_fg: rgba(213, 110, 255),
            border_fg: rgba(99, 60, 180),
            label_fg: rgba(141, 80, 200),
            value_fg: rgba(177, 160, 255),
            gradient: [
                rgba(57, 20, 100),
                rgba(93, 40, 160),
                rgba(141, 70, 220),
                rgba(213, 110, 255),
            ],
            title_face: rgba(130, 60, 210),
            title_hi: rgba(240, 210, 255),
            title_shadow: rgba(18, 5, 40),
        },
    }
}

// ---------------------------------------------------------------------------
// Character-level drawing (for text, boxes, bars)
// ---------------------------------------------------------------------------

fn draw_char(
    img: &mut RgbaImage,
    col: u32,
    row: u32,
    ch: u8,
    fg: Rgba<u8>,
    bg: Rgba<u8>,
    scale: u32,
) {
    let x0 = col * GLYPH_W * scale;
    let y0 = row * GLYPH_H * scale;
    for py in 0..GLYPH_H {
        for px in 0..GLYPH_W {
            let color = if font::glyph_pixel(ch, px, py) { fg } else { bg };
            for sy in 0..scale {
                for sx in 0..scale {
                    let ix = x0 + px * scale + sx;
                    let iy = y0 + py * scale + sy;
                    if ix < img.width() && iy < img.height() {
                        img.put_pixel(ix, iy, color);
                    }
                }
            }
        }
    }
}

fn draw_bytes(
    img: &mut RgbaImage,
    col: u32,
    row: u32,
    data: &[u8],
    fg: Rgba<u8>,
    bg: Rgba<u8>,
    scale: u32,
) {
    for (i, &ch) in data.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let c = col + i as u32;
        if c < COLS {
            draw_char(img, c, row, ch, fg, bg, scale);
        }
    }
}

fn draw_text(
    img: &mut RgbaImage,
    col: u32,
    row: u32,
    text: &str,
    fg: Rgba<u8>,
    bg: Rgba<u8>,
    scale: u32,
) {
    draw_bytes(img, col, row, text.as_bytes(), fg, bg, scale);
}

fn draw_gradient_bar(
    img: &mut RgbaImage,
    row: u32,
    gradient: &[Rgba<u8>; 4],
    reverse: bool,
    scale: u32,
) {
    // CP437: 0xB0=░ 0xB1=▒ 0xB2=▓ 0xDB=█
    let chars: [u8; 4] = if reverse {
        [0xDB, 0xB2, 0xB1, 0xB0]
    } else {
        [0xB0, 0xB1, 0xB2, 0xDB]
    };
    let bg = Rgba([0, 0, 0, 255]);
    for col in 0..COLS {
        let gi = (col as usize * 4 / COLS as usize).min(3);
        let ci = col as usize % 4;
        draw_char(img, col, row, chars[ci], gradient[gi], bg, scale);
    }
}

/// Draw text with a shadow on the row below for a bolder look.
fn draw_block_text(
    img: &mut RgbaImage,
    col: u32,
    row: u32,
    text: &str,
    fg: Rgba<u8>,
    bg: Rgba<u8>,
    scale: u32,
) {
    let dim = Rgba([fg.0[0] / 3, fg.0[1] / 3, fg.0[2] / 3, 255]);
    for (i, ch) in text.bytes().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let c = col + i as u32;
        if c < COLS {
            draw_char(img, c, row, ch, fg, bg, scale);
            draw_char(img, c, row + 1, ch, dim, bg, scale);
        }
    }
}

// ---------------------------------------------------------------------------
// Pixel-level drawing helpers (for 3D title)
// ---------------------------------------------------------------------------

fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    let x_end = (x + w).min(img.width());
    let y_end = (y + h).min(img.height());
    for py in y..y_end {
        for px in x..x_end {
            img.put_pixel(px, py, color);
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn lerp_color(a: Rgba<u8>, b: Rgba<u8>, t: f32) -> Rgba<u8> {
    let mix = |a: u8, b: u8| -> u8 {
        f32::from(a)
            .mul_add(1.0 - t, f32::from(b) * t)
            .clamp(0.0, 255.0) as u8
    };
    Rgba([mix(a.0[0], b.0[0]), mix(a.0[1], b.0[1]), mix(a.0[2], b.0[2]), 255])
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn scale_color(c: Rgba<u8>, factor: f32) -> Rgba<u8> {
    Rgba([
        (f32::from(c.0[0]) * factor).clamp(0.0, 255.0) as u8,
        (f32::from(c.0[1]) * factor).clamp(0.0, 255.0) as u8,
        (f32::from(c.0[2]) * factor).clamp(0.0, 255.0) as u8,
        255,
    ])
}

#[allow(clippy::cast_precision_loss, clippy::many_single_char_names)]
fn fill_rect_vgradient(
    img: &mut RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    top: Rgba<u8>,
    bot: Rgba<u8>,
) {
    if h == 0 {
        return;
    }
    let x_end = (x + w).min(img.width());
    let y_end = (y + h).min(img.height());
    let denom = (h - 1).max(1) as f32;
    for dy in 0..h {
        if y + dy >= y_end {
            break;
        }
        let t = dy as f32 / denom;
        let color = lerp_color(top, bot, t);
        for px in x..x_end {
            img.put_pixel(px, y + dy, color);
        }
    }
}

// ---------------------------------------------------------------------------
// Box / divider helpers
// ---------------------------------------------------------------------------

fn make_divider_bytes(width: u32) -> Vec<u8> {
    let w = width as usize;
    let half = (w.saturating_sub(1)) / 2;
    let mut out = vec![BOX_HZ; half];
    out.push(DIAMOND);
    out.resize(w, BOX_HZ);
    out
}

fn make_ornament_line(width: u32) -> Vec<u8> {
    let w = width as usize;
    let center = [BOX_RT, b' ', DIAMOND, b' ', BOX_LT];
    let start = (w.saturating_sub(center.len())) / 2;
    let mut out = vec![BOX_DB; w];
    for (i, &ch) in center.iter().enumerate() {
        if start + i < w {
            out[start + i] = ch;
        }
    }
    out
}

fn make_double_divider(width: u32) -> Vec<u8> {
    let w = width as usize;
    let half = (w.saturating_sub(1)) / 2;
    let mut out = vec![BOX_DB; half];
    out.push(DIAMOND);
    out.resize(w, BOX_DB);
    out
}

fn make_box_top(label: &str, width: usize) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_TL, BOX_DB, BOX_DB, b' '];
    out.extend_from_slice(label.as_bytes());
    out.push(b' ');
    while out.len() < width - 1 {
        out.push(BOX_DB);
    }
    out.push(BOX_TR);
    out
}

fn make_box_mid(content: &str, width: usize) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_VT, b' ', b' ', BULLET, b' '];
    out.extend_from_slice(content.as_bytes());
    while out.len() < width - 1 {
        out.push(b' ');
    }
    out.push(BOX_VT);
    out
}

fn make_box_bot(label: &str, width: usize) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_BL, BOX_DB, BOX_DB, b' '];
    out.extend_from_slice(label.as_bytes());
    out.push(b' ');
    while out.len() < width - 1 {
        out.push(BOX_DB);
    }
    out.push(BOX_BR);
    out
}

fn day_of_year() -> u32 {
    let mut t: libc::time_t = 0;
    unsafe { libc::time(&raw mut t) };
    let tm = unsafe { libc::localtime(&raw const t) };
    if tm.is_null() {
        return 0;
    }
    #[allow(clippy::cast_sign_loss)]
    unsafe {
        (*tm).tm_yday as u32
    }
}

// ---------------------------------------------------------------------------
// Big letter bitmap font (A-Z, 7 rows × 3-5 wide)
// ---------------------------------------------------------------------------

/// Returns (width, bitmaps) for A-Z block letters.
/// Each row is a u8 with set bits from MSB (bit 7 = leftmost column).
fn big_letter(ch: u8) -> Option<(u32, [u8; 7])> {
    match ch.to_ascii_uppercase() {
        b'A' => Some((5, [0x70, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88])),
        b'B' => Some((5, [0xF0, 0x88, 0x88, 0xF0, 0x88, 0x88, 0xF0])),
        b'C' => Some((5, [0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70])),
        b'D' => Some((5, [0xF0, 0x88, 0x88, 0x88, 0x88, 0x88, 0xF0])),
        b'E' => Some((4, [0xF0, 0x80, 0x80, 0xE0, 0x80, 0x80, 0xF0])),
        b'F' => Some((4, [0xF0, 0x80, 0x80, 0xE0, 0x80, 0x80, 0x80])),
        b'G' => Some((5, [0x70, 0x88, 0x80, 0x98, 0x88, 0x88, 0x70])),
        b'H' => Some((5, [0x88, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88])),
        b'I' => Some((3, [0xE0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xE0])),
        b'J' => Some((4, [0x70, 0x10, 0x10, 0x10, 0x10, 0x90, 0x60])),
        b'K' => Some((5, [0x88, 0x90, 0xA0, 0xC0, 0xA0, 0x90, 0x88])),
        b'L' => Some((4, [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF0])),
        b'M' => Some((5, [0x88, 0xD8, 0xA8, 0xA8, 0x88, 0x88, 0x88])),
        b'N' => Some((5, [0x88, 0xC8, 0xA8, 0xA8, 0xA8, 0x98, 0x88])),
        b'O' => Some((5, [0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70])),
        b'P' => Some((5, [0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80])),
        b'Q' => Some((5, [0x70, 0x88, 0x88, 0x88, 0xA8, 0x90, 0x68])),
        b'R' => Some((5, [0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88])),
        b'S' => Some((5, [0x70, 0x88, 0x80, 0x70, 0x08, 0x88, 0x70])),
        b'T' => Some((5, [0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20])),
        b'U' => Some((5, [0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70])),
        b'V' => Some((5, [0x88, 0x88, 0x88, 0x88, 0x50, 0x50, 0x20])),
        b'W' => Some((5, [0x88, 0x88, 0x88, 0x88, 0xA8, 0xD8, 0x88])),
        b'X' => Some((5, [0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88])),
        b'Y' => Some((5, [0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20])),
        b'Z' => Some((5, [0xF8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xF8])),
        _ => None,
    }
}

/// True if pixel (px, py) is set in a big letter's bitmap rows.
fn pixel_set(rows: [u8; 7], px: u32, py: u32) -> bool {
    py < 7 && px < 8 && (rows[py as usize] & (0x80 >> px)) != 0
}

// ---------------------------------------------------------------------------
// 3D block letter renderer (pixel-level)
// ---------------------------------------------------------------------------

struct BigLetter {
    width: u32,
    rows: [u8; 7],
}

#[allow(clippy::cast_precision_loss)]
fn draw_big_title(
    img: &mut RgbaImage,
    start_row: u32,
    text: &str,
    pal: &Palette,
    scale: u32,
) {
    let letters: Vec<BigLetter> = text
        .bytes()
        .filter_map(|ch| big_letter(ch).map(|(w, rows)| BigLetter { width: w, rows }))
        .collect();
    if letters.is_empty() {
        return;
    }

    // Each bitmap pixel = cell_w × cell_h screen pixels
    let cell_w = 2 * GLYPH_W * scale;
    let cell_h = GLYPH_H * scale;

    // Total width in bitmap pixels (1-pixel gap between letters)
    #[allow(clippy::cast_possible_truncation)]
    let total_cells: u32 =
        letters.iter().map(|l| l.width).sum::<u32>() + (letters.len() as u32 - 1);
    let total_w = total_cells * cell_w;
    let img_w = COLS * GLYPH_W * scale;
    let start_x = (img_w.saturating_sub(total_w)) / 2;
    let start_y = start_row * GLYPH_H * scale;

    // 3D extrusion: 4 layers, each offset by (ex, ey) pixels
    let depth: u32 = 4;
    let ex = scale * 3;
    let ey = scale * 3;
    let bevel = (scale * 2).max(1);

    // Pre-compute extrusion layer colors (deepest=darkest → shallowest=brighter)
    let extrude_mid = scale_color(pal.title_face, 0.35);

    // Pass 1: Extrusion layers (deepest first, so shallower overwrites)
    for d in (1..=depth).rev() {
        let t = d as f32 / depth as f32;
        let color = lerp_color(extrude_mid, pal.title_shadow, t);
        let mut cursor = 0u32;
        for letter in &letters {
            for by in 0..7u32 {
                for bx in 0..letter.width {
                    if pixel_set(letter.rows, bx, by) {
                        let x = start_x + (cursor + bx) * cell_w + d * ex;
                        let y = start_y + by * cell_h + d * ey;
                        fill_rect(img, x, y, cell_w, cell_h, color);
                    }
                }
            }
            cursor += letter.width + 1;
        }
    }

    // Pass 2: Face with vertical gradient + bevel edges
    let face_top = scale_color(pal.title_face, 1.2);
    let face_bot = scale_color(pal.title_face, 0.7);
    let bevel_dark = scale_color(pal.title_face, 0.3);

    let mut cursor = 0u32;
    for letter in &letters {
        for by in 0..7u32 {
            for bx in 0..letter.width {
                if !pixel_set(letter.rows, bx, by) {
                    continue;
                }
                let x = start_x + (cursor + bx) * cell_w;
                let y = start_y + by * cell_h;

                // Gradient face
                fill_rect_vgradient(img, x, y, cell_w, cell_h, face_top, face_bot);

                // Edge detection
                let has_top = by == 0 || !pixel_set(letter.rows, bx, by - 1);
                let has_left = bx == 0 || !pixel_set(letter.rows, bx - 1, by);
                let has_bottom = by >= 6 || !pixel_set(letter.rows, bx, by + 1);
                let has_right =
                    bx + 1 >= letter.width || !pixel_set(letter.rows, bx + 1, by);

                // Dark bevels (bottom/right) drawn first
                if has_bottom {
                    fill_rect(img, x, y + cell_h - bevel, cell_w, bevel, bevel_dark);
                }
                if has_right {
                    fill_rect(img, x + cell_w - bevel, y, bevel, cell_h, bevel_dark);
                }
                // Bright bevels (top/left) overwrite corners
                if has_top {
                    fill_rect(img, x, y, cell_w, bevel, pal.title_hi);
                }
                if has_left {
                    fill_rect(img, x, y, bevel, cell_h, pal.title_hi);
                }
            }
        }
        cursor += letter.width + 1;
    }
}

// ---------------------------------------------------------------------------
// Shared content helpers
// ---------------------------------------------------------------------------

fn draw_system_info(
    img: &mut RgbaImage,
    start_row: u32,
    info: &SystemInfo,
    pal: &Palette,
    scale: u32,
) {
    let info_lines: [(&str, String); 6] = [
        ("0P3R4T1NG", info.os.to_uppercase()),
        ("4RCH1T3CT", info.arch.to_uppercase()),
        ("H0STN4M3 ", info.hostname.to_uppercase()),
        ("D4T3     ", info.date.clone()),
        ("L04D     ", info.load.clone()),
        ("M3M0RY   ", info.memory.clone()),
    ];
    for (i, (label, value)) in info_lines.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let row = start_row + i as u32;
        let mut line: Vec<u8> = vec![b' ', b' ', BULLET, b' '];
        line.extend_from_slice(label.as_bytes());
        line.extend_from_slice(b"  ");
        #[allow(clippy::cast_possible_truncation)]
        let val_col = 2 + line.len() as u32;
        draw_bytes(img, 2, row, &line, pal.label_fg, pal.bg, scale);

        let val_text = format!("[ {value} ]");
        draw_text(img, val_col, row, &val_text, pal.value_fg, pal.bg, scale);
    }
}

fn draw_tagline_box(
    img: &mut RgbaImage,
    start_row: u32,
    info: &SystemInfo,
    pal: &Palette,
    scale: u32,
) {
    let taglines = [
        "proudly serving the scene since 1993",
        "where the elstrEEt meet",
        "another fine release from the underground",
        "cracked by the best, spread by the rest",
        "the future is now, old man",
        "10 nodes / USR Courier V.Everything",
        "greets to all groups worldwide",
        "the underground never sleeps",
    ];
    let tag_idx = day_of_year() as usize % taglines.len();
    let tagline = taglines[tag_idx];

    let top_label = "Terminal Underground Division";
    let node_label = format!("NODE: {}", info.hostname.to_uppercase());
    // Box width = max of all three rows (prefix + content + suffix)
    let box_w = (6 + top_label.len() + 4)
        .max(7 + tagline.len() + 2)
        .max(6 + node_label.len() + 4);
    let box_top = make_box_top(top_label, box_w);
    let box_mid = make_box_mid(tagline, box_w);
    let box_bot = make_box_bot(&node_label, box_w);
    draw_bytes(img, 0, start_row, &box_top, pal.border_fg, pal.bg, scale);
    draw_bytes(
        img,
        0,
        start_row + 1,
        &box_mid,
        pal.label_fg,
        pal.bg,
        scale,
    );
    draw_bytes(
        img,
        0,
        start_row + 2,
        &box_bot,
        pal.border_fg,
        pal.bg,
        scale,
    );
}

// ---------------------------------------------------------------------------
// Banner layouts
// ---------------------------------------------------------------------------

fn draw_classic(img: &mut RgbaImage, pal: &Palette, scale: u32) {
    let info = SystemInfo::gather();

    // Row 1: gradient bar ░▒▓█
    draw_gradient_bar(img, 1, &pal.gradient, false, scale);

    // Rows 3-6: "TERMINAL UNDERGROUND" header with shadow
    let title1 = "TERMINAL";
    let title2 = "UNDERGROUND";
    #[allow(clippy::cast_possible_truncation)]
    let t1_col = (COLS - title1.len() as u32) / 2;
    #[allow(clippy::cast_possible_truncation)]
    let t2_col = (COLS - title2.len() as u32) / 2;
    draw_block_text(img, t1_col, 3, title1, pal.header_fg, pal.bg, scale);
    draw_block_text(img, t2_col, 5, title2, pal.header_fg, pal.bg, scale);

    // Row 8: divider ────◆────
    let div = make_divider_bytes(COLS);
    draw_bytes(img, 0, 8, &div, pal.border_fg, pal.bg, scale);

    // Rows 10-15: system info
    draw_system_info(img, 10, &info, pal, scale);

    // Rows 17-19: tagline box
    draw_tagline_box(img, 17, &info, pal, scale);

    // Row 23: reverse gradient bar █▓▒░
    draw_gradient_bar(img, 23, &pal.gradient, true, scale);
}

fn draw_block3d(img: &mut RgbaImage, pal: &Palette, scale: u32) {
    let info = SystemInfo::gather();

    // Row 1: gradient bar ░▒▓█
    draw_gradient_bar(img, 1, &pal.gradient, false, scale);

    // Row 2: ornament ═══╡ ◆ ╞═══
    let ornament = make_ornament_line(COLS);
    draw_bytes(img, 0, 2, &ornament, pal.border_fg, pal.bg, scale);

    // Rows 3-9: big "INFLUX" 3D block letters (extrusion bleeds below)
    draw_big_title(img, 3, "INFLUX", pal, scale);

    // Row 11: double divider ═══◆═══
    let div = make_double_divider(COLS);
    draw_bytes(img, 0, 11, &div, pal.border_fg, pal.bg, scale);

    // Rows 12-17: system info
    draw_system_info(img, 12, &info, pal, scale);

    // Rows 19-21: tagline box
    draw_tagline_box(img, 19, &info, pal, scale);

    // Row 23: reverse gradient bar █▓▒░
    draw_gradient_bar(img, 23, &pal.gradient, true, scale);
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generates the banner PNG and writes it to stdout.
pub fn generate(scale: u32, palette_name: &str, banner_type: Option<&str>) {
    let pal = palette_by_name(palette_name);
    let width = COLS * GLYPH_W * scale;
    let height = ROWS * GLYPH_H * scale;
    let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, pal.bg);

    let resolved = banner_type.unwrap_or_else(|| {
        if day_of_year().is_multiple_of(2) {
            "block3d"
        } else {
            "classic"
        }
    });

    match resolved {
        "block3d" => draw_block3d(&mut img, &pal, scale),
        _ => draw_classic(&mut img, &pal, scale),
    }

    // Encode to PNG and write to stdout
    let mut buf = io::BufWriter::new(io::stdout().lock());
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("failed to write PNG");
    buf.flush().expect("failed to flush stdout");
}
