mod args;
mod render;
mod repo;
mod scan;

use args::Args;
use clap::Parser;
use render::Report;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};

fn expand_tilde(path: PathBuf) -> PathBuf {
    let Some(s) = path.to_str() else {
        return path;
    };
    if let Some(rest) = s.strip_prefix("~/").or_else(|| s.strip_prefix('~')) {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    path
}

fn probe_all(paths: Vec<PathBuf>, stale_days: u64, now: i64) -> Vec<repo::Repo> {
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(paths.len().max(1));

    let mut chunks: Vec<Vec<PathBuf>> = vec![Vec::new(); workers];
    for (i, path) in paths.into_iter().enumerate() {
        chunks[i % workers].push(path);
    }

    let handles: Vec<_> = chunks
        .into_iter()
        .filter(|c| !c.is_empty())
        .map(|chunk| {
            std::thread::spawn(move || {
                chunk
                    .into_iter()
                    .filter_map(|p| repo::probe(&p, stale_days, now))
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    handles
        .into_iter()
        .flat_map(|h| h.join().unwrap_or_default())
        .collect()
}

fn main() {
    let args = Args::parse();
    let root = expand_tilde(args.root.clone());
    if !root.is_dir() {
        eprintln!("gitpulse: {}: not a directory", root.display());
        exit(2);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut paths = scan::find_repos(&root, args.max_depth, args.hidden);
    paths.sort();
    paths.dedup();

    let mut repos = probe_all(paths, args.stale_days, now);
    repos.sort_by_key(|r| {
        (
            r.severity(now, args.stale_days),
            std::cmp::Reverse(r.dirty_count()),
        )
    });
    if !args.all {
        repos.retain(|r| r.severity(now, args.stale_days) < 6);
    }

    let root_str = root.display().to_string();
    if args.json {
        let report = Report {
            root: &root_str,
            summary: render::summarize(&repos, now, args.stale_days),
            repos: &repos,
        };
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        if serde_json::to_writer_pretty(&mut out, &report).is_err() {
            exit(1);
        }
        let _ = writeln!(out);
    } else {
        print!(
            "{}",
            render::table(
                &repos,
                &root_str,
                now,
                args.stale_days,
                args.color.enabled()
            )
        );
    }
}
