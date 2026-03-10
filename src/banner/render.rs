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
const BULLET: u8 = 0xF9; // ∙
const DIAMOND: u8 = 0x04; // ♦

struct Palette {
    bg: Rgba<u8>,
    header_fg: Rgba<u8>,
    border_fg: Rgba<u8>,
    label_fg: Rgba<u8>,
    value_fg: Rgba<u8>,
    gradient: [Rgba<u8>; 4],
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
        },
    }
}

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

fn make_divider_bytes(width: u32) -> Vec<u8> {
    let w = width as usize;
    let half = (w.saturating_sub(1)) / 2;
    let mut out = vec![BOX_HZ; half];
    out.push(DIAMOND);
    out.resize(w, BOX_HZ);
    out
}

fn make_box_top(label: &str) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_TL, BOX_DB, BOX_DB, b' '];
    out.extend_from_slice(label.as_bytes());
    out.extend_from_slice(&[b' ', BOX_DB, BOX_DB, BOX_TR]);
    out
}

fn make_box_mid(content: &str) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_VT, b' ', b' ', BULLET, b' '];
    out.extend_from_slice(content.as_bytes());
    out
}

fn make_box_bot(label: &str) -> Vec<u8> {
    let mut out = vec![b' ', b' ', BOX_BL, BOX_DB, BOX_DB, b' '];
    out.extend_from_slice(label.as_bytes());
    out.extend_from_slice(&[b' ', BOX_DB, BOX_DB, BOX_BR]);
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

/// Generates the banner PNG and writes it to stdout.
pub fn generate(scale: u32, palette_name: &str) {
    let pal = palette_by_name(palette_name);
    let width = COLS * GLYPH_W * scale;
    let height = ROWS * GLYPH_H * scale;
    let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, pal.bg);

    let info = SystemInfo::gather();

    // Row 1: gradient bar ░▒▓█
    draw_gradient_bar(&mut img, 1, &pal.gradient, false, scale);

    // Rows 3-6: "TERMINAL UNDERGROUND" header with shadow
    let title1 = "TERMINAL";
    let title2 = "UNDERGROUND";
    #[allow(clippy::cast_possible_truncation)]
    let t1_col = (COLS - title1.len() as u32) / 2;
    #[allow(clippy::cast_possible_truncation)]
    let t2_col = (COLS - title2.len() as u32) / 2;
    draw_block_text(&mut img, t1_col, 3, title1, pal.header_fg, pal.bg, scale);
    draw_block_text(&mut img, t2_col, 5, title2, pal.header_fg, pal.bg, scale);

    // Row 8: divider ────◆────
    let div = make_divider_bytes(COLS);
    draw_bytes(&mut img, 0, 8, &div, pal.border_fg, pal.bg, scale);

    // Rows 10-15: system info
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
        let row = 10 + i as u32;
        let mut line: Vec<u8> = vec![b' ', b' ', BULLET, b' '];
        line.extend_from_slice(label.as_bytes());
        line.extend_from_slice(b"  ");
        #[allow(clippy::cast_possible_truncation)]
        let val_col = 2 + line.len() as u32;
        draw_bytes(&mut img, 2, row, &line, pal.label_fg, pal.bg, scale);

        let val_text = format!("[ {value} ]");
        draw_text(&mut img, val_col, row, &val_text, pal.value_fg, pal.bg, scale);
    }

    // Rows 17-19: tagline box
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

    let box_top = make_box_top("Terminal Underground Division");
    let box_mid = make_box_mid(tagline);
    let node_label = format!("NODE: {}", info.hostname.to_uppercase());
    let box_bot = make_box_bot(&node_label);
    draw_bytes(&mut img, 0, 17, &box_top, pal.border_fg, pal.bg, scale);
    draw_bytes(&mut img, 0, 18, &box_mid, pal.label_fg, pal.bg, scale);
    draw_bytes(&mut img, 0, 19, &box_bot, pal.border_fg, pal.bg, scale);

    // Row 23: reverse gradient bar █▓▒░
    draw_gradient_bar(&mut img, 23, &pal.gradient, true, scale);

    // Encode to PNG and write to stdout
    let mut buf = io::BufWriter::new(io::stdout().lock());
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("failed to write PNG");
    buf.flush().expect("failed to flush stdout");
}
