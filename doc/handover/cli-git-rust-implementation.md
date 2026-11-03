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

- Git wrapper `monochromatic-git-policy-cli` 0.5.26 (`toolchain.lock.json`,
  row `cargo:monochromatic-git-policy-cli`,
  lock commit `c76a6b6d3e`;
  `git --version` prints "monochromatic-git-policy-cli 0.5.26,
  shadowing git at /usr/bin/git" over Git 2.56.0).
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

0.5.27,
dispatched 2026-11-03T10:28-05:00 from main after its gate passed (4264 of 4264,
6 skipped),
run https://github.com/Aquaticats/Monochromatic/actions/runs/41663034226;
`native:release:wait` is running,
then `upgrade-wrapper.ts 0.5.27` and the measurement on main against native Git.
0.5.26 was published to crates.io at 2026-11-03T13:21:01Z and installed by `upgrade-wrapper.ts`.

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

At 2026-11-03T10:00-05:00 (the owner's cap is five delegate agents at once):
- Shipped-limits pick 1's classing with the measured citations 7,
  8,
  13,
  14 and 15,
  branch `fix/git-policy-cli-limits-cite-measurements`,
  worktree `.claude/worktrees/limits-cite-measurements`.
- Shipped-limits picks 4 and 9,
  branch `feat/git-policy-cli-steps-end-only-on-no-progress` from the 0.5.28 base `f58510c538`,
  worktree `.claude/worktrees/steps-end-only-on-no-progress`.
- The in-process index review fixes,
  the agent that wrote `perf/git-policy-cli-follow-up-index-in-process`.
- The wrapper never runs itself as Git,
  branch `fix/git-policy-cli-never-runs-itself-as-git` from main `7ea0414e80`,
  worktree `.claude/worktrees/never-runs-itself-as-git`.
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
- Shipped-limits picks 5 and 12,
  branch `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx`,
  worktree `.claude/worktrees/pushes-wait-lfs-5xx`.
- Not agents:
  the second cross-vendor review of cargo-shadow (`pi`,
  output `~/temp/agent/reviews/cargo-shadow-sol-2.txt`,
  diff `cargo-shadow-src-2.diff` from `504a82aa3f` to `09bb7820ea`;
  the follow-ups are done:
  294 of 294 tests,
  the trust base read from Git's files with one `git` start only in the cases they cannot answer,
  saving 1.5 to 4 ms per call,
  `file-enforcer write` from source reproduces the generated line,
  and about 4300 comment lines,
  partly from a regex tool,
  which the review checks for false statements);
  the sixth cross-vendor design review (`pi`,
  output `~/temp/agent/reviews/commits-never-fail-sol-6.txt`);
  the gate of 0.5.27 (named under its branch below).
Queued,
in this order:
shipped-limits picks 2 and 6 from the 0.5.28 base,
as a slot frees.

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
Remaining:
the sixth review and its revision,
then build in the reviewer's order (models and crash fixtures first),
with the exit inventory as a gate.
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
(`lfs_bounded_push.rs`);
the auto-push fetch still does.
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
6. Waits on the service:
   A,
   heartbeats and no client wall-clock bound;
   the `COMMIT_WAIT` message naming a nonexistent setting goes.
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

### Release contents

- 0.5.27 ships without the in-process index work,
  which waits for its review's fixes and ships in 0.5.28;
  reasons under "Agents and gates running".

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
