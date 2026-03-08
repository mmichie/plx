use std::fmt::Write;

use crate::color::{bg, fg, ARROW};

#[must_use]
pub fn render() -> String {
    let in_nix = std::env::var("IN_NIX_SHELL").unwrap_or_default();
    if in_nix.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(128);
    let _ = write!(
        out,
        "{}{}{} {}❄ nix {}{}{}",
        fg(237),
        bg(68),
        ARROW,
        fg(15),
        fg(68),
        bg(237),
        ARROW
    );
    out
}

#[cfg(test)]
mod tests {
    use super::render;

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
}
