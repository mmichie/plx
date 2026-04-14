pub mod aws;
pub mod character;
pub mod cmd_duration;
pub mod custom_command;
pub mod git;
pub mod hostname;
pub mod jobs;
pub mod k8s;
pub mod nix_shell;
pub mod node;
pub mod path;
pub mod prompt;
pub mod python;
pub mod registry;
pub mod reset;
pub mod rust_toolchain;
pub mod status;
pub mod tmux_title;
pub mod username;
pub mod venv;

use std::path::PathBuf;

/// Walk up from `start` looking for `filename`. Returns the path to the file
/// if found within `max_depth` parent directories.
pub(crate) fn find_ancestor_file(start: &str, filename: &str, max_depth: usize) -> Option<PathBuf> {
    let mut dir = PathBuf::from(start);
    for _ in 0..=max_depth {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
pub(crate) mod testutil {
    use git2::Repository;

    pub fn init_repo(dir: &std::path::Path) -> Repository {
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
}
