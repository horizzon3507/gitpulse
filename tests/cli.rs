use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}

fn gitpulse(dir: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_gitpulse"))
        .arg(dir)
        .args(args)
        .arg("--json")
        .output()
        .expect("gitpulse runs");
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn init_repo(dir: &Path) {
    std::fs::create_dir(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "test@test"]);
    git(dir, &["config", "user.name", "test"]);
}

fn commit(dir: &Path, file: &str, contents: &str) {
    std::fs::write(dir.join(file), contents).unwrap();
    git(dir, &["add", file]);
    git(dir, &["commit", "-qm", "init"]);
}

fn names(report: &Value) -> Vec<&str> {
    report["repos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect()
}

#[test]
fn reports_local_only_and_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let empty = root.join("empty-repo");
    std::fs::create_dir(&empty).unwrap();
    git(&empty, &["init", "-q", "-b", "main"]);

    let local = root.join("local-repo");
    init_repo(&local);
    commit(&local, "a.txt", "hi");

    let report = gitpulse(root, &[]);
    let found = names(&report);
    assert!(found.contains(&"empty-repo"));
    assert!(found.contains(&"local-repo"));
    assert_eq!(report["summary"]["local_only"], 2);

    let empty_json = report["repos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "empty-repo")
        .unwrap();
    assert_eq!(empty_json["last_commit_ts"], Value::Null);
    assert_eq!(empty_json["remotes"], 0);
}

#[test]
fn reports_dirty_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let repo = root.join("dirty-repo");
    init_repo(&repo);
    git(
        &repo,
        &["remote", "add", "origin", "https://example.com/x.git"],
    );
    commit(&repo, "a.txt", "hi");
    std::fs::write(repo.join("b.txt"), "untracked").unwrap();
    std::fs::write(repo.join("a.txt"), "changed").unwrap();

    let report = gitpulse(root, &[]);
    let r = &report["repos"][0];
    assert_eq!(r["name"], "dirty-repo");
    assert_eq!(r["untracked"], 1);
    assert_eq!(r["modified"], 1);
    assert_eq!(r["remotes"], 1);
}

#[test]
fn all_flag_includes_clean_repos() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let repo = root.join("clean-repo");
    init_repo(&repo);
    git(
        &repo,
        &["remote", "add", "origin", "https://example.com/x.git"],
    );
    commit(&repo, "a.txt", "hi");
    // Fake the remote-tracking ref so `main` has an upstream and the repo is clean.
    git(
        &repo,
        &["update-ref", "refs/remotes/origin/main", "refs/heads/main"],
    );
    git(&repo, &["branch", "--set-upstream-to=origin/main", "main"]);

    let flagged = gitpulse(root, &[]);
    assert!(!names(&flagged).contains(&"clean-repo"));

    let all = gitpulse(root, &["--all"]);
    assert!(names(&all).contains(&"clean-repo"));
}
