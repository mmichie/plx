use git2::Repository;

use crate::segments::{
    character, cmd_duration, git, hostname, jobs, nix_shell, path, reset, status, username, venv,
};

pub struct PromptContext {
    pub home: String,
    pub pwd: String,
    pub max_dir_size: Option<usize>,
    pub repo: Option<Repository>,
    pub exit_status: i32,
    pub duration_ms: u64,
    pub job_count: u32,
}

impl PromptContext {
    #[must_use]
    pub fn gather(
        max_dir_size: Option<usize>,
        exit_status: i32,
        duration_ms: u64,
        job_count: u32,
    ) -> Self {
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
            exit_status,
            duration_ms,
            job_count,
        }
    }
}

#[must_use]
pub fn render(ctx: &mut PromptContext) -> String {
    let mut out = String::with_capacity(1024);

    out.push_str(&venv::render_prefix());

    let (seg, mut from_bg) = username::render_with(None);
    out.push_str(&seg);

    let (seg, next_bg) = hostname::render_with(from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = nix_shell::render_with(from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = path::render_with(&ctx.home, &ctx.pwd, ctx.max_dir_size, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = git::render_with(ctx.repo.as_mut(), from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = status::render_with(ctx.exit_status, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = cmd_duration::render_with(ctx.duration_ms, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = jobs::render_with(ctx.job_count, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    let (seg, next_bg) = character::render_with(ctx.exit_status == 0, from_bg);
    out.push_str(&seg);
    from_bg = next_bg;

    out.push_str(&reset::render_final(from_bg));

    out
}

#[cfg(test)]
mod tests {
    use super::{render, PromptContext};
    use crate::color::{ARROW, BRANCH_ICON, RST};
    use crate::segments::testutil::init_repo;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn renders_path_and_git() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        // SAFETY: test-only
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: tmp.path().to_string_lossy().to_string(),
            max_dir_size: None,
            repo: Some(repo),
            exit_status: 0,
            duration_ms: 0,
            job_count: 0,
        };

        let out = render(&mut ctx);
        assert!(out.contains(ARROW), "expected arrows in: {out}");
        assert!(out.contains(BRANCH_ICON), "expected branch icon in: {out}");
    }

    #[test]
    #[serial]
    fn renders_without_repo() {
        // SAFETY: test-only
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: "/tmp".to_string(),
            max_dir_size: None,
            repo: None,
            exit_status: 0,
            duration_ms: 0,
            job_count: 0,
        };

        let out = render(&mut ctx);
        assert!(out.contains(ARROW), "expected arrows in: {out}");
        assert!(!out.contains(BRANCH_ICON), "should not contain branch icon");
    }

    #[test]
    #[serial]
    fn full_chain_ends_with_reset() {
        // SAFETY: test-only
        unsafe { std::env::remove_var("IN_NIX_SHELL") };

        let mut ctx = PromptContext {
            home: "/home/user".to_string(),
            pwd: "/home/user/projects".to_string(),
            max_dir_size: Some(20),
            repo: None,
            exit_status: 0,
            duration_ms: 0,
            job_count: 0,
        };

        let out = render(&mut ctx);
        assert!(out.ends_with(RST), "should end with reset: {out}");
    }
}
