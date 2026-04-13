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

/// Wrap ANSI escape sequences in `%{...%}` so zsh can calculate visible prompt width.
#[must_use]
pub fn zsh_wrap_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 4);
    let mut parts = s.split('\x1b');

    if let Some(first) = parts.next() {
        out.push_str(first);
    }

    for part in parts {
        if let Some(m_pos) = part.find('m') {
            out.push_str("%{\x1b");
            out.push_str(&part[..=m_pos]);
            out.push_str("%}");
            out.push_str(&part[m_pos + 1..]);
        } else {
            out.push('\x1b');
            out.push_str(part);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{arrow, bg, fg, zsh_wrap_escapes, ARROW};

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

    #[test]
    fn zsh_wrap_no_escapes() {
        assert_eq!(zsh_wrap_escapes("hello"), "hello");
    }

    #[test]
    fn zsh_wrap_single_escape() {
        let input = format!("{}text", fg(31));
        let wrapped = zsh_wrap_escapes(&input);
        assert_eq!(wrapped, "%{\x1b[38;5;31m%}text");
    }

    #[test]
    fn zsh_wrap_multiple_escapes() {
        let input = format!("{}hello{}world", fg(31), bg(236));
        let wrapped = zsh_wrap_escapes(&input);
        assert_eq!(wrapped, "%{\x1b[38;5;31m%}hello%{\x1b[48;5;236m%}world");
    }

    #[test]
    fn zsh_wrap_preserves_visible_text() {
        let input = format!("{} $ {}", fg(15), fg(9));
        let wrapped = zsh_wrap_escapes(&input);
        assert!(wrapped.contains(" $ "));
        assert!(wrapped.contains("%{"));
        assert!(wrapped.contains("%}"));
    }

    #[test]
    fn arrow_with_from_bg_includes_glyph() {
        let out = arrow(Some(237), 148);
        assert!(out.contains(&fg(237)));
        assert!(out.contains(&bg(148)));
        assert!(out.contains(ARROW));
    }

    #[test]
    fn arrow_without_from_bg_is_just_bg() {
        let out = arrow(None, 31);
        assert_eq!(out, bg(31));
        assert!(!out.contains(ARROW));
    }
}
