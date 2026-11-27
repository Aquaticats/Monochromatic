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

## The state at 2026-11-27T09:17:55-05:00

`main` was `04dbc235e1`.
What landed before this time is in its own records and closed issues,
and the merges are in `git log --merges`.
Sixteen rounds merged on the evening of 2026-11-26,
and `git log --first-parent --merges --since='2026-11-27 00:00'` counts 183 more by this time.

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

- `issue-6586-test-process-never-provisions-on-the-accounts-service-manager`
  ("What the rounds found");
- for the wrapper package's lint,
  which was red on `main` with nothing running it,
  three rounds for the guest scripts of issue 6581
  (`issue-6581-oxlint-clean-guest-scripts-part-<n>-of-3`,
  each writing `Refs #6581`,
  so the issue is closed by hand once the package's `lint:oxlint` exits 0 on `main`;
  the round for the `perf/` and service-manager scripts is merged as `2218beae78`,
  and `lint:oxlint` then reported 594 errors,
  all in those guest scripts);
- `issue-6587-service-manager-proofs-run-to-their-end`,
  the issue's second round,
  `issue-6594-container-start-sites-ledger-scans-rust-sources`
  and `issue-6596-scratch-root-removal-test-owns-the-process-it-names`
  ("What the rounds found");
- four rounds for the wrapper's alpha.48 release pass ("Releases of 2026-11-27"),
  each named `wrapper-alpha-48-release-pass-<what>`:
  `slot-refusals-read-a-shell-alias-and-say-true-words`,
  `a-notice-record-outlives-a-line-that-was-not-written`,
  `merge-pass-reaches-a-last-line-without-a-newline`
  and `four-small-points-outside-the-slot-route`;
- `issue-6583-gate-suite-container-gets-a-measured-memory-bound`,
  which runs gates of its own;
- five rounds for controls that failed once under load,
  each named `issue-<number>-<the control>-...`:
  6589 (`commit::commit_preparation::tests::a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message`,
  held 31.9 s by a `gpg` in an uninterruptible wait),
  6590 (`push::child_watch::tests::a_step_that_finishes_inside_its_ceiling_is_answered`,
  whose child read "the system refused a thread"),
  6591 (`push::push_minimum_rate_socket_tests::a_transfer_that_meets_the_floor_is_work_however_slow`,
  stopped at its two-second window),
  6592 (`commit::native_commit_waiting_notice::tests::the_notice_reports_the_programs_that_ran_below_the_step`,
  whose `sleep` ended between two looks)
  and 6593 (`worktree::worktree_snapshot_index_step::tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit`,
  two change times that read alike).
  The first four failed in the gates issue 6562's round ran under a load average of 48 to 96,
  and the fifth in issue 6582's wide run;
- in `cargo-shadow`,
  the last round for its alpha.11 release pass,
  `cargo-shadow-alpha-11-release-pass-stop-report-says-only-what-it-found`.
  Its other two are merged (`d134b5f836`,
  `ea51cd2290`),
  and `0.1.5-alpha.12` is released once this one is.

Until issue 6586 merges,
a brief for a round that runs the wrapper's Rust tests on the host tells it to give each test run
a runtime directory of its own and no session-bus address
(`env --unset=DBUS_SESSION_BUS_ADDRESS XDG_RUNTIME_DIR=<a fixture folder> cargo test ...`),
and the coordinator's own module runs do the same.

Three release passes also run,
as background agents of the coordinating session,
each a Claude Sonnet read
(same vendor as some of the authors,
so a second pass and not an independent review)
that writes `keep/review.md` under its scratch folder:
the linter's `0.5.2-alpha.19`
(`~/temp/agent/monochromatic-lint-alpha-19-release-review-2026-11-27/`),
the wrapper's `0.6.1-alpha.48` read for the claim that the splits,
the loop breaks and the reformat change no behaviour
(`~/temp/agent/git-policy-cli-alpha-48-release-review-splits-and-moves-2026-11-27/`),
and `mpm-shadow`'s `0.2.7-alpha.13`
(`~/temp/agent/mpm-shadow-alpha-13-release-review-2026-11-27/`).
A pass that runs a wrapper binary starts each run with throwaway XDG folders and
`env --unset=DBUS_SESSION_BUS_ADDRESS`,
for the reason issue 6586 records.
The coordinator rewrites what lasts from each into this file before scratch expires.

Round 3 of the crate split's order is done but for three lines.
Every folder's file loops are broken and merged,
the last being the removal files of `worktree/` (`ecb3486d9f`);
`doc/planning/cli-git-source-layout-target-tree.md` lists each folder's merge in its notes of 2026-11-27 under "Next",
and that round's note there says where the plan's reading of the moves did not fit.
`layout-allow-list/file-rule.txt` went from 544 lines to 25,
with no layout finding on `main`.
Its three `file-uses` lines are `foundation/`'s loop of `diagnostic_lines_off_the_calling_thread.rs`,
`diagnostic_log.rs` and `lines_written_by_a_thread.rs`,
which no move of items closes and which waits for the owner
(`doc/handover/questions-for-the-owner.md`).
Merges that meet in that list are resolved by hand as the union of the two sides' deletions,
with `cargo check --all-targets` and the linter run on the merged tree before the commit:
the installed session wrapper's merge pass keeps deleted lines there
(issue 6529;
the source is fixed in `5914dd3810` and the installed build is older).

Round 4,
the plan's five call changes,
is not started:
each is a changed signature in release code,
and no record shows the owner accepting them
(`doc/handover/questions-for-the-owner.md`,
"does the crate split's step 4 run now?").

Waiting on a named merge:
the two `truepeak` crates' one file each,
on the branch `refactor/flac-profiler-keeps-the-rust-one`.

The whole gate on `main` at `1c8acd5bfb` passed
(unit `gpc-gate-wrapper-next-alpha-on-1c8acd5bfb-20261127T075019-0500`,
`gate-exit-status=0`):
`Summary [ 973.345s] 12115 tests run: 12115 passed (6 slow), 51 skipped`
and "Clippy passed beside the test run.",
at the bounds it printed (image build 8g,
suite 2g,
Clippy 8g).
Its worktree is `.claude/worktrees/gate-wrapper-next-alpha`,
which `native:gate:main-tip` expects to exist and does not make.

`main`'s tip is checked after a batch of merges and not by each merge:
the merge routine runs `git merge-tree` and compiles nothing.
At `7a830014b3` (2026-11-27T07:34:30-05:00),
with the tip read before and after each run and found unchanged,
`native:check` exited 0,
the four Clippy tasks exited 0
(`native:clippy`,
`native:clippy:test-only-switches`,
`native:clippy:measurement`
and `native:clippy:start-probe-raised-deadline`),
and the `source_guard` module passed 179 of 179.
The gate that passed ran the two wrapper merges after it
(issue 6562's and the split of `worktree/worktree_remove.rs`).
That check never ran the package's TypeScript side,
and it does from now on,
until the owner chooses where those checks run (issue 6584):
`mise run //package/git-policy/cli:native:ts-checks` runs them each in a capped scope,
and `lint:oxlint` is run beside it.
At `2218beae78` `test:unit` exited 0 with 138 suites passing and `lint:types` exited 0;
`lint:oxlint` reported 594 errors and 1212 warnings,
all in the guest scripts issue 6581's rounds hold;
and `package/container-limit/core`'s `test:unit` failed its ledger test (issue 6588).
A tip check that a merge lands under is void and is run again.
A threaded `worktree::` run on this host fails 155 controls unless `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1` is set,
every one with the refusal of `subvolume/fs_btrfs_test_support.rs` on a home that is on ZFS;
with it set,
1015 pass and none fails,
on `main` and on the last loop-break branch alike.

The whole container gate on `main` at `d9ec2443bd` ended at 2026-11-27T02:37:12-05:00 and refused:
its suite passed 12089 of 12089,
and its Clippy container was killed at the 2 GiB bound the two containers shared.
Issue 6562's round measured Clippy's container in six runs (4,574.4 to 5,278.7 MiB with a bound that did not bind)
and gave it a bound of its own,
8 GiB,
with the readings beside the value in `bin/native-gate-container-limits.ts`
and in `doc/planning/git-policy-cli-cachyos-test-containers.md` (`79b93df956`).
Two of its five gates passed both checks,
the last on its branch merged with `main` at `88a63ef565`:
`12115 tests run: 12115 passed (5 slow), 51 skipped` and "Clippy passed beside the test run."
The three that refused each named its test run alone,
under a host load average of 48 to 96:
one control held 31.9 s by a `gpg` in an uninterruptible wait,
one that read "the system refused a thread",
one slow transfer ended at its two-second window,
the notice control another test's spans disturb
(`doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md`),
and the stale source-reading control that issue 6573 has since fixed.
The suite's container read 1,149.3 to 1,516.5 MiB of its 2 GiB in those runs,
which issue 6583 holds.

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

### Releases of 2026-11-27

Alphas of the wrapper and of the tools on its path are published unasked
(`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
"How far the permission to publish reaches").
A release starts only when `main` equals `origin/main`:
the workflow pushes its bump commit to `origin/main`,
and a local merge made meanwhile diverges from it.
After every `mise run toolchain:install` in the main checkout the session's `git` link points at the toolchain launcher,
and it is put back on the build of 2026-11-22
(last at 2026-11-27T07:47:16-05:00).

- The linter,
  `0.5.2-alpha.19`:
  release commit `88a63ef565`,
  locked (`f8f678c26d`) and installed.
  It holds the two fix rounds for the alpha.18 release pass,
  whose seven points they took:
  refusals,
  the `..` alias and messages (`c68716cb33`),
  and costs (`cf92cc3473`).
  The package's README says how a list written before alpha.19 maps,
  since four exit statuses change against alpha.18.
  The cost round did not measure the walk's peak memory,
  nor the file-rule list with tens of thousands of keys after its key index.
  The crate has no file over the line budget (`bec5214c7d`).
- `mpm-shadow`,
  `0.2.7-alpha.12` (`fea1c1768d`),
  which intended no change of behaviour over alpha.11,
  and then `0.2.7-alpha.13`:
  release commit `2d255a625c`,
  locked (`889d87bb83`) and installed.
  alpha.13 holds issue 6585 (`c513ee060c`):
  a start refused with "Text file busy" is explained and not repeated,
  because every writer the tool starts publishes a program by renaming a finished file into place,
  measured for five of them with no refusal in some 2,400 probe starts;
  and the alpha.12 pass's narrowing of what the splits had made visible (`cc61079f82`).
- The wrapper,
  `0.6.1-alpha.48`:
  release commit `e6cc06059d`,
  cut from the commit the gate passed on
  (no Rust or manifest file of the wrapper changed between `1c8acd5bfb` and the release).
  Its binary is unpacked at `.cache/wrapper-release-0.6.1-alpha.48/`,
  beside alpha.47's;
  the wrapper's lock row and the session's `git` link are left as they are.
  Two passes read it ("Rounds running").
- `cargo-shadow`,
  `0.1.5-alpha.11`:
  release commit `82cfbcaf11`,
  locked (`c0b6187db0`) and installed;
  a `native:check` of the wrapper through it printed its two install-step lines and exited 0.
  Over alpha.10 it holds issue 6528's stop report,
  issue 6576's test programs that end by themselves,
  the test fixes of issues 6549 to 6554,
  the file splits and the reformat.

What the two shadow tools' passes found,
each with no High finding:

- `mpm-shadow` alpha.12:
  the claim of no change of behaviour holds.
  Of 929 production items,
  891 are identical token for token,
  31 differ only in layout,
  five are real edits that the splits needed (the dispatch split into five functions,
  and four structs whose fields became visible to sibling files),
  and none was removed;
  all 780 tests survive;
  216 of 223 command lines gave the same output,
  status and written files from both binaries,
  and the other seven differ only in the version string.
  Its one Medium is that the binary is built by a newer compiler and linker and a lock that moved two crates,
  which no diff of the crate shows.
  Routed:
  a start refused with "Text file busy" is not retried and its advice cannot show the error again
  (issue 6585,
  the same on alpha.11);
  items the splits made reachable from a whole folder,
  and stale paths in records (one round).
- `cargo-shadow` alpha.11:
  issue 6528's change alters what is said and not what is decided
  (`stop_everything` still answers `Ok` only when the program was reaped and the scope is empty),
  and its test-only switch cannot fire in a release build.
  The reformat is rustfmt's own output on its parent,
  byte for byte;
  the splits match every old item once and lose no `cfg`;
  all 892 test names survive.
  Routed to three rounds:
  the refusal can say "no member it listed can still write" when it listed none,
  which is wrong for a process in a group below the scope,
  with four smaller points about the report's words and its listing;
  the guard `no_lean_source_starts_cargo_or_itself` reads only the top of `src/`
  and so stopped reading 18 lean files the splits moved into subfolders;
  and a test-only feature of `rustix` in `[dependencies]`,
  48 public paths the splits added to the library,
  and a record that names a file that never existed.
  Two issues recorded as test fixes changed release code:
  issue 6550 made the unready-journal recovery carry paths as bytes,
  which fixes a manifest with a non-UTF-8 name being skipped while its journal was removed,
  and issue 6552 threads a byte-count function through 18 items with every production call passing the same one.
  The package's `test` task exits 0 on a host that cannot delegate a control group,
  leaving out the tests it cannot run and saying so;
  whether the release workflow's runner is such a host is not yet known,
  and the third round reads the workflow's log for it.

What the wrapper's alpha.48 behaviour pass found,
with no High finding:

- Medium:
  a shell alias still takes a slot removal or a prune past the refusals,
  in alpha.47 and alpha.48 alike
  (`git -c 'alias.x=!git worktree remove' x -f -f <slot>`),
  because the planner reads the alias's name where the destructive-command guard reads its resolved command.
- Medium,
  a regression:
  since `6ea539bc91` a notice is handed to the stream that writes off the command's thread,
  which answers success before anything is written,
  so the record of a failed push or a kept worktree is removed although its line never reached anyone.
  With standard error a pipe whose reader is gone,
  alpha.47 keeps the record and alpha.48 does not.
- Low:
  three sentences of the slot refusals
  (one fires for an empty record folder,
  one advises a settling command no grammar admits,
  one calls `--` an option);
  the merge pass does not reach a last line without a final newline;
  a named pipe among the merge records stops every command with no way out named;
  a drain sizes its batches from one reading of the host per round;
  the kept-copy line prints a holder's name unfiltered;
  the README holds a pasted handover excerpt.
- Right as claimed,
  exercised with both binaries:
  the slot refusals for the spellings they name,
  the merge pass's union of deletions,
  the self-check's memory (62.7 to 66.1 MB in alpha.47,
  9.3 to 11.0 MB in alpha.48),
  named pipes answered and not waited on,
  and the typed ssh relay ending about 100 ms sooner at a terminal.
- Read only,
  because nothing outside the process can start them:
  the prune race,
  the conversion environment's precedence,
  the service-only changes.

The lock names each tool by its own id:
`cargo:monochromatic-lint`,
`cargo:monochromatic-cargo-shadow`,
and plain `mpm-shadow`.
`toolchain:update` refuses an id the plan does not hold and writes nothing,
which is how `f8f678c26d` came to name `mpm-shadow` in its message and lock only the linter
(a comment on that commit corrects it).
A command line that changes into a worktree ends there when the change fails (`cd <worktree> || exit`):
on 2026-11-27 one `toolchain:update` ran in the main checkout because the worktree it meant was gone,
and only that refusal left the main checkout's lock as it was.

### What the split rounds left

On `main` at `1c8acd5bfb` the wrapper has no layout finding and 11 files over the budget,
down from 264 before the splits.

- `foundation/lines_written_by_a_thread.rs` waits for the owner,
  with its loop.
- The rest wait for the move of group-folder files into child folders,
  round 7 of the plan's order,
  which follows rounds 4 to 6:
  the files of `invocation/`,
  `health/doctor.rs` and `health/doctor_filesystem.rs`,
  where a new sibling file needs a `group-file` line,
  and `worktree/mod.rs` (339 lines of module declarations).

A split is not always a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move;
the wrapper's next release pass reads it as a change.

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
  issues 6524 to 6597,
  each filed after a search.
  Closed by a merged round:
  all of them except those with a round ("Rounds running") and these,
  which stay open on purpose and wait in `doc/handover/questions-for-the-owner.md`.
  Issues 6555,
  6557,
  6559 and 6563 have merged proposals.
  Issue 6556 (Pi starts that end without a run) has its explanation and its printed line merged (`faa52ab2af`);
  sizing the two bounds from a measurement under load is the owner's.
  Issue 6566 (a landing reads and hashes every changed path,
  about 30 s for a change list past `MAX_CHANGES` under load,
  against native Git's 0.6 s) names a cheaper form that is a decision,
  not a repair
  (`doc/planning/cli-git-landing-index-reset-reads-every-changed-path.md`).
  Issue 6571 (a passing control wrote under the real home cache) is fixed for every writer but one (`c1a3b33db5`):
  a worktree removal's service-side Git still writes there,
  and reaching it changes a signature release callers see
  (`doc/planning/cli-git-removal-work-cache-place-the-caller-hands.md`);
  the issue was closed by the fix's commit and reopened for that writer.
  Issue 6577 follows issue 6528,
  `cargo-shadow`'s install step refusing a command because a process "could not be stopped":
  the round measured that the kernel does not count a member that has exited and waits to be reaped,
  so the zombie reading was not the cause,
  and made the refusal say how long the stop waited and name each member left (`41b9597adf`);
  the 5 s the stop gives a killed scope was sized on a quiet host and is unchanged,
  and what refused under heavy disk load on 2026-11-26 is not known.
  Issue 6523 (`package/ide/app` does not compile on `main`) has its compile fix on `main` since `4df352241d`,
  and what is left of it is the owner's (a comment on the issue).
  Issue 6560 is closed as a duplicate of 6543.
- Fixes that changed release code,
  each with a control that fails without it,
  which the next release pass of its tool reads:
  issue 6544 (`b5696a3eb0`),
  where the discarded prune removed a directory whose change time had not moved
  although a file had been added inside the tick of its listing,
  and now reads what the directory holds on every removal;
  issue 6529 (`5914dd3810`),
  where the word-level merge pass wrote back lines one side had deleted;
  issue 6547 (`ccea7826d5`),
  where the wrapper's self-check now compares two files a chunk at a time instead of reading both whole;
  issue 6571 (`c1a3b33db5`),
  where the conversion environment of a landing,
  an add,
  a ladder snapshot and a removal's clean check now lets a variable its caller named win over the process's own
  (the caller's entries were appended and the first match won),
  and the clean check reads that environment where it read the bare process's;
  and,
  in `cargo-shadow`,
  issue 6528 (`41b9597adf`),
  the stop's report,
  whose new test-only switch sits behind `cfg(test)` at its call site in `control_group_stop.rs`.
- The wrapper package's TypeScript side was red on `main` and nothing ran it.
  Issue 6562's round reported it,
  and at `1c8acd5bfb` `test:unit` exits 1 on two files:
  `e2e/service-mode-words.unit.test.ts` looks for a declaration line that became a slice (issue 6579),
  and `bin/native-planted-controls.unit.test.ts` dies with "Invalid string length"
  because a recursive read of `src/native` follows the `uses/` links
  and returns 163,079 entries for a tree of 3,809 files (issue 6580).
  `lint:oxlint` exits 1 with 850 errors and 1856 warnings in fifteen files:
  eleven guest scripts in `e2e/`,
  two fence measurements in `perf/`
  and `bin/service-manager-install-proof.ts` (issue 6581).
  The release gate runs the Rust suite and Clippy,
  a round runs the Rust modules it touched,
  and the coordinator's tip check ran `native:check`,
  Clippy and `source_guard`;
  where the TypeScript checks should run is a proposal for the owner (issue 6584).
- A threaded test run of the wrapper on the host reached this account's own service manager (issue 6586).
  In a run of every test whose path holds `worktree`,
  a control's add asked the manager to start `monochromatic-git-policy-cli@<instance>` units for its fixture,
  the program the manager ran was the test binary
  (100 starts refused with "Unrecognized option:
  'journal'" between 07:31:27 and 08:01:43 on 2026-11-27),
  and the shared link `/run/user/1000/monochromatic-git-policy-cli-bin` pointed at that binary
  until another session's wrapper command pointed it back.
  Release code remembers the running executable,
  and the route provisions whenever the process's own runtime directory and session bus exist.
  Issue 6582 put its two controls in a child with a runtime directory of its own (`eeebbfbe22`);
  issue 6586's round makes it impossible for the whole test process
  and writes a proposal on whether the release should refuse to provision from a program that is not an installed wrapper.
  The coordinator stopped and reset the ten failed units the runs of 07:31 to 07:40 left,
  and removed their sockets and notes,
  each note naming a Git directory under a cargo build folder's `test-fixtures/`.
- Issue 6580's fix found seven planted controls whose text no longer occurred in the file their table named
  (four moved by splits,
  three reflowed by the reformat);
  the test that would have said so had been dying before it compared anything.
  They are repaired (`64e3eb1a55`),
  and that they still compile and that a campaign notices each is owed to the next `native:mutation:planted` run.
- This host's AppArmor profile for Git confines a container's `/usr/bin/git` as well,
  since it attaches by path.
  The service-manager install proof runs Git as the container's root in a directory its user owns,
  which needs a capability the profile refuses on purpose,
  so the proof has not run to its end since 2026-11-13 (issue 6587;
  `doc/troubleshooting/git-cannot-mkdir-inside-the-container-proof.md`).
  A round's session cannot read `/var/log/audit/audit.log`,
  where these records are and the kernel journal's are not:
  when a round reports a refusal only one program meets,
  the coordinator reads that log.
- `package/container-limit/core`'s ledger of scripts that name `podman` fell behind again and is brought up to date (`f69208a309`);
  its scan reads no Rust file,
  which issue 6594 holds.
- The service-manager install proof now passes its Git step (`6b1a8e91f4`) and stopped at the next:
  the commit service's landing failed because this host's Git profile refused the marks file
  `git fast-import` writes beside the journal when the service cannot hand it an unnamed scratch file,
  as in a container's root home.
  The profile grants that name now
  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
  seventeenth batch),
  and the issue's second round reruns both proofs.
  `test:service-manager`'s control arm needs a release from before the remedy of 2026-11-12,
  which this host does not hold;
  that round writes the choice as a proposal.
- An alpha's release workflow runs no test:
  it commits the bump without compiling,
  and `cargo-publish.yml` builds the binaries.
  The evidence for an alpha is the local run,
  and for the wrapper the gate on `main`
  (`package/cargo-shadow/cli/README.md`,
  "Which host each check runs on").
- `START_RETRY_BUDGET`,
  the five seconds a refused start is repeated for in `mpm-shadow` and in the wrapper's `push/start_retry.rs`,
  cites no measurement and no decision (issue 6597,
  no round yet).
- The two fence measurement scripts of 2026-11-21 under `perf/` can no longer measure what they measured:
  the crate no longer reads their switch,
  and this host takes another rung
  (`doc/planning/cli-git-fence-size-read-beside-source-check.md`,
  "The harness after 2026-11-27").
  They are lint-clean and kept;
  whether they are removed is the owner's.
- `cargo-shadow`'s tests left processes and units behind.
  At 2026-11-27T07:05-05:00 the host held 25 orphaned shells,
  each polling a release file every 50 ms for up to twelve hours,
  left by the killed-wrapper control whenever it ended before its release line;
  the coordinator ended them with `SIGTERM` after checking each one's command line,
  parent and age,
  and issue 6576's round gave every gate-waiting test program a count of its own
  and the control a guard and a leak control (`cf98f01e22`).
  That round also saw `cargo-shadow-<id>.scope` units that stay active with no member,
  twelve of 11,360 started that morning.
  Issue 6578's round found the cause,
  which `doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md` already held:
  the manager attaches the `systemd-run` launcher to the scope,
  and when the run kills that launcher while the start job runs,
  the manager counts an exiting process as attached and never hears the group empty.
  The runner now asks the manager to stop its unit when its run ends and the scope is empty (`5daa8810b9`;
  300 starts at a 40 ms bound left 34 units before and none after,
  and a stop of such a unit took 4 to 5 ms).
  The issue is reopened for one unit the round's last whole-suite run left,
  which no single module reproduced.
  The coordinator stopped the twelve.
- Worktrees of ended,
  merged rounds are not all removed.
  A removal of eleven on 2026-11-27 removed none:
  five were held by those orphaned shells,
  three are dirty,
  and three are refused with "validation failed,
  cannot remove working tree:
  '.git' does not exist".
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
- Read the three release passes that run ("Rounds running") and route their findings.
- When a shadow tool's fix rounds have merged and `main` equals `origin/main`:
  release `mpm-shadow` `0.2.7-alpha.13` and `cargo-shadow` `0.1.5-alpha.12`,
  lock each on a branch worktree,
  install,
  put the session's `git` link back,
  and run a Claude Sonnet pass on each.
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
