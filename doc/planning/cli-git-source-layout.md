# Source layout of the Git wrapper crate (issue 6261, third proposal)

Status: proposal, nothing moved.
Written 2026-11-04T16:59:01-05:00 on branch `refactor/git-policy-cli-source-layout` from `main` at `c3f6209fa8`.
The owner decides the questions in "Decisions for the owner" before any file moves.
The mapping of every current file to its new path is `doc/planning/cli-git-source-layout.mapping.tsv`
(paths relative to `package/git-policy/cli`; columns are named in its first row).
It is produced by rules, not typed by hand, and the move tool applies the same rules again on the latest `main`.

## What the owner asked for

Issue 6261 (Aquaticats/Monochromatic) body, required work:

- "Group existing small production files, their sidecar tests, and subsystem-specific support into cohesive subsystem
  directories inside the same crate.
  Choose boundaries from the actual module relationships, using filename families as starting evidence rather than an
  automatic prefix-only partition."
- "Make the crate root a readable map of those subsystems rather than a catalogue of hundreds of implementation files.
  Provide a short navigation guide explaining where the major concerns live and where new files belong."
- "As part of choosing the target layout, evaluate removing the redundant `native` layer now that the source tree no
  longer contains a parallel TypeScript implementation.
  This is a source-layout decision, not permission to rename shipped binaries or alter command behavior."
- "Keep one crate and preserve behavior.
  This is not a rewrite or a mandate to add abstraction layers.
  Make focused, reviewable moves with minimal semantic edits so history remains traceable."

Owner reopen comment, 2026-11-04T21:30:23Z
(https://github.com/Aquaticats/Monochromatic/issues/6261#issuecomment-6642606105):
"Reopening after another look at the directory layout.
It has improved, but only a little.
The current layout is still completely unacceptable and far from enough to consider this issue complete.
The original goal remains: recognizable, cohesive subsystem directories and a readable crate-root map, with clear
placement guidance, while preserving the small-file decomposition and file-size rule.
This needs substantially more work before it can be closed."

## Why the last round fell short

The last round is merge `fec77945dd`.
It made one level of folders under `src/native` and re-exported each with `pub use <folder>::*;`.
Measured on `main` at `c3f6209fa8`, the result still has these properties, each of which matches part of the rejection.

- Folders are filename prefixes moved into directories.
  The largest folders still hold the old prefix inside every name:
  `landing/service_index_*` (fifteen files), `worktree/worktree_trash_*`, `subvolume/subvolume_*`,
  `binary/binary_*_tests.rs`.
  The issue asked for the opposite: "filename families as starting evidence rather than an automatic prefix-only
  partition".
  The biggest folder holds 101 Rust files and the next ones 89, 77 and 66, so a reader still scrolls a flat list.
- The crate-root map is cosmetic.
  `lib.rs` is 305 lines of `pub mod x;` and `pub use x::*;` pairs.
  The glob re-exports keep every one of over a thousand module names unique and reachable as
  `crate::<name>` from anywhere,
  so the namespace a reader and the compiler see is as flat as before.
  Nothing in a `use` line names the subsystem.
- Names say little.
  `binary`, `foundation`, `testing`, `shadow`, `landing` and `lock` do not tell a newcomer what is inside.
  `binary` is not a subsystem at all: it is the root of an integration test target.
- Several modules sit in the wrong folder.
  The module graph (see "The module graph" in "Evidence used") shows `transaction_git` ("run real Git")
  used from twenty of the other folders but
  filed under `transaction`;
  `subvolume_sha` is a hash helper used by the index and landing code but filed under `subvolume`;
  the `fs_*` filesystem primitives are used by worktree, health, conflict, landing and lock code but filed under
  `subvolume`;
  `wait_ceilings` is used from eleven folders but filed under `service`;
  `child_watch`, `push_progress`, `wait_progress` and `start_retry` judge that any child is at work and are used by
  candidate, commit, invocation, landing, lfs, lock, markdown, process, repair, repo_read and transaction code but are
  filed under `push`.
- Placement guidance stopped at the folder.
  The README section "Source layout" says what each folder is.
  It does not say where a sidecar test, test support, a data file or a second-level group goes.
- The `native` layer was left in place ("Removing the `native` layer is deferred", handover entry for 6261).

## Evidence used

### The module graph

Method: every `crate::<name>` and `super::<name>` token in the 1152 Rust files under `src/native` was matched
against the set of module names (every module name is unique today), giving 3775 module-to-module edges.
Test sidecars are ignored when judging production dependencies.
The scripts are kept with the report for now and become part of the move tool in step 4.

Findings that shaped the tree:

- The crate is one strongly connected cluster: 28 pairs of the proposed areas depend on each other in both
  directions.
  The tree therefore groups by concern and cohesion, not by a layering the code does not have.
- The strongest parent and child relations are worktree code on btrfs subvolume code (67 edges),
  landing code on the service (63) and the service on landing code (28),
  the commit path on policies (45),
  policies on candidates (26),
  and index code on repository reads (16).
  These pairs are kept in one area each (`worktree`, `service`, `policy`, `repository`).
- The index lock is one protocol implemented in two folders:
  `landing/service_index_*` (the service holds `index.lock`) and `lock/index_*` (other programs wait on it).
  They call each other, so they are one directory tree, `repository/lock/index/`.
- `argument` helpers (`global_arguments`, `escape_hatch`) lived in `invocation` but were consumed by the `command`
  tables and rules.
  They move under `command/argument/` so `command` no longer depends on `invocation`.
- Known upward edges that this move records and does not repair
  (repairing them is a design change and outside this issue):
  `repository/git/runner` calls the service's busy flag, `repository/git/inner_words` writes to the service journal,
  `platform/process/watch` calls into `push/bounded_push`.

### Precedent

Read on 2026-11-04 with `gh api repos/<owner>/<repo>/contents/<path>`.

- `rust-lang/cargo` at `46a44395eb`:
  `src/` holds `bin`, `compiler`, `config`, `context`, `diagnostics`, `lib.rs`, `macros.rs`, `ops`, `resolver`,
  `sources`, `util`, `version.rs` and `workspace`.
  There is no `src/cargo/` layer and no `core/` folder at that commit
  (the brief for this round expected `src/cargo/{core,ops,sources,util}`, which no longer exists).
  Area names are domain nouns, the root is a short list, and a folder holds tens of entries
  (`util`: 43 entries, 3 subfolders; `ops`: 26 entries, 7 subfolders; `compiler`: 26 entries, 5 subfolders).
  Integration tests are not under `src`: `tests/testsuite` holds 206 entries in 48 subfolders.
- `tokio-rs/tokio` at `695a07dd5e`:
  `tokio/src` has 17 entries (`fs`, `io`, `net`, `process`, `runtime`, `signal`, `sync`, `task`, `time`, `util`...).
  `runtime` has 26 entries and nests by role to a third level
  (`runtime/scheduler/{current_thread,multi_thread,inject,util}`).

What this teaches the layout: short root of domain nouns, nesting by role until each folder is a dozen or two entries,
no redundant outer layer, integration tests outside `src`.

### Repository conventions that constrain the layout

- `rust/tests-in-sidecars` (monochromatic-lint README, "Rust test sidecars"):
  a sidecar is recognized by name, `*_tests.rs` or `tests.rs` or any file under a `tests` directory.
  The declaration is `#[cfg(test)] #[path = "thing_tests.rs"] mod tests;`, so a sidecar must stay beside its subject
  and move with it.
- The file-size rule (`AGENTS.md`, rule MXL) stays: split into sibling files, never strip docs to fit.
  Nothing here merges files.
- The source-boundary guards of issue 6260 live in `src/native/source_guard` and encode the layout:
  `source_scan_tests.rs` hard-codes `SOURCE_PREFIX = "src/native/"` and `source_root()` as `src/native`,
  `service_lock_inventory_tests.rs` lists covered file-name prefixes and checks each reaches a source,
  and `no_self_invocation_*` and `standards_review_boundary_*` name files by their path relative to `src/native`.
  Moving files changes those keys, so the guards are updated in the same slice as the files they name.
  A rename that strips prefixes (D3, strip) changes more of them than a move that keeps names.

## Target tree

Read each entry as `directory`: purpose.
Every directory under `src` also holds a `mod.rs` that declares its modules with one documented line each,
and each module has its sidecar `<name>_tests.rs` beside it.
Directories marked Data hold files that are not Rust.

  - `invocation/`: From arguments to Git: decide what one invocation does, then do it.
    - `forward/`: Starting real Git with the caller's exact arguments, streams and environment.
    - `management/`: The wrapper's own `git cli-git ...` words: grammar, selection and the decision for each.
      - `doctor/`: The `doctor` report: hooks, configuration, filesystem and fsmonitor checks.
    - `real_git/`: Finding, checking and recording the real Git executable, and refusing to start the wrapper as
      Git.
  - `command/`: What the wrapper knows about Git commands: option grammar, per-command facts and the rules that
    act on a command line.
    - `argument/`: Reading a command line: global options, wrapper-only control tokens, escape hatches, alias
      expansion and built-in names.
    - `branch/`: Branch-creation facts of `git branch`, `git checkout` and `git switch`, with their option
      tables.
    - `commit/`: The `git commit` option table and the facts read from it.
    - `conflict_side/`: Taking one side of a conflicted file: the decision, the guard, the saved copy and the
      notices.
    - `discard/`: Commands that can discard uncommitted state: the facts, the audit record, the saved copies and
      their pruning.
    - `git_config/`: The `git config` option groups and the form and scope facts read from them.
    - `options/`: The tokenizer that reads one command's arguments the way Git's option parser does.
    - `rules/`: Pure decisions of the transforms and policies that rewrite, refuse or hint at a command line.
      - `commit_only/`: The commit-only transform: decision, message, index check and sequencer check.
      - `linked_worktree/`: The linked-worktree-only and branch-worktree-only policies, with the effective
        target they judge.
    - `stash/`: The `git stash` subcommands and their option tables.
    - `testing/`: Option-table oracles and fixtures shared by the command tests.
  - `config/`: Reading and validating `cli-git.config.jsonc` for one invocation.
    - `git/`: Reading Git's own configuration: the one snapshot and the `core.fsync` value.
    - `section/`: Validation of the policy, rules-file and discarded-copies sections.
  - `policy/`: What is checked: the policy engine, the shipped policies and the candidate content they judge.
    - `candidate/`: Candidate content: what a commit or push would add, as immutable versions with identity,
      mode and bytes.
      - `read/`: Reading candidates from Git: object batches, the long-lived reader, raw records, staged entries
        and trees.
    - `direct_fix/`: The direct `git cli-git fix`: converge corrections in memory, then install them.
    - `engine/`: The registry, triggers, passes and convergence that run shipped policies for one lifecycle
      point.
    - `event/`: The events a pass produces: their JSON Lines form, paths and hints.
    - `scanner/`: The forbidden-strings scanner linked in-process, and which rules and candidates it is given.
    - `shipped/`: The shipped policies, one module or folder each.
      - `dependent_version/`: The dependent-version policy: decision, check and prepared state.
        - `plan/`: Walking the dependency graph and planning the patch bumps.
        - `testing/`: Differential fixtures and an in-memory workspace for the planner's tests.
          - `fixtures/`: JSON fixtures read by the differential tests.
        - `workspace/`: Reading workspace manifests, publishable names, imports and bundled development
          dependencies.
      - `markdown/`: The `markdown/autofix` policy and its in-process linter.
  - `commit/`: The wrapper's own commit path, from arguments to a settled commit.
    - `conclusion/`: What a settled commit leaves behind: events, merge and cherry-pick conclusion state, the
      post-commit gate.
    - `prepare/`: Preparing a commit in the shadow repository against a private index.
    - `replay/`: Replaying a prepared change onto a branch that moved.
      - `subsumption/`: Deciding that a landed change already contains the prepared one.
    - `shadow/`: The shadow repository one transaction prepares on.
  - `service/`: The resident commit service: its program, its protocol, and the landing and after-commit work it
    performs.
    - `client/`: The wrapper's side: routing a commit to the service, waiting honestly, and handing off
      after-commit work.
    - `landing/`: The serialized step that lands a batch of prepared commits on a branch.
      - `commit/`: Reading and building the commit being landed: amend facts, messages, native commits and
        captures.
      - `index_update/`: The landing's in-process update of the live index and its publication.
    - `protocol/`: The messages between the wrapper and the service.
    - `provision/`: Making socket activation exist for a worktree: unit files, the user bus and the journal
      protocol.
      - `units/`: The `systemd` template unit files the service installs.
    - `review/`: The standards-review shadow run the service performs on landed commits.
      - `gateway/`: The review's one HTTPS call, its request body and the loopback server that tests it.
    - `server/`: The service loop: accepting requests, one landing thread, the journal, timing and signals.
    - `work/`: After-commit work items: claim, lease, retry, rounds and recovery.
      - `maintenance/`: Automatic maintenance after a landing: the decision and the run.
  - `push/`: Pushes: bounded Git pushes and queries, failure notices, and the deadline advice.
    - `auto/`: The automatic push of a landed commit: destination, reconcile, single flight and trigger.
    - `lfs/`: Git LFS support: pre-push planning, pointer scanning and the interactive step.
      - `upload/`: The LFS upload: plan, settings, object plan, output mask and the run.
    - `manual/`: Manual push handling: the dry-run probe, the commit range, the porcelain parser and the scope.
    - `retry/`: The retry plan for a push that was stopped or failed, and its on-disk store.
    - `stacking/`: An explicit push that lost a race: merge on the remote tip and push the merge.
  - `merge/`: Merges and the conflicts they leave.
    - `isolation/`: A merge completed in isolation when staged changes make native Git refuse it, with recovery
      and pull handling.
    - `second_pass/`: The word-level second pass over files a merge left conflicted, with the word merge and its
      corpus.
      - `word_merge_corpus/`: Data: the corpus the word merge is tested against.
  - `worktree/`: Worktree commands and the copying, repair and btrfs support around them.
    - `add/`: Serving `git worktree add` cheaply: plan, binding, timing and the slow-route sentence.
      - `ladder/`: The cheap-worktree ladder: choice of rung, clone rung and finish.
      - `snapshot/`: The snapshot rung: source check, snapshot, attribute refusals and dirty-state restore.
      - `trash/`: The leave-out of a snapshot: setting aside, disposing and removing the trash.
    - `clean/`: Whether a worktree is clean enough to copy from or remove, by state and by content.
    - `convert/`: Converting a linked worktree into a btrfs subvolume, resumably.
    - `copy/`: Copying the ignored state of a linked worktree into a new one.
      - `install/`: Installing a validated stage into the new worktree, with the install log and rollback.
      - `journal/`: The copy's durable journal, its owner record and recovery.
      - `stage/`: Selecting, comparing and staging what the copy transfers.
    - `remove/`: Removing a worktree with one subvolume destroy: plan, registry, rescue of unshown notices.
    - `repair/`: Package-manager and toolchain repair after a worktree is created.
    - `subvolume/`: Btrfs subvolume migration of a main checkout: command, platform operations and preflight.
      - `index_rewrite/`: Rewriting the stat data of Git index files after a move.
      - `migrate/`: The migration steps: migrate, rollback, anchor, journal and locks.
      - `remove/`: Removing a tree without crossing mounts or nested subvolumes.
      - `tree_sync/`: Making one directory tree match another in process, with attributes.
  - `repository/`: The Git repository as data: in-process reads, the index file, locks, durable transactions and
    running real Git.
    - `git/`: Running real Git: captured output, private index and object store, metadata queries and inner-run
      reporting.
    - `index/`: In-process reading and writing of the Git index file.
      - `extension/`: The index extensions: cache tree, EWAH, fsmonitor, resolve-undo and untracked cache.
      - `format/`: The file format: stat fields, flags, settings and the full index with its writer.
      - `reset/`: Resetting paths in an index held in memory, the way `git reset <tree> -- <paths>` does.
    - `lock/`: Locks and their owners: the index lock, stale reference locks and single-owner claims.
      - `index/`: Git's own `index.lock`: the commit service's hold and every program that waits on it.
        - `foreign/`: A lock held by another program: evidence, holders, waiting, and forwarding index writers.
        - `hold/`: The service's hold on one index lock across a landing: placing, attempting, waiting and
          releasing.
        - `marker/`: The marker written into a held lock, who it names, and its staging files.
        - `reclaim/`: Removing a stale index lock by one reclaimer at a time.
      - `stale_ref/`: Reference locks a killed Git left behind: found, judged and removed on proof.
        - `forward/`: Clearing reference locks around the caller's own fetch, push and stopped children.
    - `read/`: Reading the repository without a Git process.
      - `config/`: Configuration files read in process, with the proofs that stand in for Git's own parse.
      - `layout/`: Where the repository, its worktree and its Git directories are, and the facts policies ask
        about them.
      - `object/`: Objects: the database, packs, trees, abbreviations.
      - `query/`: In-process answers to the read-only command lines the wrapper issues.
      - `reference/`: References: files ref storage, query answers and sequencer markers.
    - `transaction/`: Durable transaction state: owner records, journal, phases and fail-closed recovery.
  - `platform/`: Operating-system facts and small shared building blocks that depend on no feature.
    - `data/`: Small value helpers: JSON records, ISO time, JavaScript text semantics, paths, hashes and shell
      quoting.
    - `fs/`: Linux filesystem primitives: beneath-handle operations, btrfs ioctls, clone and exchange, mount
      options.
    - `parallel/`: Bounded parallelism: how many workers, and running items on them in order.
    - `process/`: Process identities, trees, signals, output capture and start retries.
      - `watch/`: Judging that a child is still at work: progress meters, quiet windows, wait notices and
        sockets.
    - `storage/`: Private crash-safe files, scratch locations and random identifiers.
  - `testing/`: Test-only fixtures shared by many subsystems; never part of the release executable.
    - `source_guard/`: Controls that read the crate's own source text.
- `src/`: The crate: `lib.rs` is the map of the areas it declares and `main.rs` is the thin executable.
- `tests/`: Integration tests that reach the crate only through its public interface or its built executable.
  - `binary/`: The `native_binary` test target: controls that start the built executable against real Git.
    - `commit/`: Commits, policies, event paths and budgets.
    - `harness/`: Fixtures, the hang guard, fault injection and tracing shared by the binary controls.
    - `invocation/`: Forwarding, resolution, management words, hooks-off and version checks.
    - `merge/`: Merges, pulls, word merge and conflict sides.
    - `push/`: Pushes and LFS.
    - `service/`: The resident service, landings and upgrades.
    - `worktree/`: Worktree add, copy, convert, subvolume and repair.

## Crate-root map as it would read in `lib.rs`

This is the module documentation and declarations the move writes into `src/lib.rs`.
It assumes the owner accepts option B1 of D2 (hierarchical paths, no glob re-exports);
under B2 the same text stays and each `pub mod x;` is followed by `pub use x::*;`.

```rust
// src/lib.rs

//! What:
//!  The Git policy wrapper as a library: the decision for one invocation, the policies, the commit path and the
//!  resident commit service.
//! Why:
//!  The executable stays thin (`main.rs`), and every decision lives here where tests can reach it.
//!
//! Where things live:
//!  - `invocation`: from arguments to Git. Start here to follow one command.
//!  - `command`: what the wrapper knows about Git commands, and the rules acting on a command line.
//!  - `config`: `cli-git.config.jsonc` and Git configuration.
//!  - `policy`: what is checked: the engine, the shipped policies and the candidate content they judge.
//!  - `commit`: the wrapper's own commit path: prepare, shadow repository, replay, conclusion.
//!  - `service`: the resident commit service, its protocol, landing and after-commit work.
//!  - `push`: automatic and manual pushes, retries, Git LFS.
//!  - `merge`: isolated merges and the word-level second pass.
//!  - `worktree`: worktree commands, copying, repair and btrfs subvolume support.
//!  - `repository`: the repository as data: reads, index file, locks, transactions, running real Git.
//!  - `platform`: operating-system facts and small shared building blocks.
//!  - `testing`: test-only fixtures and source guards (compiled for tests only).
//!
//! Where a new file goes: README "Source layout".

pub mod invocation;
pub mod command;
pub mod config;
pub mod policy;
pub mod commit;
pub mod service;
pub mod push;
pub mod merge;
pub mod worktree;
pub mod repository;
pub mod platform;

#[cfg(test)]
pub mod testing;
```

Each `pub mod` line carries a documented one-liner in the real file, like the present `lib.rs`.
The result is a crate root of a dozen areas where the present one has 29 declarations and as many re-exports.

## Navigation and placement guide

This text is for the package README ("Source layout") and for `lib.rs` module docs.

Reading a path.
`crate::<area>::<group>::<module>` is the whole address.
`service::landing::follow` is "the service, the landing step, what follows the branch move".
Inside a group the file name does not repeat the group:
`repository/lock/index/hold/attempts.rs`, not `service_index_attempts.rs`.

Where a concern lives.

- Following one command from `argv` to real Git: `invocation`, then `command` for the grammar it asked.
- A policy or its message: `policy` (`policy/shipped/<name>` for one policy, `policy/engine` for how they run).
- What the wrapper does for `git commit`: `commit` (wrapper side) and `service` (service side).
- Anything about `index.lock`: `repository/lock/index`.
- Reading objects, refs or configuration without Git: `repository/read`.
- A `git worktree` subcommand or btrfs conversion: `worktree`.
- A process, a file primitive, a timestamp, a JSON record: `platform`.
- A test double or fixture used by several areas: `testing`.

Where a new file goes.

1. Name the concern, then find the deepest directory whose purpose line (in its `mod.rs`) already covers it.
2. Put the production file there.
   Do not repeat the directory name in the file name, and do not name a file like its directory
   (clippy `module_inception`).
3. Put its sidecar test beside it as `<name>_tests.rs`, declared in the subject with
   `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;`.
4. Test support used by one directory: `test_support.rs` beside the code;
   by several files of one area: a `testing/` folder in that area;
   by several areas: `src/testing`.
5. Data a module reads (fixtures, corpora, unit files): a named folder beside the module that reads it.
6. A directory that reaches a dozen modules (about two dozen files with sidecars) gets a subdirectory by role,
   never a longer prefix.
7. Declare the file with a documented `pub mod` line in the directory's `mod.rs`, and use `crate::` paths.
   `super::` is for a sidecar's `use super::*;` only.
8. Controls that start the built executable go in `tests/binary/<group>/`;
   controls that use only the public library interface go in `tests/`.

## Before and after

Four subsystems, production modules only (each has its sidecar beside it in both layouts).
The full mapping is the TSV.

### The index lock

Before, two folders and two prefixes; after, one tree by role.

```text
landing/service_hold_retry.rs  ->  repository/lock/index/hold/hold_retry.rs
landing/service_index_attempts.rs  ->  repository/lock/index/hold/attempts.rs
landing/service_index_claim.rs  ->  repository/lock/index/reclaim/claim.rs
landing/service_index_found.rs  ->  repository/lock/index/hold/found.rs
landing/service_index_hold.rs  ->  repository/lock/index/hold/held.rs
landing/service_index_identity.rs  ->  repository/lock/index/marker/identity.rs
landing/service_index_lock.rs  ->  repository/lock/index/hold/acquire.rs
landing/service_index_marker.rs  ->  repository/lock/index/marker/content.rs
landing/service_index_ownership.rs  ->  repository/lock/index/marker/ownership.rs
landing/service_index_placement.rs  ->  repository/lock/index/marker/placement.rs
landing/service_index_prestage.rs  ->  repository/lock/index/marker/prestage.rs
landing/service_index_reclaim.rs  ->  repository/lock/index/reclaim/remove.rs
landing/service_index_same.rs  ->  repository/lock/index/hold/same.rs
landing/service_index_staging.rs  ->  repository/lock/index/marker/staging.rs
landing/service_index_wait.rs  ->  repository/lock/index/hold/wait.rs
landing/service_index_watch.rs  ->  repository/lock/index/hold/watch.rs
lock/index_holder_wait.rs  ->  repository/lock/index/foreign/holder_wait.rs
lock/index_lock_evidence.rs  ->  repository/lock/index/foreign/evidence.rs
lock/index_lock_holder_parsers.rs  ->  repository/lock/index/foreign/holder_parsers.rs
lock/index_lock_holders.rs  ->  repository/lock/index/foreign/holders.rs
lock/index_lock_wait.rs  ->  repository/lock/index/foreign/wait.rs
lock/index_writer_commands.rs  ->  repository/lock/index/foreign/writer_commands.rs
lock/index_writer_forward.rs  ->  repository/lock/index/foreign/writer_forward.rs
```

### Adding a worktree

Before, these sit in `worktree/` with its other files (seventy-seven Rust files in all); after, three groups by rung.

```text
worktree/add_timing.rs  ->  worktree/add/timing.rs
worktree/checkout_workers.rs  ->  worktree/add/checkout_workers.rs
worktree/worktree_add_binding.rs  ->  worktree/add/binding.rs
worktree/worktree_add_plan.rs  ->  worktree/add/plan.rs
worktree/worktree_attributes.rs  ->  worktree/add/snapshot/attributes.rs
worktree/worktree_clone.rs  ->  worktree/add/ladder/clone.rs
worktree/worktree_dirty_restore.rs  ->  worktree/add/snapshot/dirty_restore.rs
worktree/worktree_ladder.rs  ->  worktree/add/ladder/around_add.rs
worktree/worktree_ladder_finish.rs  ->  worktree/add/ladder/finish.rs
worktree/worktree_ladder_run.rs  ->  worktree/add/ladder/run.rs
worktree/worktree_ladder_target.rs  ->  worktree/add/ladder/target.rs
worktree/worktree_slow_route.rs  ->  worktree/add/slow_route.rs
worktree/worktree_snapshot.rs  ->  worktree/add/snapshot/take.rs
worktree/worktree_snapshot_source.rs  ->  worktree/add/snapshot/source.rs
worktree/worktree_subvolume.rs  ->  worktree/add/subvolume_target.rs
worktree/worktree_trash.rs  ->  worktree/add/trash/leave_out.rs
worktree/worktree_trash_dispose.rs  ->  worktree/add/trash/dispose.rs
worktree/worktree_trash_entries.rs  ->  worktree/add/trash/entries.rs
worktree/worktree_trash_manifest.rs  ->  worktree/add/trash/manifest.rs
worktree/worktree_trash_remove.rs  ->  worktree/add/trash/remove.rs
```

### Git command grammar

Before, `command_branch_*`, `command_stash*` and `command_commit*` sit among sixty-six files in `command/`;
after, one small folder per Git command.

```text
command/command_branch_create.rs  ->  command/branch/create.rs
command/command_branch_mode.rs  ->  command/branch/mode.rs
command/command_branch_table.rs  ->  command/branch/table.rs
command/command_branch_target.rs  ->  command/branch/target.rs
command/command_checkout_table.rs  ->  command/branch/checkout_table.rs
command/command_commit.rs  ->  command/commit/facts.rs
command/command_commit_table.rs  ->  command/commit/table.rs
command/command_stash.rs  ->  command/stash/facts.rs
command/command_stash_table.rs  ->  command/stash/table.rs
```

### After-commit work of the service

Before, these are mixed into the forty-nine production files of `landing/`; after, one folder.

```text
landing/claim_time.rs  ->  service/work/claim_time.rs
landing/maintenance_decision.rs  ->  service/work/maintenance/decision.rs
landing/maintenance_run.rs  ->  service/work/maintenance/run.rs
landing/service_claim_renewal.rs  ->  service/work/claim_renewal.rs
landing/service_claim_watch.rs  ->  service/work/claim_watch.rs
landing/service_retry.rs  ->  service/work/retry.rs
landing/service_round.rs  ->  service/work/round.rs
landing/service_work.rs  ->  service/work/consumer.rs
landing/work_item.rs  ->  service/work/work_item.rs
landing/work_item_recovery.rs  ->  service/work/work_item_recovery.rs
```

### One module, end to end

`landing/service_index_attempts.rs` becomes `repository/lock/index/hold/attempts.rs`.

- Module path: `landing::service_index_attempts` (also reachable as `crate::service_index_attempts`) becomes
  `repository::lock::index::hold::attempts`.
- Sidecar: `landing/service_index_attempts_tests.rs` becomes `repository/lock/index/hold/attempts_tests.rs`;
  the `#[path = "service_index_attempts_tests.rs"]` line in the subject becomes `#[path = "attempts_tests.rs"]`.
- Callers: `use crate::service_index_attempts::Attempt;` becomes
  `use crate::repository::lock::index::hold::attempts::Attempt;`.
- Test names: `landing::service_index_attempts::tests::name` becomes
  `repository::lock::index::hold::attempts::tests::name`.
  The move tool lists every old name against its new name and fails on any test that has no counterpart.
- Directory: `mod.rs` of `hold/` gains `pub mod attempts;` with a one-line doc.

## Decisions for the owner

### D1: the `native` layer

Options.

- Option A: Remove it in the same pass as the move.
  `src/lib.rs` and `src/main.rs` replace `src/native/lib.rs` and `src/native/main.rs`;
  `[lib]`, `[[bin]]` and `[[test]]` paths, `include`, the guards and the references follow.
  Pros: every path changes anyway, so the rewrite of references happens once;
  standard Cargo layout; cargo itself dropped its `src/cargo/` layer.
  Cons: the biggest single change set, and it touches every place named in the blast radius that follows the ranking.
- Option B: Keep `native` in the move, remove it in a second mechanical commit.
  Pros: the move diff is smaller.
  Cons: repeats the reference rewrite, and a second window of conflicts with whatever branches are open then.
- Option C: Keep `native`.
  Pros: no reference changes.
  Cons: keeps the layer the issue calls redundant; one more path segment in every path and document.

Ranking: A > B > C, because A does the one rewrite that the move forces anyway, where B pays for it twice and
extends the conflict window with the four open branches;
B > C because C keeps a layer with no remaining reason to exist, while B only delays its removal.

Blast radius of A (measured by `rg "src/native"` on `c3f6209fa8`):
`Cargo.toml` (thirteen mentions: `[lib]`, `[[bin]]`, both `[[test]]`, `include`, comments), `mise.toml`, `clippy.toml`,
`native-test-base.Containerfile`, `SPEC.md`, `README.md`, `e2e/*`, `bin/native-*.ts` (planted-control and snapshot
tools), `package/git-policy/cli.fuzz/bin/planted-*.ts` and its README,
`package/mutation-test/cli-cargo-mutants/src/*.ts` and `golden/*.json`,
`monochromatic-lint.config.jsonc`, `slopo.ignore.txt`,
the source guards, and live docs.
Historical records (handovers, dated ledgers) keep their original text with an in-place note giving the new path
(rule WR6), not a rewrite.
The `package/ide/app` crate has a `src/native` of its own; it is unrelated and untouched.
Binary names do not change: `[[bin]] name = "cli-git-native"` and the test target names `native_binary` and
`native_candidates` stay.

### D2: module paths, and the glob re-exports

Options.

- Option B1: Hierarchical paths, no glob re-exports.
  The address of a module is its directory path.
  Pros: the tree is what the compiler and the reader see; a `use` line names its subsystem;
  the root namespace stops being a thousand names; renames can strip prefixes (D3).
  Cons: nearly every file's `use` lines change (3775 module edges to rewrite) and so do the roughly one hundred
  module names the fuzz crate, `main.rs` and the integration tests use through `git_policy_cli::`;
  test names all change (accounted for by the tool).
  This is a path change for in-repo consumers of an unpublished library interface;
  it is not a visibility change.
- Option B2: Nested directories, flat names kept, glob re-exports kept at every level.
  Pros: no `use` line changes; external consumers unchanged; smallest diff.
  Cons: repeats round two's property that nothing in a `use` line names the subsystem; module names must stay unique
  forever; prefixes cannot be stripped, so `repository/lock/index/hold/service_index_attempts.rs` stays.
- Option B3: Keep the present one-level folders and globs.
  Pros: nothing to do.
  Cons: this is the rejected state.

Ranking: B1 > B2 > B3, because B1 is the only one where the layout the owner asked to see is also the code's own
structure, at the cost of a mechanical rewrite the move tool must do anyway for `mod` and `#[path]` lines;
B2 > B3 because B2 at least delivers the nested directories and the readable root, where B3 delivers neither.

### D3: strip the repeated prefix from file names

Only possible with B1 (under B2 names must stay unique).
Options.

- Strip.
  Pros: a directory already says what it is, so `hold/attempts.rs` replaces `service_index_attempts.rs`;
  the issue's "not a prefix-only partition" is met in names too.
  Cons: every file name changes (only 57 of 491 production files keep their basename),
  so `git log --follow` carries the history by similarity rather than by name;
  basenames repeat across directories (at most six files share any one name, for example `tree.rs`, `journal.rs`,
  `facts.rs`), which the editor tab bar shows as the same title;
  `source_guard` keys that match by prefix need new keys.
- Keep names.
  Pros: one fewer thing to review; guard keys by prefix survive.
  Cons: `hold/service_index_attempts.rs` repeats the path it already has.

Ranking: strip > keep, because the repeated prefix is the visible symptom the owner objected to,
and the duplicated basenames are bounded and each sits in a different directory whose path disambiguates it.
Stripping needs B1 and is the deciding reason B1 outranks B2.

### D4: where the integration tests live

`src/native/binary` (one hundred and one Rust files) is the root of the `native_binary` test target and
`candidate/candidate_consumer_tests.rs` the root of `native_candidates`.
Neither is part of the library.

Options.

- Option A: `tests/binary/<group>/` and `tests/candidate_consumer.rs`.
  Pros: Cargo's own convention, and cargo's repository keeps its suite in `tests/testsuite`;
  `src` then holds library code only; groups by role (invocation, commit, service, push, merge, worktree, harness).
  Cons: `include` and the `[[test]]` paths change; the sources leave the tree `source_guard` walks, so the guards that
  must cover them need a second root;
  Cargo auto-discovers `tests/*/main.rs`, so duplicate-target behaviour with the explicit `[[test]]` entries must be
  verified before the slice lands.
- Option B: Keep them in `src`, nested as `src/testing/binary/<group>/`.
  Pros: no target discovery question; guards unchanged.
  Cons: integration code stays among library code.

Ranking: A > B, because A removes the largest folder from `src` by a rule Cargo itself documents,
where B only renames its location; the cons of A are verifiable before the move.

### D5: test support convention

Proposal written in the placement guide (rule 4): one directory, `test_support.rs`;
several files in one area, a `testing/` folder there; several areas, `src/testing`.
Alternative: always one `testing/` folder per area.
Ranking: proposed hybrid > always-a-folder, because a folder of one file is a layer with no content.
This is a convention, not a rule that exists in `AGENTS.md` today; it needs the owner's word before it is recorded
as one.

### D6: sequencing with the four open branches

Measured touch sets (`git diff --name-status <merge-base>...<tip> -- package/git-policy/cli`):

- `fix/git-policy-cli-one-executable`: `binary/` (two new files, `binary_tests.rs`), `landing/service_index_found.rs`,
  `lock/stale_ref_locks_tests.rs`, `service/service_main*`, `source_guard/mod.rs` and a new source guard test.
- `fix/git-policy-cli-side-ref-proof-round5`: about thirty files across `foundation`, `landing`, `merge`, `process`,
  `push`, `repair`, `service`, `shadow`, `standards_review`, `transaction`, `worktree_copying`;
  one new module (`foundation/scoped_start.rs`).
- `feat/git-policy-cli-commit-protocol-models`: about one hundred and forty new files, all under new
  `commit/commit_*` names and a new `model/` folder, plus `service_main.rs` as a flat file that `main` does not have
  (its merge base is `fec77945dd`, so it is stale against `main` and must be rebased first).
- `fix/git-policy-cli-claim-watch-identity-and-errors`: four files in `landing` and `service`.

Options.

- Option A: Move after all four merge.
  Pros: no rename conflicts.
  Cons: delays the issue by the slowest branch, and the owner has twice asked why this issue is not moving.
- Option B: Move everything now, in no order.
  Pros: finishes soonest.
  Cons: Git follows renames by content similarity, and small sidecar test files that a branch edited can fall under the
  threshold, so every branch meets conflicts at once.
- Option C: Slice by area, least-touched areas first, re-running the tool on the latest `main` before each slice.
  Pros: progress without waiting; a touched area waits for its branch; each commit is reviewable.
  Cons: the crate is in a mixed state between slices, so the glob-free paths of B1 need the tool to rewrite every
  `use` of a moved module in the same slice.

Ranking: C > A > B, because C makes progress where A waits and still keeps the touched areas out of the way;
A > B because B maximizes the conflicts the issue asks to coordinate.

Incoming families the mapping does not know yet:
the `commit-protocol-models` branch adds `commit/commit_*` (lanes, flusher, spool, inbox, ledger, helper launch) and
`model/model_*` files.
The tool maps unknown files by the longest matching prefix rule and stops with a list when none matches.
Provisional homes: `commit/commit_*` to `commit/lane/` (stripped), `model/*` to `commit/model/`.
Whoever owns that branch should confirm or correct this before it is merged.

### D7: visibility of the public module surface

Options.

- Keep every module `pub` as today; only paths change.
  Pros: the acceptance criterion says to reduce visibility only where compatibility permits and not to mix an API
  break into a mechanical move.
  Cons: the broad surface stays.
- Reduce now.
  Pros: one pass.
  Cons: mixes a visibility change into the move and cannot be reviewed as a pure rename.

Ranking: keep > reduce now, because the issue forbids mixing them; the audit becomes a separate change after the move,
starting from the module list external consumers actually use (about one hundred names today, from the fuzz crate,
`main.rs` and the integration tests).

## Measured result of the candidate layout

Computed from the mapping rules over `main` at `c3f6209fa8`:

- Areas under `src`: 12, against 29 folders before.
- Directories with Rust files: 123, nested at most 5 levels under `src`.
- Most files in one directory, counting its `mod.rs`: 23 (before: 101).
- Rust files mapped: 1152 of 1152; data files mapped: 197; unmapped: none;
  two files never share a new path.
- No production module is named like its directory.

## Migration plan (step 4, after the owner answers)

1. A re-runnable tool `bin/native-source-layout.ts` (run with `node`) reads the rule table, lists the current tree,
   and refuses unmapped files.
   Its output on an unchanged tree is empty (idempotent).
2. Each slice moves one area with `git mv`, rewrites `mod` lines, `#[path]` lines, `use` and inline `crate::` and
   `super::` paths, Cargo targets, scripts and live docs, writes the directory `mod.rs` documents from the purpose
   lines in this document, and updates the guards that name the moved files.
3. After each slice: `cargo check` for the library, the binary, the integration tests and the fuzz crate,
   `mise run //package/git-policy/cli:lint:types`, and the source guards.
4. Test discovery is compared with `cargo nextest list` before and after.
   The tool maps each old name to a new name by the module mapping and reports names that appear on one side only.
   Zero unaccounted names is the pass condition.
5. One commit per slice, naming every new file in its pathspec.

## Open questions and unknowns

- Whether Cargo accepts the explicit `[[test]]` entries beside auto-discovered `tests/binary/main.rs` without
  `autotests = false`: unverified, to be probed on a throwaway before D4 option A lands.
- Whether clippy `module_inception` is enabled for this crate: `Cargo.toml` lints do not mention it, and the layout
  avoids it regardless.
- The source guards' prefix inventories (`service_lock_inventory_tests.rs`) need a design decision for B1 plus
  stripping: key by new path, or by role directory.
  The proposal assumes new path keys.
- `commit-protocol-models` homes (D6).
- Several modules are larger than the file-size rule's usual range
  (`worktree/copy/journal/encoding.rs`, formerly `worktree_copy_journal.rs`, is over a thousand lines).
  This move does not split them, as the rule says splits are a separate change.
