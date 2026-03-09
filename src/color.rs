pub const ARROW: &str = "\u{E0B0}";
pub const THIN: &str = "\u{E0B1}";
pub const BRANCH_ICON: &str = "\u{E0A0}";
pub const PENCIL_ICON: &str = "\u{F040}";
pub const RST: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const UNBOLD: &str = "\x1b[22m";

/// Powerline arrow transition. `None` means first segment (no arrow glyph).
#[must_use]
pub fn arrow(from_bg: Option<u8>, to_bg: u8) -> String {
    if let Some(prev) = from_bg {
        format!("{}{}{ARROW}", fg(prev), bg(to_bg))
    } else {
        bg(to_bg)
    }
}

#[must_use]
pub fn fg(color: u8) -> String {
    format!("\x1b[38;5;{color}m")
}

#[must_use]
pub fn bg(color: u8) -> String {
    format!("\x1b[48;5;{color}m")
}

#[cfg(test)]
mod tests {
    use super::{bg, fg};

    #[test]
    fn fg_produces_ansi_color() {
        assert_eq!(fg(31), "\x1b[38;5;31m");
        assert_eq!(fg(0), "\x1b[38;5;0m");
        assert_eq!(fg(255), "\x1b[38;5;255m");
    }

    #[test]
    fn bg_produces_ansi_color() {
        assert_eq!(bg(31), "\x1b[48;5;31m");
        assert_eq!(bg(0), "\x1b[48;5;0m");
    }
}
