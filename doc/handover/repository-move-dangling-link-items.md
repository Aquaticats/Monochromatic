# Dangling-link work items W2 to W15 for the repository move to the 860 drive: state and next steps

Written 2026-11-07, after the runner fixes of the first fixture runs and the lint of the two new packages.
The plan is `doc/planning/repository-on-the-860-drive.md`
(sections "Work items per tool", "Status and order of the work items", "The symlink design", "The send plan");
the runner is `package/repository-move/cli` (runbook `doc/runbook/repository-on-the-860-drive-guided-runner.md`);
the shared check is `package/repository-link/check` (W1).
Branch `feat/repository-move-dangling-link-items`, worktree `.claude/worktrees/dangling-link-items`;
everything below "Done" is merged into main with `git merge --no-ff`.

## Owner words that bind this work

Verbatim, from the brief and the plan:

- The owner picked a symbolic link at the old path ("take care of dangling links").
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "Commits should never fail."
- "We should never invoke ourselves recursively."
- "Principle 1: Lint issues need to be taken care of asap."
- "I'm authorizing you to bump and publish any updates this session." and "Label decides" (releases, as alphas, through the release tool, as the latest records in `doc/planning/one-release-each-batch-2026-11.md` did).
- Never generate synthetic host load; every non-trivial command in `systemd-run --user --scope` with `CPUQuota`, `MemoryMax`, `TasksMax` and `IOWeight` (never above 10); `GIT_ASKPASS=` and `GIT_TERMINAL_PROMPT=0` on Git and `gh`; never `sudo`; never edit `doc/decision/`; commit with `git commit --message ... -- <paths>`, tests and changes in separate commits, never amend.

## Done and merged

- Lint of `package/repository-move/cli` and `package/repository-link/check`: zero warnings, nothing loosened.
  The sequential awaits that are the design carry one justified suppression each, with
  `package/repository-move/cli/doc/lint/sequential-awaits.md`.
- Runner fixes, each test-first: stage list rebuilt before each stage; `f1a` resolves links; `f1-images` rerun-safe;
  reconcile compares resolved paths; fixture snapshot folders inside the source image and a shared-filesystem preflight;
  the empty-parent send plan, the destination check, `verify-first` and `verify-final`; placement by what is at the path;
  strict runner arguments; stages declare their accepted exit statuses (`f5a`); the clone-limit allowance in the preflight.
  Facts and sources: `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`,
  `doc/troubleshooting/findmnt-two-operands-need-source-and-target.md`, the plan section "The send plan".
- W8: `package/claude-cpu-cap/daemon` unit with `ExecStartPre` running the installed check, and the unit installed
  as a regular copy on this host (originals under `${HOME}/temp/agent/860-w-items/`).
  Not run: the dangling-link run of the unit, which the plan leaves to the owner.
- W1 bins reinstalled to `~/.local/share/monochromatic-dev/repository-link/` (stale chunks
  `record-identity-D7BnSfbH.mjs` and `resolve-command-auK8bmv9.mjs` remain; the installer deletes nothing).

## Decisions, with their source

Coordinator answers of 2026-11-07, after the file list for W2 and W3:

1. Reader for W2: reuse what is on main, never copy from another worktree.
   First grep main for an existing `by-uuid` lookup; if none, add one small function beside
   `src/native/subvolume/fs_mount_options.rs`, with its own tests, and use `read_mount_facts` for the table.
   One reader of each fact in the crate.
2. One fixture table: a single JSON file inside `package/git-policy/cli` (the container gate copies only a crate snapshot,
   so a path outside the crate fails there and in the packaged crate).
   The W1 TypeScript test reads it by relative path; a comment in both suites names the other;
   both assert the row count and iterate every row; add a W1 test that fails when the path does not resolve.
3. Folder `src/native/repository_link_check/` until the layout move (issue 6261 steps 2 to 6).
   No `uses/` folder (no folder of the crate has one; the move adds them all at once).
   Add the folder to the list of folders to move in `doc/planning/cli-git-source-layout.md`.
4. Gate in `startup_gate.rs`, `recover_for_command(context, identity)`, in this order:
   identity file absent, return; resolved Git directory not under the link, the canonical root or the stale NVMe path,
   return; phase `before` with one `lstat` saying directory, return; only phase `after` with a link does the full walk.
   No process start on the hot path (forwarded p95 near 2 ms): no `findmnt`.
   Take the `native:perf` baseline from current main before the first edit of the invocation entry
   (same-binary spread first, saved under `${HOME}/temp/agent/860-w-items/`) and report before and after.
5. Frozen-move message ("the repository is frozen for the move to the 860; nothing was written; try again after the move finishes"):
   translate `ErrorKind::ReadOnlyFilesystem` at the one place where commit I/O errors become user text
   (grep for it; send the coordinator the file and function before editing it if it is under `service/` or `landing/`).
6. `src/native/service/service_instance.rs` and `src/native/landing/service_route.rs` may be edited now:
   `try_instance_of(git_dir) -> Result<String, GitDirUnreachable>` (error when `canonicalize` fails; `instance_of` keeps
   its signature, with a doc line not to use it to name anything new);
   `choose_route` returns no route for an unresolvable Git directory; `service_status` says the cause.
7. W3 waits: `service_work.rs`, `service_work_handlers_replanning_and_acceptance.rs` and `service_server_work_loop.rs`
   only after the coordinator says the commit-stall branch is in main.
   Keep the change to the smallest function-level edit with behavior tests, because tokio step T10 converts the work loop.
8. W9: rule SCR forbids shell scripts, so do not write one and do not raise it with the owner yet.
   Find a form without a script (a link to an existing bin, a TypeScript bin in a repository package, a Rust bin,
   a mise or systemd setting) and tell the coordinator what the two lines must do and which forms were tried.
   One candidate: a bin in `package/repository-link/check` that, invoked through a link named `spawn-claude` or
   `spawn-pi`, runs the check and then the program that name maps to.
9. Order of work: W2, W3 (when told), W4 and W5, W10, W6, W7 and W9, W11, W12 to W15, then the releases and lock moves.
   Keep this handover current after every merged item.
10. Merge gate for the wrapper: merge main first, `mise run toolchain:install`, the package lint (0 warnings), Clippy,
    one `native:test:container` unit; merge `--no-ff` only on a full pass. The repository-lint branch that reflows doc
    comments in the package merges first: use the recipe in `package/git-policy/cli/README.md`,
    "Merging main into a branch that edited doc comments".
    The wrapper is the one crate not released by the agent: report its change, the owner releases it.

## Next, in order

W2 (nothing written yet):
the TS W1 table in `check-link.unit.test.ts` (world: entries, links, real, texts, uuids, mounted) is the model for the
JSON table; the TS answers and messages in `package/repository-link/check/src/check-link.ts` are the text to match.
Hot-path facts to keep: phase `before` is one `lstat` plus a UUID compare; the identity file read is itself an
open and parse on every command, so measure it.
Test list from the plan: the dangling fixture asserts the message, status 128, and no new directory under the service
state folder or the runtime folder; a worktree-through-missing-link fixture asserts the commit message names the
repository, not the working folder; alternate spellings (`/home/user/Monochromatic`, the canonical root, the stale NVMe
path) and `GIT_DIR`, `GIT_WORK_TREE`, `GIT_COMMON_DIR` and a worktree `.git` file.

W4 (`package/mpm-shadow/cli`): merge main first (it carries the shadows' identity and arrival marker work; keep
`shadow_identity.rs` byte-identical across the four crates, a test fails otherwise); re-read `run_launcher` and the lock
read (`.ok()`) and `ensure_launcher`'s `create_dir_all`, whose line numbers moved.
W5 (`cargo-shadow`, `pnpm-shadow`, `search-shadow`): only the launcher fixture with a missing `.launch` target.
W10 (`package/env-local-json/core`): the cause text from W1 in `EnvLocalJsonPathError` and `EnvLocalJsonReadError`,
dangling-path tests in the reader and writer.
W6, W7, W9: user configs (`~/.claude/settings.json`, `~/.claude.json`, `~/.codex/config.toml`, links in `~/.local/bin`)
through `repository-link-run` at its installed path; keep a copy of each original under
`${HOME}/temp/agent/860-w-items/` and list every file changed.
W11: the `mount-860.service` edit as a runner stage for the coordinator (never run `sudo`).
W12 to W15: the checks after the cut-over, added to the runner and the runbook.
Then release the changed crates as alphas through the release tool and lock and install; the wrapper is the exception.

## Things a fresh agent should not repeat

- `btrfs subvolume show` is refused to a user; `btrfs send` needs root, so send behavior is only checked by the coordinator
  on the fixture (`prepare`), through the commands this session sent as messages.
- Never merge into main while another agent's merge is in progress in the main checkout (check `git status` there).
- `git commit` through the wrapper may rewrite a file's final newline: check `git status` after.
