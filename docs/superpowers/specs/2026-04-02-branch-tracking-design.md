# Branch Tracking Design Spec

## Overview

Add opt-in per-branch database support to tokensave, gated behind the background daemon. When enabled, the daemon watches for git branch switches and maintains separate databases per branch, enabling accurate graphs regardless of which branch is checked out.

## Problem

The tokensave database at `.tokensave/tokensave.db` is gitignored and shared across all branches. Switching branches leaves ghost nodes from the previous branch and stale data until the next sync. The schema has no concept of branches. See `docs/MULTI-BRANCH-DESIGN.md` for the full problem analysis and rejected approaches.

## Solution: Daemon-Gated Branch Tracking

Branch tracking is **off by default** and requires the daemon to be running. Users opt in explicitly.

### Config

New field in `~/.tokensave/config.toml`:

```toml
track_branches = false
```

Persisted via `UserConfig`. Default is `false`.

### CLI

Two new flags on the `Daemon` subcommand:

```
tokensave daemon --track-branches      # sets track_branches=true in config, restarts daemon if running
tokensave daemon --untrack-branches    # sets track_branches=false in config, restarts daemon if running
```

These are config toggles (like `--enable-autostart`), not runtime commands. They write to `config.toml` and restart the daemon so it picks up the change.

### DB Layout

```
.tokensave/
  tokensave.db              # main/master (always exists, the canonical baseline)
  branches/
    feature-foo.db          # per-branch snapshot
    bugfix-bar.db
```

#### Path Resolution

```
resolve_db_path(branch):
  "main" | "master" -> .tokensave/tokensave.db
  <other>           -> .tokensave/branches/<sanitized-branch-name>.db
```

Branch names are sanitized for filesystem safety (replace `/` with `--`, strip leading dots).

### Daemon Behavior

#### When `track_branches = false` (default)

No change from current behavior. The daemon watches for file changes, debounces, and runs incremental sync against the single `tokensave.db`.

#### When `track_branches = true`

The daemon adds a **HEAD watcher** alongside the existing file watcher:

1. **Poll interval**: Short (2-3 seconds) — branch switches should feel near-instant since there's nothing to debounce.
2. **Detection**: Read `.git/HEAD` for each tracked project. Compare against the last-known branch stored in the daemon's in-memory state.
3. **On branch switch**:
   a. Read new branch name from `.git/HEAD` (parse `ref: refs/heads/<name>` or detect detached HEAD).
   b. Resolve target DB path via `resolve_db_path()`.
   c. If target DB doesn't exist, copy the currently active DB as a seed (creates `branches/` dir if needed).
   d. Swap the active DB connection for that project.
   e. Run incremental sync immediately (no debounce).
   f. Update in-memory state with the new branch name.

File-change debounce remains at the configured interval (default 15s). The short poll interval applies only to HEAD watching.

#### Detached HEAD

When HEAD is detached (e.g., `git checkout <sha>`), use the commit SHA as the branch name for DB resolution. This is uncommon but should not crash.

### Metadata Table

Add a `current_branch` key to the existing `metadata` table in each project DB:

```sql
INSERT OR REPLACE INTO metadata (key, value) VALUES ('current_branch', 'feature-foo');
```

This allows the MCP server and CLI to know which branch a DB represents, even outside the daemon.

### Sync Integration

`TokenSave::open()` and `tokensave sync` (CLI) do **not** perform branch-aware DB swapping. They use whatever DB exists at the standard path. The daemon is the only component that manages branch DBs.

When the daemon is not running, the user gets the current behavior: a single DB at `tokensave.db` that reflects whatever branch was last synced.

### Doctor Integration

`tokensave doctor` reports:

- **Branch tracking**: enabled / disabled
- **Active branch**: the `current_branch` from the DB's metadata (if set)
- **Branch DBs found**: list of `branches/*.db` files with sizes
- **Stale branch DBs**: branches that no longer exist in git (informational warning, no auto-delete)

### README Section: "Branch Tracking"

Standalone top-level section explaining three operating modes:

| Mode | Daemon | Branch tracking | Use case |
|---|---|---|---|
| Manual | off | n/a | Full control, sync when you want |
| Auto-sync | on | off (default) | Fire-and-forget, single-branch workflow |
| Branch-aware | on | on | Multi-branch workflows, per-branch graphs |

Includes examples for enabling/disabling and explains that without the daemon, tokensave uses a single DB regardless of branch.

### CLI Usage Update

Add to the daemon block in CLI Usage:

```
tokensave daemon --track-branches    # Enable per-branch database tracking
tokensave daemon --untrack-branches  # Disable per-branch database tracking
```

## Out of Scope

- **Cross-branch blast radius tool** (`tokensave_branch_impact`): future work. Needs branch DBs to exist first.
- **Automatic pruning of stale branch DBs**: future work. Doctor reports them for now.
- **MCP server mid-session DB swap**: the MCP server uses whatever DB exists at tool call time. No hot-swapping.
- **CLI branch-aware sync**: `tokensave sync` always uses `tokensave.db`. Branch DB management is daemon-only.

## Implementation Sequence

1. **UserConfig**: add `track_branches: bool` field (default false).
2. **CLI flags**: add `--track-branches` / `--untrack-branches` to Daemon subcommand. Write config + restart daemon.
3. **DB path resolution**: `resolve_db_path()` function, `branches/` directory creation, branch name sanitization.
4. **Metadata**: store `current_branch` in metadata table on sync.
5. **HEAD watcher**: poll `.git/HEAD` on a short interval in the daemon loop. Detect branch changes.
6. **Copy-on-switch**: on branch change, copy active DB to new path if target doesn't exist, swap connection, sync.
7. **Doctor**: report branch tracking state, list branch DBs, flag stale ones.
8. **README**: add "Branch Tracking" section with the three-mode table.
9. **CHANGELOG**: document the feature.
