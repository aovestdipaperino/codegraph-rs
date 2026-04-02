# Branch Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add opt-in per-branch database support to the tokensave daemon, so each git branch gets its own index that stays accurate across branch switches.

**Architecture:** Branch tracking is gated behind a `track_branches` config flag. When enabled, the daemon polls `.git/HEAD` every 3 seconds. On branch switch it copies the active DB as a seed (if needed), swaps DB connections, and runs an immediate incremental sync. Main/master always uses `tokensave.db`; other branches use `branches/<name>.db`.

**Tech Stack:** Rust, tokio, existing `UserConfig` TOML system, existing `daemon.rs` event loop.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/user_config.rs` | Modify | Add `track_branches: bool` field |
| `src/main.rs` | Modify | Add `--track-branches` / `--untrack-branches` CLI flags, dispatch |
| `src/daemon.rs` | Modify | HEAD watcher, branch DB resolution, copy-on-switch, modified event loop |
| `src/doctor.rs` | Modify | Report branch tracking state and branch DBs |
| `README.md` | Modify | Add "Branch Tracking" top-level section |
| `CHANGELOG.md` | Modify | Document the feature |
| `tests/daemon_branch_test.rs` | Create | Tests for branch name parsing, DB path resolution, sanitization |

---

### Task 1: Add `track_branches` to UserConfig

**Files:**
- Modify: `src/user_config.rs:12-64` (struct + Default impl)

- [ ] **Step 1: Add the field to `UserConfig` struct**

In `src/user_config.rs`, add after the `last_flags_fetch_at` field (line 63):

```rust
    /// Whether the daemon should maintain per-branch databases.
    #[serde(default)]
    pub track_branches: bool,
```

- [ ] **Step 2: Add field to `Default` impl**

In the `Default` impl (around line 74), add after `last_flags_fetch_at: 0`:

```rust
            track_branches: false,
```

- [ ] **Step 3: Verify build**

Run: `cargo check`
Expected: clean build

- [ ] **Step 4: Commit**

```bash
git add src/user_config.rs
git commit -m "feat: add track_branches config field"
```

---

### Task 2: Add CLI flags to Daemon subcommand

**Files:**
- Modify: `src/main.rs:224-240` (Daemon enum variant)
- Modify: `src/main.rs:744-757` (Daemon command dispatch)

- [ ] **Step 1: Add flags to Daemon variant**

In `src/main.rs`, add two new fields to the `Daemon` variant after `disable_autostart`:

```rust
        /// Enable per-branch database tracking (requires daemon)
        #[arg(long)]
        track_branches: bool,
        /// Disable per-branch database tracking
        #[arg(long)]
        untrack_branches: bool,
```

- [ ] **Step 2: Update the match arm destructuring**

Change the `Commands::Daemon` match arm (line 744) to include the new fields:

```rust
        Commands::Daemon { foreground, stop, status, enable_autostart, disable_autostart, track_branches, untrack_branches } => {
```

- [ ] **Step 3: Add dispatch logic**

Add two new `else if` arms after the `disable_autostart` branch and before the `else` (which calls `run`):

```rust
            } else if track_branches {
                tokensave::daemon::set_track_branches(true)?;
            } else if untrack_branches {
                tokensave::daemon::set_track_branches(false)?;
            } else {
```

- [ ] **Step 4: Implement `set_track_branches` in daemon.rs**

Add to `src/daemon.rs`, after the `disable_autostart` function:

```rust
/// Enable or disable branch tracking and restart the daemon if running.
pub fn set_track_branches(enable: bool) -> Result<()> {
    let mut config = crate::user_config::UserConfig::load();
    config.track_branches = enable;
    config.save();

    let label = if enable { "enabled" } else { "disabled" };
    eprintln!("\x1b[32m✔\x1b[0m Branch tracking {label}");

    // Restart daemon if currently running so it picks up the new config.
    if running_daemon_pid().is_some() {
        eprintln!("  Restarting daemon...");
        stop()?;
        // Give it a moment to release the PID file.
        std::thread::sleep(std::time::Duration::from_millis(500));
        // Re-launch in background (non-foreground).
        let bin = crate::agents::which_tokensave().unwrap_or_else(|| "tokensave".to_string());
        std::process::Command::new(bin)
            .args(["daemon"])
            .spawn()
            .ok();
        eprintln!("  Daemon restarted");
    }

    Ok(())
}
```

- [ ] **Step 5: Verify build**

Run: `cargo check`
Expected: clean build

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/daemon.rs
git commit -m "feat: add --track-branches / --untrack-branches CLI flags"
```

---

### Task 3: Branch name parsing and DB path resolution

**Files:**
- Modify: `src/daemon.rs` (add helper functions)
- Create: `tests/daemon_branch_test.rs`

- [ ] **Step 1: Write tests for branch name parsing and path resolution**

Create `tests/daemon_branch_test.rs`:

```rust
use std::path::PathBuf;

#[test]
fn parse_git_head_branch() {
    assert_eq!(
        tokensave::daemon::parse_head_branch("ref: refs/heads/main"),
        Some("main".to_string())
    );
    assert_eq!(
        tokensave::daemon::parse_head_branch("ref: refs/heads/feature/foo-bar"),
        Some("feature/foo-bar".to_string())
    );
}

#[test]
fn parse_git_head_detached() {
    // Detached HEAD is a raw SHA
    assert_eq!(
        tokensave::daemon::parse_head_branch("abc123def456"),
        None
    );
}

#[test]
fn sanitize_branch_name() {
    assert_eq!(tokensave::daemon::sanitize_branch("main"), "main");
    assert_eq!(tokensave::daemon::sanitize_branch("feature/foo"), "feature--foo");
    assert_eq!(tokensave::daemon::sanitize_branch("feature/deep/nest"), "feature--deep--nest");
    assert_eq!(tokensave::daemon::sanitize_branch(".hidden"), "_hidden");
}

#[test]
fn resolve_db_path_main() {
    let ts_dir = PathBuf::from("/project/.tokensave");
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "main"),
        ts_dir.join("tokensave.db")
    );
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "master"),
        ts_dir.join("tokensave.db")
    );
}

#[test]
fn resolve_db_path_feature_branch() {
    let ts_dir = PathBuf::from("/project/.tokensave");
    assert_eq!(
        tokensave::daemon::resolve_branch_db_path(&ts_dir, "feature/foo"),
        ts_dir.join("branches/feature--foo.db")
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test daemon_branch_test`
Expected: FAIL — functions don't exist yet

- [ ] **Step 3: Implement the helper functions**

Add to `src/daemon.rs`, before `run_loop`:

```rust
/// Parse the branch name from `.git/HEAD` content.
/// Returns `None` for detached HEAD (raw SHA).
pub fn parse_head_branch(head_content: &str) -> Option<String> {
    let trimmed = head_content.trim();
    trimmed
        .strip_prefix("ref: refs/heads/")
        .map(|s| s.to_string())
}

/// Sanitize a branch name for use as a filename.
/// Replaces `/` with `--` and strips leading dots.
pub fn sanitize_branch(name: &str) -> String {
    let sanitized = name.replace('/', "--");
    if sanitized.starts_with('.') {
        format!("_{}", &sanitized[1..])
    } else {
        sanitized
    }
}

/// Resolve the DB path for a given branch.
/// `main` and `master` map to `tokensave.db`; others to `branches/<sanitized>.db`.
pub fn resolve_branch_db_path(tokensave_dir: &Path, branch: &str) -> PathBuf {
    if branch == "main" || branch == "master" {
        tokensave_dir.join("tokensave.db")
    } else {
        tokensave_dir.join("branches").join(format!("{}.db", sanitize_branch(branch)))
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test daemon_branch_test`
Expected: all 5 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/daemon.rs tests/daemon_branch_test.rs
git commit -m "feat: branch name parsing and DB path resolution"
```

---

### Task 4: HEAD watcher in the daemon event loop

**Files:**
- Modify: `src/daemon.rs:66-141` (run_loop function)

This is the core change. The daemon loop gets a new branch: a 3-second HEAD poll for each tracked project. When a branch switch is detected, it copies the DB (if needed) and triggers an immediate sync.

- [ ] **Step 1: Add branch tracking state to `run_loop`**

At the top of `run_loop`, after `let mut dirty: ...` (line 70), add:

```rust
    let config = crate::user_config::UserConfig::load();
    let track_branches = config.track_branches;

    // Per-project branch tracking: last known branch name.
    let mut known_branches: HashMap<PathBuf, String> = HashMap::new();
```

- [ ] **Step 2: Add HEAD poll interval**

After `discovery_interval` setup (line 83), add:

```rust
    let mut head_poll_interval = time::interval(Duration::from_secs(3));
    head_poll_interval.tick().await; // consume first immediate tick
```

- [ ] **Step 3: Initialize known branches on startup**

After the initial project discovery loop (after line 78), add:

```rust
    if track_branches {
        for path in &project_paths {
            if let Some(branch) = read_project_branch(path) {
                known_branches.insert(path.clone(), branch);
            }
        }
    }
```

- [ ] **Step 4: Add HEAD poll branch to the `tokio::select!`**

Add a new branch inside the `tokio::select!` block, after the `discovery_interval.tick()` branch (before the closing `}`):

```rust
            _ = head_poll_interval.tick(), if track_branches => {
                for (path, _watcher) in &watchers {
                    let Some(current_branch) = read_project_branch(path) else {
                        continue;
                    };
                    let previous = known_branches.get(path).map(String::as_str);
                    if previous == Some(&current_branch) {
                        continue;
                    }

                    daemon_log(&format!(
                        "branch switch detected in {}: {} → {}",
                        path.display(),
                        previous.unwrap_or("(unknown)"),
                        current_branch
                    ));

                    let ts_dir = crate::config::get_tokensave_dir(path);
                    let target_db = resolve_branch_db_path(&ts_dir, &current_branch);

                    // Copy-on-switch: seed from current DB if target doesn't exist.
                    if !target_db.exists() {
                        let source_db = ts_dir.join("tokensave.db");
                        if source_db.exists() {
                            if let Some(parent) = target_db.parent() {
                                std::fs::create_dir_all(parent).ok();
                            }
                            if let Err(e) = std::fs::copy(&source_db, &target_db) {
                                daemon_log(&format!(
                                    "failed to seed branch DB {}: {e}",
                                    target_db.display()
                                ));
                            } else {
                                daemon_log(&format!(
                                    "seeded branch DB: {}",
                                    target_db.display()
                                ));
                            }
                        }
                    }

                    known_branches.insert(path.clone(), current_branch);

                    // Immediate sync (no debounce for branch switches).
                    sync_project(path).await;
                }
            }
```

- [ ] **Step 5: Implement `read_project_branch`**

Add to `src/daemon.rs`, after `resolve_branch_db_path`:

```rust
/// Read the current branch name from a project's `.git/HEAD`.
fn read_project_branch(project_root: &Path) -> Option<String> {
    let head_path = project_root.join(".git/HEAD");
    let content = std::fs::read_to_string(head_path).ok()?;
    parse_head_branch(&content)
}
```

- [ ] **Step 6: Verify build**

Run: `cargo check`
Expected: clean build

- [ ] **Step 7: Commit**

```bash
git add src/daemon.rs
git commit -m "feat: HEAD watcher in daemon loop with copy-on-switch"
```

---

### Task 5: Store `current_branch` in metadata on sync

**Files:**
- Modify: `src/daemon.rs:221-249` (`sync_project_inner`)

- [ ] **Step 1: Write metadata after successful sync**

In `sync_project_inner`, after the successful sync log line (inside the `Ok(result)` arm, after the `daemon_log` call around line 237), add:

```rust
            // Record which branch this DB was synced on.
            if let Some(branch) = read_project_branch(project_root) {
                cg.set_metadata("current_branch", &branch).await.ok();
            }
```

- [ ] **Step 2: Add `set_metadata` to TokenSave if it doesn't exist**

Check if `set_metadata` exists. If not, add to `src/tokensave.rs`:

```rust
    /// Write a key-value pair to the metadata table.
    pub async fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.db.set_metadata(key, value).await
    }
```

And to `src/db/connection.rs` (or wherever `Database` methods live):

```rust
    /// Insert or update a metadata key-value pair.
    pub async fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                libsql::params![key, value],
            )
            .await
            .map_err(|e| TokenSaveError::Database {
                message: format!("failed to write metadata: {e}"),
            })?;
        Ok(())
    }
```

- [ ] **Step 3: Verify build**

Run: `cargo check`
Expected: clean build

- [ ] **Step 4: Commit**

```bash
git add src/daemon.rs src/tokensave.rs src/db/connection.rs
git commit -m "feat: store current_branch in metadata on sync"
```

---

### Task 6: Doctor branch tracking report

**Files:**
- Modify: `src/doctor.rs:163-175` (`check_daemon`)

- [ ] **Step 1: Add branch tracking info to doctor**

Expand `check_daemon` in `src/doctor.rs`. After the autostart check (line 174), add:

```rust
    let config = crate::user_config::UserConfig::load();
    if config.track_branches {
        dc.pass("Branch tracking enabled");
    } else {
        dc.info("Branch tracking disabled (enable with `tokensave daemon --track-branches`)");
    }

    // Report branch DBs if any exist.
    let project_path = std::env::current_dir().unwrap_or_default();
    let branches_dir = crate::config::get_tokensave_dir(&project_path).join("branches");
    if branches_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&branches_dir) {
            let dbs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "db"))
                .collect();
            if !dbs.is_empty() {
                for entry in &dbs {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let name = entry.file_name();
                    dc.info(&format!(
                        "Branch DB: {} ({})",
                        name.to_string_lossy(),
                        crate::display::format_bytes(size)
                    ));
                }
            }
        }
    }
```

- [ ] **Step 2: Add `info` method to `DoctorCounters` if it doesn't exist**

Check if `DoctorCounters` has an `info` method. If not, add one that prints with a blue `ℹ` prefix (similar to `warn` but informational, no counter increment):

```rust
    pub fn info(&self, msg: &str) {
        eprintln!("  \x1b[34mℹ\x1b[0m {msg}");
    }
```

- [ ] **Step 3: Verify build**

Run: `cargo check`
Expected: clean build

- [ ] **Step 4: Commit**

```bash
git add src/doctor.rs
git commit -m "feat: doctor reports branch tracking state and branch DBs"
```

---

### Task 7: README "Branch Tracking" section

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add standalone section after "Key Features"**

Insert a new top-level section after the "Key Features" section (after line 75, before the "What's New" section). Add:

```markdown
## Branch Tracking

tokensave supports three operating modes depending on how you want to manage your index:

| Mode | Daemon | Branch tracking | Best for |
|---|---|---|---|
| **Manual** | off | n/a | Full control — you run `tokensave sync` when you want |
| **Auto-sync** | on | off *(default)* | Fire-and-forget — daemon syncs on every file change |
| **Branch-aware** | on | on | Multi-branch workflows — each branch gets its own index |

### Manual (no daemon)

Run `tokensave sync` yourself. The index reflects whatever branch you last synced on. If you switch branches, run `tokensave sync` again to re-index.

### Auto-sync (daemon, no branch tracking)

```bash
tokensave daemon                     # start the daemon
tokensave daemon --enable-autostart  # auto-start on login
```

The daemon watches for file changes and syncs automatically. There is one database shared across all branches — switching branches triggers a re-sync on the next file change.

### Branch-aware (daemon + branch tracking)

```bash
tokensave daemon --track-branches    # enable per-branch databases
```

The daemon monitors `.git/HEAD` and maintains a separate database per branch:

```
.tokensave/
  tokensave.db          # main/master (always the canonical baseline)
  branches/
    feature-foo.db      # created when you first switch to feature-foo
    bugfix-bar.db
```

When you switch branches, the daemon:
1. Seeds a new branch database from the current one (if it doesn't exist yet)
2. Swaps to the branch database
3. Runs an incremental sync immediately

Switching back to a previously visited branch is instant — the database is already there.

To disable:

```bash
tokensave daemon --untrack-branches
```
```

- [ ] **Step 2: Add the new flags to the CLI Usage block**

In the CLI Usage section (around line 358-361), add after the existing daemon lines:

```
tokensave daemon --track-branches    # Enable per-branch database tracking
tokensave daemon --untrack-branches  # Disable per-branch database tracking
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add Branch Tracking section to README"
```

---

### Task 8: CHANGELOG entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add entry under a new unreleased section**

At the top of CHANGELOG.md, after the header and before the `[3.1.1]` entry, add:

```markdown
## [Unreleased]

### Added
- **Per-branch database tracking** — `tokensave daemon --track-branches` enables the daemon to maintain separate databases per git branch. On branch switch, the daemon seeds a new DB from the current one and runs an immediate incremental sync. Disable with `--untrack-branches`. Requires the background daemon.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: add branch tracking to CHANGELOG"
```

---

### Task 9: Integration smoke test

**Files:**
- Modify: `tests/daemon_branch_test.rs`

- [ ] **Step 1: Add integration test for the full copy-on-switch flow**

Add to `tests/daemon_branch_test.rs`:

```rust
#[test]
fn copy_on_switch_creates_branch_db() {
    let dir = tempfile::tempdir().unwrap();
    let ts_dir = dir.path().join(".tokensave");
    std::fs::create_dir_all(&ts_dir).unwrap();

    // Create a fake main DB.
    let main_db = ts_dir.join("tokensave.db");
    std::fs::write(&main_db, b"fake-db-content").unwrap();

    // Resolve path for a feature branch.
    let branch_db = tokensave::daemon::resolve_branch_db_path(&ts_dir, "feature/new-thing");
    assert!(!branch_db.exists());

    // Simulate copy-on-switch: create parent dir and copy.
    if let Some(parent) = branch_db.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::copy(&main_db, &branch_db).unwrap();

    assert!(branch_db.exists());
    assert_eq!(
        std::fs::read(&branch_db).unwrap(),
        b"fake-db-content"
    );
    assert_eq!(
        branch_db,
        ts_dir.join("branches/feature--new-thing.db")
    );
}
```

- [ ] **Step 2: Run all daemon branch tests**

Run: `cargo test --test daemon_branch_test`
Expected: all 6 tests PASS

- [ ] **Step 3: Run full test suite**

Run: `cargo test --lib --test daemon_branch_test --test cloud_test`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add tests/daemon_branch_test.rs
git commit -m "test: integration test for branch DB copy-on-switch"
```

---

### Task 10: Final verification and push

- [ ] **Step 1: Run cargo clippy**

Run: `cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: no errors (warnings allowed for inactive cfg blocks on non-Windows)

- [ ] **Step 2: Run full build**

Run: `cargo build`
Expected: clean build

- [ ] **Step 3: Verify `--help` shows the new flags**

Run: `cargo run -- daemon --help`
Expected output includes:
```
--track-branches     Enable per-branch database tracking (requires daemon)
--untrack-branches   Disable per-branch database tracking
```

- [ ] **Step 4: Push beta branch**

```bash
git push origin beta
```
