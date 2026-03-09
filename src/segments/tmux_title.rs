use git2::{Repository, StatusOptions, StatusShow};

use crate::color::{BRANCH_ICON, PENCIL_ICON};
use crate::segments::git::GitInfo;

/// Render tmux title from pre-computed `GitInfo`, avoiding redundant repo discovery.
#[must_use]
pub fn render_from_info(home: &str, pwd: &str, git_info: Option<&GitInfo>) -> String {
    if pwd == home {
        return "\u{1F3E0} ~".to_string();
    }

    let dir_name = std::path::Path::new(pwd)
        .file_name()
        .map_or_else(|| pwd.to_string(), |n| n.to_string_lossy().to_string());

    let Some(info) = git_info else {
        return format!("\u{1F4C1} {dir_name}");
    };

    if info.dirty {
        format!(
            "#[fg=colour174]{BRANCH_ICON}#[default] {} {} #[fg=colour245]{PENCIL_ICON}#[default]",
            info.repo_name, info.branch
        )
    } else {
        format!(
            "#[fg=colour117]{BRANCH_ICON}#[default] {} {}",
            info.repo_name, info.branch
        )
    }
}

#[must_use]
pub fn render(home: &str, pwd: &str) -> String {
    if pwd == home {
        return "\u{1F3E0} ~".to_string();
    }

    let dir_name = std::path::Path::new(pwd)
        .file_name()
        .map_or_else(|| pwd.to_string(), |n| n.to_string_lossy().to_string());

    // Try to open git repo
    let Ok(repo) = Repository::discover(pwd) else {
        return format!("\u{1F4C1} {dir_name}");
    };

    // Get branch name
    let branch = if repo.head_detached().unwrap_or(false) {
        repo.head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .map_or_else(
                || "HEAD".to_string(),
                |c| c.id().to_string()[..7].to_string(),
            )
    } else {
        repo.head()
            .ok()
            .and_then(|h| h.shorthand().map(str::to_string))
            .unwrap_or_else(|| "HEAD".to_string())
    };

    // Get repo name from workdir
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .map_or(dir_name, |n| n.to_string_lossy().to_string());

    // Check if dirty (any status entries = dirty)
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);

    let dirty = repo
        .statuses(Some(&mut opts))
        .map(|statuses| !statuses.is_empty())
        .unwrap_or(false);

    if dirty {
        format!(
            "#[fg=colour174]{BRANCH_ICON}#[default] {repo_name} {branch} #[fg=colour245]{PENCIL_ICON}#[default]"
        )
    } else {
        format!("#[fg=colour117]{BRANCH_ICON}#[default] {repo_name} {branch}")
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::color::{BRANCH_ICON, PENCIL_ICON};
    use crate::segments::testutil::init_repo;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn home_directory() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_string_lossy().to_string();
        let out = render(&home, &home);
        assert!(out.contains('\u{1F3E0}'), "expected house emoji");
        assert!(out.contains('~'));
    }

    #[test]
    fn non_repo_directory() {
        let tmp = TempDir::new().unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        let out = render("/nonexistent", &pwd);
        assert!(out.contains('\u{1F4C1}'), "expected folder emoji");
    }

    #[test]
    fn clean_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        let pwd = tmp.path().to_string_lossy().to_string();

        let out = render("/nonexistent", &pwd);
        assert!(
            out.contains("#[fg=colour117]"),
            "expected blue branch in: {out}"
        );
        assert!(out.contains(BRANCH_ICON));
        assert!(!out.contains(PENCIL_ICON), "clean repo should not have pencil");
    }

    #[test]
    fn dirty_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        fs::write(tmp.path().join("dirty.txt"), "x").unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();

        let out = render("/nonexistent", &pwd);
        assert!(
            out.contains("#[fg=colour174]"),
            "expected pink branch in: {out}"
        );
        assert!(out.contains(PENCIL_ICON), "dirty repo should have pencil");
    }
}
