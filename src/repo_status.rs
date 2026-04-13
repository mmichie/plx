use std::fmt::Write;
use std::process::Command;

use git2::Repository;

pub fn run() {
    let repo = match Repository::discover(".") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("not a git repository");
            std::process::exit(1);
        }
    };

    let mut out = String::with_capacity(2048);

    render_header(&repo, &mut out);
    render_branch_status(&repo, &mut out);
    render_recent_commits(&repo, &mut out);
    render_drift(&repo, &mut out);

    if has_gh() {
        render_open_prs(&mut out);
        render_ci_status(&mut out);
    }

    print!("{out}");
}

fn render_header(repo: &Repository, out: &mut String) {
    let name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .map_or_else(String::new, |n| n.to_string_lossy().to_string());

    let branch = current_branch(repo);

    let _ = writeln!(out, "\x1b[1m{name}\x1b[0m on \x1b[36m{branch}\x1b[0m");
    let _ = writeln!(out);
}

fn render_branch_status(repo: &Repository, out: &mut String) {
    let (ahead, behind) = ahead_behind(repo);

    if ahead == 0 && behind == 0 {
        let _ = writeln!(out, "  \x1b[32mup to date with remote\x1b[0m");
    } else {
        if ahead > 0 {
            let _ = writeln!(out, "  \x1b[33m{ahead} commit{} ahead\x1b[0m", plural(ahead));
        }
        if behind > 0 {
            let _ = writeln!(out, "  \x1b[33m{behind} commit{} behind\x1b[0m", plural(behind));
        }
    }
    let _ = writeln!(out);
}

fn render_recent_commits(repo: &Repository, out: &mut String) {
    let _ = writeln!(out, "\x1b[1mRecent commits:\x1b[0m");

    let Ok(mut revwalk) = repo.revwalk() else {
        return;
    };
    let _ = revwalk.push_head();

    let mut count = 0;
    for oid in revwalk.flatten().take(5) {
        let Ok(commit) = repo.find_commit(oid) else {
            continue;
        };
        let short_id = &commit.id().to_string()[..7];
        let summary = commit.summary().unwrap_or("");
        let time = commit.time();
        let age = format_age(time.seconds());

        let _ = writeln!(
            out,
            "  \x1b[33m{short_id}\x1b[0m {summary} \x1b[90m({age})\x1b[0m"
        );
        count += 1;
    }

    if count == 0 {
        let _ = writeln!(out, "  (no commits)");
    }
    let _ = writeln!(out);
}

fn render_drift(repo: &Repository, out: &mut String) {
    // Find how many commits on this branch are not on main/master
    let branch = current_branch(repo);
    let main = find_main_branch(repo);

    if branch == main {
        return;
    }

    let Ok(branch_oid) = repo.revparse_single(&branch).map(|o| o.id()) else {
        return;
    };
    let Ok(main_oid) = repo.revparse_single(&main).map(|o| o.id()) else {
        return;
    };

    if let Ok((ahead, behind)) = repo.graph_ahead_behind(branch_oid, main_oid) {
        let _ = writeln!(out, "\x1b[1mDrift from {main}:\x1b[0m");
        if ahead == 0 && behind == 0 {
            let _ = writeln!(out, "  \x1b[32meven\x1b[0m");
        } else {
            if ahead > 0 {
                let _ = writeln!(
                    out,
                    "  \x1b[36m{ahead} commit{} ahead\x1b[0m",
                    plural(ahead as u32)
                );
            }
            if behind > 0 {
                let _ = writeln!(
                    out,
                    "  \x1b[33m{behind} commit{} behind\x1b[0m",
                    plural(behind as u32)
                );
            }
        }
        let _ = writeln!(out);
    }
}

fn render_open_prs(out: &mut String) {
    // Use --jq to format each PR as: number\ttitle\tbranch\tci_state
    // ci_state is one of: passing, running, failing, unknown
    let jq = concat!(
        ".[] | [(.number | tostring), .title, .headRefName, ",
        "(if (.statusCheckRollup // [] | length) == 0 then \"unknown\" ",
        "elif (.statusCheckRollup | all(.conclusion == \"SUCCESS\")) then \"passing\" ",
        "elif (.statusCheckRollup | any(.status == \"IN_PROGRESS\")) then \"running\" ",
        "else \"failing\" end)] | @tsv",
    );

    let result = Command::new("gh")
        .args([
            "pr", "list", "--author", "@me", "--state", "open", "--limit", "10",
            "--json", "number,title,headRefName,statusCheckRollup",
            "--jq", jq,
        ])
        .output();

    let Ok(output) = result else { return };
    if !output.status.success() {
        return;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim().is_empty() {
        return;
    }

    let _ = writeln!(out, "\x1b[1mOpen PRs:\x1b[0m");
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            continue;
        }
        let (number, title, branch, ci_state) = (parts[0], parts[1], parts[2], parts[3]);

        let ci = match ci_state {
            "passing" => " [\x1b[32mpassing\x1b[0m]",
            "running" => " [\x1b[33mrunning\x1b[0m]",
            "failing" => " [\x1b[31mfailing\x1b[0m]",
            _ => "",
        };

        let _ = writeln!(out, "  \x1b[36m#{number}\x1b[0m {title} ({branch}){ci}");
    }
    let _ = writeln!(out);
}

fn render_ci_status(out: &mut String) {
    // Use --jq to format each check as: name\tstate\tconclusion
    let result = Command::new("gh")
        .args([
            "pr", "checks", "--json", "name,state,conclusion",
            "--jq", ".[] | [.name, .state, .conclusion] | @tsv",
        ])
        .output();

    let Ok(output) = result else { return };
    if !output.status.success() {
        return;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim().is_empty() {
        return;
    }

    let _ = writeln!(out, "\x1b[1mCI checks:\x1b[0m");
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let (name, state, conclusion) = (parts[0], parts[1], parts[2]);

        let icon = match conclusion {
            "SUCCESS" => "\x1b[32m\u{2713}\x1b[0m",
            "FAILURE" => "\x1b[31m\u{2717}\x1b[0m",
            _ if state == "IN_PROGRESS" => "\x1b[33m\u{25cf}\x1b[0m",
            _ => "\x1b[90m\u{25cb}\x1b[0m",
        };

        let _ = writeln!(out, "  {icon} {name}");
    }
    let _ = writeln!(out);
}

fn has_gh() -> bool {
    Command::new("gh")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn current_branch(repo: &Repository) -> String {
    if repo.head_detached().unwrap_or(false) {
        return repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .map_or_else(|| "HEAD".to_string(), |c| c.id().to_string()[..7].to_string());
    }
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(str::to_string))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn ahead_behind(repo: &Repository) -> (u32, u32) {
    let Ok(head) = repo.head() else {
        return (0, 0);
    };
    let Some(local_oid) = head.target() else {
        return (0, 0);
    };
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
        .map(|(a, b)| (a as u32, b as u32))
        .unwrap_or((0, 0))
}

fn find_main_branch(repo: &Repository) -> String {
    for name in ["main", "master"] {
        if repo.find_branch(name, git2::BranchType::Local).is_ok() {
            return name.to_string();
        }
    }
    "main".to_string()
}

fn format_age(epoch_secs: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let delta = now - epoch_secs;
    if delta < 60 {
        "just now".to_string()
    } else if delta < 3600 {
        let m = delta / 60;
        format!("{m} min{} ago", plural(m as u32))
    } else if delta < 86400 {
        let h = delta / 3600;
        format!("{h} hour{} ago", plural(h as u32))
    } else {
        let d = delta / 86400;
        format!("{d} day{} ago", plural(d as u32))
    }
}

fn plural(n: u32) -> &'static str {
    if n == 1 { "" } else { "s" }
}
