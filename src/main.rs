use git2::{Repository, StatusOptions, StatusShow};
use std::env;
use std::fmt::Write as FmtWrite;

// Powerline characters
const ARROW: &str = "\u{E0B0}";
const THIN: &str = "\u{E0B1}";
const BRANCH_ICON: &str = "\u{E0A0}";

// ANSI color helpers
fn fg(color: u8) -> String {
    format!("\x1b[38;5;{color}m")
}

fn bg(color: u8) -> String {
    format!("\x1b[48;5;{color}m")
}

const RST: &str = "\x1b[0m";

fn render_path(home: &str, pwd: &str) -> String {
    // Replace HOME with ~
    let path = if !home.is_empty() && pwd.starts_with(home) {
        format!("~{}", &pwd[home.len()..])
    } else {
        pwd.to_string()
    };

    // Split into components
    let mut parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Handle root /
    if parts.is_empty() {
        parts = vec!["/"];
    }

    // Truncate to 5 components
    let n = parts.len();
    let truncated;
    let parts = if n > 5 {
        truncated = [&["…"][..], &parts[n - 4..]].concat();
        &truncated
    } else {
        &parts
    };
    let n = parts.len();

    let mut out = String::with_capacity(256);

    if n <= 1 {
        // Single component: hostname(238) -> cwd(31), then transition to 237
        let _ = write!(
            out,
            "{}{}{} {}{} {}{}{}",
            fg(238), bg(31), ARROW,
            fg(15), parts.first().unwrap_or(&""),
            fg(31), bg(237), ARROW
        );
    } else {
        // First component on bg:31 (blue)
        let _ = write!(
            out,
            "{}{}{} {}{} {}{}{}",
            fg(238), bg(31), ARROW,
            fg(15), parts[0],
            fg(31), bg(237), ARROW
        );

        // Remaining components on bg:237
        for (i, part) in parts.iter().enumerate().skip(1) {
            if i > 1 {
                let _ = write!(out, " {}{THIN}", fg(245));
            }
            let _ = write!(out, " {}{part}", fg(254));
        }
        let _ = write!(out, " ");
    }

    out
}

#[allow(clippy::too_many_lines)]
fn render_git(discover_from: &std::path::Path) -> String {
    let Ok(mut repo) = Repository::discover(discover_from) else {
        // Not in a git repo — just output the closing arrow (dir_end)
        return format!("{}{}{ARROW}{RST}", fg(237), bg(236));
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

    // Get file status counts
    let mut staged = 0u32;
    let mut modified = 0u32;
    let mut untracked = 0u32;
    let mut conflicted = 0u32;

    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir);
    opts.include_untracked(true);

    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        for entry in statuses.iter() {
            let s = entry.status();
            if s.is_conflicted() {
                conflicted += 1;
            } else if s.is_index_new()
                || s.is_index_modified()
                || s.is_index_deleted()
                || s.is_index_renamed()
                || s.is_index_typechange()
            {
                staged += 1;
                if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_typechange() {
                    modified += 1;
                }
                if s.is_wt_new() {
                    untracked += 1;
                }
            } else {
                if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_typechange() {
                    modified += 1;
                }
                if s.is_wt_new() {
                    untracked += 1;
                }
            }
        }
    }

    // Ahead/behind
    let (ahead, behind) = get_ahead_behind(&repo);

    // Stash count
    let mut stashed = 0u32;
    let _ = repo.stash_foreach(|_, _, _| {
        stashed += 1;
        true
    });

    // Git state
    let state = match repo.state() {
        git2::RepositoryState::Rebase
        | git2::RepositoryState::RebaseInteractive
        | git2::RepositoryState::RebaseMerge => Some("REBASING"),
        git2::RepositoryState::Merge => Some("MERGING"),
        git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
            Some("CHERRY")
        }
        git2::RepositoryState::Bisect => Some("BISECT"),
        _ => None,
    };

    // Determine if dirty
    let dirty = staged + modified + untracked + conflicted + stashed + ahead + behind > 0
        || state.is_some();

    let mut out = String::with_capacity(512);

    if dirty {
        // Pink branch: arrow from path(237) to 161
        let _ = write!(
            out,
            "{}{}{ARROW} {}{BRANCH_ICON} {branch} ",
            fg(237), bg(161),
            fg(15),
        );
        let mut prev: u8 = 161;

        // Git state
        if let Some(st) = state {
            let _ = write!(
                out,
                "{}{}{ARROW} {}{st} ",
                fg(prev), bg(220),
                fg(0),
            );
            prev = 220;
        }

        // Status segments: (bg_color, text)
        let mut segs: Vec<(u8, String)> = Vec::new();
        if ahead > 0 {
            segs.push((240, format!("{ahead}⬆")));
        }
        if behind > 0 {
            segs.push((240, format!("{behind}⬇")));
        }
        if staged > 0 {
            segs.push((22, format!("{staged}✔")));
        }
        if modified > 0 {
            segs.push((130, format!("{modified}✎")));
        }
        if untracked > 0 {
            segs.push((52, format!("{untracked}+")));
        }
        if conflicted > 0 {
            segs.push((9, format!("{conflicted}✼")));
        }
        if stashed > 0 {
            segs.push((20, format!("{stashed}⚑")));
        }

        for (seg_bg, seg_text) in &segs {
            let _ = write!(
                out,
                "{}{}{ARROW} {}{seg_text} ",
                fg(prev), bg(*seg_bg),
                fg(15),
            );
            prev = *seg_bg;
        }

        // Final arrow to terminal bg (236)
        let _ = write!(out, "{}{}{ARROW}{RST}", fg(prev), bg(236));
    } else {
        // Green branch (clean): arrow from path(237) to 148
        let _ = write!(
            out,
            "{}{}{ARROW} {}{BRANCH_ICON} {branch} {}{}{ARROW}{RST}",
            fg(237), bg(148),
            fg(0),
            fg(148), bg(236),
        );
    }

    out
}

fn get_ahead_behind(repo: &Repository) -> (u32, u32) {
    let Ok(head) = repo.head() else {
        return (0, 0);
    };

    let Some(local_oid) = head.target() else {
        return (0, 0);
    };

    // Get upstream
    let Some(branch_name) = head.shorthand() else {
        return (0, 0);
    };

    let Ok(branch) = repo.find_branch(branch_name, git2::BranchType::Local) else {
        return (0, 0);
    };

    let Ok(upstream) = branch.upstream() else {
        return (0, 0);
    };

    let Some(upstream_oid) = upstream.get().target() else {
        return (0, 0);
    };

    repo.graph_ahead_behind(local_oid, upstream_oid)
        .map(|(a, b)| {
            (
                u32::try_from(a).unwrap_or(u32::MAX),
                u32::try_from(b).unwrap_or(u32::MAX),
            )
        })
        .unwrap_or((0, 0))
}

// Font Awesome pencil icon
const PENCIL_ICON: &str = "\u{F040}";

fn render_tmux_title(home: &str, pwd: &str) -> String {
    // Home directory
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
            "#[fg=colour67]{BRANCH_ICON}#[default] {repo_name} {branch} #[fg=colour245]{PENCIL_ICON}#[default]"
        )
    } else {
        format!("#[fg=colour39]{BRANCH_ICON}#[default] {repo_name} {branch}")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("path") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            print!("{}", render_path(&home, &pwd));
        }
        Some("git") => print!("{}", render_git(std::path::Path::new("."))),
        Some("tmux-title") => {
            let home = env::var("HOME").unwrap_or_default();
            let pwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("{}", render_tmux_title(&home, &pwd));
        }
        _ => {
            eprintln!("Usage: plx <path|git|tmux-title>");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::fs;
    use tempfile::TempDir;

    // ── fg / bg helpers ──────────────────────────────────────────

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

    // ── render_path ──────────────────────────────────────────────

    #[test]
    fn path_home_shows_tilde() {
        let out = render_path("/home/user", "/home/user");
        assert!(out.contains('~'), "expected ~ in: {out}");
        assert!(!out.contains("/home"), "should not contain raw home path");
    }

    #[test]
    fn path_root_shows_slash() {
        let out = render_path("/home/user", "/");
        assert!(out.contains('/'), "expected / in: {out}");
    }

    #[test]
    fn path_deep_truncation() {
        let out = render_path("", "/a/b/c/d/e/f/g");
        assert!(out.contains('…'), "expected ellipsis in: {out}");
        assert!(out.contains('g'), "expected last component");
    }

    #[test]
    fn path_five_components_no_truncation() {
        let out = render_path("", "/a/b/c/d/e");
        assert!(!out.contains('…'), "should not truncate 5 components");
        assert!(out.contains('a'));
        assert!(out.contains('e'));
    }

    #[test]
    fn path_non_home_no_tilde() {
        let out = render_path("/home/user", "/var/log");
        assert!(!out.contains('~'), "should not contain ~ for non-home path");
        assert!(out.contains("var"));
        assert!(out.contains("log"));
    }

    #[test]
    fn path_single_component() {
        let out = render_path("/home/user", "/tmp");
        assert!(out.contains("tmp"));
    }

    #[test]
    fn path_home_subdir_shows_tilde() {
        let out = render_path("/home/user", "/home/user/projects/plx");
        assert!(out.contains('~'), "expected ~ for home subdir");
        assert!(out.contains("plx"));
    }

    // ── Helper: create a temp git repo with an initial commit ────

    fn init_repo(dir: &std::path::Path) -> Repository {
        let repo = Repository::init(dir).expect("failed to init repo");
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();

        {
            let sig = repo.signature().unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
        repo
    }

    // ── render_git ───────────────────────────────────────────────

    #[test]
    fn git_not_a_repo() {
        let tmp = TempDir::new().unwrap();
        let out = render_git(tmp.path());
        assert!(out.contains(ARROW));
        assert!(out.contains(RST));
        assert!(!out.contains(BRANCH_ICON));
    }

    #[test]
    fn git_clean_repo_green() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        let out = render_git(tmp.path());
        assert!(out.contains(&bg(148)), "expected green bg(148) in: {out}");
        assert!(out.contains(BRANCH_ICON));
    }

    #[test]
    fn git_modified_file_shows_pencil_count() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        let file_path = tmp.path().join("file.txt");
        fs::write(&file_path, "hello").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("file.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "add file", &tree, &[&head])
            .unwrap();

        fs::write(&file_path, "modified").unwrap();

        let out = render_git(tmp.path());
        assert!(out.contains(&bg(161)), "expected pink bg(161) in: {out}");
        assert!(out.contains('✎'), "expected pencil icon in: {out}");
    }

    #[test]
    fn git_staged_file_shows_checkmark() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        let file_path = tmp.path().join("new.txt");
        fs::write(&file_path, "new").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("new.txt")).unwrap();
        index.write().unwrap();

        let out = render_git(tmp.path());
        assert!(out.contains('✔'), "expected checkmark in: {out}");
    }

    #[test]
    fn git_untracked_file_shows_plus() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        fs::write(tmp.path().join("untracked.txt"), "x").unwrap();

        let out = render_git(tmp.path());
        assert!(out.contains('+'), "expected + for untracked in: {out}");
    }

    // ── render_tmux_title ────────────────────────────────────────

    #[test]
    fn tmux_home_directory() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_string_lossy().to_string();
        let out = render_tmux_title(&home, &home);
        assert!(out.contains('\u{1F3E0}'), "expected house emoji");
        assert!(out.contains('~'));
    }

    #[test]
    fn tmux_non_repo_directory() {
        let tmp = TempDir::new().unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();
        let out = render_tmux_title("/nonexistent", &pwd);
        assert!(out.contains('\u{1F4C1}'), "expected folder emoji");
    }

    #[test]
    fn tmux_clean_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        let pwd = tmp.path().to_string_lossy().to_string();

        let out = render_tmux_title("/nonexistent", &pwd);
        assert!(
            out.contains("#[fg=colour39]"),
            "expected blue branch in: {out}"
        );
        assert!(out.contains(BRANCH_ICON));
        assert!(!out.contains(PENCIL_ICON), "clean repo should not have pencil");
    }

    #[test]
    fn tmux_dirty_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        fs::write(tmp.path().join("dirty.txt"), "x").unwrap();
        let pwd = tmp.path().to_string_lossy().to_string();

        let out = render_tmux_title("/nonexistent", &pwd);
        assert!(
            out.contains("#[fg=colour67]"),
            "expected grey branch in: {out}"
        );
        assert!(out.contains(PENCIL_ICON), "dirty repo should have pencil");
    }
}
