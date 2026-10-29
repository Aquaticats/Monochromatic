# cli-git: fewer processes and fewer syncs per commit, and retry on process exhaustion

Status: accepted for implementation by the user (2026-10-29), scope below.
Owner package: `package/git-policy/cli`. Tracking issue: 1804 (follow-up work found while stabilizing it).

## Why

A wrapped `git commit` of one explicit path starts about 45 real Git processes and makes about 41 sync calls,
against one process and no syncs for direct Git. Measured on an idle host with the debug and release builds alike:

- wall time about 180 ms per commit, of which about 77 ms is inside sync calls (strace, same fixture);
- the container gate takes 12 to 32 seconds for the commit tests under host load, and the same suite takes about 67
  seconds when the host is quiet;
- a host-wide fork-failure episode (the user cgroup recorded 598 limit events) failed commits with
  `Resource temporarily unavailable (os error 11)`.

Every process start and every sync is multiplied by load, so the fix is to do less of both and to ride out a short
refusal to start a process.

## Decisions (user)

- Process-exhaustion safeguard: retry failed starts everywhere, and also retry read-only and shadow-repository Git
  commands that fail inside themselves (`unable to create thread`, `cannot fork`), with an allow-list.
- Spawn reduction: save as many processes as possible, engineering the replacement in the wrapper where the result is
  provably identical to Git's, in the manner of `package/forbidden-strings/core` (own engine, an
  `OPTIMIZATIONS.md`-style log of measured wins and rejections, oracle-checked parity).
- Journal syncs: option B of the earlier question (sync at the barriers before side effects), after a crash
  harness exists that tears, drops and reorders records.

## Method

- Parity first: every in-process replacement is tested against real Git (`git` is the oracle) over generated cases,
  and falls back to spawning Git for any shape it does not handle exactly. A fallback is never a failure.
- Every replacement shows its win with the same scripts: `GIT_TRACE2_PERF` process counts and `strace` sync counts.
  A permanent budget test per measure ratchets down after each slice (a regression fails the host and gate suites).
- A guard test is shown failing before the change is kept (the repo's rule).
- What is measured and rejected is recorded here, with the number or the counterexample.

## Process-start retry (slice R)

A start that fails with `EAGAIN` (`ErrorKind::WouldBlock`) started nothing, so retrying is always safe.

- One helper starts every child: 10 ms backoff doubling to 500 ms, a 5 s total budget, one warning per retry, then the
  existing error plus the attempt count. It covers the roughly 20 call sites (`run_git`, bounded children,
  forwarding including the `execve` path, hooks, signal relay, post-commit, probes).
- Git commands that start and then fail inside themselves are retried only from an allow-list of commands that are
  harmless to re-run (read-only queries, and the idempotent shadow-repository steps), matched on the exit status and
  the wording of the failure; mutating steps (reference updates, landing) are not retried, and recovery already
  completes or rolls back those.
- Tests: a unit test with a start that refuses K times; the fork-failure sweep gains burst cases
  (`--inject=clone3:error=EAGAIN:when=N..N+K`), which must now succeed for bursts shorter than the budget.

## Candidate replacements, with the exactness condition for each

Measured process lifetimes of one commit (strace): `findmnt` 3 runs about 25 ms, `rev-parse` 12 runs about 32 ms,
`config` 7 runs about 16 ms, `git hook` 4 runs about 14 ms plus the wrapper's own hook shim re-executions.

- Hook shims for events with no hook: the shadow commit points `core.hooksPath` at shims that re-enter the wrapper,
  which takes the hook lock (a synced lock candidate) and runs `git hook run --ignore-missing <event>`. An event with
  no user hook needs no shim. Exact when conservative: the shim is omitted only if the user's hooks directory has no
  entry of that name at all (any mode, because Git prints a hint for a non-executable file) and the configuration has
  no `hook.<name>.event` or `.command` key; any other shape keeps today's behavior. Saves up to four wrapper
  re-executions, four `git hook` runs and four synced lock candidates.
  Done (slice S1): `package/git-policy/cli/src/native/hook_presence.rs` decides per event, `write_hook_dispatch`
  writes only the entries whose event may have a hook, and `run_post_commit_hook` skips the author lookup, the lock
  and `git hook run` for `post-commit` when no entry of that name exists when it comes due (looked up then, not at
  planning, so a hook that `pre-commit` installs still runs). An error other than "not found" while looking keeps the
  entry. The replay's `pre-commit` re-run is skipped when its entry was omitted. Parity:
  `binary_hook_shim_tests.rs` runs a wrapped and a direct commit over fifteen shapes (no hooks, all executable, one
  event, not executable, links, configuration hooks, relative, absolute, empty, missing and `-c` hooks paths,
  disabled events, a linked worktree, a hook installed during the commit, a failing hook) and compares status,
  output, hook log, message and tree.
- `findmnt` (3 runs): the filesystem identity of `.git` and `index.lock`. To be replaced by `statx`, `/proc/self/mountinfo`
  and `/dev/disk/by-uuid` when the mapping is unambiguous; otherwise `findmnt` stays. Needs a check of how the
  identity is consumed before choosing (recorded below when read).
- One configuration snapshot (`git config --list -z --show-scope`) in place of the separate `config` runs
  (`core.hooksPath`, the four `hook.<event>.enabled` reads already batched, the maintenance keys, ref storage).
  Last value wins, `-c` and `GIT_CONFIG_*` values come last, bool canonicalized by Git's rules; `--type=path` values
  starting with `~` or `%(prefix)` fall back to a spawn.
  Done (slice S3): `config_snapshot.rs` reads `git config --list -z --show-scope` once, in `capture_invocation_layout`,
  and answers `core.hooksPath`, the four `hook.<event>.enabled` switches, the maintenance keys (bool-or-int text),
  whether any `hook.<name>.event` or `.command` key exists, the files ref storage, and the shadow's repository
  format. Any entry it does not model makes that lookup `Unhandled` and the caller starts Git for the key, as
  before: a valueless or empty path, a path starting with `~`, `%(` or `:(`, a boolean or integer with a unit suffix,
  a sign, a leading zero or hexadecimal, a bad boolean, text that is not UTF-8. `git config` formats every entry of
  a key, so one unmodelled entry makes the whole key unhandled even when it is not the last. Details found by probe
  on Git 2.56.0: listing order is precedence order (files, then `GIT_CONFIG_COUNT`, then `GIT_CONFIG_PARAMETERS`
  with `-c` appended last, all in the `command` scope); `--type=bool` of `1k` is `true` and of `08` is fatal;
  `--type=bool-or-int` of `007` prints `7`; `--type=path` of a valueless key is fatal.
  Differential tests: `config_snapshot_tests.rs` draws 60 deterministic configurations over files, includes,
  conditional includes, a global file, `GIT_CONFIG_COUNT`, `-c`, repeated and valueless keys and bad values, and
  compares each lookup with `git config --type=<kind> --get`; it counts answered and declined lookups so a run
  that declines everything fails.
  Which consumers share the read: the plan (hooks path, switches, named hooks) always, because it reads with the
  caller's global options as the snapshot does; the ref storage, the maintenance keys and the shadow format only
  when the read carried no global options, because their original reads ignored the options or ran elsewhere.
  The ref storage is proven `files` only when no entry names `extensions.refstorage`; any listing that names it
  asks Git. The shadow format comes from the `local` scope only when no `include.path` or `includeIf.*.path` key
  exists, because `git config --file config --list` reads that one file without includes.
- Empty-tree object name: the hash of `tree 0` under the repository's object format (a constant per format) in place of
  `git hash-object -t tree --stdin`.
- `symbolic-ref HEAD` (files ref storage): read `HEAD`; anything but `ref: <name>` or a full object name falls back.
  The landing re-read stays a read of the same file, so the compare-and-swap protection is unchanged.
- Sequencer markers (`MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`) on the files backend: when none of the
  loose files (including the six lookup spellings `rev-parse` tries) exists and `packed-refs` does not contain the
  name, they are absent without a spawn; any hit falls back to `rev-parse --verify`. A dangling marker therefore
  stays a conclusion (pinned by `a_marker_naming_a_missing_object_is_still_a_conclusion`).
- `git version`: cached per binary identity (path, size, mtime, inode) in the repository's private directory.

## Rejected, with evidence

- Reading the three sequencer markers with `cat-file --batch-check`: it reports a marker that names a missing object
  as missing, so a corrupt merge would become a plain commit that drops the merge parent.
- Merging `core.hooksPath` with the maintenance reads by hand: different phases and a `--type=path` value; covered
  instead by the snapshot above.
- Capping Git's threads (`pack.threads=1`, `core.preloadindex=false`): 101 ms against 105 and 100 ms baseline, and
  the container was never throttled (`nr_throttled 0`).
- Re-reading `HEAD` and the target ref once instead of twice: the second reads at landing are the compare-and-swap
  protection.

## Journal syncs (slice J, task 69)

Order from the independent review: skip directory syncs where nothing changed (three `ensure_*` syncs), skip the two
syncs after removing a landed transaction directory, defer directory syncs to barriers, add torn-record tolerance for
pre-landing records with a tearing, dropping and reordering harness (a kill is not a power loss), and only then drop
file syncs of pre-landing records. Lock-candidate syncs stay: a torn blocking `owner.json` would stick every
acquirer. The sync-site table is in `doc/handover/cli-git-rust-implementation.md`.

## Results of the hook-entry and configuration-snapshot slices

Method: `strace --follow-forks -e trace=execve,fsync,fdatasync,syncfs,sync_file_range` over one wrapped
`git commit --quiet --message=x -- a.txt` of the debug build in a fresh repository with the host's Git 2.56.0,
counting `execve` of `/usr/bin/git` (equal to the `| start ` lines of `GIT_TRACE2_PERF`, 52 in both) and sync calls;
scratch script `measure.ts <binary> <scenario>` beside the repository's `attribute-commit.ts`.
Before is the commit that preceded these slices; after is the branch tip.

- No hooks: 52 to 43 Git starts, 42 to 38 sync calls, three wrapper re-executions as dispatcher entries to none.
- One `pre-commit` hook: 52 to 44 Git starts, 42 to 39 sync calls.
- All four hooks: 52 to 48 Git starts, 42 to 42 sync calls (the configuration reads are the only saving).
- A configuration-defined hook: 52 to 48 Git starts, 42 to 42 sync calls.
- A custom `core.hooksPath` holding one `commit-msg` hook: 52 to 44 Git starts, 42 to 39 sync calls.
- `measure-fsync.ts` (an `--allow-empty` commit, no hooks): 41 to 37 sync calls.

Permanent budget: `binary_hook_shim_tests.rs` fails a commit with no hooks that starts any hook runner or more than
one `git config`, a commit with four hooks that does not start exactly four runners, and a commit with one hook that
does not start exactly one.

Known deviations, each accepted for the saving and each recorded here because it is not byte-for-byte what separate
reads gave:

- The snapshot is read when the commit begins, so the maintenance decision no longer sees a change that a hook or
  another process makes to `maintenance.auto`, `gc.auto` or the detach keys during the commit. Example: a
  `pre-commit` hook that runs `git config maintenance.auto false` used to stop that commit's maintenance run; it no
  longer does. The hooks path and the switches were read just before preparation and are now read earlier by the
  length of the policy pass.
- An entry is omitted from a plan-time look at the hooks directory, so a `pre-commit`, `prepare-commit-msg` or
  `commit-msg` hook created between planning and the native commit (milliseconds) does not run. `post-commit` is
  looked up when it comes due and has no such gap.
- A named hook (`hook.<name>.event`) is found from the real repository's configuration, while the dispatcher's
  `git hook run` sees the shadow's configuration, which includes the real one. They differ only if a conditional
  include matches the shadow's path and not the real one, for example a `gitdir:` pattern that names the shadow
  directory; that include would define a hook the wrapper no longer starts.

Found while probing and left unchanged: the dispatcher skips a user hook of an event with `hook.<event>.enabled=false`,
while Git 2.56.0 runs a traditional hook whatever that switch says (it disables only hooks defined by name).
`a_disabled_event_with_a_hook_is_still_skipped_by_the_wrapper` pins the wrapper's existing behavior.

Refused, with the counterexample:

- Using the snapshot for the ref storage whenever it names `extensions.refStorage`: with
  `core.repositoryformatversion = 0` Git fails (`repo version is 0, but v1-only extension found`), and with the key
  only in an included file Git prints `files`; the repository-format rules are Git's, so the key's presence asks Git.
- Modelling `1k`, `-1`, `007`, `0x10` and `08`: `--type=bool` gives `true`, `true`, `true`, `true` and a fatal error;
  `--type=bool-or-int` gives `1024`, `-1`, `7`, `16` and a fatal error; the numeric grammar is `strtoimax` base 0
  with unit suffixes, so only `0` and digit strings from `1` to `9` are decided in Rust.
- Reusing the snapshot for the consumers that ignore global options when the read carried a `-c`: the original reads
  would not have seen the option, and a `-C` or `--git-dir` changes which repository is listed.

Not done: the `git config --file config --list` of the shadow stays when the configuration includes another file;
the `config --get` reads of the auto-push (`remote.<name>.push`, `commit.gpgsign`) are outside the commit path
measured here; a commit made with global options (`git -c k=v commit`) still starts one read each for the ref
storage, the maintenance keys and the shadow format, because those reads ignore the options or run elsewhere.

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`); each slice lowers its budget to the measured count. Starting point: 45 processes, 41 syncs.
