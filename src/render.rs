use crate::repo::Repo;
use serde::Serialize;
use std::fmt::Write as _;

#[derive(Default, Serialize)]
pub struct Summary {
    pub repos: usize,
    pub dirty: usize,
    pub unpushed: usize,
    pub local_only: usize,
    pub behind: usize,
    pub stale: usize,
    pub empty: usize,
    pub clean: usize,
}

pub fn summarize(repos: &[Repo], now: i64, stale_days: u64) -> Summary {
    let mut s = Summary {
        repos: repos.len(),
        ..Default::default()
    };
    for repo in repos {
        for flag in repo.flags(now, stale_days) {
            match flag {
                "dirty" => s.dirty += 1,
                "unpushed" => s.unpushed += 1,
                "local-only" => s.local_only += 1,
                "behind" => s.behind += 1,
                "stale" => s.stale += 1,
                "empty" => s.empty += 1,
                "clean" => s.clean += 1,
                _ => {}
            }
        }
    }
    s
}

#[derive(Serialize)]
pub struct Report<'a> {
    pub root: &'a str,
    pub summary: Summary,
    pub repos: &'a [Repo],
}

fn flag_color(flag: &str) -> &'static str {
    match flag {
        "dirty" => "33",          // yellow
        "unpushed" => "35",       // magenta
        "local-only" => "31",     // red
        "behind" => "36",         // cyan
        "stale" | "empty" => "2", // dim
        _ => "32",                // clean → green
    }
}

pub fn age(ts: Option<i64>, now: i64) -> String {
    let Some(ts) = ts else {
        return "never".to_string();
    };
    let secs = (now - ts).max(0) as u64;
    let days = secs / 86_400;
    if days == 0 {
        "today".to_string()
    } else if days < 7 {
        format!("{days}d")
    } else if days < 30 {
        format!("{}w", days / 7)
    } else if days < 365 {
        format!("{}mo", days / 30)
    } else {
        format!("{:.1}y", days as f64 / 365.0)
    }
}

fn visible_flags(flags: &[&str]) -> usize {
    flags.iter().map(|f| f.len()).sum::<usize>() + flags.len().saturating_sub(1)
}

pub fn table(repos: &[Repo], root: &str, now: i64, stale_days: u64, color: bool) -> String {
    if repos.is_empty() {
        return "all quiet — no flagged repos\n".to_string();
    }

    let paint = |code: &str, text: &str| -> String {
        if color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    };

    let flag_list: Vec<Vec<&'static str>> =
        repos.iter().map(|r| r.flags(now, stale_days)).collect();

    let w_name = repos.iter().map(|r| r.name.len()).max().unwrap_or(4).max(4);
    let w_branch = repos
        .iter()
        .map(|r| r.branch.len())
        .max()
        .unwrap_or(6)
        .max(6);
    let w_status = flag_list
        .iter()
        .map(|f| visible_flags(f))
        .max()
        .unwrap_or(6)
        .max(6);

    let mut out = String::new();
    let _ = writeln!(out, "{}", paint("1", &format!("gitpulse — {root}")));
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{}",
        paint(
            "2",
            &format!(
                "{:<w_name$}  {:<w_branch$}  {:<w_status$}  {:<8}  {:<5}  {}",
                "REPO",
                "BRANCH",
                "STATUS",
                "SYNC",
                "AGE",
                "CHANGES",
                w_name = w_name,
                w_branch = w_branch,
                w_status = w_status
            )
        )
    );
    for (repo, flags) in repos.iter().zip(&flag_list) {
        let painted = flags
            .iter()
            .map(|f| paint(flag_color(f), f))
            .collect::<Vec<_>>()
            .join(" ");
        let pad = w_status.saturating_sub(visible_flags(flags));
        let sync = match (repo.ahead, repo.behind) {
            (0, 0) => "·".to_string(),
            (a, b) => format!("+{a}/−{b}"),
        };
        let detail = format!("+{} ~{} ?{}", repo.staged, repo.modified, repo.untracked);
        let _ = writeln!(
            out,
            "{:<w_name$}  {:<w_branch$}  {painted}{pad}  {:<8}  {:<5}  {}",
            repo.name,
            repo.branch,
            sync,
            age(repo.last_commit_ts, now),
            paint("2", &detail),
            pad = " ".repeat(pad),
            w_name = w_name,
            w_branch = w_branch
        );
    }

    let s = summarize(repos, now, stale_days);
    let mut parts = vec![format!("{} repos", s.repos)];
    for (count, label) in [
        (s.dirty, "dirty"),
        (s.unpushed, "unpushed"),
        (s.local_only, "local-only"),
        (s.behind, "behind"),
        (s.stale, "stale"),
        (s.empty, "empty"),
        (s.clean, "clean"),
    ] {
        if count > 0 {
            parts.push(format!("{count} {label}"));
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", paint("1", &parts.join(" · ")));
    out
}
