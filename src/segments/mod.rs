pub mod git;
pub mod nix_shell;
pub mod path;
pub mod prompt;
pub mod tmux_title;

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
