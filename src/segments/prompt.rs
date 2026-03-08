use git2::Repository;

use crate::segments::{git, nix_shell, path};

pub struct PromptContext {
    pub home: String,
    pub pwd: String,
    pub max_dir_size: Option<usize>,
    pub repo: Option<Repository>,
}

impl PromptContext {
    #[must_use]
    pub fn gather(max_dir_size: Option<usize>) -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let pwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let repo = Repository::discover(".").ok();
        Self {
            home,
            pwd,
            max_dir_size,
            repo,
        }
    }
}

#[must_use]
pub fn render(ctx: &mut PromptContext) -> String {
    let mut from_bg: u8 = 238;
    let mut out = String::with_capacity(1024);

    let (seg, next_bg) = nix_shell::render_with(from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = path::render_with(&ctx.home, &ctx.pwd, ctx.max_dir_size, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, _) = git::render_with(ctx.repo.as_mut(), from_bg);
    out.push_str(&seg);

    out
}

#[cfg(test)]
mod tests {
    use super::{render, PromptContext};
    use crate::color::{ARROW, BRANCH_ICON, RST};
    use crate::segments::testutil::init_repo;
    use tempfile::TempDir;

    #[test]
    fn renders_path_and_git() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: tmp.path().to_string_lossy().to_string(),
            max_dir_size: None,
            repo: Some(repo),
        };

        let out = render(&mut ctx);
        assert!(out.contains(ARROW), "expected arrows in: {out}");
        assert!(out.contains(BRANCH_ICON), "expected branch icon in: {out}");
    }

    #[test]
    fn renders_without_repo() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: "/tmp".to_string(),
            max_dir_size: None,
            repo: None,
        };

        let out = render(&mut ctx);
        assert!(out.contains(ARROW), "expected arrows in: {out}");
        assert!(!out.contains(BRANCH_ICON), "should not contain branch icon");
    }

    #[test]
    fn full_chain_ends_with_reset() {
        // SAFETY: test-only, single-threaded test runner
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: "/home/user/projects".to_string(),
            max_dir_size: Some(20),
            repo: None,
        };

        let out = render(&mut ctx);
        assert!(out.ends_with(RST), "should end with reset: {out}");
    }
}
