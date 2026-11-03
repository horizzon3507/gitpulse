use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Default)]
pub struct Status {
    pub branch: String,
    pub detached: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicts: usize,
}

/// Parses `git status --porcelain=v2 --branch` output.
pub fn parse_porcelain(input: &str) -> Status {
    let mut status = Status::default();

    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            if rest == "(detached)" {
                status.detached = true;
                status.branch = "HEAD".to_string();
            } else {
                status.branch = rest.trim().to_string();
            }
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            status.upstream = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for token in rest.split_whitespace() {
                if let Some(n) = token.strip_prefix('+') {
                    status.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = token.strip_prefix('-') {
                    status.behind = n.parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("u ") {
            status.conflicts += 1;
        } else if line.starts_with("? ") {
            status.untracked += 1;
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            let mut chars = line[2..4].chars();
            if let Some(x) = chars.next() {
                if x != '.' {
                    status.staged += 1;
                }
            }
            if let Some(y) = chars.next() {
                if y != '.' {
                    status.modified += 1;
                }
            }
        }
    }

    status
}

#[derive(Debug, Serialize)]
pub struct Repo {
    pub path: PathBuf,
    pub name: String,
    pub branch: String,
    pub detached: bool,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicts: usize,
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
    pub remotes: usize,
    pub last_commit_ts: Option<i64>,
    pub stale_branches: usize,
}

impl Repo {
    pub fn dirty_count(&self) -> usize {
        self.staged + self.modified + self.untracked + self.conflicts
    }

    pub fn flags(&self, now: i64, stale_days: u64) -> Vec<&'static str> {
        let mut flags = Vec::new();
        if self.last_commit_ts.is_none() {
            flags.push("empty");
        }
        if self.remotes == 0 {
            flags.push("local-only");
        }
        if self.dirty_count() > 0 {
            flags.push("dirty");
        }
        if self.remotes > 0 && (!self.has_upstream || self.ahead > 0) {
            flags.push("unpushed");
        }
        if self.behind > 0 {
            flags.push("behind");
        }
        let cutoff = now - (stale_days as i64 * 86_400);
        if self.stale_branches > 0 || self.last_commit_ts.is_some_and(|ts| ts < cutoff) {
            flags.push("stale");
        }
        if flags.is_empty() {
            flags.push("clean");
        }
        flags
    }

    /// Lower sorts first in the report.
    pub fn severity(&self, now: i64, stale_days: u64) -> u8 {
        let flags = self.flags(now, stale_days);
        for (rank, flag) in ["dirty", "unpushed", "local-only", "behind", "empty", "stale"]
            .iter()
            .enumerate()
        {
            if flags.contains(flag) {
                return rank as u8;
            }
        }
        6 // clean
    }
}

fn git(path: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn probe(path: &Path, stale_days: u64, now: i64) -> Option<Repo> {
    let status = parse_porcelain(&git(path, &["status", "--porcelain=v2", "--branch"])?);

    let remotes = git(path, &["remote"])
        .map(|o| o.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0);

    let last_commit_ts = git(path, &["log", "-1", "--format=%ct"])
        .and_then(|o| o.trim().parse::<i64>().ok());

    let branch = status.branch.clone();
    let cutoff = now - (stale_days as i64 * 86_400);
    let stale_branches = git(
        path,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(committerdate:unix)",
            "refs/heads/",
        ],
    )
    .map(|o| {
        o.lines()
            .filter(|line| {
                let (name, ts) = line.rsplit_once(' ').unwrap_or(("", "0"));
                let ts: i64 = ts.parse().unwrap_or(0);
                name != branch && ts < cutoff
            })
            .count()
    })
    .unwrap_or(0);

    Some(Repo {
        path: path.to_path_buf(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
        branch: if branch.is_empty() {
            "unknown".to_string()
        } else {
            branch
        },
        detached: status.detached,
        staged: status.staged,
        modified: status.modified,
        untracked: status.untracked,
        conflicts: status.conflicts,
        ahead: status.ahead,
        behind: status.behind,
        has_upstream: status.upstream.is_some(),
        remotes,
        last_commit_ts,
        stale_branches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_tracking_branch() {
        let out = "\
# branch.oid abc123
# branch.head main
# branch.upstream origin/main
# branch.ab +0 -0
";
        let s = parse_porcelain(out);
        assert_eq!(s.branch, "main");
        assert_eq!(s.upstream.as_deref(), Some("origin/main"));
        assert_eq!((s.ahead, s.behind), (0, 0));
        assert_eq!(s.staged + s.modified + s.untracked, 0);
    }

    #[test]
    fn parses_dirty_and_diverged() {
        let out = "\
# branch.oid abc123
# branch.head feature
# branch.upstream origin/feature
# branch.ab +3 -2
1 M. N... 100644 100644 100644 aaa bbb staged.txt
1 .M N... 100644 100644 100644 aaa bbb modified.txt
1 MM N... 100644 100644 100644 aaa bbb both.txt
2 R. N... 100644 100644 100644 aaa bbb R100 new.txt\told.txt
u UU N... 100644 100644 100644 100644 aaa bbb ccc conflict.txt
? untracked.txt
! ignored.txt
";
        let s = parse_porcelain(out);
        assert_eq!((s.ahead, s.behind), (3, 2));
        assert_eq!(s.staged, 3); // M., MM, R.
        assert_eq!(s.modified, 2); // .M, MM
        assert_eq!(s.untracked, 1);
        assert_eq!(s.conflicts, 1);
    }

    #[test]
    fn parses_detached_and_no_commits() {
        let detached = "# branch.oid abc\n# branch.head (detached)\n";
        let s = parse_porcelain(detached);
        assert!(s.detached);
        assert_eq!(s.upstream, None);

        let empty = "# branch.oid (initial)\n# branch.head main\n";
        let s = parse_porcelain(empty);
        assert_eq!(s.branch, "main");
        assert!(!s.detached);
    }
}
