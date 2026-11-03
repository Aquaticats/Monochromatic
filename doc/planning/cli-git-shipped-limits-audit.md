# Shipped limits of the git wrapper against LQM

This is an audit, not a decision.
It lists every numeric bound the shipped Rust git wrapper (`monochromatic-git-policy-cli`, binaries `cli-git-native`
installed as `git` and `cli-git-service`) can make a person hit, and says for each whether a measurement or an owner
decision is written beside the value.
It changes no product code and decides nothing; the questions at the end are for the owner.

Audited commit: `d48d0a66f4` (main).
Rule `LQM` of `AGENTS.md` (line 1357), quoted verbatim from the owner's approval:
"Shipped resource limits (unit caps, timeouts, size bounds) cite a workload measurement or owner decision beside the
value; agent-command cgroup rules never size shipped code."

Paths in this document that start with `src/` or name `SPEC.md`, `README.md` or `DESIGN.md` are relative to the crate
directory
`package/git-policy/cli`.
Paths that start with `doc/` are relative to the repository root.

## Method and scope

### What counts as shipped

- Shipped: the modules `src/native/lib.rs` declares without `#[cfg(test)]`, `src/native/main.rs`,
  `src/native/service_main.rs`, and `src/native/service_units/*`.
- Not shipped, so not audited: every `*_tests.rs`; every module that `lib.rs` declares under `#[cfg(test)]`
  (`test_support`, `command_test_support`, `command_test_cases`, `command_test_parseopt`, `command_test_completion`,
  `policy_test_support`, `scanner_test_support`, `subvolume_test_support`, `fs_btrfs_test_support`,
  `parallel_test_support`, `worktree_ladder_test_support`, `dependent_version_test_support`,
  `dependent_version_fixture_*`, `recording_git`, `stale_ref_lock_killed_git`, `standards_review_mock`).
- The `binary_*.rs` files (for example `binary_wrapper_store.rs` with `WAIT_CAP` and `IDLE_BUILD_AGE`,
  `binary_support.rs` with `SOCKET_RELEASE_PATIENCE`, `binary_supervisor.rs` with `NO_PROGRESS_WINDOW`) are compiled
  only into the `native_binary` test target (`src/native/binary_tests.rs` declares them as its modules; `Cargo.toml`
  names that target), never into `lib.rs` or the two binaries.
  Their waits are test-harness bounds and are out of scope.
- The environment variables named `CLI_GIT_TEST_ONLY_*` (`transaction_phase.rs`, `service_fault.rs`,
  `client_journal.rs`) are test seams that live in shipped files; their polls are not listed.
- Windows and macOS are out of scope until the owner says otherwise ("Ignore Win and mac until I say so."), so the
  start-time resolutions of those platforms in `src/native/process_start.rs` are not listed.

### What counts as a limit

A limit is listed when its effect changes with the workload or the clock: a timeout, a wait, a retry count, a backoff,
a poll or notice interval, a size or entry bound, a queue, frame or batch size, a retention age, a parallelism cap or
memory estimate, an environment-variable default, or a setting the wrapper passes to a program it starts.
Excluded, with one sentence each:

- Format and protocol constants (hash lengths, index header and stat sizes, file mode bits, ioctl numbers, D-Bus
  codes,
  base64 and SHA tables, option-table ordinals, exit codes, schema versions): they are fixed by a specification, not
  chosen for a workload.
- Git's own limit values the wrapper only mirrors (`GIT_MAX_CONFIG_COUNT`, `MINIMUM_ABBREVIATION`,
  `FALLBACK_ABBREVIATION`, `BACKOFF_MAX_MULTIPLIER` and the jitter numbers in `src/native/index_lock_wait.rs`): the
  wrapper needs the same number as Git to behave as Git does.
The reader bounds that merely say they equal a Git limit are listed, because no Git source line is written beside
  them.
- The `Debug`-level and test-only values.

### Searches that produced this list

All were run over the shipped file set (446 files after the exclusions in "What counts as shipped").

- Durations and sleeps: `Duration::`, `from_secs`, `from_millis`, `from_micros`, `from_nanos`, `sleep(`, `Timeout`.
- Constants: every `const` or `static` whose type is an integer, float or `Duration`, then read by name for limit-like
  words (`LIMIT`, `MAX`, `BUDGET`, `WAIT`, `PAUSE`, `POLL`, `INTERVAL`, `ATTEMPT`, `RETRY`, `AGE`, `BATCH`, `DEPTH`,
  `GRACE`, `PATIENCE`, `CEILING`, `LEASE`, `IDLE`, `CAPACITY`, `WORKERS`, `MEMORY`, and similar).
- Literals outside constants: `GiB`, `MiB`, `<< 20`, `<< 30`, `.take(`, `.min(`, `chunks(`, `available_parallelism`.
- Environment: every `CLI_GIT_*` name and every `env::var` read, then each default and parse rule.
- Messages: `giving up`, `still waiting`, `was still running`, `made no progress`, `limit=`, `after {} seconds`.
- Systemd: `systemd-run`, `CPUQuota`, `MemoryMax`, `MemoryHigh`, `TasksMax`, `IOWeight`, `WatchdogSec`,
  `RuntimeMaxSec`, `StartLimit`, `RestartSec`, `Restart=`, `TimeoutStartSec`, `--property`.
- Positive controls: the systemd search printed the four caps of `src/native/service_units/cli-git@.service` and
  nothing else, and a search for `MemoryMax` alone hit that same file, so the zero hits in code are real.
  The duration search hit the known constants (`COMMIT_WAIT`, `DEFAULT_LIVE_WAIT`), and the message search hit the
  known "giving up after" sentences of `src/native/service_index_wait.rs`.
- Result: the wrapper starts no `systemd-run` unit and passes no cgroup property anywhere in shipped code.
  The only systemd settings it writes are in the two unit templates.
  The memory constants (`REPOSITORY_MEMORY`, `QUERY_MEMORY`, `SLICE_MEMORY`, `ENTRY_MEMORY`, `CONVERSION_MEMORY`,
`INDEX_REWRITE_COPIES`) feed only `src/native/parallel_budget.rs`, which divides the memory the kernel reports by them
  to choose a thread count; none becomes a unit property or a `ulimit`.
- Not done: no test, build or process was run.
  Message texts are read from source, not produced.

### Branches that change the picture

The audit reads main.
Three decided changes are not on main yet, and one design is still in review.

- `fix/git-policy-cli-service-no-memory-cap` (`741d42932e`, `474c987ae2`): removes the four unit caps.
  `git diff main...` shows it edits `src/native/service_units/cli-git@.service`, `SPEC.md`,
  `doc/planning/cli-git-resident-service.md` and two test files.
  On main the unit still carries the caps, and `SPEC.md:2515` still says the unit "allows two CPUs and one gibibyte".
- `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits`: "Wait while connected" for pushes (handover
  `doc/handover/cli-git-rust-implementation.md:327`).
  Update, 2026-11-03: it is in the 0.5.27 integration tip, and picks 5 and 12 below were then built on branch
  `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx`; the entries of "Pushes" carry their outcomes.
Read with `git show`: its `bounded_push.rs` keeps `DEFAULT_PUSH_DEADLINE` (30 s), `DEFAULT_PUSH_CEILING` (1800 s) and
  the
  low-speed constants at the same values and stops giving a push the low-speed abort (its lines 221 to 224).
  The progress rule changes (bytes on the push's socket count), the numbers do not.
- `fix/git-policy-cli-audit-rotation-race` (`2eb2cf1bfc`): fixes the race of the audit log rotation; the bound and the
  one-generation history are unchanged (its `discard_audit.rs:27`).
- `fix/git-policy-cli-commits-never-fail`: the design that is meant to replace the interim loops and waits; not built.

### Citation classes

- cited: a measurement (with where it was taken) or an owner decision determines or is written beside the value, and I
  give the path and line.
- partial: a measurement or an owner decision is beside the value but does not determine it (the rule is decided and
  the number is not; the typical case is measured and the bound is a judgement).
  The owner's complaint was about the number, so partial counts as uncited for LQM.
- none: only a rationale, a precedent, an estimate, or nothing.
  A sentence that says why a bound exists is not a citation of its value.
- Doc lines that merely repeat the value (`README.md`, `SPEC.md` listing "10 minutes") are not citations.

## Owner decisions that bear on limits

Quoted; none is reinterpreted.

- "You should (almost) never be waiting." (`SPEC.md:1647`)
- "never fail doesn't primarily mean retries!" (`SPEC.md:1168`)
  A failure that can happen is "a design defect to remove by construction"; "A retry is at most the last resort for a
  cause outside the system".
Outside our control (`SPEC.md:1184` to `1200`): network down, the remote refusing (auth, a server hook, GH009, branch
  protection), disk full or quota, disk I/O errors, a memory or process-count cap set by someone else, a kill or power
  loss, a corrupt repository, the person's signer failing.
  Designed out (`SPEC.md:1202` to `1206`): a slow remote, other programs' locks, Git's own crashes or bugs.
- "Auto merging should never fail.", "Manual merge should never fail too.", "Commits should never fail."
  (`SPEC.md:1162` to `1164`)
- "commits vs native git should not take 5s longer." (`README.md:1419`, `SPEC.md:3977`)
- Pushes, "Wait while connected (Recommended)" (`doc/handover/cli-git-rust-implementation.md:327`): bytes on the
  push's socket count as progress; once the pack is sent, keep waiting while the connection is open.
  "Stop only on no progress (Recommended)" (same handover, line 913).
- `doc/decision/cli-git-concurrent-commits.md:151` to `158`: an `index.lock` owner proven alive "gets an unbounded
  wait
  with one line naming the holder"; a dead or evidence-free owner gets quadratic backoff "up to
  `indexLock.unprovenOwnerTimeoutMs` (default 1000)".
  Line 347 of the same record repeats that a live owner gets an unbounded wait.
- "Same as the ref lock (Recommended)" for a live foreign `index.lock` (`SPEC.md:1234`): a commit lands and exits 0
  and
the index catches up when the lock is free; a merge saves on a side ref and finishes the checkout when the lock frees.
  Recorded as answered and not yet built (handover lines 281 to 292).
- "Ship interim now (Recommended)" (handover line 284) covers the 0.5.26 interim code: the index-lock wait and a
  bounded
  moved-head loop.
  It does not name the per-class child kill, the push limits or any other bound.
- "Stack on the new head (Recommended)" for an amend, a conclusion or a revert on a head that moved (`SPEC.md:1227`
  to `1228`).
- "Offline is retried, but auth and server hook declines stop until your next commit." (`SPEC.md:1270`)
- "60s is fine for adding worktrees. And make sure we're running everything that can be run in parallel in parallel."
  (handover lines 864 and 865)
- "never discard user data" (`SPEC.md:1645`).
- "Remove all three (Recommended)" for the CPU, task and disk caps of the commit service unit (handover lines 148 to
  160).
- "1800s for git pack-objects is laughably inappropriate." (`doc/planning/cli-git-commit-timeouts.md:4`)
- "Races with other programs ... That is not an our problem. By using this command or this program, make the user
  understand and accept the risk." (handover lines 854 to 858; adopted reading, veto open)
- `DESIGN.md:124`: "Whatever the system can resolve itself (a version mismatch, a stale socket, a crashed service, a
  slow start, a client killed mid-request) is resolved without the user".

## Summary

- Limits listed: 73; grouped entries (for example the three copies of an include-depth bound) count once.
- Counts by class are in the final section "Counts".
- Cited: the conversion storage ceiling, the conversion memory estimate, the `checkout.workers=0` setting, and the
  review
  gateway call deadline.
- Partial: the two per-class Git child bounds, the Markdown linter deadline, the push stall window, the batch size,
  the saved-copies age and size defaults, and the reader bounds that name a Git limit without a Git source line.
- Four unit caps: decided to be removed, removal unmerged.
- Findings that are not about a missing citation:
  - The 1800 second wait for a live holder conflicts with the decision record's unbounded wait.
  - The same wait is also applied to an unproven owner, where the decision record says 1000 ms of backoff.
  - `COMMIT_WAIT` has no setting, yet its message tells the person to raise it in the settings.
  - `FOREIGN_OWNER_PATIENCE` is already marked as an open value call for the owner.
  - `DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS` is a dead constant and the decision record still names it.
  - Several retry counts bound causes the owner put on the "designed out" list.

## The commit service unit

The unit templates are `src/native/service_units/cli-git@.service` and `cli-git@.socket`.
`Restart=on-failure` (line 14) is written without `RestartSec` or `StartLimitBurst`, so systemd's defaults apply
(on this host `systemctl show` reports `DefaultStartLimitIntervalUSec=10s` and `DefaultStartLimitBurst=5`, so a sixth
start within ten seconds is refused).
That is an inherited bound the unit does not state; it is not counted.
The socket template sets only the mode (`0600`) and directory mode (`0700`), which are not limits.

### Unit MemoryMax

- Where: `src/native/service_units/cli-git@.service:15`.
- Value: `MemoryMax=1G`.
- Bounds: the whole service cgroup, including the Git children it starts.
- When hit: the kernel reclaims, then kills; the owner reported 274,736 hits of the cap on the main checkout and
  `memory.peak` at 1,073,766,400 bytes (handover lines 148 to 150).
- Fails or waits: slows, then kills the service, which the manager restarts on the next connection.
- Citation: none on main. Owner decision to remove it: handover line 159 and 791 ("Remove all three (Recommended)",
  and the memory cap was found unmeasured); removal is on `fix/git-policy-cli-service-no-memory-cap`, unmerged.
- Conflict: with LQM and with "never fail" (a memory cap is "set by someone else" only when someone else sets it; this
  one was the wrapper's own).
  Decided, removal unmerged.

### Unit TasksMax

- Where: `src/native/service_units/cli-git@.service:16`.
- Value: `TasksMax=512`.
- Bounds: processes and threads of the service cgroup.
- When hit: a thread or process start fails; `src/native/service_server.rs:589` to `594` shows the acceptor backing
  off
  when it cannot start a thread, and `src/native/start_retry.rs` retries a refused process start.
- Fails or waits: waits up to the start-retry budget, then the connection or step fails.
- Citation: none on main; decided removal as recorded in "Unit MemoryMax".
- Conflict: the same as in "Unit MemoryMax"; decided, removal unmerged.

### Unit CPUQuota

- Where: `src/native/service_units/cli-git@.service:17`.
- Value: `CPUQuota=200%`.
- Bounds: CPU time of the service cgroup.
- When hit: throttling.
- Fails or waits: waits (slower commits and slower after-commit work).
- Citation: none on main; decided removal as recorded in "Unit MemoryMax".
- Conflict: "commits vs native git should not take 5s longer" could be hit by a throttled service; decided, unmerged.
- Side effect to carry into the question on parallelism: `src/native/service_round.rs:129` sizes a round by the CPUs
  the
  process may use, so the unit's two CPUs set the round width on main; after removal the host's CPUs do.

### Unit IOWeight

- Where: `src/native/service_units/cli-git@.service:18`.
- Value: `IOWeight=50`.
- Bounds: the service's share of block I/O under contention.
- When hit: slower I/O when other cgroups compete.
- Fails or waits: waits.
- Citation: none on main; decided removal as recorded in "Unit MemoryMax".
- Conflict: none beyond LQM; decided, unmerged.

## Waits for locks and for the service's index hold

### Live-holder wait, 1800 seconds

- Where: `src/native/wait_limit.rs:26` (`DEFAULT_LIVE_WAIT`), variable `CLI_GIT_LOCK_WAIT_SECONDS`
  (`wait_limit.rs:22`).
Read by `src/native/index_holder_wait.rs:152` (merge), `src/native/service_index_wait.rs:235` (forwarded index
  writers),
  and the service's landing (`src/native/service_main.rs:59`, `src/native/service_land.rs:42`).
- Value: 30 minutes; a missing, zero or malformed variable falls back to it, never to no limit
  (`wait_limit.rs:40` to `42`).
- Bounds: how long a command or the service waits for a lock held by a live program, and also for a holder whose owner
  cannot be proven alive (`src/native/index_holder_wait.rs:13` and `64`; `src/native/service_index_attempts.rs:50`).
- When hit, the person sees (`src/native/index_lock_evidence.rs:447` and `474`):
"cli-git: waiting for {holder}, which holds {lock} (proven by {proof}); giving up after {n} seconds" and, every notice
  interval, "still waiting ... after {n} seconds; giving up after {n} seconds ({LIVE_WAIT_VARIABLE} changes this),
  leaving the lock in place."
  For an unproven owner: "waiting for {lock}, whose owner cannot be proven alive, after {n} seconds; giving up after
  {n} seconds."
  A forwarded writer that waited for the service's own hold prints "the commit service was still finishing a commit in
  this repository after {n} seconds, so git {command} runs now and reports what Git says"
  (`src/native/service_index_wait.rs:153`).
  The service's landing ends with failure code `IndexInUse` (`src/native/service_index_attempts.rs:132` to `135`),
  rendered as "{holder} is using this repository's index, so this commit was not made. Wait for it to finish, or close
  it, and run the commit again." (`src/native/service_messages.rs:190`).
  Where no inotify watch can be set, `watch.is_none()` makes the wait count as exhausted at once
  (`service_index_attempts.rs:109`).
- Fails or waits: waits up to the bound, then the commit fails (landing) or the merge refuses.
- Citation: none.
`doc/planning/cli-git-commit-timeouts.md:177` to `181` leaves "the number unchanged" because a holder "can also sit in
  the user's hooks, whose runtime nobody here can bound"; the wrapper runs no external hooks
  (`SPEC.md:1843`, "Real Git starts with traditional hooks off"), so that stated reason no longer applies.
  The code comment (`wait_limit.rs:25`) says "longer than any ordinary hook run, short enough that a stuck holder is
  noticed within the hour": a rationale, not a measurement.
- Conflict, yes, with four decisions:
  - `doc/decision/cli-git-concurrent-commits.md:151` to `154` and `347`: a proven-alive owner gets an unbounded wait,
    the code stops at 1800 s.
- Same record, lines 155 to 158: an unproven or dead owner gets backoff up to 1000 ms, the code waits the full 1800 s
    for it (`index_holder_wait.rs:13`, "an unproven holder is waited for to the limit").
  - "Commits should never fail" and "Designed out ... other programs' locks": at the bound the commit fails with
    `IndexInUse`; "Same as the ref lock (Recommended)" (`SPEC.md:1234`) is the owner's chosen replacement and is not
    built.
  - "You should (almost) never be waiting."
- Record mismatch: `src/native/doctor_config.rs:139` and `src/native/config_schema.rs:215` say
`indexLock.unprovenOwnerTimeoutMs` "does nothing", while the decision record (lines 156 and 157) still prescribes it.

### Service hold age limit, 60 seconds

- Where: `src/native/service_index_wait.rs:48` (`HOLD_LIMIT`), used at line 210.
- Value: 60 seconds.
- Bounds: the age at which a service hold whose mark names no holder, or a holder this process cannot ask about, is
  taken for a dead service's leftover.
- When hit: "the commit service's hold on this repository's index is {n} seconds old, longer than a commit takes, so
  git
  {command} does not wait for it." (`service_index_wait.rs:138`).
- Fails or waits: stops waiting; the forwarded command then runs and may meet Git's own refusal.
- Citation: none.
  The comment says "Every step of a landing is bounded", but the bulk child bound is 300 seconds
  (`src/native/child_class.rs:39`), so a legitimate hold can outlive 60 seconds; the two numbers are not reconciled.
- Conflict: none found against the quoted decisions; it is a staleness judgment, not a wait.

### Look counts and pauses of the lock wait

- Where: `src/native/service_index_wait.rs:33` and `34` (`EMPTY_LOOKS` 5, `EMPTY_PAUSE` 2 ms),
  `src/native/service_index_hold.rs:27` and `28` (`SETTLE_LOOKS` 5, `SETTLE_PAUSE` 10 ms),
  `src/native/index_holder_wait.rs:41` (`POLL_PAUSE` 50 ms, only where no watch can be set).
- Value: as listed.
- Bounds: how long a lock that reads empty or a failing question is re-asked before it is judged foreign or given up.
- When hit: the wait classifies the lock as a foreign program's; no message of its own.
- Fails or waits: waits a few milliseconds.
- Citation: none (the comment at `service_index_wait.rs:21` to `26` explains the case, not the numbers).
- Conflict: none found.

### Service scan budget, 250 milliseconds

- Where: `src/native/index_lock_holders.rs:293` (`SERVICE_SCAN_BUDGET`).
- Value: 250 ms.
- Bounds: the scan of open descriptors that looks for the process holding a lock.
- When hit: the scan returns what it found; the holder may then read as "cannot be proven alive".
- Fails or waits: waits at most the budget; changes which message is printed.
- Citation: none.
- Conflict: none found.

### Index reconciliation tries, 5

- Where: `src/native/service_land_follow.rs:24` (`INDEX_ATTEMPTS` 5) and `27` (`INDEX_PAUSE` 100 ms), loop at line 147
  in `update_index`.
The only caller is `src/native/service_prepare.rs:448`, the ending of a commit whose corrections removed every change.
- Value: five tries, 100 ms apart.
- Bounds: how long the client retries resetting the real index entries of the corrected files to the base while
  another
  program holds the index.
- When hit: failure code `IndexInUse` (`service_land_follow.rs:59` to `60`, from Git's "index.lock" text), and the
  command
  stops with "the index could not be reconciled with the corrected files: {word}" (`service_prepare.rs:449`).
  That commit exits 1 in any case, because it has no change left to record.
- Fails or waits: waits half a second, then the command ends with that message and the index is not reconciled.
- Citation: none.
- Conflict: partial. The cause is another program's lock, which the owner put on the designed-out list, and this is a
  retry; the command was already ending without a commit, so no commit fails here.
- Not this path: the service's landing builds the new index in a file of its own (`build_index`,
  `service_land_follow.rs:176`) and installs it by releasing the lock it holds.
`follow_up` returns a flag rather than an error (`src/native/service_land.rs:147`), and its caller says "The branch
  has
  moved: the commits exist whatever happens to the index, which recovery finishes" (`service_land.rs:408` to `410`).
  So a landing does not fail on a foreign index lock after the ref moved, and no `INDEX_ATTEMPTS` bound applies to it.

### Rerun of a forwarded index writer, 50 starts

- Where: `src/native/service_hold_retry.rs:28` (`ATTEMPTS`).
- Value: 50.
- Bounds: how many times `git add`, `rm` or `mv` is started again when the commit service takes the index first.
- When hit: the last refusal of Git ("Unable to create ... index.lock") is shown.
- Fails or waits: fails after the last start.
- Citation: none.
- Conflict, yes: the contender is the wrapper's own service, a cause the owner would count as designed out ("never
  fail
  doesn't primarily mean retries!").

### Foreign-owner patience, one hour

- Where: `src/native/pid_namespace.rs:19` (`FOREIGN_OWNER_PATIENCE`).
- Value: 3600 seconds.
- Bounds: how long an owner record from another PID namespace counts as alive, measured from its write.
  `SPEC.md:739` to `740` uses it for work-item generations; `doc/planning/cli-git-spawn-and-sync-reduction.md:619`
  reuses it for ownerless directories.
- When hit: a fresh ownerless directory "stops every guarded command for up to an hour, with the message ... naming
  the
  manual way out" (`doc/planning/cli-git-spawn-and-sync-reduction.md:713` to `716`).
- Fails or waits: waits (commands refuse until the hour passes or the person clears the directory).
- Citation: none. That document marks it "Open value call for the owner" at line 713; no answer is recorded.
- Conflict, yes: "You should (almost) never be waiting." and "commits vs native git should not take 5s longer."

### Notice intervals and graces

- Where: `src/native/wait_progress.rs:33` (`DEFAULT_NOTICE_INTERVAL` 10 s) and `38` (`NOTICE_GRACE` 1 s);
  `src/native/service_index_lock.rs:49` (`NOTICE_EVERY` 10 s, parsed at line 65 from the same variable);
  `src/native/service_timing.rs:171` (`SLOW_LOCK_STEP` 1 s); `src/native/markdown_linter_guard.rs:152`
  (`NOTICE_POLL` 50 ms); `src/native/service_client.rs:31` (`START_NOTICE_GRACE` 1 s).
- Value: as listed; `CLI_GIT_WAIT_NOTICE_SECONDS` replaces the interval, zero or malformed values are ignored
  (`wait_progress.rs:93`).
- Bounds: how often a waiting command repeats its line.
  The notice line is "cli-git: still waiting for {what} (process {pid}) after {n} seconds; stopping it after {n}
  seconds{note}." (`wait_progress.rs:231`).
- When hit: a line is printed; nothing fails.
- Fails or waits: neither; display pacing.
- Citation: none (`README.md:908` and `1325` describe the variable; `doc/planning/cli-git-commit-timeouts.md:164` to
  `165` states the grace rule).
- Conflict: none found.

### Retired unproven-owner timeout

- Where: `src/native/config_schema.rs:41` (`DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS`); read in
  `src/native/config_parse.rs:167`.
- Value: 1000 ms.
- Bounds: nothing; the comment says "nothing reads it now".
- When hit: never.
- Fails or waits: neither.
- Citation: the value is the decision record's default (`doc/decision/cli-git-concurrent-commits.md:157`), but the
  behavior it stood for is gone.
- Conflict: the decision record still prescribes behavior the code does not have (see "Live-holder wait, 1800
  seconds").

## Git child steps of a commit

### Quick query step bound, 60 seconds

- Where: `src/native/child_class.rs:34` (`PLUMBING_CHILD_WAIT`); override `CLI_GIT_CHILD_WAIT_SECONDS`
  (`child_class.rs:30`, parse at `117`: a missing, zero or malformed value falls back to the class default).
- Value: 60 seconds for `rev-parse`, `rev-list`, `symbolic-ref`, `for-each-ref`, `ls-tree`, `cat-file`, `diff-tree`,
  `merge-base`, `write-tree`, `update-ref`, `config`, `remote`, `version` (`child_class.rs:54` to `66`).
- Bounds: one Git step a commit starts (`src/native/child_watch.rs`, `src/native/transaction_git.rs:525`).
- When hit (`child_watch.rs:390`): "{what} did not finish within {n} seconds and was stopped
  (CLI_GIT_CHILD_WAIT_SECONDS changes this). If the step belonged to a commit, the next Git command of this wrapper
  finishes or undoes that commit, so run the command again. If the step was only slow, raise the bound with
  CLI_GIT_CHILD_WAIT_SECONDS=<seconds>; if it stalls again, check with `ps` what that process waits for."
- Fails or waits: waits, then the step is killed and the commit fails.
- Citation: partial.
  `child_class.rs:21` to `22` says "a hundred times the slowest such step measured on a host at several times its core
  count in load".
`doc/planning/cli-git-commit-timeouts.md:151` to `154` measured these steps on a 20,207-file clone of this repository
  at
  a load average of 80 to 145 on 28 cores (lines 9 to 18) and found the worst at 44 ms.
  A hundred times 44 ms is 4.4 s, not 60 s, so the stated derivation does not produce the value.
- Conflict, yes: "Commits should never fail" and Git hangs are "designed out"; the 60 s kill turns a stall into a
  failed
  commit.
The one interim the owner approved ("Ship interim now") names the index-lock wait and the moved-head loop, not this
  kill.
  `doc/planning/cli-git-commit-timeouts.md:4` is the owner objecting to the older 1800 s bound; it does not choose 60.

### Bulk step bound, 300 seconds

- Where: `src/native/child_class.rs:39` (`OBJECT_CHILD_WAIT`), same override.
- Value: 300 seconds for every step not on the quick list (`pack-objects`, `index-pack`, `add`, `reset`, `read-tree`,
  `update-index`, `ls-files`, `diff`, `diff-index`, `show`, `ls-remote`, unknown subcommands, and the object reader).
- Bounds: as in "Quick query step bound, 60 seconds".
- When hit: the same sentence as in that entry, with "a bulk step" as the label.
- Fails or waits: waits, then killed, commit fails.
- Citation: partial.
  `doc/planning/cli-git-commit-timeouts.md:139` to `148` measured `reset` at 2.9 to 5.8 s, `update-index --refresh` at
2.8 s and `ls-files --modified` at 2.8 s on a stat-dirty 20,207-file index, warm cache, load above 80; the comment
  says
  the bound "leaves room"; no figure derives 300.
- Conflict: same as the quick bound.

### First-notice graces of the two classes

- Where: `src/native/child_class.rs:42` (`PLUMBING_NOTICE_GRACE` 5 s) and `45` (`OBJECT_NOTICE_GRACE` 10 s);
  `src/native/child_watch.rs:101` caps the grace at the interval and at half the bound.
- Value: 5 s and 10 s.
- Bounds: when the first "still waiting" line appears.
- When hit: the "still waiting" line quoted in "Notice intervals and graces".
- Fails or waits: neither.
- Citation: none (`doc/planning/cli-git-commit-timeouts.md:151` and `156` state them).
- Conflict: none found.

### Refused-start retry, five seconds

- Where: `src/native/start_retry.rs:27` (`START_RETRY_BUDGET`), pauses `30` (10 ms first) and `33` (500 ms longest).
- Value: pauses double from 10 ms to 500 ms until 5 s have been spent.
- Bounds: a process start refused with `EAGAIN` (no processes left).
- When hit (`start_retry.rs:70` to `73`): the start error plus "; the start was refused {n} times over {n} ms".
- Fails or waits: waits up to 5 s, then the step fails.
- Citation: none.
- Conflict: none; a process-count cap is on the owner's "outside our control" list, so a retry is the allowed last
  resort.

### Signal guard installation wait

- Where: `src/native/commit_preparation.rs:148` (`GUARD_WAIT_LIMIT` 1200) and `151` (`GUARD_WAIT_DELAY` 50 ms).
- Value: 1200 tries of 50 ms, 60 seconds.
- Bounds: contention between threads of one process while the guard is installed.
- When hit: an error from the preparation.
- Fails or waits: waits then fails.
- Citation: none.
  `doc/planning/cli-git-commit-timeouts.md:182` to `185` says production never waits there ("only library tests do").
- Conflict: none found.

## Moved-branch, landing and push-contention loops

### Landing rebuilds, 3

- Where: `src/native/service_land.rs:31` (`LANDING_ATTEMPTS`), loop `220` to `402`.
- Value: 3.
- Bounds: how many times the service builds a batch again when the branch moved under it.
- When hit: `BranchKeepsChanging`, rendered at `src/native/service_messages.rs:159`: "The branch {branch} changed
  {times} times while this commit was being prepared, so it was not made. Run the commit again; if other people commit
  to the same branch at the same moment, it will succeed once they pause."
- Fails or waits: fails after the third round.
- Citation: none.
- Conflict, yes: the cause (other commits moving the branch) is not on the "outside" list; the owner chose "Stack on
  the
  new head (Recommended)" for amends, conclusions and reverts; and "Commits should never fail".
  Covered in part by "Ship interim now" (a bounded moved-head loop), but not the number.

### Client rebuilds, 5

- Where: `src/native/service_commit.rs:60` (`REBUILD_ATTEMPTS`), uses `204`, `461`, `481`, `510`.
- Value: 5 ("A build is one signature, so the bound keeps a busy branch from keeping a person's command running").
- Bounds: how many times Git builds the commit again on a new tip.
- When hit: the same `BranchKeepsChanging` message, with 5, or `BranchMovedMessageSaved` when a typed message was
  saved
  (`service_messages.rs:169`).
- Fails or waits: fails.
- Citation: none (a rationale for the bound, not for 5).
- Conflict, yes: as in "Landing rebuilds, 3"; also the signer is on the "outside" list, which is the only part of this
  loop that the
  owner's
  rules allow to cost a retry.

### Merge rounds, 20

- Where: `src/native/merge_isolation.rs:136` (`MAX_ROUNDS`), loop `387`, `README.md:922` and `940`.
- Value: 20.
- Bounds: whole merges recomputed on the newest head after a branch move or a taken index lock.
- When hit (`merge_isolation.rs:399`): "cli-git: git merge: the branch moved or the index lock was taken in 20 merges
  in a
  row, so the wrapper stopped; nothing was changed. Run the merge again."
- Fails or waits: fails with exit 2.
- Citation: none (the comment says "a loop, not a race").
- Conflict, yes: "Manual merge should never fail too." and other programs' locks are "designed out".

### Reconcile attempts, 10

- Where: `src/native/auto_push_reconcile.rs:41` (`MAX_RECONCILE_ATTEMPTS`), loop `548`, `SPEC.md:1461`.
- Value: 10.
- Bounds: fetch, merge and push cycles after the remote refuses a push as stale.
- When hit (`auto_push_reconcile.rs:690`): "{remote}/{branch} advanced again during each of 10 attempts, so nothing
  was
  pushed and the worktree was not touched." with the prefix "cli-git: auto-push: not published: ".
- Fails or waits: the push fails; the service plans a new try with growing pauses.
- Citation: none (`SPEC.md:1461` gives the reason, "sibling worktrees that keep pushing are contention").
- Conflict: partial. The cause is the remote advancing or siblings pushing, which is not on the "outside" list, and
  "Auto merging should never fail."

### Reference-lock push retries, 3

- Where: `src/native/auto_push.rs:46` (`PUSH_LOCK_RETRIES`) and `49` (`PUSH_LOCK_PAUSE` 250 ms times the try number),
  loop `242` to `246`, `SPEC.md:1577`.
- Value: three more pushes after 250, 500 and 750 ms.
- Bounds: a push that delivered but could not update the tracking ref because another service held its lock.
- When hit: the push's own output, including "cannot lock ref", and the note
  "cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready." only when exit is not 0.
- Fails or waits: waits under two seconds; the tracking ref may stay stale.
- Citation: none.
- Conflict, yes: the lock is another program's (a sibling worktree's service), a cause "designed out" by the owner,
  and
  this is a retry.

### Claim take rounds, 64

- Where: `src/native/single_owner_claim.rs:44` (`MOST_ROUNDS`).
- Value: 64.
- Bounds: rounds of one `take_claim` while other claimants keep arriving.
- When hit: the claim reports that claimants arrive faster than it can look.
- Fails or waits: fails to take.
- Citation: none.
- Conflict: none found.

### Work item tries, 3

- Where: `src/native/service_work.rs:46` (`MAX_TRIES`), used at `378` and `420`; message at `187`.
- Value: 3.
- Bounds: how many times an item whose holder stopped renewing its claim is run again before it is given up.
- When hit: "the commit service stopped 3 times before it finished pushing {branch} (a restart, a kill or a crash each
  time), so it g[ave up]" (`service_work.rs:187`).
- Fails or waits: the item is failed and a notice is left.
- Citation: none.
- Conflict: none; the cause is a kill or crash, which is on the "outside" list.

## Commit service client and server

### Service start wait, 10 seconds

- Where: `src/native/service_client.rs:28` (`START_WAIT`), `README.md:959`.
- Value: 10 seconds, used for the connect loop (`service_client.rs:142`), the welcome read (`120`), the retire loop
  (`189`) and `ask_to_retire` (`334`).
- Bounds: how long the client waits for the operating system to start the service.
- When hit (`service_messages.rs:126`): "The commit service did not start, so this commit was not made. Its log shows
  why: `{LOG_COMMAND}`. Run the commit again after fixing that."; for a service of another release,
  "... did not stop when asked ... Restart it with `{RESTART_COMMAND}`" (`147`).
- Fails or waits: waits, then the commit fails.
- Citation: none (`doc/planning/cli-git-resident-service.md:849` says "ten seconds by default"; a design statement).
- Conflict, yes: `DESIGN.md:124`, "a slow start ... is resolved without the user".

### Commit queue wait, 10 minutes

- Where: `src/native/service_client.rs:34` (`COMMIT_WAIT`), `README.md:960`; fixed in
`src/native/service_commit.rs:181`, `src/native/auto_push_trigger.rs:327`, `src/native/worktree_trash_dispose.rs:152`.
- Value: 600 seconds as a socket read timeout (`service_client.rs:123`, check at `266`).
- Bounds: how long a client waits for the answer to its request.
- When hit (`service_messages.rs:182`): "This commit waited {minutes} minutes behind other commits and was not made.
  Run
  it again later, or raise the waiting limit in the settings for the wrapper."
- Fails or waits: waits, then fails.
- Citation: none. `doc/planning/cli-git-resident-service.md:280` says "default 10 minutes, configurable".
  No setting exists: the only call sites pass the constant, and the environment and configuration readers name none.
  The message therefore tells the person to change something that cannot be changed.
- Conflict, yes: "Commits should never fail", "You should (almost) never be waiting."
  The handover plans "heartbeats instead of `COMMIT_WAIT`" (line 190) on the unbuilt design.

### Reconnects, 5

- Where: `src/native/service_client.rs:37` (`RECONNECTS`), pause `40` (`CONNECT_PAUSE` 25 ms), uses `199`, `313`,
  `317`.
- Value: five resends of the same request id, pausing 25 ms times the try number.
- Bounds: sends after the connection broke before an answer.
- When hit (`service_messages.rs:138`): "The commit service stopped before it confirmed this commit. Run `{check}` to
  see
  whether it was made, and `{check}` to see what is still staged. If it was not made, run the commit again."
- Fails or waits: waits under a second, then fails with an uncertain outcome.
- Citation: none.
- Conflict: none; the cause is a service kill or crash, on the "outside" list.

### Request read timeout and accept backoff

- Where: `src/native/service_server.rs:44` (`REQUEST_READ_TIMEOUT` 10 s, set at 265) and `255` (`ACCEPT_BACKOFF` 100
  ms).
- Value: as listed.
- Bounds: how long a new connection may take to send its request line; how long the acceptor pauses when it cannot
  start a thread.
- When hit: the connection is dropped silently, or the client gets `Unavailable` ("The commit service could not finish
  this commit, so it was not made. Run the commit again ...", `service_messages.rs:220`).
- Fails or waits: fails that connection.
- Citation: none.
- Conflict: none found.

### Idle exit, three hours

- Where: `src/native/service_server.rs:48` (`IDLE_EXIT`), argument `--idle-seconds` (`service_main.rs:61` to `63`),
  `README.md:1989`.
- Value: 3 hours without work.
- Bounds: how long the service stays resident.
- When hit: the service exits; the socket stays and the next connection starts it again.
- Fails or waits: the next commit after an idle exit pays a service start (up to the start wait).
- Citation: none. The comment cites Bazel's `--max_idle_secs` default as "this value's precedent"
  (`service_server.rs:47`), and `doc/planning/cli-git-resident-service.md:862` repeats it; a precedent is not a
  measurement or an owner decision.
- Conflict: none found; the cost of the first commit after idle is not measured against "should not take 5s longer".

### Batch size, 64

- Where: `src/native/service_server.rs:41` (`MAX_BATCH`), loop `618`.
- Value: 64 requests landed as one batch.
- Bounds: group commit size.
- When hit: the rest wait for the next batch.
- Fails or waits: waits one landing.
- Citation: partial. `doc/planning/cli-git-resident-service.md:274` to `276` measured, in a prototype, batches that
"grow from 1 under light load to 21 to 64 in a burst, the section stays 49 to 68 ms"; the largest observed batch
  equals
  the cap, so the measurement does not show where the cap should be.
- Conflict: none found.

### Frame and change bounds

- Where: `src/native/service_protocol.rs:27` (`MAX_LINE_BYTES` 1 MiB; checks at `397` and `572`, read bound at
`service_server.rs:268`) and `30` (`MAX_CHANGES` 4096; `service_protocol.rs:445`,
  `src/native/service_prepare.rs:161`).
- Value: a line over 1 MiB is refused; a commit that changes more than 4096 paths is not handed to the service.
- Bounds: request size.
- When hit: for more than 4096 changes `service_prepare.rs:161` returns `None` and the commit takes the native path
  instead of the service (the code path returns "no changes to send"); a longer line is a decode failure.
- Fails or waits: falls back or refuses; I did not read the long-line failure text.
- Citation: none.
- Conflict: the fallback is the opposite of "Invoke real git as rarely as possible" (handover owner quote), but only
  for
  commits above the bound.

### Claim lease and renewal

- Where: `src/native/service_work.rs:335` (`CLAIM_LEASE` 120 s), `src/native/service_claim_renewal.rs:34`
  (`RENEWALS_PER_LEASE` 3) and `39` (`LOOKS` 3), `src/native/service_retry.rs:18` (`RETRY_SLACK` 1 s),
  `src/native/worktree_removal_rescue.rs:53` (`LOOK_ROUNDS` 2).
- Value: a claim not renewed for 120 s is taken back; the holder renews every 40 s.
- Bounds: how long work claimed by a killed service waits before another instance runs it.
- When hit: no message; the item runs again, or after `MAX_TRIES` is failed with the notice quoted in "Work item
  tries, 3".
- Fails or waits: waits up to the lease after a kill.
- Citation: none (`SPEC.md:3346` onward explains why a lease exists, not why 120 s).
- Conflict: none; a kill is "outside our control".

### Journal retention and capture memory

- Where: `src/native/service_journal.rs:258` (`DONE_RETAINED` 1024), `src/native/service_captures.rs:25` (`REMEMBERED`
  128).
- Value: 1024 finished batches; 128 landed captures per worktree.
- Bounds: how far back a resent request is recognized as already landed; how many landed commits are remembered.
- When hit: an older resend or capture is unknown (`REMEMBERED`: "older ones are forgotten, which makes them
  unknown").
- Fails or waits: neither; may re-land.
- Citation: none. The 1024 comment derives from `RECONNECTS` and `START_WAIT`, which are themselves uncited.
- Conflict: none found.

### Stale temporary and scratch ages

- Where: `src/native/service_journal.rs:33` (`TEMPORARY_GRACE` 10 min), `src/native/work_item_recovery.rs:29`
  (`STRAY_TEMPORARY_AGE` 24 h), `src/native/scratch_location.rs:46` (`SCRATCH_STALE_AFTER` 24 h).
- Value: as listed.
- Bounds: the age before a recovery or preparation deletes a leftover temporary or scratch file.
- When hit: the file is deleted.
- Fails or waits: neither.
- Citation: none (`scratch_location.rs:44` to `45` says "far longer than any command of the wrapper runs (its longest
  bound is minutes)", but the live-lock wait is 30 minutes and the foreign-owner patience an hour).
- Conflict: none found, since scratch is not user data.

### Earlier-service retirement wait

- Where: `src/native/service_provision.rs:86` (`RETIREMENT_WAIT` 10 s) and `367` (`RETIREMENT_POLL` 50 ms).
- Value: 10 seconds.
- Bounds: how long the earlier single service may take to stop.
- When hit (`service_provision.rs:339`): "the earlier commit service ({unit}) is still stopping after {n} seconds; run
  the
  commit again in a moment, or stop it with `systemctl --user stop ...`".
- Fails or waits: waits, then the commit fails.
- Citation: none.
- Conflict, yes: `DESIGN.md:124` (a version mismatch resolved without the user).

### Service manager exchange

- Where: `src/native/systemd_dbus.rs:32` (`EXCHANGE_WAIT` 20 s) and `42` (`MAX_REPLY_BYTES` 1 MiB).
- Value: 20 seconds for the whole connect, authenticate, hello and calls; 1 MiB per reply.
- Bounds: one exchange with the user service manager.
- When hit: a timeout error rendered as a setup failure ("The commit service could not be set up ...",
  `service_messages.rs` `CouldNotStart`).
- Fails or waits: waits, then the commit fails.
- Citation: none.
- Conflict, yes: `DESIGN.md:124` for a manager that is only slow.

## Pushes

### Push stall window, 30 seconds

- Where: `src/native/bounded_push.rs:49` (`DEFAULT_PUSH_DEADLINE`), variable `CLI_GIT_PUSH_DEADLINE_SECONDS` (`45`),
  `SPEC.md:1511`.
- Value: 30 seconds with no sign of work (bytes moved, CPU used, a task running or in disk wait), sampled every 500 ms
  (`src/native/push_progress.rs:44`).
  Where `/proc` cannot be read it is a fixed deadline for the whole push.
- Bounds: the auto-push and the reconcile fetch (`src/native/lfs_bounded_push.rs:92`,
  `src/native/auto_push_reconcile.rs:155`).
- When hit (`bounded_push.rs:555`): "cli-git: git push made no progress for {n} seconds and was stopped (limit=stall,
  bytes_moved={n}, elapsed_seconds={n}); {extend clause}. Hint: if you were pushing over a slow or offline connection,
  your commit is already saved; push again when the connection works." Exit code 124.
- Fails or waits: waits while progress shows, then the push is stopped; the service plans a new try
  (`src/native/push_retry.rs`).
- Citation: partial.
`SPEC.md:1492` to `1510` measured on 2026-11-03 which signs show a push at work (a 30 MB push to a local bare remote;
  `rchar` and `wchar`, processor ticks, task state), and the owner chose "Stop only on no progress (Recommended)".
  Neither decides 30 seconds.
- Conflict, yes, on main: "Wait while connected (Recommended)" (bytes on the socket count; keep waiting while
  connected).
  `SPEC.md:1505` to `1509` itself says a server that checks the push before answering "is stopped" after the window.
  A slow remote is on the owner's "designed out" list.
  The unmerged branch changes the progress rule and keeps 30.
- Outcome (pick 5, the coordinator's under the owner's delegation "Pick options as if I had picked them and invite veto
  when I say I'm back", open to the owner's veto):
  the pack-sent test is gone, and an open connection (a process of the tree holds a socket in state `ESTABLISHED`,
  read from the kernel's socket diagnostics) is waited for in every phase.
  The window now applies only to a tree with no open connection and no sign of work.
  It covers the push, the reconcile fetch and the LFS upload, and the wait line names the program that waits.
  The 30 seconds stays uncited, and `bounded_push.rs` and `SPEC.md` say so beside it.
  The journal of the commit service (read-only, 2026-10-20 to 2026-11-03, 794 pushes that landed) gives whole-push times
  of p50 3.5 s, p99 26.4 s and maximum 72.4 s, which include connected time.
  No silence of a disconnected push is logged, so nothing determines the number.
  Open question: "The stall window of a push with no connection, and a connection that holds and never answers",
  in "Questions raised while building picks 5 and 12".

### Push ceiling, 1800 seconds

- Where: `src/native/bounded_push.rs:55` (`DEFAULT_PUSH_CEILING`), variable `CLI_GIT_PUSH_CEILING_SECONDS` (`52`),
  `SPEC.md:1515`.
- Value: 30 minutes in all, however much the push moves.
- Bounds: the same pushes.
- When hit (`bounded_push.rs:564`): "cli-git: git push was still running after {n} seconds, though it kept moving
  data, and
  was stopped (limit=ceiling, ...); ... Hint: your commit is already saved; push again when the connection is faster."
- Fails or waits: waits, then stops; the service retries.
- Citation: none (`SPEC.md:1515` to `1517`: "so a trickle that never stalls cannot hold the service for good").
- Conflict, yes: "Wait while connected ... once the pack is sent, keep waiting while the connection is open."
- Outcome (pick 5, delegated, open to the owner's veto): removed.
  `DEFAULT_PUSH_CEILING`, `CLI_GIT_PUSH_CEILING_SECONDS`, its advice, `StopCause::Ceiling` and `limit=ceiling` are gone.
  Cost, recorded in `SPEC.md`: a process that spins or sits in state `D` for good shows a sign of work for ever
  and holds the service's push of its repository.

### Git low-speed abort, 1000 bytes per second for 60 seconds

- Where: `src/native/bounded_push.rs:69` and `72` (`LOW_SPEED_LIMIT`, `LOW_SPEED_TIME`), applied at
  `bounded_push.rs:216` to `222` from `src/native/lfs_bounded_push.rs:91` (the push) and
  `src/native/auto_push_reconcile.rs:154` (the fetch); `SPEC.md:1565`.
- Value: `GIT_HTTP_LOW_SPEED_LIMIT=1000`, `GIT_HTTP_LOW_SPEED_TIME=60`, set only when the invocation sets neither.
- Bounds: Git's own HTTP transfer, via curl.
- When hit: curl's "Operation too slow", as the handover records (line 342).
- Fails or waits: the transfer fails.
- Citation: none.
- Conflict, yes on main: "Wait while connected".
  The handover says the push no longer sets it (branch `fb1e093809`); main still does.
- Outcome (pick 5, delegated, open to the owner's veto): removed for every run the wrapper starts.
  The push stopped setting it in `fb1e093809`.
  The reconcile fetch stopped here (`apply_low_speed_defaults`, `LOW_SPEED_LIMIT` and `LOW_SPEED_TIME` are gone),
  shown by `the_reconcile_fetch_sets_no_low_speed_abort`.
  A value the person sets in the environment or in `http.lowSpeedLimit` and `http.lowSpeedTime` still reaches Git.

### Push output cap, 4 MiB

Status after picks 5 and 12: unchanged. `bounded_push.rs` and `SPEC.md` now say beside the value that no citation exists.

- Where: `src/native/bounded_push.rs:62` (`MAX_PUSH_OUTPUT`), `is_flood` at `277`, `SPEC.md:1562`.
- Value: 4 MiB; only the first 4 MiB are kept.
- Bounds: the captured output of a push.
- When hit (`bounded_push.rs:572`): "cli-git: git push wrote more than {n} bytes of output and was stopped. Hint:
  ...".
- Fails or waits: the push is stopped.
- Citation: none.
- Conflict: none found.

### Stop grace and polls

- Where: `src/native/bounded_push.rs:58` (`STOP_GRACE` 5 s), `75` (`POLL_PAUSE` 10 ms);
  `src/native/child_watch.rs:123` (`STOP_POLL` 10 ms); `src/native/package_repair_command.rs:81` (10 ms).
- Value: a catchable stop, five seconds, then the kill; polls every 10 ms.
- Bounds: how long a stopped child gets to clean up its own lock files.
- When hit: the kill, then the lock sweep (`SPEC.md:1526` to `1560`).
- Fails or waits: waits up to five seconds.
- Citation: none (`SPEC.md:1526` states it).
- Conflict: none found.

### Read-only query ceiling, 120 seconds

- Where: `src/native/bounded_query.rs:35` (`QUERY_CEILING`), deadline `65` (the shorter of the push deadline and the
  ceiling),
  `SPEC.md:1574`.
- Value: 120 seconds, or the push stall window when that is shorter (so 30 s by default).
- Bounds: `config`, `for-each-ref`, `rev-parse`, `remote get-url`, `merge-base` before a push.
- When hit (`src/native/auto_push_single_flight.rs:74`): "git {args} did not finish within {n} seconds and was
  stopped; {clause}".
- Fails or waits: waits then the push attempt fails; the service retries.
- Citation: none ("queries read local repository state and finish in milliseconds", no figure).
- Conflict: yes in part, by the same reasoning as the child bounds.

### Push retry pauses, 10 seconds to 10 minutes

- Where: `src/native/push_retry.rs:35` (`FIRST_PAUSE`) and `38` (`LONGEST_PAUSE`), variable
  `CLI_GIT_PUSH_RETRY_SECONDS` (`28`), `SPEC.md:1268`, `README.md:1275`.
- Value: the pause doubles from 10 s to 600 s and stays there; there is no count limit.
- Bounds: the service's own new tries of a push that was stopped or failed for a reason that can pass.
- When hit (`src/native/service_work.rs:256`): "cli-git: the commit service tries this push again by itself in {n}
  seconds (try {n}), and keeps trying with longer pauses until the push lands or a conflict needs you."
- Fails or waits: waits; never gives up while the cause can pass.
- Citation: none for the values.
The owner decision "Offline is retried, but auth and server hook declines stop until your next commit." decides which
  causes retry, not the pauses.
- Conflict: partial. The retried causes include a held lock and a stall ("Being offline, a stall, the ceiling and a
  held
  lock are tried again", `SPEC.md:1272`); a held lock is another program's and a slow remote is designed out.

### LFS server-error retry, 2 and 5 seconds

- Where: `src/native/lfs_server_retry.rs:23` (`RETRY_DELAYS`), `30` (`MAX_CONFIGURED_RETRIES` 5), `33`
  (`MAX_CONFIGURED_DELAY` 60 s); variable `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS`; `SPEC.md:1974`.
- Value: two retries, after 2 s and after 5 s; a configured list is clipped to five delays of at most 60 s.
- Bounds: `git-lfs push` after an HTTP 5xx answer.
- When hit (`lfs_server_retry.rs:167`): "cli-git: `git-lfs push` failed with a server error, so it is run again in {n}
  seconds (retry {n} of {total}); objects the server already has are not sent twice."
- Fails or waits: waits seven seconds in all, then the push fails.
- Citation: none.
- Conflict: unclear. An HTTP 5xx is a remote answering, which the owner's two lists do not name; "the remote refusing"
  lists auth, a server hook, GH009 and branch protection, and "a slow remote" is designed out.
  The classification is a question for the owner.
- Outcome (pick 12, the coordinator's under the owner's delegation, open to the owner's veto):
  an HTTP 5xx from the LFS server is classed as outside our control, a server fault like "Network down",
  and is retried by the service's push retry plan alone.
  `lfs_server_retry.rs`, `RETRY_DELAYS`, `MAX_CONFIGURED_RETRIES`, `MAX_CONFIGURED_DELAY`,
  the variable `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS` and both loops of `lfs_upload.rs` are removed.
  The plan reads nothing in a 5xx as final
  (`an_lfs_server_error_is_tried_again_by_the_plan`, shown red by classing a 5xx as final),
  the auto-push is left to the plan (`an_auto_push_that_meets_a_server_error_is_left_to_the_services_retry_plan`),
  and a typed push ends at the first 5xx (`a_server_error_ends_a_typed_push_at_once`).
  Visible change for a typed `git push`: no "retry 1 of 2" lines, and a brief outage now fails the push
  where two in-place retries used to ride it out.

### LFS chunking, relay and pointer scan

- Where: `src/native/lfs_upload_plan.rs:57` (`OBJECTS_PER_RUN` 500), `src/native/lfs_upload.rs:123`
(`MAX_RELAYED_BYTES` 4 MiB), `src/native/lfs_pointer_scan.rs:28` (`MAX_COMMITS` 100,000) and `31` (`SCAN_BUDGET` 3 s),
  `src/native/lfs_pointer.rs:18` (`BLOB_SIZE_CUTOFF` 1024).
- Value: as listed.
- Bounds: object names per `git-lfs push` run; relayed standard error; the in-process scan for pointers.
- When hit: more runs; text dropped; the scan "gives the question to `git-lfs`".
- Fails or waits: slows, never fails.
- Citation: none (`BLOB_SIZE_CUTOFF` mirrors the LFS pointer-size rule, with no source line beside it).
- Conflict: none found.

### Auto-push walk budget, 4096 commits

- Where: `src/native/auto_push_trigger.rs:42` (`WALK_BUDGET`), `SPEC.md:1295`.
- Value: 4096 commits per ancestry question.
- Bounds: the check that decides whether a merge or similar command hands a push to the service.
- When hit: "a walk that cannot tell hands nothing over" (`SPEC.md:1295`); no push is handed over by that command.
- Fails or waits: neither; the push is not made until a later trigger.
- Citation: none.
- Conflict: possible with "The owner's rule is that auto-push must land" (`src/native/push_retry.rs:5`): a branch far
  ahead is not handed over at all.

### Concurrent per-update queries, 4

- Where: `src/native/manual_push_probe.rs:133` (`MOST_CONCURRENT_QUERIES`), `SPEC.md:1619` and `1624`.
- Value: four Git queries at once for a manual push.
- Bounds: parallelism, and the connections opened to remotes.
- When hit: queries queue.
- Fails or waits: waits.
- Citation: none; the comment says "the bound `SPEC.md` gives", and `SPEC.md` gives it without a measurement or an
  owner quote.
- Conflict, yes in part: "make sure we're running everything that can be run in parallel in parallel."

## Policies, reviews and repair tools

### Markdown linter deadline, 60 seconds

- Where: `src/native/markdown_linter.rs:64` (`LINT_DEADLINE`), guard `src/native/markdown_linter_guard.rs`.
- Value: 60 seconds per candidate.
- Bounds: one run of the in-process Markdown linter.
- When hit (`markdown_linter.rs:184`): "monochromatic-lint did not finish within 60000 milliseconds and cli-git
  stopped
  waiting for it (its thread stays in this process until the command ends); run the command again, run
  monochromatic-lint by hand on the file to see why it stalls, or set the policy markdown/autofix to off to skip the
  check", after a notice "still waiting for {file} after {n} seconds; giving up on it after {n} seconds"
  (`markdown_linter_guard.rs:104`).
- Fails or waits: waits then the policy check fails.
- Citation: partial. `markdown_linter.rs:53` to `58`: "Measured on the host with the former child process, one run
  takes about 11 milliseconds warm and 227 cold ... the host has run at load averages of 40 to 90.
  A minute leaves room."
  The typical case is measured; 60 s is a judgement three orders above it.
- Conflict: yes in part, "Commits should never fail" (a hung linter in this process cannot be killed, so the bound is
  the
  only exit).

### Markdown output bounds

- Where: `src/native/markdown_linter_output.rs:32` (`EXPLANATION_LIMIT` 400), `37` (`MAX_FIXED_BYTES` 64 MiB),
  `42` (`MAX_FINDINGS_BYTES` 4 MiB).
- Value: as listed; the two byte bounds are "the bound the former child process had" on its output streams.
- When hit (`markdown_linter_output.rs:228`, `235`): "monochromatic-lint produced a fixed source of {n} bytes, over
  the
  67108864 bytes cli-git accepts" or the findings equivalent.
- Fails or waits: the check fails.
- Citation: none (inherited, not measured).
- Conflict: none found.

### Policy convergence passes, 8

- Where: `src/native/policy_convergence.rs:39` (`MAXIMUM_CHANGED_PASSES`), `SPEC.md:2097` and `2108`.
- Value: eight changed passes, then one more pass.
- Bounds: the loop that re-runs policies after a correction changed content.
- When hit: a pass-limit failure ("settles when that pass proposes nothing and is a pass-limit failure when it still
  proposes a change").
- Fails or waits: fails.
- Citation: none ("The incumbent stops after eight changed passes").
- Conflict: none found; a policy that keeps changing its own output is the wrapper's own defect.

### Standards review gateway call, 10 seconds

- Where: `src/native/service_review.rs:28` (`CALL_DEADLINE`).
- Value: 10 seconds.
- Bounds: the service's one call to the review gateway.
- When hit: the item is failed and the work thread moves on; the failure text goes to the review log.
- Fails or waits: the review fails; the commit is unaffected.
- Citation: cited, with a caveat.
  `src/native/service_review.rs:26` to `27` bases it on "several times the largest figure on record (1.4 s in
`doc/audit/standards-review-gate-shadow.md`)"; that file shows a wall-clock cost of 1.4 s at line 2562 for one logged
  review, and I did not check that it is the largest.
- Conflict: none found; the review does not block a commit.

### Standards review script and whole-review deadlines

- Where: `src/native/standards_review_client.rs:18` (`SCRIPT_DEADLINE` 30 s) and `src/native/service_review.rs:31`
  (`BUDGET` 20 s).
- Value: 30 seconds for the script's request; 20 seconds for the whole review, Git reads and write included.
- Bounds: the review that runs after a commit.
- When hit: the item is failed; the log says "The operation was aborted due to timeout"
  (`standards_review_client.rs:107`).
- Fails or waits: the review fails; the commit is unaffected.
- Citation: none (30 s is inherited from the old script, 20 s from "a third of the script's 30 s" argument at
  `service_review.rs:26` to `27`).
- Conflict: none found.

### Standards review input caps

- Where: `src/native/standards_review_input.rs:54` (`CAP_LINES` 2000), `57` (`MAX_EXCERPT_FILES` 8), `60`
  (`EXCERPT_CONTEXT` 30), `63` (`EXCERPT_LINE_CAP` 4000), `69` (`INPUT_UNIT_CAP` 24000);
  `src/native/standards_review_source.rs:22` (`MAX_PATCH_BYTES` 64 MiB, "the script's `maxBuffer`").
- Value: as listed.
- Bounds: what part of a commit the model sees.
- When hit: the input is truncated and the log says "capped" (`standards_review_entry.rs:97`).
- Fails or waits: neither.
- Citation: none (carried from the script).
- Conflict: none found.

### Package repair tool limit, 600 seconds

- Where: `src/native/package_repair.rs:36` (`TOOL_LIMIT`), parse `package_repair_command.rs:176` to `205`.
- Value: ten minutes per tool (install, rebuild).
- Bounds: one package-manager tool the add runs to repair a new worktree.
- When hit (`package_repair_report.rs:92`): the finding "it ran past its time limit and was stopped", with the command
  to
  run instead.
- Fails or waits: the repair is reported failed; the worktree add itself succeeds.
- Citation: none ("a hung tool must not hold an add forever").
- Conflict: possible with "Install online during the add" (`SPEC.md:1105`), where an install can download for longer;
  and "60s is fine for adding worktrees", which sets no target to stop at (handover line 501).

### Configuration file size

- Where: `src/native/config_file.rs:83` (`MAX_CONFIG_BYTES` 1 MiB), `SPEC.md:147`.
- Value: 1 MiB.
- Bounds: `cli-git.config.jsonc`.
- When hit: a configuration failure naming the file (exit 2 before any policy runs).
- Fails or waits: the command fails.
- Citation: none ("A repository file must not control how much memory every Git command reads").
- Conflict: none found.

## Worktree add, copy and conversion

### Conversion storage ceiling, 16 workers

- Where: `src/native/worktree_convert_command.rs:76` (`CONVERSION_STORAGE_CEILING`), `README.md:535`, `SPEC.md:694`.
- Value: at most 16 conversions at once.
- Bounds: parallel conversions of nested worktrees to subvolumes.
- When hit: extra conversions wait.
- Fails or waits: waits.
- Citation: cited.
The comment records a measurement on 2026-11-03 "on a btrfs volume (32 worktrees of about ten thousand files each, 28
threads, other sessions running, medians of three or more runs at 1, 2, 4, 8, 16 and 18 workers: 61, 35, 19, 14, 12
  and 12
  seconds ...)" (`worktree_convert_command.rs:76` to `82`).
- Conflict: none; the cap sits at the measured plateau.

### Conversion memory estimate, 128 MiB

- Where: `src/native/worktree_convert_command.rs:62` (`CONVERSION_MEMORY`).
- Value: 128 MiB per conversion in the budget.
- Bounds: how many conversions the free memory admits.
- When hit: fewer workers.
- Fails or waits: waits.
- Citation: cited.
  "measured on 2026-11-03 at 7484 KiB resident for a worktree of 16 thousand files (about 0.45 KiB per file), which is
  about 90 MiB at two hundred thousand files" (`worktree_convert_command.rs:66` to `68`), rounded up.
  The host and volume are not named beside it, only the worktree size.
- Conflict: none.

### Parallel checkout setting

- Where: `src/native/checkout_workers.rs:24` and `27` (`checkout.workers=0`), `SPEC.md:651`.
- Value: `-c checkout.workers=0` (one worker per logical CPU) added only where nothing sets the key.
- Bounds: Git's checkout parallelism in a new worktree.
- When hit: not a cap; it removes Git's default of one worker.
- Fails or waits: speeds the add.
- Citation: cited.
Owner decision "Pass only if unset" (`SPEC.md:651`, `src/native/checkout_workers.rs:4` to `5`) and a measurement
  beside it:
  "a checkout of twenty thousand files measured several times faster".
- Conflict: none.

### Copy and clone thread caps and thresholds

- Where: `src/native/worktree_copy_stage.rs:221` (`PARALLEL_COPY_MIN_ENTRIES` 4096) and `232`
  (`PARALLEL_COPY_MAX_WORKERS` 8), `src/native/worktree_copy_install.rs:188` (`PARALLEL_INSTALL_MIN_LEAVES` 1024),
  `src/native/worktree_copy_journal.rs:1018` (`REMOVAL_SPLIT_DEPTH` 3),
  `src/native/package_repair_tree_clone.rs:39` (`PARALLEL_MIN_ENTRIES` 512);
  `doc/planning/cli-git-main-worktree-ignored-copy.md:268` and `275`, `SPEC.md:929`.
- Value: eight threads at most; fewer than 4096 entries stay on one thread; removal splits three levels deep on up to
  eight threads.
- Bounds: parallelism of the ignored-state copy, install and stage removal.
- When hit: more work per thread.
- Fails or waits: waits.
- Citation: none ("Cloning is a metadata operation bound by the filesystem's locks more than by CPU, and other
  sessions
  share this host"; no figure).
- Conflict, yes in part: "make sure we're running everything that can be run in parallel in parallel", on a 28-thread
  host;
  and the owner's normal-here rule on load, which says a loaded host does not excuse a cap.

### Install batch, 4096 entries

- Where: `src/native/worktree_copy_install.rs:38` (`INSTALL_BATCH_ENTRIES`),
  `doc/planning/cli-git-main-worktree-ignored-copy.md:271`.
- Value: 4096.
- Bounds: entries per durable intent record, so "the work an interrupted owner can leave unrecorded".
- When hit: another batch and another journal append.
- Fails or waits: waits.
- Citation: none.
- Conflict: "60s is fine for adding worktrees" bounds the add's total, not the batch; none found.

### Free space preflight, 5 GiB

- Where: `src/native/subvolume_preflight.rs:26` (`MIN_FREE_BYTES`), check `162`, `README.md:493`.
- Value: 5 GiB free on the parent directory.
- Bounds: whether the migration to a subvolume may start.
- When hit (`subvolume_preflight.rs:164`): "{parent} has {free} bytes free, fewer than the {n} bytes the migration
  requires
  for its metadata".
- Fails or waits: the command refuses and changes nothing.
- Citation: none.
  The comment says "the script's bound, 5 GiB; about 1 KiB per entry is an assumption, not a measurement".
- Conflict: none found; disk full is on the "outside" list, but this refuses before the disk is full.

### Changing-file copy attempts, 3

- Where: `src/native/subvolume_sync.rs:43` (`COPY_ATTEMPTS`), check `522`.
- Value: 3.
- Bounds: copies of a file that keeps changing under the pass.
- When hit (`subvolume_sync.rs:526`): "it changed while it was being copied, 3 times in a row; stop the programs
  writing to
  it and run the command again".
- Fails or waits: the pass fails.
- Citation: none.
- Conflict: none that is clear; the cause is another program writing, which the owner's adopted reading ("races caused
  by
  other processes changing files ... are outside what it defends against", veto open) places outside the wrapper.

### Parallel budget divisors and memory estimates

- Where: `src/native/parallel_budget.rs:17` (`CPU_MARGIN_DIVISOR` 8) and `21` (`MEMORY_SHARE_DIVISOR` 2) and `171`
  (`SLICE_MEMORY` 16 MiB); `src/native/service_round.rs:26` (`REPOSITORY_MEMORY` 64 MiB);
  `src/native/manual_push_probe.rs:137` (`QUERY_MEMORY` 32 MiB); `src/native/worktree_trash_entries.rs:24`
  (`ENTRY_MEMORY` 64 MiB); `src/native/subvolume_index_files.rs:465` (`INDEX_REWRITE_COPIES` 4).
- Value: leave one CPU in eight free (at least one when there are more than two); use half of the available memory;
  and the per-item peaks as listed, all "estimated".
- Bounds: how many repositories the service drains side by side, how many queries, trash entries and index rewrites
  run at
  once (`worker_budget`).
- When hit: fewer workers.
- Fails or waits: waits; never fails.
- Citation: none; each comment says "estimated".
  `REPOSITORY_MEMORY` derives from "the items of a burst, the output of a push, and a review's patch and answer".
- Conflict, yes in part: "make sure we're running everything that can be run in parallel in parallel."
- Interaction with the unit caps: `SPEC.md:2515` computes "two repositories at once" from the unit's two CPUs and one
  gibibyte.
Once the caps are removed, the round width is set by the host's CPUs less one in eight and by half the available
  memory
  divided by 64 MiB, so these estimates are the only memory number left.

## Saved copies, audit and log retention

### Saved copies of conflicted files, 30 days and 1 GiB

- Where: `src/native/discarded_prune.rs:35` (`DEFAULT_MAX_AGE_DAYS` 30), `38` (`DEFAULT_MAX_BYTES` 1 GiB), `41`
  (`YOUNG_FLOOR_MILLISECONDS` 24 h); settings `discardedCopies.maxAgeDays` and `maxSize` (`SPEC.md:239` to `250`);
  `README.md:756`, `SPEC.md:3880`.
  `src/native/discarded_store.rs:50` (`MAX_DEPTH` 128) keeps and reports a deeper tree.
- Value: saves older than 30 days go; then, while the folder is over 1 GiB, the oldest first; nothing younger than 24
  hours.
- Bounds: the folder `cli-git/discarded/` of copies the wrapper saved before a conflict-side command replaced a file.
- When hit: the oldest save is deleted and one audit line is written per removed save (`README.md:765`).
- Fails or waits: neither; it removes copies.
- Citation: partial.
  The rule is the owner's: "Prune by both age and size." (`doc/handover/cli-git-logging.md:269`).
  The same section says "The coordinator's defaults" (line 272); neither 30 days nor 1 GiB is an owner value or a
  measurement.
  The README sentence that gave this audit its "1 GiB" example (`README.md:756`) describes this folder.
- Conflict: this is the one place the wrapper deletes content it called "the only remaining content of replaced files"
  (`doc/handover/cli-git-logging.md:85`) against "never discard user data" (`SPEC.md:1645`).
  The owner chose the age-and-size rule, so the rule stands; the numbers are open.

### Audit log rotation, 1 MiB and one generation

- Where: `src/native/discard_audit.rs:27` and `28` (`ROTATE_AT_BYTES`), check `278`, move aside `348` to `372`.
On `fix/git-policy-cli-audit-rotation-race` (`2eb2cf1bfc`) the same bound is at its lines 27 and 28 and the rotation
  is at
  its lines 430 to 475.
  `README.md:786`, `SPEC.md:3912`, `doc/handover/cli-git-logging.md:65`.
- Value: at 1 MiB the live file is renamed to `.1`, replacing the previous `.1`; so at most two files exist and every
  rotation after the second drops the oldest history.
  The one shared appender (`append_line`) serves five records: `cli-git-audit.jsonl` (discard decisions),
`cli-git-commit-timing.jsonl`, `cli-git-worktree-add-timing.jsonl`, `cli-git-push-trigger.jsonl`, and the prune lines
  the
  saved-copies pass writes into the audit file (`src/native/commit_timing.rs:403`, `add_timing.rs:278`,
  `push_trigger_log.rs:161`, `discarded_prune.rs:512`).
- Bounds: how much audit and timing history stays on disk.
- When hit: nothing is printed; the oldest generation is overwritten.
  The race fix's text says "Nothing is deleted, so no line is lost by the rotation" (branch `discard_audit.rs`, line
  436), which is true of the rotation step and false of the next rotation, which replaces `.1`.
- Fails or waits: neither; a failed write never changes a command's result (`README.md:786`).
- Citation: none. `doc/handover/cli-git-logging.md:65` records the behavior ("moves to `.1` at one mebibyte, replacing
  the
  older copy"), not a decision or a measurement.
The audit file exists so that a loss of uncommitted work "is attributable" (`src/native/discard_audit.rs:5` to `7`),
  and
each audit line names the saved copies by path (`doc/handover/cli-git-logging.md:79` and `84`), so a dropped
  generation can leave
  a saved copy with no line that names it.
  By contrast the standards-review shadow log "has no rotation" (`README.md:2010`) and is unbounded.
- Conflict: possible with "never discard user data" (`SPEC.md:1645`); see "The audit and timing logs' history" in the
  questions.

### Clean-check content and attribute reads

- Where: `src/native/worktree_clean_content.rs:37` (`CONTENT_LIMIT` 32 MiB) and `40` (`ATTRIBUTES_LIMIT` 4 MiB);
  `src/native/worktree_attributes.rs:28` (`MATCH_WORK_LIMIT` 4,194,304 cells);
  `src/native/conflict_side_guard.rs:52` (`INDEX_LIMIT_BYTES` 1 GiB).
- Value: as listed.
- Bounds: the content comparison that decides whether a path is clean.
- When hit (`worktree_clean_content.rs:419`): "{path} is larger than 33554432 bytes, which is more than is read to
  compare it
by content" and the path is not judged clean; an over-large index makes the guard decline
  (`conflict_side_guard.rs:122`).
- Fails or waits: refuses the clean decision, so the guarded command is refused or goes native.
- Citation: none.
- Conflict: none found.

## Locks left by a Git that was killed

### Lock age margin, two seconds

- Where: `src/native/stale_ref_lock_judge.rs:24` (`MARGIN`), `SPEC.md:1705`.
- Value: a lock younger than two seconds is left alone and not mentioned.
- Bounds: when the wrapper judges whether a reference lock is left behind.
- When hit: nothing is printed for a young lock.
- Fails or waits: neither; a lock that is a leftover and younger than the margin stays until a later pass.
- Citation: none.
  `SPEC.md:1830` to `1842` measured that Git fills a reference lock after creating it empty, but records no duration.
- Conflict: none found.

### Boot proof slack, one hour

- Where: `src/native/stale_ref_lock_judge.rs:37` (`BOOT_SLACK`), `41` (`REUSE_SLACK_MS` 100), `SPEC.md:1769`.
- Value: a lock must predate the boot by more than 3600 seconds for the boot proof.
- Bounds: the proof that a lock belongs to an earlier boot.
- When hit: the lock is not removed by that proof.
- Fails or waits: neither.
- Citation: none (reasoned from clock steps).
- Conflict: none found.

### Empty-lock settle time, one second

- Where: `src/native/stopped_git_locks.rs:59` (`EMPTY_LOCK_SETTLE`) and `54` (`COARSE_CLOCK_SLACK` 50 ms),
  `SPEC.md:1544`.
- Value: an empty lock without a pid file must stay empty for one second after a stopped child's tree has ended before
  the wrapper removes it.
- Bounds: the sweep after a push or step was stopped.
- When hit: the sweep waits out what remains of the second.
- Fails or waits: waits up to one second.
- Citation: none (`SPEC.md:1542` to `1545` gives the reason; no figure).
- Conflict: none found.

## In-process readers, parsers and chunking

These bounds make a reader decline and leave the question to real Git, or split a command line.
None changes a command's result; a hit costs a Git start.
That cost is against "Invoke real git as rarely as possible" only when the bound is reached.

### File size bounds of internal readers

- Where: `src/native/ref_files.rs:27` and `31` (512, 256 bytes); `src/native/repo_read_refs.rs:19` (4096) and `22`
  (256 MiB);
  `src/native/repo_read_odb.rs:25` (1 MiB) and `28` (256 MiB); `src/native/sequencer_marker.rs:26` (64 MiB);
  `src/native/repo_read_config_text.rs:15` (1 MiB); `src/native/repo_read_layout.rs:61` (4096);
  `src/native/service_index_marker.rs:28` (512); `src/native/single_owner_claim.rs:207` (4096);
  `src/native/worktree_convert.rs:35` (64 KiB); `src/native/worktree_convert_journal.rs:35` (1 MiB);
  `src/native/worktree_trash_dispose.rs:42` (64 KiB); `src/native/real_git_candidate.rs:49` (64 KiB);
`src/native/doctor.rs:268` (1024); `src/native/git_system_config.rs:31` (4096); `src/native/git_version_cache.rs:48`
  (1024);
  `src/native/repo_read_object.rs:17` (256 MiB) and `102` (512 KiB inflate input);
`src/native/candidate_batch.rs:63` (128-byte reply header); `src/native/systemd_dbus.rs:42` is listed with the manager
  exchange.
- Value: as listed.
- Bounds: the largest file a reader looks at.
- When hit: "anything bigger is left to Git" (`repo_read_object.rs:16`), "a bigger file is left to Git"
  (`sequencer_marker.rs:25`), or the check answers "not a match".
- Fails or waits: falls back, never fails.
- Citation: none (rationales such as "a real one is a short line"; none measured on a real repository).
- Conflict: none found.

### Depth and loop bounds of readers

- Where: `src/native/repo_read_config_typed.rs:27`, `src/native/repo_read_config_scan.rs:20`,
  `src/native/repo_read_config_list.rs:22` (`INCLUDE_DEPTH_LIMIT` 10); `src/native/repo_read_odb.rs:22`
(`ALTERNATE_DEPTH_LIMIT` 5); `src/native/repo_read_refs.rs:25` (5 symbolic hops);
  `src/native/repo_read_query_refs.rs:29`
and `src/native/lfs_object_plan.rs:64` (peel 8, 5); `src/native/repo_read_pack.rs:23` (`DELTA_DEPTH_LIMIT` 100) and
  `27`
  (`CACHE_BYTE_LIMIT` 32 MiB); `src/native/repo_read_tree.rs:23` (256), `26` (4,000,000 entries), `168` (3 lists);
  `src/native/repo_read_*` `ENV_ENTRY_LIMIT` 1024 and `DIGIT_LIMIT` 9 (`repo_read_config_typed.rs:30`, `33`;
`repo_read_config_list.rs:25`; `repo_read_layout.rs:64`; `config_snapshot.rs:30`);
  `src/native/forwarded_command.rs:39`
  (`MAX_ALIAS_EXPANSIONS` 16); `src/native/service_pruned.rs:66` (`CLOSURE_ENTRY_BUDGET` 5,000,000);
  `src/native/discarded_store.rs:50` is listed with saved copies.
- Value: as listed.
- Bounds: loops and recursion.
- When hit: the reader declines and Git answers.
  Exceeding sixteen alias expansions is the one that changes behavior: "Alias expansions followed before giving up".
- Fails or waits: falls back.
- Citation: partial for the ones the comment calls "Git's own limit" (`INCLUDE_DEPTH_LIMIT` 10 and the alternate depth
  5):
the owner's practice is to cite the Git 2.56.0 file and line (handover, "Recorded here only"), and none is beside
  them.
  The rest: none.
- Conflict: none found.

### Argument chunking and display caps

- Where: `src/native/candidate_store.rs:107` (`COMMIT_CHUNK_SIZE` 2048), `src/native/candidate_prediction.rs:87`
  (`PATHSPEC_CHUNK_SIZE` 2048), `src/native/candidate_attribute.rs:43` (`ATTRIBUTE_CHUNK_SIZE` 2048),
  `src/native/commit_staged_additions.rs:26` (`ENTRIES_PER_START` 200); display and record caps:
  `src/native/auto_push_dirty_paths.rs:80` (5 paths), `src/native/worktree_copy_transaction.rs:305` (3 paths),
  `src/native/discarded_prune.rs:51` (100 names), `src/native/subvolume_sync.rs:118` (20 samples),
  `src/native/index_lock_evidence.rs:37` (64 characters), `src/native/push_failure_notice.rs:44` and `47` (100, 4000),
  `src/native/package_repair_command.rs:78` (600 bytes), `src/native/standards_review_request.rs:30` (200),
  `src/native/standards_review_entry.rs:18` (12), `src/native/journald_native.rs:47`, `50`, `117`, `484`
  (16 KiB message, 1024 field, 64 name, 4096 pipe chunk); `src/native/worktree_copy_compare.rs:33` (64 KiB read).
- Value: as listed.
- Bounds: how many paths one Git start takes, how many items a message names, and how many characters a log field
  keeps.
- When hit: more Git starts, or "and {n} more", or text cut.
  For the journal, an entry too large for the socket "goes to standard error" (`journald_native.rs:47`).
- Fails or waits: neither.
- Citation: none. The chunk sizes say they are "far below any argument limit".
  The 64-character journald field-name bound is journald's own rule, with no source line.
- Conflict: none found.

## Counts

The limit sections hold 73 limits (a grouped entry, such as the three include-depth copies, counts once).

- cited: 4.
  The conversion storage ceiling (`src/native/worktree_convert_command.rs:76`), the conversion memory estimate
(`worktree_convert_command.rs:62`), the `checkout.workers=0` setting (`src/native/checkout_workers.rs:3` to `5`), and
  the review gateway call deadline (`src/native/service_review.rs:26`, with the caveat in its entry).
- partial: 7.
The quick and bulk child bounds, the push stall window, the batch size, the Markdown linter deadline, the saved-copies
  age and size defaults, and the depth and loop bounds of the readers.
- decided to be removed, removal unmerged: 4 (the four unit caps).
- none: 58, one of which (`DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS`) is a dead constant.

So 11 of 73 have a measurement or an owner decision beside them in some form, and 4 of those 11 determine the value.

### Uncited limits

Each line is a limit with no citation (class none) or only a partial one.

- Live-holder wait 1800 s, `wait_limit.rs:26`.
- Service hold age limit 60 s, `service_index_wait.rs:48`.
- Lock-wait look counts and pauses (5 x 2 ms, 5 x 10 ms, 50 ms), `service_index_wait.rs:33`,
  `service_index_hold.rs:27`,
  `index_holder_wait.rs:41`.
- Service scan budget 250 ms, `index_lock_holders.rs:293`.
- Index reconciliation tries 5 x 100 ms, `service_land_follow.rs:24`.
- Forwarded index writer reruns 50, `service_hold_retry.rs:28`.
- Foreign-owner patience 3600 s, `pid_namespace.rs:19` (marked an open value call for the owner).
- Notice intervals and graces 10 s, 1 s, 50 ms, `wait_progress.rs:33`.
- Retired unproven-owner timeout 1000 ms (dead), `config_schema.rs:41`.
- Quick child bound 60 s and bulk child bound 300 s (partial), `child_class.rs:34`, `child_class.rs:39`.
- Class notice graces 5 s and 10 s, `child_class.rs:42`.
- Refused-start retry 5 s, `start_retry.rs:27`.
- Signal guard wait 1200 x 50 ms, `commit_preparation.rs:148`.
- Landing rebuilds 3, `service_land.rs:31`; client rebuilds 5, `service_commit.rs:60`; merge rounds 20,
  `merge_isolation.rs:136`; reconcile attempts 10, `auto_push_reconcile.rs:41`; reference-lock push retries 3,
  `auto_push.rs:46`; claim rounds 64, `single_owner_claim.rs:44`; work item tries 3, `service_work.rs:46`.
- Service start wait 10 s, `service_client.rs:28`; commit queue wait 600 s, `service_client.rs:34`; reconnects 5,
  `service_client.rs:37`; request read timeout 10 s, `service_server.rs:44`; idle exit 3 h, `service_server.rs:48`;
  batch size 64 (partial), `service_server.rs:41`; frame 1 MiB and 4096 changes, `service_protocol.rs:27`.
- Claim lease 120 s, `service_work.rs:335`; journal retention 1024 and 128, `service_journal.rs:258`,
  `service_captures.rs:25`; temporary and scratch ages 10 min and 24 h, `service_journal.rs:33`,
  `work_item_recovery.rs:29`, `scratch_location.rs:46`.
- Retirement wait 10 s, `service_provision.rs:86`; manager exchange 20 s, `systemd_dbus.rs:32`.
- Push stall window 30 s (partial, now only for a tree with no open connection), `bounded_push.rs`; push output 4 MiB,
  `bounded_push.rs`; stop grace 5 s (partial), `bounded_push.rs`; query ceiling 120 s, `bounded_query.rs:35`;
  push retry pauses 10 s to 600 s, `push_retry.rs:35`.
  Removed by pick 5: the push ceiling 1800 s and the low-speed abort.
- LFS chunking, relay and scan, `lfs_upload_plan.rs:57`,
  `lfs_pointer_scan.rs:28`; auto-push walk budget 4096, `auto_push_trigger.rs:42`; concurrent queries 4,
  `manual_push_probe.rs:133`.
- Markdown linter deadline 60 s (partial), `markdown_linter.rs:64`; Markdown output bounds,
  `markdown_linter_output.rs:37`;
  convergence passes 8, `policy_convergence.rs:39`; review script and whole-review deadlines, `service_review.rs:31`;
  review input caps, `standards_review_input.rs:54`; package repair tool limit 600 s, `package_repair.rs:36`;
  configuration file 1 MiB, `config_file.rs:83`.
- Copy thread cap 8 and thresholds, `worktree_copy_stage.rs:232`; install batch 4096, `worktree_copy_install.rs:38`;
free space 5 GiB, `subvolume_preflight.rs:26`; changing-file copy attempts 3, `subvolume_sync.rs:43`; parallel budget
  divisors and memory estimates, `parallel_budget.rs:17`.
- Saved copies 30 days and 1 GiB (partial), `discarded_prune.rs:35`; audit rotation 1 MiB, one generation,
  `discard_audit.rs:28`; clean-check reads, `worktree_clean_content.rs:37`.
- Lock margins 2 s, 1 h, 1 s, `stale_ref_lock_judge.rs:24`, `stale_ref_lock_judge.rs:37`, `stopped_git_locks.rs:59`.
- Reader file sizes, depth and loop bounds (partial), chunking and display caps: `ref_files.rs:27`, `repo_read_*`,
  `candidate_store.rs:107`.

## Questions for the owner

Each question groups limits that share a cause.
I do not decide any of them; the ranking is my reading of the options against the decisions quoted in "Owner decisions
that bear on limits".
Options that are complements (for example an interim and its replacement) can be chosen together, in order.

### Which limits need a citation under LQM

Covers: the pacing and display constants (polls of 10 ms to 100 ms, notice intervals, argument chunk sizes, truncation
caps, journald field caps), the reader bounds that make the wrapper fall back to Git, and the format and protocol
constants already left out of this list.
`LQM` says "unit caps, timeouts, size bounds" and does not say whether a 50 ms poll or a 2048-path chunk is one.

- A. Every listed limit cites a measurement or an owner decision.
  Visible result: roughly 58 entries gain a sentence; polls and truncation caps get a measured or decided value, which
  costs measurements of constants that change nothing a person sees.
- B. Only limits that can change a command's result or its duration by more than a stated slop need a citation; polls,
  display caps, chunk sizes and reader fallbacks are classed once, in one place, as exempt with the reason.
  Visible result: about half the entries need evidence; the exempt ones carry a one-line classification.
- C. `LQM` stays as worded and whoever writes a constant decides case by case whether it is a limit.
  Visible result: no extra work now, and the same gap that let the unit caps through remains for borderline constants.

Ranking: B > A > C, because B spends measurement where the unit incident happened (a bound that changed what a person
saw) and A spends the same effort on constants whose value no person can observe; and A > C because C is the
arrangement
under which an unmeasured cap was written and merged.

### What a live or unproven index lock holder costs

Covers: `DEFAULT_LIVE_WAIT` 1800 s (`wait_limit.rs:26`) for the merge, the forwarded writers and the service's
landing; the
same 1800 s for an owner that cannot be proven alive; `service_hold_retry::ATTEMPTS` 50.
The 5 x 100 ms index reconciliation of a no-change commit is a minor sibling and follows whichever option is chosen.
The decision record says an owner proven alive gets "an unbounded wait with one line naming the holder" and an
unproven
one gets backoff up to 1000 ms; the code does neither, and the doctor says the 1000 ms key "does nothing".

- A. Follow the record: a proven-alive holder is waited for without a bound, with the line naming it; an unproven or
  dead
  owner gets 1000 ms of backoff and then a diagnostic that lists the evidence.
  Visible result: a merge or commit behind a live program waits as long as the program holds the lock, repeating the
holder's line every notice interval, and never prints "giving up after 1800 seconds"; behind a stale lock the person
  gets
  the diagnostic after a second instead of thirty minutes of silence.
  The record then needs no amendment.
- B. Build the owner's answer "Same as the ref lock (Recommended)": a commit lands and exits 0 and the index catches
  up
  when the lock is free; a merge saves its result on a side ref and finishes the checkout later.
Visible result: no wait line and no `IndexInUse` failure for a live foreign lock; the command exits 0 with a line
  saying
  the index will catch up.
  A is the interim and B the replacement, so "A, then B" is reachable.
- C. Keep a bounded wait and cite its number (the owner names it, or a measurement of how long live holders hold the
  lock
  on this host).
  Visible result: unchanged messages, plus a stated reason for the number; the commit still fails at the bound.
  The decision record is amended to say so.

Ranking: B > A > C, because B removes the failure the owner calls a design defect and the wait with it, while A
removes
the failure but still waits ("You should (almost) never be waiting."); and A > C because C leaves a commit failing at
a
number for a cause (another program's lock) that the owner put on the designed-out list.

### Retry counts for moved branches and contended locks

Covers: `LANDING_ATTEMPTS` 3, `REBUILD_ATTEMPTS` 5, merge `MAX_ROUNDS` 20, `MAX_RECONCILE_ATTEMPTS` 10,
`PUSH_LOCK_RETRIES` 3.
Their causes (other commits moving the branch, a sibling worktree's service holding a ref lock, the remote advancing)
are
not on the owner's "outside our control" list.
"Ship interim now" covers the moved-head loop in general, not these numbers.

- A. Keep them as interim loops until the by-construction design is built, and write a measured number beside each
  (for example the most rounds seen when K committers hit one branch).
Visible result: the same failure texts, now with a recorded basis; "The branch ... changed 3 times" still appears
  under
  enough contention.
- B. Remove the counts: repeat on the new head until it stops moving, naming each round and the cause.
Visible result: no "changed N times" failure and no "merges in a row" stop; under a branch that never rests the
  command
  keeps running and prints a line per round.
- C. Extend the owner's "Stack on the new head (Recommended)", which names an amend, a conclusion and a revert, to
  every
  moved branch: the commit or merge is built on the head as it is when the ref moves, so there is no round to count.
  Visible result: no retry, no count, and no failure for a moved branch.

Ranking: C > B > A, because C designs the failure out, as "never fail doesn't primarily mean retries!" asks, and B
removes
the failure but keeps a loop whose end depends on the other writers; and B > A because A still fails a commit or merge
at a
number the owner has not seen.

### Time limits on Git child steps of a commit

Covers: `PLUMBING_CHILD_WAIT` 60 s and `OBJECT_CHILD_WAIT` 300 s (`child_class.rs:34`, `39`), the override
`CLI_GIT_CHILD_WAIT_SECONDS`, the signal guard wait, the refused-start retry of 5 s.
The commit-timeouts record measured the steps on one 20,207-file clone at load 80 to 145 (worst quick step 44 ms;
stat-dirty
bulk steps 2.8 to 5.8 s), which does not produce 60 or 300.
A killed step fails the commit, and "Commits should never fail" and "Git's own crashes or bugs" are designed out.

- A. Notice only: no kill; the step is waited for with the "still waiting" lines until it ends or the person
  interrupts.
  Visible result: no "did not finish within N seconds" failure; a truly hung Git holds the commit and the service lane
  until someone acts.
- B. Kill only on no progress, with the same signs the push uses (bytes moved, processor time, a task running or in
  disk wait),
  and no wall-clock bound.
Visible result: a hung child is stopped after a stall window and the commit fails with a message naming the stall; a
  slow
  but working step is never cut.
  A child wedged in the kernel counts as alive under those signs, so it is waited for.
- C. Keep 60 and 300 and write beside them a measurement taken under load (the largest step times on a stat-dirty
  large
  repository on a loaded host, and the factor used).
  Visible result: unchanged messages, with a basis; a step slower than the bound still fails the commit.

Ranking: B > C > A, because B ends real hangs without cutting slow work and uses a mechanism the push already has; C >
A
because C keeps an end for a hang, which A removes and thereby lets one stuck child hold every later commit.

### Push stall window, ceiling and Git's low-speed abort

Covers: `DEFAULT_PUSH_DEADLINE` 30 s, `DEFAULT_PUSH_CEILING` 1800 s, `LOW_SPEED_LIMIT` 1000 and `LOW_SPEED_TIME` 60
(still set for
the push on main), `MAX_PUSH_OUTPUT` 4 MiB, `STOP_GRACE` 5 s, `QUERY_CEILING` 120 s.
The owner chose "Wait while connected (Recommended)"; the branch that builds it keeps 30 s and 1800 s.
A slow remote is on the designed-out list.

- A. Adopt the decision whole: a push that has an open connection is never stopped for time; the stall window applies
  only to a
  push with no connection and no other sign of work; no ceiling; no low-speed abort for the push.
Visible result: no "limit=stall" or "limit=ceiling" for a slow server, one line naming the wait, and the push ends
  when the
  server answers or the connection drops.
- B. Merge the branch as it is and cite the two numbers with a measurement (a push to a deliberately slow remote that
  keeps the socket open).
Visible result: a slow server is waited for as long as bytes move on the socket and stopped at thirty seconds of
  silence
  or thirty minutes in all, with a cited basis.
- C. Keep main as it is.
Visible result: a hook or server check longer than thirty seconds still prints "made no progress for 30 seconds", and
  curl's "Operation too slow" can still end a push.

Ranking: A > B > C, because A is the owner's decision with no leftover number to cite; B follows the decision for
progress
but leaves two numbers that end a connected push; C contradicts the decision on main.

Delegated pick, 2026-11-03: A, built; see the outcomes under "Pushes".
The coordinator picked it under the owner's delegation, and the owner may veto it.
The stop grace, output cap and query ceiling are memory and cleanup bounds and can be cited by measurement under any
option.

### Waits on the commit service itself

Covers: `START_WAIT` 10 s, `COMMIT_WAIT` 600 s, `RETIREMENT_WAIT` 10 s, `EXCHANGE_WAIT` 20 s, `REQUEST_READ_TIMEOUT`
10 s,
`RECONNECTS` 5.
`DESIGN.md:124` says a slow start and a version mismatch are "resolved without the user".
The message for `COMMIT_WAIT` tells the person to "raise the waiting limit in the settings", and no setting exists.

- A. No wall-clock client bound: heartbeats from the service while it works (the unbuilt design's "heartbeats instead
  of
  `COMMIT_WAIT`"), and a start that waits until the manager reports failure.
Visible result: no "waited N minutes ... not made" and no "did not start" for a slow start; position notices continue.
- B. Keep the bounds and cite them with measurements (a cold start of the service, the queue time at many committers:
  the
  commit-timeouts record measured makespans of up to about 17 s at 32 at once).
  Visible result: the same failures at measured numbers.
- C. Make `COMMIT_WAIT` a real setting, as the message says.
  Visible result: the message becomes true; the default is unchanged.
  C and B are complements.

Ranking: A > B > C, because A removes the failure the owner's rule says the system should resolve, and B keeps it but
bases it on evidence; B > C because C makes the message honest without giving the number a basis.

### Foreign-owner patience

Covers: `FOREIGN_OWNER_PATIENCE` 3600 s (`pid_namespace.rs:19`).
`doc/planning/cli-git-spawn-and-sync-reduction.md:713` already calls it an open value call for the owner: a fresh
ownerless
directory stops every guarded command for up to an hour, and a shorter bound widens the window in which a live wrapper
whose record was removed by hand could lose its directory.

- A. Keep an hour as the owner's decision.
Visible result: unchanged; a stuck directory blocks guarded commands up to an hour with a message that names the
  manual
  way out.
- B. Tie it to the live-lock wait (30 minutes in the planning document's suggestion).
  Visible result: stuck directories clear in half the time; the risk window for a live idle wrapper doubles.
- C. Measure the longest time a live wrapper sits with no file open (from the service and timing logs) and set the
  patience
  from it.
  Visible result: a number with a basis; the block lasts as long as the longest observed idle.

Ranking: C > B > A, because both trade waiting against a risk to a live wrapper's data and C is the only option that
measures the risk; B > A because "You should (almost) never be waiting." favors the shorter bound once the risk is
stated.

### Parallelism caps and memory estimates

Covers: copy threads 8, install and clone thresholds, `MOST_CONCURRENT_QUERIES` 4, `CPU_MARGIN_DIVISOR` 8,
`MEMORY_SHARE_DIVISOR` 2, `REPOSITORY_MEMORY` 64 MiB, `QUERY_MEMORY` 32 MiB, `SLICE_MEMORY` 16 MiB, `ENTRY_MEMORY` 64
MiB.
With the unit caps gone, these are the only numbers that size how many repositories the service drains at once.
The owner said "make sure we're running everything that can be run in parallel in parallel."
`CONVERSION_STORAGE_CEILING` and `CONVERSION_MEMORY` show the pattern the other numbers lack: a measurement beside the
value.

- A. Measure each: scaling at 1, 2, 4, 8, 16 workers on a realistic repository for the thread caps, and the resident
  peak of
  one item for each memory estimate, and write the result beside the value.
  Visible result: the numbers may change; each has a basis; the add and the round may get faster or slower.
- B. Remove the caps and the divisors and use every thread and all the free memory.
  Visible result: the fastest on an idle host and the heaviest on a shared one; no basis exists for the choice either.
- C. Keep the numbers and label them estimates.
  Visible result: comments only; `LQM` is still not met.

Ranking: A > B > C, because A meets `LQM` and the owner's parallelism request with evidence; B meets the request but
trades
one unmeasured number for another; C does not meet `LQM`.

### Deadlines for tools and checks that are not Git

Covers: `TOOL_LIMIT` 600 s for a package-manager tool during a worktree add, `LINT_DEADLINE` 60 s, the review
`SCRIPT_DEADLINE` 30 s and `BUDGET` 20 s.
The owner chose "Install online during the add", so an install can legitimately run long.

- A. End a tool or check only when it shows no sign of work for a window (the push's signs); no wall clock.
Visible result: a long install is never reported as "ran past its time limit"; a hung tool is stopped after the
  window.
- B. Keep the deadlines and measure them (a cold `pnpm install` of this repository, the slowest lint of a large
  Markdown file).
  Visible result: the same failures at numbers with a basis.
- C. Notice only.
  Visible result: a hung tool holds the add until the person interrupts it.

Ranking: A > B > C, because A never cuts work that is making progress, and B cuts at a number that a heavier
repository
will exceed; B > C because C lets a hung tool hold the add with no end ("a hung tool must not hold an add forever").

### Defaults for saved conflict copies

Covers: `DEFAULT_MAX_AGE_DAYS` 30, `DEFAULT_MAX_BYTES` 1 GiB, `YOUNG_FLOOR_MILLISECONDS` 24 h
(`discarded_prune.rs:35`, `38`, `41`).
The rule ("Prune by both age and size.") is the owner's; the handover calls the numbers "The coordinator's defaults".
The pass deletes copies that are "the only remaining content of replaced files".

- A. Record the current defaults as the owner's decision.
  Visible result: nothing changes; the numbers are cited.
- B. The owner names the numbers.
  Visible result: defaults change to those numbers.
- C. Measure how large and how old the saved-copies folders get across the repositories in use, and set the defaults
  from
  the distribution.
  Visible result: defaults with a basis; possibly larger than 1 GiB if the data say a 30-day copy is routinely bigger.

Ranking: C > B > A, because the folder holds the last copy of a person's replaced file and only C bases the loss
threshold
on how the folder actually grows; B > A because A approves numbers the owner has not seen.

### The audit and timing logs' history

Covers: `ROTATE_AT_BYTES` 1 MiB in `discard_audit.rs` with one `.1` generation, shared by `cli-git-audit.jsonl`,
`cli-git-commit-timing.jsonl`, `cli-git-worktree-add-timing.jsonl`, `cli-git-push-trigger.jsonl`.
The next rotation replaces `.1`, so older history is dropped; the race fix on `fix/git-policy-cli-audit-rotation-race`
does not change that.
The audit file exists to attribute lost uncommitted work and each line names the saved copies it made.
The standards-review log has no rotation at all.
Does "never discard user data" cover this audit history?

- A. Keep one generation as today.
Visible result: nothing changes; after two rotations the earliest decisions and the names of the saved copies they
  made
  are gone, and a saved copy can remain with no line that names it.
- B. Keep every generation (`.1`, `.2`, and on).
  Visible result: history is complete; the common Git directory grows without a limit, at a rate nobody has measured.
- C. Keep generations up to a measured or owner-chosen total size, and never drop an audit generation that names a
  saved
  copy that still exists; the timing logs stay at one generation.
Visible result: the audit trail lasts as long as the copies it explains; the timing logs are unchanged; the size bound
  has a
  basis.

Ranking: C > B > A, because C keeps the part of the history that explains user data and still bounds the rest; B > A
because
B never loses a line but has no size basis, while A silently drops what a person needs to find their copy.

### LFS server-error retry

Covers: `RETRY_DELAYS` 2 s and 5 s (`lfs_server_retry.rs:23`).
An HTTP 5xx from the LFS server is not on either of the owner's lists.

- A. Treat 5xx as outside our control: keep a bounded retry, with the numbers cited by the server's observed recovery
  time.
  Visible result: the "run again in 2.0 seconds (retry 1 of 2)" line stays.
- B. Treat it as designed out: end the step at once and let the service's push retry plan (10 s pauses, growing to 600
  s)
  try the push again.
  Visible result: the first 5xx prints the failure and the plan note; there is one retry mechanism.
- C. Owner puts 5xx on one of the two lists first, and the choice follows.
  Visible result: none yet; this is the missing classification.

Ranking: C > B > A, because the classification decides which of the others is allowed, so it comes first; B > A
because B
removes a second retry mechanism and its two uncited numbers, while A keeps both.

Delegated pick, 2026-11-03: the classification "outside our control" (the owner's list, like "Network down")
and B's single mechanism, built; see the outcome under "LFS server-error retry".
The coordinator picked it under the owner's delegation, and the owner may veto it.

### Sizes and ages of the service's own housekeeping

Covers: `MAX_BATCH` 64, `MAX_LINE_BYTES` 1 MiB, `MAX_CHANGES` 4096, `CLAIM_LEASE` 120 s, `DONE_RETAINED` 1024,
`REMEMBERED`
128, `IDLE_EXIT` 3 h, `TEMPORARY_GRACE` 10 min, `STRAY_TEMPORARY_AGE` and `SCRATCH_STALE_AFTER` 24 h, `WALK_BUDGET`
4096.
Only `MAX_BATCH` has a measurement beside it, and its largest observed batch equals the cap.

- A. Measure each from the service's own journal and timing logs (batch sizes, request sizes, hold times, restarts)
  and
  write the result beside the value.
  Visible result: a basis for each; some numbers may move.
- B. The owner names the numbers.
  Visible result: values and basis in one decision.
- C. Remove the ones that only guard hostile input or a case that cannot occur, and keep the rest uncited.
  Visible result: fewer limits; `LQM` is still unmet for the rest.

Ranking: A > B > C, because the logs already hold the distributions A needs; B > C because C leaves unmeasured numbers
in
place and a limit that was removed once might be needed.

### Margins for removing left-behind locks

Covers: `MARGIN` 2 s, `BOOT_SLACK` 1 h, `EMPTY_LOCK_SETTLE` 1 s, `COARSE_CLOCK_SLACK` 50 ms, `REUSE_SLACK_MS` 100.
These decide when the wrapper removes a lock, so a margin too short could remove a live Git's lock.
`SPEC.md:1830` measured that Git fills a reference lock after creating it empty, with no duration.

- A. Measure the delay between a lock's creation and its fill under this host's load, and set each margin from it.
  Visible result: margins with a basis, possibly larger than two seconds on a loaded host.
- B. Keep the margins and cite the reasoning in `SPEC.md`, labelled as reasoning.
  Visible result: no change; `LQM` is not met.

Ranking: A > B, because a wrong margin can remove a live lock, which is the one error the section's own rules treat as
unacceptable, and only A bases the margin on the host.

### The free-space preflight

Covers: `MIN_FREE_BYTES` 5 GiB (`subvolume_preflight.rs:26`), "about 1 KiB per entry is an assumption, not a
measurement".

- A. Compute the need from the entry count with a measured cost per entry (convert a real worktree and read the
  metadata
  growth).
Visible result: a small repository no longer needs 5 GiB free; a large one may need more; the refusal names the
  figure.
- B. Keep 5 GiB as the owner's decision.
  Visible result: unchanged.
- C. Drop the preflight and rely on the disk-full failure.
  Visible result: a full disk mid-conversion leaves a journaled partial migration for the next run to finish.

Ranking: A > B > C, because A matches the need to the repository and cites a measurement; B > C because refusing
before the
migration starts is safer than failing in the middle, even when the number is the owner's.

## Questions raised while building picks 5 and 12

Added 2026-11-03 by the agent that built the two picks.
The coordinator picks under the owner's delegation.

### The stall window of a push with no connection, and a connection that holds and never answers

What the code can observe about a push, with no time window and no invented number:
whether some process of its tree holds a TCP socket in state `ESTABLISHED` (the kernel's socket diagnostics),
the bytes that socket carried, and the three signs from `/proc` (bytes, processor time, a task in state `R` or `D`).
It cannot tell a slow remote from a live remote that is hung.
It cannot tell a dead peer from a slow one until TCP gives up
(retransmission for unacknowledged data; keepalive probes for an idle connection,
which Git's HTTP client turns on and `ssh` and `git://` leave to the kernel's default).
A tree with no connection, such as a push to a local path, a connect that has not completed, or a name lookup,
shows nothing a clock can read either, and today it is stopped after the 30 second stall window,
which no measurement or owner decision determines.

- A. As built: an open connection is waited for in every phase;
  30 seconds stays for a tree with no connection and no sign of work, uncited.
  Visible result: a remote that accepts the connection and never answers holds the push until TCP or the remote ends it;
  a push while offline is stopped after 30 seconds and retried by the plan.
  Pro: the pick as written, and no number for "connected".
  Con: one uncited number remains, and a hung live remote blocks the repository's pushes with no end.
- B. A with the first answer excepted: an open connection before any pack was seen is a stall after the window,
  as 0.5.27 has it.
  Visible result: a remote that is slow to answer the first request is stopped after 30 seconds and retried.
  Pro: a hung remote cannot hold the service for ever.
  Con: contradicts "no stall window for a connected push", and keeps the number.
- C. A, with the 30 seconds measured: time, on this host, from the start of a push to its first established socket
  and its longest silence with no connection over many pushes, and write the figure beside the value.
  Pro: satisfies the citation rule without changing behavior.
  Con: needs new logging first, because no log holds it, and it measures one host.
- D. Drop the window for a tree with no connection too, so the code stops a push only on a closed connection, or never.
  Pro: no number at all.
  Con: a hung local hook or a lookup that never ends holds the repository's pushes for ever.

Ranking: A > C > B > D, because A is the delegated pick and changes the least behavior;
C > B because C keeps behavior and adds the missing basis while B reverses part of the pick;
B > D because B still ends a hung state that D leaves for ever.
C is a complement of A and can follow it.

### Limits that remain beside the push, with no citation

`STOP_GRACE` 5 s (partial), `MAX_PUSH_OUTPUT` 4 MiB and `QUERY_CEILING` 120 s have no measurement or owner decision
that determines them.
The audit said they could be cited by measurement under any option.
They are marked as uncited beside the value in the code and in `SPEC.md`, with what exists, and not measured here,
because a measurement needs a load the brief forbids
(a push to a remote that prints megabytes, and a stopped Git whose locks are timed).

- A. Measure each from the service's own journal once it logs the data (push output size, time from stop to child exit).
- B. The owner names the numbers.
- C. Remove `QUERY_CEILING` with the Git child steps work (its kill is the audit's question on Git child steps)
  and keep the other two uncited.

Ranking: A > B > C, because the journal already holds most distributions A needs and C leaves two numbers uncited.
