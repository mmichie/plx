use std::fmt::Write;
use std::path::PathBuf;

use crate::color::{arrow, fg};
use crate::segments::prompt::PromptContext;
use crate::segments::registry::{Segment, SegmentOutput};

const MY_BG: u8 = 27;
const MY_FG: u8 = 15;
const ICON: &str = "\u{2388}"; // Helm symbol

pub struct K8sSegment;

impl Segment for K8sSegment {
    fn name(&self) -> &'static str {
        "k8s"
    }

    fn render(&self, _ctx: &mut PromptContext, from_bg: Option<u8>) -> SegmentOutput {
        let Some(context) = read_kube_context() else {
            return SegmentOutput {
                text: String::new(),
                end_bg: from_bg,
            };
        };

        let mut out = String::with_capacity(128);
        let _ = write!(
            out,
            "{} {}{ICON} {context} ",
            arrow(from_bg, MY_BG),
            fg(MY_FG),
        );
        SegmentOutput {
            text: out,
            end_bg: Some(MY_BG),
        }
    }
}

fn kubeconfig_path() -> PathBuf {
    if let Ok(val) = std::env::var("KUBECONFIG") {
        // KUBECONFIG can be colon-separated; use the first entry
        let first = val.split(':').next().unwrap_or(&val);
        return PathBuf::from(first);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".kube").join("config")
}

fn read_kube_context() -> Option<String> {
    let contents = std::fs::read_to_string(kubeconfig_path()).ok()?;
    for line in contents.lines() {
        if let Some(ctx) = line.trim().strip_prefix("current-context:") {
            let ctx = ctx.trim().trim_matches('"');
            if !ctx.is_empty() {
                return Some(ctx.to_string());
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
    #[serial]
    fn no_kubeconfig_returns_none() {
        let tmp = TempDir::new().unwrap();
        let fake = tmp.path().join("nonexistent");
        unsafe { std::env::set_var("KUBECONFIG", fake.to_str().unwrap()) };
        assert!(read_kube_context().is_none());
        unsafe { std::env::remove_var("KUBECONFIG") };
    }

    #[test]
    #[serial]
    fn reads_current_context() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config");
        std::fs::write(
            &config,
            "apiVersion: v1\ncurrent-context: my-cluster\nkind: Config\n",
        )
        .unwrap();
        unsafe { std::env::set_var("KUBECONFIG", config.to_str().unwrap()) };
        assert_eq!(read_kube_context().unwrap(), "my-cluster");
        unsafe { std::env::remove_var("KUBECONFIG") };
    }

    #[test]
    #[serial]
    fn reads_quoted_context() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config");
        std::fs::write(
            &config,
            "current-context: \"arn:aws:eks:us-east-1:123:cluster/prod\"\n",
        )
        .unwrap();
        unsafe { std::env::set_var("KUBECONFIG", config.to_str().unwrap()) };
        assert_eq!(
            read_kube_context().unwrap(),
            "arn:aws:eks:us-east-1:123:cluster/prod"
        );
        unsafe { std::env::remove_var("KUBECONFIG") };
    }

    #[test]
    #[serial]
    fn empty_context_returns_none() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config");
        std::fs::write(&config, "current-context:\n").unwrap();
        unsafe { std::env::set_var("KUBECONFIG", config.to_str().unwrap()) };
        assert!(read_kube_context().is_none());
        unsafe { std::env::remove_var("KUBECONFIG") };
    }
}
