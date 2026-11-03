# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/). gitpulse is a single CLI surface.

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.1.1-stable`, `v0.1.0-stable`.

Here the installable surface is **CLI** (`gitpulse`). Other apps swap in their own names the same way — e.g. **GNOME**, **Desktop**, **Web** — whatever you actually ship.

| Part | What you install | Example |
| --- | --- | --- |
| **CLI** | `gitpulse` in the terminal | `v0.1.1-stable` |

With one surface there is no `m` in the tag and no per-surface sections — just the version notes.

Each release heading is the version and date (`## v0.1.1-stable · 03/11/2026`); under it, a short summary ends with a plain sentence like: “This version was made for CLI with a stable release channel on 03/11/2026 (v0.1.1-stable).”

### What the channel suffix means

| Suffix | In plain words |
| --- | --- |
| **-alpha** | Very early. Expect missing pieces and lots of bugs. |
| **-beta** | Mostly there, but still rough. Fine to try; not the “official” install. |
| **-stable** | Ready for daily use. This is what we put on GitHub Releases. |

We only call something **stable** when we mean it.

</details>

## v0.1.1-stable · 03/11/2026

Toolchain bump. This version was made for CLI with a stable release channel on 03/11/2026 (v0.1.1-stable).

- Migrate to Rust edition 2024, with `rust-version` pinned at 1.85.
- Reformat sources under the edition's style rules; no behavior changes.

## v0.1.0-stable · 03/11/2026

First release. This version was made for CLI with a stable release channel on 03/11/2026 (v0.1.0-stable).

- Scans a directory tree for git repositories, skipping hidden directories and common vendor trees (`node_modules`, `target`, `dist`, `build`, `vendor`).
- Flags repos as `dirty`, `unpushed`, `behind`, `local-only`, `stale`, `empty`, or `clean`, sorted by severity then by uncommitted work.
- Parses `git status --porcelain=v2 --branch` for staged/modified/untracked counts and ahead/behind divergence in a single subprocess call.
- Detects stale side branches via `for-each-ref` against the `--stale-days` threshold.
- Detects linked worktrees through `.git` files, not only `.git` directories.
- `--json` emits a machine-readable report; `--color auto|always|never` controls colors with `NO_COLOR` support.
- Probes repos in parallel across available CPU threads.
