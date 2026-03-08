use std::fmt::Write;

use crate::color::{bg, fg, ARROW};

#[must_use]
pub fn render_with(from_bg: u8) -> (String, u8) {
    let in_nix = std::env::var("IN_NIX_SHELL").unwrap_or_default();
    if in_nix.is_empty() {
        return (String::new(), from_bg);
    }

    let mut out = String::with_capacity(128);
    let _ = write!(
        out,
        "{}{}{} {}❄ nix {}{}{}",
        fg(from_bg),
        bg(68),
        ARROW,
        fg(15),
        fg(68),
        bg(237),
        ARROW
    );
    (out, 237)
}

#[must_use]
pub fn render() -> String {
    render_with(237).0
}

#[cfg(test)]
mod tests {
    use super::{render, render_with};
    use crate::color::fg;

    #[test]
    fn empty_when_unset() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };
        assert_eq!(render(), "");
    }

    #[test]
    fn renders_segment_when_set() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::set_var("IN_NIX_SHELL", "impure") };
        let out = render();
        assert!(out.contains("nix"), "expected 'nix' in: {out}");
        assert!(out.contains('❄'), "expected snowflake in: {out}");
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };
    }

    #[test]
    fn render_with_passthrough_when_unset() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };
        let (out, end_bg) = render_with(238);
        assert_eq!(out, "");
        assert_eq!(end_bg, 238);
    }

    #[test]
    fn render_with_uses_from_bg() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::set_var("IN_NIX_SHELL", "impure") };
        let (out, end_bg) = render_with(238);
        assert!(out.contains(&fg(238)), "expected fg(238) in: {out}");
        assert_eq!(end_bg, 237);
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };
    }
}
