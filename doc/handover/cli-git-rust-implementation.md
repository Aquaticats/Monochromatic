# cli-git implementation handover

This page holds only the open work of the Git wrapper effort (`package/git-policy/cli`) and its current
state.
Nothing on it is approved or final:
it is an agent's account,
and only the owner declares work complete.
It was rewritten on 2026-11-26T18:55-05:00 to the state the coordinating session read then,
when the owner asked for the handover to be updated,
holding the current state and nothing that has landed.

What landed left this page for the record that owns it:
a root cause or a workaround went to `doc/troubleshooting/`,
a decision to `doc/decision/`,
a measured number to the page that measures it,
and unfinished work nobody is on to a GitHub issue.
`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` held the run logs from
2026-11-09 on until they were sorted the same way and the file was deleted (`f0f870e305`).

## How the work is laid out

The levels are Linear's,
by the owner's word of 2026-11-23.

- An **initiative** is a broad effort with a reason behind it.
  It lives on this page,
  one heading each under "Initiatives".
- A **sub-initiative** is one outcome inside an initiative that takes several runs to reach.
  It lives on this page under its initiative:
  the outcome,
  the state,
  what comes next,
  and where its records are.
- An **issue** is one run or one action with one end that can be checked.
  While it is being worked it is an entry in the coordinating session's task list,
  marked in progress only while a run on it is live.
  A defect,
  or work nobody is on,
  is a GitHub issue (`doc/agent/issue-tracker.md`).

Neither an initiative nor a sub-initiative is ever an entry in the task list.

## Standing directions

Each is the owner's,
and each stays in force until the owner changes it.

- The ZFS route comes first.
  Heavy use of worktrees waits for it,
  by the owner's word of 2026-11-18:
  "We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
  For the ZFS route's own work the width is the owner's of 2026-11-23:
  "Up to 64 rounds at once.",
  and later steps' pieces are built ahead of the plan's order,
  unwired:
  "Build them ahead,
  unwired (Recommended)"
  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
- A branch may not make anything `main` does well worse.
  The owner waived it once,
  for one case:
  `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "The merge bar is waived for an add above the warning's count".
- The merge bar,
  restated on 2026-11-25 and unchanged:
  a branch merges into `main` when its round has ended and its report is read,
  and no gate,
  pass,
  review or mutation run holds a merge;
  one whole container gate runs on `main` per batch of merges;
  a design branch merges as a proposal and stays one until accepted;
  Sonnet passes run only at a release,
  over everything merged since the last one
  (`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`).
- An alpha release of any package may be bumped and dispatched unasked;
  any other release is asked each time
  (the same record,
  "Release dispatches,
  answered 2026-11-23").
- This session answers for the wrapper,
  the Pi driver,
  mpm and the way Cargo is run here.
  The owner,
  2026-11-13:
  "You're responsible for the driver,
  the wrapper,
  mpm,
  cargo.
  If you find a issue you or a subagent fix it."
- Every run is kept moving:
  when one ends,
  its result is read,
  recorded here,
  and its next step started in the same turn.
- No limit on runs is in force for memory
  (`doc/handover/agents-at-once-on-this-machine.md`,
  "The freeze of 2026-11-22 set no limit on runs").
- Who writes and who reviews:
  the skill `pi-as-subagent`,
  section "Who writes and who reviews".
- The owner,
  2026-11-23:
  "Do whatever you need to do,
  but don't abandon any work."

## The state at 2026-11-27T06:37:37-05:00

`main` was `9e46b96ef0`.
What landed before this time is in its own records and closed issues,
and the merges are in `git log --merges`.
Sixteen rounds merged on the evening of 2026-11-26,
and `git log --first-parent --merges --since='2026-11-27 00:00'` counts 145 more by this time.

### Rounds running

Every round runs on Pi's virtual writer,
pinned to no member
(`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`),
under the owner's "Up to 64 rounds at once."
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
All were started through the driver package,
with the brief at `~/temp/agent/pi-runs/<round>-prompt.md`,
the run under `~/temp/agent/pi-runs/<round>-<n>`,
the unit `pi-<round>`,
the worktree `.claude/worktrees/pi-<round>`
and the whole report at `~/temp/agent/<round>-scratch/keep/report.md`:

- the last loop-break round,
  `break-wrapper-file-loop-of-the-worktree-removal-and-remove-files`:
  one loop of twelve files,
  27 lines,
  which the `worktree/` round left unattempted
  (its brief carries that round's reading of the moves as suggestions).
- split rounds for the files the loop breaks freed:
  `split-wrapper-landing-files-over-the-line-budget-5` (four files),
  `-merge-isolation-carry-files-` (two),
  `-merge-isolation-recovery-kept-copies-and-take-asides-files-` (three),
  `-push-files-over-the-line-budget-3` (four)
  and `-worktree-copying-files-over-the-line-budget-4` (four).
- rounds for issues,
  each named `issue-<number>-<what it makes true>`:
  6477,
  6562,
  6567,
  6568,
  6571,
  6572,
  6573,
  6574 and 6575.
  Issue 6569's round has ended and its branch waits for a one-line import union with `main`
  in `landing/service_land_take_index_lock_tests.rs`.
- the two fix rounds for the linter's alpha.18 release pass ("The linter").

Round 3 of the crate split's order is done but for that one loop and three lines:
every other folder's loops are broken and merged
(`lfs`,
`policy`,
the relay,
`kept_references`,
two clone folders,
`zfs_mounted_dataset`,
`command`,
`conflict`,
`dependent_version`,
`service`,
`shadow`,
`start_probe`,
`transaction`,
`repair`,
`repo_read`,
`worktree`'s other two,
`merge`,
`lock`,
`process`,
`commit`,
`push`,
`worktree_copying` and `landing`,
the last as `9e46b96ef0`).
`layout-allow-list/file-rule.txt` went from 544 lines to 52,
with no layout finding on `main`:
27 for the `worktree/` loop,
and three for `foundation/`'s loop of `diagnostic_lines_off_the_calling_thread.rs`,
`diagnostic_log.rs` and `lines_written_by_a_thread.rs`,
which no move of items closes and which waits for the owner
(`doc/handover/questions-for-the-owner.md`).
Two merges met in the list
(`foundation` and `push`),
and each was resolved by hand as the union of the two sides' deletions,
with `cargo check --all-targets` and the linter run on the merged tree before the commit.
The installed session wrapper's merge pass kept deleted lines there (issue 6529);
the source is fixed (`5914dd3810`) and the installed build is older.
The plan document's loop section and its "Where each step stands today" are stale for these folders:
each round's commits say what its moves were,
and the plan's own text is not yet rewritten.

Round 4,
the plan's five call changes,
is not started:
each is a changed signature in release code,
and no record shows the owner accepting them
(`doc/handover/questions-for-the-owner.md`,
"does the crate split's step 4 run now?").

Waiting on a named merge:
the two `truepeak` crates' one file each,
on the branch `refactor/flac-profiler-keeps-the-rust-one`;
`worktree/worktree_remove.rs`'s split,
on its loop-break round;
the whole gate on `main`,
on issues 6562 and 6573.

`main`'s tip is checked after a batch of merges and not by each merge:
the merge routine runs `git merge-tree` and compiles nothing.
At `9e46b96ef0`,
`cargo check --locked --offline --all-targets` exited 0;
Clippy and the `source_guard` module were still running on that tip at this time
(`~/temp/agent/main-tip-compile-clippy-and-source-guard-check-2026-11-27/keep/result.txt`).
One control cannot pass on `main` until issue 6573 merges:
`process::captured_child_run_to_its_end::tests::the_one_shot_git_questions_no_longer_block_a_thread_on_command_output`
reads `repo_read/git_system_config.rs`,
which issue 6540's merge split.

The whole container gate on `main` at `d9ec2443bd` ended at 2026-11-27T02:37:12-05:00 and refused.
Its suite passed:
`12089 tests run: 12089 passed (9 slow), 51 skipped`.
Its Clippy container was killed for memory while it compiled the wrapper's test build
(the kernel log has "Memory cgroup out of memory:
Killed process (clippy-driver)" with 2,074,084 kB of anonymous memory at the 2g bound the two containers share).
`doc/planning/git-policy-cli-cachyos-test-containers.md` foresaw it:
"The first gate that fails Clippy on memory is when to move it".
The same Clippy command on the host at that commit,
with two jobs and no swap,
exited 0 with a peak of 4.4G,
so the batch's code passed Clippy and the bound is what failed.
Issue 6562 holds it;
its round runs whole gates of its own to measure the bound,
which is why it has run for hours.

A package-launched round has ended when `pi-runs/<round>-<n>/summary.json` exists,
which the package's `list-runs-that-ended-since` lists,
and it merges by its branch,
once `origin/<branch>` equals the local tip.
A run whose latest run folder holds no `summary.json` and has no process never started:
read the launch unit's `.out` file and the unit's journal.
The driver refuses a run folder that already holds anything,
so a steering note goes into `<run folder>/inbox/` only after the run's `feed.jsonl` exists:
a note written between the launcher making the folder and the driver starting
stopped issue 6566's first start on 2026-11-27.

### The linter

`0.5.2-alpha.18` is released (`52d5d51251`,
published as `monochromatic-lint-v0.5.2-alpha.18`;
an alpha is not published to crates.io),
locked in `toolchain.lock.json` (`cd3cf42a9a`)
and installed in the main checkout.
It holds issue 6513's near-miss marker finding (`7e67d1a3a4`)
and the two fix rounds for the alpha.17 release pass (`d16248bd0e`,
`eee9a8c3a7`).
The release workflow pushed its bump commit to `origin/main` while two merges were still local,
so `main` diverged for a minute and was merged back by hand (`cc3f2173c1`):
start a release only when the last merge has been pushed.
`mise run toolchain:install` moved the session's `git` link to the toolchain launcher again,
and it was restored to the build of 2026-11-22 at 2026-11-27T05:29:11-05:00.
Its four files over the line budget are split (`bec5214c7d`),
and the crate has none left.

Its release pass,
a Claude Sonnet second pass (same vendor as some of the authors,
so not an independent review),
ended at about 06:20 with no High,
two Medium and five Low points,
and the gate clean on `package/git-policy/cli` and the ZFS package.
Every alpha.17 point is fixed,
left as documented,
or has a new shape among these:

- the near-miss search is quadratic over a long run of blank or comment lines in a top doc,
  155 to 227 times alpha.17's CPU time at 160,000 lines with the control arm at 0.82 to 0.97;
- a group that closes a loop still walks everything that reaches it,
  2.31 to 2.49 times alpha.16 on a shape where every group is reached from the whole graph,
  against the README's "as alpha.16 did";
- the arrow-name refusal also runs when only the file rule is selected,
  and names another cause;
- spacing before the arrow of a `uses` line stops the whole folder rule,
  where spacing after it is only a dead entry;
- a `..` alias escapes the repeat check,
  and `file-uses` does not read it;
- two messages point a little off;
- several cost guards and wordings have no test that fails without them.

All are routed to two fix rounds,
`monochromatic-lint-alpha-18-release-pass-cost-findings`
and `-refusal-alias-and-message-findings`,
started at about 06:22.
`0.5.2-alpha.19` is released when both have merged.

### What the split rounds left

On `main` at `9e46b96ef0` the wrapper has no layout finding and 29 files over the budget,
down from 264 before the splits.
What is left falls in three classes.

- Files with a split round running now:
  four in `landing/`,
  five `merge_isolation_*` files in `merge/`,
  four in `push/`
  and four `worktree_copy_*` files in `worktree_copying/`.
- Files whose loop is not broken yet:
  `worktree/worktree_remove.rs`,
  on its loop-break round,
  and `foundation/lines_written_by_a_thread.rs`,
  whose loop waits for the owner.
- Files in a group folder (`invocation/`,
  seven files;
  `health/doctor.rs` and `health/doctor_filesystem.rs`),
  where a new sibling file needs a `group-file` line,
  and `worktree/mod.rs` (332 lines of module declarations).
  They wait for the move of those files into child folders,
  round 7 of the plan's order,
  which follows rounds 4 to 6.

A brief names a file over the budget only from the linter's listing of `main` taken when the brief is written:
the round for `command/rule_linked_worktree.rs` was briefed from an earlier count,
found the loop break had already brought the file to 298 lines,
and split nothing (`bb5d25980b`).

The coordinator's reading,
not the owner's:
a branch is open when a round or a session works on it or this handover names it,
because each crate's reformat already conflicts with every older branch in these files
(`57bf0918fc` for the wrapper,
`f262797330` for `mpm-shadow`,
`b4ae047c05` for `cargo-shadow`,
`639f90484d` for the linter).
The briefs say so,
and tell a round to leave a file only for a branch whose newest commit is later than 2026-11-26T12:00-05:00.

Outside the wrapper,
`mpm-shadow`,
`cargo-shadow` and the linter have no file over the budget,
and the two `truepeak` files wait on their branch.

### What the rounds found

- What the rounds reported as failing or misbehaving is on GitHub:
  issues 6524 to 6569,
  each filed after a search.
  Closed by a merged round:
  all of them except those under "Rounds running" and these,
  which stay open on purpose.
  Issue 6556 (Pi starts that end without a run) has its explanation and its printed line merged (`faa52ab2af`);
  sizing the two bounds from a measurement under load is the owner's.
  Issues 6555,
  6557,
  6559 and 6563 have merged proposals,
  and issue 6566 (a landing reads and hashes every changed path,
  about 30 s for a change list past `MAX_CHANGES` under load,
  against native Git's 0.6 s) names a cheaper form that is a decision,
  not a repair;
  all five wait in `doc/handover/questions-for-the-owner.md`.
  Issues 6570 to 6575 were filed on 2026-11-27 from rounds' reports
  (a feature build's Clippy warning,
  controls that write under the real home cache,
  a landing the service pushed that left no answer line,
  a control that reads a source file a split emptied,
  a control that races the system `talk` program,
  and a feature with no Clippy task);
  6570 is merged and closed,
  and the others have rounds.
  Issues 6523 (`package/ide/app` does not compile on `main`)
  and 6528 (`cargo-shadow`'s install step refuses a command when a `git` child is left as a zombie)
  have no round yet.
  Issue 6560 is closed as a duplicate of 6543.
- Two fixes changed release code,
  each with a control that fails without it:
  issue 6544 (`b5696a3eb0`),
  where the discarded prune removed a directory whose change time had not moved
  although a file had been added inside the tick of its listing,
  and now reads what the directory holds on every removal;
  and issue 6529 (`5914dd3810`),
  where the word-level merge pass wrote back lines one side had deleted.
  Issue 6547's fix also made the wrapper's self-check compare two files a chunk at a time
  instead of reading both whole (`ccea7826d5`).
- A second writer makes issue 6339's shape:
  a build in a worktree whose add was interrupted writes that worktree's own `cargo-shadow` stamp,
  and the owed ignored-state copy can then never finish
  (a comment on that issue has the round's lines).
- One blocking pool per process,
  one thread per runtime worker (`foundation/runtime_plan.rs`),
  is what several threaded-only failures in `landing::` wait on:
  issue 6565's round measured it for the prestage controls with a planted full pool,
  and issues 6567 and 6569 are the two modules it read the same way without measuring.
- The Slint round (`f1957916b0`,
  the owner's "Finish the move (Recommended)",
  `doc/decision/rust-dependency-catalog.md`) left three points open:
  the IDE's two `wrap_keys` fixtures need re-measuring for parley 0.13.0 (issue 6523),
  the IDE needs one container run before it is called green,
  and the three regenerated locks were written by `cargo fetch`,
  not through `cargo-shadow`'s trust policy.
- The driver's pool gate counts no virtual run (`88a9363636`).
  A member a caller closes with `PI_RUNS_POOL_LIMITS` therefore no longer stops a run on a virtual selection.
- `source_guard/standards_review_boundary_tests.rs` now treats the declaration of a module
  whose name begins with a service-only module's name as a declaration,
  not a mention (`3fd13e17fe`);
  the round marked it for the owner to weigh.
- Two rounds split the four byte-identical copies of `src/shadow_identity.rs` in different ways.
  A file that several crates must keep identical belongs to one round.
- `monochromatic-lint.config.jsonc` turns `rust/max-lines` off for `**/tests/**/*.rs`,
  so the linter never reported the test files the briefs named.

### Next

- Merge each round when it has ended and its report is read,
  by the merge bar in "Standing directions".
- One whole container gate on `main` once issue 6562's round has merged,
  since the last one refused on its Clippy container's memory bound and not on the code
  ("Rounds running").
  The last gate that passed whole ran at `42a59e812e`,
  12028 of 12028
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`).
- A new brief names `package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`
  and,
  for the wrapper,
  its guide.
- A round starts through the driver package,
  one batch list for all the rounds started together:
  `mise run //package/pi-shared/subagent-driver:start-a-batch-in-its-own-unit -- --batch-file=<list> --deadline-seconds=7200 --output-file=<list's name>.out`,
  the list a JSON array of entries with `name`,
  `branch`,
  `prompt`,
  `cpu`,
  `memory`,
  `tasks`,
  `wall`,
  `provider` (`rate-limit-fallback`) and `model` (`writer`),
  the briefs beside it in `~/temp/agent/pi-runs`.
  Never several lists side by side:
  three at once on 2026-11-27 overlapped 27 worktree adds,
  each passed its fifteen-minute bound twice,
  and sixteen of thirty starts failed (issue 6563,
  `doc/planning/pi-round-start-failures-and-what-would-stop-them.md`).
  A start that failed that way leaves its worktree,
  and starting its entry again needs no add.
  The launcher removes the worktree of a run that ended idle with nothing uncommitted,
  and a run's whole report is in `~/temp/agent/<round>-scratch/keep/report.md`,
  which each brief asks for.
  A merge is `git merge --no-ff <branch>` in the main checkout once `git merge-tree` shows no conflict;
  the coordinator's scratch merge script does both until the cleanup of 2026-11-27T15:26:37-05:00 deletes it.
- The wrapper's gate has tasks of its own,
  which the coordinator's gate of 2026-11-27 did not use and the next one does:
  `mise run //package/git-policy/cli:native:gate:main-tip` starts it on `main`'s tip in a capped unit,
  and `native:gate:verdict -- <log path> <deadline seconds>` reads its verdict.
- Owner questions wait in `doc/handover/questions-for-the-owner.md`,
  among them the next step a typed network command names at a new host key
  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`).

### Scratch expires after a day

The owner set `/etc/tmpfiles.d/temp.conf` on 2026-11-26 to `d /home/user/temp - - - 24h -`.
`systemd-tmpfiles-clean.timer` runs daily (next at 2026-11-27T15:26:37-05:00) and 15 minutes after a boot,
and the home dataset is mounted `noatime`,
so a file under `~/temp` survives only if it changed in the last 24 hours.
The owner,
the same evening:
"Don't promote anything in ~/temp that breaks repo rules or plain broken into the repo.
If you can't move everything before the scheduled cleanup,
too bad.
That is for you to learn."

So the coordinator's private launchers,
merge lanes and gate runner under `~/temp/agent` expire
and are not carried into the repository.
What replaces them is built in the repository by its own rules;
until then a round starts through the driver package's commands
and a merge runs by hand under the merge bar.
801 repository documents cite paths under `~/temp/agent` (3827 citations,
counted 2026-11-26),
and those citations point at nothing once the cleanup reaches them.

### The session wrapper is a build of 2026-11-22

`.cache/monochromatic-git-policy-cli-bin/git` links to
`.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`.
That build predates `4acb1b04b6` (2026-11-24T19:39:21-05:00),
so every `git worktree remove` warns and keeps the removed worktree's service journal
because it "holds rules-repository,
a file this release does not know";
the kept journals are harmless and sit in `~/.local/state/monochromatic-git-policy-cli/service/`.
`0.6.1-alpha.47` (release commit `68af165851`) is unpacked at `.cache/wrapper-release-0.6.1-alpha.47/`,
and its sweep,
run by absolute path,
removes them.
The session link stays on the reviewed build:
the AppArmor rules name that exact path,
and pointing them at a newer build takes the fresh review that waits in `doc/handover/questions-for-the-owner.md`.
`mise run toolchain:install <tool>` in the main checkout re-points that link at the toolchain's launcher,
whichever tool it names:
on 2026-11-27 at 01:47 an install of `cargo:monochromatic-lint` did,
and the coordinator put the link back on the reviewed build about a minute later.
After any toolchain install in the main checkout,
read the link (`ls -l .cache/monochromatic-git-policy-cli-bin/`) and put it back if it moved.

### Threaded-only test failures

A control that passes in the gate and fails in a threaded `cargo test` is that control's defect
(`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
"Tests hold as threads of one process").
A `panicked at` line from a thread named `git-policy-worker` in a combined run is not a failure:
it is the wrapper or the service started as a child by the fault-limit controls,
by the same guide's section.

### Memory

Memory is judged by the stall and by `MemAvailable` plus the ZFS ARC above `c_min`
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`).
No start or merge check waits on memory or on a spacing between starts:
the coordinator had applied both,
and the owner,
2026-11-26:
"I didn't tell you to apply the 3 minutes apart and memory test rules."
At 18:25 that evening,
with 58 worktree adds copying at once,
the disk stall read `some avg10=76.24` and `MemAvailable` 10290 MiB:
each add through the session wrapper copies about 39,000 ignored entries,
the cost the owner's hold of 2026-11-18 on heavy worktree use names.

## Initiatives

### The ZFS worktree route

Why:
on this machine's ZFS pool `git worktree add` falls back to slower routes,
and the owner made a ZFS route the first priority on 2026-11-17.
The design is decided in `doc/decision/cli-git-zfs-worktree-design.md`
and built step by step from `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

State,
at 2026-11-26T18:55-05:00,
by the plan's 23 steps.
Steps 1 to 16 are on `main`,
with 12 to 16 wired;
and the removal command path is still cut:
steps 15 and 16 name commands and `doctor` lines that steps 17 and 18 now have,
and the switch still needs step 22's request budget,
so nothing calls `enter_the_removal` yet and `source_guard`'s
`removal_command_path_writes_nothing_tests.rs` holds the cut.

- Step 9's unmount,
  staging and reattach is done (`5d93d9cfd7`),
  behind the pin plan step 19 writes,
  and nothing calls it yet.
- Step 10's add route serves an add whose source is the main checkout on a checkout dataset only step 19
  makes (`SPEC.md`,
  "The add that registers a ZFS clone before it fills the slot").
  Its build matches its accepted designs and its guest arms pass.
  The three points that kept a person's add off it are closed:
  6462 as `d169557d37`,
  2C as `98cc2c9420`,
  and the attach path's A5 as `d647717f85`.
- Steps 11 to 16 are each merged with their guest arms,
  and their limits are `SPEC.md`'s "What this release does not yet do".
- Step 17's cancel command merged as `df15f27a7a`,
  and the forced arm's decision half as `64a71a4c9b`;
  and its kernel calls are proven in a guest (`3ebc78c3dc`).
  The forced removal is measured for the holder kinds the guest showed and refuses the rest
  (`doc/decision/cli-git-zfs-worktree-design.md`,
  "What still gates shipping").
- Step 18's `doctor` checks merged unwired (`838b7f5e71` and `d463965dcd`),
  as did the store and `snapdir` delegation check (`2bf72fb230`).
- Step 19's migration entry and units are built ahead (`72a0be07ef`),
  and the migration has not run on this host.
  The owner answered every choice
  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`),
  and the chosen design merged as `1ef4afefa8`.
- Step 20 merged unwired (`a7502138a4`);
  writing `sanoid.conf` is the owner's act.
  Step 21 merged.
- Step 22's guest half is measured (`2ddf2853ad`);
  its storage half is the agent's act,
  logged,
  by the owner's answers of 2026-11-26.
- Step 23's host-kernel qualification run merged as `0c4064831d`
  (`doc/planning/cli-git-zfs-step-23-host-kernel-qualification-run.md`);
  the owner declares the release qualified or not.

Before a person's add takes the route:
wiring steps 17 and 18 with step 22's request budget and lifting the removal path's cut;
step 19's migration on this host;
steps 19,
22 and 23;
and a release carrying them,
installed.
Nothing of the route is installed on this machine.
The checkout migration of plan steps 12,
18 and 19 is the agent's act,
run with sudo when no process has the checkout open,
by the owner's answer of 2026-11-24 ("You run it whenever you feel convenient.",
`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
"Who runs the checkout migration").
The other host changes the plan keeps for the owner
(`zfs allow`,
the units,
the zedlet,
the immutable entry,
`sanoid.conf`,
other datasets' properties,
step 22's storage half,
step 23's image build and any reboot)
are the agent's act too,
logged,
by the owner's answers of 2026-11-26
(`doc/decision/cli-git-zfs-host-setup-measurement-and-image-build-are-the-agents-act.md`);
the owner chooses the moment of the migration and of any reboot,
which the agent prepares.

Next:
the wiring of steps 17 and 18 with step 22's request budget,
which lifts the removal path's cut.
It edits the crate the split rounds rewrite now,
so the coordinator briefs it against `main` once they have merged
(the coordinator's sequencing,
not an owner decision).

Rounds keep their files under `~/temp/agent/<round name>-<date>/`,
which expires a day after its last change ("Scratch expires after a day");
a round's durable findings land in its branch's records.

#### Accepted designs that wait for their build steps

- Private propagation after a replacement mount,
  for step 19:
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- The triggers a lost last event waits for,
  for steps 19 and 22:
  `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.
  The wrapper left that design's worktree in place at the merge;
  look whether it is gone before removing it by hand.
- The forced removal's V10 answer for a locked holder,
  and its taskless arm,
  both waiting for step 17's guest result and both put to the owner:
  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.

Each one's acceptance is the section "The design is accepted" of its decision record.

#### Later steps' pieces built ahead, unwired

Pieces that need only work on `main` are merged as code no person's command reaches,
so their steps wait only for their wiring.
What is still built ahead:
step 18's `doctor` lines (`838b7f5e71`,
`d463965dcd`,
`2bf72fb230`),
step 19's migration entry and units (`72a0be07ef`),
and step 20's `sanoid.conf` writing (`a7502138a4`).
Step 16's deferred-destroy checks,
step 15's deferred-origin check and step 13's managed lock are wired,
though no command reaches them while the cut stands.
Step 12's move refusal and unmounted report are wired and in use (`3ea4fa16c3`).

### The crate split, and the layout rules that land before it

Why:
the ZFS route's rounds build this crate alone,
and one build of it peaked around 6.6 GiB,
of which half its test code left out measured 4.7 GiB
(`package/git-policy/cli/doc/development-profile-measurements.md`,
"Memory per build,
measured 2026-11-25").
The owner chose how to cut it on 2026-11-25 at about 22:05:
"Window after live rounds (Recommended)".

State,
at 2026-11-26T18:55-05:00:
the split's round 1 merged as `3418cdaebc`,
round 2's 19 files that were over the budget before the reformat merged in five branches,
and the owner's "Keep 300,
split everything" put every Rust file over 300 lines in a round
("Rounds running").

The layout rules `AGENTS.md` sets for every crate are switched on for this one,
with every finding of the two rules on a list that only shrinks,
by the owner's answer "Everything now,
untangle over time."
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
"What the list holds").
`toolchain.lock.json` locks `monochromatic-lint` `0.5.2-alpha.17` (`d413a71714`).

Every crate outside this one is formatted by the root `rustfmt.toml`,
which the owner set on 2026-11-25.

### The wrapper's test gate tells the truth

Why:
one gate on `main` per batch of merges is the check the batch rest on afterwards,
and a control that fails once in a gate for no fault of the product costs a round each time.

State:
the gate's known causes are lettered in
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
with what each fixes and the runs each appeared in.
Cause H,
the two standing failures,
is diagnosed and fixed in its controls
(issues 6466 and 6468).
AppArmor enforcement costs `main` no test now,
and every rule the work needed is in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
with the owner's "Always widen apparmor."
A denial the work meets is answered by a rule in the profile's `local/` file,
never by moving the work aside and never by naming the profile.

Next:
run the next whole gate on `main`'s tip,
read its window's denials with `list-denials-in-a-window.ts`,
and start a fix round for any failure outside the recorded baseline.

#### The Markdown the line-break sweep left out

`monochromatic-lint`'s `markdown/semantic-line-breaks` still reports in the files the whitespace-only
sweep of 2026-11-23 left out,
because the branches that were changing them could not take the diff:
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` alone reports 683 findings on
`main`,
and everything under `package/git-policy/cli` and `package/pi-shared/subagent-driver/README.md` is the
same.
`fix-whitespace-only-line-breaks.ts` in `~/temp/agent/lint-line-breaks-of-the-2026-11-23-markdown/`
applies that fix,
and each file it rewrites is equal to its old text with whitespace collapsed,
which is how the sweep's 108 files were committed.
Run it once the rounds that edit those files have merged,
one file's commit each;
the script is in scratch and expires with it.

#### The release passes

`0.6.1-alpha.47` is released from `68af165851`.
Its two Sonnet passes are read and their fixes are on `main`,
except the owner's two follow-up refusals,
which are in their merge check
(`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
The next alpha follows the next batch's gate.

### The desktop stays usable under agent runs

Why:
the desktop froze for minutes on 2026-11-22 under this session's runs.

State:
two kernel settings,
`vm.watermark_scale_factor` 125 and `lru_gen` `min_ttl_ms` 1000,
are in force and kept across reboots by `/etc/sysctl.d/90-vm-watermark-scale-factor.conf` and
`/etc/tmpfiles.d/lru-gen-min-ttl.conf`,
by the owner's "Install both files (Recommended)" of 2026-11-24
(`doc/decision/memory-settings-persist-and-lfs-url-mask-shows-short-segments.md`).
The swapping that followed was `kswapd` and not the ARC,
and the fix that held was Madvise defrag with `compaction_proactiveness` 20;
the measurement,
its limit,
and the zram writeback to the old NVMe are in
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`.
`systemd-oomd` reads the cgroups again since AppArmor's rules allow it.

Next:
after the next reboot,
read both values once to confirm the files applied them.

## Held work

The wrapper and driver backlog waits under the owner's hold of 2026-11-18,
and other packages' work waits for the owner to name it.
It lives on GitHub,
by the owner's standing instruction of 2026-11-23
(`doc/agent/triage-labels.md`,
"Blocked work"):
an issue that waits on something carries a `blockedBy:<specific>` label,
most of them `blockedBy:zfs-route`,
some `blockedBy:owner-decision`,
some `blockedBy:wrapper-focus`,
and a few another issue's number.

What no public tracker holds,
or that is no issue:

- When the held queue is drained,
  the coordinating session re-reviews the whole wrapper effort once.
- Nothing of the WireGuard tools (`package/wg`) is deployed.
  Deploying is the owner's act,
  it takes a live connection down and up,
  and its steps were written for the owner on 2026-11-11 in a scratch digest,
  not in the repository.
- A scan on 2026-11-14 found secret values in the saved sessions of six Pi runs.
  Those sessions are never resumed or opened,
  and rotating the values is the owner's.

## Things a new session would not find elsewhere

- Do not run `pnpm install` in a new worktree (issue 6339).
- The coordinating session makes a host change the owner has authorised itself,
  with the passwordless `sudo`,
  and does not hand it back.
  The owner,
  2026-11-23,
  after enabling `audit-rules.service` when the session had left that to the owner:
  "I didn't need to run it myself but I just ran it as a courtesy."
  The rule against acting on the system service manager in the skill `pi-as-subagent` binds the agents a
  session briefs.
  The checkout migration is the agent's act by the owner's answer of 2026-11-24;
  the other ZFS host changes are the agent's act,
  logged (owner,
  2026-11-26),
  and the owner picks the moment of the migration and of any reboot.
- No round runs `mise run toolchain:install`,
  not even for one tool in its own worktree:
  it waits for issue 6316,
  in the owner's words
  "Because we need to make mpm-shadow prefer system package managers on non-atomic distros first."
- An answer from the owner changes the bar for reviewers that are already running.
  Send it to each of them in the turn it is recorded.
- A question to the owner blocks the coordinator until it is answered.
  Start every round that can run before asking one.
- A round's report is not evidence of its edits.
  Read the diff before a design goes to the owner or a branch is merged.
- A Pi round can hang in one command of its own without the driver noticing.
  The coordinator's watcher for that was a script in scratch,
  and it expires with it.
- The repository's tool links live in the main checkout's `.cache/toolchain-bin`,
  so `monochromatic-lint` and `file-enforcer` are there and not in a fresh worktree.
- Orphaned cargo build folders under `~/.cargo/build`,
  left by integration passes,
  are not removed without the owner.
- The checkout migration is the agent's act (owner,
  2026-11-24);
  the other ZFS host changes are the agent's act,
  logged (owner,
  2026-11-26),
  and the owner picks the moment of the migration and of any reboot.
  Old drives stay read-only and unwiped until the ZFS route is released.
