use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Directories that are never worth descending into while hunting for repos.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "__pycache__",
];

pub fn find_repos(root: &Path, max_depth: usize, hidden: bool) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut it = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    while let Some(entry) = it.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() {
            if name == ".git" {
                if let Some(parent) = entry.path().parent() {
                    repos.push(parent.to_path_buf());
                }
                it.skip_current_dir();
                continue;
            }
            if name == ".jj" {
                it.skip_current_dir();
                continue;
            }
            if entry.depth() > 0
                && (SKIP_DIRS.contains(&name.as_ref()) || (!hidden && name.starts_with('.')))
            {
                it.skip_current_dir();
                continue;
            }
        } else if name == ".git" {
            // Linked worktrees carry a `.git` file pointing at `…/.git/worktrees/…`.
            // Submodules also carry one, but theirs points at `…/.git/modules/…`
            // and belongs to the parent repo — noise, not a repo of its own.
            let is_submodule = std::fs::read_to_string(entry.path())
                .map(|c| c.contains("/.git/modules/"))
                .unwrap_or(false);
            if !is_submodule {
                if let Some(parent) = entry.path().parent() {
                    repos.push(parent.to_path_buf());
                }
            }
        }
    }

    repos
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_git_dir_and_skips_hidden() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("repo/.git")).unwrap();
        fs::create_dir_all(root.join("hidden/.secret/.git")).unwrap();
        fs::create_dir_all(root.join("repo/node_modules/pkg/.git")).unwrap();

        let mut repos = find_repos(root, 4, false);
        repos.sort();
        assert_eq!(repos, vec![root.join("repo")]);

        let mut repos = find_repos(root, 4, true);
        repos.sort();
        assert_eq!(repos, vec![root.join("hidden/.secret"), root.join("repo")]);
    }
}
