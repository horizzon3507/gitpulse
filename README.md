# 🩺 gitpulse

Pulse-check every local git repo — dirty trees, unpushed commits, stale branches, and repos with no remote at all.

```bash
gitpulse              # scan ~/Projects
gitpulse ~            # scan your whole home directory
gitpulse --all        # include clean repos
gitpulse --json       # machine-readable report
```

gitpulse walks a directory tree looking for `.git` directories, then runs the standard `git` binary against each repository it finds. It is a read-only audit: it never fetches, never writes, and never touches your remotes. It answers one question — *what did I forget?*

## What it reports

- **dirty** — staged, modified, untracked, or conflicted files sitting in the worktree
- **unpushed** — commits ahead of upstream, or a branch that was never pushed at all
- **local-only** — no remote configured; the repo exists only on this disk
- **behind** — upstream has commits you have not pulled
- **stale** — last commit (or a side branch tip) older than `--stale-days` (default 90)
- **empty** — a repository with no commits yet
- **clean** — none of the above (shown with `--all`)

Flagged repos sort first by severity, then by how much work is sitting uncommitted.

## Options

- `[ROOT]` sets the scan root (default `~/Projects`, `~` expands to your home).
- `--max-depth N` limits how deep the search descends (default 4).
- `--all` includes clean repositories in the report.
- `--hidden` also descends into dot-directories like `.config`.
- `--stale-days N` sets the staleness threshold (default 90).
- `--json` emits a JSON report with `root`, `summary`, and `repos` fields.
- `--color auto|always|never` controls color. `auto` colors only terminal output, and `NO_COLOR` disables it unless `--color always` is set.
- `-h` or `--help` prints usage.

## Install

Build the release binary with Rust and Cargo:

```bash
git clone https://github.com/horizzon3507/gitpulse.git
cd gitpulse
cargo build --release
install -Dm755 target/release/gitpulse ~/.local/bin/gitpulse
```

## Build from source

```bash
cargo build --release
cargo test
```

gitpulse is written in Rust, has no runtime service, and probes repos by calling the `git` binary you already have installed. All inspection happens locally; nothing is sent over the network — not even a `git fetch`.

## License

MIT
