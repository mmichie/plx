use std::fmt::Write;

use crate::color::{arrow, fg};
use crate::segments::find_ancestor_file;
use crate::segments::prompt::PromptContext;
use crate::segments::registry::{Segment, SegmentOutput};

const MY_BG: u8 = 208;
const MY_FG: u8 = 0;
const ICON: &str = "\u{E7A8}"; // Nerd Font Rust icon

pub struct RustToolchainSegment;

impl Segment for RustToolchainSegment {
    fn name(&self) -> &'static str {
        "rust_toolchain"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let empty = SegmentOutput {
            text: String::new(),
            end_bg: from_bg,
        };

        // Only show in Rust projects
        if find_ancestor_file(&ctx.pwd, "Cargo.toml", 5).is_none() {
            return empty;
        }

        let Some(toolchain) = read_rust_toolchain(&ctx.pwd) else {
            return empty;
        };

        let mut out = String::with_capacity(128);
        let _ = write!(
            out,
            "{} {}{ICON} {toolchain} ",
            arrow(from_bg, MY_BG),
            fg(MY_FG),
        );
        SegmentOutput {
            text: out,
            end_bg: Some(MY_BG),
        }
    }
}

fn read_rust_toolchain(pwd: &str) -> Option<String> {
    // rust-toolchain.toml (modern format, has [toolchain]\nchannel = "...")
    if let Some(path) = find_ancestor_file(pwd, "rust-toolchain.toml", 5)
        && let Ok(contents) = std::fs::read_to_string(path)
        && let Some(channel) = parse_toolchain_toml(&contents)
    {
        return Some(channel);
    }

    // rust-toolchain (legacy format, just the channel name)
    if let Some(path) = find_ancestor_file(pwd, "rust-toolchain", 5)
        && let Ok(contents) = std::fs::read_to_string(path)
    {
        let trimmed = contents.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('[') {
            return Some(trimmed.to_string());
        }
    }

    // RUSTUP_TOOLCHAIN env var
    std::env::var("RUSTUP_TOOLCHAIN")
        .ok()
        .filter(|v| !v.is_empty())
}

/// Extract channel from a `rust-toolchain.toml` file without a TOML parser.
/// Looks for `channel = "..."` under `[toolchain]`.
fn parse_toolchain_toml(contents: &str) -> Option<String> {
    let mut in_toolchain_section = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_toolchain_section = trimmed == "[toolchain]";
            continue;
        }
        if in_toolchain_section && let Some(val) = trimmed.strip_prefix("channel") {
            let val = val.trim_start_matches([' ', '=']);
            let val = val.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn parse_toolchain_toml_extracts_channel() {
        let contents = "[toolchain]\nchannel = \"nightly-2024-01-01\"\n";
        assert_eq!(
            parse_toolchain_toml(contents).unwrap(),
            "nightly-2024-01-01"
        );
    }

    #[test]
    fn parse_toolchain_toml_stable() {
        let contents = "[toolchain]\nchannel = \"stable\"\ncomponents = [\"rustfmt\"]\n";
        assert_eq!(parse_toolchain_toml(contents).unwrap(), "stable");
    }

    #[test]
    fn parse_toolchain_toml_ignores_other_sections() {
        let contents = "[other]\nchannel = \"wrong\"\n\n[toolchain]\nchannel = \"nightly\"\n";
        assert_eq!(parse_toolchain_toml(contents).unwrap(), "nightly");
    }

    #[test]
    fn parse_toolchain_toml_missing_channel() {
        let contents = "[toolchain]\ncomponents = [\"rustfmt\"]\n";
        assert!(parse_toolchain_toml(contents).is_none());
    }

    #[test]
    fn reads_toolchain_toml_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"stable\"\n",
        )
        .unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        assert_eq!(read_rust_toolchain(&pwd).unwrap(), "stable");
    }

    #[test]
    fn reads_legacy_rust_toolchain_file() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("rust-toolchain"), "nightly\n").unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        assert_eq!(read_rust_toolchain(&pwd).unwrap(), "nightly");
    }

    #[test]
    fn toolchain_toml_takes_precedence() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"stable\"\n",
        )
        .unwrap();
        std::fs::write(tmp.path().join("rust-toolchain"), "nightly\n").unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        assert_eq!(read_rust_toolchain(&pwd).unwrap(), "stable");
    }

    #[test]
    #[serial]
    fn reads_rustup_toolchain_env() {
        let tmp = TempDir::new().unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        unsafe { std::env::set_var("RUSTUP_TOOLCHAIN", "beta") };
        assert_eq!(read_rust_toolchain(&pwd).unwrap(), "beta");
        unsafe { std::env::remove_var("RUSTUP_TOOLCHAIN") };
    }

    #[test]
    #[serial]
    fn no_toolchain_returns_none() {
        let tmp = TempDir::new().unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        unsafe { std::env::remove_var("RUSTUP_TOOLCHAIN") };
        assert!(read_rust_toolchain(&pwd).is_none());
    }
}
