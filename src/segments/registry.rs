use crate::config::Config;
use crate::segments::custom_command::CustomCommandSegment;
use crate::segments::k8s::K8sSegment;
use crate::segments::node::NodeSegment;
use crate::segments::prompt::PromptContext;
use crate::segments::python::PythonSegment;
use crate::segments::rust_toolchain::RustToolchainSegment;
use crate::segments::{
    aws, character, cmd_duration, git, hostname, jobs, nix_shell, path, status, username, venv,
};

/// Output from a single segment render.
pub struct SegmentOutput {
    /// The ANSI-formatted string to append to the prompt.
    pub text: String,
    /// The background color after this segment, used by the next segment's arrow.
    pub end_bg: Option<u8>,
}

/// Every prompt segment implements this trait.
pub trait Segment {
    /// Unique identifier used in config keys (e.g. "username", "git", "k8s").
    fn name(&self) -> &'static str;

    /// Render this segment given the shared prompt context and the previous
    /// segment's ending background color.
    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput;
}

pub struct VenvSegment;

impl Segment for VenvSegment {
    fn name(&self) -> &'static str {
        "venv"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        SegmentOutput {
            text: venv::render_prefix(),
            end_bg: from_bg,
        }
    }
}

pub struct UsernameSegment;

impl Segment for UsernameSegment {
    fn name(&self) -> &'static str {
        "username"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = username::render_with(from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct HostnameSegment;

impl Segment for HostnameSegment {
    fn name(&self) -> &'static str {
        "hostname"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = hostname::render_with(from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct NixShellSegment;

impl Segment for NixShellSegment {
    fn name(&self) -> &'static str {
        "nix_shell"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = nix_shell::render_with(from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct AwsSegment;

impl Segment for AwsSegment {
    fn name(&self) -> &'static str {
        "aws"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = aws::render_with(from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct PathSegment;

impl Segment for PathSegment {
    fn name(&self) -> &'static str {
        "path"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = path::render_with(&ctx.home, &ctx.pwd, ctx.max_dir_size, from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct GitSegment;

impl Segment for GitSegment {
    fn name(&self) -> &'static str {
        "git"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg, info) = git::render_with(ctx.repo.as_mut(), from_bg);
        ctx.git_info = info;
        SegmentOutput { text, end_bg }
    }
}

pub struct StatusSegment;

impl Segment for StatusSegment {
    fn name(&self) -> &'static str {
        "status"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = status::render_with(ctx.exit_status, from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct CmdDurationSegment;

impl Segment for CmdDurationSegment {
    fn name(&self) -> &'static str {
        "cmd_duration"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = cmd_duration::render_with(ctx.duration_ms, from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct JobsSegment;

impl Segment for JobsSegment {
    fn name(&self) -> &'static str {
        "jobs"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = jobs::render_with(ctx.job_count, from_bg);
        SegmentOutput { text, end_bg }
    }
}

pub struct CharacterSegment;

impl Segment for CharacterSegment {
    fn name(&self) -> &'static str {
        "character"
    }

    fn render(&self, ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let (text, end_bg) = character::render_with(ctx.exit_status == 0, from_bg);
        SegmentOutput { text, end_bg }
    }
}

/// Map a segment name from config to its implementation.
fn segment_by_name(name: &str) -> Option<Box<dyn Segment>> {
    match name {
        "venv" => Some(Box::new(VenvSegment)),
        "username" => Some(Box::new(UsernameSegment)),
        "hostname" => Some(Box::new(HostnameSegment)),
        "nix_shell" => Some(Box::new(NixShellSegment)),
        "aws" => Some(Box::new(AwsSegment)),
        "k8s" => Some(Box::new(K8sSegment)),
        "path" => Some(Box::new(PathSegment)),
        "git" => Some(Box::new(GitSegment)),
        "node" => Some(Box::new(NodeSegment)),
        "python" => Some(Box::new(PythonSegment)),
        "rust_toolchain" => Some(Box::new(RustToolchainSegment)),
        "status" => Some(Box::new(StatusSegment)),
        "cmd_duration" => Some(Box::new(CmdDurationSegment)),
        "jobs" => Some(Box::new(JobsSegment)),
        "character" => Some(Box::new(CharacterSegment)),
        "custom_command" => Some(Box::new(CustomCommandSegment)),
        _ => None,
    }
}

/// Returns the default ordered list of segments matching the original hardcoded chain.
#[must_use]
pub fn default_segments() -> Vec<Box<dyn Segment>> {
    vec![
        Box::new(VenvSegment),
        Box::new(UsernameSegment),
        Box::new(HostnameSegment),
        Box::new(NixShellSegment),
        Box::new(AwsSegment),
        Box::new(PathSegment),
        Box::new(GitSegment),
        Box::new(StatusSegment),
        Box::new(CmdDurationSegment),
        Box::new(JobsSegment),
        Box::new(CharacterSegment),
    ]
}

/// Build the segment list according to the loaded config.
/// If no custom order is specified, uses the default order.
/// Segments explicitly disabled via `enabled = false` are excluded.
#[must_use]
pub fn build_segments(config: &Config) -> Vec<Box<dyn Segment>> {
    if config.segments.order.is_empty() {
        return default_segments()
            .into_iter()
            .filter(|s| config.segment_enabled(s.name()))
            .collect();
    }

    config
        .segments
        .order
        .iter()
        .filter(|name| config.segment_enabled(name))
        .filter_map(|name| segment_by_name(name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{build_segments, default_segments};
    use crate::config::Config;

    #[test]
    fn default_order_has_all_segments() {
        let segs = default_segments();
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            [
                "venv",
                "username",
                "hostname",
                "nix_shell",
                "aws",
                "path",
                "git",
                "status",
                "cmd_duration",
                "jobs",
                "character",
            ]
        );
    }

    #[test]
    fn build_with_default_config_matches_default_order() {
        let cfg = Config::default();
        let segs = build_segments(&cfg);
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        let defaults: Vec<&str> = default_segments().iter().map(|s| s.name()).collect();
        assert_eq!(names, defaults);
    }

    #[test]
    fn build_with_custom_order() {
        let toml = r#"
[segments]
order = ["path", "git", "character"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let segs = build_segments(&cfg);
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["path", "git", "character"]);
    }

    #[test]
    fn build_filters_disabled_segments() {
        let toml = r"
[segment.hostname]
enabled = false
";
        let cfg: Config = toml::from_str(toml).unwrap();
        let segs = build_segments(&cfg);
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        assert!(!names.contains(&"hostname"));
        assert!(names.contains(&"username"));
        assert!(names.contains(&"git"));
    }

    #[test]
    fn build_custom_order_skips_disabled() {
        let toml = r#"
[segments]
order = ["path", "git", "hostname", "character"]

[segment.hostname]
enabled = false
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let segs = build_segments(&cfg);
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["path", "git", "character"]);
    }

    #[test]
    fn build_ignores_unknown_segment_names() {
        let toml = r#"
[segments]
order = ["path", "nonexistent", "git"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let segs = build_segments(&cfg);
        let names: Vec<&str> = segs.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["path", "git"]);
    }
}
