use std::fmt::Write;

use git2::{Repository, StatusOptions, StatusShow};

use crate::color::{bg, fg, ARROW, BRANCH_ICON, RST};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_with(repo: Option<&mut Repository>, from_bg: u8) -> (String, u8) {
    let Some(repo) = repo else {
        // Not in a git repo — just output the closing arrow (dir_end)
        return (format!("{}{}{ARROW}{RST}", fg(from_bg), bg(236)), 236);
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
    let (ahead, behind) = ahead_behind(repo);

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
            fg(from_bg), bg(161),
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
            fg(from_bg), bg(148),
            fg(0),
            fg(148), bg(236),
        );
    }

    (out, 236)
}

#[must_use]
pub fn render(discover_from: &std::path::Path) -> String {
    let mut repo = Repository::discover(discover_from).ok();
    render_with(repo.as_mut(), 237).0
}

fn ahead_behind(repo: &Repository) -> (u32, u32) {
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

#[cfg(test)]
mod tests {
    use super::{render, render_with};
    use crate::color::{bg, fg, ARROW, BRANCH_ICON, RST};
    use crate::segments::testutil::init_repo;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn not_a_repo() {
        let tmp = TempDir::new().unwrap();
        let out = render(tmp.path());
        assert!(out.contains(ARROW));
        assert!(out.contains(RST));
        assert!(!out.contains(BRANCH_ICON));
    }

    #[test]
    fn clean_repo_green() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        let out = render(tmp.path());
        assert!(out.contains(&bg(148)), "expected green bg(148) in: {out}");
        assert!(out.contains(BRANCH_ICON));
    }

    #[test]
    fn modified_file_shows_pencil_count() {
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

        let out = render(tmp.path());
        assert!(out.contains(&bg(161)), "expected pink bg(161) in: {out}");
        assert!(out.contains('✎'), "expected pencil icon in: {out}");
    }

    #[test]
    fn staged_file_shows_checkmark() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        let file_path = tmp.path().join("new.txt");
        fs::write(&file_path, "new").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("new.txt")).unwrap();
        index.write().unwrap();

        let out = render(tmp.path());
        assert!(out.contains('✔'), "expected checkmark in: {out}");
    }

    #[test]
    fn untracked_file_shows_plus() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        fs::write(tmp.path().join("untracked.txt"), "x").unwrap();

        let out = render(tmp.path());
        assert!(out.contains('+'), "expected + for untracked in: {out}");
    }

    #[test]
    fn render_with_pre_discovered_repo() {
        let tmp = TempDir::new().unwrap();
        let mut repo = init_repo(tmp.path());

        let (out, end_bg) = render_with(Some(&mut repo), 237);
        assert!(out.contains(&bg(148)), "expected green bg(148) in: {out}");
        assert!(out.contains(BRANCH_ICON));
        assert_eq!(end_bg, 236);
    }

    #[test]
    fn render_with_no_repo() {
        let (out, end_bg) = render_with(None, 240);
        assert!(out.contains(&fg(240)), "expected fg(240) in: {out}");
        assert!(out.contains(ARROW));
        assert_eq!(end_bg, 236);
    }
}
