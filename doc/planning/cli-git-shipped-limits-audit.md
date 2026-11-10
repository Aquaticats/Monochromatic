# Shipped limits of the git wrapper against LQM

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

This is an audit,
not a decision.
It lists every numeric bound the shipped Rust git wrapper (`monochromatic-git-policy-cli`,
binaries `cli-git-native`
installed as `git` and `cli-git-service`) can make a person hit,
and says for each whether a measurement or an owner
decision is written beside the value.
It changes no product code and decides nothing;
the questions at the end are for the owner.
Two entries and their questions carry outcomes added later on
`feat/git-policy-cli-saved-copies-and-audit-generations-measured`:
"Saved copies of conflicted files" (delegated pick 10,
no number could be set) and
"Audit log rotation" (delegated pick 11,
implemented).
Each says it was the coordinator's pick under the owner's delegation and is open to the owner's veto.
The owner's delegation,
quoted:
"Pick options as if I had picked them and invite veto when I say I'm back."
Two more entry groups carry outcomes added on `feat/git-policy-cli-steps-end-only-on-no-progress`:
the Git child steps
of a
commit (delegated pick 4,
option B) and the deadlines of tools and checks that are not Git (delegated pick 9,
option A),
with the read-only query ceiling added to both by the coordinator,
and the new section "Quiet gaps in the existing
logs".
Two more groups of entries and their questions carry outcomes added on
`feat/git-policy-cli-index-lock-and-service-waits`
(2026-11-03):
delegated pick 2 ("What a live or unproven index lock holder costs",
option A) and delegated pick 6
("Waits on the commit service itself",
option A).
Each says it was the coordinator's pick under the owner's delegation ("Pick options as if I had picked them and invite
veto
when I say I'm back.")
and is open to the owner's veto.
Four follow-up picks of the coordinator's (2026-11-03,
same delegation,
open to the owner's veto)
are recorded where they
apply:
an owner nothing proves alive is waited for like a proven one;
the waits poll where no watch can be set;
an inactive service unit is asked to start after the start wait;
a waiting command renews its scratch directory.
The 150 s hold age is not among them and awaits the owner's approval.

Audited commit:
`d48d0a66f4` (main).
Rule `LQM` of `AGENTS.md` (line 1357),
quoted verbatim from the owner's approval:
"Shipped resource limits (unit caps,
timeouts,
size bounds) cite a workload measurement or owner decision beside the
value;
agent-command cgroup rules never size shipped code."

Paths in this document that start with `src/` or name `SPEC.md`,
`README.md` or `DESIGN.md` are relative to the crate
directory
`package/git-policy/cli`.
Paths that start with `doc/` are relative to the repository root.

## Method and scope

### What counts as shipped

- Shipped:
  the modules `src/native/lib.rs` declares without `#[cfg(test)]`,
  `src/native/main.rs`,
  `src/native/service_main.rs`,
  and `src/native/service_units/*`.
- Not shipped,
  so not audited:
  every `*_tests.rs`;
  every module that `lib.rs` declares under `#[cfg(test)]`
  (`test_support`,
  `command_test_support`,
  `command_test_cases`,
  `command_test_parseopt`,
  `command_test_completion`,
  `policy_test_support`,
  `scanner_test_support`,
  `subvolume_test_support`,
  `fs_btrfs_test_support`,
  `parallel_test_support`,
  `worktree_ladder_test_support`,
  `dependent_version_test_support`,
  `dependent_version_fixture_*`,
  `recording_git`,
  `stale_ref_lock_killed_git`,
  `standards_review_mock`).
- The `binary_*.rs` files (for example `binary_wrapper_store.rs` with `WAIT_CAP` and `IDLE_BUILD_AGE`,
  `binary_support.rs` with `SOCKET_RELEASE_PATIENCE`,
  `binary_supervisor.rs` with `NO_PROGRESS_WINDOW`) are compiled
  only into the `native_binary` test target (`src/native/binary_tests.rs` declares them as its modules;
  `Cargo.toml`
  names that target),
  never into `lib.rs` or the two binaries.
  Their waits are test-harness bounds and are out of scope.
- The environment variables named `CLI_GIT_TEST_ONLY_*` (`transaction_phase.rs`,
  `service_fault.rs`,
  `client_journal.rs`) are test seams that live in shipped files;
  their polls are not listed.
- Windows and macOS are out of scope until the owner says otherwise ("Ignore Win and mac until I say so."),
  so the
  start-time resolutions of those platforms in `src/native/process_start.rs` are not listed.

### What counts as a limit

A limit is listed when its effect changes with the workload or the clock:
a timeout,
a wait,
a retry count,
a backoff,
a poll or notice interval,
a size or entry bound,
a queue,
frame or batch size,
a retention age,
a parallelism cap or
memory estimate,
an environment-variable default,
or a setting the wrapper passes to a program it starts.
Excluded,
with one sentence each:

- Format and protocol constants (hash lengths,
  index header and stat sizes,
  file mode bits,
  ioctl numbers,
  D-Bus
  codes,
  base64 and SHA tables,
  option-table ordinals,
  exit codes,
  schema versions):
  they are fixed by a specification,
  not
  chosen for a workload.
- Git's own limit values the wrapper only mirrors (`GIT_MAX_CONFIG_COUNT`,
  `MINIMUM_ABBREVIATION`,
  `FALLBACK_ABBREVIATION`,
  `BACKOFF_MAX_MULTIPLIER` and the jitter numbers in `src/native/index_lock_wait.rs`):
  the
  wrapper needs the same number as Git to behave as Git does.
The reader bounds that merely say they equal a Git limit are listed,
  because no Git source line is written beside
  them.
- The `Debug`-level and test-only values.

### Searches that produced this list

All were run over the shipped file set (446 files after the exclusions in "What counts as shipped").

- Durations and sleeps:
  `Duration::`,
  `from_secs`,
  `from_millis`,
  `from_micros`,
  `from_nanos`,
  `sleep(`,
  `Timeout`.
- Constants:
  every `const` or `static` whose type is an integer,
  float or `Duration`,
  then read by name for limit-like
  words (`LIMIT`,
  `MAX`,
  `BUDGET`,
  `WAIT`,
  `PAUSE`,
  `POLL`,
  `INTERVAL`,
  `ATTEMPT`,
  `RETRY`,
  `AGE`,
  `BATCH`,
  `DEPTH`,
  `GRACE`,
  `PATIENCE`,
  `CEILING`,
  `LEASE`,
  `IDLE`,
  `CAPACITY`,
  `WORKERS`,
  `MEMORY`,
  and similar).
- Literals outside constants:
  `GiB`,
  `MiB`,
  `<< 20`,
  `<< 30`,
  `.take(`,
  `.min(`,
  `chunks(`,
  `available_parallelism`.
- Environment:
  every `CLI_GIT_*` name and every `env::var` read,
  then each default and parse rule.
- Messages:
  `giving up`,
  `still waiting`,
  `was still running`,
  `made no progress`,
  `limit=`,
  `after {} seconds`.
- Systemd:
  `systemd-run`,
  `CPUQuota`,
  `MemoryMax`,
  `MemoryHigh`,
  `TasksMax`,
  `IOWeight`,
  `WatchdogSec`,
  `RuntimeMaxSec`,
  `StartLimit`,
  `RestartSec`,
  `Restart=`,
  `TimeoutStartSec`,
  `--property`.
- Positive controls:
  the systemd search printed the four caps of `src/native/service_units/cli-git@.service` and
  nothing else,
  and a search for `MemoryMax` alone hit that same file,
  so the zero hits in code are real.
  The duration search hit the known constants (`COMMIT_WAIT`,
  `DEFAULT_LIVE_WAIT`),
  and the message search hit the
  known "giving up after" sentences of `src/native/service_index_wait.rs`.
- Result:
  the wrapper starts no `systemd-run` unit and passes no cgroup property anywhere in shipped code.
  The only systemd settings it writes are in the two unit templates.
  The memory constants (`REPOSITORY_MEMORY`,
  `QUERY_MEMORY`,
  `SLICE_MEMORY`,
  `ENTRY_MEMORY`,
  `CONVERSION_MEMORY`,
`INDEX_REWRITE_COPIES`) feed only `src/native/parallel_budget.rs`,
  which divides the memory the kernel reports by them
  to choose a thread count;
  none becomes a unit property or a `ulimit`.
- Not done:
  no test,
  build or process was run.
  Message texts are read from source,
  not produced.

### Branches that change the picture

The audit reads main.
Three decided changes are not on main yet,
and one design is still in review.

- `fix/git-policy-cli-service-no-memory-cap` (`741d42932e`,
  `474c987ae2`):
  removes the four unit caps.
  `git diff main...` shows it edits `src/native/service_units/cli-git@.service`,
  `SPEC.md`,
  `doc/planning/cli-git-resident-service.md` and two test files.
  On main the unit still carries the caps,
  and `SPEC.md:2515` still says the unit "allows two CPUs and one gibibyte".
- `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits`:
  "Wait while connected" for pushes (handover
  `doc/handover/cli-git-rust-implementation.md:327`).
  Update,
  2026-11-03:
  it is in the 0.5.27 integration tip,
  and picks 5 and 12 were then built on branch
  `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx`;
  the entries of "Pushes" carry their outcomes.
Read with `git show`:
  its `bounded_push.rs` keeps `DEFAULT_PUSH_DEADLINE` (30 s),
  `DEFAULT_PUSH_CEILING` (1800 s) and
  the
  low-speed constants at the same values and stops giving a push the low-speed abort (its lines 221 to 224).
  The progress rule changes (bytes on the push's socket count),
  the numbers do not.
- `fix/git-policy-cli-audit-rotation-race` (`2eb2cf1bfc`):
  fixes the race of the audit log rotation;
  the bound and the
  one-generation history are unchanged (its `discard_audit.rs:27`).
  Since this audit:
  the race fix is in the base of
  `feat/git-policy-cli-saved-copies-and-audit-generations-measured`,
  which
  also replaces the one-generation history of the audit file with generations (see "Audit log rotation,
  1 MiB and one
  generation").
- `fix/git-policy-cli-commits-never-fail`:
  the design that is meant to replace the interim loops and waits;
  not built.

### Citation classes

- cited:
  a measurement (with where it was taken) or an owner decision determines or is written beside the value,
  and I
  give the path and line.
- partial:
  a measurement or an owner decision is beside the value but does not determine it (the rule is decided and
  the number is not;
  the typical case is measured and the bound is a judgement).
  The owner's complaint was about the number,
  so partial counts as uncited for LQM.
- none:
  only a rationale,
  a precedent,
  an estimate,
  or nothing.
  A sentence that says why a bound exists is not a citation of its value.
- Doc lines that merely repeat the value (`README.md`,
  `SPEC.md` listing "10 minutes") are not citations.

## Delegated picks applied on 2026-11-03

The owner was away and delegated the answers to the questions at the end:
"I have to be away for around 8 hours.
Pick
options as if I had picked them and invite veto when I say I'm back."
Clarified:
"\"As if I had chosen\" means what I would have picked most likely if I'm always here."
Every pick under the `Delegated picks applied on 2026-11-03` heading
is therefore the coordinator's pick under that delegation,
open to the owner's veto;
none of them is
an owner decision,
and each doc and code comment that cites them says so.

These picks were applied on branch `fix/git-policy-cli-limits-cite-measurements`,
from the 0.5.27 candidate tip
`3848f9581d`,
which already carries the removal of the four unit caps (`741d42932e`):

- Which limits need a citation under `LQM`:
  option B (see "Exempt limits").
- Foreign-owner patience:
  option C,
  measured from the logs.
- Parallelism caps and memory estimates:
  option A,
  measured.
- Sizes and ages of the service's own housekeeping:
  option A,
  measured from its journal and timing logs.
- Margins for removing left-behind locks:
  option A,
  measured on this host.
- The free-space preflight:
  option A,
  computed from the entry count with a measured cost per entry.

Left to other agents and not touched here:
pushes and LFS,
saved conflict copies and audit or timing log history,
the
Git child step bounds and the non-Git tool deadlines,
the index-lock holder wait and the waits on the service,
and the
commit service unit (it has no limits,
by owner decision).
Where one of the limits in the `Left to other agents and not touched here` list
sits in a file those agents own,
its entry is unchanged and says so.

Each measurement is recorded with its command,
window and sample count in "Measurement record".
Where the data cannot support a value,
the entry says so and the question lists options with a ranking;
no number was
invented.


## Owner decisions that bear on limits

Quoted;
none is reinterpreted.

- "You should (almost) never be waiting."
  (`SPEC.md:1647`)
- "never fail doesn't primarily mean retries!"
  (`SPEC.md:1168`)
  A failure that can happen is "a design defect to remove by construction";
  "A retry is at most the last resort for a
  cause outside the system".
Outside our control (`SPEC.md:1184` to `1200`):
  network down,
  the remote refusing (auth,
  a server hook,
  GH009,
  branch
  protection),
  disk full or quota,
  disk I/O errors,
  a memory or process-count cap set by someone else,
  a kill or power
  loss,
  a corrupt repository,
  the person's signer failing.
  Designed out (`SPEC.md:1202` to `1206`):
  a slow remote,
  other programs' locks,
  Git's own crashes or bugs.
- "Auto merging should never fail.",
  "Manual merge should never fail too.",
  "Commits should never fail."
  (`SPEC.md:1162` to `1164`)
- "commits vs native git should not take 5s longer."
  (`README.md:1419`,
  `SPEC.md:3977`)
- Pushes,
  "Wait while connected (Recommended)" (`doc/handover/cli-git-rust-implementation.md:327`):
  bytes on the
  push's socket count as progress;
  once the pack is sent,
  keep waiting while the connection is open.
  "Stop only on no progress (Recommended)" (same handover,
  line 913).
- `doc/decision/cli-git-concurrent-commits.md:151` to `158`:
  an `index.lock` owner proven alive "gets an unbounded
  wait
  with one line naming the holder";
  a dead or evidence-free owner gets quadratic backoff "up to
  `indexLock.unprovenOwnerTimeoutMs` (default 1000)".
  Line 347 of the same record repeats that a live owner gets an unbounded wait.
- "Same as the ref lock (Recommended)" for a live foreign `index.lock` (`SPEC.md:1234`):
  a commit lands and exits 0
  and
the index catches up when the lock is free;
  a merge saves on a side ref and finishes the checkout when the lock frees.
  Recorded as answered and not yet built (handover lines 281 to 292).
- "Ship interim now (Recommended)" (handover line 284) covers the 0.5.26 interim code:
  the index-lock wait and a
  bounded
  moved-head loop.
  It does not name the per-class child kill,
  the push limits or any other bound.
- "Stack on the new head (Recommended)" for an amend,
  a conclusion or a revert on a head that moved (`SPEC.md:1227`
  to `1228`).
- "Offline is retried,
  but auth and server hook declines stop until your next commit."
  (`SPEC.md:1270`)
- "60s is fine for adding worktrees.
  And make sure we're running everything that can be run in parallel in parallel."
  (handover lines 864 and 865)
- "never discard user data" (`SPEC.md:1645`).
- "Remove all three (Recommended)" for the CPU,
  task and disk caps of the commit service unit (handover lines 148 to
  160).
- "1800s for git pack-objects is laughably inappropriate."
  (`doc/planning/cli-git-commit-timeouts.md:4`)
- "Races with other programs ... That is not an our problem.
  By using this command or this program,
  make the user
  understand and accept the risk."
  (handover lines 854 to 858;
  adopted reading,
  veto open)
- `DESIGN.md:124`:
  "Whatever the system can resolve itself (a version mismatch,
  a stale socket,
  a crashed service,
  a
  slow start,
  a client killed mid-request) is resolved without the user".

## Summary

- Limits listed:
  73;
  grouped entries (for example the three copies of an include-depth bound) count once.
- Counts by class are in the final section "Counts".
- Cited:
  the conversion storage ceiling,
  the conversion memory estimate,
  the `checkout.workers=0` setting,
  and the
  review
  gateway call deadline.
- Partial:
  the two per-class Git child bounds,
  the Markdown linter deadline,
  the push stall window,
  the batch size,
  the saved-copies age and size defaults,
  and the reader bounds that name a Git limit without a Git source line.
- Four unit caps:
  decided to be removed,
  removal unmerged.
- Findings that are not about a missing citation:
  - The 1800 second wait for a live holder conflicts with the decision record's unbounded wait.
  - The same wait is also applied to an unproven owner,
    where the decision record says 1000 ms of backoff.
  - `COMMIT_WAIT` has no setting,
    yet its message tells the person to raise it in the settings.
  - `FOREIGN_OWNER_PATIENCE` is already marked as an open value call for the owner.
  - `DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS` is a dead constant and the decision record still names it.
  - Several retry counts bound causes the owner put on the "designed out" list.

## The commit service unit

The unit templates are `src/native/service_units/cli-git@.service` and `cli-git@.socket`.
`Restart=on-failure` (line 14) is written without `RestartSec` or `StartLimitBurst`,
so systemd's defaults apply
(on this host `systemctl show` reports `DefaultStartLimitIntervalUSec=10s` and `DefaultStartLimitBurst=5`,
so a sixth
start within ten seconds is refused).
That is an inherited bound the unit does not state;
it is not counted.
The socket template sets only the mode (`0600`) and directory mode (`0700`),
which are not limits.

### Unit MemoryMax

- Where:
  `src/native/service_units/cli-git@.service:15`.
- Value:
  `MemoryMax=1G`.
- Bounds:
  the whole service cgroup,
  including the Git children it starts.
- When hit:
  the kernel reclaims,
  then kills;
  the owner reported 274,736 hits of the cap on the main checkout and
  `memory.peak` at 1,073,766,400 bytes (handover lines 148 to 150).
- Fails or waits:
  slows,
  then kills the service,
  which the manager restarts on the next connection.
- Citation:
  none on main.
  Owner decision to remove it:
  handover line 159 and 791 ("Remove all three (Recommended)",
  and the memory cap was found unmeasured);
  removal is on `fix/git-policy-cli-service-no-memory-cap`,
  unmerged.
- Conflict:
  with LQM and with "never fail" (a memory cap is "set by someone else" only when someone else sets it;
  this
  one was the wrapper's own).
  Decided,
  removal unmerged.

### Unit TasksMax

- Where:
  `src/native/service_units/cli-git@.service:16`.
- Value:
  `TasksMax=512`.
- Bounds:
  processes and threads of the service cgroup.
- When hit:
  a thread or process start fails;
  `src/native/service_server.rs:589` to `594` shows the acceptor backing
  off
  when it cannot start a thread,
  and `src/native/start_retry.rs` retries a refused process start.
- Fails or waits:
  waits up to the start-retry budget,
  then the connection or step fails.
- Citation:
  none on main;
  decided removal as recorded in "Unit MemoryMax".
- Conflict:
  the same as in "Unit MemoryMax";
  decided,
  removal unmerged.

### Unit CPUQuota

- Where:
  `src/native/service_units/cli-git@.service:17`.
- Value:
  `CPUQuota=200%`.
- Bounds:
  CPU time of the service cgroup.
- When hit:
  throttling.
- Fails or waits:
  waits (slower commits and slower after-commit work).
- Citation:
  none on main;
  decided removal as recorded in "Unit MemoryMax".
- Conflict:
  "commits vs native git should not take 5s longer" could be hit by a throttled service;
  decided,
  unmerged.
- Side effect to carry into the question on parallelism:
  `src/native/service_round.rs:129` sizes a round by the CPUs
  the
  process may use,
  so the unit's two CPUs set the round width on main;
  after removal the host's CPUs do.

### Unit IOWeight

- Where:
  `src/native/service_units/cli-git@.service:18`.
- Value:
  `IOWeight=50`.
- Bounds:
  the service's share of block I/O under contention.
- When hit:
  slower I/O when other cgroups compete.
- Fails or waits:
  waits.
- Citation:
  none on main;
  decided removal as recorded in "Unit MemoryMax".
- Conflict:
  none beyond LQM;
  decided,
  unmerged.

## Waits for locks and for the service's index hold

### Live-holder wait, 1800 seconds

- Outcome (delegated pick 2,
  option A;
  open to the owner's veto):
  the 1800 seconds and `CLI_GIT_LOCK_WAIT_SECONDS` are gone,
  with `src/native/wait_limit.rs`.
  A holder proven alive is waited for with no bound,
  in the merge wait (`index_holder_wait.rs`),
  the service's landing
  (`service_index_attempts.rs`),
  the index reconciliation (`service_land_follow.rs`)
  and the forwarded writers' wait for the service's hold (`service_index_wait.rs`).
  The line names the holder and its proof and says the wait lasts as long as the process holds the lock.
  A dead owner (Git's lock PID file names a process that is gone) gets a diagnostic at once.
- Follow-up outcome (coordinator's pick under the delegation,
  2026-11-03;
  open to the owner's veto):
  an owner nothing proves alive is waited for the same way,
  with no bound.
  The first version gave it a 1000 ms backoff (`UNPROVEN_OWNER_BACKOFF`,
  with a measurement) and then a diagnostic;
  that
  constant and `src/native/unproven_backoff.rs` are gone,
  because the person's own `git commit` with its editor open holds
  `index.lock` with no proof,
  and a backoff would make our commit fail after it ("Commits should never fail.").
  The line says what is known ("whose owner cannot be proven alive (no process the scan could see has the lock open,
  and no
  Git lock PID file beside it ...)"),
  repeats every notice interval,
  and names the file to remove if no program is using the
  repository,
  which is the one case the wait cannot end by itself.
  Rule `LQM` has no number left to cite for this wait.
- The measurement of Git's own holds (kept,
  now beside `INDEX_ATTEMPTS` in `service_land_follow.rs`):
  20 runs each of
  `git status`,
  `git add -A`,
  `git reset -q` and `git update-index --refresh` on a copy of this repository's index
  (20,739 tracked files,
  2.9 MB),
  2026-11-03:
  at most 100 ms (medians 26 to 53 ms).
  No journal or log holds a foreign holder:
  none of 1,335 landings in the service journal recorded finding the lock held
  (no `lock_looks`),
  and none of the 1,090 records of `.git/cli-git-commit-timing.jsonl` has a waiting notice.
- Earlier entry,
  kept for the record:
- Where:
  `src/native/wait_limit.rs:26` (`DEFAULT_LIVE_WAIT`),
  variable `CLI_GIT_LOCK_WAIT_SECONDS`.
- Value:
  30 minutes,
  for a holder proven alive and also for one whose owner cannot be proven alive.
- Conflict,
  yes,
  with the decision record's lines 151 to 158 and with "Commits should never fail."
  and "You should
  (almost) never
  be waiting.";
  this is what the outcome above removes.
- The record's lines 155 to 158 (quoted):
  "A dead or evidence-free owner gets Git-style quadratic backoff with jitter
  up to
  `indexLock.unprovenOwnerTimeoutMs` (default 1000),
  then a diagnostic listing the evidence."

- Superseded 2026-11-03 by the owner's rule "nothing should ever wait forever or indefinitely" and his answers of
  about 12:45-05:00 (see the "Every wait has a bound" section):
  the wait for a foreign `index.lock` (proven alive or not)
  ends at `LOCK_CEILING`,
  10 minutes,
  unmeasured (no log holds a
  foreign holder).
  At the bound the landing is withdrawn before the branch moved,
  the built commit is kept on
  `refs/cli-git/kept/<batch>`,
  and the error names the blocker;
  the merge names it and ends with Git's refusal.

### Service hold age limit, 60 seconds

- Outcome (delegated pick 2;
  open to the owner's veto):
  `HOLD_LIMIT` is now `UNASKABLE_HOLD_AGE`,
  150 seconds,
  and the limit parameter of the wait is gone.
  A hold whose mark proves the service runs is waited for with no bound (it already was,
  apart from the 1800 seconds).
- Citation:
  the service's journal logged the time the service held the lock (`lock_hold_ms`) for 715 landings
  (2026-11-02 to 2026-11-03):
  median 34 ms,
  90th percentile 92 ms,
  99th percentile 65,787 ms,
  longest 119,721 ms.
  Thirteen holds ran 58 to 120 seconds,
  all in the disk sync of the new index (`sync_ms`),
  so a live service on a slow disk
  holds the lock for two minutes.
  The earlier 60 seconds took those for leftovers whenever the holder could not be
  asked about
  (another PID namespace).
  150 seconds is a quarter more than the longest measured hold.
- Deviation from pick 2's "short backoff":
  the measurement contradicts a short age for a live service's own hold,
  so this
  number is not short.
  It is listed under the open questions.

- Superseded 2026-11-03 by the owner's rule "nothing should ever wait forever or indefinitely" and his answers of
  about 12:45-05:00 (see the "Every wait has a bound" section):
  a hold the mark proves to run is waited for until `SERVICE_HOLD_CEILING`,
  10 minutes (five times the longest measured hold,
  119.7 s);
  `UNASKABLE_HOLD_AGE` (150 s) is unchanged and awaits the owner's approval.

### Look counts and pauses of the lock wait

- Outcome:
  unchanged.
  They decide only whether an empty lock is read as the service's or as a foreign program's (8 to
  40 ms),
  and the poll where no watch can be set (now `POLL_PAUSE`,
  classed exempt).
  No measurement was made for them;
  the measured holds of Git's own commands
  (15 to 100 ms) show an empty foreign lock is the usual case,
  so reading it as foreign after the looks is the right call
  whatever the exact count.
  Still uncited.

### Service scan budget, 250 milliseconds

- Outcome (delegated pick 2):
  cited beside the value (`index_lock_holders.rs`).
  Walking every process's open descriptors (`ls -l /proc/[0-9]*/fd`)
  over 858 processes took 0.10 s in each of three runs on
  2026-11-03,
  so 250 ms is two and a half times a whole walk.
  The scan is made at the first look and at each repeat of the
  notice,
  not at every wake of the watch.

### Index reconciliation tries, 5

- Outcome (delegated pick 2):
  the reconciliation of a commit whose corrections left no change now waits through
  `wait_for_index_release_here` like a merge:
  a holder proven alive is waited for with no bound,
  an owner nothing proves alive for
  the same unbounded wait,
  a dead one not at all.
  `INDEX_ATTEMPTS` (5) and `INDEX_PAUSE` (100 ms) stay for a lock that was gone by the time it was looked at;
  100 ms is the
  longest measured hold of Git's own short commands (the measurement is now beside `INDEX_ATTEMPTS`).
  That commit still exits 1 in any case,
  because it has no change left to record.

### Rerun of a forwarded index writer, 50 starts

- Where:
  `src/native/service_hold_retry.rs:28` (`ATTEMPTS`).
- Value: 50.
- Bounds:
  how many times `git add`,
  `rm` or `mv` is started again when the commit service takes the index first.
- When hit:
  the last refusal of Git ("Unable to create ... index.lock") is shown.
- Fails or waits:
  fails after the last start.
- Citation:
  none.
- Conflict,
  yes:
  the contender is the wrapper's own service,
  a cause the owner would count as designed out ("never
  fail
  doesn't primarily mean retries!").

### Foreign-owner patience, one hour

- Where:
  `src/native/pid_namespace.rs:19` (`FOREIGN_OWNER_PATIENCE`).
- Value:
  3600 seconds.
- Bounds:
  how long an owner record from another PID namespace counts as alive,
  measured from its write.
  `SPEC.md:739` to `740` uses it for work-item generations;
  `doc/planning/cli-git-spawn-and-sync-reduction.md:619`
  reuses it for ownerless directories.
- When hit:
  a fresh ownerless directory "stops every guarded command for up to an hour,
  with the message ... naming
  the
  manual way out" (`doc/planning/cli-git-spawn-and-sync-reduction.md:713` to `716`).
- Fails or waits:
  waits (commands refuse until the hour passes or the person clears the directory).
- Citation:
  none.
  That document marks it "Open value call for the owner" at line 713;
  no answer is recorded.
- Conflict,
  yes:
  "You should (almost) never be waiting."
  and "commits vs native git should not take 5s longer."

- After the delegated pick (option C,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  value
  kept.
  Three kinds of record are judged by this age:
  a merge's intent record (`merge_isolation_journal.rs`,
  `write_merge_intent`,
  written once and synced per merge),
  a claim file (`single_owner_claim.rs`,
  `write_private`,
  written once),
  and the item path of a worktree add (`worktree_copy_owner.rs`,
  `is_alive`,
  judged by its own
  modification time).
  The first two are not rewritten while the owner works,
  so the age is the owner's whole hold;
  the patience must outlast
  the longest hold.
  Measured:
  the longest worktree add took 539.7 s and the nearest log of a command that waits,
  the commit timing log,
  has a longest of 750.6 s ("Measurement record",
  service housekeeping).
  No log records the length of a merge,
  which waits up to the 1800 s live lock wait with its record already written,
  so an hour is twice the longest wait the code allows and 4.8 times the longest command measured.
  The comment is at `src/native/pid_namespace.rs:19`.
- Dependency:
  the 1800 s wait belongs to the pick on the index-lock holder wait,
  which other agents own.
  If that wait is lengthened or removed,
  this patience must be revisited with it,
  because a wrapper that waits holds
  its record for the whole wait.
- Limit of the data:
  the logs hold commands of the host's own PID namespace,
  so no hold of a foreign namespace was
  observed directly.
- Follow-up carried out 2026-11-04T09:35:19-05:00 (branch `fix/git-policy-cli-branch-write-and-tool-ceilings`):
  the dependency above had landed (the 1800 s wait is gone;
  a merge waits for a foreign `index.lock` at most `LOCK_CEILING`,
  10 minutes,
  and `wait_limit.rs` no longer exists),
  and the follow-up "derive the patience from the wait" (option B of
  "Foreign-owner patience and the live lock wait") was recorded and never run.
  `FOREIGN_OWNER_PATIENCE` is now `2 * LOCK_CEILING.as_secs()`,
  1200 seconds,
  in `src/native/process/pid_namespace.rs`,
  with the basis beside it:
  the longest a merge waits with its record written,
  twice over,
  which is 1.6 times the longest commit
  measured (750.6 s of 1,838 commits to 2026-11-04T14:08:47Z) and 2.2 times the longest add (539.7 s of 79).
  Tests:
  `the_patience_is_twice_the_lock_ceiling` (red against 3600 s) and `the_patience_outlasts_the_longest_measured_command`.
  The block of an ownerless directory by a foreign-namespace record is at most 20 minutes,
  not an hour.
  Still open:
  a foreign hold longer than 20 minutes (a merge whose own work,
  after its wait,
  runs long) would be judged dead;
  no log of a foreign namespace exists to measure it,
  and the margin is the coordinator's,
  open to the owner's veto.
- Note 2026-11-08T16:01:42-05:00:
  the basis this entry states is stale.
  The merge waits `MERGE_LOCK_CEILING` (one minute) since the entry
  "Timing record names every step;
  the other lock waits" of 2026-11-07,
  so twice the longest a merge waits is two minutes,
  not twenty;
  the constant keeps its value because another change owns it.
  The timing log read that day (1,332 records,
  "The lock ceiling and the three waits it fed" in the measurement record) holds a whole command of 1,228,797 ms,
  which is longer than the 20 minutes,
  though that figure is of a command and not of an owner record.

### Notice intervals and graces

- Where:
  `src/native/wait_progress.rs:33` (`DEFAULT_NOTICE_INTERVAL` 10 s) and `38` (`NOTICE_GRACE` 1 s);
  `src/native/service_index_lock.rs:49` (`NOTICE_EVERY` 10 s,
  parsed at line 65 from the same variable);
  `src/native/service_timing.rs:171` (`SLOW_LOCK_STEP` 1 s);
  `src/native/markdown_linter_guard.rs:152`
  (`NOTICE_POLL` 50 ms);
  `src/native/service_client.rs:31` (`START_NOTICE_GRACE` 1 s).
- Value:
  as listed;
  `CLI_GIT_WAIT_NOTICE_SECONDS` replaces the interval,
  zero or malformed values are ignored
  (`wait_progress.rs:93`).
- Bounds:
  how often a waiting command repeats its line.
  The notice line is "cli-git:
  still waiting for {what} (process {pid}) after {n} seconds;
  stopping it after {n}
  seconds{note}."
  (`wait_progress.rs:231`).
- When hit:
  a line is printed;
  nothing fails.
- Fails or waits:
  neither;
  display pacing.
- Citation:
  none (`README.md:908` and `1325` describe the variable;
  `doc/planning/cli-git-commit-timeouts.md:164` to
  `165` states the grace rule).
- Conflict:
  none found.

- After the delegated pick (option B,
  the coordinator's pick under delegation,
  open to the owner's veto):
  the notice
  intervals `DEFAULT_NOTICE_INTERVAL`,
  `NOTICE_EVERY`,
  `SLOW_LOCK_STEP` and the poll `NOTICE_POLL` are classed exempt (see "Exempt limits").
  `NOTICE_GRACE` is already exempt-classed in its own comment.
  The class graces in `child_class.rs` and `START_NOTICE_GRACE` in `service_client.rs` sit with the agents that own the
  child bounds and the service waits and are not touched.

### Retired unproven-owner timeout

- Outcome:
  the key stays retired.
  The 1000 ms it stood for was the fixed `UNPROVEN_OWNER_BACKOFF` of the first version
  of
  pick 2,
  and that constant is gone too (see "Live-holder wait,
  1800 seconds").
  `doctor_config.rs` and `config_schema.rs` no longer mention `CLI_GIT_LOCK_WAIT_SECONDS`;
  the doctor says what the service
  does instead.

## Git child steps of a commit

### Quick query step bound, 60 seconds

- Where:
  `src/native/child_class.rs:34` (`PLUMBING_CHILD_WAIT`);
  override `CLI_GIT_CHILD_WAIT_SECONDS`
  (`child_class.rs:30`,
  parse at `117`:
  a missing,
  zero or malformed value falls back to the class default).
- Value:
  60 seconds for `rev-parse`,
  `rev-list`,
  `symbolic-ref`,
  `for-each-ref`,
  `ls-tree`,
  `cat-file`,
  `diff-tree`,
  `merge-base`,
  `write-tree`,
  `update-ref`,
  `config`,
  `remote`,
  `version` (`child_class.rs:54` to `66`).
- Bounds:
  one Git step a commit starts (`src/native/child_watch.rs`,
  `src/native/transaction_git.rs:525`).
- When hit (`child_watch.rs:390`):
  "{what} did not finish within {n} seconds and was stopped
  (CLI_GIT_CHILD_WAIT_SECONDS changes this).
  If the step belonged to a commit,
  the next Git command of this wrapper
  finishes or undoes that commit,
  so run the command again.
  If the step was only slow,
  raise the bound with
  CLI_GIT_CHILD_WAIT_SECONDS=<seconds>;
  if it stalls again,
  check with `ps` what that process waits for."
- Fails or waits:
  waits,
  then the step is killed and the commit fails.
- Citation:
  partial.
  `child_class.rs:21` to `22` says "a hundred times the slowest such step measured on a host at several times its core
  count in load".
`doc/planning/cli-git-commit-timeouts.md:151` to `154` measured these steps on a 20,207-file clone of this repository
  at
  a load average of 80 to 145 on 28 cores (lines 9 to 18) and found the worst at 44 ms.
  A hundred times 44 ms is 4.4 s,
  not 60 s,
  so the stated derivation does not produce the value.
- Conflict,
  yes:
  "Commits should never fail" and Git hangs are "designed out";
  the 60 s kill turns a stall into a
  failed
  commit.
The one interim the owner approved ("Ship interim now") names the index-lock wait and the moved-head loop,
  not this
  kill.
  `doc/planning/cli-git-commit-timeouts.md:4` is the owner objecting to the older 1800 s bound;
  it does not choose 60.
- Outcome (pick 4,
  option B,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  the step is ended only when it has shown no sign of work for the window,
  judged with the signs of the push (bytes
  moved,
  processor time,
  a task running or waiting for the disk,
  an open connection) on the step and everything it started;
  elapsed
  time alone never ends it.
  `PLUMBING_QUIET_WINDOW` (renamed from `PLUMBING_CHILD_WAIT`) keeps 60 seconds as a **labelled unmeasured quiet
  window**
  with the citation "none" beside it,
  because no log held a quiet gap (see "Quiet gaps in the existing logs"),
  and
  `CLI_GIT_CHILD_WAIT_SECONDS` sets the window.
  A step that is safe to run twice (`may_be_rerun`) and was ended is run once more;
  a second quiet run fails with
  "showed no
  sign of work for {n} seconds and was stopped".
  The redo count of once is the coordinator's reading of the owner's
  "ended and its step redone" and is flagged under `Time limits on Git child steps of a commit`.
  A writer that may not be run twice (`update-ref`,
  a commit Git builds) is announced and waited for,
  never ended.
  The person's hook is never ended:
  every Git the wrapper starts carries the hooks-off switches,
  so none runs below a
  step.
  Where nothing is measured (no `/proc/<pid>/io`) a step is never ended.

### Bulk step bound, 300 seconds

- Where:
  `src/native/child_class.rs:39` (`OBJECT_CHILD_WAIT`),
  same override.
- Value:
  300 seconds for every step not on the quick list (`pack-objects`,
  `index-pack`,
  `add`,
  `reset`,
  `read-tree`,
  `update-index`,
  `ls-files`,
  `diff`,
  `diff-index`,
  `show`,
  `ls-remote`,
  unknown subcommands,
  and the object reader).
- Bounds:
  as in "Quick query step bound,
  60 seconds".
- When hit:
  the same sentence as in that entry,
  with "a bulk step" as the label.
- Fails or waits:
  waits,
  then killed,
  commit fails.
- Citation:
  partial.
  `doc/planning/cli-git-commit-timeouts.md:139` to `148` measured `reset` at 2.9 to 5.8 s,
  `update-index --refresh` at
2.8 s and `ls-files --modified` at 2.8 s on a stat-dirty 20,207-file index,
  warm cache,
  load above 80;
  the comment
  says
  the bound "leaves room";
  no figure derives 300.
- Conflict:
  same as the quick bound.
- Outcome (pick 4,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  as for the quick bound;
  `OBJECT_QUIET_WINDOW` keeps 300 seconds as a labelled unmeasured
  quiet window.
  The largest healthy whole time of a bulk step in the add timing log is 59.9 s for one call of `git status` or three
  calls of
  `git ls-files` (see "Quiet gaps in the existing logs"),
  which is an upper bound of that step's gap and one fifth of
  300.

### First-notice graces of the two classes

- Where:
  `src/native/child_class.rs:42` (`PLUMBING_NOTICE_GRACE` 5 s) and `45` (`OBJECT_NOTICE_GRACE` 10 s);
  `src/native/child_watch.rs:101` caps the grace at the interval and at half the bound.
- Value:
  5 s and 10 s.
- Bounds:
  when the first "still waiting" line appears.
- When hit:
  the "still waiting" line quoted in "Notice intervals and graces".
- Fails or waits:
  neither.
- Citation:
  none (`doc/planning/cli-git-commit-timeouts.md:151` and `156` state them).
- Conflict:
  none found.
- Outcome (pick 4):
  unchanged.
  The graces time a message,
  not an end,
  and no sign of work applies to them.

### Refused-start retry, five seconds

- Where:
  `src/native/start_retry.rs:27` (`START_RETRY_BUDGET`),
  pauses `30` (10 ms first) and `33` (500 ms longest).
- Value:
  pauses double from 10 ms to 500 ms until 5 s have been spent.
- Bounds:
  a process start refused with `EAGAIN` (no processes left).
- When hit (`start_retry.rs:70` to `73`):
  the start error plus ";
  the start was refused {n} times over {n} ms".
- Fails or waits:
  waits up to 5 s,
  then the step fails.
- Citation:
  none.
- Conflict:
  none;
  a process-count cap is on the owner's "outside our control" list,
  so a retry is the allowed last
  resort.
- Outcome (pick 4):
  unchanged.
  No process exists to measure (the start was refused),
  the cause is on the owner's
  "outside our
  control" list,
  and a retry is the allowed last resort;
  the 5 seconds stays uncited and is not a step's time limit.

### Signal guard installation wait

- Where:
  `src/native/commit_preparation.rs:148` (`GUARD_WAIT_LIMIT` 1200) and `151` (`GUARD_WAIT_DELAY` 50 ms).
- Value:
  1200 tries of 50 ms,
  60 seconds.
- Bounds:
  contention between threads of one process while the guard is installed.
- When hit:
  an error from the preparation.
- Fails or waits:
  waits then fails.
- Citation:
  none.
  `doc/planning/cli-git-commit-timeouts.md:182` to `185` says production never waits there ("only library tests do").
- Conflict:
  none found.
- Outcome (pick 4):
  unchanged.
  It is contention between threads of one process,
  production never waits there,
  and
  there is no
  process or thread work to read;
  it stays uncited.

## Moved-branch, landing and push-contention loops

### Landing rebuilds, 3

- Where:
  `src/native/service_land.rs:31` (`LANDING_ATTEMPTS`),
  loop `220` to `402`.
- Value: 3.
- Bounds:
  how many times the service builds a batch again when the branch moved under it.
- When hit:
  `BranchKeepsChanging`,
  rendered at `src/native/service_messages.rs:159`:
  "The branch {branch} changed
  {times} times while this commit was being prepared,
  so it was not made.
  Run the commit again;
  if other people commit
  to the same branch at the same moment,
  it will succeed once they pause."
- Fails or waits:
  fails after the third round.
- Citation:
  none.
- Conflict,
  yes:
  the cause (other commits moving the branch) is not on the "outside" list;
  the owner chose "Stack on
  the
  new head (Recommended)" for amends,
  conclusions and reverts;
  and "Commits should never fail".
  Covered in part by "Ship interim now" (a bounded moved-head loop),
  but not the number.

### Client rebuilds, 5

- Where:
  `src/native/service_commit.rs:60` (`REBUILD_ATTEMPTS`),
  uses `204`,
  `461`,
  `481`,
  `510`.
- Value:
  5 ("A build is one signature,
  so the bound keeps a busy branch from keeping a person's command running").
- Bounds:
  how many times Git builds the commit again on a new tip.
- When hit:
  the same `BranchKeepsChanging` message,
  with 5,
  or `BranchMovedMessageSaved` when a typed message was
  saved
  (`service_messages.rs:169`).
- Fails or waits:
  fails.
- Citation:
  none (a rationale for the bound,
  not for 5).
- Conflict,
  yes:
  as in "Landing rebuilds,
  3";
  also the signer is on the "outside" list,
  which is the only part of this
  loop that the
  owner's
  rules allow to cost a retry.

### Merge rounds, 20

- Where:
  `src/native/merge_isolation.rs:136` (`MAX_ROUNDS`),
  loop `387`,
  `README.md:922` and `940`.
- Value: 20.
- Bounds:
  whole merges recomputed on the newest head after a branch move or a taken index lock.
- When hit (`merge_isolation.rs:399`):
  "cli-git:
  git merge:
  the branch moved or the index lock was taken in 20 merges
  in a
  row,
  so the wrapper stopped;
  nothing was changed.
  Run the merge again."
- Fails or waits:
  fails with exit 2.
- Citation:
  none (the comment says "a loop,
  not a race").
- Conflict,
  yes:
  "Manual merge should never fail too."
  and other programs' locks are "designed out".

### Reconcile attempts, 10

- Where:
  `src/native/auto_push_reconcile.rs:41` (`MAX_RECONCILE_ATTEMPTS`),
  loop `548`,
  `SPEC.md:1461`.
- Value: 10.
- Bounds:
  fetch,
  merge and push cycles after the remote refuses a push as stale.
- When hit (`auto_push_reconcile.rs:690`):
  "{remote}/{branch} advanced again during each of 10 attempts,
  so nothing
  was
  pushed and the worktree was not touched."
  with the prefix "cli-git:
  auto-push:
  not published:
  ".
- Fails or waits:
  the push fails;
  the service plans a new try with growing pauses.
- Citation:
  none (`SPEC.md:1461` gives the reason,
  "sibling worktrees that keep pushing are contention").
- Conflict:
  partial.
  The cause is the remote advancing or siblings pushing,
  which is not on the "outside" list,
  and
  "Auto merging should never fail."

### Reference-lock push retries, 3

- Where:
  `src/native/auto_push.rs:46` (`PUSH_LOCK_RETRIES`) and `49` (`PUSH_LOCK_PAUSE` 250 ms times the try number),
  loop `242` to `246`,
  `SPEC.md:1577`.
- Value:
  three more pushes after 250,
  500 and 750 ms.
- Bounds:
  a push that delivered but could not update the tracking ref because another service held its lock.
- When hit:
  the push's own output,
  including "cannot lock ref",
  and the note
  "cli-git:
  commit saved locally,
  but auto-push to origin failed;
  run `git push` when ready."
  only when exit is not 0.
- Fails or waits:
  waits under two seconds;
  the tracking ref may stay stale.
- Citation:
  none.
- Conflict,
  yes:
  the lock is another program's (a sibling worktree's service),
  a cause "designed out" by the owner,
  and
  this is a retry.

### Claim take rounds, 64

- Where:
  `src/native/single_owner_claim.rs:44` (`MOST_ROUNDS`).
- Value: 64.
- Bounds:
  rounds of one `take_claim` while other claimants keep arriving.
- When hit:
  the claim reports that claimants arrive faster than it can look.
- Fails or waits:
  fails to take.
- Citation:
  none.
- Conflict:
  none found.

### Work item tries, 3

- Where:
  `src/native/service_work.rs:46` (`MAX_TRIES`),
  used at `378` and `420`;
  message at `187`.
- Value: 3.
- Bounds:
  how many times an item whose holder stopped renewing its claim is run again before it is given up.
- When hit:
  "the commit service stopped 3 times before it finished pushing {branch} (a restart,
  a kill or a crash each
  time),
  so it g[ave up]" (`service_work.rs:187`).
- Fails or waits:
  the item is failed and a notice is left.
- Citation:
  none.
- Conflict:
  none;
  the cause is a kill or crash,
  which is on the "outside" list.

## Commit service client and server

### Service start wait, 10 seconds

- Outcome (delegated pick 6,
  option A;
  open to the owner's veto):
  where the client can ask the service manager (the route has the session bus and the worktree's instance),
  the wait has no clock:
  every second the client asks for the state of `cli-git@<instance>.service`,
  and two answers of `failed` in a row end the wait
  with the existing "The commit service did not start" message and the journal command.
  Where the manager cannot be asked (no bus,
  or a bus call that fails),
  the wait ends at `START_WAIT` as before.
  The wait for an older service to retire keeps `START_WAIT` as well.
- Citation of the remaining 10 seconds:
  ten times the slowest of 52 cold starts in `.git/cli-git-commit-timing.jsonl`
  (`startWaitMs` longest 1041 ms,
  90th percentile 213 ms,
  median 47 ms);
  the 70 commits that asked an older service to retire
  all reached the service within the same 1041 ms (`reachMs`).
- Follow-up outcome (coordinator's pick under the delegation,
  2026-11-03;
  open to the owner's veto):
  a unit the manager reports `inactive` once `START_WAIT` has passed,
  while the socket holds the client's connection and nothing
  starts the service,
  is asked to start through the manager (the same watch that reads `ActiveState`,
  now a `state` function
  and a `start` function),
  again after each further `START_WAIT`,
  and the client goes on waiting.
  A unit that is `activating`
  is left alone.
  No new number:
  the interval is the existing `START_WAIT`.


- Superseded 2026-11-03 by the owner's rule "nothing should ever wait forever or indefinitely" and his answers of
  about 12:45-05:00 (see the "Every wait has a bound" section):
  a start the manager reports as under way is waited for until `START_CEILING`,
  60 seconds (58 times the slowest observed
  start,
  1.04 s),
  then "The commit service did not start".

### Commit queue wait, 10 minutes

- Outcome (delegated pick 6,
  option A;
  open to the owner's veto):
  `COMMIT_WAIT` and its message "raise the waiting limit in the settings for the wrapper" (`WaitedTooLong`) are gone.
  The service sends a `heartbeat` line once a second (`HEARTBEAT_EVERY`,
  `service_protocol.rs`)
  to every client whose request is
  queued or landing,
  from the thread that read the request (`service_reply.rs`);
  the protocol version is 6.
  The client waits for the answer for as long as it keeps hearing from the service.
  A service silent for `HEARTBEAT_SILENCE` (5 seconds) is taken for stopped:
  the client prints "The commit service stopped
  answering;
  asking it again with the same commit,
  which cannot land twice.",
  connects again and sends the same request
  identity.
  A service that greets nothing then ends the command with `ServiceSilent`:
  "The commit service stopped answering
  while this commit waited,
  and it did not answer when asked again,
  so it is not known whether the commit was made."
  followed by
  `git log -1`,
  `git status`,
  the journal command and the restart command.
- Citation:
  queue waits the journal shows reach 89.9 s (`queued_ms`,
  1,332 requests,
  99th percentile 0 ms),
  so no fixed client
  bound near the old 600 s is supported either way;
  the beat replaces it.
  `HEARTBEAT_EVERY` is the pace of the other waiting notices;
  300 one-second timers on this host fired a median of 1.0 ms and
  at most 2.2 ms late,
  so five missed beats (`HEARTBEAT_SILENCE`) are a stopped process and not a late timer,
  and a wrong call
  costs one resend of a request the service recognizes by identity.
- Limit of the beat:
  it is sent by the thread that read the request,
  so it proves the process and the connection live,
  not that
  the landing thread makes progress;
  a stuck Git child is ended by the step bounds,
  not by this.

- Superseded 2026-11-03 by the owner's rule "nothing should ever wait forever or indefinitely" and his answers of
  about 12:45-05:00 (see the "Every wait has a bound" section):
  the client's wait for the answer ends at `ANSWER_CEILING`,
  20 minutes (3.9 times the longest observed answer wait,
  308 s;
  1.6 times the longest whole commit,
  751 s).
  It withdraws the request (protocol 7,
  `withdraw`/`withdrawn`):
  the service answers
  between two landings,
  so the commit either landed (success) or never will;
  the error says it will not land by itself.

### Reconnects, 5

- Outcome:
  unchanged.
  A connection that closes before an answer (the service died)
  is sent the same request again up to five
  times;
  a silence is asked again once per try as well,
  and the reach after a silence is bound by the same silence.
  Still uncited.

### Request read timeout and accept backoff

- Outcome (delegated pick 6):
  `REQUEST_READ_TIMEOUT` (10 s) is unchanged and now cited:
  the request line of 1,332 requests took
  at most 99 ms to arrive (`read_ms`;
  median 0,
  99th percentile 0),
  so 10 s is about a hundred times the slowest.
  `ACCEPT_BACKOFF` is unchanged and uncited.

### Idle exit, three hours

- Where:
  `src/native/service_server.rs:48` (`IDLE_EXIT`),
  argument `--idle-seconds` (`service_main.rs:61` to `63`),
  `README.md:1989`.
- Value:
  3 hours without work.
- Bounds:
  how long the service stays resident.
- When hit:
  the service exits;
  the socket stays and the next connection starts it again.
- Fails or waits:
  the next commit after an idle exit pays a service start (up to the start wait).
- Citation:
  none.
  The comment cites Bazel's `--max_idle_secs` default as "this value's precedent"
  (`service_server.rs:47`),
  and `doc/planning/cli-git-resident-service.md:862` repeats it;
  a precedent is not a
  measurement or an owner decision.
- Conflict:
  none found;
  the cost of the first commit after idle is not measured against "should not take 5s longer".

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  value
  kept.
  Measured cost of exceeding it:
  the 10 commits that followed an idle gap of more than three hours reached the service
  in at most 70 ms after a start wait of at most 47 ms;
  the 28 running services hold 212 MiB together,
  7.9 MiB each
  ("Measurement record",
  service housekeeping).
  The data would also allow a much shorter idle exit at a cost of tens of milliseconds per cold start;
  that is a
  trade between memory and a few milliseconds that only the owner can value,
  so it is listed as an open question.

### Batch size, 64

- Where:
  `src/native/service_server.rs:41` (`MAX_BATCH`),
  loop `618`.
- Value:
  64 requests landed as one batch.
- Bounds:
  group commit size.
- When hit:
  the rest wait for the next batch.
- Fails or waits:
  waits one landing.
- Citation:
  partial.
  `doc/planning/cli-git-resident-service.md:274` to `276` measured,
  in a prototype,
  batches that
"grow from 1 under light load to 21 to 64 in a burst,
  the section stays 49 to 68 ms";
  the largest observed batch
  equals
  the cap,
  so the measurement does not show where the cap should be.
- Conflict:
  none found.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  the data
  cannot support a value.
  Of 1,302 answered requests in the service journal every one was a batch of one with position 1,
  so no observation of
  this build reaches the cap or shows where it should be.
  The prototype's burst measurement stays the only basis.
  The comment at `src/native/service_server.rs:41` says so;
  the options are in the question "Sizes and ages of the
  service's own housekeeping".

- After the follow-up pick (option B,
  picked by the coordinator under the owner's delegation,
  open to the owner's
  veto):
  measured and changed to 256 (`BATCH_COST_KNEE`).
  A burst of simultaneous requests against a throwaway service landed in 71 to 76 ms for 62 to 63 requests,
  82 to 85
  ms for
  126,
  104 to 109 ms for 253 to 255,
  193 to 204 ms for 509 to 511 and 525 to 547 ms for 1,021 to 1,023
  ("Measurement record",
  batch burst).
  The added cost of a request is 0.16 ms up to 254 requests,
  0.36 ms to 510 and 0.66 ms to 1,022,
  so the cost per
  request had not doubled at 256,
  and a burst of 128 that waited 243 to 249 ms in three landings at the old cap of 64
  waited 176 to 206 ms in one.
  Production never reached a batch of more than one request,
  so the cap bites only in a burst.

### Frame and change bounds

- Where:
  `src/native/service_protocol.rs:27` (`MAX_LINE_BYTES` 1 MiB;
  checks at `397` and `572`,
  read bound at
`service_server.rs:268`) and `30` (`MAX_CHANGES` 4096;
  `service_protocol.rs:445`,
  `src/native/service_prepare.rs:161`).
- Value:
  a line over 1 MiB is refused;
  a commit that changes more than 4096 paths is not handed to the service.
- Bounds:
  request size.
- When hit:
  for more than 4096 changes `service_prepare.rs:161` returns `None` and the commit takes the native path
  instead of the service (the code path returns "no changes to send");
  a longer line is a decode failure.
- Fails or waits:
  falls back or refuses;
  I did not read the long-line failure text.
- Citation:
  none.
- Conflict:
  the fallback is the opposite of "Invoke real git as rarely as possible" (handover owner quote),
  but only
  for
  commits above the bound.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  values kept.
  Of 21,696 non-merge commits of this repository since 2026-09-01 the median changed 2 paths,
  the 99th percentile 38,
  the 99.9th percentile 377,
  and 10 (0.05 percent) changed more than 4,096;
  those commits take the native path.
  The 99th percentile path is 118 characters,
  and a request of 4,096 changes at that length fits the one mebibyte line
  (a control in `src/native/service_protocol_tests.rs` encodes it).
  A request of 4,096 paths of the longest length seen (189 characters) would need about 1.06 MB and is refused as a
  decode failure;
  that case needs a commit of more than 4,000 paths of unusual length.

### Claim lease and renewal

- Where:
  `src/native/service_work.rs:335` (`CLAIM_LEASE` 120 s),
  `src/native/service_claim_renewal.rs:34`
  (`RENEWALS_PER_LEASE` 3) and `39` (`LOOKS` 3),
  `src/native/service_retry.rs:18` (`RETRY_SLACK` 1 s),
  `src/native/worktree_removal_rescue.rs:53` (`LOOK_ROUNDS` 2).
- Value:
  a claim not renewed for 120 s is taken back;
  the holder renews every 40 s.
- Bounds:
  how long work claimed by a killed service waits before another instance runs it.
- When hit:
  no message;
  the item runs again,
  or after `MAX_TRIES` is failed with the notice quoted in "Work item
  tries,
  3".
- Fails or waits:
  waits up to the lease after a kill.
- Citation:
  none (`SPEC.md:3346` onward explains why a lease exists,
  not why 120 s).
- Conflict:
  none;
  a kill is "outside our control".

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  the data
  cannot support a value.
  The user journal of this host (19 boots) holds no kill,
  out-of-memory stop or failed result of a `cli-git@` unit,
  so
  the cost of waiting out a lease after a kill was never observed.
  The renewal runs on its own thread,
  so a landing that stalled for 119.8 s (the longest `total_ms` in the service
  journal) does not stop renewals.
  The comment at `src/native/service_work.rs` says so;
  the options are in the question "Sizes and ages of the
  service's own housekeeping".

- After the follow-up pick (option B,
  picked by the coordinator under the owner's delegation,
  open to the owner's
  veto):
  measured,
  value kept.
  A throwaway service that held a claim on a push was killed with SIGKILL,
  and the push was made again 119.57,
  119.56
  and
  119.55 s after the kill in three runs ("Measurement record",
  claim recovery),
  so the cost of a kill is the lease to
  within
  half a second.
  The other side of the trade,
  a live holder losing its claim,
  needs a stall of the whole process longer than the 80 s a
  renewal may be late;
  the longest landing stall seen was 119.8 s but the renewal thread is separate,
  and a stall of the
  process itself was not observed or produced.
  The value is kept at 120 s with both facts beside it (`src/native/service_work.rs`).

### Journal retention and capture memory

- Where:
  `src/native/service_journal.rs:258` (`DONE_RETAINED` 1024),
  `src/native/service_captures.rs:25` (`REMEMBERED`
  128).
- Value:
  1024 finished batches;
  128 landed captures per worktree.
- Bounds:
  how far back a resent request is recognized as already landed;
  how many landed commits are remembered.
- When hit:
  an older resend or capture is unknown (`REMEMBERED`:
  "older ones are forgotten,
  which makes them
  unknown").
- Fails or waits:
  neither;
  may re-land.
- Citation:
  none.
  The 1024 comment derives from `RECONNECTS` and `START_WAIT`,
  which are themselves uncited.
- Conflict:
  none found.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  values kept.
  In the service journal the busiest `cli-git@` unit landed 4 batches in its busiest 10 seconds,
  4 in its busiest
  minute and 15 in its busiest ten minutes,
  and 349 in the whole window,
  so 1024 finished batches reach back over
  sixty times the busiest ten minutes and 128 remembered captures cover eight times it.
  The derivation in the old comment from `RECONNECTS` and `START_WAIT` stays;
  those two belong to the pick on service
  waits.

### Stale temporary and scratch ages

- Where:
  `src/native/service_journal.rs:33` (`TEMPORARY_GRACE` 10 min),
  `src/native/work_item_recovery.rs:29`
  (`STRAY_TEMPORARY_AGE` 24 h),
  `src/native/scratch_location.rs:46` (`SCRATCH_STALE_AFTER` 24 h).
- Value:
  as listed.
- Bounds:
  the age before a recovery or preparation deletes a leftover temporary or scratch file.
- When hit:
  the file is deleted.
- Fails or waits:
  neither.
- Citation:
  none (`scratch_location.rs:44` to `45` says "far longer than any command of the wrapper runs (its longest
  bound is minutes)",
  but the live-lock wait is 30 minutes and the foreign-owner patience an hour).
- Conflict:
  none found,
  since scratch is not user data.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  values kept.
  The longest journal write step of 1,302 landings was 40,188 ms,
  so the ten minute grace is fifteen times it.
  The longest command in the commit timing log took 750.6 s and the longest worktree add 539.7 s,
  so a day is about a
  hundred times the longest command;
  the 1,800 s lock wait and the 3,600 s patience are longer than the longest command
  measured and still far under a day.
- Follow-up outcome (coordinator's pick under the delegation,
  2026-11-03;
  open to the owner's veto):
  the live-lock wait no longer
  has a bound,
  so a day is no longer above every wait.
  A command that waits now renews the modification time of its scratch
  directory (`scratch_location::renew`,
  every `SCRATCH_REFRESH_EVERY`,
  a twenty-fourth of `SCRATCH_STALE_AFTER`,
  at each message
  from the service),
  so the sweep,
  which removes entries by age,
  never removes the directory of a command that is still
  waiting;
  a directory whose command died stops being renewed and ages out as before.
  Scratch files with no name are not swept.
  The waiter's own liveness is what keeps the directory,
  with no lock and no record of the process.
  The renewal covers the commit
  wait on the service;
  other waits hold only unnamed files.

### Earlier-service retirement wait

- Where:
  `src/native/service_provision.rs:86` (`RETIREMENT_WAIT` 10 s) and `367` (`RETIREMENT_POLL` 50 ms).
- Value:
  10 seconds.
- Bounds:
  how long the earlier single service may take to stop.
- When hit (`service_provision.rs:339`):
  "the earlier commit service ({unit}) is still stopping after {n} seconds;
  run
  the
  commit again in a moment,
  or stop it with `systemctl --user stop ...`".
- Fails or waits:
  waits,
  then the commit fails.
- Citation:
  none.
- Conflict,
  yes:
  `DESIGN.md:124` (a version mismatch resolved without the user).

### Service manager exchange

- Where:
  `src/native/systemd_dbus.rs:32` (`EXCHANGE_WAIT` 20 s) and `42` (`MAX_REPLY_BYTES` 1 MiB).
- Value:
  20 seconds for the whole connect,
  authenticate,
  hello and calls;
  1 MiB per reply.
- Bounds:
  one exchange with the user service manager.
- When hit:
  a timeout error rendered as a setup failure ("The commit service could not be set up ...",
  `service_messages.rs` `CouldNotStart`).
- Fails or waits:
  waits,
  then the commit fails.
- Citation:
  none.
- Conflict,
  yes:
  `DESIGN.md:124` for a manager that is only slow.

## Pushes

### Push stall window, 30 seconds

- Where:
  `src/native/bounded_push.rs:49` (`DEFAULT_PUSH_DEADLINE`),
  variable `CLI_GIT_PUSH_DEADLINE_SECONDS` (`45`),
  `SPEC.md:1511`.
- Value:
  30 seconds with no sign of work (bytes moved,
  CPU used,
  a task running or in disk wait),
  sampled every 500 ms
  (`src/native/push_progress.rs:44`).
  Where `/proc` cannot be read it is a fixed deadline for the whole push.
- Bounds:
  the auto-push and the reconcile fetch (`src/native/lfs_bounded_push.rs:92`,
  `src/native/auto_push_reconcile.rs:155`).
- When hit (`bounded_push.rs:555`):
  "cli-git:
  git push made no progress for {n} seconds and was stopped (limit=stall,
  bytes_moved={n},
  elapsed_seconds={n});
  {extend clause}.
  Hint:
  if you were pushing over a slow or offline connection,
  your commit is already saved;
  push again when the connection works."
  Exit code 124.
- Fails or waits:
  waits while progress shows,
  then the push is stopped;
  the service plans a new try
  (`src/native/push_retry.rs`).
- Citation:
  partial.
`SPEC.md:1492` to `1510` measured on 2026-11-03 which signs show a push at work (a 30 MB push to a local bare remote;
  `rchar` and `wchar`,
  processor ticks,
  task state),
  and the owner chose "Stop only on no progress (Recommended)".
  Neither decides 30 seconds.
- Conflict,
  yes,
  on main:
  "Wait while connected (Recommended)" (bytes on the socket count;
  keep waiting while
  connected).
  `SPEC.md:1505` to `1509` itself says a server that checks the push before answering "is stopped" after the window.
  A slow remote is on the owner's "designed out" list.
  The unmerged branch changes the progress rule and keeps 30.
- Outcome (pick 5,
  the coordinator's under the owner's delegation "Pick options as if I had picked them and invite veto
  when I say I'm back",
  open to the owner's veto):
  the pack-sent test is gone,
  and an open connection (a process of the tree holds a socket in state `ESTABLISHED`,
  read from the kernel's socket diagnostics) is waited for in every phase.
  The window now applies only to a tree with no open connection and no sign of work.
  It covers the push,
  the reconcile fetch and the LFS upload,
  and the wait line names the program that waits.
  That scope is the building agent's choice,
  flagged for the coordinator:
  the owner's answer named the push.
  The 30 seconds stays uncited,
  and `bounded_push.rs` and `SPEC.md` say so beside it.
  The journal of the commit service (read-only,
  2026-10-20 to 2026-11-03,
  794 pushes that landed) gives whole-push times
  of p50 3.5 s,
  p99 26.4 s and maximum 72.4 s,
  which include connected time.
  No silence of a disconnected push is logged,
  so nothing determines the number.
  Open question:
  "The stall window of a push with no connection,
  and a connection that holds and never answers",
  in "Questions raised while building picks 5 and 12".

- Superseded 2026-11-03 by the owner's rule "nothing should ever wait forever or indefinitely" and his answers of
  about 12:45-05:00 (see the "Every wait has a bound" section):
  a push whose connection is open is ended when it shows no sign of work for `CONNECTED_PUSH_QUIET_WINDOW`,
  150 seconds,
  unmeasured (twice the longest whole push,
  72.4 s);
  the service's retry plan keeps it ("Bound it,
  keep retrying").

### Push ceiling, 1800 seconds

- Where:
  `src/native/bounded_push.rs:55` (`DEFAULT_PUSH_CEILING`),
  variable `CLI_GIT_PUSH_CEILING_SECONDS` (`52`),
  `SPEC.md:1515`.
- Value:
  30 minutes in all,
  however much the push moves.
- Bounds:
  the same pushes.
- When hit (`bounded_push.rs:564`):
  "cli-git:
  git push was still running after {n} seconds,
  though it kept moving
  data,
  and
  was stopped (limit=ceiling,
  ...);
  ... Hint:
  your commit is already saved;
  push again when the connection is faster."
- Fails or waits:
  waits,
  then stops;
  the service retries.
- Citation:
  none (`SPEC.md:1515` to `1517`:
  "so a trickle that never stalls cannot hold the service for good").
- Conflict,
  yes:
  "Wait while connected ... once the pack is sent,
  keep waiting while the connection is open."
- Outcome (pick 5,
  delegated,
  open to the owner's veto):
  removed.
  `DEFAULT_PUSH_CEILING`,
  `CLI_GIT_PUSH_CEILING_SECONDS`,
  its advice,
  `StopCause::Ceiling` and `limit=ceiling` are gone.
  Cost,
  recorded in `SPEC.md`:
  a process that spins or sits in state `D` for good shows a sign of work for ever
  and holds the service's push of its repository.

### Git low-speed abort, 1000 bytes per second for 60 seconds

- Where:
  `src/native/bounded_push.rs:69` and `72` (`LOW_SPEED_LIMIT`,
  `LOW_SPEED_TIME`),
  applied at
  `bounded_push.rs:216` to `222` from `src/native/lfs_bounded_push.rs:91` (the push) and
  `src/native/auto_push_reconcile.rs:154` (the fetch);
  `SPEC.md:1565`.
- Value:
  `GIT_HTTP_LOW_SPEED_LIMIT=1000`,
  `GIT_HTTP_LOW_SPEED_TIME=60`,
  set only when the invocation sets neither.
- Bounds:
  Git's own HTTP transfer,
  via curl.
- When hit:
  curl's "Operation too slow",
  as the handover records (line 342).
- Fails or waits:
  the transfer fails.
- Citation:
  none.
- Conflict,
  yes on main:
  "Wait while connected".
  The handover says the push no longer sets it (branch `fb1e093809`);
  main still does.
- Outcome (pick 5,
  delegated,
  open to the owner's veto):
  removed for every run the wrapper starts.
  The push stopped setting it in `fb1e093809`.
  The reconcile fetch stopped here (`apply_low_speed_defaults`,
  `LOW_SPEED_LIMIT` and `LOW_SPEED_TIME` are gone),
  shown by `the_reconcile_fetch_sets_no_low_speed_abort`.
  A value the person sets in the environment or in `http.lowSpeedLimit` and `http.lowSpeedTime` still reaches Git.

### Push output cap, 4 MiB

Status after picks 5 and 12:
unchanged.
`bounded_push.rs` and `SPEC.md` now say beside the value that no citation
exists.

- Where:
  `src/native/bounded_push.rs:62` (`MAX_PUSH_OUTPUT`),
  `is_flood` at `277`,
  `SPEC.md:1562`.
- Value:
  4 MiB;
  only the first 4 MiB are kept.
- Bounds:
  the captured output of a push.
- When hit (`bounded_push.rs:572`):
  "cli-git:
  git push wrote more than {n} bytes of output and was stopped.
  Hint:
  ...".
- Fails or waits:
  the push is stopped.
- Citation:
  none.
- Conflict:
  none found.

### Stop grace and polls

- Where:
  `src/native/bounded_push.rs:58` (`STOP_GRACE` 5 s),
  `75` (`POLL_PAUSE` 10 ms);
  `src/native/child_watch.rs:123` (`STOP_POLL` 10 ms);
  `src/native/package_repair_command.rs:81` (10 ms).
- Value:
  a catchable stop,
  five seconds,
  then the kill;
  polls every 10 ms.
- Bounds:
  how long a stopped child gets to clean up its own lock files.
- When hit:
  the kill,
  then the lock sweep (`SPEC.md:1526` to `1560`).
- Fails or waits:
  waits up to five seconds.
- Citation:
  none (`SPEC.md:1526` states it).
- Conflict:
  none found.

### Read-only query ceiling, 120 seconds

- Where:
  `src/native/bounded_query.rs:35` (`QUERY_CEILING`),
  deadline `65` (the shorter of the push deadline and the
  ceiling),
  `SPEC.md:1574`.
- Value:
  120 seconds,
  or the push stall window when that is shorter (so 30 s by default).
- Bounds:
  `config`,
  `for-each-ref`,
  `rev-parse`,
  `remote get-url`,
  `merge-base` before a push.
- When hit (`src/native/auto_push_single_flight.rs:74`):
  "git {args} did not finish within {n} seconds and was
  stopped;
  {clause}".
- Fails or waits:
  waits then the push attempt fails;
  the service retries.
- Citation:
  none ("queries read local repository state and finish in milliseconds",
  no figure).
- Conflict:
  yes in part,
  by the same reasoning as the child bounds.
- Outcome (picks 4 and 9,
  scope added by the coordinator,
  the coordinator's under the owner's delegation,
  open to the
  owner's veto):
  `QUERY_CEILING` is removed.
  A read-only query is judged by the push meter with the push's stall window
  (`CLI_GIT_PUSH_DEADLINE_SECONDS`,
  30 seconds,
  uncited and unchanged by this pick),
  has no wall-clock stop,
  and is
  never
  judged where nothing is measured.
  The journal's `git status` stopped at 2026-11-03T08:28 with `outcome=retrying
  push_ms=97398` is the case it ends:
  it stopped for being slow and now stops only for being quiet.
  The failure reads "git {args} showed no sign of work for {n} seconds and was stopped",
  and the query's longest quiet
  gap
  is kept in the ledger.

### Push retry pauses, 10 seconds to 10 minutes

- Where:
  `src/native/push_retry.rs:35` (`FIRST_PAUSE`) and `38` (`LONGEST_PAUSE`),
  variable
  `CLI_GIT_PUSH_RETRY_SECONDS` (`28`),
  `SPEC.md:1268`,
  `README.md:1275`.
- Value:
  the pause doubles from 10 s to 600 s and stays there;
  there is no count limit.
- Bounds:
  the service's own new tries of a push that was stopped or failed for a reason that can pass.
- When hit (`src/native/service_work.rs:256`):
  "cli-git:
  the commit service tries this push again by itself in {n}
  seconds (try {n}),
  and keeps trying with longer pauses until the push lands or a conflict needs you."
- Fails or waits:
  waits;
  never gives up while the cause can pass.
- Citation:
  none for the values.
The owner decision "Offline is retried,
  but auth and server hook declines stop until your next commit."
  decides which
  causes retry,
  not the pauses.
- Conflict:
  partial.
  The retried causes include a held lock and a stall ("Being offline,
  a stall,
  the ceiling and a
  held
  lock are tried again",
  `SPEC.md:1272`);
  a held lock is another program's and a slow remote is designed out.

### LFS server-error retry, 2 and 5 seconds

- Where:
  `src/native/lfs_server_retry.rs:23` (`RETRY_DELAYS`),
  `30` (`MAX_CONFIGURED_RETRIES` 5),
  `33`
  (`MAX_CONFIGURED_DELAY` 60 s);
  variable `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS`;
  `SPEC.md:1974`.
- Value:
  two retries,
  after 2 s and after 5 s;
  a configured list is clipped to five delays of at most 60 s.
- Bounds:
  `git-lfs push` after an HTTP 5xx answer.
- When hit (`lfs_server_retry.rs:167`):
  "cli-git:
  `git-lfs push` failed with a server error,
  so it is run again in {n}
  seconds (retry {n} of {total});
  objects the server already has are not sent twice."
- Fails or waits:
  waits seven seconds in all,
  then the push fails.
- Citation:
  none.
- Conflict:
  unclear.
  An HTTP 5xx is a remote answering,
  which the owner's two lists do not name;
  "the remote refusing"
  lists auth,
  a server hook,
  GH009 and branch protection,
  and "a slow remote" is designed out.
  The classification is a question for the owner.
- Outcome (pick 12,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  an HTTP 5xx from the LFS server is classed as outside our control,
  a server fault like "Network down",
  and is retried by the service's push retry plan alone.
  `lfs_server_retry.rs`,
  `RETRY_DELAYS`,
  `MAX_CONFIGURED_RETRIES`,
  `MAX_CONFIGURED_DELAY`,
  the variable `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS` and both loops of `lfs_upload.rs` are removed.
  The plan reads nothing in a 5xx as final
  (`an_lfs_server_error_is_tried_again_by_the_plan`,
  shown red by classing a 5xx as final),
  the auto-push is left to the plan (`an_auto_push_that_meets_a_server_error_is_left_to_the_services_retry_plan`),
  and a typed push ends at the first 5xx (`a_server_error_ends_a_typed_push_at_once`).
  Visible change for a typed `git push`:
  no "retry 1 of 2" lines,
  and a brief outage now fails the push
  where two in-place retries used to ride it out.

### LFS chunking, relay and pointer scan

- Where:
  `src/native/lfs_upload_plan.rs:57` (`OBJECTS_PER_RUN` 500),
  `src/native/lfs_upload.rs:123`
(`MAX_RELAYED_BYTES` 4 MiB),
  `src/native/lfs_pointer_scan.rs:28` (`MAX_COMMITS` 100,000) and `31` (`SCAN_BUDGET` 3 s),
  `src/native/lfs_pointer.rs:18` (`BLOB_SIZE_CUTOFF` 1024).
- Value:
  as listed.
- Bounds:
  object names per `git-lfs push` run;
  relayed standard error;
  the in-process scan for pointers.
- When hit:
  more runs;
  text dropped;
  the scan "gives the question to `git-lfs`".
- Fails or waits:
  slows,
  never fails.
- Citation:
  none (`BLOB_SIZE_CUTOFF` mirrors the LFS pointer-size rule,
  with no source line beside it).
- Conflict:
  none found.

### Auto-push walk budget, 4096 commits

- Where:
  `src/native/auto_push_trigger.rs:42` (`WALK_BUDGET`),
  `SPEC.md:1295`.
- Value:
  4096 commits per ancestry question.
- Bounds:
  the check that decides whether a merge or similar command hands a push to the service.
- When hit:
  "a walk that cannot tell hands nothing over" (`SPEC.md:1295`);
  no push is handed over by that command.
- Fails or waits:
  neither;
  the push is not made until a later trigger.
- Citation:
  none.
- Conflict:
  possible with "The owner's rule is that auto-push must land" (`src/native/push_retry.rs:5`):
  a branch far
  ahead is not handed over at all.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  value
  kept.
  The 439 most recent merges on `main` of this repository brought in a median of 8 commits (90th percentile 50,
  99th
  percentile 317,
  largest 6,542);
  only 1 of 439 exceeded 4,096.
  The comment is at `src/native/auto_push_trigger.rs:42`.

### Concurrent per-update queries, 4

- Where:
  `src/native/manual_push_probe.rs:133` (`MOST_CONCURRENT_QUERIES`),
  `SPEC.md:1619` and `1624`.
- Value:
  four Git queries at once for a manual push.
- Bounds:
  parallelism,
  and the connections opened to remotes.
- When hit:
  queries queue.
- Fails or waits:
  waits.
- Citation:
  none;
  the comment says "the bound `SPEC.md` gives",
  and `SPEC.md` gives it without a measurement or an
  owner quote.
- Conflict,
  yes in part:
  "make sure we're running everything that can be run in parallel in parallel."

## Policies, reviews and repair tools

### Markdown linter deadline, 60 seconds

- Where:
  `src/native/markdown_linter.rs:64` (`LINT_DEADLINE`),
  guard `src/native/markdown_linter_guard.rs`.
- Value:
  60 seconds per candidate.
- Bounds:
  one run of the in-process Markdown linter.
- When hit (`markdown_linter.rs:184`):
  "monochromatic-lint did not finish within 60000 milliseconds and cli-git
  stopped
  waiting for it (its thread stays in this process until the command ends);
  run the command again,
  run
  monochromatic-lint by hand on the file to see why it stalls,
  or set the policy markdown/autofix to off to skip the
  check",
  after a notice "still waiting for {file} after {n} seconds;
  giving up on it after {n} seconds"
  (`markdown_linter_guard.rs:104`).
- Fails or waits:
  waits then the policy check fails.
- Citation:
  partial.
  `markdown_linter.rs:53` to `58`:
  "Measured on the host with the former child process,
  one run
  takes about 11 milliseconds warm and 227 cold ... the host has run at load averages of 40 to 90.
  A minute leaves room."
  The typical case is measured;
  60 s is a judgement three orders above it.
- Conflict:
  yes in part,
  "Commits should never fail" (a hung linter in this process cannot be killed,
  so the bound is
  the
  only exit).
- Outcome (pick 9,
  option A,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  `LINT_QUIET_WINDOW` (renamed from `LINT_DEADLINE`) keeps 60 seconds as a labelled unmeasured quiet window.
  The linter runs on a thread of the wrapper,
  so the signs are those of that thread from `/proc/self/task/<tid>/`
  (bytes read
  and written,
  processor ticks,
  state),
  in `src/native/thread_quiet.rs`;
  a linter that works is waited for however
  long it
  takes,
  and a thread that shows none is abandoned as before (it cannot be ended from outside,
  and the command starts no
  second linter thread).
  Its longest quiet gap is kept under `monochromatic-lint`.

### Markdown output bounds

- Where:
  `src/native/markdown_linter_output.rs:32` (`EXPLANATION_LIMIT` 400),
  `37` (`MAX_FIXED_BYTES` 64 MiB),
  `42` (`MAX_FINDINGS_BYTES` 4 MiB).
- Value:
  as listed;
  the two byte bounds are "the bound the former child process had" on its output streams.
- When hit (`markdown_linter_output.rs:228`,
  `235`):
  "monochromatic-lint produced a fixed source of {n} bytes,
  over
  the
  67108864 bytes cli-git accepts" or the findings equivalent.
- Fails or waits:
  the check fails.
- Citation:
  none (inherited,
  not measured).
- Conflict:
  none found.

### Policy convergence passes, 8

- Where:
  `src/native/policy_convergence.rs:39` (`MAXIMUM_CHANGED_PASSES`),
  `SPEC.md:2097` and `2108`.
- Value:
  eight changed passes,
  then one more pass.
- Bounds:
  the loop that re-runs policies after a correction changed content.
- When hit:
  a pass-limit failure ("settles when that pass proposes nothing and is a pass-limit failure when it still
  proposes a change").
- Fails or waits:
  fails.
- Citation:
  none ("The incumbent stops after eight changed passes").
- Conflict:
  none found;
  a policy that keeps changing its own output is the wrapper's own defect.

### Standards review gateway call, 10 seconds

- Where:
  `src/native/service_review.rs:28` (`CALL_DEADLINE`).
- Value:
  10 seconds.
- Bounds:
  the service's one call to the review gateway.
- When hit:
  the item is failed and the work thread moves on;
  the failure text goes to the review log.
- Fails or waits:
  the review fails;
  the commit is unaffected.
- Citation:
  cited,
  with a caveat.
  `src/native/service_review.rs:26` to `27` bases it on "several times the largest figure on record (1.4 s in
`doc/audit/standards-review-gate-shadow.md`)";
  that file shows a wall-clock cost of 1.4 s at line 2562 for one logged
  review,
  and I did not check that it is the largest.
- Conflict:
  none found;
  the review does not block a commit.
- Outcome (pick 9,
  option A,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  `CALL_QUIET_WINDOW` (renamed from `CALL_DEADLINE`) keeps 10 seconds as a labelled unmeasured quiet window;
  the
  caveat above
  stands,
  because 1.4 s is a whole time.
  The total deadline is gone.
  The call ends on its transport's own idle timeouts:
  resolving,
  connecting,
  and each read
  and write
  of the connection (TLS handshake,
  request,
  response head and body) may stay silent for the window
  (`timeout_per_read` and
  `timeout_per_write` of `ureq` 3.5.0,
  `Cargo.lock`;
  they restart at each read or write),
  so a reply that keeps
  arriving is read
  whole.
  The call's quiet gap is measured beside it by a sampler of the calling thread and the process's connected
  sockets
  (`thread_quiet.rs`),
  not by the judge,
  and kept under `standards review gateway call`.

### Standards review script and whole-review deadlines

- Where:
  `src/native/standards_review_client.rs:18` (`SCRIPT_DEADLINE` 30 s) and `src/native/service_review.rs:31`
  (`BUDGET` 20 s).
- Value:
  30 seconds for the script's request;
  20 seconds for the whole review,
  Git reads and write included.
- Bounds:
  the review that runs after a commit.
- When hit:
  the item is failed;
  the log says "The operation was aborted due to timeout"
  (`standards_review_client.rs:107`).
- Fails or waits:
  the review fails;
  the commit is unaffected.
- Citation:
  none (30 s is inherited from the old script,
  20 s from "a third of the script's 30 s" argument at
  `service_review.rs:26` to `27`).
- Conflict:
  none found.
- Outcome (pick 9,
  option A,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  `SCRIPT_QUIET_WINDOW` (renamed from `SCRIPT_DEADLINE`) keeps 30 seconds as the default quiet window of the script's
  own
  options,
  labelled unmeasured;
  the service overrides it with `CALL_QUIET_WINDOW`.
  The `BUDGET` of 20 seconds around the whole review is removed:
  a wall clock around steps that each end on no
  progress is still
  a wall-clock bound.
  Its Git reads end under the watch of `child_watch.rs`,
  its call on the idle timeouts,
  and
  `run_within` is
  now `run_contained`,
  which contains a panic and waits.
  What is left with no judge is the local write of the log entry
  (a write blocked in the kernel has no bound);
  flagged below.

### Standards review input caps

- Where:
  `src/native/standards_review_input.rs:54` (`CAP_LINES` 2000),
  `57` (`MAX_EXCERPT_FILES` 8),
  `60`
  (`EXCERPT_CONTEXT` 30),
  `63` (`EXCERPT_LINE_CAP` 4000),
  `69` (`INPUT_UNIT_CAP` 24000);
  `src/native/standards_review_source.rs:22` (`MAX_PATCH_BYTES` 64 MiB,
  "the script's `maxBuffer`").
- Value:
  as listed.
- Bounds:
  what part of a commit the model sees.
- When hit:
  the input is truncated and the log says "capped" (`standards_review_entry.rs:97`).
- Fails or waits:
  neither.
- Citation:
  none (carried from the script).
- Conflict:
  none found.

### Package repair tool limit, 600 seconds

- Where:
  `src/native/package_repair.rs:36` (`TOOL_LIMIT`),
  parse `package_repair_command.rs:176` to `205`.
- Value:
  ten minutes per tool (install,
  rebuild).
- Bounds:
  one package-manager tool the add runs to repair a new worktree.
- When hit (`package_repair_report.rs:92`):
  the finding "it ran past its time limit and was stopped",
  with the command
  to
  run instead.
- Fails or waits:
  the repair is reported failed;
  the worktree add itself succeeds.
- Citation:
  none ("a hung tool must not hold an add forever").
- Conflict:
  possible with "Install online during the add" (`SPEC.md:1105`),
  where an install can download for longer;
  and "60s is fine for adding worktrees",
  which sets no target to stop at (handover line 501).
- Outcome (pick 9,
  option A,
  the coordinator's under the owner's delegation,
  open to the owner's veto):
  `TOOL_QUIET_WINDOW` (renamed from `TOOL_LIMIT`;
  `RepairEnvironment::quiet_window`,
  renamed from `tool_limit`) keeps
  600
  seconds as a labelled unmeasured quiet window.
  A tool is judged on the leader of its process group with the push meter and ends with its whole group;
  an install that
  downloads for a long time is never reported as having run past a limit.
  The finding now reads "it showed no sign of
  work for
  the quiet window and was stopped",
  with the command to run instead.
  The largest whole time of a repair tool in the add
  timing log is 117.7 s (see "Quiet gaps in the existing logs").
  The add timing record carries each tool's longest
  quiet gap
  under `quiet`.
- Amended 2026-11-04T09:35:19-05:00 by the owner's answer "Silence plus a ceiling (Recommended)":
  a tool is also ended with its whole process group at `REPAIR_TOOL_CEILING`,
  18 minutes
  (`src/native/service/wait_ceilings.rs`),
  whatever it shows,
  with a finding that names the tool,
  what it was doing,
  the seconds and
  the command to run.
  The same answer adds `LINT_CEILING` (4 minutes),
  `REVIEW_CALL_CEILING` (1 minute) and `REVIEW_CEILING` (2 minutes) to the Markdown
  linter,
  the standards-review call and the whole review.
  The measurements,
  the margin rule (the smallest whole number of minutes at least five times the longest run) and the limits of the
  sample are in `doc/planning/cli-git-no-unbounded-waits.md`,
  "Branch write and tool ceilings,
  2026-11-04T09:35:19-05:00".
  The quiet windows are unchanged;
  the longest quiet gaps measured (12 add records:
  `cargo clean` 30.3 s,
  `mpm-shadow install`
  7.6 s,
  `cargo metadata` and `pnpm install` 0.5 s;
  116 commit records:
  the Markdown linter 404 ms) are recorded there as the margin
  of each window,
  not as new windows.

### Configuration file size

- Where:
  `src/native/config_file.rs:83` (`MAX_CONFIG_BYTES` 1 MiB),
  `SPEC.md:147`.
- Value:
  1 MiB.
- Bounds:
  `cli-git.config.jsonc`.
- When hit:
  a configuration failure naming the file (exit 2 before any policy runs).
- Fails or waits:
  the command fails.
- Citation:
  none ("A repository file must not control how much memory every Git command reads").
- Conflict:
  none found.

## Worktree add, copy and conversion

### Conversion storage ceiling, 16 workers

- Where:
  `src/native/worktree_convert_command.rs:76` (`CONVERSION_STORAGE_CEILING`),
  `README.md:535`,
  `SPEC.md:694`.
- Value:
  at most 16 conversions at once.
- Bounds:
  parallel conversions of nested worktrees to subvolumes.
- When hit:
  extra conversions wait.
- Fails or waits:
  waits.
- Citation:
  cited.
The comment records a measurement on 2026-11-03 "on a btrfs volume (32 worktrees of about ten thousand files each,
  28
threads,
  other sessions running,
  medians of three or more runs at 1,
  2,
  4,
  8,
  16 and 18 workers:
  61,
  35,
  19,
  14,
  12
  and 12
  seconds ...)" (`worktree_convert_command.rs:76` to `82`).
- Conflict:
  none;
  the cap sits at the measured plateau.

### Conversion memory estimate, 128 MiB

- Where:
  `src/native/worktree_convert_command.rs:62` (`CONVERSION_MEMORY`).
- Value:
  128 MiB per conversion in the budget.
- Bounds:
  how many conversions the free memory admits.
- When hit:
  fewer workers.
- Fails or waits:
  waits.
- Citation:
  cited.
  "measured on 2026-11-03 at 7484 KiB resident for a worktree of 16 thousand files (about 0.45 KiB per file),
  which is
  about 90 MiB at two hundred thousand files" (`worktree_convert_command.rs:66` to `68`),
  rounded up.
  The host and volume are not named beside it,
  only the worktree size.
- Conflict:
  none.

### Parallel checkout setting

- Where:
  `src/native/checkout_workers.rs:24` and `27` (`checkout.workers=0`),
  `SPEC.md:651`.
- Value:
  `-c checkout.workers=0` (one worker per logical CPU) added only where nothing sets the key.
- Bounds:
  Git's checkout parallelism in a new worktree.
- When hit:
  not a cap;
  it removes Git's default of one worker.
- Fails or waits:
  speeds the add.
- Citation:
  cited.
Owner decision "Pass only if unset" (`SPEC.md:651`,
  `src/native/checkout_workers.rs:4` to `5`) and a measurement
  beside it:
  "a checkout of twenty thousand files measured several times faster".
- Conflict:
  none.

### Copy and clone thread caps and thresholds

- Where:
  `src/native/worktree_copy_stage.rs:221` (`PARALLEL_COPY_MIN_ENTRIES` 4096) and `232`
  (`PARALLEL_COPY_MAX_WORKERS` 8),
  `src/native/worktree_copy_install.rs:188` (`PARALLEL_INSTALL_MIN_LEAVES` 1024),
  `src/native/worktree_copy_journal.rs:1018` (`REMOVAL_SPLIT_DEPTH` 3),
  `src/native/package_repair_tree_clone.rs:39` (`PARALLEL_MIN_ENTRIES` 512);
  `doc/planning/cli-git-main-worktree-ignored-copy.md:268` and `275`,
  `SPEC.md:929`.
- Value:
  eight threads at most;
  fewer than 4096 entries stay on one thread;
  removal splits three levels deep on up to
  eight threads.
- Bounds:
  parallelism of the ignored-state copy,
  install and stage removal.
- When hit:
  more work per thread.
- Fails or waits:
  waits.
- Citation:
  none ("Cloning is a metadata operation bound by the filesystem's locks more than by CPU,
  and other
  sessions
  share this host";
  no figure).
- Conflict,
  yes in part:
  "make sure we're running everything that can be run in parallel in parallel",
  on a 28-thread
  host;
  and the owner's normal-here rule on load,
  which says a loaded host does not excuse a cap.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  the cap of
  eight is cited and kept;
  the thresholds are not changed.
  `clone_tree` (the same file-making primitive as the copy stage) took 1,549,
  880,
  555,
  397 and 415 ms at 1,
  2,
  4,
  8
  and 16 workers for 22,718 entries (three runs each;
  "Measurement record",
  parallelism),
  so more than eight workers
  were never faster,
  at 9,301 and at 4,723 entries as well.
  The parallel path won at every size it was allowed to run (555 entries and up),
  by 26 ms at 555 entries and by 230
  ms at 4,723,
  so the thresholds of 512 (clone),
  1024 (install) and 4096 (copy) give up that much on copies just
  under them.
  Only `clone_tree` was run at these sizes;
  the copy stage and the install were not,
  so their thresholds are kept and
  listed as an open question.
- Not measured at all:
  `REMOVAL_SPLIT_DEPTH` (3).

- After the follow-up pick (option A,
  picked by the coordinator under the owner's delegation,
  open to the owner's
  veto):
  the thresholds are measured on the real functions and changed.
  The copy stage (`copy_manifest_entries`),
  the install (`install_snapshot`) and `clone_tree`,
  each forced to 1,
  2,
  4
  and 8
  workers,
  were never slower in parallel from 16 entries up,
  so the three thresholds (4096,
  1024 and 512) are now one
  value,
  `PARALLEL_BREAK_EVEN_ENTRIES`,
  16 ("Measurement record",
  thresholds).
  The old copy threshold left up to 145 ms on the table,
  the install 50 ms and the clone 12 ms.
- `REMOVAL_SPLIT_DEPTH` (3) was measured at depths 1 to 6 on a tree of 22,717 entries:
  medians 365,
  281,
  304,
  325,
  437
  and
  512 ms,
  depths 2 and 3 overlapping;
  kept.
  The tree has no very large subtrees,
  so a stage with a few was not measured.

### Install batch, 4096 entries

- Where:
  `src/native/worktree_copy_install.rs:38` (`INSTALL_BATCH_ENTRIES`),
  `doc/planning/cli-git-main-worktree-ignored-copy.md:271`.
- Value: 4096.
- Bounds:
  entries per durable intent record,
  so "the work an interrupted owner can leave unrecorded".
- When hit:
  another batch and another journal append.
- Fails or waits:
  waits.
- Citation:
  none.
- Conflict:
  "60s is fine for adding worktrees" bounds the add's total,
  not the batch;
  none found.

- After the follow-up pick (option A,
  picked by the coordinator under the owner's delegation,
  open to the owner's
  veto):
  measured,
  value kept.
  With a recorder that appends each batch to a file and syncs twice,
  installing 22,717 entries took 6,542,
  2,265,
  983,
  746,
  688 and 663 ms for batches of 64,
  256,
  1,024,
  4,096,
  16,384 and one batch,
  so a batch costs about 16 ms of syncs and
  4096
  entries add 12 percent to the install.
  The real journal's append was not run,
  and how many entries an interrupted owner may leave unrecorded is the owner's
  trade,
  so the value is kept.

### Free space preflight, 5 GiB

- Where:
  `src/native/subvolume_preflight.rs:26` (`MIN_FREE_BYTES`),
  check `162`,
  `README.md:493`.
- Value:
  5 GiB free on the parent directory.
- Bounds:
  whether the migration to a subvolume may start.
- When hit (`subvolume_preflight.rs:164`):
  "{parent} has {free} bytes free,
  fewer than the {n} bytes the migration
  requires
  for its metadata".
- Fails or waits:
  the command refuses and changes nothing.
- Citation:
  none.
  The comment says "the script's bound,
  5 GiB;
  about 1 KiB per entry is an assumption,
  not a measurement".
- Conflict:
  none found;
  disk full is on the "outside" list,
  but this refuses before the disk is full.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  replaced.
  The need is now the entries of the checkout times 1 KiB of metadata times 2 copies,
  2048 bytes an entry
  (`METADATA_BYTES_PER_ENTRY`,
  `METADATA_COPIES`,
  `required_free_bytes` in `src/native/subvolume_preflight.rs`).
  Measured:
  reflink-copying a 22,719-entry tree eight times grew `Metadata, DUP` used by 957.9 to 984.8 bytes per
  entry in three alternating rounds,
  against a no-copy control that moved by at most 9.9 MB ("Measurement record",
  free-space preflight);
  the filesystem keeps two copies of metadata (`Metadata ratio: 2.00`).
  The checkout's entries are counted by listing every directory once (`src/native/subvolume_entry_count.rs`),
  so the
  dry run now reads directory listings of the whole tree,
  without opening a file.
  A small checkout no longer needs 5 GiB free;
  a checkout of two million entries needs about 3.8 GiB,
  and one of four
  million about 7.6 GiB.
  Cost of the walk:
  `find package -xdev -printf .` listed 149,589 entries of this repository in 0.60,
  0.42 and 0.43 s
  (2.8 to 4.0 microseconds an entry,
  warm cache,
  `IOWeight=10`),
  so a checkout of two million entries takes about 6 to 8
  s warm;
  a cold cache was not measured.
  The count and the comparison now run only where the checkout is on btrfs.
  Limits:
  the metadata of the copy is assumed to need a new chunk,
  because the free space the check reads did not move
  with the metadata in the runs (preallocated metadata chunks absorbed it);
  the cost of the later passes and the index
  rewrite was not added;
  a filesystem with one copy of metadata is over-estimated by two.

### Changing-file copy attempts, 3

- Where:
  `src/native/subvolume_sync.rs:43` (`COPY_ATTEMPTS`),
  check `522`.
- Value: 3.
- Bounds:
  copies of a file that keeps changing under the pass.
- When hit (`subvolume_sync.rs:526`):
  "it changed while it was being copied,
  3 times in a row;
  stop the programs
  writing to
  it and run the command again".
- Fails or waits:
  the pass fails.
- Citation:
  none.
- Conflict:
  none that is clear;
  the cause is another program writing,
  which the owner's adopted reading ("races caused
  by
  other processes changing files ... are outside what it defends against",
  veto open) places outside the wrapper.

### Parallel budget divisors and memory estimates

- Where:
  `src/native/parallel_budget.rs:17` (`CPU_MARGIN_DIVISOR` 8) and `21` (`MEMORY_SHARE_DIVISOR` 2) and `171`
  (`SLICE_MEMORY` 16 MiB);
  `src/native/service_round.rs:26` (`REPOSITORY_MEMORY` 64 MiB);
  `src/native/manual_push_probe.rs:137` (`QUERY_MEMORY` 32 MiB);
  `src/native/worktree_trash_entries.rs:24`
  (`ENTRY_MEMORY` 64 MiB);
  `src/native/subvolume_index_files.rs:465` (`INDEX_REWRITE_COPIES` 4).
- Value:
  leave one CPU in eight free (at least one when there are more than two);
  use half of the available memory;
  and the per-item peaks as listed,
  all "estimated".
- Bounds:
  how many repositories the service drains side by side,
  how many queries,
  trash entries and index rewrites
  run at
  once (`worker_budget`).
- When hit:
  fewer workers.
- Fails or waits:
  waits;
  never fails.
- Citation:
  none;
  each comment says "estimated".
  `REPOSITORY_MEMORY` derives from "the items of a burst,
  the output of a push,
  and a review's patch and answer".
- Conflict,
  yes in part:
  "make sure we're running everything that can be run in parallel in parallel."
- Interaction with the unit caps:
  `SPEC.md:2515` computes "two repositories at once" from the unit's two CPUs and one
  gibibyte.
Once the caps are removed,
  the round width is set by the host's CPUs less one in eight and by half the available
  memory
  divided by 64 MiB,
  so these estimates are the only memory number left.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  - `REPOSITORY_MEMORY` changed from 64 MiB to 128 MiB:
    the 90th percentile of the `MemoryPeak` of 28 commit services
    is 130,482,176 bytes (median 33 MB,
    75th percentile 49 MB),
    rounded up to a power of two as `CONVERSION_MEMORY`
    is.
  - `ENTRY_MEMORY` changed from 64 MiB to 128 MiB:
    removing a flat directory of a million names grew the resident
    size by 97,320 KiB (99 bytes a name,
    against the 64 the old comment assumed).
  - `QUERY_MEMORY` kept at 32 MiB and now derived:
    `git rev-list --all` on this repository peaks at 29.0 MB resident
    plus 1.4 MB of output.
  - `SLICE_MEMORY` kept at 16 MiB as a conservative bound:
    `clone_tree` of 22,718 entries at 16 workers grew by 1.9
    MiB in all;
    the copy stage itself was not measured.
  - `MOST_CONCURRENT_QUERIES` kept at 4:
    local queries still gained 37 and 18 percent from four to eight at once,
    but the remote side was not measured (open question).
  - `INDEX_REWRITE_COPIES` is not measured.
  - `CPU_MARGIN_DIVISOR` and `MEMORY_SHARE_DIVISOR` are shares,
    not quantities;
    neither binds on this host (25 workers
    against measured ceilings of 8 and 16;
    more than 180 items against memory),
    and the owner's words "make sure we're
    running everything that can be run in parallel in parallel" are cited beside them.

- After the follow-up picks (options A and B,
  picked by the coordinator under the owner's delegation,
  open to the
  owner's veto):
  - `MOST_CONCURRENT_QUERIES` is split:
    `MOST_CONCURRENT_LOCAL_QUERIES` is 8 (`git ls-tree -r` took 80 and 50 ms at
    four and
    eight at once,
    `git rev-list` 231 and 190 ms,
    and no less at sixteen) and `MOST_CONCURRENT_REMOTE_QUERIES`,
    the
    `git ls-remote` queries that open a connection,
    stays 4.
  - `INDEX_REWRITE_COPIES` (4) was measured:
    an index of 2,786,564 bytes grew the process by 13,123,584 bytes and one of
    1,066,894 bytes by 7,929,856 bytes,
    which is 3.0 copies plus about 4.7 MB that does not grow with the index;
    kept.

## Saved copies, audit and log retention

### Saved copies of conflicted files, 30 days and 1 GiB

- Where:
  `src/native/discarded_prune.rs:35` (`DEFAULT_MAX_AGE_DAYS` 30),
  `38` (`DEFAULT_MAX_BYTES` 1 GiB),
  `41`
  (`YOUNG_FLOOR_MILLISECONDS` 24 h);
  settings `discardedCopies.maxAgeDays` and `maxSize` (`SPEC.md:239` to `250`);
  `README.md:756`,
  `SPEC.md:3880`.
  `src/native/discarded_store.rs:50` (`MAX_DEPTH` 128) keeps and reports a deeper tree.
  On `feat/git-policy-cli-saved-copies-and-audit-generations-measured` the three constants are at
  `discarded_prune.rs:42`,
  `46` and `49`,
  each with the measurement beside it.
- Value:
  saves older than 30 days go;
  then,
  while the folder is over 1 GiB,
  the oldest first;
  nothing younger than 24
  hours.
- Bounds:
  the folder `cli-git/discarded/` of copies the wrapper saved before a conflict-side command replaced a file.
- When hit:
  the oldest save is deleted and one audit line is written per removed save (`README.md:765`).
- Fails or waits:
  neither;
  it removes copies.
- Citation:
  partial.
  The rule is the owner's:
  "Prune by both age and size."
  (`doc/handover/cli-git-logging.md:269`).
  The same section says "The coordinator's defaults" (line 272);
  neither 30 days nor 1 GiB is an owner value or a
  measurement.
  The README sentence that gave this audit its "1 GiB" example (`README.md:756`) describes this folder.
- Conflict:
  this is the one place the wrapper deletes content it called "the only remaining content of replaced files"
  (`doc/handover/cli-git-logging.md:85`) against "never discard user data" (`SPEC.md:1645`).
  The owner chose the age-and-size rule,
  so the rule stands;
  the numbers are open.
- Measurement,
  2026-11-03T14:55Z,
  read-only (delegated pick 10,
  option C:
  "defaults set from the measured growth").
  The window is about one day,
  because the feature landed on
  2026-11-02 (`1d2f4bec8e` at 14:08:46 -05:00 takes a side of a
  conflicted file after saving it;
  `177fb4ca9b` at 16:20:34 -05:00 bounds the folder).
  - Command:
    `find /var/home/user -maxdepth 3 -name .git -type d`
    lists eleven repositories;
    `test -e <repo>/.git/cli-git/discarded`
    on each finds the folder in none of them.
    The linked worktrees under `.claude/worktrees` share the
    main repository's common Git directory,
    so they add no folder.
  - Command:
    `jq -c 'select(.copies or .outcome=="saved-conflict" or .outcome=="pruned")' .git/cli-git-audit.jsonl | wc -l`
    prints `0` for the main repository's audit file (950 lines,
    2026-10-21T00:35:32Z to 2026-11-03T14:15:51Z).
    The other repository with an audit file (`xinyanchen-portfolio/portfolio-xinyanchen0124`) holds 607 bytes of
    `allowed` lines.
  - The user's state directory (`~/.local/state/cli-git`) holds the service's per-repository state,
    `scratch` and
    `trust`,
    and no audit file,
    timing log or saved copy:
    the logs and the copies live in each repository's common Git
    directory,
    not in an XDG state directory.
  - The only proxy for how often a person takes a side of a conflicted file is the audit file's refusals:
    `jq` over
    the lines whose arguments match `--theirs` or `--ours` finds 12,
    all form `CheckoutPaths` and outcome `rejected`,
    on 6 different days from 2026-10-21T09:09Z to 2026-11-02T08:28Z,
    all before the carve-out landed that evening.
    That is fewer than one a day,
    and some may not have been
    conflicts;
    it says nothing about the size of a file or a save.
  - Verdict:
    there is no growth to measure,
    so no age or size can be derived from it.
    The constants stay at their earlier values with this measurement written beside them (`discarded_prune.rs`),
    and the question "Defaults for saved conflict copies" under "Questions for the owner" carries the options.
- Only-copy finding:
  yes,
  a saved copy can be the only copy of a person's work.
  `save_copies` (`src/native/conflict_sides.rs`) copies the worktree file as it is when the person runs
  `git checkout --theirs` or `--ours` (or `git restore` with them) on a conflicted path.
  That file is the merged text with its conflict markers plus
  any hand edits;
  the index holds only the base,
  ours and theirs
  stages,
  and no commit holds the merged text.
  After the command the file is replaced by one side,
  so the copy is the only place those bytes are,
  until a person
  commits or stages the same bytes.
  This is the case "never discard user data" (`SPEC.md:1645`) names,
  and the owner decided it knowingly:
  the handover
  section the owner answered says the copies are "the only remaining content of a file the command replaced" and that
  "pruning user data is the owner's decision" (`doc/handover/cli-git-logging.md:85` to `87`),
  and the owner chose
  "Prune by both age and size."
  (`doc/handover/cli-git-logging.md:269`).
  The coordinator's brief for this work says age and size limits must never delete such a copy;
  the code is unchanged,
  because honoring that would reverse an informed owner decision for every save,
  and that is a choice for the
  coordinator and the owner,
  not for the number-setting this pick asked for
  (see "Defaults for saved conflict copies" under "Questions for the owner").

### Audit log rotation, 1 MiB and one generation

- Note:
  the bullets from "Where" to "Citation" describe the audited commit.
  The bullets "Measurement,
  2026-11-03T14:55Z" and "Outcome of the delegated pick 11" supersede their account of a
  one-generation history for the audit file.
- Where:
  `src/native/discard_audit.rs:27` and `28` (`ROTATE_AT_BYTES`),
  check `278`,
  move aside `348` to `372`.
On `fix/git-policy-cli-audit-rotation-race` (`2eb2cf1bfc`) the same bound is at its lines 27 and 28 and the rotation
  is at
  its lines 430 to 475.
  On `feat/git-policy-cli-saved-copies-and-audit-generations-measured` the bound is `discard_audit.rs:36`,
  the history
  bound `AUDIT_HISTORY_BYTES` is at `45`,
  the check `298`,
  and the generations are in `discard_audit_generations.rs`
  (`install_generation` `121`,
  `trim_generations` `220`).
  `README.md:786`,
  `SPEC.md:3912`,
  `doc/handover/cli-git-logging.md:65`.
- Value:
  at 1 MiB the live file is renamed to `.1`,
  replacing the previous `.1`;
  so at most two files exist and every
  rotation after the second drops the oldest history.
  The one shared appender (`append_line`) serves five records:
  `cli-git-audit.jsonl` (discard decisions),
`cli-git-commit-timing.jsonl`,
  `cli-git-worktree-add-timing.jsonl`,
  `cli-git-push-trigger.jsonl`,
  and the prune lines
  the
  saved-copies pass writes into the audit file (`src/native/commit_timing.rs:403`,
  `add_timing.rs:278`,
  `push_trigger_log.rs:161`,
  `discarded_prune.rs:512`).
- Bounds:
  how much audit and timing history stays on disk.
- When hit:
  nothing is printed;
  the oldest generation is overwritten.
  The race fix's text says "Nothing is deleted,
  so no line is lost by the rotation" (branch `discard_audit.rs`,
  line
  436),
  which is true of the rotation step and false of the next rotation,
  which replaces `.1`.
- Fails or waits:
  neither;
  a failed write never changes a command's result (`README.md:786`).
- Citation:
  none.
  `doc/handover/cli-git-logging.md:65` records the behavior ("moves to `.1` at one mebibyte,
  replacing
  the
  older copy"),
  not a decision or a measurement.
The audit file exists so that a loss of uncommitted work "is attributable" (`src/native/discard_audit.rs:5` to `7`),
  and
each audit line names the saved copies by path (`doc/handover/cli-git-logging.md:79` and `84`),
  so a dropped
  generation can leave
  a saved copy with no line that names it.
  By contrast the standards-review shadow log "has no rotation" (`README.md:2010`) and is unbounded.
- Conflict:
  possible with "never discard user data" (`SPEC.md:1645`);
  see "The audit and timing logs' history" in the
  questions.
- Measurement,
  2026-11-03T14:55Z,
  read-only,
  in the main repository's common Git directory
  (`/var/home/user/Monochromatic/.git`;
  no `.1` file exists for any of the four).
  - Audit file:
    `wc -lc cli-git-audit.jsonl` prints 950 lines and 322,900 bytes;
    `jq -r '.timeMillis|tonumber/1000|todate' cli-git-audit.jsonl | sed -n '1p;$p'` prints
    2026-10-21T00:35:32Z and 2026-11-03T14:15:51Z,
    a window of about 13.6 days.
    Bytes per day (from `awk '{print length($0)+1}'` joined to the day of each line):
    from 1,074 (2026-10-28) to 71,328
    (2026-11-01,
    211 lines),
    about 23 KB a day on average.
    One mebibyte is therefore about 45 days of audit lines at the mean rate and about 15 days at the busiest day.
    Lines grew with the schema:
    the mean line over the whole
    file is 340 bytes,
    the newest day's 67 lines are 489 bytes each.
  - Commit timing:
    1,040 lines and 760,605 bytes from 2026-11-02T09:01:37Z to 2026-11-03T14:55:02Z (about 30 hours),
    so
    one mebibyte arrives in about two days.
    Worktree-add timing:
    46 lines and 87,158 bytes since 2026-11-03T08:55:46Z.
    Push trigger:
    141 lines and 64,785 bytes from 2026-11-02T19:48:24Z.
    These windows are short:
    the commit timing file has no `.1` and starts one day ago,
    so it may have been started or
    emptied then;
    the rate is the only thing measured.
- Outcome of the delegated pick 11,
  option C (the coordinator's pick under the owner's delegation,
  open to the owner's
  veto):
  implemented in `src/native/discard_audit.rs` and `src/native/discard_audit_generations.rs`.
  - Each rotation of the audit file keeps its file under the next free number
    (`.1`,
    `.2`,
    and on;
    the highest number is the newest),
    claimed with a hard link
    so that two writers never take one number and nothing is replaced.
  - After a rotation,
    a pass removes the oldest generations
    until the generations that name no saved copy on disk,
    and the
    files a killed rotation left at a private name,
    total at most `AUDIT_HISTORY_BYTES` (4 MiB).
    A generation with a `saved-conflict` line whose copy place
    exists,
    that cannot be read,
    or whose line cannot be parsed
    or shows a place as unreadable text,
    is outside that total and is never removed.
    A file left at a private name by a killed rotation is given the next generation number by that pass
    (readers go by number),
    and counts like any generation from then on;
    its second name,
    when a kill came
    between the link and the removal,
    is removed,
    and so is the higher of two numbers that name one file
    (fixed 2026-11-07 on `fix/git-policy-cli-flakes-and-open-findings`;
    before it such a file was counted and never read).
  - The timing logs and the push trigger log keep one generation,
    `.1`,
    as before.
  - The 4 MiB is derived,
    not measured:
    the busiest measured
    day in the current line size is about 211 x 489 = 103 KB,
    30
    days of it (the age default of the saved copies the lines
    explain) is about 3.0 MiB,
    and four generations hold that with
    one to spare.
  - The citation class of the rotation size stays partial:
    the rate is measured,
    and the rotation size and the history
    horizon are judgements on it.
- Conflict:
  resolved for the audit file by delegated pick 11,
  because no generation that explains a copy on disk is dropped.
  A timing log still drops its older generation;
  it is a diagnostic,
  not user data.

### Clean-check content and attribute reads

- Where:
  `src/native/worktree_clean_content.rs:37` (`CONTENT_LIMIT` 32 MiB) and `40` (`ATTRIBUTES_LIMIT` 4 MiB);
  `src/native/worktree_attributes.rs:28` (`MATCH_WORK_LIMIT` 4,194,304 cells);
  `src/native/conflict_side_guard.rs:52` (`INDEX_LIMIT_BYTES` 1 GiB).
- Value:
  as listed.
- Bounds:
  the content comparison that decides whether a path is clean.
- When hit (`worktree_clean_content.rs:419`):
  "{path} is larger than 33554432 bytes,
  which is more than is read to
  compare it
by content" and the path is not judged clean;
  an over-large index makes the guard decline
  (`conflict_side_guard.rs:122`).
- Fails or waits:
  refuses the clean decision,
  so the guarded command is refused or goes native.
- Citation:
  none.
- Conflict:
  none found.

## Locks left by a Git that was killed

### Lock age margin, two seconds

- Where:
  `src/native/stale_ref_lock_judge.rs:24` (`MARGIN`),
  `SPEC.md:1705`.
- Value:
  a lock younger than two seconds is left alone and not mentioned.
- Bounds:
  when the wrapper judges whether a reference lock is left behind.
- When hit:
  nothing is printed for a young lock.
- Fails or waits:
  neither;
  a lock that is a leftover and younger than the margin stays until a later pass.
- Citation:
  none.
  `SPEC.md:1830` to `1842` measured that Git fills a reference lock after creating it empty,
  but records no duration.
- Conflict:
  none found.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  changed to
  four seconds.
  `MARGIN` is the whole second above `LONGEST_LIVE_REF_STEP`,
  3,438 ms,
  the largest reference step of 1,302
  landings in the service journal (1 above 2 s).
  Real Git on a throwaway repository held a reference lock for at most 2.8 ms over 7,700 traced updates,
  so the
  journal tail is the loaded case.
  The longest of a sample understates the tail;
  whether to use a multiple of it is an open question.
  `SPEC.md` states the new value.

- After the follow-up pick (option B,
  picked by the coordinator under the owner's delegation,
  open to the owner's
  veto):
  changed again,
  to seven seconds.
  `MARGIN` is `MARGIN_MULTIPLE` (2) times `LONGEST_LIVE_REF_STEP` (3,438 ms),
  rounded up to a whole second,
  7 s;
  a
  stale lock
  is left that long before it is judged.
  The binary tests whose sleeps follow `MARGIN` run that much longer.

### Boot proof slack, one hour

- Where:
  `src/native/stale_ref_lock_judge.rs:37` (`BOOT_SLACK`),
  `41` (`REUSE_SLACK_MS` 100),
  `SPEC.md:1769`.
- Value:
  a lock must predate the boot by more than 3600 seconds for the boot proof.
- Bounds:
  the proof that a lock belongs to an earlier boot.
- When hit:
  the lock is not removed by that proof.
- Fails or waits:
  neither.
- Citation:
  none (reasoned from clock steps).
- Conflict:
  none found.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  changed to
  `ZONE_OFFSET` plus `MARGIN`,
  3604 s.
  `chronyd` stepped this host by -3599.507590 s on 2026-11-01,
  0.49 s short of an hour,
  so a step in the other direction
  can be as far over the hour;
  the old slack of exactly an hour left no room.
  The same journal shows a step of 2,934,868 s (34 days,
  2026-10-17,
  a hardware clock far behind),
  which no slack can
  cover without refusing every real reboot;
  that case stays the one left,
  as `SPEC.md` already says.
  The other two constants of the entry are kept with measurements beside them:
  `REUSE_SLACK_MS` (100 ms) against a
  start-time error of at most 2.331 ms beyond the 20 ms resolution (two runs of 1,000 spawns),
  and the kernel tick of
  1 ms (`CONFIG_HZ=1000`).

- After the follow-up pick on the margin:
  `BOOT_SLACK` follows `MARGIN`,
  so it is 3607 s.
- Retired (the branch of this fix,
  `fix/git-policy-cli-never-removes-a-live-lock`):
  the boot proof,
  `BOOT_SLACK`,
  `ZONE_OFFSET` and `MEASURED_ZONE_STEP` are gone.
  No slack makes a wall-clock comparison a proof,
  because a forward correction of the clock during the boot moves `btime`
  and every file time alike (cross-vendor review finding S-2).
  A lock that looks older than the boot is now left,
  and the notice says so beside the command that clears it.
  `REUSE_SLACK_MS` and its measurement stay.

### Empty-lock settle time, one second

- Where:
  `src/native/stopped_git_locks.rs:59` (`EMPTY_LOCK_SETTLE`) and `54` (`COARSE_CLOCK_SLACK` 50 ms),
  `SPEC.md:1544`.
- Value:
  an empty lock without a pid file must stay empty for one second after a stopped child's tree has ended before
  the wrapper removes it.
- Bounds:
  the sweep after a push or step was stopped.
- When hit:
  the sweep waits out what remains of the second.
- Fails or waits:
  waits up to one second.
- Citation:
  none (`SPEC.md:1542` to `1545` gives the reason;
  no figure).
- Conflict:
  none found.

- After the delegated pick (option A,
  the coordinator's pick under delegation,
  open to the owner's veto):
  cited,
  values kept.
  A reference lock was empty for at most 2.244 ms between creation and first write over 7,700 traced updates (median
  0.38 ms),
  and `COARSE_CLOCK_SLACK` (50 ms) is about thirty times the longest lag of a new file's birth time behind a
  clock read (1.729 ms in two runs of 10,000).
  The load average of 80 to 145 that the commit-timeouts record names was not present during the runs,
  so the factors
  are not a proof for it.

- Retired (the branch of this fix,
  `fix/git-policy-cli-never-removes-a-live-lock`):
  `EMPTY_LOCK_SETTLE` and
  `COARSE_CLOCK_SLACK` are gone with the removal of an empty lock without a pid file.
  A live writer of another Git holds such a lock with no descriptor open for as long as its transaction lasts,
  so no settle
  time,
  however long,
  tells it from a left one (cross-vendor review finding S-1).
  The sweep after a stop now removes a lock only on the proofs of `SPEC.md`,
  "Reference locks left by a Git that was
  killed".
  `COARSE_CLOCK_SLACK` (50 ms,
  measurement unchanged in `Empty-lock settle time, one second`) returned in
  `stale_ref_lock_judge.rs` for one use:
  telling a lock
  born in a stopped child's life from an older one,
  in the wrapper's own proof for a child it stopped and saw end.

## In-process readers, parsers and chunking

These bounds make a reader decline and leave the question to real Git,
or split a command line.
None changes a command's result;
a hit costs a Git start.
That cost is against "Invoke real git as rarely as possible" only when the bound is reached.

### File size bounds of internal readers

- Where:
  `src/native/ref_files.rs:27` and `31` (512,
  256 bytes);
  `src/native/repo_read_refs.rs:19` (4096) and `22`
  (256 MiB);
  `src/native/repo_read_odb.rs:25` (1 MiB) and `28` (256 MiB);
  `src/native/sequencer_marker.rs:26` (64 MiB);
  `src/native/repo_read_config_text.rs:15` (1 MiB);
  `src/native/repo_read_layout.rs:61` (4096);
  `src/native/service_index_marker.rs:28` (512);
  `src/native/single_owner_claim.rs:207` (4096);
  `src/native/worktree_convert.rs:35` (64 KiB);
  `src/native/worktree_convert_journal.rs:35` (1 MiB);
  `src/native/worktree_trash_dispose.rs:42` (64 KiB);
  `src/native/real_git_candidate.rs:49` (64 KiB);
`src/native/doctor.rs:268` (1024);
  `src/native/git_system_config.rs:31` (4096);
  `src/native/git_version_cache.rs:48`
  (1024);
  `src/native/repo_read_object.rs:17` (256 MiB) and `102` (512 KiB inflate input);
`src/native/candidate_batch.rs:63` (128-byte reply header);
  `src/native/systemd_dbus.rs:42` is listed with the manager
  exchange.
- Value:
  as listed.
- Bounds:
  the largest file a reader looks at.
- When hit:
  "anything bigger is left to Git" (`repo_read_object.rs:16`),
  "a bigger file is left to Git"
  (`sequencer_marker.rs:25`),
  or the check answers "not a match".
- Fails or waits:
  falls back,
  never fails.
- Citation:
  none (rationales such as "a real one is a short line";
  none measured on a real repository).
- Conflict:
  none found.

- After the delegated pick (option B,
  the coordinator's pick under delegation,
  open to the owner's veto):
  the reader
  bounds that make the reader decline and Git answer are classed
  exempt with their reason beside each constant (see "Exempt limits").
  The bounds on journals and registration files that the wrapper itself writes (`worktree_convert_journal.rs`,
  `worktree_convert.rs`,
  `worktree_trash_dispose.rs`,
  `single_owner_claim.rs`,
  `service_index_marker.rs`) were not
  classed:
  what a hit does there was not read for this pick,
  so they stay uncited.

### Depth and loop bounds of readers

- Where:
  `src/native/repo_read_config_typed.rs:27`,
  `src/native/repo_read_config_scan.rs:20`,
  `src/native/repo_read_config_list.rs:22` (`INCLUDE_DEPTH_LIMIT` 10);
  `src/native/repo_read_odb.rs:22`
(`ALTERNATE_DEPTH_LIMIT` 5);
  `src/native/repo_read_refs.rs:25` (5 symbolic hops);
  `src/native/repo_read_query_refs.rs:29`
and `src/native/lfs_object_plan.rs:64` (peel 8,
  5);
  `src/native/repo_read_pack.rs:23` (`DELTA_DEPTH_LIMIT` 100) and
  `27`
  (`CACHE_BYTE_LIMIT` 32 MiB);
  `src/native/repo_read_tree.rs:23` (256),
  `26` (4,000,000 entries),
  `168` (3 lists);
  `src/native/repo_read_*` `ENV_ENTRY_LIMIT` 1024 and `DIGIT_LIMIT` 9 (`repo_read_config_typed.rs:30`,
  `33`;
`repo_read_config_list.rs:25`;
  `repo_read_layout.rs:64`;
  `config_snapshot.rs:30`);
  `src/native/forwarded_command.rs:39`
  (`MAX_ALIAS_EXPANSIONS` 16);
  `src/native/service_pruned.rs:66` (`CLOSURE_ENTRY_BUDGET` 5,000,000);
  `src/native/discarded_store.rs:50` is listed with saved copies.
- Value:
  as listed.
- Bounds:
  loops and recursion.
- When hit:
  the reader declines and Git answers.
  Exceeding sixteen alias expansions is the one that changes behavior:
  "Alias expansions followed before giving up".
- Fails or waits:
  falls back.
- Citation:
  partial for the ones the comment calls "Git's own limit" (`INCLUDE_DEPTH_LIMIT` 10 and the alternate depth
  5):
the owner's practice is to cite the Git 2.56.0 file and line (handover,
  "Recorded here only"),
  and none is beside
  them.
  The rest:
  none.
- Conflict:
  none found.

- After the delegated pick (option B,
  the coordinator's pick under delegation,
  open to the owner's veto):
  `SYMBOLIC_DEPTH_LIMIT` and `ALTERNATE_DEPTH_LIMIT` are classed exempt.
  The include depth copies and the others of the entry were not classed here and stay as audited.

### Argument chunking and display caps

- Where:
  `src/native/candidate_store.rs:107` (`COMMIT_CHUNK_SIZE` 2048),
  `src/native/candidate_prediction.rs:87`
  (`PATHSPEC_CHUNK_SIZE` 2048),
  `src/native/candidate_attribute.rs:43` (`ATTRIBUTE_CHUNK_SIZE` 2048),
  `src/native/commit_staged_additions.rs:26` (`ENTRIES_PER_START` 200);
  display and record caps:
  `src/native/auto_push_dirty_paths.rs:80` (5 paths),
  `src/native/worktree_copy_transaction.rs:305` (3 paths),
  `src/native/discarded_prune.rs:51` (100 names),
  `src/native/subvolume_sync.rs:118` (20 samples),
  `src/native/index_lock_evidence.rs:37` (64 characters),
  `src/native/push_failure_notice.rs:44` and `47` (100,
  4000),
  `src/native/package_repair_command.rs:78` (600 bytes),
  `src/native/standards_review_request.rs:30` (200),
  `src/native/standards_review_entry.rs:18` (12),
  `src/native/journald_native.rs:47`,
  `50`,
  `117`,
  `484`
  (16 KiB message,
  1024 field,
  64 name,
  4096 pipe chunk);
  `src/native/worktree_copy_compare.rs:33` (64 KiB read).
- Value:
  as listed.
- Bounds:
  how many paths one Git start takes,
  how many items a message names,
  and how many characters a log field
  keeps.
- When hit:
  more Git starts,
  or "and {n} more",
  or text cut.
  For the journal,
  an entry too large for the socket "goes to standard error" (`journald_native.rs:47`).
- Fails or waits:
  neither.
- Citation:
  none.
  The chunk sizes say they are "far below any argument limit".
  The 64-character journald field-name bound is journald's own rule,
  with no source line.
- Conflict:
  none found.

- After the delegated pick (option B,
  the coordinator's pick under delegation,
  open to the owner's veto):
  classed
  exempt beside each constant (see "Exempt limits").
  The three chunk sizes of argument lists are now one value (`ATTRIBUTE_CHUNK_SIZE` and `COMMIT_CHUNK_SIZE` derive from
  `PATHSPEC_CHUNK_SIZE`,
  rule `MGN`),
  and a control checks a chunk of the longest arguments against half the operating
  system's argument limit.

## Every wait has a bound

The owner,
about 2026-11-03T12:37-05:00 (quoted):
"nothing should ever wait forever or indefinitely".
The inventory of every wait,
with the options for the ones not decided,
is `doc/planning/cli-git-no-unbounded-waits.md`;
the bounds are in
`src/native/wait_ceilings.rs`,
each with its measurement beside it:

- `ANSWER_CEILING` 20 minutes;
  `WITHDRAW_CEILING` 10 minutes;
  `START_CEILING` 60 seconds;
  `SERVICE_HOLD_CEILING` 10 minutes
  (measured,
  with the margins given at each limit's own section).
- `LOCK_CEILING` 10 minutes:
  measured on 2026-11-08 from the commit timing log (1,332 records,
  2026-11-05T03:13:38Z to 2026-11-08T20:59:20Z).
  The six logged waits for another program's hold on the index were two that landed after 3,636
  and 23,783 ms of answer time and four queued behind the lock that never went on 2026-11-07
  (62 waiting notices each,
  644,710 to 1,200,776 ms,
  every one stopped).
  No wait a holder released came near the ten minutes,
  and the longest of those four is just past the client's
  twenty-minute `ANSWER_CEILING`.
  No wait takes it:
  it stays as the basis of `FOREIGN_OWNER_PATIENCE` (twice this),
  which another change owns,
  and the three waits it once bounded take `MERGE_LOCK_CEILING`,
  `SETTLE_LOCK_CEILING` and
  `RECONCILE_LOCK_CEILING`,
  one minute each.
  See "The lock ceiling and the three waits it fed" in the measurement record.
- `CONNECTED_PUSH_QUIET_WINDOW` 150 seconds:
  unmeasured,
  labelled as the quiet windows are.
- `PHASE_PATIENCE` 5 minutes:
  the limit of any test (`.config/nextest.toml`).
- `PLUMBING_STEP_CEILING` 120 s,
  `OBJECT_STEP_CEILING` 600 s,
  `REFERENCE_STEP_CEILING` 60 s:
  the longest one Git step runs,
  whatever it
  shows (the coordinator's pick,
  under the owner's delegation,
  veto open).
  Measured:
  longest read step 9,663 ms (`reads_ms`,
  1,651
  landings),
  longest client preparation 250,280 ms (`prepareMs`,
  1,400 records),
  longest object step 1,358 ms,
  longest reference
  update 3,438 ms (`ref_ms`).
- The data read on 2026-11-03:
  the timing log of this repository (1,329 records;
  `answerWaitMs` longest 308,267 ms,
  `totalMs` longest
  750,604 ms,
  `startWaitMs` longest 1,041 ms) and the service journal (715 logged holds,
  longest 119,721 ms;
  794 pushes,
  longest 72.4 s).
  No log holds a foreign index-lock holder or a quiet gap of a connected push.

The owner's answers of about 12:45-05:00:
at a bound "Return with an error";
the saved commit does not land later,
"No,
keep it for
a retry (Recommended)";
pushes "Bound it,
keep retrying (Recommended)".
These replace the unbounded waits of the delegated picks 2
and 6,
the follow-up pick that waited for an unproven owner with no bound,
and "Wait while connected".
Open for the coordinator
to pick,
with options and rankings in the inventory:
a ceiling for Git steps that keep working or are never stopped,
the person's
own Git and hooks,
the value of the connected window,
and a person's `git commit` with an editor open at the lock ceiling.

## Exempt limits

The pick on "Which limits need a citation under LQM" is option B (the coordinator's pick under the owner's delegation,
open to the owner's veto):
a limit needs a citation when it can change a command's result or its duration by more than
a poll.
Limits that cannot are classed here once,
with the reason,
and each carries a comment that names its class and points
here.
They still follow rule `MGN`:
no value is a bare literal,
and a value that equals another is derived from it.

### Classes

- Reader fallback:
  a file,
  chain or count past the bound makes an in-process reader decline,
  and real Git answers.
  The result is the same;
  a Git start is added.
- Chunk size:
  the bound splits one command into several starts of the same command.
  The result is the same;
  only the number of starts changes.
- Display cap:
  the bound limits how much text a message,
  log field or record names.
  No command result or duration depends on it.
- Notice pacing:
  the bound times a line the person reads or a log threshold.
  Nothing waits on it beyond the wait that already exists.
- Poll:
  the bound sets how often a condition is looked at again.
  It adds at most one poll interval to a wait that already exists.
- Buffer size:
  the bound sizes one read buffer;
  the result does not depend on it.

### Constants classed

- Reader fallback:
  `HEAD_LIMIT` and `LOOSE_REF_LIMIT` (`ref_files.rs`);
  `LOOSE_LIMIT`,
  `PACKED_LIMIT` and
  `SYMBOLIC_DEPTH_LIMIT` (`repo_read_refs.rs`);
  `ALTERNATE_DEPTH_LIMIT`,
  `ALTERNATES_LIMIT` and `PACKED_REFS_LIMIT`
  (`repo_read_odb.rs`);
  `PACKED_REFS_LIMIT` (`sequencer_marker.rs`);
  `CONFIG_TEXT_LIMIT` (`repo_read_config_text.rs`);
  `SMALL_FILE_LIMIT` and `ENV_CONFIG_ENTRY_LIMIT` (`repo_read_layout.rs`);
  `OBJECT_SIZE_LIMIT` (`repo_read_object.rs`).
- Chunk size:
  `COMMIT_CHUNK_SIZE` (`candidate_store.rs`),
  `PATHSPEC_CHUNK_SIZE` (`candidate_prediction.rs`),
  `ATTRIBUTE_CHUNK_SIZE` (`candidate_attribute.rs`),
  `ENTRIES_PER_START` (`commit_staged_additions.rs`).
- Display cap:
  `LISTED_LIMIT` (`auto_push_dirty_paths.rs`),
  `NOTICE_PATHS` (`worktree_copy_transaction.rs`),
  `LISTED_COPIES` (`discarded_prune.rs`),
  `SAMPLE_SIZE` (`subvolume_sync.rs`),
  `QUOTED_TEXT_LIMIT`
  (`index_lock_evidence.rs`),
  `LONGEST_REASON` and `LONGEST_KEY` (`push_failure_notice.rs`),
  `OUTPUT_TAIL_BYTES`
  (`package_repair_command.rs`),
  `FAILURE_EXCERPT_UNITS` (`standards_review_request.rs`),
  `SHORT_SHA_LENGTH`
  (`standards_review_entry.rs`),
  `MESSAGE_LIMIT` and `FIELD_LIMIT` (`journald_native.rs`).
- Notice pacing:
  `DEFAULT_NOTICE_INTERVAL` and `NOTICE_GRACE` (`wait_progress.rs`),
  `NOTICE_EVERY`
  (`service_index_lock.rs`),
  `SLOW_LOCK_STEP` (`service_timing.rs`),
  `PUSH_QUEUED_AFTER` (`service_handoff.rs`,
  15 seconds:
  a push record still in `pending` after it has outlasted every in-flight claim measured on this host,
  and the line only says the push has not been made).
- Poll:
  `NOTICE_POLL` (`markdown_linter_guard.rs`);
  `POLL_PAUSE` (`service_index_watch.rs`,
  50 ms),
  the pause of a lock wait
  that has no watch (delegated follow-up pick 2,
  2026-11-03:
  the merge wait,
  the landing and the forwarded writers' wait for the
  service's hold poll instead of failing or waiting blind).
  It sets how soon a release is noticed,
  never whether a wait ends,
  and 50 ms is within the 15 to 100 ms holds measured for Git's own short commands.
- Buffer size:
  `COMPARE_BUFFER_BYTES` (`worktree_copy_compare.rs`).

### Not classed here

- Constants in files that other agents own:
  the look counts and pauses of the lock wait,
  the class graces and bounds of
  Git child steps,
  the notice grace and waits of the service client,
  the push and LFS polls and pauses,
  and the
  tool deadlines.
- Bounds whose hit was not read for this pick,
  such as the include depth copies,
  `MAX_ALIAS_EXPANSIONS`,
  the peel
  limits,
  the delta depth and cache bytes,
  the tree entry bounds,
  the record and journal size limits of files the
  wrapper writes,
  and `CLOSURE_ENTRY_BUDGET`:
  they stay in "Uncited limits".
- The 64-character journald field-name bound is journald's own rule,
  with no source line yet.

## Measurement record

All figures were taken on 2026-11-03 on this host (28 threads,
a btrfs volume,
other sessions running,
load average 3 to
20 during the runs) and under the load there was;
no quiet host was waited for and no synthetic load was made.
Scratch scripts were under `${HOME}/temp/agent/limits`;
the real logs,
`.git`,
systemd user directory and service were
read only.

### Free-space preflight

- Command:
  three rounds,
  each alternating a copy arm and a no-copy control arm.
  Copy arm:
  `btrfs filesystem sync`,
  `btrfs filesystem df --block-size=1`,
  eight `cp --archive --reflink=always` of a
  tree of 22,719 entries (`git archive HEAD` of this repository,
  825 MB apparent,
  on the same volume) into a throwaway
  directory,
  sync,
  `btrfs filesystem df` again.
  Control arm:
  the same reads around a 20 s pause.
  Run in `systemd-run --user --scope` with `CPUQuota=200%`,
  `MemoryMax=2G`,
  `TasksMax=50`,
  `IOWeight=10`.
- Copy arm,
  `Metadata, DUP` used delta over 181,752 entries:
  178,995,200,
  177,930,240 and 174,096,384 bytes,
  which is
  984.8,
  979.0 and 957.9 bytes an entry;
  wall time 11.6,
  11.9 and 12.1 s.
- Control arm delta:
  -737,280,
  -81,920 and -9,863,168 bytes (run-to-run noise of the volume,
  at most 9.9 MB).
- `btrfs filesystem usage`:
  `Metadata ratio: 2.00`.
- The statfs free figure did not follow the metadata delta (preallocated metadata chunks absorbed it),
  so the check
  assumes the metadata needs new chunks.

### Reference lock timing

- Command:
  `strace -f -ttt` on `/usr/bin/git -c core.lockfilePid=true update-ref` (Git 2.56.0) in a throwaway
  repository,
  300 runs then four arms of 2,000,
  alternating;
  strace slows each call,
  so the figures are upper bounds.
- Create to first write (fill):
  median 0.381 to 0.389 ms,
  99th percentile 0.552 to 0.569 ms,
  maximum 1.040 to 2.244 ms.
- Create to rename:
  median 0.905 to 0.919 ms,
  99th percentile 1.237 to 1.305 ms,
  maximum 1.582 to 2.756 ms.
- Arms whose labels repeated (`armA` twice,
  `armB` twice) agree within 2 percent at the median.
  The arm `armA` produced 1,700 usable runs because 300 of its refs already existed.
- Service journal:
  `journalctl --user --output=json SYSLOG_IDENTIFIER=cli-git-service`,
  1,302 `answered` entries from
  2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z;
  `ref_ms` median 2,
  90th percentile 4,
  99th percentile 64,
  maximum 3,438;
  1 entry above 2,000 and 2 at or above 1,000.
- Clock:
  `CONFIG_HZ=1000`;
  a new file's birth time lagged a clock read just before the create by at most 1.684 and
  1.729 ms (two runs of 10,000 creates);
  the start time derived from `/proc/<pid>/stat` and `/proc/uptime` exceeded the
  clock read before the spawn by at most 2.331 and 2.329 ms (two runs of 1,000 spawns).
- Entry listing:
  `find package -xdev -printf .` in `systemd-run --user --scope` with `CPUQuota=100%` and `IOWeight=10`,
  three runs,
  149,589 entries each:
  0.60,
  0.42 and 0.43 s,
  warm cache.
- Boot:
  `journalctl -u chronyd` since 2026-10-06 (19 boots) shows steps of 1.292345 s (2026-10-30),
  -3599.507590 s
  (2026-11-01) and 2,934,868.227998 s (2026-10-17).

### Parallelism

- Command:
  a probe test (not committed) calling `clone_tree` on reflinked trees made from `git ls-files` of this
  repository,
  destinations on the same volume,
  orders alternating by round,
  run through
  `mise run //package/git-policy/cli:native:test:host` in a scope with `CPUQuota=1800%`,
  `MemoryMax=12G`,
  `TasksMax=600`,
  `IOWeight=10`.
- 22,718 entries,
  medians of three runs,
  milliseconds at 1,
  2,
  4,
  8 and 16 workers:
  1549,
  880,
  555,
  397,
  415 (16 workers
  ranged 383 to 444).
- 9,301 entries:
  619,
  363,
  236,
  183, 194.
  4,723 entries:
  330,
  190,
  124,
  99, 106.
  2,119 entries:
  143,
  78,
  47,
  33, 35.
  1,077 entries:
  74,
  46,
  29,
  20, 21.
  555 entries:
  39.6,
  25.9,
  16.0,
  13.5,
  13.0.
- Below 512 leaves the function stays on one thread by design;
  296 entries took 20.6 ms and 167 took 10.7 ms on one
  thread
  in all columns.
- Resident size of the probe process:
  21.6 MiB before a 16 worker clone of 22,718 entries,
  23.5 MiB at the peak.
- Trash removal,
  `remove_tree_beneath` on one flat directory of 42 character names,
  fresh process,
  peak minus start:
  100,000 names 131 bytes a name,
  200,000 names 113 (twice),
  400,000 names 104,
  1,000,000 names 97,320 KiB (99 bytes).
- Queries:
  16 branch tips of this repository,
  medians of three runs alternating order,
  `git ls-tree -r` at 1,
  2,
  4,
  8,
  16
  at once:
  218,
  114,
  80,
  50,
  46 ms;
  `git rev-list <tip>`:
  845,
  413,
  231,
  190,
  194 ms.
  `git rev-list --all` (34,481 commits):
  resident 29,164,
  29,032 and 28,848 KiB,
  1,413,721 bytes of output;
  `git
  ls-tree -r`
  of `main`:
  26,080 KiB,
  2,459,743 bytes.
- Service peaks:
  `systemctl --user show 'cli-git@*.service' --property=MemoryPeak,MemoryCurrent` for the 28 running
  services;
  peaks sorted:
  5.7,
  11.1,
  12.7,
  18.8,
  20.0,
  22.2,
  27.7,
  29.9,
  29.9,
  30.0,
  30.8,
  32.9,
  35.6,
  36.2,
  36.9,
  38.6,
  40.7,
  43.9,
  48.0,
  49.0,
  126.7,
  128.4,
  129.0,
  130.0,
  130.5,
  280.9,
  355.2 and 1073.8 MB;
  the last ran under the old one
  gibibyte cap;
  `MemoryCurrent` summed to 222,420,992 bytes.

### Service housekeeping

- Command:
  `journalctl --user --output=json --output-fields=MESSAGE,CLI_GIT_RELEASE,_SYSTEMD_USER_UNIT
  SYSLOG_IDENTIFIER=cli-git-service` into a scratch file,
  5,424 entries from 2026-11-01T13:22:38Z to
  2026-11-03T15:12:02Z,
  of which 1,302 are `answered` and 1,085 belong to `cli-git@` units.
- Commit timing log:
  `.git/cli-git-commit-timing.jsonl`,
  1,050 rows at the first read (1,051 at the last;
  the file
  grew),
  2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z;
  `totalMs` median 829,
  90th percentile 8,552,
  maximum 750,604 (26 over 60 s,
  1 over 300 s).
- Add timing log:
  48 rows,
  2026-11-03T08:55:46Z to 15:06:54Z;
  `totalMs` median 49.6 s,
  maximum 539.7 s.
- Batches:
  `size` and `position` are 1 in all 1,302 answers;
  `attempts` 1;
  outcomes 1,291 landed and 11 branch-moved.
- Landing steps:
  `journal_ms` median 8,
  99th percentile 3,792,
  maximum 40,188;
  `landing_ms` median 69,
  90th percentile
  535,
  maximum 119,836.
- Bursts of the `cli-git@` units:
  busiest unit 4 landings in 10 s,
  4 in 60 s,
  15 in 600 s,
  349 in all.
- Idle exits:
  110 over 60 s,
  uptime 10,800,000 to 24,236,563 ms;
  reasons `idle` 111,
  `retire` 71,
  `signal-15` 8.
- Cold start:
  commits after an idle gap over 3 h (10 commits) had `reachMs` at most 70 and `startWaitMs` at most 47;
  over
  1 h (13 commits) `startWaitMs` at most 47.
- Changes per commit:
  `git log --no-merges --format=@%H --name-only --all --since=2026-09-01`,
  21,696 commits,
  176,042
  paths:
  median 2,
  90th percentile 7,
  99th percentile 38,
  99.9th percentile 377,
  maximum 8,178,
  10 over 4,096;
  path
  length median 58,
  90th percentile 86,
  99th percentile 118,
  99.9th percentile 159,
  maximum 189.
- Merges:
  `git log --first-parent --merges -n 1500 main`,
  439 merges;
  commits brought in per merge median 8,
  99th
  percentile 317,
  maximum 6,542,
  1 over 4,096.
- Kills:
  `journalctl --user --grep` for killed,
  out-of-memory,
  failed result or main-process-exit lines of `cli-git@`
  units found none in 19 boots.

## Quiet gaps in the existing logs

This section is the measurement the quiet windows of picks 4 and 9 need,
and what it found.
Read-only reads of `/var/home/user/Monochromatic/.git` at 2026-11-03T10:53-05:00;
no other repository on the host
holds a
timing log (checked for every directory in `/var/home/user`).

**Finding.**
No log held a quiet gap before this change.
The commit timing record held the time of each phase,
and the add timing record held the time of each step,
and
neither says
how long a step showed no sign of work.
A whole time is an upper bound of a gap,
never the gap,
so the logs cannot set an interval.
The class numbers therefore stay as labelled unmeasured quiet windows,
and no margin is invented.

Commit timing,
`cli-git-commit-timing.jsonl`:

- Command:
  `jq -s '{count:length, first:(map(.startMillis)|min), last:(map(.startMillis)|max)}'
  cli-git-commit-timing.jsonl`.
- Window:
  2026-11-02T09:01:37Z (`startMillis` 1793610097597,
  release 0.5.10) to 2026-11-03T15:52:58Z (release 0.5.27).
- Count:
  1105 records at the time of the read (the file grows as commits land):
  1088 landed,
  5 ended,
  12 stopped.
- Phases,
  whole times (`jq -s` over `totalMs`,
  `prepareMs`,
  `buildMs`,
  `submitMs`,
  `answerWaitMs`,
  `gateMs`,
  `afterLandingMs`):
  total p50 0.79 s,
  p99 184.2 s,
  maximum 750.6 s;
  prepare maximum 250.3 s;
  build maximum 183.3 s;
  submit maximum 308.3
  s of which
  the answer wait is 308.3 s;
  gate maximum 38.6 s;
  after landing maximum 43.1 s.
- What it says:
  nothing about one step.
  A phase holds many steps and the waits for the queue and for locks,
  so a long
  phase is
  not a long quiet gap of a Git child.

Add timing,
`cli-git-worktree-add-timing.jsonl`:

- Command:
  `jq -s -c '[.[] | .steps | to_entries[] | select(.key|startswith("git ")) | {step:.key, ms:.value.ms,
  calls:.value.calls}]
  | group_by(.step) | map({step:.[0].step, records:length, maxTotalMs:(map(.ms)|max), totalCalls:(map(.calls)|add)})'
  cli-git-worktree-add-timing.jsonl`,
  and the same over `.work` for the tools.
- Window:
  2026-11-03T08:55:46.633Z (release 0.5.20) to 2026-11-03T15:50:52.634Z (release 0.5.27).
  Count:
  53 records,
  44 with `work`.
- Git steps,
  whole time of one record's calls of that step (every one of these adds ended with exit code 0):
  `git status` 28 records,
  28 calls,
  maximum 59.9 s (one call);
  `git ls-files` 53 records,
  119 calls,
  maximum 59.9 s
  for three
  calls (19.98 s a call at most on average);
  `git worktree` maximum 0.33 s;
  `git diff-files` maximum 29 ms;
  `git
  diff-index`
  maximum 7 ms;
  `git check-ignore` 2 ms.
  No quick-class step appears:
  the quick steps run in process on those paths.
  The slow steps reported by the
  coordinator on the
  commit path (`diff-index --cached --quiet HEAD` over 50 s,
  `git add` over 50 s,
  `read-tree --reset` over 20 s) are
  in no log.
- Tools,
  whole time:
  the `repair` step of the add has median 26.2 s,
  p90 82.1 s and maximum 117.7 s over 53 records;
  `repairCargo` maximum 117.7 s,
  `repairPnpm` 45.8 s,
  `repairToolchain` 28.5 s;
  one `cargo clean` averages at most
  26.2 s a call
  and one `cargo metadata` of a worktree 2.9 s.
- Sampling:
  a healthy bulk step took at most 59.9 s whole,
  one fifth of the 300 s window,
  and a healthy tool at most
  117.7 s whole,
  one fifth of the 600 s window.
  Those are upper bounds of the gaps,
  and the gaps may be far shorter.

The Markdown linter and the review call have no log of their own (the audit's 11 ms warm and 227 ms cold,
and 1.4 s,
are whole
times from other documents).

What now collects the gap:
every Git step the wrapper starts,
every read-only query,
the object reader,
every package
tool,
every linter run and the gateway call note their longest quiet gap in `src/native/quiet_ledger.rs`,
and the commit
timing record
writes `longestQuietMs` and `quietMs` and the add timing record writes `quiet` when the command ends.
A step that was
ended for
being quiet is listed apart under a name ending in ` [ended for being quiet]` and left out of `longestQuietMs`,
so the
figures
are of healthy runs.
The resolution is the 500 millisecond sampling interval of the meter.
A real wrapped commit under the controls already writes figures,
for example `git add` 12.9 ms,
`git cat-file --batch`
14.3 ms
and `git version` 15.2 ms (`binary_commit_timing_tests.rs`,
the record printed by
`a_wrapped_commit_leaves_one_record_and_the_service_one_line_for_it`).
The commit service's landing steps run in the service process,
so the `answered` line of each request in its journal
ends
with `quiet_ms` and `quiet_steps` (the ledger is shared,
so the first answer after a batch holds the batch's figures).
The slow steps the design record names (`fast-import`,
`update-ref`,
the follow-up `diff-tree` and `reset`) are
therefore
collected in the service's journal and not in the client's record,
which shows only trivial gaps.

The gateway call's other transport timeouts default to none in the locked `ureq` 3.5.0 (`Timeouts::default` in its
`src/config.rs`:
`global`,
`per_call`,
`send_body`,
`recv_response` and `recv_body` are `None`;
only the one second
wait for a
`100 Continue` is set,
which the call never sends),
so no hidden total deadline remains.

Options to collect,
with the coordinator's ranking (for the coordinator to pick):

- A.
  Keep the labelled windows and collect from ordinary use,
  healthy runs only,
  then set each class's window above
  the longest
  healthy gap by a margin the owner approves,
  recording the file,
  the count and the margin beside the constant.
  Pro:
  real hosts and real loads,
  and the margin is the owner's.
  Con:
  slow,
  and a rarely quiet step may not appear for
  a long time.
- B.
  Collect on purpose:
  time the slow steps on a stat-dirty large checkout during the work the host already does,
  with the
  new records on.
  Pro:
  fast.
  Con:
  a fixture's gap is the fixture's,
  and the synthetic-load ban rules out making the load.
- C.
  Set the windows now to the largest whole time seen (59.9 s for a bulk step,
  117.7 s for a tool).
  Pro:
  no margin invented.
  Con:
  it reads a whole time as a gap,
  so it is a guess,
  and it would cut a quick step's
  window to a
  figure no quick step has ever shown.

Ranking:
A > B > C,
because A is the only one whose figure is a quiet gap on the person's own hosts and loads with the
owner's
margin;
B > C because B at least measures gaps,
while C relabels elapsed time as quiet time.

Picked (coordinator,
2026-11-03,
under the owner's delegation,
veto open):
A,
collect from ordinary use;
the margin above the longest healthy gap is the owner's to approve on return.
Also picked for the same branch's open items:
a quiet step that is safe to run twice is redone once (R1,
the owner's "redone";
a hang that repeats on the same input
goes to the embedded path of the commits-never-fail design);
`update-ref` stays out of the ended set until the eighth design review confirms how a killed child's lock is handled;
the review's local log write is left without a judge.

### Follow-up measurements

All on 2026-11-03 on this host,
under the load there was.
Probe code was scratch and is not committed;
the throwaway targets were a fixture repository,
a service started by the
fixture,
and reflinked copies of `git archive HEAD` of this repository (20,768 files,
22,717 entries) under
`${HOME}/temp/agent/limits`.
Every run was in `systemd-run --user --scope` with `IOWeight=10` and a `CPUQuota`,
`MemoryMax` and `TasksMax` sized to
it
(burst:
800%,
8G,
1500 and 3000 tasks;
thresholds:
1800%,
12G,
600 tasks).
The claim-kill run's scope had ended by the time its tests ran,
so it was bounded only by the test's own five
minutes;
it ran one fixture service and sleeping hooks.

#### Batch burst

- Command:
  the native binary test harness (`cargo nextest --test native_binary`) with a scratch test that starts the
  real
  `cli-git-service` on an abstract socket in a fixture repository,
  prepares N commit requests,
  releases N client threads
  at one barrier,
  and reads the service's `answered` lines;
  `MAX_BATCH` was raised to 4096 for the second run.
- Old cap of 64,
  three rounds alternating order over 8,
  16,
  32,
  64 and 128 clients:
  makespan 139 to 149 ms for 8
  clients,
  145 to 146 for 16,
  150 to 159 for 32,
  155 to 159 for 64,
  243 to 249 for 128 (three landings).
  A landing took 62 to 85 ms for 3 to 63 requests.
- Cap of 4096,
  three rounds over 64,
  128,
  256,
  512 and 1024 clients:
  makespan 163 to 171 ms for 64,
  176 to 206 for 128,
  216 to 218 for 256,
  313 to 331 for 512 (once 2,119 ms,
  a host stall),
  673 to 697 for 1024;
  landing times as in the
  entry
  "Batch size,
  64".
  All requests landed in every run.
- The probes were a debug build.

#### Claim recovery

- Command:
  the native binary test harness with a scratch test (three tests,
  2026-11-03):
  a fixture repository with a
  bare
  remote whose `pre-receive` hook sleeps 20 s;
  a push item is recorded and the service woken;
  when the item appears in
  `claimed`,
  1.5 s pass,
  the service is killed with SIGKILL together with its process group,
  the hook is removed,
  a new
  service is started and woken,
  and the time until the item is in `done` is read.
- Result:
  119.57,
  119.56 and 119.55 s;
  the lease is 120 s.

#### Thresholds, depth, batch and index rewrite

- Command:
  scratch probes inside the crate that call `copy_manifest_entries`,
  `install_snapshot`,
  `clone_tree`,
  `remove_tree_split` and `refresh_index_file` with the worker count,
  the batch size or the depth forced,
  five runs per
  cell in alternating orders,
  medians;
  destinations on the same btrfs volume as the sources,
  removed after each run.
- Copy stage,
  milliseconds at 1,
  2,
  4 and 8 workers:
  30 entries 1.1,
  1.0,
  0.7,
  0.7;
  90 entries 3.6,
  2.8,
  2.1,
  1.9;
  296
  entries
  13.4,
  9.3,
  7.2,
  5.9;
  1,077 entries 47.9,
  31.5,
  20.6,
  15.7;
  2,119 entries 91.4,
  53.9,
  32.3,
  25.4;
  4,722 entries 202.6,
  121.0,
  76.1,
  57.3;
  9,301 entries 397.6,
  222.2,
  139.4,
  104.4.
- Install,
  same columns:
  30 entries 1.6,
  1.4,
  1.4,
  1.3;
  90 entries 5.0,
  3.6,
  3.3,
  3.1;
  296 entries 14.8,
  11.1,
  9.1,
  8.9;
  1,077
  entries 54.7,
  37.6,
  31.3,
  25.9;
  2,119 entries 107.7,
  71.2,
  56.8,
  49.2;
  4,722 entries 254.7,
  182.5,
  144.1,
  129.7;
  9,301
  entries 500.2,
  349.3,
  277.6,
  244.9.
- `clone_tree`,
  same columns:
  6 entries 0.4 on all;
  10 entries 0.6,
  0.5,
  0.6,
  0.6;
  16 entries 1.1,
  0.8,
  0.7,
  0.7;
  30
  entries
  1.8,
  1.5,
  1.2,
  1.2;
  56 entries 3.5,
  2.5,
  2.2,
  2.0;
  90 entries 5.9,
  3.9,
  3.1,
  2.7;
  296 entries 20.7,
  13.7,
  9.5,
  8.1;
  555 entries
  39.2,
  23.4,
  16.4,
  13.4.
- Removal at depths 1 to 6,
  22,717 entries:
  365,
  281,
  304,
  325,
  437,
  512 ms (ranges 360 to 377,
  251 to 305,
  253 to
  309,
  307
  to 339,
  418 to 451,
  495 to 525).
- Install batches,
  22,717 entries,
  a recorder that appends and syncs twice per batch:
  6,542,
  2,265,
  983,
  746,
  688 and
  663 ms
  for batches of 64,
  256,
  1,024,
  4,096,
  16,384 and one batch;
  the ranges of the five runs were 6,301 to 7,925,
  2,064 to
  5,389,
  976 to 2,355,
  730 to 4,407,
  655 to 2,604 and 640 to 752 ms.
- Index rewrite,
  fresh process,
  resident size sampled every 0.2 ms:
  index 2,786,564 bytes grew 13,123,584 bytes (4.71
  times);
  index 1,066,894 bytes grew 7,929,856 bytes (7.43 times);
  later runs in the same process reuse freed memory and
  grew less.

### The lock ceiling and the three waits it fed (2026-11-08T16:01:42-05:00)

Item 5 of the queue audit asked for the measurement `LOCK_CEILING` never had,
and for the three waits once derived from it:
the merge,
the settling of earlier batches and the index reconciliation of a commit whose corrections removed every change.
By this date none of the three takes `LOCK_CEILING` any more:
each takes a ceiling of its own beside it in `src/native/service/wait_ceilings.rs`
(`MERGE_LOCK_CEILING`,
`SETTLE_LOCK_CEILING` and `RECONCILE_LOCK_CEILING`,
one minute each),
and the ten minutes remains only as the basis of `FOREIGN_OWNER_PATIENCE`
(`src/native/process/pid_namespace.rs`).

- Command:
  read-only reads of both generations of the commit timing log,
  `<common Git dir>/monochromatic-git-policy-cli-commit-timing.jsonl` and its `.1`
  (the file moves aside at one mebibyte and keeps one generation),
  by a scratch script under `${HOME}/temp/agent/lock-ceiling`,
  not committed.
  No journal was read for this measurement and nothing was written to the repository.
- Window and count:
  1,332 records from 2026-11-05T03:13:38Z to 2026-11-08T20:59:20Z,
  from 102 worktrees and releases 0.5.34 to 0.6.1-alpha.4:
  1,307 landed,
  23 stopped and 2 ended.
  The file grows as commits land,
  so a later read counts more;
  anything before 2026-11-05T03:13:38Z was replaced at the rotation and is not in the sample.
- Whole commands (`totalMs`,
  milliseconds),
  1,332 records:
  p50 1,214,
  p90 13,300,
  p99 58,782,
  longest 1,228,797.
  The answer wait (`answerWaitMs`,
  which holds the queue,
  another program's hold on the index and the landing):
  p50 109,
  p99 14,875,
  longest 1,200,776.
  The preparation (`prepareMs`,
  which holds the reconciliation when it runs):
  p50 379,
  p99 21,878,
  longest 230,525.
  The work after the landing (`afterLandingMs`):
  at most 7,856.
- The waits for another program's hold on the index,
  the population these ceilings are about:
  six records hold a waiting notice.
  Two landed,
  after 3,636 and 23,783 ms of answer time.
  Five of the six ended with a notice whose holder kind was `unproven`,
  the sixth with one that named no kind,
  and no notice in the sample ever proved a holder alive.
  The other four are the commits that met the lock that never went on 2026-11-07:
  they stopped after 644,710,
  756,215,
  1,087,175 and 1,200,776 ms of answer wait,
  which is the 678 to 1,229 s the handover recorded when the same four are measured as whole commands
  (678,557 to 1,228,797 ms).
- Under load:
  the reading ran at load average 115.9 of 32 threads (`/proc/loadavg`),
  and the log itself says which hours were busy.
  Within one release,
  which is one binary on both sides:
  the two quietest hours of 0.6.1-alpha.1 (15 and 16 landed commits) had answer waits of 114 and 599 ms at the 99th percentile,
  against 2,890 to 9,086 ms in its five hours of 29 to 81;
  the quietest hour of 0.6.0 (13 records) had 3,892 ms,
  against 5,854 to 38,545 ms in its five hours of 46 to 182.
  So the busy hours carry the long tail,
  and its size is the spread between one busy hour and another:
  the 182-commit hour of 0.6.0 has 11,975 ms at the 99th percentile,
  below the 38,545 ms of the 116-commit hour beside it,
  where the longest landed answer wait of the sample (46,061 ms) is.
  Grouping instead by how many other commits of this repository started within a minute of each landed record
  (alone,
  1 to 3,
  4 to 9,
  10 or more) gives 23,783,
  11,842,
  7,895 and 5,918 ms at the 99th percentile,
  an order that runs the other way.
  The host was loaded by the measure above,
  and both groupings show that in the tail;
  neither separates a load effect on any one figure from the spread between comparable hours.
- What the log cannot measure:
  the merge and the settling of earlier batches write no record at all.
  A merge is forwarded and never reaches the commit command's timing (`CommitTiming::start` is in
  `src/native/commit/commit_command.rs` alone),
  and 278 merge commits were made in the window without one;
  the settling is the management command `service settle-earlier-batches`.
  The reconciliation runs in the client's preparation,
  and no record of the window names the `git reset` it runs,
  so it did not run in this sample.
  Each of the three keeps the sample its own entry already names:
  the landing's journal waits for the merge's and the reconciliation's ceiling,
  and the journal's `settle_ms` for the settle's.
- What it says about the ten minutes:
  no wait that a holder released came near it.
  The largest self-released wait in the log is under 23,783 ms of answer time,
  and the file's five-times margin over the journal's 8 s is the one minute those ceilings take;
  ten minutes is 25 times the log's largest self-released wait.
  The four waits that passed a minute are the ones where nothing proved a live holder:
  the longest is 1,200,776 ms,
  776 ms past the client's twenty-minute `ANSWER_CEILING`,
  which bounds the whole wait rather than any one step of it.
- Figure this measurement would pick,
  recorded and not applied,
  because the owner decides limits case by case:
  one minute for the merge's wait,
  which is the value shipped;
  two minutes if the file's five-times margin were applied to the log's 23,783 ms instead of the journal's 8 s,
  which that figure does not justify,
  since it is the whole answer wait and not a measured wait for a lock.
  Twice the merge's wait would make `FOREIGN_OWNER_PATIENCE` two minutes rather than the twenty it ships;
  that constant is left as it is,
  because another change owns it,
  and its stated basis no longer holds.
- Limit of the data:
  one repository on one host,
  one generation of the log,
  and five of the six waits are the same incident.

### The lock ceilings' waits read again from the service journal (2026-11-08T18:43-05:00)

Item 5 of the queue audit asked for the measurement `LOCK_CEILING` never had.
The entry "The lock ceiling and the three waits it fed" answered it from the commit timing log alone.
This read adds the commit service's journal,
and gives every wait of that population with its count,
median,
95th percentile,
99th percentile and maximum,
so the citation beside each value in `src/native/service/wait_ceilings.rs` states a sample size and a spread.
No value changed.

- Command:
  read-only reads of both generations of the commit timing log
  (`<common Git dir>/monochromatic-git-policy-cli-commit-timing.jsonl` and its `.1`)
  and of the service's journal,
  by one scratch script under `${HOME}/temp/agent/lock-ceiling-journal`,
  not committed.
  The journal was taken with
  `journalctl --user -t cli-git-service -t monochromatic-git-policy-cli-service --since 2026-10-14`.
  Both identifiers are needed because the program was renamed:
  a read filtered to the current one alone sees 1,429 of the 4,386 answers and understates every count.
  Nothing was written to the repository.
- Window and count:
  1,608 timing records from 2026-11-05T03:13:38Z to 2026-11-08T23:49:56Z,
  from 134 worktrees and releases 0.5.34 to 0.6.1-alpha.5:
  1,564 landed,
  41 stopped and 3 ended.
  The journal answers 4,386 requests from 2026-11-02T04:01:38-05:00 to 2026-11-08T18:49:55-05:00:
  4,348 landed,
  32 answered `branch-moved`,
  1 was `already` and 5 failed with `index-in-use`.
  The timing log rotates at one mebibyte and keeps one generation,
  so anything before 2026-11-05T03:13:38Z has left it;
  the journal holds the landings of the same window plus the two days before.
- Percentiles are the nearest rank of the samples a wait had.
  The read ran at load average 63.2 on 32 CPUs.
- The whole command (`totalMs`,
  1,608 records):
  median 1,244 ms,
  95th percentile 27,159 ms,
  99th percentile 60,615 ms,
  longest 1,228,797 ms.
  The answer wait (`answerWaitMs`,
  1,608 records):
  median 103 ms,
  95th percentile 3,924 ms,
  99th percentile 14,875 ms,
  longest 1,200,776 ms.
  Six records hold a waiting notice for another program's hold on the index:
  two landed (3,636 and 23,783 ms of answer time)
  and four stopped at the lock that never went (644,710,
  756,215,
  1,087,175 and 1,200,776 ms).
  Five name the holder kind `unproven` and the sixth names none;
  no record holds a queued position.
  The preparation (`prepareMs`,
  1,608 records):
  median 391.5 ms,
  95th percentile 7,226 ms,
  99th percentile 20,078 ms,
  longest 230,525 ms.
  The build (`buildMs`):
  median 107 ms,
  95th percentile 11,134 ms,
  99th percentile 36,685 ms,
  longest 493,699 ms.
  The policy gate (`gateMs`):
  median 26 ms,
  95th percentile 694 ms,
  99th percentile 4,001 ms,
  longest 8,687 ms.
  The work after the landing (`afterLandingMs`):
  zero in 1,595 of the 1,608 records and 7,856 ms at most.
- The landing's wait for the index (`lock_wait_ms`,
  4,386 answers):
  3,777 are zero,
  median 0 ms,
  95th percentile 6 ms,
  99th percentile 158 ms,
  longest 600,718 ms.
  609 landings waited at all and 20 waited more than a second.
  Fourteen of the twenty landed,
  at 1,034,
  1,110,
  1,132,
  1,230,
  1,389,
  1,389,
  1,516,
  1,915,
  2,208,
  2,483,
  5,629,
  8,000 and 11,490 ms;
  one answered `branch-moved` after 4,611 ms;
  one more of 497,803 ms landed only when a person removed the lock;
  and five were refused after 600,090,
  600,098,
  600,164,
  600,395 and 600,718 ms.
  The field holds the wrapper's own staging of the index beside the wait for the holder:
  `lock_stage_ms` is above zero in 124 landings,
  and 9,467 of the 11,490 ms are the staging of a cold index.
- The wait for the holder's place (`lock_place_ms`,
  4,186 answers):
  4,115 are zero,
  median 0 ms,
  95th percentile 0 ms,
  99th percentile 4 ms,
  longest 600,132 ms,
  and 71 landings are above zero.
  The largest place wait that ended when the lock was released is 8,000 ms (2026-11-02T23:59:30-05:00;
  the line names no holder,
  so it does not say whether that was this worktree's own service landing or another program),
  then 2,022,
  1,389,
  798,
  341,
  265,
  262 and 258 ms.
  Above it are the 497,482 ms that a person ended by removing the lock
  and the five refusals at 600,090,
  600,095,
  600,098,
  600,109 and 600,132 ms.
- The service's own hold of the index (`lock_hold_ms`,
  3,769 answers):
  median 34 ms,
  95th percentile 328 ms,
  99th percentile 1,844 ms,
  longest 119,721 ms.
  The queue (`queued_ms`,
  4,386 answers):
  median 0 ms,
  95th percentile 0 ms,
  99th percentile 41 ms,
  longest 580,364 ms (the third of the commits behind the never-went lock).
  The landing itself (`landing_ms`,
  4,386 answers):
  median 64 ms,
  95th percentile 1,449 ms,
  99th percentile 11,808 ms,
  longest 756,118 ms.
  The settle (`settle_ms`,
  4,386 answers):
  4,214 zero,
  172 above zero,
  median 0 ms,
  95th percentile 0 ms,
  99th percentile 17 ms,
  longest 172 ms.
- What could not be measured:
  a merge writes no record (the commit command writes every one of the 1,608),
  and no record names the `git reset` of the index reconciliation,
  so neither of those two waits has a sample of its own;
  no log names a PID namespace,
  so no hold of another namespace was observed;
  and the journal's oldest line is 2026-11-01T08:22:38-05:00,
  so a wait older than that has left it.
- What the values rest on after this read:
  `LANDING_LOCK_CEILING` one minute is 7.5 times the longest place wait that ended when the lock was
  released (8,000 ms) and 5.2 times the longest whole lock acquisition (11,490 ms),
  and both readings keep it.
  `MERGE_LOCK_CEILING` and `RECONCILE_LOCK_CEILING` have no sample of their own,
  and the borrowed population's longest self-released wait is an eighth of their one minute.
  `SETTLE_LOCK_CEILING` keeps its whole-minute rounding:
  five times the longest settle (172 ms) is 0.86 s.
  `LOCK_CEILING` ten minutes is 52 times the longest whole lock acquisition,
  and no wait takes it.
  `FOREIGN_OWNER_PATIENCE`,
  twice `LOCK_CEILING`,
  is ten times the longest service-side hold
  (119,721 ms).
  The longest whole command,
  1,228,797 ms,
  is 28,797 ms past it,
  but the log times a command and not a record's hold,
  so it does not show the patience is too short.
- Open,
  for the owner:
  the patience's stated basis is twice the longest wait of a merge,
  and the merge waits one minute now,
  which would make it two minutes rather than the twenty it ships.
  Another change owns that constant;
  nothing was changed here.

## Counts

The limit sections hold 73 limits (a grouped entry,
such as the three include-depth copies,
counts once).
These totals are for the audited commit;
picks 5 and 12 removed the push ceiling,
the low-speed abort and the LFS retry,
and picks 4 and 9 removed `QUERY_CEILING` and the review `BUDGET` and turned the other time limits of Git steps,
tools
and
checks into labelled unmeasured quiet windows (the review call's window moved from "cited,
with a caveat" to "none,
unmeasured",
because 1.4 s is a whole time),
which the totals below still count as they were.

- cited: 4.
  The conversion storage ceiling (`src/native/worktree_convert_command.rs:76`),
  the conversion memory estimate
(`worktree_convert_command.rs:62`),
  the `checkout.workers=0` setting (`src/native/checkout_workers.rs:3` to `5`),
  and
  the review gateway call deadline (`src/native/service_review.rs:26`,
  with the caveat in its entry).
- partial: 7.
The quick and bulk child bounds,
  the push stall window,
  the batch size,
  the Markdown linter deadline,
  the saved-copies
  age and size defaults,
  and the depth and loop bounds of the readers.
- decided to be removed,
  removal unmerged:
  4 (the four unit caps).
- none:
  58,
  one of which (`DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS`) is a dead constant.

So 11 of 73 have a measurement or an owner decision beside them in some form,
and 4 of those 11 determine the value.

### After the follow-up picks

- Changed:
  lock age margin (4 s to 7 s) and with it the boot proof slack (3604 s to 3607 s);
  batch size (64 to 256);
  the three parallel thresholds of the copy stage,
  the install and the clone (4096,
  1024 and 512 to 16);
  `MOST_CONCURRENT_QUERIES` split into local (8) and remote (4).
- Newly cited,
  value kept:
  claim lease,
  install batch,
  removal depth,
  index rewrite copies.
- Unchanged by the pick:
  idle exit,
  foreign-owner patience,
  free-space preflight.

### After the delegated picks

Counted by entry as in the lists under the `Counts` heading;
the first count is unchanged and describes main at the audited commit.

- Newly cited,
  value kept:
  foreign-owner patience,
  idle exit,
  frame and change bounds,
  journal retention and capture
  memory,
  stale temporary and scratch ages,
  auto-push walk budget,
  the copy worker cap of eight,
  the empty-lock settle
  time and the clock and reuse slacks,
  `QUERY_MEMORY`.
- Newly cited,
  value changed:
  lock age margin (2 s to 4 s),
  boot proof slack (3600 s to 3604 s),
  `REPOSITORY_MEMORY` (64
  to 128 MiB),
  `ENTRY_MEMORY` (64 to 128 MiB),
  and the free-space preflight (5 GiB to 2048 bytes an entry).
- Classed exempt:
  the 35 constants of "Exempt limits".
- Data cannot support a value:
  batch size 64,
  claim lease 120 s.
- Decision or measurement still missing,
  kept as they were:
  the copy-stage and install thresholds,
  `REMOVAL_SPLIT_DEPTH`,
  `INSTALL_BATCH_ENTRIES`,
  `INDEX_REWRITE_COPIES`,
  `SLICE_MEMORY` (measured to be conservative),
  and
  `MOST_CONCURRENT_QUERIES`.
- Owned by other agents and unchanged:
  everything under pushes and LFS,
  saved copies and log history,
  Git child steps
  and tool deadlines,
  the index-lock holder wait and the service waits.
- Closed after that count (2026-11-08):
  the four ceilings of the foreign-holder waits are cited,
  `LOCK_CEILING` 10 minutes,
  `MERGE_LOCK_CEILING`,
  `SETTLE_LOCK_CEILING` and `RECONCILE_LOCK_CEILING` one minute each,
  and no wait takes the ten minutes any more;
  the measurement is "The lock ceiling and the three waits it fed" in the measurement record.

### Uncited limits

Each line is a limit with no citation (class none) or only a partial one.

- Live-holder wait 1800 s,
  `wait_limit.rs:26`.
- Service hold age limit 60 s,
  `service_index_wait.rs:48`.
- Lock-wait look counts and pauses (5 x 2 ms,
  5 x 10 ms,
  50 ms),
  `service_index_wait.rs:33`,
  `service_index_hold.rs:27`,
  `index_holder_wait.rs:41`.
- Service scan budget 250 ms,
  `index_lock_holders.rs:293`.
- Index reconciliation tries 5 x 100 ms,
  `service_land_follow.rs:24`.
- Forwarded index writer reruns 50,
  `service_hold_retry.rs:28`.
- Foreign-owner patience 3600 s,
  `pid_namespace.rs:19` (marked an open value call for the owner).
- Notice intervals and graces 10 s,
  1 s,
  50 ms,
  `wait_progress.rs:33`.
- Retired unproven-owner timeout 1000 ms (dead),
  `config_schema.rs:41`.
- Quick child bound 60 s and bulk child bound 300 s (partial),
  `child_class.rs:34`,
  `child_class.rs:39`.
- Class notice graces 5 s and 10 s,
  `child_class.rs:42`.
- Refused-start retry 5 s,
  `start_retry.rs:27`.
- Signal guard wait 1200 x 50 ms,
  `commit_preparation.rs:148`.
- Landing rebuilds 3,
  `service_land.rs:31`;
  client rebuilds 5,
  `service_commit.rs:60`;
  merge rounds 20,
  `merge_isolation.rs:136`;
  reconcile attempts 10,
  `auto_push_reconcile.rs:41`;
  reference-lock push retries 3,
  `auto_push.rs:46`;
  claim rounds 64,
  `single_owner_claim.rs:44`;
  work item tries 3,
  `service_work.rs:46`.
- Service start wait 10 s,
  `service_client.rs:28`;
  commit queue wait 600 s,
  `service_client.rs:34`;
  reconnects 5,
  `service_client.rs:37`;
  request read timeout 10 s,
  `service_server.rs:44`;
  idle exit 3 h,
  `service_server.rs:48`;
  batch size 64 (partial),
  `service_server.rs:41`;
  frame 1 MiB and 4096 changes,
  `service_protocol.rs:27`.
- Claim lease 120 s,
  `service_work.rs:335`;
  journal retention 1024 and 128,
  `service_journal.rs:258`,
  `service_captures.rs:25`;
  temporary and scratch ages 10 min and 24 h,
  `service_journal.rs:33`,
  `work_item_recovery.rs:29`,
  `scratch_location.rs:46`.
- Retirement wait 10 s,
  `service_provision.rs:86`;
  manager exchange 20 s,
  `systemd_dbus.rs:32`.
- Push stall window 30 s (partial,
  now only for a tree with no open connection),
  `bounded_push.rs`;
  push output 4 MiB,
  `bounded_push.rs`;
  stop grace 5 s (partial),
  `bounded_push.rs`;
  query ceiling 120 s,
  `bounded_query.rs:35`;
  push retry pauses 10 s to 600 s,
  `push_retry.rs:35`.
  Removed by pick 5:
  the push ceiling 1800 s and the low-speed abort.
- LFS chunking,
  relay and scan,
  `lfs_upload_plan.rs:57`,
  `lfs_pointer_scan.rs:28`;
  auto-push walk budget 4096,
  `auto_push_trigger.rs:42`;
  concurrent queries 4,
  `manual_push_probe.rs:133`.
- Markdown linter deadline 60 s (partial),
  `markdown_linter.rs:64`;
  Markdown output bounds,
  `markdown_linter_output.rs:37`;
  convergence passes 8,
  `policy_convergence.rs:39`;
  review script and whole-review deadlines,
  `service_review.rs:31`;
  review input caps,
  `standards_review_input.rs:54`;
  package repair tool limit 600 s,
  `package_repair.rs:36`;
  configuration file 1 MiB,
  `config_file.rs:83`.
- Copy thread cap 8 and thresholds,
  `worktree_copy_stage.rs:232`;
  install batch 4096,
  `worktree_copy_install.rs:38`;
free space 5 GiB,
  `subvolume_preflight.rs:26`;
  changing-file copy attempts 3,
  `subvolume_sync.rs:43`;
  parallel budget
  divisors and memory estimates,
  `parallel_budget.rs:17`.
- Saved copies 30 days and 1 GiB (partial),
  `discarded_prune.rs:35`;
  audit rotation 1 MiB,
  one generation,
  `discard_audit.rs:28`;
  clean-check reads,
  `worktree_clean_content.rs:37`.
- Lock margins 2 s,
  1 h,
  1 s,
  `stale_ref_lock_judge.rs:24`,
  `stale_ref_lock_judge.rs:37`,
  `stopped_git_locks.rs:59`.
- Reader file sizes,
  depth and loop bounds (partial),
  chunking and display caps:
  `ref_files.rs:27`,
  `repo_read_*`,
  `candidate_store.rs:107`.

## Questions for the owner

Each question groups limits that share a cause.
I do not decide any of them;
the ranking is my reading of the options against the decisions quoted in "Owner decisions
that bear on limits".
Options that are complements (for example an interim and its replacement) can be chosen together,
in order.

### Which limits need a citation under LQM

Applied under the delegation:
option B;
see "Exempt limits".

Covers:
the pacing and display constants (polls of 10 ms to 100 ms,
notice intervals,
argument chunk sizes,
truncation
caps,
journald field caps),
the reader bounds that make the wrapper fall back to Git,
and the format and protocol
constants already left out of this list.
`LQM` says "unit caps,
timeouts,
size bounds" and does not say whether a 50 ms poll or a 2048-path chunk is one.

- A.
  Every listed limit cites a measurement or an owner decision.
  Visible result:
  roughly 58 entries gain a sentence;
  polls and truncation caps get a measured or decided value,
  which
  costs measurements of constants that change nothing a person sees.
- B.
  Only limits that can change a command's result or its duration by more than a stated slop need a citation;
  polls,
  display caps,
  chunk sizes and reader fallbacks are classed once,
  in one place,
  as exempt with the reason.
  Visible result:
  about half the entries need evidence;
  the exempt ones carry a one-line classification.
- C.
  `LQM` stays as worded and whoever writes a constant decides case by case whether it is a limit.
  Visible result:
  no extra work now,
  and the same gap that let the unit caps through remains for borderline constants.

Ranking:
B > A > C,
because B spends measurement where the unit incident happened (a bound that changed what a person
saw) and A spends the same effort on constants whose value no person can observe;
and A > C because C is the
arrangement
under which an unmeasured cap was written and merged.

### What a live or unproven index lock holder costs

Covers:
`DEFAULT_LIVE_WAIT` 1800 s (`wait_limit.rs:26`) for the merge,
the forwarded writers and the service's
landing;
the
same 1800 s for an owner that cannot be proven alive;
`service_hold_retry::ATTEMPTS` 50.
The 5 x 100 ms index reconciliation of a no-change commit is a minor sibling and follows whichever option is chosen.
The decision record says an owner proven alive gets "an unbounded wait with one line naming the holder" and an
unproven
one gets backoff up to 1000 ms;
the code does neither,
and the doctor says the 1000 ms key "does nothing".

- A.
  Follow the record:
  a proven-alive holder is waited for without a bound,
  with the line naming it;
  an unproven or
  dead
  owner gets 1000 ms of backoff and then a diagnostic that lists the evidence.
  Visible result:
  a merge or commit behind a live program waits as long as the program holds the lock,
  repeating the
holder's line every notice interval,
  and never prints "giving up after 1800 seconds";
  behind a stale lock the person
  gets
  the diagnostic after a second instead of thirty minutes of silence.
  The record then needs no amendment.
- B.
  Build the owner's answer "Same as the ref lock (Recommended)":
  a commit lands and exits 0 and the index catches
  up
  when the lock is free;
  a merge saves its result on a side ref and finishes the checkout later.
Visible result:
  no wait line and no `IndexInUse` failure for a live foreign lock;
  the command exits 0 with a line
  saying
  the index will catch up.
  A is the interim and B the replacement,
  so "A,
  then B" is reachable.
- C.
  Keep a bounded wait and cite its number (the owner names it,
  or a measurement of how long live holders hold the
  lock
  on this host).
  Visible result:
  unchanged messages,
  plus a stated reason for the number;
  the commit still fails at the bound.
  The decision record is amended to say so.

Ranking:
B > A > C,
because B removes the failure the owner calls a design defect and the wait with it,
while A
removes
the failure but still waits ("You should (almost) never be waiting.");
and A > C because C leaves a commit failing at
a
number for a cause (another program's lock) that the owner put on the designed-out list.

Outcome (delegated pick 2,
option A,
built on `feat/git-policy-cli-index-lock-and-service-waits`;
the coordinator picked while
the owner was away,
open to the owner's veto):
a holder proven alive is waited for with no bound and a line that names it;
an owner that cannot be proven alive first got
a measured 1000 ms and a diagnostic,
and now gets the same unbounded wait (the coordinator's follow-up pick,
below);
a dead owner
gets the diagnostic at once;
`CLI_GIT_LOCK_WAIT_SECONDS`,
`DEFAULT_LIVE_WAIT` and `src/native/wait_limit.rs` are gone.
Option B ("Same as the ref lock (Recommended)":
a commit lands and exits 0,
the index catches up when the lock frees)
is not
built;
it comes with the commits-never-fail design.

Open questions this pick raised,
for the owner:

- Resolved by the coordinator's follow-up pick (2026-11-03,
  open to the owner's veto):
  the backoff is gone,
  and an owner
  nothing proves alive is waited for like a proven one.
  This is option c of the question that stood here,
  which ranked it below
  keeping 1000 ms;
  the coordinator picked it because an editor-held `git commit` would otherwise make ours fail after
  a second.
  A leftover from a killed program now waits until a person removes the lock;
  the repeated line names the file.
- Should a dead owner wait like any other owner,
  as the record says?
  It ends at once,
  because the PID file names a process that cannot let go.
  Options:
  (a) at once (built);
  (b)
  wait like the
  others,
  as the record says.
  Ranking:
  a > b,
  because b would wait for nothing,
  with no end.
- Is 150 seconds the right age for the service's own hold when its holder cannot be asked about?
  (Awaiting the owner's
  approval;
  the coordinator did not pick it.)
  Pick 2 said "short backoff";
  the measurement (live holds up to 119.7 s)
  says a short age takes a live service for a leftover.
  Options:
  (a) 150 seconds (built);
  (b) 1000 ms like the foreign owner;
  (c) no age at all,
  wait while the lock carries the
  service's mark.
  Ranking:
  a > c > b,
  because a is cited and still ends a dead service's leftover;
  c never ends it;
  b would
  refuse a `git add` during any landing longer than a second from a container.
- `service_hold_retry::ATTEMPTS` (50 starts of a forwarded index writer while the service takes the index first)
  is unchanged and
  still uncited.
- Resolved by the coordinator's follow-up pick (2026-11-03,
  open to the owner's veto):
  where no inotify watch can be set,
  the
  waits poll every `POLL_PAUSE` instead of failing the commit or running the command into Git's refusal.

### Retry counts for moved branches and contended locks

Covers:
`LANDING_ATTEMPTS` 3,
`REBUILD_ATTEMPTS` 5,
merge `MAX_ROUNDS` 20,
`MAX_RECONCILE_ATTEMPTS` 10,
`PUSH_LOCK_RETRIES` 3.
Their causes (other commits moving the branch,
a sibling worktree's service holding a ref lock,
the remote advancing)
are
not on the owner's "outside our control" list.
"Ship interim now" covers the moved-head loop in general,
not these numbers.

- A.
  Keep them as interim loops until the by-construction design is built,
  and write a measured number beside each
  (for example the most rounds seen when K committers hit one branch).
Visible result:
  the same failure texts,
  now with a recorded basis;
  "The branch ... changed 3 times" still appears
  under
  enough contention.
- B.
  Remove the counts:
  repeat on the new head until it stops moving,
  naming each round and the cause.
Visible result:
  no "changed N times" failure and no "merges in a row" stop;
  under a branch that never rests the
  command
  keeps running and prints a line per round.
- C.
  Extend the owner's "Stack on the new head (Recommended)",
  which names an amend,
  a conclusion and a revert,
  to
  every
  moved branch:
  the commit or merge is built on the head as it is when the ref moves,
  so there is no round to count.
  Visible result:
  no retry,
  no count,
  and no failure for a moved branch.

Ranking:
C > B > A,
because C designs the failure out,
as "never fail doesn't primarily mean retries!"
asks,
and B
removes
the failure but keeps a loop whose end depends on the other writers;
and B > A because A still fails a commit or merge
at a
number the owner has not seen.

### Time limits on Git child steps of a commit

Covers:
`PLUMBING_CHILD_WAIT` 60 s and `OBJECT_CHILD_WAIT` 300 s (`child_class.rs:34`,
`39`),
the override
`CLI_GIT_CHILD_WAIT_SECONDS`,
the signal guard wait,
the refused-start retry of 5 s.
The commit-timeouts record measured the steps on one 20,207-file clone at load 80 to 145 (worst quick step 44 ms;
stat-dirty
bulk steps 2.8 to 5.8 s),
which does not produce 60 or 300.
A killed step fails the commit,
and "Commits should never fail" and "Git's own crashes or bugs" are designed out.

- A.
  Notice only:
  no kill;
  the step is waited for with the "still waiting" lines until it ends or the person
  interrupts.
  Visible result:
  no "did not finish within N seconds" failure;
  a truly hung Git holds the commit and the service lane
  until someone acts.
- B.
  Kill only on no progress,
  with the same signs the push uses (bytes moved,
  processor time,
  a task running or in
  disk wait),
  and no wall-clock bound.
Visible result:
  a hung child is stopped after a stall window and the commit fails with a message naming the stall;
  a
  slow
  but working step is never cut.
  A child wedged in the kernel counts as alive under those signs,
  so it is waited for.
- C.
  Keep 60 and 300 and write beside them a measurement taken under load (the largest step times on a stat-dirty
  large
  repository on a loaded host,
  and the factor used).
  Visible result:
  unchanged messages,
  with a basis;
  a step slower than the bound still fails the commit.

Ranking:
B > C > A,
because B ends real hangs without cutting slow work and uses a mechanism the push already has;
C >
A
because C keeps an end for a hang,
which A removes and thereby lets one stuck child hold every later commit.

Outcome under the delegation (the coordinator picked option B,
the owner may veto it):
implemented on
`feat/git-policy-cli-steps-end-only-on-no-progress`.
A Git child step is ended only on no progress,
judged with the push's signs,
with no wall-clock bound.
The windows (60 and 300 seconds,
`CLI_GIT_CHILD_WAIT_SECONDS`) are labelled unmeasured;
see the entries for the limits
and
"Quiet gaps in the existing logs".
Open for the coordinator,
with the owner's veto:
how often an ended step is redone.
- R1.
  Once (built):
  the owner said "redone";
  a deterministic hang still fails after two windows,
  and the person sees
  why.
- R2.
  Until it ends:
  nothing fails,
  but a deterministic hang loops with a notice each round and waits for the person.
- R3.
  Doubling the window each round (the earlier attempt on `fix/git-policy-cli-commits-never-fail`):
  invents a
  growth rule.
Ranking:
  R1 > R2 > R3,
  because R1 keeps "redone" and invents no growth rule,
  and still fails loudly;
  R2 > R3 because
R2 adds no number.
Also open:
  whether `update-ref` (a compare-and-swap,
  safe twice by the design record) joins the steps that are ended,
and
starting the object reader again after it was ended (not built;
  its request fails with the stall sentence).

### Push stall window, ceiling and Git's low-speed abort

Covers:
`DEFAULT_PUSH_DEADLINE` 30 s,
`DEFAULT_PUSH_CEILING` 1800 s,
`LOW_SPEED_LIMIT` 1000 and `LOW_SPEED_TIME` 60
(still set for
the push on main),
`MAX_PUSH_OUTPUT` 4 MiB,
`STOP_GRACE` 5 s,
`QUERY_CEILING` 120 s.
The owner chose "Wait while connected (Recommended)";
the branch that builds it keeps 30 s and 1800 s.
A slow remote is on the designed-out list.

- A.
  Adopt the decision whole:
  a push that has an open connection is never stopped for time;
  the stall window applies
  only to a
  push with no connection and no other sign of work;
  no ceiling;
  no low-speed abort for the push.
Visible result:
  no "limit=stall" or "limit=ceiling" for a slow server,
  one line naming the wait,
  and the push ends
  when the
  server answers or the connection drops.
- B.
  Merge the branch as it is and cite the two numbers with a measurement (a push to a deliberately slow remote that
  keeps the socket open).
Visible result:
  a slow server is waited for as long as bytes move on the socket and stopped at thirty seconds of
  silence
  or thirty minutes in all,
  with a cited basis.
- C.
  Keep main as it is.
Visible result:
  a hook or server check longer than thirty seconds still prints "made no progress for 30 seconds",
  and
  curl's "Operation too slow" can still end a push.

Ranking:
A > B > C,
because A is the owner's decision with no leftover number to cite;
B follows the decision for
progress
but leaves two numbers that end a connected push;
C contradicts the decision on main.

Delegated pick,
2026-11-03:
A,
built;
see the outcomes under "Pushes".
The coordinator picked it under the owner's delegation,
and the owner may veto it.
The stop grace,
output cap and query ceiling are memory and cleanup bounds and can be cited by measurement under any
option.

### Waits on the commit service itself

Covers:
`START_WAIT` 10 s,
`COMMIT_WAIT` 600 s,
`RETIREMENT_WAIT` 10 s,
`EXCHANGE_WAIT` 20 s,
`REQUEST_READ_TIMEOUT`
10 s,
`RECONNECTS` 5.
`DESIGN.md:124` says a slow start and a version mismatch are "resolved without the user".
The message for `COMMIT_WAIT` tells the person to "raise the waiting limit in the settings",
and no setting exists.

- A.
  No wall-clock client bound:
  heartbeats from the service while it works (the unbuilt design's "heartbeats instead
  of
  `COMMIT_WAIT`"),
  and a start that waits until the manager reports failure.
Visible result:
  no "waited N minutes ... not made" and no "did not start" for a slow start;
  position notices continue.
- B.
  Keep the bounds and cite them with measurements (a cold start of the service,
  the queue time at many committers:
  the
  commit-timeouts record measured makespans of up to about 17 s at 32 at once).
  Visible result:
  the same failures at measured numbers.
- C.
  Make `COMMIT_WAIT` a real setting,
  as the message says.
  Visible result:
  the message becomes true;
  the default is unchanged.
  C and B are complements.

Ranking:
A > B > C,
because A removes the failure the owner's rule says the system should resolve,
and B keeps it but
bases it on evidence;
B > C because C makes the message honest without giving the number a basis.

Outcome (delegated pick 6,
option A,
built on `feat/git-policy-cli-index-lock-and-service-waits`;
the coordinator picked while
the owner was away,
open to the owner's veto):
the client waits on heartbeats from the service and has no wall-clock bound of its own;
`COMMIT_WAIT` and the message that named a
setting that does not exist are gone.
A service that stops sending for five seconds is asked again with the same request identity;
one that then greets nothing ends the
command with "The commit service stopped answering while this commit waited ...
it is not known whether the commit was made".
The start waits without a clock where the client can ask the service manager,
and ends when the manager reports the unit
`failed` twice in a row.

Open questions this pick raised,
for the owner:

- Resolved by the coordinator's follow-up pick (2026-11-03,
  open to the owner's veto):
  after `START_WAIT`,
  a unit the manager
  reports `inactive` is asked to start (option b of the question that stood here).
- The beat proves the service process and the connection live,
  not that the landing thread makes progress.
  Options:
  (a) as built;
  (b) the landing thread increments a counter at each phase and the beat stops while it does
  not move.
  Ranking:
  a > b for now,
  because b needs a clock for "does not move",
  which the beat was chosen to avoid,
  and stuck Git children
  are ended by the step bounds of another branch.
- `EXCHANGE_WAIT` (20 s) and `RETIREMENT_WAIT` (10 s) are in this question's list and are unchanged here.
- A connection thread now lives through the whole queue and landing of its request,
  so the service's thread count during a
  batch is the queue depth plus the acceptors,
  and the 0.5.27 unit sets no `TasksMax`.
  It is not a limit this change ships.
- With a manager watch the read for the first greeting times out every second instead of at `START_WAIT`,
  so a slow
  socket-activated start leaves one idle backlog connection per second until the service greets;
  the service's acceptor drains
  them harmlessly.
- What the controls prove:
  the scripted listener of `binary_service_heartbeat_tests.rs` exercises the manager watch
  with a
  function standing in for the manager;
  no test calls the real D-Bus `unit_has_failed`,
  and only the `start_watch: None` path
  runs against a real `cli-git-service`.

### Foreign-owner patience

Applied under the delegation:
option C;
see the entry "Foreign-owner patience,
one hour".

Covers:
`FOREIGN_OWNER_PATIENCE` 3600 s (`pid_namespace.rs:19`).
`doc/planning/cli-git-spawn-and-sync-reduction.md:713` already calls it an open value call for the owner:
a fresh
ownerless
directory stops every guarded command for up to an hour,
and a shorter bound widens the window in which a live wrapper
whose record was removed by hand could lose its directory.

- A.
  Keep an hour as the owner's decision.
Visible result:
  unchanged;
  a stuck directory blocks guarded commands up to an hour with a message that names the
  manual
  way out.
- B.
  Tie it to the live-lock wait (30 minutes in the planning document's suggestion).
  Visible result:
  stuck directories clear in half the time;
  the risk window for a live idle wrapper doubles.
- C.
  Measure the longest time a live wrapper sits with no file open (from the service and timing logs) and set the
  patience
  from it.
  Visible result:
  a number with a basis;
  the block lasts as long as the longest observed idle.

Ranking:
C > B > A,
because both trade waiting against a risk to a live wrapper's data and C is the only option that
measures the risk;
B > A because "You should (almost) never be waiting."
favors the shorter bound once the risk is
stated.

### Parallelism caps and memory estimates

Applied under the delegation:
option A;
see the entries "Copy and clone thread caps and thresholds" and "Parallel
budget divisors and memory estimates",
and "Measurement record".

Covers:
copy threads 8,
install and clone thresholds,
`MOST_CONCURRENT_QUERIES` 4,
`CPU_MARGIN_DIVISOR` 8,
`MEMORY_SHARE_DIVISOR` 2,
`REPOSITORY_MEMORY` 64 MiB,
`QUERY_MEMORY` 32 MiB,
`SLICE_MEMORY` 16 MiB,
`ENTRY_MEMORY` 64
MiB.
With the unit caps gone,
these are the only numbers that size how many repositories the service drains at once.
The owner said "make sure we're running everything that can be run in parallel in parallel."
`CONVERSION_STORAGE_CEILING` and `CONVERSION_MEMORY` show the pattern the other numbers lack:
a measurement beside the
value.

- A.
  Measure each:
  scaling at 1,
  2,
  4,
  8,
  16 workers on a realistic repository for the thread caps,
  and the resident
  peak of
  one item for each memory estimate,
  and write the result beside the value.
  Visible result:
  the numbers may change;
  each has a basis;
  the add and the round may get faster or slower.
- B.
  Remove the caps and the divisors and use every thread and all the free memory.
  Visible result:
  the fastest on an idle host and the heaviest on a shared one;
  no basis exists for the choice either.
- C.
  Keep the numbers and label them estimates.
  Visible result:
  comments only;
  `LQM` is still not met.

Ranking:
A > B > C,
because A meets `LQM` and the owner's parallelism request with evidence;
B meets the request but
trades
one unmeasured number for another;
C does not meet `LQM`.

### Deadlines for tools and checks that are not Git

Covers:
`TOOL_LIMIT` 600 s for a package-manager tool during a worktree add,
`LINT_DEADLINE` 60 s,
the review
`SCRIPT_DEADLINE` 30 s and `BUDGET` 20 s.
The owner chose "Install online during the add",
so an install can legitimately run long.

- A.
  End a tool or check only when it shows no sign of work for a window (the push's signs);
  no wall clock.
Visible result:
  a long install is never reported as "ran past its time limit";
  a hung tool is stopped after the
  window.
- B.
  Keep the deadlines and measure them (a cold `pnpm install` of this repository,
  the slowest lint of a large
  Markdown file).
  Visible result:
  the same failures at numbers with a basis.
- C.
  Notice only.
  Visible result:
  a hung tool holds the add until the person interrupts it.

Ranking:
A > B > C,
because A never cuts work that is making progress,
and B cuts at a number that a heavier
repository
will exceed;
B > C because C lets a hung tool hold the add with no end ("a hung tool must not hold an add forever").

Amended by the owner 2026-11-04T09:35:19-05:00,
"Silence plus a ceiling (Recommended)" ("Keep ending on silence,
and add a generous
overall ceiling per tool,
measured from real runs,
after which it's ended with an error naming the tool.
Con:
an extremely slow but
working run on a huge repository can hit the ceiling."):
the four tools below also have a ceiling on their whole run now
(`REPAIR_TOOL_CEILING`,
`LINT_CEILING`,
`REVIEW_CALL_CEILING`,
`REVIEW_CEILING`;
see
`doc/planning/cli-git-no-unbounded-waits.md`,
"Branch write and tool ceilings,
2026-11-04T09:35:19-05:00"),
so the text of the
outcome below that says a tool "is never ended for being slow" is superseded.

Outcome under the delegation (the coordinator picked option A,
the owner may veto it):
implemented on
`feat/git-policy-cli-steps-end-only-on-no-progress`,
for the package repair tool,
the Markdown linter,
the gateway
call and the
whole review (the 20 second budget is removed),
with the read-only Git queries added by the coordinator.
The numbers (600,
60,
10 and 30 seconds) are labelled unmeasured quiet windows.
Open for the coordinator:
the local write of the review's log entry has no judge now;
a write blocked in the kernel is
waited
for.
Options:
leave it (the log is local and the write is small),
judge the review's thread with `thread_quiet.rs` (it
would
also judge the waits for Git children,
which have their own watch),
or bound the write with a quiet window of its own.
Ranking:
leave it > own window > judge the thread,
because the first adds nothing that can end a healthy review wrongly.

### Defaults for saved conflict copies

Covers:
`DEFAULT_MAX_AGE_DAYS` 30,
`DEFAULT_MAX_BYTES` 1 GiB,
`YOUNG_FLOOR_MILLISECONDS` 24 h
(`discarded_prune.rs:35`,
`38`,
`41`).
The rule ("Prune by both age and size.")
is the owner's;
the handover calls the numbers "The coordinator's defaults".
The pass deletes copies that are "the only remaining content of replaced files".

- A.
  Record the current defaults as the owner's decision.
  Visible result:
  nothing changes;
  the numbers are cited.
- B.
  The owner names the numbers.
  Visible result:
  defaults change to those numbers.
- C.
  Measure how large and how old the saved-copies folders get across the repositories in use,
  and set the defaults
  from
  the distribution.
  Visible result:
  defaults with a basis;
  possibly larger than 1 GiB if the data say a 30-day copy is routinely bigger.

Ranking:
C > B > A,
because the folder holds the last copy of a person's replaced file and only C bases the loss
threshold
on how the folder actually grows;
B > A because A approves numbers the owner has not seen.

Outcome under the delegation (the coordinator's,
open to the
owner's veto):
option C was attempted and the data are too thin.
The folder does not exist on this host,
there is no `saved-conflict` or `pruned` line,
and the window is about one day;
see the measurement in the entry for the limit.
No number was invented:
`DEFAULT_MAX_AGE_DAYS`,
`DEFAULT_MAX_BYTES` and `YOUNG_FLOOR_MILLISECONDS` are unchanged and
each carries the measurement beside it.

The measurement also confirms what the owner already knew when choosing the rule:
any saved copy can be the only copy of a person's work (see "Only-copy finding" in the entry for the limit),
which makes the numbers secondary.
Options for the coordinator to pick,
with the owner's veto open:

- O1.
  Suspend age and size pruning of saves until explicit disposal;
  measure growth meanwhile and re-enable pruning with numbers from the data.
  Visible result:
  nothing is ever removed;
  `git cli-git doctor` reports the folder's size so growth can be watched.
  Pro:
  cannot lose the only copy;
  costs almost nothing now,
  because zero saves exist;
  matches "never discard user data" and
  the adopted rule that saved copies of carried edits are kept until explicit disposal.
  Con:
  asks the owner to reverse an informed decision ("Prune by both age and size.")
  for now;
  a folder that grows has
  no bound and no disposal command exists yet.
- O2.
  Keep pruning,
  but never remove a save unless every file
  in it is byte-identical to a blob in the repository's object
  database.
  Visible result:
  only copies whose bytes were committed or staged elsewhere can be pruned.
  Pro:
  keeps the owner's rule for the copies that are not the only copy and keeps the only copies.
  Con:
  needs an object lookup per file that the wrapper does not do today;
  the only copies stay forever,
  so the folder
  has no bound either,
  and the same disposal command is missing.
- O3.
  Keep the rule and the numbers;
  measure for a stated window
  (for example until a number of saves exist),
  then set the
  numbers from the data.
  Visible result:
  nothing changes now.
  Pro:
  honest basis for the numbers and the owner's rule intact.
  Con:
  until then a save older than 30 days or past 1 GiB may be removed,
  and it may be the only copy.
- O4.
  Record the current numbers as the delegated pick now (the earlier option A).
  Visible result:
  nothing changes;
  the numbers are cited as the coordinator's choice.
  Pro:
  satisfies the rule's wording at once.
  Con:
  the numbers have no basis and still delete possibly-only copies.

Ranking:
O3 > O1 > O2 > O4,
because the owner chose pruning with the only-copy fact in front of them,
so under the
delegation standard ("what I would have picked most likely") O3 keeps that decision and gets the basis the question
asked for,
while O1 asks the owner to reverse an informed decision,
although it is the safer option for the data;
O1 > O2 because O1 loses nothing and needs no object lookup,
whereas O2 adds the lookup and still leaves the folder
unbounded;
O2 > O4 because O4 cites numbers with no basis and keeps deleting possibly-only copies.
O1 and O2 depart from the owner's recorded rule;
O3 and O4 keep it.
If the coordinator reads "never discard user data" as the stronger rule,
the order is O1 > O2 > O3 > O4.

### The audit and timing logs' history

Covers:
`ROTATE_AT_BYTES` 1 MiB in `discard_audit.rs` with one `.1` generation,
shared by `cli-git-audit.jsonl`,
`cli-git-commit-timing.jsonl`,
`cli-git-worktree-add-timing.jsonl`,
`cli-git-push-trigger.jsonl`.
Since 2026-11-10 that limit is `ROTATE_AT_BYTES` in
`foundation/log_lines_appended_without_a_lock.rs`,
and every log named here keeps numbered generations under
`LOG_HISTORY_BYTES`
(`foundation/log_generations_kept_to_a_bound.rs`),
as the outcome below records.
The next rotation replaces `.1`,
so older history is dropped;
the race fix on `fix/git-policy-cli-audit-rotation-race`
does not change that.
The audit file exists to attribute lost uncommitted work and each line names the saved copies it made.
The standards-review log had no rotation at all.
Does "never discard user data" cover this audit history?

- A.
  Keep one generation as today.
Visible result:
  nothing changes;
  after two rotations the earliest decisions and the names of the saved copies they
  made
  are gone,
  and a saved copy can remain with no line that names it.
- B.
  Keep every generation (`.1`,
  `.2`,
  and on).
  Visible result:
  history is complete;
  the common Git directory grows without a limit,
  at a rate nobody has measured.
- C.
  Keep generations up to a measured or owner-chosen total size,
  and never drop an audit generation that names a
  saved
  copy that still exists;
  the timing logs stay at one generation.
Visible result:
  the audit trail lasts as long as the copies it explains;
  the timing logs are unchanged;
  the size bound
  has a
  basis.

Ranking:
C > B > A,
because C keeps the part of the history that explains user data and still bounds the rest;
B > A
because
B never loses a line but has no size basis,
while A silently drops what a person needs to find their copy.

Outcome under the delegation (the coordinator's,
open to the owner's veto):
option C,
implemented.
See "Outcome of the delegated pick 11" in the entry for the limit,
with the measurement and its commands.
The audit generations were kept up to `AUDIT_HISTORY_BYTES` (4 MiB,
derived from the measured rate and the 30-day age
default of the copies),
and a generation that names a saved
copy still on disk is never removed;
the timing logs kept one
generation,
and the standards-review shadow log
was outside this pick.

The owner answered the question on 2026-11-10 in the question tool,
"Keep more,
capped (Recommended)":
32 MiB per log,
about six days of commit timing at today's pace,
and an audit line naming a saved copy that still exists is never dropped.
That answer replaced the four mebibytes and the timing logs' single
`.1` generation:
every log of the wrapper (audit,
commit timing,
worktree-add timing,
push trigger and the standards-review shadow log,
which had no rotation at all)
now keeps numbered generations up to `LOG_HISTORY_BYTES` (32 MiB),
and only the audit log pins one.
The pace it rests on was measured on this host on 2026-11-10T14:31Z:
the commit timing log held 1,143,526 bytes in
229 records from 2026-11-10T09:08:07Z to 2026-11-10T14:31:41Z (about 0.21 MiB an hour,
so 32 MiB is about six days),
and the audit log held 782,685 bytes in 1,619
lines from 2026-10-21 to 2026-11-10 (about 42 KB a day,
so the bound is the whole history of an ordinary
repository).
The reading that decides whether an audit generation pins costs, on this host,
a median of 13 ms and a 99th percentile
of 254 ms when it drops one generation,
and a median of 369 ms and a 99th percentile of 432 ms when every generation pins
and is read to its last line,
so it runs on a thread of its own and the command that rotated the log pays a
median of 385 us and a 99th percentile of 580 us
(`foundation/log_rotation_cost_measurement_tests.rs`,
`mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture rotation_cost_measurement`).
The bound counts every generation of one log,
so a pinned generation spends part of it and the others shrink;
the newest generation is never removed.

Left for the coordinator,
with the owner's veto open,
and settled by the answer of 2026-11-10 unless the owner says otherwise:

- The horizon of the 4 MiB.
  The rate is measured;
  how far back unexplained history should reach is not.
  Options:
  a horizon in days times the measured peak rate (chosen,
  with the 30 days the copies are kept by default);
  a
  fixed count of generations;
  no bound for the audit file,
  which is small.
  The answer of 2026-11-10 set 32 MiB for every log,
  which is the whole history of an ordinary audit log.
  Ranking:
  days times the peak rate > a count of generations
  > no bound,
  > because the first states its own basis and tracks
  the measurement,
  > the second states none,
  > and the third lets the common Git directory grow with no basis at all.
- The timing logs' window.
  At about 760 KB in 30 hours,
  the commit timing file rotates
  about every two days and keeps three to four days of history
  in total.
  The investigation of the 60-second steps reads these files;
  if it needs a longer window,
  give the commit timing log more
  generations (the same retention,
  with no copy to pin).
  This is a finding;
  nothing was changed.
  The answer of 2026-11-10 settled it:
  every log keeps its generations up to 32 MiB,
  and the commit timing file now reaches about six days back at the pace measured on that day.

### LFS server-error retry

Covers:
`RETRY_DELAYS` 2 s and 5 s (`lfs_server_retry.rs:23`).
An HTTP 5xx from the LFS server is not on either of the owner's lists.

- A.
  Treat 5xx as outside our control:
  keep a bounded retry,
  with the numbers cited by the server's observed recovery
  time.
  Visible result:
  the "run again in 2.0 seconds (retry 1 of 2)" line stays.
- B.
  Treat it as designed out:
  end the step at once and let the service's push retry plan (10 s pauses,
  growing to 600
  s)
  try the push again.
  Visible result:
  the first 5xx prints the failure and the plan note;
  there is one retry mechanism.
- C.
  Owner puts 5xx on one of the two lists first,
  and the choice follows.
  Visible result:
  none yet;
  this is the missing classification.

Ranking:
C > B > A,
because the classification decides which of the others is allowed,
so it comes first;
B > A
because B
removes a second retry mechanism and its two uncited numbers,
while A keeps both.

Delegated pick,
2026-11-03:
the classification "outside our control" (the owner's list,
like "Network down")
and B's single mechanism,
built;
see the outcome under "LFS server-error retry".
The coordinator picked it under the owner's delegation,
and the owner may veto it.

### Sizes and ages of the service's own housekeeping

Applied under the delegation:
option A,
where the data allow;
see the entries of the same names and "Measurement
record".
The questions under the `Follow-up questions after the delegated picks` heading
list what the data could not settle.

Covers:
`MAX_BATCH` 64,
`MAX_LINE_BYTES` 1 MiB,
`MAX_CHANGES` 4096,
`CLAIM_LEASE` 120 s,
`DONE_RETAINED` 1024,
`REMEMBERED`
128,
`IDLE_EXIT` 3 h,
`TEMPORARY_GRACE` 10 min,
`STRAY_TEMPORARY_AGE` and `SCRATCH_STALE_AFTER` 24 h,
`WALK_BUDGET`
4096.
Only `MAX_BATCH` has a measurement beside it,
and its largest observed batch equals the cap.

- A.
  Measure each from the service's own journal and timing logs (batch sizes,
  request sizes,
  hold times,
  restarts)
  and
  write the result beside the value.
  Visible result:
  a basis for each;
  some numbers may move.
- B.
  The owner names the numbers.
  Visible result:
  values and basis in one decision.
- C.
  Remove the ones that only guard hostile input or a case that cannot occur,
  and keep the rest uncited.
  Visible result:
  fewer limits;
  `LQM` is still unmet for the rest.

Ranking:
A > B > C,
because the logs already hold the distributions A needs;
B > C because C leaves unmeasured numbers
in
place and a limit that was removed once might be needed.

### Margins for removing left-behind locks

Applied under the delegation:
option A;
see the three lock entries and "Measurement record".

Covers:
`MARGIN` 2 s,
`BOOT_SLACK` 1 h,
`EMPTY_LOCK_SETTLE` 1 s,
`COARSE_CLOCK_SLACK` 50 ms,
`REUSE_SLACK_MS` 100.
These decide when the wrapper removes a lock,
so a margin too short could remove a live Git's lock.
`SPEC.md:1830` measured that Git fills a reference lock after creating it empty,
with no duration.

- A.
  Measure the delay between a lock's creation and its fill under this host's load,
  and set each margin from it.
  Visible result:
  margins with a basis,
  possibly larger than two seconds on a loaded host.
- B.
  Keep the margins and cite the reasoning in `SPEC.md`,
  labelled as reasoning.
  Visible result:
  no change;
  `LQM` is not met.

Ranking:
A > B,
because a wrong margin can remove a live lock,
which is the one error the section's own rules treat as
unacceptable,
and only A bases the margin on the host.

### The free-space preflight

Applied under the delegation:
option A;
see the entry "Free space preflight,
5 GiB".

Covers:
`MIN_FREE_BYTES` 5 GiB (`subvolume_preflight.rs:26`),
"about 1 KiB per entry is an assumption,
not a
measurement".

- A.
  Compute the need from the entry count with a measured cost per entry (convert a real worktree and read the
  metadata
  growth).
Visible result:
  a small repository no longer needs 5 GiB free;
  a large one may need more;
  the refusal names the
  figure.
- B.
  Keep 5 GiB as the owner's decision.
  Visible result:
  unchanged.
- C.
  Drop the preflight and rely on the disk-full failure.
  Visible result:
  a full disk mid-conversion leaves a journaled partial migration for the next run to finish.

Ranking:
A > B > C,
because A matches the need to the repository and cites a measurement;
B > C because refusing
before the
migration starts is safer than failing in the middle,
even when the number is the owner's.

### Questions raised while building picks 5 and 12

Added 2026-11-03 by the agent that built the two picks.
The coordinator picks under the owner's delegation.

#### The stall window of a push with no connection, and a connection that holds and never answers

What the code can observe about a push,
with no time window and no invented number:
whether some process of its tree holds a TCP socket in state `ESTABLISHED` (the kernel's socket diagnostics),
the bytes that socket carried,
and the three signs from `/proc` (bytes,
processor time,
a task in state `R` or `D`).
It cannot tell a slow remote from a live remote that is hung.
It cannot tell a dead peer from a slow one until TCP gives up
(retransmission for unacknowledged data;
keepalive probes for an idle connection,
which Git's HTTP client turns on and `ssh` and `git://` leave to the kernel's default).
A tree with no connection,
such as a push to a local path,
a connect that has not completed,
or a name lookup,
shows nothing a clock can read either,
and today it is stopped after the 30 second stall window,
which no measurement or owner decision determines.

- A.
  As built:
  an open connection is waited for in every phase;
  30 seconds stays for a tree with no connection and no sign of work,
  uncited.
  Visible result:
  a remote that accepts the connection and never answers holds the push until TCP or the remote ends it;
  a push while offline is stopped after 30 seconds and retried by the plan.
  Pro:
  the pick as written,
  and no number for "connected".
  Con:
  one uncited number remains,
  and a hung live remote blocks the repository's pushes with no end.
- B.
  A with the first answer excepted:
  an open connection before any pack was seen is a stall after the window,
  as 0.5.27 has it.
  Visible result:
  a remote that is slow to answer the first request is stopped after 30 seconds and retried.
  Pro:
  a hung remote cannot hold the service for ever.
  Con:
  contradicts "no stall window for a connected push",
  and keeps the number.
- C.
  A,
  with the 30 seconds measured:
  time,
  on this host,
  from the start of a push to its first established socket
  and its longest silence with no connection over many pushes,
  and write the figure beside the value.
  Pro:
  satisfies the citation rule without changing behavior.
  Con:
  needs new logging first,
  because no log holds it,
  and it measures one host.
- D.
  Drop the window for a tree with no connection too,
  so the code stops a push only on a closed connection,
  or never.
  Pro:
  no number at all.
  Con:
  a hung local hook or a lookup that never ends holds the repository's pushes for ever.

Ranking:
A > C > B > D,
because A is the delegated pick and changes the least behavior;
C > B because C keeps behavior and adds the missing basis while B reverses part of the pick;
B > D because B still ends a hung state that D leaves for ever.
C is a complement of A and can follow it.

Picked (coordinator,
under the owner's delegation,
veto open):
C,
A as built plus the measurement once logging records quiet time,
because A alone leaves the value uncited against `LQM`.

#### Limits that remain beside the push, with no citation

`STOP_GRACE` 5 s (partial),
`MAX_PUSH_OUTPUT` 4 MiB and `QUERY_CEILING` 120 s have no measurement or owner decision
that determines them.
The audit said they could be cited by measurement under any option.
They are marked as uncited beside the value in the code and in `SPEC.md`,
with what exists,
and not measured here,
because a measurement needs a load the brief forbids
(a push to a remote that prints megabytes,
and a stopped Git whose locks are timed).

- A.
  Measure each from the service's own journal once it logs the data (push output size,
  time from stop to child exit).
- B.
  The owner names the numbers.
- C.
  Remove `QUERY_CEILING` with the Git child steps work (its kill is the audit's question on Git child steps)
  and keep the other two uncited.

Ranking:
A > B > C,
because the journal already holds most distributions A needs and C leaves two numbers uncited.

Picked (coordinator,
under the owner's delegation,
veto open):
C for `QUERY_CEILING`,
which moved to the Git child steps work (pick 4 ends a Git child only on no progress),
and A for `STOP_GRACE` and `MAX_PUSH_OUTPUT`.

### Follow-up questions after the delegated picks

Each lists what the measurements could not settle.
The ranking is the coordinator's reading of the options against the decisions quoted in
"Owner decisions that bear on limits".

#### Batch size and claim lease

The service journal shows batches of one only,
and no kill of a service,
so neither value has data behind it.

- A.
  Keep 64 and 120 s as design values,
  cited as such.
  Visible result:
  nothing changes;
  the audit keeps both as "data cannot support".
- B.
  Measure a deliberate burst against a throwaway service (8,
  16,
  32,
  64 and 128 simultaneous commit requests) for the
  batch size,
  and kill a throwaway service during a claim for the lease.
  Visible result:
  a curve or a recovery time beside each value;
  the numbers may move.

Ranking:
B > A for the batch size,
because a burst is one cheap real operation and the cap is the only value the
prototype measured at a single point;
A > B for the lease,
because no kill has been seen,
so recovery time is not yet a
cost anyone pays.

#### Idle exit

- A.
  Keep three hours.
  Visible result:
  unchanged;
  the 28 running services hold 212 MiB together.
- B.
  Thirty minutes.
  Visible result:
  fewer resident services,
  and a cold start (at most 70 ms measured) after any idle gap past thirty
  minutes.
- C.
  Ten minutes.
  Visible result:
  the same,
  more often.

Ranking:
A > B > C,
because the memory saved is about 200 MiB on a 63 GB host while every cold start is a wait ("You
should (almost) never be waiting.");
B > C because it pays fewer cold starts for the same kind of saving.

#### Concurrent per-update queries

- A.
  Keep four and call it a bound on connections to remotes.
  Visible result:
  unchanged;
  local queries stay 37 and 18 percent slower than at eight.
- B.
  Eight for the local queries,
  four for the ones that open a connection to a remote.
  Visible result:
  faster local queries;
  one more distinction in the code.
- C.
  Eight for all.
  Visible result:
  the same gain;
  an unmeasured number of connections to a remote at once.

Ranking:
B > A > C,
because B takes the measured gain without an unmeasured remote risk and fits "make sure we're
running
everything that can be run in parallel in parallel";
A > C because C trades an unmeasured risk for the same gain.

#### Thresholds of the copy stage and the install, and the unmeasured parallel constants

Covers:
`PARALLEL_COPY_MIN_ENTRIES` 4096,
`PARALLEL_INSTALL_MIN_LEAVES` 1024,
`REMOVAL_SPLIT_DEPTH` 3,
`INSTALL_BATCH_ENTRIES` 4096,
`INDEX_REWRITE_COPIES` 4,
`SLICE_MEMORY` 16 MiB.

- A.
  Measure each on the real stage,
  install,
  removal and index rewrite (a worktree add by the copy route on a throwaway
  copy,
  three to five runs,
  alternating arms).
  Visible result:
  a number with a basis for each;
  the thresholds probably fall toward the 512 of `clone_tree`.
- B.
  Move the two thresholds to 512 on the strength of `clone_tree` alone.
  Visible result:
  copies of 512 to 4095 entries finish tens to hundreds of milliseconds sooner;
  the stage itself is not
  measured.
- C.
  Keep the values with their comments.
  Visible result:
  unchanged;
  the audit keeps them open.

Ranking:
A > B > C,
because A is the only option that measures the code that runs;
B > C because the proxy and the
primitive agree and the cost of being wrong is a few threads started early.

#### Foreign-owner patience and the live lock wait

The patience is twice the 1800 s live wait,
and the wait belongs to another pick.

- A.
  Keep an hour now and revisit when the wait pick lands.
  Visible result:
  unchanged.
- B.
  Derive the patience from the wait (twice `DEFAULT_LIVE_WAIT`) once that pick lands.
  Visible result:
  the two cannot drift apart;
  if the wait becomes unbounded the relation has to be replaced.
- C.
  Shorten it to a multiple of the measured 750.6 s.
  Visible result:
  a stuck directory clears sooner;
  a wrapper that waits for a lock longer than the patience could lose
  its
  directory.

Ranking:
A > B > C,
because A and B are complements (A now,
then B) and neither risks a live owner's data;
C shortens
the
block by trading it against that risk,
which the planning document already marks as the owner's call.

#### Lock age margin

- A.
  Keep four seconds (the whole second above the longest sample).
  Visible result:
  as shipped.
- B.
  Twice the longest sample,
  seven seconds.
  Visible result:
  a stale lock is left a few seconds longer;
  headroom for a tail the sample missed.
- C.
  Ten seconds.
  Visible result:
  more delay with no new evidence.

Ranking:
B > A > C,
because the sample maximum understates the tail and a wrong removal costs more than a few seconds
of a
stale lock;
A > C because C adds delay without evidence.

#### Free-space preflight scope

- A.
  As built:
  count entries by listing the tree,
  2048 bytes each.
  Visible result:
  exact for the copy,
  and the dry run reads every directory once (about 3 microseconds an entry warm).
- B.
  Count only the entries of the index.
  Visible result:
  no walk;
  ignored files such as `node_modules` and `target` are not counted,
  so large checkouts are
  under-estimated.
- C.
  A as built plus the later passes and the index rewrite.
  Visible result:
  a larger need with a measured basis for each pass.

Ranking:
A > C > B,
because A counts what the copy writes;
C adds costs that were not measured here and so needs another
measurement first;
B is cheap and wrong where the entries are.

#### Picked for these follow-ups

Coordinator,
under the owner's delegation,
veto open:
- Batch size and claim lease:
  B for both,
  because `LQM` wants a measurement or an owner decision and a design value is neither;
  the burst and the kill run against a throwaway service.
- Idle exit:
  A.
- Concurrent per-update queries:
  B.
- Thresholds of the copy stage and the install:
  A.
- Foreign-owner patience:
  A now,
  then derived again when the index-lock wait pick lands (that pick removes the 1800 s bound for a live holder).
- Lock age margin:
  B.
- Free-space preflight scope:
  A.

Applied on branch `fix/git-policy-cli-limits-cite-measurements`:
batch size 64 to 256 (measured burst),
claim lease kept at
120 s (measured kill recovery 119.55 to 119.57 s),
idle exit and foreign-owner patience and preflight unchanged,
local
per-update queries 8 and remote 4,
the three parallel thresholds 16,
install batch and removal depth and index rewrite
copies measured and kept,
lock age margin 7 s.
See "After the follow-up picks" and "Follow-up measurements".
