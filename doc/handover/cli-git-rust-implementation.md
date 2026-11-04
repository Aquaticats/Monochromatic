# cli-git implementation handover

This page holds only open work and current state of the Git wrapper effort (`package/git-policy/cli`),
per rule H0P.
Landed and superseded work is recorded in the package docs (`SPEC.md`,
`README.md`,
`DESIGN.md`),
in `doc/decision/`,
in `doc/planning/cli-git-*.md`,
in `doc/troubleshooting/` and in the lane evidence files `doc/handover/cli-git-*.md`.
The append-only log that preceded this page ends at commit `e10b8186bc`;
`git show e10b8186bc:doc/handover/cli-git-rust-implementation.md` prints it.
Add a short entry here only for something still open;
when it lands,
move its durable facts to their canonical doc and delete it from this page.

## Current state

Written 2026-11-03T08:16:08-05:00 against main `8ec558b68a` (committed 2026-11-03T07:59:21-05:00).

### Installed versions

- Git wrapper `monochromatic-git-policy-cli` 0.5.27 (`toolchain.lock.json`,
  row `cargo:monochromatic-git-policy-cli`,
  lock commit `4779907f6e`;
  `git --version` prints "monochromatic-git-policy-cli 0.5.27,
  shadowing git at /usr/bin/git" over Git 2.56.0).
  Its commit service unit runs with systemd's defaults
  (`systemctl --user show` at 2026-11-03T10:47-05:00:
  `MemoryMax=infinity`,
  `MemoryHigh=infinity`,
  `CPUQuotaPerSecUSec=infinity`,
  `IOWeight` not set,
  `TasksMax=75462`,
  systemd's own default).
- `monochromatic-mpm-shadow` 0.2.5,
  `monochromatic-pnpm-shadow` 0.1.1,
  `monochromatic-search-shadow` 0.2.1,
  `monochromatic-file-enforcer-cli` 0.1.1,
  `monochromatic-lint` 0.5.0 (all from the same lock file).
- `monochromatic-dependency-catalog` 0.1.0 is the published version;
  0.2.0 (with `trustLockfile`) exists only on the cargo-shadow branch.
- The global toolchain lock (`~/.config/monochromatic-dev/toolchain/toolchain.lock.json`) was moved to wrapper 0.5.25
  on
  2026-11-03;
  its copy from before that move is `~/temp/agent/global-lock/toolchain.lock.json.before-2026-11-03T0715`.

### Release in flight

None.
0.5.28 was published to crates.io at 2026-11-04T03:09:07Z (publish run 41764751088,
built from `7b41c215ec`) and installed by `upgrade-wrapper.ts 0.5.28`
(`git --version`:
"monochromatic-git-policy-cli 0.5.28,
shadowing git at /usr/bin/git").
Its lock commit on main took 1743 ms in all,
1140 ms of it provisioning the new service (the 0.5.27 service refused the protocol-7 request line,
then retired after its batch);
the 0.5.28 service's landing took 44 ms (`lock_hold_ms=25`,
`sync_ms=22`).
`sync_ms` is not a disk sync:
it times removing the conclusion state and building and installing the new index (`service_timing.rs:84`);
queued:
rename it by its role.
History of the 0.5.28 release:
base gate `gpc-gate-base-0-5-28-final-20261103T214518-0500` passed 4706 of 4706 (7 skipped) on `d9e8907ae8`;
the base was merged into main as `920fdce60d` (2026-11-03T21:57:04-05:00,
body `Closes #6260`);
dispatched as run https://github.com/Aquaticats/Monochromatic/actions/runs/41764055048;
`native:release:wait` is following it;
next:
`upgrade-wrapper.ts 0.5.28`,
then time commits on main against native Git.
Before 0.5.28:
0.5.27 was dispatched 2026-11-03T10:28-05:00 from main after its gate passed (4264 of 4264,
6 skipped),
run https://github.com/Aquaticats/Monochromatic/actions/runs/41663034226,
published to crates.io at 2026-11-03T15:44:38Z,
and installed by `upgrade-wrapper.ts 0.5.27`.
Its lock commit on main took 1625 ms in all,
of which 1169 ms was provisioning the new service (the 0.5.26 service retired after its batch);
the service's landing took 49 ms (`lock_hold_ms=28`,
`sync_ms=25`).
0.5.26 was published to crates.io at 2026-11-03T13:21:01Z.

### Unmerged branches and worktrees of this effort

Tips were read from `git for-each-ref` at 2026-11-03T08:16:08-05:00 and keep moving while the owning agents work.

- `fix/git-policy-cli-commits-never-fail` `f160209787`,
  worktree `.claude/worktrees/commits-never-fail`.
- `fix/git-policy-cli-plain-commit-finishes-conflicted-merge` `a06af6102c` (finished),
  worktree `.claude/worktrees/plain-commit-conflicted`.
- `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits` `e38a7dea6a` (finished),
  worktree `.claude/worktrees/no-inner-quiet-slow-server`.
- `feat/git-policy-cli-word-level-second-merge-pass` `8cc36cd4b6`,
  fifteen commits ahead of the merge base,
  worktrees `.claude/worktrees/word-level-second-merge-pass` and the gate copy
  `.claude/worktrees/gate-word-level-merge-pass`.
- `feat/cargo-shadow-cli` `504a82aa3f`,
  worktree `.claude/worktrees/cargo-shadow`,
  fifty commits ahead,
  all on origin.
- `worktree-agent-ab3dd6294f9d91284` `fe2081f2a1`,
  the copy-in-service design that was never merged (see the P1 section).
- `e2e-trace-diagnosis-20261101` `a0126e571c`,
  unmerged and kept on purpose.
- `integrate/git-policy-cli-deadline-stop-and-conversion-real-git` `2ac6da873e` is the stopped first 0.5.21 candidate;
  both of its parts shipped by other routes,
  so it is stale and can be deleted once the owner agrees to branch deletions.

Many detached worktrees under `.claude/worktrees/` (`integrate-0NN`,
`gate-*`,
`survivors-*` and others) remain from merged work;
their files and mutation evidence were kept deliberately,
and removing them is open (see "Housekeeping").

### Agents and gates running

Update after the last list:
the integration branch (`fix/git-policy-cli-commits-fast-under-disk-pressure` tip `c7ce91581e`,
gate 4703 of 4703) was fast-forwarded into the 0.5.28 base,
then the batch-number fix (`fix/git-policy-cli-service-batch-numbers-unique` tip `9c3d279e1a`,
gate 4666 of 4666) merged as `d9e8907ae8`;
the base gate `gpc-gate-base-0-5-28-final-20261103T214518-0500` is running,
and 0.5.28 is released from that tip when it passes.
Measured on that integration tip,
median of 19 warm paired rounds:
signed commits wrapped 207 ms against native 118 ms (1.75 times),
unsigned 171 against 115 ms (1.45 times);
0 wrapper syncs waited on a publishing landing (7 queued to a background thread),
3 forced on a withdrawal;
Git's own defaults (`core.fsync` read from Git 2.56.0 source) are followed.
The batch-number bug was real for services sharing a journal (development runs and the test harness;
production gives each instance its own journal):
two landings could share a number and one landing's records overwrite the other's;
fixed with an `O_EXCL` claim file per number.
Models round 9 done (`feat/git-policy-cli-commit-protocol-models` tip `3e86918e4c`,
252 of 252):
W1 and the register rebuilt for revision 13 (red under 12),
controls 1 to 6,
9 to 17 and F7/F9 re-opened,
historical models labelled by a guard test,
state log distinguishes absent,
damaged and unreadable,
legacy pair and binding removed (spool kept).
cargo-shadow fifth review:
all four parts "not correct";
round 6 running with the owner's kept-files answer.
Thirteenth design review:
"Approve continued model work only";
revision 14 written (`fc28bb5219`);
fourteenth review (`commits-never-fail-sol-14.txt`):
owner questions "accurate after the edits below,
not accurate as written",
6a to be rewritten (it omitted fail-closed recovery),
and planned shutdown added a hook-cancellation exception the owner never approved (R14-9);
revision 15 written (`4a6ad9d34a`,
decision 14 recorded,
6a and 6b dropped,
planned shutdown is a drain that ends no protected program),
with questions 5,
7a,
7b and 7c in `commits-never-fail-questions-4a6ad9d34a.md`;
the fifteenth review (`commits-never-fail-sol-15.txt`) checks them before they are asked.
Models round 10 done in part (`feat/git-policy-cli-commit-protocol-models` tip `ffc98d1eae`,
293 of 293):
all twenty-two revision 14 controls carried by named tests in an `ACCEPTANCE` manifest (red under 13,
green under 14);
the open owner questions modelled as parameters;
`StateWriter` single writer,
validated checkpoint replacement root,
`sync_prefix` watermark.
Not done:
dependency discharge in a component,
T1's phased observations,
the keeper write end inside the W1 model,
components 4.2,
4.4,
4.6,
4.7.
Its reviews are running (`commit-components-sol-7.txt`,
`commit-models-sol-5a.txt`,
`commit-models-sol-5b.txt`);
the next models round takes them with revision 15 and its review (the 6a `StopPolicy` parameter is obsolete under decision 14).

At 2026-11-03T20:14:48-05:00 (the owner's cap is five delegate agents at once),
five running:
- Integration and the owner's power-cut answer on today's landing,
  branch `fix/git-policy-cli-commits-fast-under-disk-pressure` (from `ce711b0379`),
  worktree `.claude/worktrees/fix-commits-fast-under-disk-pressure`:
  merges the base tip `f89fa63f10` and index round 5 (`b47467ebd1`),
  then takes the wrapper's own syncs off the waiting path of a publishing landing
  ("Force at leaving, else background",
  in "Owner decisions in force"),
  then measures and gates.
  Its first round,
  measured on a main-shaped fixture (20,740 files) under the load there was:
  the 0.5.28 base was worse than 0.5.27
  (warm paired delta 273 against 139 ms unsigned,
  298 against 172 ms signed;
  12 fsyncs per landing against about 4,
  native none);
  its tip is 2.0 times native warm (116 against about 113 ms),
  but still waited for 11 fsyncs per landing,
  the cause of the 8 to 36 s commits
  (the production journal attributes 78% of slow-landing time to `sync_ms`,
  up to 119,715 ms).
  Fixed in that round:
  pack lookups and pack opens read once instead of per search step
  (46.9 MB and 79,951 `pread64` per commit against native's 1.5 MB and 8),
  `diff-tree` reads only differing subtrees,
  commit listing once per process,
  the child watch looks at its child before listing the host's processes,
  the gate stopped walking whole trees (69 to 10 ms),
  one redundant sync removed;
  gate 4622 of 4622.
  No measured pair reached 5 s,
  so no raise of the 5 s standard was asked;
  `doc/planning/cli-git-commit-latency-under-disk-pressure.md` says no finite bound holds while a sync stays on the waiting path.
- Design revision 13,
  `fix/git-policy-cli-commits-never-fail` (from `18ca87c579`):
  records the owner's four revision-12 answers and answers the twelfth review
  (`commits-never-fail-sol-12.txt`:
  "Approve continued model work only";
  first need:
  a complete submitted-attempt and normal-exit state machine).
- Models round 7,
  `feat/git-policy-cli-commit-protocol-models` (from `8dbe45a649`):
  the components rebuilt on rule V (one checksummed request file,
  a double-buffered state pair,
  zero wrapper syncs on the waiting path),
  the open components findings (standalone `repair_tail` contract,
  amendment parents as a recoverable closure,
  a working native fallback for refused gitlinks),
  and the model findings that do not depend on revision 13.
  Round 6 (178 of 178 model tests):
  the explorer's false-green bounded wait fixed red-first;
  the W1/P2 model now shows revision 11's COMMITTING exit as red against the owner's rule;
  publication split into send,
  apply and acknowledge;
  the disposable Git runner bounded;
  components refuse a symlinked root,
  resend without a temporary,
  append reading only the tail,
  replay without the quadratic scan,
  and a replay from a closure rebuilt after pruning.
- cargo-shadow round 5 done:
  tip `8e42ae47c9`,
  428 of 428 tests,
  clippy clean,
  103 `avoidable-closure` warnings.
  One workspace lock (`.cache/cargo-shadow/lock`,
  non-waiting `flock`,
  close-on-exec) replaces the per-journal lock (agent's pick,
  veto open);
  cargo children run in a recorded process group and `bounded_run` stops the whole group;
  the reader asks the answering Git for its system configuration;
  kept copies capped at `MAX_KEPT_FILES` 1,000 (unmeasured;
  past it a new transaction is refused with a listing,
  nothing deleted):
  an owner question.
  Bounds now cite measurements from this repository (cargo calls 0.05 to 0.91 s against 30 min;
  Git 0.006 to 0.018 s against 60 s),
  the rest labelled unmeasured.
  A pnpm 12.16.1 oracle confirmed pnpm also refuses an unchanged update that leaves a too-young version locked.
  The agent used Python and shell scratch scripts (`bench_bounds.py`,
  `t.sh`),
  against AGENTS.md's TypeScript-only script rule;
  to be said in its next brief.
  Fifth review running in four parts (`cargo-shadow-sol-5a/5b/5b2/5c.txt`).
- (Earlier entry) cargo-shadow round 5,
  `feat/cargo-shadow-cli` (from `e4451f1a75`),
  answering the fourth review (all four parts "not correct":
  4a Git-equivalent object acceptance;
  4b two durable-preservation gaps,
  kept paths lost on errors,
  kept copies unbounded;
  4b2 journal scanning is not transaction exclusion,
  the lock's lifetime does not cover surviving cargo writers,
  no final durability barrier;
  4c cleanup can erase a refusal,
  the bounded runner does not prove writers stopped,
  a placeholder race)
  and measuring its bounds (LQM).
- Batch numbers,
  `fix/git-policy-cli-service-batch-numbers-unique` (from `f89fa63f10`):
  a gate run once saw two of sixteen services answer with the same batch number in `service_sequencer::a_pathspec_commit_after_a_conflicted_cherry_pick...`;
  product bug or test count,
  to be found out.

Done since the last list:
- Every wait has a bound:
  fast-forwarded into the 0.5.28 base (tip `f89fa63f10`).
  Step ceilings with measurements:
  `PLUMBING_STEP_CEILING` 120 s (longest read step of 1,651 landings 9,663 ms),
  `OBJECT_STEP_CEILING` 600 s (longest client preparation 250,280 ms),
  `REFERENCE_STEP_CEILING` 60 s (longest of 1,651 reference updates 3,438 ms);
  a reference update ended at its ceiling is followed by a read of the branch:
  moved means landed,
  not moved means withdrawn as at `LOCK_CEILING`.
  The push stall prints that the push is queued (not yet seen through a real service).
  The coordinator's picks on its option sets are in `doc/planning/cli-git-no-unbounded-waits.md`,
  "Picks" (veto open).
- Index writer round 5,
  `fix/git-policy-cli-index-recovery-never-overwrites-staging` tip `b47467ebd1`,
  gate 4641 of 4641:
  a restart replays a batch only if the live index is still the very file the landing saw (inode,
  size,
  times,
  checksum tail);
  otherwise it resets nothing and logs a notice.
  Visible change:
  any rewrite of the index between a failed install and the restart,
  even a `git status` refresh,
  leaves the commit's paths un-updated in the index (nothing is lost).
  Not closed:
  the `HEAD` race after the last check (the same window native Git has);
  physical-index reclamation is mitigated,
  with options A (one reclaimer per physical alternate index:
  refuse to finish an index outside the instance's own Git directory) > B (document);
  coordinator's pick A under the delegation,
  veto open,
  queued after the integration.
  The weaker options it offered (per-path replay,
  content-only witness) are not taken:
  both discard ABA staging.
- Design revision 12 (`18ca87c579`) and the owner's four answers.
- Models round 7,
  `feat/git-policy-cli-commit-protocol-models` tip `7b87e79d3d`,
  188 of 188 model tests,
  both clippy tasks clean:
  components on rule V (`commit_inbox.rs`:
  one checksummed request file linked from `inbox/tmp/`,
  identity the hash of content and lane instance;
  `commit_state.rs`:
  `state.a`/`state.b` written in place without a sync);
  red test first counted the revision 11 path at exactly 22 syncs,
  the new waiting path makes 0,
  a departure syncs request file,
  inbox directory and newest state file in that order before returning.
  `LedgerOwner` is a non-blocking exclusive lock on `<ledger>.owner`
  (the standalone `repair_tail` and `append` are still public);
  amendments and gitlink changes are refused before acceptance and go to real Git (a test shows real Git commits them).
  The explorer gained `explore_eventually` (cycles with no exit).
  Not done:
  the order and ledger lines at the departure barrier (not built yet);
  F2,
  F1 reflog,
  F5,
  E1,
  persistence intermediate states,
  M1 compare-and-unlink;
  deferred to revision 13:
  the register's cumulative deadlines,
  the merge edit-loss invariant.
  The revision 11 binding,
  spool and record modules are unused by the new path;
  coordinator's pick (veto open):
  remove what revision 13 still leaves unused,
  after it lands.
  Reviews:
  `commit-components-sol-5.txt` "Not correct as submitted" (the state pair loses its checkpoint across successive unsynced transitions;
  the departure barrier omits state-name durability and the order and ledger lines;
  equal payloads get one identity),
  each answered by a revision 13 mechanism (state log,
  D-sets,
  invocation nonce) and sent to models round 8;
  `commit-models-sol-3a.txt` "sound after named fixes" (liveness semantics and truncation of `explore_eventually`;
  historical greens kept out of approval evidence);
  `commit-models-sol-3b.txt` "Not sound as verification of revision 12" (recovery still infallible,
  child progress owner-dependent,
  the register still assumes timeout completion;
  F7 `EEXIST` and F9 not retired by rule V).
  Both were sent to models round 8.
- Models round 8 (tip `8b31735dd3`,
  198 of 198,
  both clippy tasks clean),
  partly done before that agent ran out of room:
  `commit_state_log.rs` (hash-chained log,
  longest valid prefix,
  rotation safe at every step,
  0 syncs while waiting);
  `repair_tail` and `append` crate-private behind `LedgerOwner`;
  the departure forces request files,
  inbox directory,
  order file,
  state log and ledger with their directories;
  a 16-byte invocation nonce in each request (`REQUEST_VERSION` 4);
  `explore_eventually` stated as AG EF and truncation no longer makes violations.
  A fresh agent took the rest (models round 9,
  same branch):
  the W1 and register models rebuilt for revision 13 with fallible recovery,
  the twenty controls,
  F7 and F9 re-opened,
  the leftovers,
  historical greens labelled,
  components items 3.5 to 3.7 and removal of unused legacy modules.
- #6260 merged into the base.

Queue,
in order:
- After the integration:
  gate the base,
  release 0.5.28,
  install,
  time commits against native Git on main.
- Then #6261 (directory regroup;
  the #6260 agent listed the paths it must update),
  lint task 88,
  the gate container base check (Debian trixie against the CachyOS rule),
  the reclamation pick A,
  and the reviews of cargo-shadow round 5,
  models round 7 and revision 13.

Older entries,
from the 2026-11-03T10:00-05:00 list,
are history to prune into canonical docs:
- Shipped-limits pick 1's classing with the measured citations 7,
  8,
  13,
  14 and 15,
  branch `fix/git-policy-cli-limits-cite-measurements`,
  worktree `.claude/worktrees/limits-cite-measurements`.
- Shipped-limits picks 4 and 9,
  branch `feat/git-policy-cli-steps-end-only-on-no-progress` from the 0.5.28 base `f58510c538`,
  worktree `.claude/worktrees/steps-end-only-on-no-progress`.
- Done and merged into the 0.5.28 base:
  the wrapper never runs itself as Git,
  `fix/git-policy-cli-never-runs-itself-as-git` `7ed8e9abe5`.
  Cause:
  `classify_candidate` (`real_git_candidate.rs`) compared a `PATH` candidate with the running file by inode,
  path and bytes,
  so the toolchain launcher counted as real Git,
  and the `forwarded_to_self` guard (`entry.rs`) named the launcher,
  not the binary behind it.
  Fix:
  candidates with a `<name>.launch` record of this tool are skipped,
  and every real-Git start sets `CLI_GIT_STARTED_AS_GIT` (starter pid,
  birth,
  mode),
  so a wrapper that arrives as its own starter's Git stops with exit 2 and a message naming every candidate.
  Coordinator's picks under the delegation,
  veto open:
  a user script named `git` that runs the wrapper without `exec` stays a documented limit
  (a depth cap would need an uncited number;
  a script that uses `exec` is caught);
  falling back to the real Git the wrapper was installed over when `PATH` holds none is queued after 0.5.28.
  Found by the cargo-shadow agent's bench on 0.5.26:
  `PATH` held `cli-git-bin/git` (a symlink to the toolchain launcher `.cache/toolchain-launchers/cli-git-native`)
  and no real Git;
  `git rev-parse ... @{upstream}^{commit}` became a parent-to-child chain of wrappers,
  each adding its `-c hook.*.enabled=false` options,
  exactly 100 at the scope's `TasksMax=100`,
  all sleeping,
  until the scope was stopped;
  `git --version` still printed "shadowing git at /usr/bin/git".
  Against "We should never invoke ourselves recursively."
  (`doc/planning/cli-git-no-self-invocation.md`);
  without a cap it can exhaust the host's process limit.
- Done and merged into the 0.5.28 base (`177f9ebc50`):
  shipped-limits picks 2 and 6,
  `feat/git-policy-cli-index-lock-and-service-waits` `008b74d3a3`.
  Its follow-ups,
  picked under the delegation (veto open) and queued:
  an unproven lock owner is waited for like a proven one,
  because an editor-held `git commit` would otherwise make ours fail after 1 s;
  poll when no inotify watch is available;
  ask the service manager to start an inactive unit after `START_WAIT`;
  a scratch file can be aged out during a wait longer than a day.
  The 150 s hold-age margin is for the owner to approve on return.
- Done and merged into the 0.5.28 base (`cfd30ccd18`):
  the two shipped lock-deletion P1s from the eighth design review (S-1,
  S-2),
  `fix/git-policy-cli-never-removes-a-live-lock` `9f92dae98c` (from `177f9ebc50`).
  The stopped-child sweep no longer deletes anything itself
  (it deleted a live foreign writer's empty lock,
  and a lock made after an old pid file);
  it calls the same judgement as every fetch and push.
  The boot-time proof is gone (a clock step fooled it);
  a pid number proves a holder gone only from the initial PID namespace,
  without `hidepid`,
  and when no process carries that number inside a nested namespace (`NSpid:`).
  Companion files (pid file,
  `packed-refs.new`) go through the same rename,
  compare and remove-or-put-back step as the lock;
  that step now checks identity twice and moves with `RENAME_NOREPLACE`.
  7 red tests first,
  98 library and 10 binary tests,
  clippy clean.
  Its four questions are in "Picks made under the owner's delegation".
  Conflict to expect:
  the citations follow-ups set `BOOT_SLACK` to 3607 s,
  which this branch removes.
- Done:
  the second cross-vendor review of cargo-shadow,
  split in three after the whole 733 KB diff timed out at 3600 s with no output
  (2026-11-03T11:23:48-05:00),
  each part from `504a82aa3f` to `09bb7820ea` with the first review attached.
  None of the three parts passes.
  Part 2a (the Git reader,
  `cargo-shadow-sol-2a.txt`,
  2026-11-03T11:38:43-05:00) says "not correct",
  with three P1s that can pick `HEAD` where Git picks an older upstream:
  a `\` at the end of a comment line joins the next line,
  hiding an `[include]`;
  `extensions.worktreeConfig = yes` (or `on`,
  or a nonzero number) is not read as true;
  the last `branch.<name>.merge` entry is used where Git uses the first.
  P2s:
  the section header parser (dotted section with a quoted subsection,
  `]` inside quotes);
  trailing whitespace trimmed from quoted values;
  a bare key read as the string `true`;
  worktree-private refs read from the common directory;
  `refs/remotes/<name>/HEAD` missing from short-name lookup;
  a hexadecimal ref value taken as a commit without reading the object;
  `/etc/gitconfig` assumed as the system file;
  `gitdir:` path whitespace trimmed and `commondir` read errors taken as absence;
  unknown extensions and invalid refspecs answered natively.
  P3:
  the fallback can start four Git processes,
  not one.
  It asks for a differential suite with Git as the oracle.
  Part 2b (the journal and restore,
  `cargo-shadow-sol-2b.txt`) says "Not correct":
  output attributed by reading after Cargo returns,
  an exchange-then-unlink race,
  a fallback race,
  predictable temporary names,
  read errors taken as empty,
  no fsync ordering,
  an unreadable manifest index read as success,
  a global `kept_a_lock`,
  no recovery claim;
  P2s for lossy paths and CSV memory.
  Part 2c (install,
  verify,
  policy and the planning record,
  `cargo-shadow-sol-2c.txt`,
  2026-11-03T11:37:58-05:00) says "not ready for 0.1.0",
  with seven P1s:
  a Cargo call that writes a lockfile and then exits nonzero leaves it unrecorded,
  and the `--locked` fallback builds it;
  a lookup error after an age refusal erases the refusal and builds;
  `cargo update` checks only versions this command added,
  so an unchanged `--precise` target is never judged;
  the same post-Cargo read attribution as 2b;
  placeholder cleanup reads,
  then deletes,
  so a lockfile published in between is lost;
  a sync-failure rollback writes the old catalog over another writer's edit;
  a malformed historical timestamp sorts first and gives a new publisher earlier history.
  P2s:
  frozen non-strict age warnings are dropped;
  `cargo update -m` without a value becomes a full update;
  an option value equal to the verb is taken as the verb;
  partial sync edits are not recorded.
  P3:
  seven comments say the opposite of the code.
  Before those reviews,
  the first review's follow-ups were done:
  294 of 294 tests,
  the trust base read from Git's files with one `git` start only in the cases they cannot answer,
  saving 1.5 to 4 ms per call,
  `file-enforcer write` from source reproduces the generated line,
  and about 4300 comment lines,
  partly from a regex tool,
  which the review checks for false statements).
- Done,
  not merged anywhere (design code,
  nothing live calls it):
  step 1 of the commits-never-fail build,
  `feat/git-policy-cli-commit-protocol-models` `22e0ba1f37`,
  worktree `.claude/worktrees/commit-protocol-models`.
  A hand-written schedule explorer (stateright is not in the offline registry;
  loom and shuttle cannot model system calls)
  and models F1 to F9,
  each red under revisions 6 and 7 with literal replays and green for revision 8 or a candidate;
  step 3 components (ledger,
  identity binding,
  request,
  spool) and the step 4 pure replay,
  differential against Git 2.56.0;
  105 of 105 tests (`native:test:model`),
  clippy clean.
  Four findings against revision 8,
  sent to the revision 9 agent:
  a resend answered from a visible but not yet durable inbox record (F7);
  an owner wedged with no gate cannot be ended by a catchable signal (F3,
  L7);
  L4 must raise the capture generation to odd first (F6);
  a recorded swap-failed replays silently after the post-anchor reflog lines are erased (F1,
  R4).
  The eighth review's section-4 controls are not built;
  they follow revision 9.
  Its comment density is below the dum-dum skill's in the F2 to F9 models and three component files.
- Done and merged into the 0.5.28 base (`afc9a8621d`,
  both merges clean,
  clippy clean on the result):
  the fallback to the real Git the wrapper was installed over (task 150),
  `feat/git-policy-cli-falls-back-to-installed-git` `6453c380b1`.
  When `PATH` offers no real Git,
  it uses the `--git` path from the commit service's unit file under `$XDG_RUNTIME_DIR/systemd/user`,
  checked as every `PATH` candidate is,
  and prints one line naming the path and the directory to add to `PATH`;
  no usable record keeps the stop.
  7 new binary tests and 17 new unit tests,
  red first;
  its four questions are in "Picks made under the owner's delegation".
- Done and merged into the 0.5.28 base (`afc9a8621d`):
  the shipped-limits follow-ups after the citations (task 151),
  `fix/git-policy-cli-limits-cite-measurements` `e314460f79`
  (2026-11-03,
  measured under the load other sessions put on the host;
  figures in the audit doc's "Follow-up measurements" section).
  Lock age `MARGIN` 7 s (twice 3438 ms,
  rounded up),
  `BOOT_SLACK` 3607 s;
  8 local queries at once (the earlier measurement,
  not repeated) and 4 remote;
  batch cap 256,
  the size at which the per-request landing cost (0.16 ms) had not yet doubled,
  from a burst of up to 1024 clients against a throwaway service;
  claim lease kept at 120 s (a killed claim recovered 119.55 to 119.57 s after the kill;
  a live holder losing its claim was not measured);
  copy,
  install and clone go parallel from 16 entries (`PARALLEL_BREAK_EVEN_ENTRIES`;
  the old 4096,
  1024 and 512 left up to 145,
  50 and 12 ms unused);
  removal split depth 3,
  install batch 4096 (12% slower than one batch with a stand-in recorder;
  the owner's trade) and 4 index copies kept as measured.
  Red tests first for each changed value;
  688 unit tests,
  8 binary tests and clippy clean;
  the full gate was not run.
  The claim-kill probe's scope showed inactive before the tests ran,
  so that run was bounded only by the tests' five-minute limit.
- Done and merged into the 0.5.28 base (`a840964c97`):
  the third round on the in-process index writer,
  `perf/git-policy-cli-follow-up-index-in-process` `01e4d4c610`.
  Findings 1 to 5 of the third re-review fixed,
  each red first;
  6 (the reclaimer's check-then-unlink) recorded in SPEC,
  not fixed.
  The shipped P1:
  `IndexLock::release_with_record` installs the index,
  records `<batch>.indexed` (the commit name,
  synced) while the lock is held,
  then unlocks;
  recovery that finds the record replays only the conclusion state.
  The HEAD check is asked again once the lock is held;
  recovery that finds another branch checked out leaves its index alone.
  `PrivateCopy` is made with `O_EXCL` and never reused,
  so copies a killed run left can pile up (no cleanup yet).
  720 library and 70 binary tests,
  clippy clean.
  The fourth cross-vendor re-review (`~/temp/agent/reviews/follow-up-index-sol-4.txt`,
  2026-11-03T12:08:52-05:00):
  "Safe after named fixes,
  not safe to ship as shown".
  P1s:
  recovery can still overwrite later staging (a build or install error unlocks with a replayable intent;
  recovery unlinks a stale lock before taking its own;
  a panic in `record` unlocks),
  and an unreadable `.indexed` record is taken as absent.
  P2s:
  durability ordering of the install,
  record and done mark;
  the second HEAD check is still apart from the ref update;
  the private copy is filled at default permissions;
  the private name is not owned for the whole operation;
  branch-change recovery drops an alternate index's update and pending cleanup;
  HEAD read from the wrong worktree;
  sixteen abandoned copies exhaust the names for good.
  P3:
  a long alternate index name overflows `NAME_MAX`.
- Done:
  fourth round on the index writer,
  `perf/git-policy-cli-follow-up-index-in-process` `2dd575cfc1`,
  all ten findings red first,
  774 library and 70 binary tests,
  clippy clean.
  Recovery resets only paths whose entries still equal the `<seq>.start` record taken under the lock before the ref moves;
  `.indexed` is three-valued and recovery stops on an unreadable one;
  the index directory is synced before the record,
  and the journal directory before `.indexed` goes;
  the release disarms before syncing,
  so an error or a panic keeps the lock;
  an alternate index is still finished after a switch;
  HEAD is read from the request's own worktree Git directory;
  private copies live in an owner-only `cli-git-private-<uuid>` directory with an owner marker,
  swept only when its pid is dead.
  Finding 4 differs from the pick:
  Git 2.56.0 refuses `symref-verify HEAD` beside an update of its referent ("multiple updates for 'HEAD' ...
  are not allowed",
  probed),
  so HEAD is read again after the ref moved,
  under the lock,
  and a worktree that left the branch keeps its index;
  accepted under the delegation,
  veto open.
  The gate's ACL test now sets the default ACL by the `system.posix_acl_default` xattr (passed in the existing image).
  Present in 0.5.27 (tag `monochromatic-git-policy-cli-v0.5.27`):
  1(A),
  1(B),
  1(C),
  3,
  4,
  6 (worse),
  8;
  not 2,
  5,
  9,
  10;
  7 inverted (0.5.27 replays onto a switched worktree).
  Not fixed:
  the `remove_stale` check-then-unlink;
  a batch with no start record (a commit Git built) replays every path;
  two extra fsyncs per landing,
  unmeasured.
  It merged the base `14c26dbcc9` with no textual conflicts (`f6be7b42e2`);
  one semantic conflict found at compile time,
  the carried conflict stages against the private-copy directory,
  fixed in `e7e304ffd6` (stages written into the copy's own `index` before the release);
  794 of 795 library tests (`service_index_lock::tests::a_foreign_lock_is_described_with_the_process_that_has_it_open` failed once,
  then passed alone and in its 64-test module:
  a load flake to confirm in the gate)
  and 95 of 95 binary tests,
  clippy clean.
  The base was fast-forwarded to `e7e304ffd6`.
  Running,
  not an agent:
  the fifth cross-vendor re-review (`follow-up-index-sol-5.txt`).
- Check the container image `native:test:container` builds:
  the index agent reports a Debian trixie base,
  against the repository's rule of the newest CachyOS image (`doc/decision/container-base-distro.md`).
  Picked under the delegation (veto open):
  recovery resets a path only while its index entry still equals the one the landing started from,
  so a later `git add` survives however the lock handoff goes ("never discard user data");
  the HEAD expectation moves into the ref transaction (`update-ref --stdin` `symref-verify`,
  to be checked against the installed Git).
- Done:
  design revision 9 of `doc/planning/cli-git-commits-never-fail.md`,
  `a7b917e47e` to `5ff9706edc` (2026-11-03T11:44:10-05:00 to 11:54:15),
  answering all of the eighth review and the four model findings;
  the three choices it left were picked in `7afb60a024` (see "Picks made under the owner's delegation").
  The ninth cross-vendor review (`~/temp/agent/reviews/commits-never-fail-sol-9.txt`,
  2026-11-03T12:09:51-05:00):
  "Approve continued executable-model work and disposable backend experiments only.
  Do not approve the live protocol."
  Closed:
  the ungated restart,
  the shared ledger repair,
  shared candidates,
  the suppression example,
  the signal choice.
  P1s:
  C0 releases the lane lock while its client lives,
  so the next owner kills a healthy client (R9-1);
  a resend acknowledges a binding whose directory entry is not durable (R9-2);
  M1's journaled inode number is not deletion authority (R9-3);
  MG2 and MG3 erase unrelated staging and ignore a completed switch (R9-4);
  T1 assumes reachability holds through a maintenance run's earlier view (R9-6);
  and the shipped lock-removal step (R9-S,
  see "Shipped lock removal").
  P2s:
  merge staging captured before publish-begin and a reverse carry that is no inverse (R9-5),
  R4 cannot tell an erased suffix from none so the F1 closure is withdrawn (R9-7),
  E1 retries a refusal after a foreign change (R9-8),
  B3 with path-set changes (R9-9),
  undefined exits (R9-10);
  P3 rule S's arithmetic (R9-11).
- Done:
  design revision 10,
  `7afb60a024` to `dc6e809eca` (2026-11-03T12:22:02-05:00 to 12:27:13),
  answering all of the ninth review;
  new rule D (whoever relies on a visible name makes it durable first),
  rule N (the lane removes only names no foreign program creates),
  a recovery helper for C0,
  delta merge installs with attachment authority,
  a hold ref rooting each recovery copy,
  E1 by exit status first,
  amendments as fresh identities,
  a `resume` point on every waiting request,
  a durable hook result file.
  It states that shipped code should stop removing reference locks until a primitive that cannot revoke a replacement lock is shown;
  that is the open owner question in "Shipped lock removal".
  Its five choices picked in `eed561efaa` (see "Commits never fail,
  revision 10").
  The tenth cross-vendor review (`~/temp/agent/reviews/commits-never-fail-sol-10.txt`,
  2026-11-03T12:41:05-05:00):
  "Approve continued executable-model work,
  pure components without reclamation,
  and disposable backend experiments only.
  Do not approve the live protocol."
  P1s:
  an absent `index.lock` does not prove M1 installed the index (R10-1);
  W-detect and MG6 cannot restore after a detected switch (R10-3,
  so W-detect is not approved as specified);
  G-root's invariant is not established (R10-4).
  P2s include a recovery helper that can wait without end (R10-5)
  and P-wait being possibly indefinite,
  both now against the owner's no-unbounded-wait rule.
  Done:
  design revision 11,
  `eed561efaa` to `1430d7ddfa` (2026-11-03T12:48:49-05:00 to 12:54:14),
  answering the tenth review and applying the owner's bound answers:
  rule B registers every blocked process with its bound;
  W1 at a command's bound withdraws the request durably,
  keeps the commit on `refs/cli-git/<instance>/withdrawn/<id>`,
  leaves staging alone and exits non-zero;
  P2 is `update-ref --stdin` prepare,
  decide,
  commit,
  and W1 races the decision by `link` to `lane/decision/<id>`,
  so a request with a client lands or never lands;
  bounds START,
  FOREGROUND,
  MARKER,
  ACQUIRE,
  REAP,
  KILL,
  COMMITTING,
  STALL and PUSH start from shipped constants,
  labelled unmeasured.
  Its choices (`9c2b2ec0ab`,
  `e6199d4e4f`):
  the owner decided P-carry ("Merge anyway,
  unstage x") and W-detect with its residue ("Allow the rare leftover");
  the coordinator picked G-order,
  N-silent and H-episode.
  Running,
  not an agent:
  the eleventh cross-vendor review (`commits-never-fail-sol-11.txt`).
  Running:
  step 1,
  fifth round (revision 11 models with `explore_bounded` over every registered wait,
  the W1-versus-P2 race,
  the tenth review's controls,
  the components' third-review fixes,
  the per-site comment pass).
- Done,
  not merged:
  step 1,
  third round,
  `feat/git-policy-cli-commit-protocol-models` `27fa595034`,
  149 of 149 model tests,
  clippy clean.
  Revision 10 modelled for F1 (an earlier "green" relied on an oracle and is corrected),
  E1,
  F8 and M1 (each red under revision 9,
  green under 10);
  most of the ninth review's controls are not built.
  The components re-review's P1s are fixed (a durable chain through an anchor on every call;
  a canonical manifest checked against the bytes returned),
  with the P2s and most P3s;
  not done:
  a populated-submodule fixture,
  deterministic fault points,
  the comment-density pass.
  The components' third review (`commit-components-sol-3.txt`,
  2026-11-03T12:51:01-05:00):
  "Ready after named fixes".
  A and B are repaired provided the anchor is durable;
  to do:
  an initialized-root constructor that makes the root durable through a known-durable parent
  (P1 integration prerequisite),
  bare relative ledger filenames and ancestor durability before a repair (P2),
  gitlink and amendment eligibility and retention contracts (P2),
  the trace gated out of production (P3),
  a fully canonical manifest syntax or an accurate claim (P3),
  deterministic filesystem-ordering tests,
  and bumping `REQUEST_VERSION` for the new mandatory line.
  In step 1's fifth round.
  Done:
  step 1,
  fourth round (`bc3ff62cd4`,
  157 of 157 model tests,
  clippy clean):
  `explore_bounded` flags a wait longer than N of the protocol's own steps,
  or a wait with no own step left;
  `model_observe.rs` separates what a reader sees from what a power loss keeps;
  deterministic fault points before the link in `bind`,
  `publish` and the spool;
  a real populated-submodule fixture for `GitlinkNeedsAncestry`.
  The comment pass was a script adding one generic block per concept from a fixed list
  (F2 and F6 got none);
  a real per-site pass is still owed.
  The models' first review waits for revision 11's models
  (the models diff is 386 KB and must be split).
- Done,
  not merged anywhere:
  step 1,
  second round,
  `feat/git-policy-cli-commit-protocol-models` `52ccb3e38e`.
  Revision 9 modelled:
  each changelog schedule red under revision 8 and green under 9
  (F1,
  F3,
  F4,
  F6,
  F8,
  F9,
  plus new two-lane ledger,
  M1 kill-point and merge models),
  135 of 135 tests.
  It found one revision 9 defect:
  C4 answers `handed over` when its no-replace link finds a visible but not durable record
  (fix proposed as variant `NineFsyncOnExists`,
  sent to revision 10).
  Components review P1s fixed
  (type-aware replay,
  per-lane ledgers,
  durable directory creation,
  resends sync the name,
  request format version 2 with the base bytes in the spool);
  not fixed:
  byte paths,
  gitlink fast-forwards,
  `merge-tree -z`,
  race-test barriers,
  and the comment-density pass.
  Some red states were compile errors rather than failing assertions.
  The components re-review (`~/temp/agent/reviews/commit-components-sol-2.txt`,
  2026-11-03T12:21:53-05:00):
  "not correct".
  The type-merge P1 and the `.torn` ordering are fixed;
  per-lane ledgers hold under revision 9's single-appender contract.
  P1s:
  `create_dir_all_durable` stops at the first existing ancestor,
  so an ancestor another creator made but never synced can vanish after acceptance;
  the spool manifest's order and indices are unchecked,
  so a correctly hashed manifest can make `read_closure` return old and new bytes swapped,
  and an index such as `../other` is read as a path.
  P2s:
  an agreed type change (symlink to regular file on both sides) is refused;
  a relative path fails (`create_dir("")`);
  an empty commit and a message-only amendment are indistinguishable (no operation kind);
  v1 requests are rejected (fine before release).
  P3s:
  the oracle strips legitimate `~<hex>` names;
  the test sync counter runs in production code;
  the counter tests prove neither the target nor the order.
- Step 1,
  second round:
  its agent is resumed to model revision 9 (red under revision 8,
  green under 9 for each changelog schedule),
  build the eighth review's section-4 controls,
  raise the comment density,
  and then fix the components review:
  "not correct" (`commit-components-sol-1.txt`),
  P1s for a mode and object merged across file types (a symlink made from an edited file),
  ledger repair racing an append (answered by revision 9's per-lane ledgers),
  new directory names never synced into their parents,
  a resend acknowledging a link whose directory entry was never synced,
  and a spool that cannot be replayed once the base objects are pruned.
- cargo-shadow fixes for all three second-review parts,
  its agent resumed;
  it writes the three diffs for the third review to `~/temp/agent/reviews/cargo-shadow-3a.diff`,
  `-3b.diff` and `-3c.diff`.
- The word-level second merge pass (task 114),
  `feat/git-policy-cli-word-level-second-merge-pass` at `2a37bb7682`:
  the 0.5.28 base merged in (`9763599f3a`;
  in `merge_isolation.rs` a clean-index conflict still goes to native Git first,
  then the pass runs on the stopped merge);
  autostash and `branch.<name>.mergeOptions` verified;
  a pull that is stated to merge now runs as `git fetch` plus the isolated `git merge FETCH_HEAD`
  (`pull_isolation.rs`),
  because native `git pull` starts its merge as Git's own child,
  out of the wrapper's sight.
  Second round done (`411e98e732`) and merged into the 0.5.28 base (`9e0b06ad83`):
  the three picks built
  (an autostash whose apply would conflict leaves the merge stopped as Git does,
  the work kept in `MERGE_AUTOSTASH`;
  the pull's working-tree update waits for the index lock);
  a fast-forwarding pull skips the isolated attempt.
  Measured under the load there was (load average 9 to 11,
  debug build,
  nine alternating rounds,
  native-versus-native spread 2 ms or less):
  a merging pull 367 ms against native 14 to 16 ms,
  a fast-forwarding pull 128 ms against 11 to 12 ms;
  most of the merging cost is the isolated merge itself (a typed merge 302 ms).
  Answering the pull's queries in process is open.
  179 library and 83 binary tests,
  clippy clean.
  Its agent is resumed on `fix/git-policy-cli-conflict-warning-reads-the-content` from `9e0b06ad83`:
  the false conflict-marker warning in "Known flakes and failures not yet fixed",
  then carry-back A.
- Passed:
  a second early gate of the 0.5.28 base at `afc9a8621d` (after the waits,
  citations follow-ups and fallback merges),
  unit `gpc-gate-candidate-0-5-28-fallback-citations-20261103T114530-0500.service`,
  4468 of 4468,
  6 skipped,
  one slow.
- Failed,
  2 of 4575 (8 skipped):
  a third early gate at `9e0b06ad83` (after the index writer's third round,
  the lock-removal fix and the word-level pass),
  unit `gpc-gate-candidate-0-5-28-index-r3-locks-word-level-20261103T120421-0500.service`.
  `index_reset_file::tests::the_file_mode_under_a_default_acl_equals_git`:
  the gate's container has no `setfacl` (`NotFound`);
  sent to the index round 4 agent (add the ACL tools to the image).
  `git_stop_lock_tests::a_stop_without_grace_sweeps_the_locks_on_a_thread_of_its_own`:
  a regression from the lock-removal fix,
  passing on the host;
  the stopped child's lock and its pid file stay,
  read as the general judge declining inside the container's own PID namespace,
  so inside any container every Git child the wrapper ends would leave a lock.
  Fixed and merged into the base (`3c7aaa1122`):
  `fix/git-policy-cli-sweeps-its-own-reaped-childs-locks` `9176f9302c`.
  The reading was confirmed (the judge reports a PID namespace of its own and returns Unproven).
  The sweep passes the judge the ended members of the tree it reaped (`Judge.own_ended`);
  a pid file born at or after the lock,
  a lock born after the child started (`COARSE_CLOCK_SLACK` 50 ms back),
  and no process with that number again prove it gone in any process view,
  without the 4 s margin.
  Container run 2 red then 93 of 93;
  host 104 of 104;
  six negative controls for the container view.
  It adds occasions for the move-aside residue (R9-S) without changing the step;
  SPEC now states that residue beside the removal step.
- Failed,
  7 of 4596:
  a fourth early gate at `3c7aaa1122` (after the waits follow-ups and the regression fix),
  unit `gpc-gate-candidate-0-5-28-waits-own-child-locks-20261103T122526-0500.service`.
  The ACL test as expected (no `setfacl` in the image,
  with the index round 4);
  the stop-lock test now passes;
  and six `budget::` binary tests:
  "starts of git check-attr: observed 1,
  budget 0" on every ordinary commit,
  a regression from the conflict-warning fix (`conflict_markers.rs` asks for `conflict-marker-size` even when no path was conflicted).
  Its agent is fixing it on `fix/git-policy-cli-conflict-check-only-for-conflicted-paths`
  (start nothing new for an ordinary commit;
  budget constants unchanged).
- Done and merged into the 0.5.28 base (fast-forward to `d300bb806d`):
  `fix/git-policy-cli-conflict-warning-reads-the-content`.
  The marker claim reads the committed blob (`conflict_markers.rs`,
  honouring `conflict-marker-size`,
  default 7,
  a lone `=======` not enough);
  any named-path commit during a merge or cherry-pick prints one line saying how to finish it.
  Carry-back A:
  a conflicted autostash's `stash@{0}: autostash` and its unmerged stages reach the real repository
  (objects already land in the real store;
  `autostash_carry.rs`,
  stored once so a restart does not store it twice);
  removing either half turned its control red.
  134 library and 90 binary tests,
  clippy clean.
- Done:
  the index-lock and service waits follow-ups,
  `feat/git-policy-cli-index-lock-and-service-waits` `82633304d0`,
  each pick red first:
  unproven index-lock owners waited for with no bound (`UNPROVEN_OWNER_BACKOFF` and `unproven_backoff.rs` deleted);
  polling every 50 ms (`POLL_PAUSE`,
  classed exempt) where no inotify watch can be set;
  an `inactive` unit asked to start after each `START_WAIT`;
  a waiting command renews its scratch directory once an hour so the sweep keeps it.
  903 library and 92 binary tests,
  clippy clean;
  the real manager calls (`active_state`,
  `start_unit`) were not run against a real user manager.
  Open audit question:
  the heartbeat proves the service and its connection,
  not that the landing thread makes progress.
  It merged the base `d300bb806d` with no conflicts (`db0e92df64`),
  dropped its handover edit (`afbf402146`;
  the base branch's copy of this handover is stale,
  and main's copy wins when the base merges into main),
  and the base was fast-forwarded to `afbf402146`
  (260 wait,
  68 carry-back and conflict,
  29 binary tests;
  clippy clean).
- Passed:
  an early gate of the 0.5.28 base at its tip after the index writer's second round
  (unit `gpc-gate-candidate-0-5-28-early-20261103T111639-0500.service`,
  4424 of 4424,
  7 skipped,
  one slow).
- Done:
  the eighth cross-vendor design review (`~/temp/agent/reviews/commits-never-fail-sol-8.txt`,
  2026-11-03T11:33:50-05:00),
  with the shipped stale-lock code attached.
  Recorded in `doc/planning/cli-git-commits-never-fail.md` on `fix/git-policy-cli-commits-never-fail` (`a7b917e47e`,
  section on the owner questions):
  the shipped stale-lock rule does not survive (S-1,
  S-2,
  fixed on their own branch);
  B2 is withdrawn for B1,
  because a restarter's correct "no program running" observation can be followed by a hook starting,
  which the restart then ends (R8-1);
  A2 stays as an accepted narrowing:
  ending a stalled Git child that ignores the terminate signal can leave a lock only the person clears.
  The other findings need design revision 9:
  the ledger has several writers,
  C2 is not immutable,
  generations do not cover every state,
  X1's classification,
  M2 brings back a removed path,
  T1 conflicts with G3,
  and the state machine has gaps,
  merge among them (R8-2 to R8-8).
  Revision 8 `833b4ef80a` to `3f49d76ec6` (2026-11-03T11:04:37-05:00 to 11:06:22) is the normative state machine
  (L1 to L7,
  C1 to C5,
  B1,
  B2,
  P1 to P4 with recovery R1 to R5,
  M1,
  M2,
  T1,
  X1,
  G1 to G4);
  its two owner questions were picked in `3beecf2368`:
  A2 (kill a Git child that ignores the terminate signal,
  the owner's "End Git") and B2 (restart a quiet owner that holds the gate,
  against handing our own wedge to the person);
  a lock either leaves goes through the stale-lock rule shipped since 0.5.26,
  which revision 8's "the wrapper never removes a ref lock" contradicts,
  so the eighth review judges that shipped rule too.
- Done and merged into the 0.5.28 base (`a7a19b590a`,
  audit doc conflict resolved by keeping both sections):
  shipped-limits picks 4 and 9,
  `feat/git-policy-cli-steps-end-only-on-no-progress` `1d30940265`
  (Git steps,
  queries,
  the linter,
  the review gateway and the repair tool end only after a labelled unmeasured quiet window;
  `QUERY_CEILING` gone;
  every step now records its longest quiet gap in the timing records and the service journal).
  Picked under the delegation:
  collect quiet gaps from ordinary use and set each window above the longest by a margin the owner approves on return;
  redo a quiet step that is safe to run twice once;
  `update-ref` waited for,
  not ended,
  until the eighth review settles the lock question;
  the review's local log write left without a judge.
- Done and merged into the 0.5.28 base (`fc19017f67`,
  audit doc conflict resolved by keeping both agents' sections):
  shipped-limits pick 1 and the measured citations 7,
  8,
  13,
  14,
  15,
  `fix/git-policy-cli-limits-cite-measurements` `9b65660e95`
  (free space from the entry count,
  about 1 KiB of btrfs metadata per entry measured;
  ref-lock `MARGIN` 4 s from the longest journal `ref_ms` 3438 ms;
  `BOOT_SLACK` 3604 s;
  `REPOSITORY_MEMORY` and `ENTRY_MEMORY` 128 MiB;
  35 constants classed exempt;
  `MAX_BATCH` and `CLAIM_LEASE` have no data).
  Its follow-up questions,
  picked under the delegation (recorded in the audit doc,
  "Picked for these follow-ups"):
  measure the batch size and claim lease against a throwaway service;
  keep the 3 h idle exit;
  8 concurrent local queries and 4 remote;
  measure the copy-stage and install thresholds on the real stage;
  patience 1 h now,
  derived again when the index-lock wait pick lands;
  lock margin 7 s (twice the sample maximum);
  the preflight as built.
  Those follow-ups are queued for the next free slot (resume that agent).
  The gate owes its `native_binary` run:
  `binary_stale_ref_lock_tests.rs:151` now sleeps the margin plus 0.5 s.
- Not an agent:
  the cross-vendor re-review of the in-process index fixes (`pi`,
  output `~/temp/agent/reviews/follow-up-index-sol-2.txt`);
  the fixes are done (`dac87edc5e`,
  merged into the 0.5.28 base):
  the helper refuses to run unless the live `index.lock` marker names this process and batch,
  the landing holds that lock from the copy through the install (`service_land.rs:359`,
  `147-153`,
  `service_index_hold.rs:140-165`),
  one exclusively created temporary per run,
  Git's mode under `core.sharedRepository`,
  EWAH bitmaps checked and bounded,
  every protected-name spelling declined;
  the intent-to-add finding does not reproduce on Git 2.56.0 (a byte-for-byte differential pins it).
Queued,
in this order:
- Done (`cb2b9fe766`,
  merged into the 0.5.28 base with the no-`--quiet` test conflict resolved by keeping both sides),
  third re-review running (`~/temp/agent/reviews/follow-up-index-sol-3.txt`,
  with `service_land.rs`,
  `service_index_hold.rs`,
  `service_index_lock.rs`,
  `service_land_follow.rs` and `index_reset_publish.rs` attached whole):
  second round on the in-process index writer,
  from its re-review (2026-11-03T10:51:00-05:00,
  `~/temp/agent/reviews/follow-up-index-sol-2.txt`:
  "safe after fixing the remaining NTFS alias,
  correcting permission ordering,
  resolving the test-build/isolation issues,
  and verifying the cited outer locking and installation code";
  the temporary-file race and EWAH are fixed,
  and the intent-to-add finding is withdrawn):
  decline the hashed NTFS short name of `.gitmodules` (`GI7EBA~1`) with a real-reset differential;
  set the file mode before writing any byte,
  tested with a mode narrower than the umask gives;
  pass a borrow of the held `IndexLock` guard to the helper instead of a marker check;
  run umask-changing tests in a child process;
  assert the intent-to-add entry's flags,
  not the index version bytes;
  tests for injected write and sync failures,
  concurrent publication with distinct payloads,
  and an unselected change staged before the lock that must survive.
  The non-Linux test imports stay paused ("Ignore Win and mac until I say so.").
  Then a third re-review with `service_land.rs`,
  `service_index_hold.rs` and the lock-marker code attached whole.
- Third round on the in-process index writer and its outer landing code,
  first in line,
  starting now that the index-lock and service waits branch is merged (`177f9ebc50`);
  resume its agent with the findings in this entry.
  From the third re-review (2026-11-03T11:29:26-05:00,
  `~/temp/agent/reviews/follow-up-index-sol-3.txt`:
  "Safe after named fixes,
  not safe to ship as shown";
  the second round's fixes hold):
  - P1,
    also in shipped 0.5.27's outer code:
    `release_with` (`service_index_hold.rs` 168-172) installs the index and removes `index.lock` before `mark_done`,
    and recovery replays an open intent's reset,
    so a `git add` made between the unlock and a service death is overwritten;
    `follow_up` also installs and unlocks when conclusion cleanup returned false.
    Record index completion durably while the lock is held,
    and keep index completion apart from cleanup completion.
  - P2:
    the mode calculation `0o666 & !umask` (`index_reset_publish.rs` 98-110) ignores an inherited default ACL;
    create the file as Git would,
    read its mode,
    adjust before the first byte.
  - P2:
    `private_index_path` (`service_land_follow.rs` 173-176) can equal the live index under `GIT_INDEX_FILE`
    or another client's index;
    reserve the private copy by exclusive creation and refuse aliasing.
  - P2,
    also in shipped outer code:
    `head_names` (`service_land.rs` 233-243) is checked before waiting for the index lock;
    revalidate after acquisition and before the ref move,
    and give recovery a rule for a branch no longer checked out.
  - P3:
    retry `Interrupted` writes;
    restore the deleted `later_status_and_diff_are_right_after_a_reset_without_a_refresh` coverage and the racy-entry
    fallback assertion;
    a reader-start handshake in the concurrent publication test.
  - Not a blocker for this Linux-only release:
    the reclamation check-then-unlink in `service_index_lock.rs` 285-289 needs a per-physical-index single-reclaimer
    guarantee or a test;
    the non-Linux test imports stay paused.
- The fifth index re-review once the fourth round writes `follow-up-index-fixes-4.diff` and its `index-r5-` files.
- Running:
  every wait in the wrapper gets a bound (the owner's "nothing should ever wait forever or indefinitely"),
  the waits agent on `fix/git-policy-cli-every-wait-has-a-bound` from `14c26dbcc9`:
  an inventory in `doc/planning/cli-git-no-unbounded-waits.md` first,
  then each bound with a red test,
  values measured from the timing logs and journals,
  unclear end behaviors left as choices for the coordinator.
  A blocker for 0.5.28.
- Done and fast-forwarded into the 0.5.28 base (`14c26dbcc9`):
  `fix/git-policy-cli-conflict-check-only-for-conflicted-paths`;
  an ordinary commit starts no `check-attr` again (the six `budget::` tests red on the base,
  green after;
  16 of 16 with the `commit_during_merge` controls).
- GitHub issue #6260 (2026-11-03T16:23:05Z,
  no comments):
  the source scans must find nested modules and key identities and exceptions by relative path.
  Running:
  a new agent on `fix/git-policy-cli-source-scans-cover-nested-modules` from `14c26dbcc9`.
- GitHub issue #6261 (2026-11-03T16:23:43Z,
  blocked by #6260):
  group the 1,012 flat files under `src/native/` into subsystem directories and weigh dropping the `native` layer,
  keeping the file-size rule.
  Queued after the 0.5.28 release and the open branches merge,
  because it moves every file;
  then the lint-debt pass (task 88).
- Retire option B of the autostash pick now that carry-back A exists,
  so the wrapper's finish ends a conflicting autostash as Git does
  (merge commit made,
  stash kept,
  `UU` entries),
  instead of leaving the merge stopped (picked under the delegation:
  "Auto merging should never fail.").
  Done and fast-forwarded into the 0.5.28 base:
  `fix/git-policy-cli-finish-applies-a-conflicting-autostash` `2b5e99f5a8`
  (`autostash_blocks_finish` removed;
  merge and pull controls compare with a twin repository on which Git stops,
  the person resolves and `git commit` concludes;
  the wrapper had ended with status 1 where Git's commit gives 0;
  6 of 6 autostash controls plus one for unsaved changes to a stash file;
  44 library and 41 binary tests,
  clippy clean).
- A gate after the regression fix,
  the index round 4 and the waits follow-ups merge,
  then release 0.5.28.
- After step 1's third round:
  the components' third review from `commit-components-3.diff`,
  and the models' first cross-vendor review from `commit-models-1.diff` (split if it exceeds about 250 KB).
- After the tenth design review:
  revision 11 if it needs one.
- Done:
  the cargo-shadow fix round for all three second-review parts,
  `feat/cargo-shadow-cli` `ce9a17d7d7`,
  354 of 354 tests,
  clippy and `monochromatic-lint` clean (20 new `avoidable-closure` warnings that capture variables).
  Ownership of an output file now comes from one place (`file_owner.rs`):
  a changed file whose inode change time falls in a window around the cargo call is this run's,
  any other is another writer's and is never restored over
  (blind spot:
  a writer within about one scheduler tick of the call,
  which loses no data because every displaced file is kept).
  The restore is a preservation protocol (`RENAME_EXCHANGE`,
  a hard link of what came out before any unlink,
  undo on a mismatch,
  no destructive fallback).
  The Git reader is a lexer after Git's `config.c`,
  proves commits from loose objects or pack entries,
  delegates everything else,
  and falls back with one `git cat-file --batch-check` start;
  a 20-case differential suite against `/usr/bin/git`.
  Not fixed:
  the system config path is assumed to be `/etc/gitconfig`;
  power loss is untested.
  The third cross-vendor review,
  in three parts (all finished within the hour):
  3a,
  the Git reader (`~/temp/agent/reviews/cargo-shadow-sol-3a.txt`):
  "still not equivalent to Git";
  P1s:
  a symbolic-link `HEAD` is read as detached,
  remote `"."` is not the merge spelling returned literally;
  P2s:
  trailing path characters,
  the object reader proves a type marker rather than a readable commit,
  a fatal batch error can drop an existing upstream,
  configuration validation narrower than Git's,
  the system config path,
  Git's dubious-ownership rejection bypassed;
  P3 an unreaped child on a pipe-write failure.
  3b,
  journal and ownership (`-3b.txt`):
  seven P1s
  (the undo deletes an inode another writer can have open;
  per-root manifest recovery fails open;
  `begin` can truncate an earlier transaction's only snapshots;
  a `/proc` lookup failure can call a live owner dead;
  claim-by-rename does not stop directory-name reuse or a live transaction;
  durability ordering incomplete;
  a missing manifest index still authorizes deleting manifest snapshots);
  P2s:
  `ctime` cannot establish ownership,
  `atomic_write` uses a non-exclusive temporary,
  a concurrent deletion is undone.
  3c,
  install and policy (`-3c.txt`):
  "not ready for 0.1.0";
  fix the refusal-preservation path,
  the verification scope of ordinary updates and the placeholder race first.
- Running:
  cargo-shadow's fourth fix round from the three third-review parts,
  with the owner's "nothing should ever wait forever or indefinitely" and the error-at-the-bound answers applied to its waits;
  it writes `cargo-shadow-4a.diff`,
  `-4b.diff` and `-4c.diff` for the fourth review.
- Gate the 0.5.28 base once the index round and the lock-deletion fix are merged into it
  (and the word-level merge pass,
  if it is ready by then;
  it does not hold the release),
  then release 0.5.28,
  install it and time commits against native Git.

0.5.27 ships without the in-process index work
(coordinator's pick under the delegation,
veto open:
the cross-vendor review of it,
2026-11-03T10:09:59-05:00,
`~/temp/agent/reviews/follow-up-index-sol.txt`,
says "safe after named fixes,
not safe as-is",
and its measured wall-clock gain was nil,
a median 139 ms against 137 ms before and 38 ms native on a throwaway copy of main's shape,
so holding back the rest of 0.5.27 for it buys nothing).
Its branch is `integrate/git-policy-cli-0-5-27-without-index-in-process` at `00259fff5e`,
worktree `.claude/worktrees/integrate-0-5-27-without-index`,
gate unit `gpc-gate-release-0-5-27-without-index-20261103T101115-0500.service`
(log `~/temp/agent/gate/gpc-gate-release-0-5-27-without-index-20261103T101115-0500.log`).
The gate on the combined tip `3848f9581d` was stopped by the coordinator at 3908 of 4338 tests passed.

The review's findings on `perf/git-policy-cli-follow-up-index-in-process` (`48d6abb28c`),
queued first for the next free slot,
with red tests first:
- P1:
  `index_reset_file.rs` writes every run to the same `<file>.cli-git-reset` name and removes it unconditionally,
  so two runs can publish each other's partial file;
  the writer takes no `index.lock`,
  so the outer protocol must be shown to hold the live `index.lock` from the read through the install.
- P2:
  an intent-to-add entry whose blob equals the tree's (an empty file) is kept with its flag,
  where `git reset --no-refresh` writes an ordinary entry (`index_reset.rs` 212-215).
- P2:
  `path_is_plain` accepts names Git refuses under `core.protectNTFS`
  (for example `.gitmodules::$DATA` as a link);
  port Git's checks or decline those settings and names.
- P2:
  a malformed EWAH bitmap of a few bytes expands without a bound and can exhaust memory before the fallback
  (`index_ewah.rs` 175-224);
  validate and bound by entry or directory count.
- P2:
  the rewritten file takes the process umask,
  losing `core.sharedRepository` modes.
- P3 (non-Linux test builds,
  paused by the owner's "Ignore Win and mac until I say so."):
  Linux-only fixture imports.
- Also asked:
  SHA-1 reference vectors (empty,
  `abc`,
  lengths around block edges,
  over 512 MiB),
  and tests of later `status` and `diff` after `--no-refresh` and zero stat data.
It found no mismatch in the quiet check for ordinary entries,
and no SHA-1 arithmetic or padding defect.

The combined branch `integrate/git-policy-cli-plain-commit-no-quiet-index-in-process` (tip `3848f9581d`,
worktree `.claude/worktrees/integrate-plain-commit-no-quiet-index`) becomes the 0.5.28 base once those fixes land.
It adds to 0.5.27's content
the follow-up `reset` and `diff-tree` and the `diff-index --cached --quiet` checks in process
(the remaining Git `reset` calls pass `--no-refresh`;
merge conflict with the no-`--quiet` branch resolved as `reset --no-refresh` with both branches' tests kept).
0.5.27's content:
plain commit over unmerged files,
no inner `--quiet`,
the service unit without limits,
the audit rotation fix,
the flaky-test fixes,
the doctor fsmonitor line (`072ac84868`)
and main up to `a9e81617a1`.
0.5.27 waits for its gate,
then merges into main,
is released,
installed,
and measured on main against native Git.

## Open work

Each item names what remains,
where it lives and its evidence.

### Verify 0.5.26 against native Git

Measured with 0.5.27 installed (`~/temp/agent/tools/observe-main-commit.ts`,
host under its usual load):
the handover commit `90f5d99c36` on main at 2026-11-03T15:47:03Z took 324 ms wall,
with no memory-cap hit in the service cgroup and one new major fault;
native Git on a throwaway copy of main's shape takes a median 38 ms
(in-process index branch's measurement),
so the wrapper adds about 0.3 s against the owner's five-second bound.
One sample so far;
the next commits on main add more.

The owner's standing complaint (2026-11-03T07:38:40-05:00 entry):
wrapped commits on main must not take five seconds longer than
native Git.
Before 0.5.26:
on main a commit waited over 50 s in the client's `git add --all -- <path>`,
20 s in `git read-tree --reset`,
over 50 s in a merge's `diff-index`,
and 120 s in a full commit (two Git children of about 60 s inside a 119 s `index.lock` hold;
why each took 60 s is unexplained).
The perf branch changed this (throwaway of main's shape:
wrapped commit median 651 ms to 437 ms,
native 63 ms).
Measured with 0.5.26 installed (2026-11-03T08:27:22-05:00):
the lock commit `c76a6b6d3e` of one file took 66.8 s,
of which the service's follow-up `git reset -q <commit> --pathspec-from-file=-` ran 59 s inside the `index.lock`
hold (`lock_hold_ms=59122`,
`sync_ms=59118`);
the next `git merge` waited 21 s for that commit and then over 40 s in the wrapper's
`git diff-index --cached --quiet HEAD`.
Owner: "Can't we skip the git reset?"
Answer:
the index entries must be updated (or the next commit reverts the committed paths),
but no Git needs to run for it.
Agent on `perf/git-policy-cli-follow-up-index-in-process` writes the committed entries in process,
as `builtin/commit.c` does for a partial commit,
and answers the merge's `diff-index` check in process,
with differential tests against real Git and a measured before and after.
Service journal 2026-11-03T08:55 to 09:20 (main instance):
eight one-file commits landed in 43 to 52 ms each (`sync_ms` 20 to 27,
so the service's `git reset` took milliseconds);
only `3d138d5bc6` (09:19:18) was slow,
with `sync_ms=50370` in the service's `git reset`,
and in the same commit the client's `git read-tree --reset` ran over 50 s and its `git add --all -- <path>` over
40 s.
Three separate Git processes each stalled about 50 s in one commit,
so the cause is shared by them in that window,
not a slow step of the wrapper;
the observer now records each Git process's current system call (`/proc/<pid>/syscall`) to find what they block on.
Observed 2026-11-03T09:21:59-05:00 (commit `53f96ecc24`,
7.7 s wall):
from 0.5 s to 7.0 s the wrapper's Git child (`/usr/bin/git -c core.hooksPath=...`) sat in `wait4` on its own
child `git fsmonitor--daemon start`,
which was on the CPU with a `fsmonitor--daemon run --detach` beside it;
the commit finished half a second after that start ended.
A Git process of another session showed the same wait.
Stopped there,
because the owner's rule is to never chase Git's fsmonitor as a cause
(see "Git's own fsmonitor");
the observation is raised with the owner.
An observed one-file commit on main at 2026-11-03T08:42:37-05:00 took 343 ms
(`~/temp/agent/tools/observe-main-commit.ts`),
with no new cap hit and one new major fault in the service cgroup,
so the slow case is conditional;
the 66.8 s commit was the first after the service started,
at host load 18 to 57.
Found 2026-11-03T08:38:42-05:00 to 08:40:51-05:00 (open, being observed):
plain `/usr/bin/git` on main runs `diff-index --cached --quiet HEAD --` in 12 ms and
`--no-optional-locks status --porcelain=v1 -z --untracked-files=no` in 10 ms,
both inside and outside the shared Bash cgroup
(`~/temp/agent/tools/main-git-timing.ts`),
so the 40 s and 30 s wrapped runs are not Git's own work.
Index 2.8 MB with 20,720 entries;
3 packs and 8,506 loose objects.
Main's commit service cgroup
(`cli-git@fc85fdf7a58f866b7f7a84b44fe0b69a.service`,
started 08:26:19)
had hit its `MemoryMax=1G` (`src/native/service_units/cli-git@.service`) 274,736 times,
with 16,125,924 major page faults;
the service slice as a whole 582,767 and 29,926,443.
Inference until observed:
the service's Git children thrash against the 1 GiB cap.
The client-side slow steps ran in `claude-code-bash`,
which never hit its cap (50,897 major faults),
so they need another explanation.
Git's `builtin/reset.c` lines 500-504 refresh the whole index unless `--no-refresh`,
and its "It took N seconds to refresh the index" advice (line 507) is suppressed by `-q`.
The main service's `memory.peak` read 1,073,766,400 bytes,
at the cap.
Owner, 2026-11-03:
"Why did you cap the commit service to the magic number of 1GB of memory w/o evidence?"
Answer:
an agent wrote `MemoryMax=1G`,
`CPUQuota=200%`,
`TasksMax=512` and `IOWeight=50` in `e95a43090c` (2026-11-01T08:26:09-05:00) with no measurement,
apparently carrying the agent-command cgroup rule over to the shipped unit;
its test checked only that the lines existed;
the coordinator merged it without catching that.
The owner chose "Remove all three (Recommended)" for the other limits and "Add it (Recommended)" for a rule.
Done:
branch `fix/git-policy-cli-service-no-memory-cap` (red `45a7590037` and `b539a588bf`,
fix `741d42932e`,
SPEC `474c987ae2`),
unit-template and provisioning tests 31 of 31,
merged into the 0.5.27 integration branch;
rule `LQM` on main `708fbd7e47`.
The shipped-limits audit is on branch `docs/git-policy-cli-shipped-limits-audit` `7387f29a52`
(`doc/planning/cli-git-shipped-limits-audit.md`):
73 limits,
4 cited,
7 partly,
58 uncited,
one dead constant (`DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS`);
conflicts with decisions:
the 1800 s live-holder wait against `doc/decision/cli-git-concurrent-commits.md` lines 151-158 and "Same as the ref
lock",
`COMMIT_WAIT`'s message naming a setting that does not exist,
main's push window and ceiling against "Wait while connected",
retry counts on causes the owner put on the designed-out list.
Fifteen owner questions are in the doc;
not yet asked.
Remaining:
merge the audit doc,
ask its questions,
and measure the service's commits on main once 0.5.27 is installed.
Remaining after that:
measure commits and merges on the real main checkout against native Git,
under the load there is,
and report counts.
Plan:
`doc/planning/cli-git-main-commit-within-native-plus-five.md`.
The gate step still owed for 0.5.26:
confirm the container branch of
`a_fetch_after_a_killed_fetch_clears_the_locks_where_the_holder_is_provably_gone` ran.

### Commits, merges and pushes that never fail

Branch `fix/git-policy-cli-commits-never-fail`,
design `doc/planning/cli-git-commits-never-fail.md` (on the branch).
Design summary:
remove the class-bound kill and move the slow follow-up steps in process;
the client lands a commit itself when the service is absent;
heartbeats instead of `COMMIT_WAIT`;
the service is the one writer of its worktree's branch;
move the ref first and reconcile the index when the lock is free;
a per-worktree hold ref keeps captured blobs from pruning.
The cross-vendor review returned:
`~/temp/agent/reviews/commits-never-fail-sol.txt`,
verdict "not ready to implement as written",
four Critical and five High findings:
a deferred merge checkout can overwrite work created after the merge was checked;
a deferred index update can erase another program's staged content;
the client fallback creates another coordinator without an ownership protocol;
foreign lock reclamation lacks a sufficient proof;
moved-head stacking leaves content replay underspecified;
hold refs do not establish the pruning guarantee;
some items remove an error return but not the liveness failure;
the native fallback is not an end-to-end escape path;
additional refusals and native failures are not classified.
The revised design had a second cross-vendor review:
`~/temp/agent/reviews/commits-never-fail-sol-2.txt`,
verdict "Not ready for approval",
four Critical and five High findings (file headings:
an epoch check does not fence an obsolete worker;
carry-forward does not protect writes through an already-open descriptor;
the off-lock index build can erase unrelated foreign staging;
current ancestry cannot show whether publication already happened;
acknowledgement and transaction completion are conflated;
pending requests need a logical head;
the spool does not establish recoverability after pruning;
the side-ref path does not remove every lock dependency;
remaining Git hangs are acknowledged,
not designed out;
pending-ref non-transmission is an assertion without enforcement).
Foreign-lock reclamation is closed,
and captured-change replay and the refusal classes are substantively corrected.
Third cross-vendor review:
`~/temp/agent/reviews/commits-never-fail-sol-3.txt`,
verdict "Not ready to build as the live transaction protocol",
four Critical and eight High findings
(receipt does not prove publication;
receipt deletion loses deduplication;
the ref fence does not cover obsolete worktree,
index and cleanup work;
index checksum is not provenance;
every displaced inode needs retention;
capture and takeover need ref writes a foreign lock blocks;
the fence ref is repository-wide;
`queue/<n>` is not fenced acceptance;
logical head needs a logical-index model;
index extensions cannot be kept byte for byte;
spool reclamation needs an overlapping-prune proof;
push enforcement,
hooks and deterministic hangs).
Revision `a7bfea3b4a` and `d428b27385` (2026-11-03T08:47:23-05:00 and 08:48:42) removes the features instead:
no live takeover,
receipt,
fence ref,
numbered queue,
checksum provenance or base hold ref;
one kernel lock per worktree held on an open file description that Git children inherit;
a systemd watchdog ends a wedged owner;
acceptance writes no ref;
publication is a compare-and-swap `update-ref`;
retirement writes an fsynced ledger line first.
The agent's five questions for the owner,
held until the fourth review answers whether each narrowing is needed:
lock per worktree or a service-only lane;
a signed commit whose parent moved waits for a new signature;
a Git child that hangs twice on identical input leaves the commit pending with exit 0;
a prune that overlaps a request and deletes after the lane's check is treated as corruption by another tool;
after a crash on a repository with no branch reflog,
a published commit removed by a non-descendant rewrite reappears once.
Fourth review returned 2026-11-03T09:05:02-05:00:
`~/temp/agent/reviews/commits-never-fail-sol-4.txt`,
verdict "not ready to build as the live transaction protocol";
one Critical (carry-forward crash recovery deletes an editor's newer inode)
and five new High (kill-before-acquire and unsupervised inline owners;
the watchdog is a time bound that can repeat a hook;
lockfile contents do not prove ownership;
dry-run expansion differs from the real push;
reachability does not make an old index intent current).
It judges the lock-versus-service question an implementation choice,
and only the signed-commit,
prune-boundary,
deterministic-Git-bug and no-reflog cases owner decisions.
Fifth revision `6d727bcab0` and `42aff9b39c` (2026-11-03T09:08:32-05:00 and 09:09:29):
no kill-before-acquire,
no unit watchdog or takeover,
no lock sweeping after an owner crash,
a permanent ledger,
an unconditional `^refs/cli-git/*` on every push,
deletion only of inodes the lane created and journaled,
a per-path consumption map,
worktree attachment and generation on each request.
Its own trade-off,
not an owner decision:
a wedged owner has no automatic recovery and the person restarts the unit,
which the coordinator flagged against "never stuck".
Its four owner questions:
signed commit with a moved parent;
a Git hang on identical input (the owner's "embed instead of spawning" may already decide it);
an overlapping destructive prune;
no-reflog ambiguity.
Fifth review returned 2026-11-03T09:23:28-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-5.txt`):
not ready;
open:
publication inferred from an incomplete reflog or a reused commit id,
staging intent inferred from stat equality (a tree-created zero-stat entry can be recreated exactly),
the attachment check not synchronized with the mutation,
deferred intents against a foreign successor commit,
descendants of an inline owner holding the lane lock forever,
age cleanup deleting the only saved copy of a carried edit,
maintenance admitted at an idle lane but not covering its lifetime,
a hook's external effect repeated by recovery after an owner death.
Its judgments:
no construction recovers a silent wedge without a time bound and also never repeats a hook's effect,
and manual restart cannot silently satisfy "never stuck";
the deterministic Git hang is settled by "embed instead of spawning";
corruption by another tool after release is on the outside list,
but the wrapper's own maintenance must be coordinated over its lifetime;
signed replay and no-reflog ambiguity are owner choices.
Coordinator adoptions,
veto open:
re-sign a replayed signed commit in the background with the configured signer,
as the service already signs its reconcile merges;
saved copies of carried edits are kept until explicit disposal ("never discard user data").
Owner answers 2026-11-03 (question tool):
stuck step "End Git, hold hooks (Recommended)" (a Git child with no progress for a measured interval is ended and its
step redone;
the person's hook is never ended,
its commit is set aside as pending with a notice and the others keep landing on top);
deferred staging "Update the index (Recommended)" (a deliberate restage to exactly the old entry inside a foreign
lock's window is replaced,
accepted as undetectable);
unclear publication "Land it (Recommended)" (landed with a message naming it and how to drop it).
Sixth revision `766f623dd0`,
`851aa5b446` and `d09c58e7d1` (2026-11-03T09:41:10-05:00 to 09:42:05):
a closure line for every fifth-review finding;
publication evidence only after a reflog anchor recorded before the swap
(no anchor:
owner answer "Land it");
staging targets the current tip's entry;
the attachment check brackets reads of `HEAD` and `logs/HEAD` around the index read under `index.lock`;
inline owners move into a transient scope;
`hook-dispatch` journaled before `update-ref`;
wrapper maintenance runs recorded in `lane/maintenance`.
Still open in it:
the embedded path for a deterministic Git hang,
the exit inventory,
and four owner questions,
picked under the delegation in `75cf58f345`
("Picks made under the owner's delegation").
Sixth cross-vendor review started 2026-11-03T09:50-05:00
(`~/temp/agent/reviews/commits-never-fail-sol-6.txt`,
prompt `commits-never-fail-prompt-6.txt` beside it).
Sixth review returned 2026-11-03T10:33:37-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-6.txt`):
"Ready to build step 1:
executable models and disposable crash fixtures only.
Not ready to build the live protocol."
Its findings (P1 unless marked):
- F1 (P2):
  an intact reflog anchor does not prove the entries after it are complete;
  treat missing evidence as ambiguous ("Land it" with the notice),
  never as "not published".
- F2:
  a dead inline owner's descendant can hold the lane lock,
  and the next owner may stop that scope only after acquiring it,
  so cleanup can deadlock;
  a closed-description descendant can still mutate after a new owner acquires;
  needs a lifecycle authority that reaps a proven-dead owner's scope without the lock.
- F3:
  restarting a quiet owner (question 2,
  A) races a hook's dispatch,
  and a delayed restart can hit a replacement owner;
  restart must be serialized with hook admission and the owner's incarnation.
- F4:
  stopping `update-ref`,
  or restarting its owner,
  leaves a branch lock the wrapper caused and recovery refuses to sweep;
  "stop and redo" and "never reclaim" do not compose.
- F5 (P2):
  the HEAD bracket is a guard,
  not a fence;
  reflog growth is neither a complete switch detector nor proof of a switch.
- F6:
  reading the same state version twice allows a mixed snapshot
  (a materializer pausing before it advances the version makes a plain commit capture `x=1` and revert `R2`);
  needs a seqlock-style in-progress marker before every relevant mutation.
- F7:
  renaming a temporary request over `inbox/<id>` lets a second client replace an accepted request with the same id;
  bind each id to its content without overwrite across inbox,
  queue and ledger.
- F8:
  maintenance records per worktree miss the shared object store
  (a prune started from worktree A deletes objects of B's request);
  needs an object-store-wide,
  crash-safe registry.
- F9 (P2):
  hooks held while others land need an explicit ordering exception,
  including the held request later refusing or succeeding.
- On question 1:
  keep A;
  the coordinator's "record restaged paths" is not adopted as a closed solution;
  versioned restaging events (resulting entry,
  operation,
  consumed or not) are an optional later improvement,
  with its listed cases (`git add` of unchanged content,
  `add -p`,
  `restore --staged`,
  `stash`,
  `commit -a`,
  aliases,
  bypassing writers fall back to A).
- Questions 2 to 4:
  no conflict with the owner's rules,
  given F3 and F4 for question 2.
- P3:
  several operative sections still describe superseded behavior;
  one authoritative rule each for cancellation,
  inline-owner lifetime,
  optional index locking,
  background re-signing and attachment generations.
Revision 7 `e2c2d9d8cc`,
`2b65f37d52` and `b658f5f01f` (2026-11-03T10:45:46-05:00 to 10:47:38):
a closure for each finding
(evidence classes with everything unproven landed with the notice;
children never hold the lane lock and the next owner reaps the recorded scope before mutating;
a `lane/gate` lock shared around every person's program and ref transaction,
taken exclusively by the restarter,
which ends the owner through a pidfd;
one cancellation protocol,
terminate first,
then remove only a lock absent before launch and born inside the child's life;
a seqlock for capture;
a permanent `ids/<xx>/<id>` binding created with `O_EXCL` under `$GIT_COMMON_DIR/cli-git/`;
one maintenance registry per object store;
held requests out of the logical tail);
no new owner questions.
Seventh review returned 2026-11-03T11:00:08-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-7.txt`):
"step 1 remains the right first build.
R7 is not ready for the live protocol,
and passing step 1's models alone would not make it ready."
New P1s from revision 7's own mechanisms:
reaping a recorded service cgroup that may hold the replacement owner (it kills itself);
deleting a lock born inside a stalled child's life (it can be a foreign writer's;
"Do not implement this deletion rule.");
an `O_EXCL` identity binding that can be partial,
or complete with no request published;
an empty or absent maintenance scope counted as finished,
and process-group escape by `setsid`;
the capture seqlock's odd generation held across a hook that may wait for ever;
the gate's restarter dying with an asynchronous stop outstanding,
a wedged shared holder,
and a PID read across namespaces.
Also:
F1 positive evidence must outrank a recorded `swap-failed`;
F5 immediate materialization needs the attachment guard;
F9 one materialization-aware consumption release and versioned completion for superseded held requests;
seven operative contradictions (its section 3).
Its section 4 lists what the models must express and the properties they check.
Revision 8 running:
the design agent writes one compact normative state machine with named transitions,
removes the birth-time deletion rule,
and states narrowed guarantees plainly where only waiting on something outside our control would close them,
with any real owner choice as a question.
The step 1 agent has the seventh review's schedules as additional red controls.
Step 1 building:
branch `feat/git-policy-cli-commit-protocol-models` from the 0.5.28 base `5e8c96de21`,
worktree `.claude/worktrees/commit-protocol-models`;
its first output is the choice of model-checking approach
(a new crate is reported for a pick before it is added).
Step 1 is:
failing models for F2 to F8 with the review's schedules as red controls,
the hook-bypass ordering model,
immutable request,
spool and permanent ledger components with no reclamation,
pure tree comparison and replay with no live mutation.
Not enabled until the models pass:
automatic restart,
production pending publication,
live carry-forward,
retention deletion.
The exit inventory stays a gate.
Interim code in 0.5.26 (index-lock wait and a bounded moved-head loop) stays until the by-construction design replaces
it
(owner:
"Ship interim now (Recommended)").
The 2026-11-03 question-tool answers listed in `SPEC.md`,
"Merges and commits never fail",
still need building:
"Save on a side ref" (commit meets a live foreign ref lock),
"Same as the ref lock" (live foreign `index.lock`:
a commit lands and exits 0 and the index catches up when the lock frees,
a merge saves on a side ref and finishes the checkout when the lock frees),
"Allowed refusals" (the person's hooks and required filters),
"Carry edits forward" (merge over unsaved edits),
and moved head stacks a new commit (amend,
merge conclusion,
revert).
Unexplained and still open:
why `diff-tree` and `reset -q --pathspec-from-file` took about 60 s each in the 120 s commit
(read Git's source and cite file and line,
per the owner's "git itself being OSS" rule under "Owner decisions in force").
Candidate cause,
not confirmed:
with `core.fsmonitor = true`,
a Git command that finds no daemon starts one and waits up to `fsmonitor.startTimeout`,
60 s by default (`builtin/fsmonitor--daemon.c` lines 39-40 in 2.56.0),
the same length as each step.
Confirming it needs a trace of such a step waiting on the daemon,
in a disposable repository with fsmonitor set in its own config;
fsmonitor is now off globally,
so main no longer shows it.

### Plain commit with unmerged files finishes the merge

Branch `fix/git-policy-cli-plain-commit-finishes-conflicted-merge`,
for 0.5.27.
Owner answer:
"Warn and finish the merge (Recommended)":
a plain `git commit` with unmerged files lands as the merge commit with
the markers and warns per file.
Finished at `a06af6102c` (2026-11-03T08:16:26-05:00),
host suite 3606 and binary suite 605 passed on its agent's run.
Remaining:
merge into the 0.5.27 candidate,
gate,
release.

### No wrapper `--quiet` anywhere, and pushes that wait while connected

Branch `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits`.
Owner answers:
"Drop it everywhere (Recommended)" (no `--quiet` on any Git run the wrapper starts:
maintenance,
auto-push fetch,
fast-forward merges,
reset,
clean,
init) and "Wait while connected (Recommended)" (count bytes on the push's network socket as
progress through TCP counters,
because `push_progress.rs` notes `rchar` and `wchar` miss `send` and `recv`;
once the pack is sent keep waiting while the connection is open,
with a line naming the wait).
Built and finished by its agent at tip `e38a7dea6a` (2026-11-03T08:24:13-05:00),
main merged in as `0fec1a97e6`.
The wrapper's own `git push` no longer sets `GIT_HTTP_LOW_SPEED_LIMIT` and `GIT_HTTP_LOW_SPEED_TIME`
(`lfs_bounded_push.rs`),
and since `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx` (in the 0.5.28 base) the auto-push reconcile fetch
sets none either.
A value the person set reaches Git unchanged,
and Git reads `http.lowSpeedLimit` and `http.lowSpeedTime` itself
(Git 2.56.0 `http.c` lines 453-460,
1477-1478 and 1218-1223).
Red `f0c0ad7112`:
a push whose hook sleeps past the old 60 s cutoff failed with curl's "Operation too slow";
fix `fb1e093809` makes it land;
re-adding the call made it fail again.
Control:
a person's 2-second setting still ends a 12-second silent wait with Git's error.
Checks on the tip:
clippy clean,
host suite 3634 of 3634;
the binary suite had 3 failures at host load average 57,
two passed alone,
and `a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait` failed twice alone at load 38-44
and passed at load 11
(see "Known flakes and failures not yet fixed").
Unknown:
whether GitHub sends keepalives (Git's `receive.keepAlive` default is 5 s).
Not exercised:
IPv6 sockets,
real GitHub keepalives,
real `ssh://` or `git://` pushes.
Remaining:
merge into the 0.5.27 candidate,
gate,
release.

### Word-level second merge pass

Branch `feat/git-policy-cli-word-level-second-merge-pass` `8cc36cd4b6`,
not merged.
Built:
`word_merge.rs` on `similar` 3.2.1 (Myers),
equal to Jujutsu 0.46.0 on the stored corpus;
runs in a person's merge,
rebase,
cherry-pick,
revert and `git pull`,
in the isolated merge,
in the auto-push reconcile merge
and in the commit replay;
last recorded checks host 3442 and binary 595 passed,
clippy clean.
Remaining:
the last gate copy failed two tests that main had already fixed
(`worktree_add_plan` dash-h test and a `supervisor_guard` harness race),
so merge current main,
regate,
merge,
release.
Follow-ups the branch records as done (commit `8bc4b70b7a`,
"finish autostash and mergeOptions...",
and the former log),
not verified by me:
finishing `--autostash` and `branch.<name>.mergeOptions` cases as Git does.
Follow-up not known to be done:
`git pull` that merges goes through the isolated merge.
Decisions:
see "Merge auto-resolution" under owner decisions in force.

### Cargo shadow and the Rust catalog tools

Branch `feat/cargo-shadow-cli` `504a82aa3f`:
182 tests at the last report,
clippy clean,
`monochromatic-lint` 0 errors
(26 `avoidable-closure` warnings left,
18 fixable).
Cross-vendor review returned 2026-11-03T08:40:49-05:00:
`~/temp/agent/reviews/cargo-shadow-sol.txt`,
verdict "Do not rely on this implementation as a policy-enforcement boundary yet",
with P1 findings (a refused version still runs through the `--locked` fallback;
the freshness stamp skips the policy and base checks;
non-frozen installs skip existing entries;
unapproved lockfiles are visible to other invocations;
registry identity is dropped;
the lazy owner lookup misses a downgrade;
unreadable timestamps grant the age exemption;
rollback and recovery can overwrite another writer's lockfile or lose the last copy;
`/proc` liveness;
`--breaking` edits outside the transaction;
the missing-lockfile placeholder;
the recursion-guard kind;
`-m` and `--package=` spellings),
P2 findings (CSV EOF and row width,
unbounded parser memory,
the global cutoff defeating exemptions,
set-aside range) and a P3 README contradiction.
Owner answers of 2026-11-03 (question tool):
"Try asking better. pnpm nowadays just warn and runs when a dep could be updated to something published within a day."
Adopted from that and pnpm 12.16.1 (`pickRespectingMinReleaseAge`;
no message found in its source):
the install step sets a newly needed release under the age limit aside for the newest older passing release and runs.
For a committed Cargo.lock holding a release under a day old (case 1) and one holding a trust downgrade (case 2),
both answered "Set it back, build (Recommended)":
the shadow changes that entry to the newest older release that passes,
prints one line,
and builds;
it stops only when nothing older fits.
Scoped to local runs;
CI keeps the frozen behavior;
operational failures keep warn-and-run with `--locked`.
Done on the branch,
tip `569596c984` (243 of 243 tests,
clippy clean):
every review finding and both answers,
red tests first.
Its nine questions for the owner,
not yet asked:
under `trustLockfile = true` with base `HEAD` a committed merge's lockfile is already base,
so cases 1 and 2 do not fire locally (compare with upstream,
check every entry locally,
or keep);
release age versus trust under `trustLockfile`;
CI still warns and runs `--locked` on a refusal;
the window between cargo writing the live lockfile and the checks;
the restore read-then-rename gap;
strict cutoff with exemptions;
`--breaking` sync failure;
a killed `--breaking` run's manifests;
whether the trust line must name the publisher account.
Remaining,
in the order the agent's report adopted:
1. the review fixes and the two answers (agent running), then a second cross-vendor review;
2. publish `monochromatic-dependency-catalog` 0.2.0 (publishes itself on merge through `cargo-publish.yml`);
3. rebuild file-enforcer-cli with it and run `toolchain:update`,
   then land `912e82a0c7` (`trustLockfile = true` in `rust-catalog.toml`;
   the installed file-enforcer rejects that key until it is rebuilt);
4. first hand-publish of `monochromatic-cargo-shadow` 0.1.0 (`doc/runbook/publish-crate-first-time.md`),
   its entry in
   `toolchain.plan.jsonc`,
   `toolchain:update`,
   check the launcher-backed link;
5. workflow commit:
   move `advisory-core.yml:170` (`task-run rust:verify`) to the shadow's own command and drop the `rust:*` mise
   tasks and `task.ts` entries (owner:
   workflows are editable like any file;
   workflows must not use mise);
6. frozen-install trust check with `trustLockfile` true checks only versions new since `CARGO_SHADOW_TRUST_BASE`
   (unset:
   `HEAD`);
   0 findings over 35 lockfiles at base `9ef2c198cf`,
   5.6 s warm.
The first design used pnpm's default `trustLockfile: false` and refused 22 existing versions;
`doc/decision/rust-dependency-catalog.md` lines 89 to 95 (amendment of 2026-10-29) say only new versions are checked,
the counterpart of `trustLockfile: true`.
A bug found by using the tool (a single-workspace `cargo update` pruned all `trustPolicyExclude` entries) was fixed in
`6a9afd9f59`.

### Conversion of the remaining worktrees and its small fixes

`git cli-git convert-worktree-to-subvolume` ran for the real repository twice (0.5.20,
0.5.21).
At the last dry run with 0.5.21 (`~/temp/agent/convert-dry-run-0.5.21.log`):
183 looked at,
164 subvolumes,
18 would convert,
1 skipped for real content (`follow-ups-059`:
`package/forbidden-strings/core/Cargo.lock` differs from the commit).
The second real run converted 17 and skipped 3:
`fast-snapshot-add` (edited file,
running tests),
`follow-ups-059`,
and a throwaway.
Remaining:
- convert the worktrees still ordinary once their holders are done (busy worktrees are refused by the tool itself);
- the dry-run summary reads "0 converted or completed" beside "would convert" lines;
  it should count what would convert (small fix,
  not done);
- sweep the conversion journal's `cli-git-convert.tmp.<pid>.<nanos>` leftovers,
  and read the migration journal without following
  a link (`Journal::load` uses `read_to_string`);
- count existing worktrees whose `node_modules` links are broken (made from main after main moved to pnpm's global
  virtual store,
  before the package repair shipped in 0.5.21) and ask the owner before relinking worktrees that belong to other
  sessions.

### Worktree add follow-ups

An add of about 60 s is acceptable to the owner and sets no target to stop at ("60s is fine for adding worktrees").
Not done:
overlap the inline trash disposal with the package repair;
on a post-snapshot mismatch of a main source,
keep the snapshot and take the restore path instead of the full copy;
make the snapshot rung settle racily clean entries by content (as the conversion does) instead of falling to the slow
route;
parallelism audit of loops over independent items (branch `perf/git-policy-cli-parallel-independent-work` merged the
first round;
the audit's remaining findings are not recorded here).
Unmeasured:
whether a later `pnpm install` in a copy made without `storeDir` writes into the source's in-tree store.
The owner's statement of accepted risk for races with other programs ("That is not an our problem...",
see "Worktree add and package managers" under owner decisions in force) is to be stated
plainly in command help,
in a short notice when a destructive command runs,
and once for the whole program in `README.md` and
`SPEC.md`;
a search of both files on 2026-11-03 found no such statement yet.

### Service follow-ups

Pre-existing,
from the lock-removal pass,
probably moot if the commits-never-fail design lands (unsure):
the scratch ref `refs/worktree/cli-git-service/<seq>` that a killed landing leaves,
and the `claimed/` scan that is quadratic for a drain of N claims.
`started` and `ready` service lines now carry distinct journal times;
lines of a retiring older service still share one stream timestamp (nothing to fix).

### Gate and tooling

- Gate-slot and campaign-lock claim files are merged,
  with the 2026-11-03 liveness and signal fixes (see "Open review findings" for what remains).
- Mutation campaigns for wrapper code merged since the last recorded campaigns (conversion,
  snapshot add,
  package repair,
  lock
  removals,
  push patience,
  stale ref locks,
  word-level pass,
  commit speed) have no recorded campaign in this page's sources
  (unsure;
  check `package/git-policy/cli/target/verification/` and the sibling handovers before assuming none ran).
  Campaigns are run by the main agent only.
- Controls for `subvolume_platform.rs:76` and `:80` that need no btrfs landed (`b9826f2d3f`);
  other btrfs-only tests still skip in the gate container,
  so a btrfs host run of the ladder is a standing gap.
- Older umbrella items from October,
  still unchecked in the former queue,
  with no later closing record found (all unsure):
  the scanner pathname mutation scope on Windows without a timeout
  (`doc/handover/scanner-native-verification.md`);
  the Windows checks and release-artifact performance gates that trail the cutover;
  long fuzz campaigns;
  mutation campaigns of non-core modules (hooks plumbing,
  auto-push,
  automatic maintenance,
  signals);
  the linter cutover task,
  the slice 6 convergence mutation rerun and the post-cutover non-core campaigns
  (the coordinator's task list names them as 24,
  25 and 29);
  reconcile each against Git state and either close it with evidence or move it here.
- macOS is not a release target (the GitHub release carries Linux assets only);
  the `native_binary` tests still fail there (`/dev/full` is absent,
  several fixture spawns fail with NotFound);
  the lib tests pass on `m1`.
- Android check (`ANDROID_HOME`) now gets its variables from the mpm env fragment;
  the Android-target `cargo check` is blocked by a compile error in `turso_sync_engine` 0.8.2 under nightly-2026-11-01
  ("cycle detected when computing when turso_parser::ast::Select needs async drop"),
  not yet reported upstream or worked around.
- `package/mpm-shadow/cli/tests/shim.rs::passes_arguments_and_exit_code_through_then_prunes` was a pre-existing
  failure when last
  recorded (unsure whether fixed).
  The nested-launch guard and the EAGAIN retry of issue 6259 are covered by package tests but were never exercised
  live.

### Known flakes and failures not yet fixed

- Shipped in 0.5.27,
  seen live 2026-11-03T12:00 on the 0.5.28 base:
  `git commit -m <subject> -- <path>` during a merge,
  with the conflict resolved in the file but not staged,
  landed a one-parent commit of that path (`f1f78f7920`,
  as SPEC designs:
  "Named paths during a merge: Git refuses; the wrapper lands them and the merge is carried on")
  and printed that the path "still has conflict markers,
  and they were committed",
  though the committed file has none.
  `service_prepare.rs:445` lists every path that was unmerged in the index before the commit,
  whatever the content holds.
  Its advice (another `git commit -- <path>`) also leaves the merge open.
  The merge was concluded with a plain `git commit` (`cfd30ccd18`);
  a corrective comment is on `f1f78f7920`
  (<https://github.com/Aquaticats/Monochromatic/commit/f1f78f7920e379b3d45893e25739a689be60331e#commitcomment-207965325>).
  Fix running with the word-level agent.
- `a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait` failed twice alone at load 38 to 44
  (`objects_ms=6760`,
  `attempts=2`:
  the stand-in holder let go before the commit's wait) and passed at load 11;
  an agent is making the holder release only once the commit is observably waiting,
  on branch `fix/git-policy-cli-foreign-index-holder-test-waits-for-the-wait` (started 2026-11-03).
- `lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other` failed
  once in a full
  binary run,
  then passed 15 of 15 alone and in later full runs.
- `fault_limits::a_descriptor_limit_hit_at_any_open_leaves_a_recoverable_repository` ran over 60 s in two separate
  runs
  (passed;
  not looked at).
- Rule for all of these:
  the owner's "Fix flaky tests properly."
  (recorded 2026-11-02T21:49:28-05:00).
  Never rerun to chase one.

### Questions waiting for the owner

- Container disk weight:
  about twenty other packages and two workflows start containers at the default weight;
  the weight constant has several copies;
  `vm-builder`'s rootful privileged bootc-image-builder run has no CPU,
  memory or process limits.
  The six packages in the owner's answer "Both flags in all six" are done.
- Delete the thirteen merged branches with nondescript names
  (`fix-push-semantics`,
  `fix-review-log-common`,
  `fix-takeover-hook-race`,
  `integ-20261101`,
  `follow-ups-059`,
  `normal-conditions-gaps`,
  `fix-service-queue`,
  `integrate/cli-git-native-command-parser`,
  `perf-harness-service`,
  `test-hang-bounds`,
  `fix-lfs-hermetic-20261101`,
  `integrate-slices`,
  `lint-lockfiles`),
  locally and on origin:
  the auto-mode classifier refused ("Git Destructive").
  Eighty-eight other nonconforming branches belong to other work (audit `~/temp/agent/branch-name-audit.txt`).
- Whether an operator escape hatch should force the native route when the commit service misbehaves
  (the owner refused install switches;
  a debug override would be a different thing).
  Not built.
- Windows and macOS CI effects of the shared pnpm `storeDir` and `virtualStoreType: global` are paused per the owner.
- Whether the 2026-10-13 standing authorization to bump and publish the wrapper still holds beyond that session (see
  working rules).

### Housekeeping

- Remove the stale detached worktrees listed by `git worktree list` under `.claude/worktrees/` when their evidence is
  no longer
  needed (needs the owner's word for deletions);
  the stale integration branch named under "Unmerged branches and worktrees of this effort" goes with them.
- Test services that crash on purpose left hundreds of coredumps a day;
  fixtures expected to abort were told to set `RLIMIT_CORE` 0
  (test harness only).
  Whether that landed is unconfirmed:
  check `coredumpctl` after 0.5.26.

### Final re-review

The owner's words,
2026-11-01T09:22:48-05:00 (when the entry was written;
the owner spoke earlier that day) on the original entry ("Final re-review"):
"You yourself do a re-review of the entire thing when all tasks are drained.
During the re-review:
Assume nothing on disk ever makes sense and I answered every question with via magic 8 ball."
Reading,
revisable by the owner:
it starts when every delegated agent has reported and its branch is merged or rejected,
gates are green,
no release is in flight
and every task-list and handover item is done or recorded as blocked with its reason.
The coordinator session does it,
not a delegate (a cross-vendor pass may run alongside,
never instead).
Scope:
the `git-policy/cli` wrapper and everything built for it (crate,
service,
embedded linter seam,
fuzz and mutation sidecars,
gate and release tooling,
planted controls,
docs).
Every comment,
doc,
test name,
commit message,
config and recorded result is an unverified claim checked against behavior run in a
throwaway fixture or against the deciding source;
a test counts only after a planted fault shows it failing;
recorded owner decisions carry no evidence of being right,
so judge each on its merits and ask the owner with options,
pros,
cons
and a ranking where judgment differs;
never change decided behavior without asking.
Output:
findings report in `doc/audit/` with evidence per finding,
clear bugs fixed with proofs,
flagged decisions to the owner.

## Open review findings

Source:
`/var/home/user/temp/agent/reviews/p1-reconciliation.md` (read-only,
160 P1 findings across 40 returned reviews;
131 fixed in main,
8 fixed only on an unmerged branch,
6 withdrawn or void,
1 accepted limit,
11 open,
3 unknown).

The audit-rotation race and the four gate-slot findings were fixed with red tests on 2026-11-03:
the wrapper fix `fix/git-policy-cli-audit-rotation-race` `2eb2cf1bfc` (lock-free:
one rename takes the name,
a mistaken take goes back by `hard_link`,
a rotation error never costs the record;
host 3611,
binary 603) is merged into the 0.5.27 integration branch and ships with it;
the gate fix `fix/mutation-test-cli-cargo-mutants-claim-liveness-foreign-namespace` `db01385349`
(no age rule,
foreign-namespace owners found in `/proc` by inner id and start time,
termination forwarded to the whole process group,
the legacy `flock` held beside the claim) is merged into main;
its trade-offs are in `package/mutation-test/cli-cargo-mutants/README.md`.
Still open from them:
- a kill between the rotation's two renames leaves an orphan `.moving-*` file that audit readers do not read
  (no line lost);
- the next rotation still replaces `.1`,
  dropping older audit history as before;
  a question for the owner (does "never discard user data" cover the audit history?),
  folded into the shipped-limits audit;
- a non-file at `slot-N.lock` makes `holdLegacyLock` return `absent`,
  which the gate treats as an earlier-release holder forever.

Accepted limit with a documented remedy,
no code fix:
downgrading to 0.5.5 revives the per-user writer
(`baefc60463`;
remedy:
stop `cli-git@*.service` and `cli-git@*.socket` first).

On the unmerged copy-in-service branch `worktree-agent-ab3dd6294f9d91284` (head `fe2081f2a1`,
19 commits not in main):
9 open (`review-wtc` findings 3,
4 and 6 and `review-wtc/v3` findings 1 to 6) and 8 fixed only there.
The branch may be moot:
the btrfs ladder replaced the copy on btrfs hosts,
but main still has the older lock-based copy modules,
which no review covers.
No later decision to merge or drop it is recorded.
The v3 decision "do not fix these on the current
repository-wide layout" (2026-11-01) stands until then.

Not P1-labelled:
the four Critical findings of the commits-never-fail review are open by construction (see "Commits,
merges and pushes that never fail").

## Picks made under the owner's delegation

Owner, 2026-11-03 (about 09:45-05:00):
"I have to be away for around 8 hours. Pick options as if I had picked them and invite veto when I say I'm back."
Clarified by the owner:
"\"As if I had chosen\" means what I would have picked most likely if I'm always here."
So each pick is the option the owner would most likely have chosen:
the ranked-first option where its ranking follows the owner's standing rules,
which matches every answer the owner gave on 2026-11-03.
Every pick below is the coordinator's,
made under that delegation;
each stays open to the owner's veto and is listed for the owner on return.

### Shipped-limits audit (`doc/planning/cli-git-shipped-limits-audit.md`, fifteen questions)

1. Which limits need a citation:
   B,
   limits that can change a command's result or duration;
   polls,
   display caps,
   chunk sizes and reader fallbacks are classed once as exempt with the reason (they still follow MGN).
2. Index lock holder:
   A now (a proven-alive holder is waited for with no bound and a line naming it;
   an unproven owner gets 1000 ms of backoff and then a diagnostic;
   the dead constant goes),
   then B with the commits-never-fail design (the owner's "Same as the ref lock").
   Follow-up pick (built on `feat/git-policy-cli-index-lock-and-service-waits` `82633304d0`):
   an unproven owner is waited for with no bound like a proven one,
   with a repeating line that says what is known and that removing the named file lets the command run
   if no program uses the repository;
   a dead owner (a pid file naming a gone process) still ends the wait at once.
   Its cost:
   a lock left by a killed program with no pid file now waits until a person removes it.
   Built 2026-11-03 on `feat/git-policy-cli-index-lock-and-service-waits`, with three differences the owner may veto:
   the 1000 ms is cited from a measurement of Git's own commands (at most 100 ms, ten times that),
   a dead owner ends the wait at once,
   and the service's own hold keeps a measured 150 seconds (`UNASKABLE_HOLD_AGE`)
   because live holds ran up to 120 seconds.
3. Retry counts:
   C with the commits-never-fail design (stack on the new head for every moved branch);
   the interim loops stay until then ("Ship interim now").
4. Git child steps:
   B,
   end only on no progress with the push's signs and no wall clock,
   matching the owner's "End Git, hold hooks".
5. Pushes:
   A,
   "Wait while connected" whole:
   no stall window for a connected push,
   no ceiling,
   no low-speed abort.
   Built:
   `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx` `3ae733155f`,
   merged into the 0.5.28 base (`5299b6d102`);
   connected means a process of the push holds an `ESTABLISHED` TCP socket (read through sock_diag).
   Its three questions,
   picked:
   the 30 s window of a push with no open connection stays and is measured once logging records quiet time
   (its option C over A,
   because A leaves the value uncited);
   its own extension of the connected wait to the reconcile fetch and the LFS upload stays
   (both are part of the push,
   and a clock there brings back the failure the owner's answer removed);
   `QUERY_CEILING` (a wall-clock stop on read-only queries such as `git status`) moves to picks 4 and 9,
   and the stop grace and the 4 MiB output cap are measured once logged.
   Cost of the rule,
   from its agent:
   a live remote that holds the connection without answering holds that repository's pushes until the remote or TCP
   ends it.
6. Waits on the service:
   A,
   heartbeats and no client wall-clock bound;
   the `COMMIT_WAIT` message naming a nonexistent setting goes.
   Built 2026-11-03 on `feat/git-policy-cli-index-lock-and-service-waits`:
   a heartbeat a second, five seconds of silence asks again with the same request,
   and the start waits on the service manager's `failed` state where the client can ask it
   (the open questions are in the audit doc).
7. Foreign-owner patience:
   C,
   measured from the logs.
8. Parallelism caps and memory estimates:
   A,
   measured.
9. Deadlines of tools that are not Git:
   A,
   end only on no progress.
10. Saved conflict copies:
    C,
    defaults set from the measured growth.
    Outcome:
    no saved copy exists yet in any of the 11 repositories under the home directory,
    so there is no growth to measure;
    the agent's O3 is taken
    (keep the owner's 2026-11-02 "Prune by both age and size." with 30 days and 1 GiB,
    measure,
    then set the numbers),
    because the owner chose pruning knowing a saved copy can be the only copy
    (`doc/handover/cli-git-logging.md`,
    "Saved conflict copies");
    nothing can be pruned for 30 days.
    Question for the owner on return:
    "never discard user data" was first recorded 2026-11-03T00:08:39-05:00 (`d4aab88c70`,
    snapshot leave-out),
    after the pruning decision;
    does it override pruning a saved copy that is the only copy of a merged file?
    Options and ranking:
    `doc/planning/cli-git-shipped-limits-audit.md`,
    "Only-copy finding".
11. Audit and timing logs:
    C,
    audit generations kept up to a measured size and never dropped while they name a saved copy that exists;
    timing logs keep one generation.
    Built:
    `feat/git-policy-cli-saved-copies-and-audit-generations-measured` `5f453574e5`,
    merged into the 0.5.28 base;
    4 MiB derived from the measured busiest day (about 103 KB a day) times the 30-day copy age.
12. LFS server errors:
    HTTP 5xx classed as outside our control (a server fault,
    like "Network down"),
    retried by the service's push retry plan alone (option B's single mechanism).
    Built on the same branch as pick 5;
    a typed `git push` has no plan,
    so it now ends at the first 5xx with the explanation.
13. Service housekeeping:
    A,
    measured from its journal and timing logs.
14. Lock-removal margins:
    A,
    measured on this host.
15. Free-space preflight:
    A,
    computed from the entry count with a measured cost per entry.
Implementation queue (agents start as the five slots free;
measurements under the load there is,
never synthetic):
pushes and LFS (5,
12);
Git child and tool no-progress ends (4,
9);
index lock interim and service waits (2,
6);
audit generations and saved copies (10,
11);
the measured citations (1,
7,
8,
13,
14,
15).

### Commits never fail, revision 6 (four questions)

The questions,
the reasons and the coordinator's candidate are in `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail`.

1. Staging a path back to its consumed entry while commits are pending:
   A (the commit lands without that path;
   a notice says so).
   The coordinator's candidate goes to the sixth review:
   record paths the person explicitly restages through the wrapper while requests are pending,
   so a plain commit gives native Git's result.
2. A quiet service owner of our own making:
   A (the waiting command restarts the unit after the measured interval),
   shipped only once the interval is measured.
3. A hook with an external effect on "prepared",
   after an owner death:
   A (run `update-ref` again;
   the hook may post twice).
4. The instant inside Git's `checkout` between the index write and the `HEAD` change:
   A (accept a rare staged change on the new branch).

### Commits never fail, revision 8 (two questions)

In `doc/planning/cli-git-commits-never-fail.md`,
the owner-questions section,
on `fix/git-policy-cli-commits-never-fail` (`3beecf2368`,
`a7b917e47e`).
These are options of the owner questions,
not the state-machine transitions of the same names;
revision 9 renames them.

1. A Git child that ignores the terminate signal:
   option A2 (end it with the kill signal,
   the owner's "End Git"),
   kept as an accepted narrowing after the eighth review:
   it can leave a lock that only the person clears.
2. A quiet owner that holds the gate:
   option B2 (restart it) picked first,
   then withdrawn for option B1 (hand it to the person with a line naming it),
   because the eighth review showed a restart can end a person's hook that starts after the "no program running"
   observation (R8-1).

### Commits never fail, revision 9 (three choices)

In `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail` (`7afb60a024`).
Options of the revision 8 questions are renamed A-keep,
A-kill,
B-hand and B-restart.

1. What the stall rule (X1) may cancel:
   C-kind (by command kind,
   `update-ref` exempt) over freezing the dispatch inputs and over re-checking at cancel time,
   because of "End Git,
   hold hooks";
   a stalled `update-ref` with no hook,
   which needs a Git bug,
   stays named and durable for the person.
2. When a held request's hook runs:
   once per version (H-version) over once per build,
   so a slow hook is not rerun each time another program moves the branch,
   and nothing waits without end.
3. Merge order:
   check out first,
   then swap (M-A),
   over swap first,
   whose window lets a plain commit capture the reverse of the merge ("never discard user data").

### Commits never fail, revision 11 (five choices)

On `fix/git-policy-cli-commits-never-fail` (`9c2b2ec0ab`,
dated in `e6199d4e4f`).

1. A merge touching a path the person has staged:
   decided by the owner (question tool,
   answered before 12:56:51-05:00):
   "Merge anyway,
   unstage x" (P-carry;
   the staged content becomes an unstaged edit merged onto the result),
   over refusing at once like Git.
2. A `git switch` finishing after the lane's last check during merge file writes:
   decided by the owner:
   "Allow the rare leftover" (W-detect;
   the owner was told a `git commit -a` in that window could commit the merge's files to the wrong branch).
3. Retention against late maintenance:
   G-order (the connectivity check,
   then a re-read of the maintenance registry,
   then the release;
   the hold ref only as extra protection),
   coordinator's pick.
4. A live owner's retry after a proven store failure:
   N-silent (no notice on an ordinary held-lock retry),
   coordinator's pick,
   recorded as a concession.
5. A hook rerun after repeated owner deaths:
   H-episode (one rerun per recovery episode,
   with a notice),
   coordinator's pick.

### Commits never fail, revision 10 (five choices)

In `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail` (`eed561efaa`).

1. A stranded `index.lock` the lane's own dead process made:
   never removed (R-never),
   reported with the incarnation that made it,
   over pinning the inode and unlinking,
   which keeps the pathname race of the shipped sweeper.
2. A merge that touches a path the person has staged:
   it changes nothing and waits (P-wait,
   Git itself refuses this merge),
   over turning the staging into an unstaged edit.
3. A `git switch` finishing between the merge's guard and its file writes:
   detect after and restore from pinned files (W-detect),
   over never writing merge files automatically,
   because of "Auto merging should never fail."
4. Keeping the recovery copy against a late `git gc`:
   a hold ref (G-root),
   over making maintenance wait on a live lane;
   the prune view including `refs/cli-git/*` must pass as a step-1 experiment first.
5. Merge order:
   check out first,
   then swap (M-A),
   kept after the ninth review's repairs.

### Shipped lock removal (four questions)

On `fix/git-policy-cli-never-removes-a-live-lock` (`9f92dae98c`).

1. A lock a reboot left without a pid file is now reported with the `rm` line,
   not cleared:
   kept (picked),
   because the wrapper sets `core.lockfilePid` for every Git it starts,
   so only a kill between the lock and its pid file,
   or a foreign Git without the setting,
   leaves one;
   a rule keyed to a recorded boot identifier could not cover locks made before the record.
2. The sweep after a stopped child judges every reference lock of the repository (picked),
   as each fetch and push already does,
   over only those born during the child's life.
3. Its notice goes to the diagnostic log on standard error (picked,
   unchanged),
   because the sweep runs on a background thread.
4. Its new lines over 120 characters:
   left for the lint-debt pass (task 88).
5. The ninth design review's R9-S:
   the removal step (rename the lock to a private name,
   compare,
   remove or put back),
   shipped since 0.5.26,
   can break mutual exclusion:
   if between the judgement and the move another cleaner removes the judged lock and a live Git makes a new one,
   the move displaces that live lock,
   and a third Git can take the path before the put-back,
   so two Git processes hold the same ref lock.
   No Linux call removes or moves a path only if it still names a given file.
   Picked:
   keep removal only on a proof about the holder,
   with the second identity check just before the move,
   and ship it in 0.5.28,
   because it removes strictly fewer locks than 0.5.27 does;
   over never removing a reference lock,
   which makes every Git the wrapper ends leave a lock the person must `rm` before the next fetch,
   push or commit ("Commits should never fail.").
   Question for the owner on return:
   accept this residue,
   or never remove reference locks (the design's X1 already says the wrapper never removes one).

### Shipped-limits follow-ups

- After the citations:
  picked in `doc/planning/cli-git-shipped-limits-audit.md`,
  "Picked for these follow-ups";
  built (`e314460f79`).
- After the index-lock and service waits (`008b74d3a3`):
  an unproven lock owner is waited for like a proven one,
  because an editor-held `git commit` would otherwise make ours fail after 1 s;
  poll when no inotify watch is available;
  ask the service manager to start an inactive unit after `START_WAIT`;
  keep a scratch file from being aged out during a wait longer than a day.
- For the owner to approve,
  not picked:
  the margin above the longest recorded quiet gap for each quiet window (picks 4 and 9),
  and the 150 s hold-age margin (pick 2).

### Fallback to the installed real Git (four questions)

In `doc/planning/cli-git-no-self-invocation.md`,
"Falling back to the Git the commit service was set up with";
built on `feat/git-policy-cli-falls-back-to-installed-git` (`6453c380b1`).

1. Fall back at all:
   yes (picked) over keeping the stop,
   because a found wrapper that cannot run Git is a failure the owner's rules name,
   and the one-line notice keeps the cause visible.
2. Which record names the real Git:
   the commit service's unit file,
   whose `ExecStart` carries `--git <path>` (picked),
   over a new record written on every successful `PATH` search
   (survives logout but adds a write to every command),
   the version-cache entries (several,
   unordered)
   and a fixed `/usr/bin/git` (a guess).
   Limit:
   the unit lives in the runtime directory,
   so before the first commit of a login,
   and after the last logout,
   the stop stays.
3. The service's own checks of its `--git`:
   left as they are (picked);
   the wrapper always writes `--git` from its checked path,
   so tighter checks guard only older or hand-written units.
4. How often the notice prints:
   on every command (picked),
   because printing once per login needs a file that outlives the process and a rule for when it expires.

### Word-level second merge pass and pull (three questions)

In `doc/planning/cli-git-merge-auto-resolution.md`,
the section dated 2026-11-03T11:45:26-05:00,
on `feat/git-policy-cli-word-level-second-merge-pass` (`2a37bb7682`).

1. A stash whose autostash apply conflicts is lost:
   the shadow repository keeps `refs/stash` and the unmerged entries,
   where Git keeps `stash@{0}: autostash` and `UU` entries.
   This also happens to a person's own `git commit` after `merge --autostash`,
   which has shipped.
   Picked:
   B now (a dry-run of the stash apply hands the wrapper's finish back to native Git when it would conflict),
   then A after the index writer's third round (carry the stash,
   its objects and reflog entry,
   and the unmerged stages back into the real repository under the landing lock),
   which also fixes the shipped path;
   over leaving it documented,
   because of "never discard user data" and "Fix even pre-existing issues of the pkgs you own."
2. A pull with nothing stated,
   and `--ff-only`:
   left to native Git's whole pull (picked),
   because its refusals are deliberate and the isolated merge already forwards fast-forwards.
3. The working-tree update after a fetch that moved the checked-out branch:
   wait on the index lock before it (picked),
   over leaving native Git's race,
   which a loaded host hits and the never-fail rules cover.

### Release contents

- 0.5.27 ships without the in-process index work,
  which waits for its review's fixes and ships in 0.5.28;
  reasons under "Agents and gates running".
- The fallback to the installed real Git was queued for after 0.5.28;
  it finished first and is merged into the 0.5.28 base (`afc9a8621d`),
  so it ships in 0.5.28 (coordinator's pick:
  the owner's "Ship interim now" favours shipping a finished fix).

### cargo-shadow (agent's nine questions)

1. Committed lockfile under `trustLockfile = true`:
   local runs compare with the upstream branch (`@{upstream}`,
   else `origin/<default>`) instead of `HEAD`,
   so cases 1 and 2 fire for versions not yet on the upstream;
   CI keeps its base.
2. Release age and trust both apply to versions new since the base,
   on both the local and the frozen path.
3. CI stops on a policy refusal (pnpm's frozen install fails on a lockfile entry the policy rejects).
4. The window in which a cargo that is not the shadow builds on an unapproved lockfile is accepted and documented
   ("Use as few locks as possible").
5. Restore uses `renameat2` with `RENAME_EXCHANGE` and an undo,
   through a safe wrapper (no lock,
   no lost writer).
6. Strict cutoff with exemptions:
   keep resolving without the cutoff and enforcing it on non-exempt crates.
7. A `--breaking` sync failure after the catalog write restores the catalog (transactional).
8. A killed `--breaking` run's manifests go in the journal.
9. The trust-downgrade line names the earlier and the new publisher account when known,
   as the owner's chosen option said ("naming the publisher change").

## Owner decisions in force

Timestamps are the commit times at which the decision was first written into the former handover
(`git log -S` over that file);
the owner's own time of speaking is recorded only where the entry gave a date.
A decision already recorded in a canonical doc is a pointer,
not a copy.

### Nothing waits forever

The owner,
2026-11-03 at about 12:37-05:00 (said in the session,
not yet in SPEC):
"nothing should ever wait forever or indefinitely".
The owner added that they are not completely back,
so the delegation continues.
What happens at a bound,
answered by the owner through the question tool at about 2026-11-03T12:45-05:00:
"Return with an error"
(the option read:
"The command stops waiting and exits non-zero,
naming what blocked it;
the saved work stays saved."),
then,
asked whether the saved commit should still land by itself later,
"No,
keep it for a retry (Recommended)"
(the option read:
"Nothing lands later by itself.
Your staged changes stay staged,
and the commit is also kept on a side ref.
The error names the blocker and the command to run again.").
This replaces the coordinator's first reading (return as success and land later),
and the owner chose it over "Commits should never fail." for a command that hits a bound.
Pushes,
same question tool:
"Bound it,
keep retrying (Recommended)"
(a connected push with no progress for a measured time is ended,
the service's retry plan keeps it,
the command says it is queued),
replacing "Wait while connected".
The rule also overrides these coordinator picks:
pick 2 (a proven-alive index-lock holder waited for with no bound),
its follow-up (an unproven owner waited for with no bound,
built in `82633304d0` and merged into the 0.5.28 base),
pick 6 (no client wall-clock bound on the service),
`update-ref` waited for,
and the design's B-hand,
P-wait and R-never.
0.5.28 does not ship until every wait in it has a bound.

### Commit protocol revision 12 answers

The owner,
through the question tool,
answered before the commit that records this entry
(design `doc/planning/cli-git-commits-never-fail.md` revision 12,
section "Owner questions for revision 12"):
- A branch write already started when the time runs out:
  "Short extra wait (Recommended)"
  (the command waits a few more seconds,
  a constant to be measured,
  for the write it started;
  if the disk is still stalled it exits non-zero saying the result is not known,
  naming `git log -1` and the side ref).
- Merge order:
  "Branch first, files after (Recommended)"
  (the branch moves first and the files follow;
  at the bound the folder and staging are untouched;
  the owner was told a direct `/usr/bin/git commit -a` in the lag window would commit the merge's reverse).
  This replaces the coordinator's delegated pick M-A.
- The person's own programs before the commit exists (editor,
  hooks,
  signer,
  forwarded `gc`):
  "No limit; Ctrl-C ends it (Recommended)".
- How soon an accepted commit must survive a power cut:
  "Force at leaving, else background (Recommended)"
  (no forced write while the person waits;
  what a command promises when it leaves is forced to disk first;
  a power cut while it still waits loses that attempt,
  as with native Git).
  The coordinator applies the same rule to today's landing (the integration agent in "Agents and gates running").

### The final disk save is never cut short

The owner,
in the session,
after being told the fourteenth review found question 6a omitted fail-closed recovery:
"Assume the final disk save would never be cut short."
Meaning for the commit protocol design:
the departure barrier (stop receipt and saved set) runs to completion with no time limit;
questions 6a and 6b are dropped;
the provisional decision 12 ("Short wait, then say unconfirmed") is superseded;
a disk that never finishes the sync falls under the owner's outside-our-control list ("disk I/O errors").
Sent to the revision 15 agent.

### cargo-shadow kept files

The owner,
through the question tool:
"Warn past a count, never refuse (Recommended)"
(past a threshold every run prints how many kept files there are,
their total size and the folder to review;
cargo keeps working;
nothing is deleted).
Replaces the agent's refusal at `MAX_KEPT_FILES`.

### Commit protocol revision 13 answers

The thirteenth review (`commits-never-fail-sol-13.txt`) found questions 6 and 7 framed inaccurately:
question 6 omitted that a departure cut short by its limit can lose the record that forbids further attempts (so a withdrawn request could still land after a power cut),
and question 7's option A claimed it never undoes a later intentional change,
which a `reset --soft` plus switch disproves.
Answers 6 and 7 are therefore provisional until revision 14 asks them again;
answer 5 stands,
with its missing option to be shown.


The owner,
through the question tool,
answered before the commit that records this entry
(design revision 13,
`cc5e0822dc`,
section "Owner questions for revision 13"):
- Staged content differing from the file at a merge:
  "Carry the file, save staged (Recommended)"
  (the file as seen is carried;
  the staged-only version is saved to `saved/<path>.staged` and named in a line).
- A slow force at leaving:
  "Short wait, then say unconfirmed (Recommended)"
  (wait up to a named,
  measured limit,
  then exit saying the saved copy is not yet confirmed on disk while the wrapper keeps forcing it).
- How long to watch for a late `git switch` after a merge:
  "A measured span, then stop (Recommended)"
  (after the span the merge's files stay as changes,
  nothing lost,
  and the merge's message names how to undo them).

### Issues the owner raised are active work

The owner,
in the session,
before commit `25c4b39f58` (2026-11-03,
the commit that first recorded it):
"Why are you still not doing gh issue 6260 and 6261?"
#6260 was fixed and merged into the 0.5.28 base but the issue had no comment;
#6261 had been queued by the coordinator behind 0.5.28 without asking.
Both are now active:
#6260 has a status comment,
and #6261 runs on `refactor/git-policy-cli-subsystem-directories` (from `f89fa63f10`) in slices,
families no running branch touches first.

### Host pressure is never an excuse

The owner,
2026-11-03 between about 13:05 and 13:17-05:00 (said in the session),
after the coordinator offered disk pressure as the reason commits took 8.99 s,
16.6 s and 35.7 s:
"Disk pressure being high is not an excuse.",
then
"I've said something to the effect of "disk pressure / host / load is not an excuse" like 10 times now.",
then,
on the coordinator's "remove it or move it off the waiting path":
"sometimes the work itself can be optimized.
And we control every part of the wrapper,
including everything it embeds.",
then
"Think optimization can't do much?
No!
Look at our forbidden-strings and forbidden-regex.
Much faster than anything else.
At least 2x faster in every way than official `regex` crate.
And I did it with a much weaker model."
Then,
asked through the question tool how AGENTS.md should carry it,
"Tighten NHR, add SLW (Recommended)",
followed in the session by
"Host pressure might sometimes be the cause but it's never an excuse.
I also told you if you think the 5s standard cannot be always met even if we put in all the work,
you can ask me to raise it with evidence.",
and
"I told you that a long time ago.
And you never used it."
(that earlier permission was not written into this handover,
SPEC or any planning doc;
a search for it just before the commit that records this entry found nothing).
Meaning for this effort:
a command slower under load,
disk pressure,
swap or a slow SSD is a defect in the command,
even where the pressure is a cause.
The fix is found by measuring,
under that same pressure,
what the wrapper does beyond native Git,
and then removing it,
making the work itself cheaper (embedded code included),
batching it,
or moving it off the path the person waits on.
Standing permission:
when the evidence shows the 5 s standard ("commits vs native git should not take 5s longer.") cannot always be met even after all that work,
ask the owner to raise it,
with the measurements;
never accept a miss silently,
and never offer the pressure as the reason instead.
Offering the pressure as the excuse is retracted wherever this handover did it
(the 1996 ms commit was split by step against the load average,
and the slowdown was then attributed to load).
The precedent for how far optimization goes is `package/forbidden-strings/core` and its bench `package/forbidden-strings/core.bench`.
AGENTS.md now carries NHR ("sometimes a cause,
never an excuse or a reason to wait") and SLW.

### Never fail, and what is outside our control

All in `package/git-policy/cli/SPEC.md`,
"Merges and commits never fail":
the three rules ("Auto merging should never fail.",
"Manual merge should never fail too.",
"Commits should never fail."),
the correction "never fail doesn't primarily mean retries!",
the settled list of causes outside our control
(settled by the question tool,
recorded 2026-11-03T07:31:44-05:00),
the allowed refusals,
"Committing a conflicted path is fine.
Warn and commit anyway,
even if conflict markers will be committed.",
"Stack on the new head (Recommended)" and the later
answers.
Also:
"Offline is retried,
but auth and server hook declines stop until your next commit."
(SPEC,
same section).
Answered by the question tool on 2026-11-03 and not yet in main's SPEC
(they are on `fix/git-policy-cli-commits-never-fail` until it merges):
"Same as the ref lock (Recommended)" for a live foreign `index.lock`;
"Save on the side ref (Recommended)" for a branch a foreign program keeps moving;
"Leave out wrapper refs (Recommended)" for `git push --mirror` (everything except `refs/cli-git/*`,
with a line saying so);
"Amend the pending commit (Recommended)" for an amend while commits are pending.

### Design stance

The commit service unit sets no resource limits of its own
(owner, 2026-11-03,
"Remove all three (Recommended)" after the 1 GiB memory cap was found unmeasured;
recorded in `package/git-policy/cli/SPEC.md`).
Every shipped limit cites a measurement or an owner decision (`AGENTS.md` rule `LQM`).

`package/git-policy/cli/DESIGN.md`:
"we control all the code" and "Good is not perfect" (2026-11-01),
concurrent committers keep
growing,
locks are an anti-pattern with a resident single-writer service,
and "socket activation (OS starts it) or a loud,
clear
failure - no,
never leak internal things that could be resolved to the user."
(2026-11-01T07:37:44-05:00).
`AGENTS.md` rule NLK carries the lock rule;
the owner's earlier "Use as few locks as possible."
(2026-11-01T11:05:47-05:00) and "Invoke real git as rarely as possible."
(2026-11-01T01:28:19-05:00) are quoted in `SPEC.md` and `doc/planning/cli-git-btrfs-worktree.md`.
"We should never invoke ourselves recursively."
is in `doc/planning/cli-git-no-self-invocation.md`.

Recorded here only (no other home found):
- "We also have the advantage of git itself being OSS so we're not just wrapping a black box."
  (2026-11-03T07:36:28-05:00).
  Practice:
  for every Git step the wrapper runs or replaces,
  read Git 2.56.0's own source for what it locks,
  waits on and can exit
  nonzero for,
  cite file and line,
  and where a step moves in process port Git's logic so the result equals Git's.
  A v2.56.0 clone is at `~/temp/agent/git-v2.56.0-2026-11-03`.

### Git's own fsmonitor

Superseded 2026-11-03 by the owner:
"If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
disable git's fsmonitor globally entirely and make doctor say so."
The coordinator found that it is
(Git 2.56.0's Linux daemon answers only after registering a watch for every directory,
`compat/fsmonitor/fsm-listen-linux.c` lines 402-439 and 702;
a client waits up to 60 s for it,
`builtin/fsmonitor--daemon.c` line 40;
main's daemon watched 2,269,377 directories because the linked worktrees live inside main,
and cost 117.7 s of CPU and 734 MB to start;
a running daemon saved about 7 ms per `git status` on main),
and at 2026-11-03T09:38-05:00 set `core.fsmonitor = false` in `~/.gitconfig`
(backup `~/temp/agent/gitconfig-backup/gitconfig.before-fsmonitor-off-2026-11-03T0938`;
it was the only scope that set it)
and stopped all 261 running daemons (253 with `git fsmonitor--daemon stop`,
8 with SIGTERM,
none left;
`~/temp/agent/tools/fsmonitor-stop-all.ts`).
Done:
the doctor line (`feat/git-policy-cli-doctor-says-fsmonitor-is-off` `072ac84868`,
shipping in 0.5.27)
and `doc/troubleshooting/git-fsmonitor-daemon-start-wait.md` with a prototype patch beside it
(`7ea0414e80` to `da42fdb07b`;
the waiting `fsmonitor--daemon start` busy-spins,
concurrent clients each start a daemon,
a cold start measured 1.88 s at 100,000 directories and 0.013 s patched,
`t7527` passed 69 with 6 skipped;
the upstream draft is not for filing as is,
because Git's `SubmittingPatches` rejects AI-generated patches and needs a human sign-off;
filing is the owner's decision).
The earlier statements stay in `package/git-policy/cli/README.md`,
"Git's own fsmonitor",
as history.

### Worktree add and package managers

All in `doc/planning/cli-git-main-worktree-ignored-copy.md`:
"node_modules must be copied."
and "Builds must be copied" (2026-11-02),
"Skip nested worktrees and caches.
The point is to make
new worktrees immediately functional.",
"Skip run records (Recommended)",
"Convert once + snapshot (Recommended)",
"Set the shared store (Recommended)",
the dirty-main answer "B:
snapshot,
then reset",
"Reuse deps,
rebuild own" with
"Run cargo clean -p",
"Install online during the add",
"Run scripts as allowBuilds says (Recommended)",
the owner's pnpm
requirements ("Our wrapper must work with pnpm w/o a storeDir set...",
"Compatibility with all pnpm configurations is required.")
and the widening "I'm widening the pnpm spawning exemption during worktree copy ops to all pkg managers."
Races with other programs (2026-11-03T00:15:33-05:00),
verbatim:
"That is not an our problem.
By using this command or this program,
make the user understand and accept the risk."
Adopted reading (veto open):
races caused by other processes changing files or names while the wrapper works are outside what it
defends against;
it states that risk plainly and keeps fixing its own deterministic bugs.
Speed (2026-11-03T04:03:31-05:00):
"60s is fine for adding worktrees.
And make sure we're running everything that can be run in
parallel in parallel."
and "No,
I didn't say speed is unnecessary!"
Completion marker:
"Drop the fsync (Recommended)" (`SPEC.md`).
Pass `checkout.workers=0` "only if unset" (`SPEC.md`).
Per-worktree service (2026-11-01T16:28:21-05:00):
"Per worktree (Recommended)"
(`doc/planning/cli-git-resident-service.md`);
"Properly support overlayfs too."
(`doc/planning/cli-git-btrfs-worktree.md`);
"there is never a 5s bound on wrapped commands" (`README.md` and `SPEC.md`).

### Merge auto-resolution

`doc/planning/cli-git-merge-auto-resolution.md`,
"Decisions":
word-level second pass for merges a person runs and for the wrapper's unattended merges,
"Own merge on similar (Recommended)",
"doc/handover/*.md only (Recommended)" for the union driver (`.gitattributes`),
"Finish like a clean merge (Recommended)".
Conflict sides:
"save,
then allow" (`SPEC.md`).
Pruning of `cli-git/discarded/`:
"Prune by both age and size."
(`doc/handover/cli-git-logging.md`).

### Hooks and pushes

Only hooks the wrapper declares,
run natively;
no external hook commands ("Stop supporting them and port anything in.",
chosen label "B:
Only hooks we declare,
natively",
2026-11-01T02:13:03-05:00):
`SPEC.md`,
"Real Git starts with traditional
hooks off".
The wrapper never appends `--quiet` or `-q`:
"No the wrapper should never append --quiet.
Letting native git emit whatever it wants
to is expected."
(2026-11-03T04:50:57-05:00,
`SPEC.md`;
inner uses are the open `--quiet` branch).
Push stop rule "Stop only on no progress (Recommended)" (`SPEC.md`,
"push patience");
"Login failures final" (`SPEC.md`).
"Keep LFS on the fast path (Recommended)" (`doc/planning/cli-git-btrfs-worktree.md`).

### Tooling and repository policy

- Lockfiles are never hand-edited:
  "You're not meant to manually edit lockfiles!
  You're meant to fix the mpm set of tools we control and run it!
  And it is meant to follow pnpm!"
  (2026-11-03T05:15:12-05:00);
  `AGENTS.md` rule FOT and LFW.
- Cargo shadow:
  "Make a cargo shadow like what we did for pnpm shadow.
  Put the existing rust catalog tools inside it instead of
  making them separate.";
  `cargo update` equals `pnpm update` ("I said follow pnpm semantics.",
  2026-11-03T05:30:00-05:00);
  "I also said to remove the Mise tasks that would then be handled via the wrapper.";
  "PnPM doesn't have the reconcile keyword.
  Our Rust catalog tooling should follow pnpm and shouldn't have that keyword too."
  (2026-11-03T04:30:37-05:00).
  These live on the unmerged cargo-shadow branch;
  `doc/decision/rust-dependency-catalog.md` records the catalog rules.
- Workflows are editable like any file ("Who said you can't touch workflows?
  No one said that.
  Just do it.")
  and "Workflows
  should never use mise at all."
  (2026-11-03T05:31:50-05:00).
  CI runs tasks through `task-run`.
- mpm shadow:
  "I want pnpm style verbs.
  We talked about this!"
  (`package/mpm-shadow/cli/README.md`);
  the repair "should start whatever pnpm is first on path."
  (`doc/planning/cli-git-main-worktree-ignored-copy.md`);
  global tool environment "Repository only (Recommended)";
  the owner is "migrating away from mise soon TM"
  (the hint fix of 2026-11-03 removed a mise task name from a user-facing remedy).
  Mise is a task runner here,
  not a package or tool manager;
  the repository's package managers are pnpm,
  Cargo,
  Gradle,
  uv,
  Bun,
  npm,
  Zig and the `mpm-shadow` toolchain wrapper.
- Disk weight:
  `IOWeight` never above 10 (`AGENTS.md` RXS);
  containers get `--blkio-weight=10 --cgroup-conf=io.weight=10`.

## Working rules for this effort

Rules not in `AGENTS.md` (each recorded at the time shown in the former handover).

- Standing authorization (2026-10-13):
  "bump and publish any updates of the native wrapper crate and its `forbidden-strings`
  dependency during this session,
  after the gate,
  a dry-run publish and a real-credential commit have passed."
  It was given for that session;
  releases since have gone through `crate-release` under it.
  Any other first crates.io publication needs the owner's explicit approval (the cargo-shadow 0.1.0 publish is one).
- Release order for the wrapper:
  gate passes (`gate-exit-status=0` and the nextest summary),
  merge to main,
  `native:clippy` clean on main,
  `mise run //package/git-policy/cli:native:release` (dispatches the `crate-release` workflow;
  it refuses while origin lacks local main commits),
  `native:release:wait`,
  then `node ~/temp/agent/upgrade-wrapper.ts <version>`
  and a sample commit.
  Owner (2026-11-02T21:58:07-05:00):
  "This is expected and fine,
  the correct answer is to release again combining all the
  changes."
  So merge gated work as it passes and dispatch again with everything combined.
  A release builds from origin's main:
  check `main...origin/main` and that fetch works before trusting a release's contents.
- Gates:
  `node ~/temp/agent/gate/launch-gate.ts <worktree name or path> <label of lowercase letters, digits and hyphens>`
  starts the container gate as its own systemd unit `gpc-gate-<label>-<timestamp>`;
  its log in `~/temp/agent/gate/` ends with
  `gate-exit-status=<n>`.
  Read the verdict from that line and the log's nextest summary,
  never from `systemctl show` on the unit
  (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`).
  Host checks:
  `mise run //package/git-policy/cli:native:test:host -- <filter>`,
  `native:clippy`,
  `native:perf`
  (30 paired samples against direct Git,
  limit 5000 ms per delta;
  last measured commits median +40.2 ms).
- Reviews and merging:
  "If we have already decided on something just do it.
  Treat reviews as async."
  (2026-11-01T07:21:50-05:00);
  decided work merges after its gate,
  review findings become follow-up fixes.
  "Figure it out amongst yourselves who should own or do what.
  You have the tools."
  "You should (almost) never be waiting.
  There is always more work."
  and
  "Avoid wasting compute on stuff that would be shortly superseded."
- Cross-vendor reviews run through `pi` (see `CLAUDE.md`);
  keep at most three running.
  Several have timed out at 3600 s with no output and an empty session folder
  (`~/.pi/agent/sessions/--var-mnt-encrypted-temp-agent-review-*--/`):
  the run stalled inside `pi` before sending.
  A one-word probe answers in about 5 s;
  relaunch with the original question,
  narrower input and a word cap.
- Mutation campaigns are run by the main agent,
  never a subagent
  (owner,
  2026-11-01:
  "Because of how a few Claude Code quirks work together,
  long-running mutation runs when handed to subagents
  would burn abnormally amount of compute,
  so they'd have to be done by the main agent.";
  scope "Limit it to mutation runs only for now.").
  Record:
  `doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md` and
  `package/mutation-test/cli-cargo-mutants/README.md`,
  "Who runs a campaign".
  Container gates,
  fuzz campaigns and the e2e suite may run in a subagent.
  A campaign starts as a background unit and never holds a release or the next item;
  survivors are killed or justified later.
  Parallel campaigns use `--passes 2` (owner-accepted).
- Briefs to wrapper agents quote `package/git-policy/cli/README.md`,
  "Git's own fsmonitor",
  and the component's recorded
  `doc/decision/` owner decisions (rule DQB),
  and the synthetic-load ban.
- Keep all lines going:
  the owner chose "Keep all lines" (2026-11-03T07:38:56-05:00):
  continue the parallel lines and release
  whenever a gate passes.
- Timestamps in notes come from `date` or `git log --format=%cI`,
  never guessed;
  waits are on PIDs,
  never on a process name.
- Queue items are written here the same turn they are deferred (rule QWD);
  finish an item by moving its facts out and deleting it.
