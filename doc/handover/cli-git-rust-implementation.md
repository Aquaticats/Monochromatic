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
- Independent work runs at once.
  The owner,
  2026-11-27,
  between 20:10 and 20:12 -05:00,
  to a session that had five rounds live and ran its own steps one after another:
  "Update handover,
  tasks list,
  and parallelize better."
  Every ready item gets its round in the same turn,
  independent commands go out together,
  and a step waits only for an input it names or for a failure on record
  ("How work is run side by side").

## The state at 2026-11-27T23:23:34-05:00

`main` was `00740b0320`.
What landed before this time is in its own records and closed issues,
and the merges are in `git log --merges`.
Sixteen rounds merged on the evening of 2026-11-26,
and `git log --first-parent --merges --since='2026-11-27 00:00'` counts 325 more by this time.

### What changed between 2026-11-27T23:27-05:00 and 2026-11-28T02:33-05:00

Written at 2026-11-28T02:33-05:00 by the coordinating session,
unapproved and non-final.
The sections of this file that follow it still describe 2026-11-27T23:23:34-05:00,
and this section is folded into them at the next cut of the file.

#### The owner's answers

`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md` holds them verbatim.
In short:
the rounds of that night ride;
a later batch is bounded by guests alive;
GitHub's hosted runners come before any paid provider;
the crate split's round 4 waits for its signatures to be read;
a 128 GiB swap partition is cut from the old NVMe;
the 860 drive's unit is carried over and the old NVMe's data moves to it;
a use that wires a built ZFS piece into its command merges with its layout finding unrecorded
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`);
the removal's round merges and alphas go out as usual;
and `mvm` gains an OVHcloud backend whose billed tests are authorised
(`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`).

#### Releases

- The wrapper's `0.6.1-alpha.52`:
  the container gate on `229d5310eb` ran no test,
  because the machine killed its compiler;
  the gate on `af9c816379` ended 0
  (2026-11-28T00:57:04-05:00 to 01:22:28);
  the install proof over alpha.51 passed
  (01:34:38 to 01:42:54);
  dispatched 01:44:05,
  published by 01:52:15,
  release commit `bf46e60f6d`.
  Its lock row stays at `0.6.1-alpha.44` and the session's link is untouched.
  Two same-vendor second passes started after 02:01:51:
  one on the merges that change what a person meets,
  one on the merges that claim no change.
  Their reviews land in
  `~/temp/agent/git-policy-cli-alpha-52-release-review-behaviour-2026-11-28/keep/review.md` and
  `~/temp/agent/git-policy-cli-alpha-52-release-review-no-change-merges-2026-11-28/keep/review.md`.
- `cargo-shadow` `0.1.5-alpha.18`:
  the two rounds on the alpha.17 read were joined in one tree,
  whose package task failed once on `control_group_stop::report_tests::a_member_in_a_group_below_the_scope_is_named_where_it_stands_and_as_one_that_may_still_write`
  at 20.177 s and passed whole on the second run;
  merged as `9f088e122a` at 01:57:38,
  dispatched 02:07:22,
  published by 02:11:00,
  release commit `baa801f43a`.
  Its lock branch,
  its install and its second pass are owed.

#### Seven rounds joined and merged

`main` took seven ended rounds as one branch,
`feat/git-policy-cli-seven-ended-rounds-joined-in-one-tree`
(merge `17cb8febaf`,
2026-11-28T02:29:05-05:00),
each as its own merge inside it:
issue 6615's guest driver cure,
issue 6642's wrapper half,
`doctor`'s checks for plan steps 19 and 20,
the lost-event triggers' handler side and wrapper side,
the management grammar that admits `cancel-worktree-removal` and `settle-worktree-clone-record`,
and last the round that switches the removal's command path on.

The joined tree failed six controls,
all from the removal round's own change:
it had changed the words of `NO_REMOVAL_WORDS` and run neither the modules that spell them,
nor the whole `source_guard` module after its guest arm was added.
The fixes are one commit,
merged as `6697c0c204` at 02:29:58:
four controls read the constant;
the marked origin's ways forward no longer name the cancel command in the form where nothing is pending;
the removal arm's sweeper child has its line in the ledger of test children;
the sentence a self-cancelled removal prints no longer offers `--force` or a command "once it is added";
and three passages of `SPEC.md` and `README.md` say what is so with both rounds in.

**`main` held the six failing controls for 53 seconds**,
from `17cb8febaf` to `6697c0c204`:
the session's first commit of the fixes named no path,
the wrapper refused it,
and the script went on to push and merge the join without it.
A round that merged `main` in that minute meets those failures;
they are not its own.

Checked on the fixed tree:
`cargo fmt --check`,
the Clippy task with the test-only switches,
and six modules
(`source_guard`,
`health::doctor_zfs_route`,
`worktree_clone_slot_state`,
`worktree_removal_deferred_origin_check`,
`worktree_clone_removal_cancel` and `worktree_clone_mount_protocol`:
829 passed,
none failed).
Checked on the tree before the fixes only:
the other two Clippy tasks,
the binary's management tests,
the package's type check,
and the rest of the modules the seven rounds changed.
The fix commit's message and an earlier text of this section said all of those ran on the fixed tree;
they did not,
and a comment on commit `daa5132fc3` says so.
A whole gate on `main` waits behind a memory test and runs after it.

What the rounds left,
by their own reports:

- The removal's guest arm is written and not green:
  its third run stopped at the mount handler's `user-cache` step,
  because the arm's fixture writes no user-cache pin.
- Almost none of the removal round's new controls was shown able to fail,
  and its plant of the live pass's no-folder guard opened `/dev/zfs` on this host once
  (2026-11-28T01:10-05:00;
  opened only,
  no request sent).
  The owner was told.
- Two guest arms of the slot state still expect the old refusal.
- The grammar round's three guest arms never printed a result:
  the machine ended its three attempts.
- The check before a forwarded command has no call site,
  and its cost is not measured.
- Issue 6615's cure removes the leftover mount,
  and the arm did not end 0 in any of eight runs after it,
  for the arm's own thirty-second wait under load and for what a run that dies inside the arm leaves.
  The issue stays open.
- A test of the wrapper wrote a 17-byte file named `add_route` into the package folder once.

#### The layout findings `main` prints, which no line records

The repository linter on `package/git-policy/cli` prints these eleven `rust/folder-uses-links` findings,
all from the grammar round's wiring,
and fourteen `rust/max-lines` findings.
A `rust/folder-uses-links` finding outside this list is a new tangle.

- `src/native/invocation/management.rs` line ?:
  `invocation` uses `worktree_clone_cancel_and_settle_commands`;
- `src/native/invocation/management.rs` line ?:
  `invocation` uses `worktree_clone_removal_cancel`;
- `src/native/invocation/management_arguments.rs` line ?:
  `invocation` uses `worktree_clone_record_settling`;
- `src/native/worktree_clone_cancel_and_settle_commands/a_cancel_command_answered.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree_clone_removal_cancel`;
- `src/native/worktree_clone_cancel_and_settle_commands/a_settling_command_answered.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree_clone_record_settling`;
- `src/native/worktree_clone_cancel_and_settle_commands/a_settling_command_answered.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `zfs_mounted_dataset`;
- `src/native/worktree_clone_cancel_and_settle_commands/the_mount_request_a_cancellation_files.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree_clone_mount_protocol`;
- `src/native/worktree_clone_cancel_and_settle_commands/the_mount_request_a_cancellation_files.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree_clone_removal_proof_and_plan`;
- `src/native/worktree_clone_cancel_and_settle_commands/the_place_the_two_commands_run_in.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `repo_read`;
- `src/native/worktree_clone_cancel_and_settle_commands/the_place_the_two_commands_run_in.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree`;
- `src/native/worktree_clone_cancel_and_settle_commands/the_pool_a_command_reads.rs` line ?:
  `worktree_clone_cancel_and_settle_commands` uses `worktree_clone_mount_handler`.

#### Rounds started at 2026-11-28T02:31:42-05:00

One batch of five,
on the virtual writer:

- `mvm-cli-ovhcloud-backend-beside-the-hetzner-one`:
  builds the backend and its unit tests,
  and makes no call with the keys;
  the session runs its billed test from the main checkout.
- `zfs-removal-switch-runs-green-in-a-guest-and-its-controls-can-fail`.
- `zfs-removal-switch-sentences-say-what-the-running-removal-does`.
- `zfs-grammar-commands-guest-arms-run-green`.
- `zfs-lost-event-check-before-a-command-wired-with-its-cost-measured`.

Nine rounds of 2026-11-27 were still running at 01:55:
the two that gather `doctor`'s facts,
the private propagation step,
the shutdown unmount body,
the two on the forced arm,
and the guest arms of plan steps 19,
21 and 22.
The Dylint driver proposal's revision round never ran on 2026-11-27:
its start ended without a run at 23:03,
because the secrets file could not be decrypted while the machine was out of memory,
and the session listed it as running until 2026-11-28T02:49.
It was started again at 02:50:06.

Owed after them:
the removal's five-point kill matrix,
its delegated batch,
its rollback arm and its split-batch arm in a guest,
and the removal's own entries in the folder of unanswered requests.

#### Started and done between 02:33 and 02:51

- Four rounds answer two second-pass reads,
  started at 02:43:02:
  two on the linter's alpha.24 read
  (the allow-list reader's cost and sentences;
  the line-break fix's passes,
  its line limit and its README)
  and two on `mpm-shadow`'s alpha.19 read
  (two installs at once and what the removal keeps;
  four sentences and a name too long to link).
  Every finding of both reads is routed to one of them,
  and each brief says what the session made of it.
- Two rounds write proposals for the owner,
  started at 02:45:56:
  the crate split's five changed signatures of round 4,
  and what GitHub's hosted runners should carry.
- `cargo-shadow` `0.1.5-alpha.18` is locked
  (merge `4cf7cc74b8`)
  and installed
  (02:38:02),
  the session's wrapper link was restored after the install,
  and a same-vendor second pass reads the release.
  Issue 6645 is closed with what merged.
- Issues filed:
  6649
  (a 17-byte file named `add_route` appeared in the wrapper's package folder,
  writer not known),
  6650
  (the `cargo-shadow` control that failed once at its 20-second bound),
  and 6651
  (a Pi run ends `pi-not-stopped` when Pi outlives SIGKILL by 5000 ms on a host out of memory).
  Issues 6615 and 6642 have comments with what their rounds measured,
  and both stay open.
- Swap was nearly full again at 02:44:23
  (`SwapFree` 3,280,544 kB),
  so a 64 GiB swap file on the 860 drive bridges until the old NVMe's partition is up
  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
  the nineteenth batch).
  It is in no `fstab`,
  and while it is on the 860 drive's unit cannot stop.
- One whole container gate started on `main` at `3d64028ef3` at 02:48:47,
  after free memory plus free swap read 40,000,000 kB on three readings a minute apart.
  Its log is `~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-3d64028ef3-20261128T024847-0500.log`.
- The OVHcloud round's brief said its worktree holds no secrets file.
  It does,
  because the worktree's creation copies ignored files,
  and a correction reached the round's inbox at 02:49:15:
  no call with the keys,
  and the five names unset in front of every check.

#### Started and done between 02:51 and 03:16

- The wrapper's two second-pass reads of `0.6.1-alpha.52` ended,
  both by a same-vendor reader,
  neither with a High finding.
  The behaviour read found one Medium:
  with `core.worktree` set in a plain main checkout,
  a bare `!git` alias typed in a subdirectory with `-C <a clean worktree>` resets the main checkout,
  and the wrapper judges the clean worktree and lets it run.
  The release before does the same.
  The read of the merges that claim no change of a release build found two Mediums in the guest drivers:
  an interrupted driver leaves its recipe step running,
  and a launcher that was refused is no longer refused at once.
  Four rounds answer the two reads,
  started at 02:55:24 and 03:13:06:
  the reading of a `!` alias,
  the `final-newline` hints,
  the guest drivers' shared steps,
  and the surface control with three source guards.
  Two things stay the owner's:
  whether an alias the wrapper cannot read is refused,
  and the three refusals that issue 6641's closing comment records.
- The round on GitHub's hosted runners ended at 03:06:55 and merged as `6b06c7149c`:
  `doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md`.
  It measured that `advisory-core`'s window counts skipped runs,
  so it admitted none of its last 200,
  and that a standard public runner opens `/dev/kvm` after one `setfacl` and boots a small guest in 44 seconds.
  A gate read by a same-vendor reader runs,
  and the owner is asked the proposal's five questions after it.
- The round that closes the last file loop in `foundation/` started at 03:13:06,
  on the third way the questions file records,
  which is the session's pick with the owner's veto open.
- `mise run //package/toml-edit/core:test:unit` and `mise run //package/logger/core:test:unit` exit 1 on `main`,
  each against a built copy older than its sources
  (issue 6637,
  with a comment that holds the measurement).
  Built first,
  the first passes,
  and the second keeps one failing control on a loaded host
  (issue 6652).

#### Started and done between 03:16 and 03:30

- The whole gate on `main` at `3d64028ef3`,
  which holds the seven joined rounds,
  passed at 03:20:06:
  `12474 tests run: 12474 passed (7 slow), 51 skipped`
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` lists it with the two gates of `0.6.1-alpha.52`).
  A second gate started at 03:23 on `05cf4c8820`,
  because the merge after the first one changes sentences the binary tests assert.
- How the next alpha goes,
  the session's pick with the owner's veto open:
  wrapper branches keep merging,
  because no gate holds a merge;
  the alpha ships from the tip once a gate is green,
  and its record names the gated commit and each wrapper merge after it with the checks that merge had.
  One thing it waits for is the lock control described under the next point.
- The round that makes every sentence beside a removal say what the running removal does merged as `b0c8bed3e8`.
  It changed words only,
  and it found behaviour that departs from the records:
  - a slot whose removal stopped part way,
    with its destruction plan standing,
    gets no answer from the wrapper,
    so `git worktree remove` there goes to Git,
    where the accepted design has the removal's next run resume the plan
    (issue 6653,
    measured by a host probe);
  - nobody has measured what a removal does at a whole clone whose registration holds a lock a person wrote:
    no file of the removal's entry reads the lock,
    and the only file of its order that names the lock is the Git settle,
    after the destroy;
  - two guest arms still assert the refusal the release gave before the removal was switched on
    (`binary_zfs_slot_state_guest_tests.rs` and `binary_zfs_managed_lock_guest_tests.rs`).
  Two rounds started at 03:27:53.
  The first works on the host
  (`fix/git-policy-cli-zfs-removal-entry-resumes-from-its-standing-plan-and-reads-a-persons-lock-before-it-destroys`):
  its first commit is a control that answers the lock question,
  and then the refusal before the first write if anything is destroyed,
  the resume from a standing plan,
  and the sentences.
  The second restructures the slot-state guest arm in a guest.
  If the lock control shows a destroy under a person's lock,
  the owner is asked before the alpha ships,
  or it ships once the refusal has merged.
  Still owed after them,
  in a guest:
  the lock arm's removal assertions,
  and the removal's kill matrix with a resumed removal.
- The first of `mpm-shadow`'s two rounds on the alpha.19 read merged as `be3a61bcd4`:
  a link another run took is no failure at any of the removal's three calls,
  a temporary link the run cannot judge is kept and named,
  and the program's own link is kept only for an install of this program.
  Eight of its controls were red on the released code.
  One choice of its own to weigh at the next second pass:
  a name an earlier release left is now kept and named on every install,
  where it used to be removed when its process number was free.
- The crate split's round-4 proposal merged as `3e05dae625`:
  `doc/planning/cli-git-crate-split-round-4-the-five-changed-signatures.md`,
  with five questions for the owner.
  Two of its first choices depart from the plan's own signature
  (one answer per process for the system settings path,
  and a value in place of a thread-local for the step record).
  A gate read by a same-vendor reader runs,
  and the owner is asked after it.
- The measurement of the wrapper's development profile is briefed and waits behind a memory test
  (free memory plus free swap at 80,000,000 kB on three readings a minute apart),
  because it builds the crate about a dozen times.

#### Started and done between 03:30 and 05:16

Unapproved and non-final:
every merge named here went in on its round's ended report,
read whole,
and no gate has read `main` since `05cf4c8820`.

- The owner corrected the session:
  an option had said "an encrypted swap partition" and the session then chose how.
  Two rules followed in `AGENTS.md`
  (`OHW`:
  an option says how it works;
  `YSC`:
  a yes covers only what the option's text stated;
  commit `87c10d6f9d`),
  and the answers are in
  `doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`
  (commit `352967bddf`).
- The gate on `05cf4c8820` ended red at 03:55:53:
  `12475 tests run: 12471 passed (10 slow), 4 failed, 51 skipped`.
  All four are in `native_binary`,
  and each met a bound while the host was stalled on memory
  (`MemAvailable` 0 kB and `/proc/pressure/memory` `full avg10=49.62` at 03:51:23,
  with the new swap partition on and 131 GB of swap free):
  `fault_trace::storage_that_fails_during_a_commit_leaves_a_recoverable_repository`
  (native Git's own commit ended at the control's 60 s bound),
  `lfs_pre_push::a_failing_upload_blocks_the_reconcile_push_of_an_auto_push`,
  `lfs_server::a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes`
  and `lfs_pre_push::the_reconcile_push_of_an_auto_push_runs_the_upload_before_the_ref_moves`.
  Owed:
  its line in the gate's troubleshooting record,
  and an issue for the four bounds after a search.
- The lock control answered the question the alpha waited for
  (round `zfs-removal-entry-resumes-from-its-standing-plan-and-reads-a-persons-lock-before-it-destroys`,
  merged as `2d5f167ae9`):
  on `main` a removal of a slot under a person's own lock sent the clone's snapshot request,
  the head destroy and the origin snapshot's request before anything refused.
  The removal now reads the lock before its first write and refuses every spelling,
  naming `git worktree unlock <slot>`;
  and a slot a destruction plan names is the removal's own,
  resumed from the plan
  (issue 6653).
  Three choices of it wait for the owner:
  what two forces do at a person's lock,
  a plan that does not read,
  and a clone mounted at the staging folder.
  Two guest arms assert the behaviour from before it
  (`binary_zfs_managed_lock_guest_tests.rs` and `binary_zfs_slot_state_guest_tests.rs`),
  and the kill matrix and the lock refusal on a real slot are owed in a guest.
- A third gate was started behind a memory test at 05:14,
  on the tip that holds `2d5f167ae9`:
  three readings a minute apart with `MemAvailable` over 6,000,000 kB and `full avg60` under 10,
  then `native:gate:main-tip`.
  Its steps and verdict go to
  `~/temp/agent/wrapper-gate-on-main-after-the-lock-refusal-merge-2026-11-28/keep/`.
  The wrapper's next alpha follows its verdict.
- The four fix rounds on the alpha.52 reads are on `main`:
  the surface control and three guards (`039e39b885`),
  the `final-newline` hints and the guest drivers' shared steps (joined,
  `79c2100e94`),
  and the shell alias judged where Git acts (joined,
  `b742da6ca8`).
  All four appended to `package/git-policy/cli/doc/release-pass-findings-2026-11-27.md`,
  so the two joins wrote that record from the commits themselves.
- The check before a command is wired
  (`fcaea34f36`):
  one call in `run_wrapped_command`,
  measured at 1.40 µs a call,
  0.022% of the cheapest forwarded command.
  It is the twelfth `rust/folder-uses-links` finding `main` prints
  (`src/native/invocation/wrapped_command.rs` line 537),
  unrecorded as the amendment of 2026-11-28 permits.
  The design's sentence that the check is compiled out with the nudge's switch off no longer holds;
  the pick,
  with the veto open,
  is to keep the check and amend that sentence.
- Plan step 19's guest arms merged as `18327d05f6`:
  the unit texts and drop-ins read by a real manager,
  the immutable entry's verdict on a real dataset (six of seven lines;
  `other-owner` is committed and unproven),
  `snapdir=disabled` set and inherited,
  the migration and its rollback run step by step,
  and six corrections of `doc/runbook/zfs-checkout-migration-and-boot-setup.md`.
  The reboot arm's idle-clone half waits for a guest whose boot the route's gate accepts:
  the recipe's guest boots its packaged 2.2.2 module.
- The round `zfs-grammar-commands-guest-arms-run-green` ended at 03:40:28 without a report:
  its agent was killed after a 40-minute wait for a free guest slot,
  at the memory stall.
  Its branch holds four commits and no guest arm ran.
  A fresh round on that branch is owed.
- `mpm-shadow` `0.2.7-alpha.20` is locked (`6b9eb2e041`) and installed,
  and `monochromatic-lint` `0.5.2-alpha.25` was published at 2026-11-28T09:54:50Z,
  locked (`4ca36f9963`) and installed.
  A second pass of each is owed.
  The second pass of `cargo-shadow` `0.1.5-alpha.18` came back with no High,
  two Medium and six Low points
  (`~/temp/agent/cargo-shadow-alpha-18-release-review-2026-11-28/keep/review.md`,
  scratch);
  its fix round is owed.
- `mvm` gained an `ovh` backend
  (`3ab6ae05c9`),
  a fix to its default regions
  (`6ce12fab98`)
  and a fix to its image choice
  (`8200df4622`).
  Two billed runs were made and neither gave a machine that answered;
  no instance is left.
  The owner chose no option for a third run and wrote that `mvm` should take vCPUs and memory and pick the size.
  Everything the owner said and what each run did is in
  `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`;
  the design is `doc/planning/mvm-create-takes-vcpus-and-memory-and-picks-a-size.md`
  (`ed7bfaad44`),
  whose gate read was started at 05:12.
  The AppArmor rules it needed are the twentieth batch.
- Proposals:
  the Dylint proposal's revision merged as `8bf343a7cf` and its second gate read was started at 04:40;
  the line-break proposal's second revision merged as `8638a53928`,
  and its third gate read answered not ready,
  with three High points and no survey rerun needed;
  the hosted-runners proposal's gate read answered not ready with five High points;
  the crate split's round-4 proposal's gate read answered not ready with five High points.
  A revision round for each of the last three is owed,
  and none of their questions has gone to the owner.
- The push hook did not push merge `8200df4622` within 150 s;
  it was pushed by hand,
  and the merges after it are checked a minute after each and pushed by hand where the hook has not.
- Rounds running at 05:16,
  from the unit list:
  - `foundation-the-words-of-the-stopped-reader-move-below-the-queue-and-the-last-file-loop-closes`
  - `zfs-doctor-gathers-each-managed-clones-facts-from-the-live-system`
  - `zfs-doctor-gathers-the-machines-route-facts-from-the-live-system`
  - `zfs-forced-arm-ends-a-holder-in-a-taskless-mount-namespace`
  - `zfs-forced-arm-stops-a-locked-holders-own-user-unit`
  - `zfs-private-propagation-step-built-ahead-unwired`
  - `zfs-removal-switch-runs-green-in-a-guest-and-its-controls-can-fail`
  - `zfs-shutdown-unmount-body-built-ahead-unwired`
  - `zfs-slot-state-guest-arm-asserts-what-the-switched-removal-does`
  - `zfs-step-21-guest-arms-for-dependency-repair-and-watches`
  - `zfs-step-22-guest-half-gaps-pinned-output-and-a-parent-above-64-clones`

#### Started and done between 05:16 and 05:52

Unapproved and non-final.

- The machine's memory stall after 05:20 was one unit:
  every guest `mvm` starts runs in `mvm-libvirt-session.service`,
  which `mvm` limited to half the machine's memory,
  and eight guests that had touched 58.2 GiB swapped against that limit while the machine had memory free.
  The reading is in `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
  "The guests' unit at its own memory cap,
  2026-11-28".
  The session's bound of nine guests alive had never been set against that limit.
- The owner answered four questions on it
  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
  "The guests' memory"):
  three quarters of memory for the unit with six guests alive;
  applied to the running unit at once
  (`MemoryMax=47364M` at 05:45:26,
  a runtime setting that a reboot undoes);
  the shelved guest-memory proposal's question on building the test binaries outside the guest reopened;
  and guests on OVHcloud pursued,
  with no memory bought.
  Within two minutes of the change the unit's and the machine's pressure read 0.
- A guest round creates its guest only when fewer than six are alive,
  and waits for a slot inside one command,
  never as an idle agent.
  No round is given a test on the machine-wide pressure line as a gate for its builds:
  that line read the guests' unit,
  and a measurement round waited 1,323 s behind it.
- Six of the eight guests had been made without free page reporting,
  because the built `mvm` the rounds ran was older than its source
  (issue 6655).
- Four rounds started at 05:50:33 for those answers:
  the unit's limit in `mvm`'s code,
  issue 6655,
  the measurement of test binaries built outside the guest,
  and the revision of the `mvm` sizes proposal against its gate read
  (not ready,
  five High points).
  The third billed OVHcloud run stays unapproved.
- A whole gate has run on `main` at `d4faaf0807` since 05:39:53
  (`gpc-gate-wrapper-next-alpha-on-d4faaf0807-20261128T053953-0500`),
  started behind a test on `MemAvailable`;
  the guests' unit swapped beside it until 05:45:26.
  The wrapper's next alpha follows its verdict.
- Merged:
  the development-profile measurements
  (`888d3646df`;
  incremental compilation lowers the largest compiler's memory by a third on a rebuild and does not raise it on a first build;
  its proposal is under a gate read),
  and the forced arm's stop of a locked holder's own user unit
  (`d4faaf0807`;
  built and measured in a guest,
  unreachable while the stop's wait has no value).
- Second passes started for `monochromatic-lint` `0.5.2-alpha.25` and `mpm-shadow` `0.2.7-alpha.20`.
  `toolchain:update` in a lock worktree had already put both new binaries into the shared toolchain root while rounds ran:
  the update is the step that changes the shared binaries,
  not `toolchain:install`.
- Gate reads came back not ready for the Dylint proposal's revision (four High points)
  and the line-break proposal's second revision (three High points);
  their next revisions,
  and those of the hosted-runners and crate split proposals,
  are owed,
  as is the fix round for the two Medium and six Low points of the `cargo-shadow` `0.1.5-alpha.18` read.
- Three choices the removal's lock and plan round left are in `doc/handover/questions-for-the-owner.md`.
- Rounds running at 05:52,
  from the unit list:
  - `zfs-doctor-gathers-the-machines-route-facts-from-the-live-system`
  - `zfs-forced-arm-ends-a-holder-in-a-taskless-mount-namespace`
  - `zfs-private-propagation-step-built-ahead-unwired`
  - `zfs-removal-switch-runs-green-in-a-guest-and-its-controls-can-fail`
  - `zfs-shutdown-unmount-body-built-ahead-unwired`
  - `zfs-slot-state-guest-arm-asserts-what-the-switched-removal-does`
  - `zfs-step-21-guest-arms-for-dependency-repair-and-watches`
  - `zfs-step-22-guest-half-gaps-pinned-output-and-a-parent-above-64-clones`

#### Started and done between 05:52 and 06:14

Unapproved and non-final.

- On the removal the owner answered what force does:
  "--force overrides anything and delete and destroy no matter what.",
  and on its reach,
  "Two force does everything.
  One force - you pick."
  A destruction plan that cannot be read and a clone at the staging folder refuse without force,
  and the foundation folder's loop stays closed as built
  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
  "What force does on a ZFS removal").
  The session's pick for one force is recorded there with the veto open.
  Two mechanisms are recorded as not yet decided:
  how a command that ends its own ancestors still finishes,
  and by what act two forces pass other processes that hold the clone.
- Two rounds started for it at 06:08:
  one builds the part that involves no holder on the host
  (`zfs-removal-reads-force-two-forces-pass-every-refusal-and-two-states-refuse-without-it`,
  steered at 06:12 to settle what one force does at modified or untracked files),
  one designs and measures the two undecided mechanisms in a guest
  (`zfs-two-forces-pass-holders-and-the-2026-11-23-limits-designed-and-measured`).
  The wrapper's next alpha,
  if it ships from the gate now running,
  still refuses two forces at a person's lock,
  with a sentence the owner has since overruled:
  its record says so.
- Merged:
  the foundation folder's last file loop
  (`74f34f6af9`),
  and the gatherer of each managed clone's facts for `doctor`,
  joined with `main` because both sides had added lines at one place in two list files
  (`a2ff2180e9`;
  on the joined tree Clippy twice and 553 tests of the touched modules and the whole source guard passed).
- The limit raised on the guests' unit lives on the running unit alone.
  Every built `mvm` on the machine still starts that unit at half,
  so a later start agrees only once the round that changes `mvm`'s code has merged and the builds are fresh
  (issue 6655).
  A sampler writes the unit's limit each minute into
  `~/temp/agent/wrapper-gate-on-main-after-the-lock-refusal-merge-2026-11-28/keep/guest-unit-each-minute.log`.
- Issues filed:
  6655 (the `mvm` command runs a built copy older than its source),
  6656 (four gate controls at their bounds when the host has no available memory),
  6657 (a merge left unpushed for more than 150 s,
  and an owed push of 2026-11-12 that names a path that is gone).
  Owed still:
  an issue for `mise run //package/mvm/cli:test:unit` hanging in the libvirt test files on this host,
  once the round on the unit's limit reports which file it hangs in.
- The development-profile proposal's gate read answered not ready
  (four High points:
  the memory case rests on rebuilds where the recorded failure was first builds,
  and the arms measured the everywhere option,
  not the one ranked first);
  its revision round is started.
  Revision rounds for the line-break,
  Dylint,
  hosted-runners and crate split proposals start behind the morning's first launcher,
  and the two `cargo-shadow` fix rounds and the second run of the grammar words' guest arms start when the gate's unit ends
  (one waiting command starts both groups).
- The lock procedure,
  corrected:
  `mise run toolchain:update -- <key>` in a lock worktree installs the new binary into the shared toolchain root at once,
  so that step,
  not `toolchain:install`,
  is the one that changes the tools rounds are running.
- Rounds running at 06:14,
  from the unit list:
  - `mvm-cli-the-mvm-command-never-runs-a-build-older-than-its-source`
  - `mvm-guest-test-binaries-built-outside-the-guest-measured-on-the-zfs-recipes-guest`
  - `mvm-sizes-proposal-revised-against-its-gate-read-with-prices-and-every-command`
  - `zfs-doctor-gathers-the-machines-route-facts-from-the-live-system`
  - `zfs-forced-arm-ends-a-holder-in-a-taskless-mount-namespace`
  - `zfs-private-propagation-step-built-ahead-unwired`
  - `zfs-removal-reads-force-two-forces-pass-every-refusal-and-two-states-refuse-without-it`
  - `zfs-removal-switch-runs-green-in-a-guest-and-its-controls-can-fail`
  - `zfs-shutdown-unmount-body-built-ahead-unwired`
  - `zfs-slot-state-guest-arm-asserts-what-the-switched-removal-does`
  - `zfs-step-21-guest-arms-for-dependency-repair-and-watches`
  - `zfs-step-22-guest-half-gaps-pinned-output-and-a-parent-above-64-clones`
  - `zfs-two-forces-pass-holders-and-the-2026-11-23-limits-designed-and-measured`

#### Started and done between 06:14 and 06:58

Unapproved and non-final.

- The owner asked why local guests were still being started three hours after pointing at OVHcloud.
  The session had put a proposal and its reviews in front of the third billed run.
  The owner then gave a standing yes for billed runs by the session and by its rounds,
  inside bounds (`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
  "A standing yes for billed runs,
  2026-11-28"),
  and chose a new wording of rule `URF` in `AGENTS.md` (commit `2b222d450b`).
  Before it ends a turn the session lists the project's machines and deletes leftovers named `mvm-*`.
- No name resolved on this machine from 06:21:08 until the owner reconnected IVPN at 06:36:00
  (`doc/troubleshooting/ivpn-3-15-inverse-split-tunnel-overrides-wireguard.md`,
  "Name lookups ride the tunnel for every program").
  Ordinary traffic was outside the tunnel the whole time,
  and neither OpenSnitch nor AppArmor was involved.
- An OVHcloud machine answered:
  a `d2-2` from the image `Ubuntu 26.04` in `BHS5`,
  running 134 s after the create call in one run and 57 s in the next,
  with a command,
  a push and a pull passing in both.
  Merge `dcbb146340` makes `mvm`'s listing leave out an instance the project lists as `DELETED`.
  Still open:
  `destroy` returns while the machine is listed as `DELETING` (read 16 s after the delete call),
  so the billed test's last check fails;
  the round `mvm-cli-ovh-destroy-returns-when-the-instance-is-gone` started at 06:54:55.
  The project listed no machine at 06:56:43.
- Two rounds toward guests on OVHcloud run since 06:24:
  `mvm-cli-create-and-run-take-vcpus-and-memory-and-pick-the-smallest-size-that-fits`
  (the owner's sizing sentence,
  built without waiting for the proposal)
  and `guest-drivers-take-the-mvm-backend-from-one-place-and-one-arm-runs-on-an-ovhcloud-machine`.
  Each has an inbox note that `main` holds `dcbb146340` and `9afd7194ab`.
  The session starts no new round on a local guest once both are merged.
- Merge `9afd7194ab` (issue 6655) makes the `mvm` command run the source beside it,
  and the main checkout was built so that worktrees inherit the launcher.
  Its report leaves for issues:
  `mvm-mcp` and twenty other commands still run a build that can be older than their source,
  and one case of `cli-session-daemon.unit.test.ts` failed in one of three suite runs.
  Its question on what a published `mvm` runs is the owner's and off the stated priority.
- Merge `5a84e361c0` puts the guests' unit at three quarters of memory in `mvm`'s code;
  the main checkout's build computes `47364M`.
- The gate on `d4faaf0807` was green
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`),
  and the wrapper's `0.6.1-alpha.53` was published at 2026-11-28T06:49:18-05:00
  and unpacked under `.cache/wrapper-release-0.6.1-alpha.53/`.
  It does not hold the owner's force answers of 06:01 to 06:05.
  Two second passes by Claude Sonnet started at about 06:57;
  their reviews go to
  `~/temp/agent/git-policy-cli-alpha-53-release-review-behaviour-2026-11-28/keep/review.md` and
  `~/temp/agent/git-policy-cli-alpha-53-release-review-no-change-merges-2026-11-28/keep/review.md`.
- Merge `471d462ccf` holds the development-profile proposal's revision;
  its second gate read started at about 06:57 and writes to
  `~/temp/agent/second-gate-review-of-the-development-profile-proposal-2026-11-28/keep/review.md`.
- Three rounds ended at 06:35:31 with `model-error`:
  their providers could not be reached while names did not resolve,
  and the last one in the writer's list answered that its usage limit was reached.
  They are the line-break revision,
  the measurement of guest test binaries built outside the guest,
  and the design of what two forces do at a holder.
  Each was resumed from its own session at 06:47.
  A resume names the run by its folder's name:
  a whole path in `sessionOf` ends the start with a stack trace,
  which is owed an issue.
  A round that falls through to the provider at its limit ends the same way again.
- The linter's `0.5.2-alpha.25` second pass found no High,
  no Medium and eight Low;
  its fix round is not started.
- At 06:49:21 a round read 106 MiB available on the host with the guests' unit under its raised cap,
  which is the state the move to OVHcloud is for.

#### The line-break proposal

Its revision merged as `5bb8813b28` at 2026-11-28T02:05:36-05:00.
The second gate read,
by a same-vendor reader,
found two High findings and says the proposal is not ready for the owner:
two of the ranking's reasons count another population or unit than the option they judge,
and the repair option rests on a route that reached 90 of 548 lines,
on a tree `main` has since left.
A round revises it,
started at 03:06:29,
and runs the survey again at its own commit
(`docs/planning-line-break-proposal-revised-against-its-second-gate-read-with-its-survey-run-again`).
A third gate read follows it,
and the owner is asked after a read that finds it ready.

#### The machine

- The 860 drive:
  `mount-860.service`,
  its key file and its polkit rule are installed,
  with the drive named by its LUKS UUID.
  Its first start failed because AppArmor's `cryptsetup` profile could not read the key folder
  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
  the eighteenth batch).
  It has run since 01:50:27,
  and a stop and a start as the account,
  without `sudo`,
  worked at 01:57:08 and 01:57:22.
  The drive's file system holds about 1.84 TB and has between 1.77 and 1.94 TB free.
- The swap partition:
  a root unit,
  `old-nvme-btrfs-shrinks-by-128-gib-for-a-plain-swap-partition-2026-11-28`,
  balanced the old NVMe's file system from 01:40:55 to 02:07:20,
  and has been shrinking device 1 by 128 GiB since.
  Its log is `~/temp/agent/old-nvme-shrink-for-a-plain-swap-partition-2026-11-28/keep/shrink.log`.
  After it:
  the partition table,
  the mappings,
  the crypttab and fstab lines,
  and the record beside the writeback partition's in
  `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`.
- The old NVMe's data,
  about 1.33 TB,
  moves to the 860 drive after that,
  and no partition is formatted before the owner is asked.

### Rounds running

At 2026-11-27T23:23:34-05:00 these 18 rounds are live,
counted from the process list.
Each runs on Pi's virtual writer,
with its brief at `~/temp/agent/pi-runs/<round>-prompt.md`,
its run under `~/temp/agent/pi-runs/<round>-1`,
its worktree `.claude/worktrees/pi-<round>`
and its report at `~/temp/agent/<round>-scratch/keep/report.md`.
The time after each name is that of its first feed line.

Guest rounds:

- `issue-6615-the-adoption-drivers-holder-scan-arm-runs-twice-in-one-guest-and-its-snapdir-pattern-means-a-dot`,
  2026-11-27T20:00:25-05:00;

For the wrapper:

- `issue-6642-the-wrappers-doc-comments-lose-the-lone-words-the-old-fix-left`,
  2026-11-27T22:18:36-05:00:
  issue 6642's last ask:
  the lone words the old fix left in the wrapper's doc comments,
  and one doc fence closed;

For what the read of `cargo-shadow`'s `0.1.5-alpha.17` found
("What `cargo-shadow`'s alpha.16 round did,
and its alpha.17";
issue 6645):

- `cargo-shadow-alpha-17-release-pass-a-start-that-heals-and-a-queued-stop-keep-the-delegated-route`,
  2026-11-27T21:37:37-05:00:
  the two Medium points and the control that holds them;

For the linter's two proposals,
each revised against its gate review before it is put to the owner
("What the gate reviews of the linter's two proposals found"):

- `line-break-proposal-revised-against-its-gate-review-with-its-survey-kept`,
  2026-11-27T23:00:54-05:00:
  records and a survey that can be run again;
  no rule and no fix;

For the ZFS route
("The ZFS worktree route"),
from the list `batch-of-2026-11-27-the-fourteen-zfs-route-rounds`,
briefed from a digest of what is left with every quoted record line checked against its file:

- `zfs-removal-command-path-runs-the-whole-removal-with-force-still-refused`,
  2026-11-27T21:12:04-05:00:
  the switch itself:
  `git worktree remove` of a bound clone runs the whole removal,
  the cut test goes,
  and the request budget gets its constant;
- `zfs-management-grammar-admits-the-cancel-and-settle-commands`,
  2026-11-27T21:12:07-05:00:
  merged before the switch,
  whose sentences name the cancel word;
- `zfs-doctor-gathers-the-machines-route-facts-from-the-live-system`,
  2026-11-27T21:13:06-05:00;
- `zfs-doctor-gathers-each-managed-clones-facts-from-the-live-system`,
  2026-11-27T21:13:15-05:00;
- `zfs-doctor-checks-that-steps-19-and-20-owe-and-the-cancel-commands-line`,
  2026-11-27T21:13:20-05:00;
- `zfs-private-propagation-step-built-ahead-unwired`,
  2026-11-27T21:13:38-05:00:
  the step and its guest arm,
  with no call from a root mode;
- `zfs-shutdown-unmount-body-built-ahead-unwired`,
  2026-11-27T21:13:44-05:00:
  the body and its guest arm,
  with the mode still refusing;
- `zfs-lost-event-triggers-wrapper-side-built-with-the-nudge-off`,
  2026-11-27T21:14:03-05:00;
- `zfs-lost-event-triggers-handler-side-built-ahead-unwired`,
  2026-11-27T21:14:23-05:00:
  the event filter and the merge device,
  with no call from the sweep;
- `zfs-forced-arm-ends-a-holder-in-a-taskless-mount-namespace`,
  2026-11-27T21:14:41-05:00:
  in a guest made from plan step 23's template,
  whose kernel has `listns`;
  steered once,
  at 2026-11-27T21:20:23-05:00,
  to read which module that guest runs after its prepare;
- `zfs-forced-arm-stops-a-locked-holders-own-user-unit`,
  2026-11-27T21:15:02-05:00;
- `zfs-step-19-written-guest-arms-and-the-reboot-arms-run`,
  2026-11-27T21:15:26-05:00;
- `zfs-step-21-guest-arms-for-dependency-repair-and-watches`,
  2026-11-27T21:15:44-05:00;
- `zfs-step-22-guest-half-gaps-pinned-output-and-a-parent-above-64-clones`,
  2026-11-27T21:16:02-05:00;

Every round live at 2026-11-27T22:54:05-05:00 got one note in its inbox:
the machine had run out of memory ("Memory"),
a build ended by signal 9 or in a scope whose result is `oom-kill` was ended by the machine and is run again,
and,
as the coordinating session's suggestion and no rule,
one reading of `/proc/meminfo` before a build that compiles the wrapper.
The note is `~/temp/agent/memory-exhausted-on-the-night-of-2026-11-27/keep/note-sent-to-the-live-rounds.md`,
scratch that expires a day after its last change.
A report that says a build was killed is read against the host's log for that minute.

All four rounds on this file's history are merged:
part B alone (`4d2fae7e63`),
parts A and C joined in one tree (`c8545f91b8`),
and part D joined with `main` (`00740b0320`),
each join because the parts added to the same package records
(the linter's README in both).
Each conflict kept both sides,
after the figures were read in this file as it stood before the cuts (`4bbd9168dc`).
The file went from 4,465 lines to 2,561,
the coordinating session's own additions of the night included.
The coordinator read no ledger whole.
Left by the merged parts for one read of this file:
the paragraph on issue 6638's round inside "Rounds that ended,
and what each left" is landed work no ledger named,
and passages elsewhere still point at that section for events the cuts removed.
Part D's run ended with the driver's reason `pi-not-stopped`,
"Pi was still running 5000 ms after SIGKILL",
after the round had settled with its work pushed;
no process of it was left.

A branch that took `main` in before another merge changed its package is checked on the tree that holds both,
in a detached worktree of the coordinating session's own under `.claude/worktrees/`,
and pushed from that worktree's root:
the wrapper refuses a push typed below it,
which a check's own log shows and its summary line may not.
A round's worktree is gone once its run ends,
and the local branch it leaves stands at the round's tip,
so a branch pushed again after such a check is merged from `origin`'s ref.
Branches of one package that end together are joined in one such worktree and merged as one branch.

Two Claude Sonnet agents of the coordinating session run beside the rounds,
each a same-vendor pass and none independent:
the second pass of the linter's `0.5.2-alpha.24`
and the second pass of `mpm-shadow`'s `0.2.7-alpha.19`.

When a round ends:
its report is read whole,
its branch merged as "Next" says,
its issue closed by hand,
and what its merge frees is started.
The wrapper's gate has to run again ("What the wrapper's alpha.51 reads found");
after a green one come the install proof over alpha.51,
`0.6.1-alpha.52` and its two second passes.
After `cargo-shadow`'s second round:
`0.1.5-alpha.18`.
The ZFS rounds all add lines to `src/native/lib.rs`,
the list of published names,
`SPEC.md`,
the README and the reach control,
so each is checked on the tree that holds the ones merged before it,
and the management grammar's round merges before the removal's switch.

### How work is run side by side

The owner's "parallelize better" of 2026-11-27 ("Standing directions") was earned.
The coordinating session had held `main` still through every release,
run its own commands one after another,
and left the ZFS route's next step unstarted behind a sequencing of its own that had ended.
None of those three holds named a failure on record.
What runs side by side now,
and the few orders that stay,
each with the failure it prevents:

- Every ready item gets its round in the same turn,
  all in one start list.
  One list,
  because three lists side by side overlapped 27 worktree adds and sixteen of thirty starts failed (issue 6563).
  A further list starts once `ps -eo args=` shows no `git worktree add` in flight.
- A release does not hold `main`.
  `crate-release.yml` commits its bump through GitHub's API on whatever `main` is at that moment,
  and when `main` has moved it sends the commit again,
  unless a path the release depends on changed
  (the crate,
  its lock directories,
  and the path dependencies `cargo metadata` reports;
  the workflow's own first comment says so).
  That is read from the workflow's comment and not from its step,
  and `main` did not move during the one release dispatched since,
  `cargo-shadow` `0.1.5-alpha.17`:
  the first release during which `main` moves is its test,
  and that run's log is read before the rule is leaned on again.
  So merges of other packages and commits of records go on while a release runs,
  and only a merge into the released crate's own paths waits for the bump commit.
  Before each commit or merge on `main` the session fetches,
  and takes origin's new commits by fast-forward when it is only behind.
- A release's follow-up runs as one background chain up to the point where `main` must move:
  the dispatch,
  the wait for `PUBLISHED`,
  and the lock on a branch in a worktree of its own.
  The session merges and installs when the chain reports.
- Releases are dispatched one behind another.
  `crate-release.yml` and `cargo-publish.yml` each sit in a `concurrency` group with `cancel-in-progress: false`,
  and the session dispatches the next once the one before has started its publish;
  what GitHub does with a third dispatch while one waits was not read.
- Merges into `main` are made back to back,
  and origin is waited for once,
  after the last.
  Each merge is still checked by itself (`HEAD^2` is the branch's tip),
  because on 2026-11-25 a `git merge` returned 0 and landed nothing;
  and the one wait is for the commit service's push,
  which a `GH009` refusal does not repeat.
- Branches of one package that ended together share one integration worktree and one run of the package's checks,
  because three `mpm-shadow` branches merged without a textual conflict on 2026-11-27 and the merged tree did not compile.
- Reads,
  measurements,
  issue filing,
  briefs and record edits that do not move `main` go out in one message and run at once.
  Two commands that move `main` in the main checkout run one after the other:
  it has one `HEAD` and one index.
- Every branch and worktree the coordinating session makes itself is made detached or with `--no-track`,
  a lock branch and an integration worktree alike,
  and `git rev-parse --abbrev-ref @{upstream}` has to fail there before the first commit.
  A branch that tracks `origin/main` has its commits pushed to `main` by the commit service,
  so an integration worktree made that way would put unchecked merges on `main`.
  The lock branch of `cargo-shadow` `0.1.5-alpha.17` was made with `git worktree add -b <branch> <path> origin/main`,
  which set its upstream to `origin/main`,
  and its commit (`7f21ae4ef9`) was on origin's `main` at 2026-11-27T20:30:27-05:00,
  the second the commit service's push of that branch ended:
  a fast-forward with no merge commit,
  which nobody chose.
  The commit holds only the lock row's version,
  address and checksum lines.

### Rounds that ended, and what each left

Every round runs on Pi's virtual writer,
pinned to no member
(`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`),
whose members the owner changed on 2026-11-27 at about 16:15 -05:00:
"Change writer's list to match the reviewer's list."
Three picks inside the router the owner chose for the writer's list
wait in `doc/handover/questions-for-the-owner.md` with the veto open.
Three things the round on the driver's watch cases chose
wait for the owner in `doc/handover/questions-for-the-owner.md`.
The rounds run
under the owner's "Up to 64 rounds at once."
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
All were started through the driver package,
with the brief at `~/temp/agent/pi-runs/<round>-prompt.md`,
the run under `~/temp/agent/pi-runs/<round>-<n>`,
the unit `pi-<round>`,
the worktree `.claude/worktrees/pi-<round>`
and the whole report at `~/temp/agent/<round>-scratch/keep/report.md`:

- the guest drivers.
  Four points issue 6639's round left wait in `doc/handover/questions-for-the-owner.md`.
  Issue 6638's is merged (`1471bdf6ad`) and the issue is closed.
  Measured in the recipe's own guest before any edit,
  a second `prepare` alone exits 0;
  what fails is a `prepare` while a dataset is mounted,
  because a mounted dataset is what holds the module
  (`/sys/module/zfs/refcnt` at 1,
  where a pool imported with nothing mounted leaves it at 0 and the removal succeeds).
  One step,
  `e2e/load-the-built-module-step.ts`,
  now stands at the four places that removed and inserted modules:
  it reads which module is loaded against the version the installed file states,
  leaves a guest that already runs the built module alone,
  and refuses in words when a module of another version is held.
  A `prepare` with a pool imported and a dataset mounted exited 0 and left the pool,
  the version and the count as they were,
  and one driver's whole `all` step passed in a fresh guest.
  A host check,
  `test:tooling`,
  runs the step's shell against stand-in trees.
  Left as it was:
  the template's own prepare,
  `prepareFromTemplate`,
  which still loads with `modprobe --quiet zfs 2>/dev/null || true`;
  that was issue 6644.
  Its round merged as `be2557ad76` and the issue is closed:
  that prepare now runs a step of its own,
  which reads what is loaded and what the installed files state before it loads anything,
  and keeps the words and the status of `modprobe`
  (`package/zfs/request-to-openzfs-kernel-device/doc/prepare-in-a-prepared-guest-2026-11-27.md`).
  The round of issue 6615 is live ("Rounds running");
- the linter,
  whose round on what the read of `0.5.2-alpha.22` found,
  `monochromatic-lint-alpha-22-release-pass-a-list-lines-sentence-is-true-for-the-name-it-wrote-and-its-controls-can-fail`,
  ended 2026-11-27T19:09:36-05:00 with its report read whole.
  Its branch is merged (`f7b79c5aa8`),
  after the wrapper's bump commit,
  because the wrapper names the linter by path.
  The release run of `0.5.2-alpha.23` was started at 2026-11-27T19:21:22-05:00
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46087442769>);
  its release commit is `9fc0e387c3`,
  which changes version rows in eight files,
  the wrapper's and its fuzz crate's among them,
  and `main` is fast-forwarded to it.
  It is published
  (publish run 46087537570;
  the follower's log holds `PUBLISHED monochromatic-lint-v0.5.2-alpha.23`),
  locked (`6d99c6a2da`) and installed here,
  where `.cache/toolchain-bin/monochromatic-lint --version` says `0.5.2-alpha.23`;
  the install named no unresolved bin,
  and the session's `git` link was put back at 2026-11-27T19:39:58-05:00.
  Its Claude Sonnet second pass,
  a same-vendor read,
  returned and was read whole
  ("What the linter's alpha.23 read found").
  It took Lows 5,
  7,
  8,
  9,
  1 and 2,
  the absolute-path half of Low 4 and the dead-line help of Low 6.
  Measured by the round with the read's own generators,
  against the released `0.5.2-alpha.22` and a byte-identical copy of it:
  of 10,081 lists none moved between read and refused,
  and none changed its exit status or its set of findings;
  of the read's 5,600 matrix lines,
  the 600 written for an arrow-named folder whose own arrow a tab,
  a no-break space or two spaces sets off now name that folder,
  where none did,
  and the 200 for a folder whose name begins with a space all name it,
  where 72 did.
  The package's tests ran 1172 and passed 1172,
  and its binary reports the same 19 findings on the wrapper's package as the locked release.
  The choices its brief did not decide wait in `doc/handover/questions-for-the-owner.md`.
  The round for the Claude Sonnet read of `0.5.2-alpha.21` is merged (`83d10dd296`),
  and `0.5.2-alpha.22` is released
  (release commit `f10132d60f`,
  published by the follower's log,
  whose run ended 2026-11-27T17:21:26-05:00),
  locked (`0b90e1d87e`) and installed,
  with the session's `git` link put back at 2026-11-27T17:23:24-05:00.
  Its Claude Sonnet read,
  a same-vendor second pass,
  started 2026-11-27T17:21:51-05:00 and wrote its review at 2026-11-27T18:12:31-05:00:
  `~/temp/agent/monochromatic-lint-alpha-22-release-review-2026-11-27/keep/review.md`.
  A `file-uses` line of six words now names the folder its words name for every spelling of its gaps:
  1,152 of the read's 1,296 lines,
  against 324 in alpha.21,
  and the other 144 name a folder that holds no whitespace and read as an entry.
  A five-word line with another word in the arrow's place is told which word belongs there.
  Every character that prints as nothing is spelled in a message:
  4,266 of 4,266 candidates of Unicode 17.0.0,
  against 7 in alpha.21,
  decided by general category and by ranges of `Default_Ignorable_Code_Point`,
  with a control that fails when the linked tables move to another release.
  A character's place is a column of the line.
  Over 6,293 lists no line moved between read and refused,
  with a byte-identical copy of alpha.21 as the control.
  Its whole suite passed 1,158 of 1,158 after its last merge of `main`,
  and its four picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The round for the `0.5.2-alpha.20` read is merged (`2261d05b14`),
  and `0.5.2-alpha.21` is released
  (release commit `117514dfd7`,
  locked `6e3314812e`,
  installed).
  What that merged round did,
  as its report gave it and before the alpha.21 read qualified the first point:
  A `file-uses` line for a folder named with a tab,
  a no-break space or two spaces names the folder again;
  each of the five behaviours the read named has a test shown red by the edit it guards;
  a byte order mark or a zero-width space is spelled,
  with its place and the edit that takes it out;
  the dead-line help names the one edit;
  each shape of arrow name gets a fix that fits it;
  a mistyped `file-uses` line says what is wrong with it;
  and three README sentences say what the releases did.
  The reader does not take a byte order mark off by itself,
  which the README's known limits record;
  `test:layout` passed 326 of 326;
- rounds for the wrapper's `0.6.1-alpha.50` reads,
  started 2026-11-27T16:01:42-05:00,
  each named `wrapper-alpha-50-release-pass-` and then:
  `a-bare-shell-alias-is-judged-where-its-own-options-point`
  (with issue 6624),
  `a-hunk-a-side-also-wrote-into-keeps-no-line-a-side-removed`,
  `two-unwired-sentences-show-a-name-safely-and-three-records-say-what-holds`,
  `the-surface-control-reads-associated-constants-and-long-headers`,
  and `each-list-guard-says-what-it-matches-and-an-excuse-names-its-use`.
  All change `package/git-policy/cli`,
  each brief names the others' files,
  and the published-surface list is where their merges meet.
  After their merges come the tip checks,
  one whole gate,
  and `0.6.1-alpha.51`.
  The round on the list guards is merged (`eaef6b630c`),
  after two checks of the merged tree in a worktree:
  the linter's release commit landed between the first check and the merge and changed four of the package's files,
  so the tree was checked again with it,
  and format,
  check,
  Clippy,
  the whole `source_guard` module (225 of 225) and the five other modules whose guards changed passed both times.
  All five rounds for the reads are merged,
  the round on a bare shell alias last (`461d441ef7`).
  A guarded command behind a bare `!git` alias is judged for the worktree its own words name,
  whether they say `-C`,
  `--git-dir` or `--work-tree`:
  before,
  a reset of a dirty main checkout typed from a clean linked worktree ran,
  and the reverse was refused.
  One control compares the alias spelling with the direct one for 17 guarded commands,
  in both directions and three spellings of the target.
  The slot refusals read the records of the repository those words name.
  A removal,
  a move or a prune behind a `!` alias whose shell directory Git could not be asked for is refused,
  with words saying what could not be read,
  where it was forwarded with no refusal;
  a prune only when its own words name a repository.
  Every refused byte of the alias reader has a row,
  each shown red by widening the reader with it,
  and `SPEC.md` and the README state the bytes the reader trusts.
  Every slot refusal keeps the lines the policy stage printed before it,
  which closed issue 6624.
  That round ran the modules it changed and the whole `source_guard` module,
  and no `binary::` module and no gate.
  It put two more files of `invocation/` over the line budget,
  `entry_forward_action_planning.rs` and `global_arguments.rs`,
  and added one `group-file` line to the folder rule's allow list for a file it made;
  nine files of that folder are over the budget now,
  and their splits run after the crate's move
  (`doc/planning/cli-git-source-layout-target-tree.md`,
  the note of 2026-11-26T20:03:52-05:00).
  Its three picks wait in `doc/handover/questions-for-the-owner.md`.
  The tip checks passed on `beb61ec46b`,
  from 2026-11-27T18:40:16-05:00 to 2026-11-27T18:46:23-05:00 in a worktree no merge moved:
  `native:check`,
  the four Clippy tasks,
  the `source_guard` module (225 of 225),
  `native:ts-checks` and `lint:oxlint` with no warning and no error.
  The whole gate passed on `main` at `beb61ec46b`,
  started 2026-11-27T18:39:13-05:00 and read 2026-11-27T19:05:33-05:00:
  `12307 tests run: 12307 passed (7 slow), 51 skipped`,
  `gate-exit-status=0`,
  and no failed,
  flaky or signalled line in its log
  (unit `gpc-gate-wrapper-next-alpha-on-beb61ec46b-20261127T183913-0500`,
  log `~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-beb61ec46b-20261127T183913-0500.log`,
  the tip checks' logs in `~/temp/agent/wrapper-alpha-51-gate-and-tip-checks-2026-11-27/keep/`).
  Measured 2026-11-27T19:10:12-05:00 against `main` at `af1647ef36`:
  no path changed since the gated commit under the wrapper's package,
  under the sources of the crates it names by path
  (the ZFS request crate,
  `jsonc-edit`,
  `catalog-specifier`,
  the scanner's `src/` and the linter),
  under `package/crate-release` or in the release workflow,
  so the tree a dispatch builds is the gated one.
  The release run of `0.6.1-alpha.51` was started at 2026-11-27T19:16:27-05:00 from `e4c3117285`
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46087009444>);
  its release commit is `81b06993bd`,
  which changes one version row in each of four files,
  the crate's and the fuzz crate's `Cargo.toml` and `Cargo.lock`,
  and nothing else,
  and `main` is fast-forwarded to it.
  It is published for both targets
  (publish run 46087105171;
  the follower's log holds `PUBLISHED monochromatic-git-policy-cli-v0.6.1-alpha.51`),
  and unpacked at `.cache/wrapper-release-0.6.1-alpha.51/`,
  where the binary's version line says `0.6.1-alpha.51`.
  Its install proof over alpha.50 passed in a disposable container at 2026-11-27T19:27:02-05:00
  (`THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once`;
  log `~/temp/agent/wrapper-alpha-51-install-proof-over-alpha-50-2026-11-27/keep/install-proof.log`).
  Two Claude Sonnet second passes of it,
  same-vendor reads and not independent reviews,
  one on the merges that change what a person meets and one on those that claim to change nothing,
  returned and were read whole
  ("What the wrapper's alpha.51 reads found").
  The round of issue 6625 is merged (`3caae5b48d`) and the issue is closed.
  It changed no release code:
  measured with Git 2.56.1 at a repository's top,
  in a folder below it,
  in its Git directory,
  in a folder inside that,
  in a bare repository,
  in a folder inside it and at a linked worktree's top,
  the directory the shell of a `!` alias prints
  and the directory `alias_run_directory` answers are the same everywhere.
  Its controls hold the bound-slot refusal for a removal,
  a move and a prune,
  typed directly and behind a shell alias,
  in a Git directory and in a bare repository,
  each removal and move compared word for word with the spelling typed at the top;
  three plants turned them red.
  `SPEC.md` says what a bare repository's records are.
  One shape is left to Git as before:
  a slot command typed with `-C` at a directory that resolves to no repository.
  The next gate on `main` is the first to run these controls in the container.
  Nine planted spellings passed their guard on `main`:
  a renamed import,
  a function taken as a value,
  a call with its types written out,
  another crate's name for the same call,
  a handle reached through a module,
  a macro defined outside the guard's scope,
  and a key passed through a constant.
  The call reader reads each now but the macro,
  which every guard's top doc names as unseen,
  beside a glob import and a line break between a name and its parenthesis.
  An excuse names the line of the one use it covers,
  where it named a count,
  so a use removed and another added fails twice;
  and the two exemptions that named no file are entries now.
  Its question about the line-break rule is answered by "The Markdown the line-break sweep left out":
  the package's findings were that sweep's,
  and the linter's own fix took them out after the alias round's merge (`beb61ec46b`).
  It reported that the rule broke an inline code span;
  a probe of `0.5.2-alpha.21` on a Markdown file found no finding for a comma inside one,
  beside two findings for a plain sentence on the next line,
  so the round's instance is not reproduced there.
  The round on a hunk a side also wrote into is merged (`85be73acf1`).
  An answer of the word merge is refused when it holds more copies of a line the base held than the two sides leave,
  for every shape of region,
  where the earlier rule judged only a region both sides deleted from;
  and a file a rule refuses keeps both sides of every region the sides disagree about.
  Over the read's own 2,300 seed cases,
  run through this crate's merge entry,
  21 of 427 resolved answers broke that bound on `main` and none does now;
  those 21 are conflicts now,
  and no answer moved the other way.
  Where both sides wrote more copies of one line than the base held,
  nothing judges the answer,
  as before.
  Its merge time against `main` sat inside the spread of two byte-identical copies.
  Its planning record's heading carries a round time,
  which the coordinator corrects from the commit's own.
  The round on the surface control is merged (`35fdc1797c`),
  after a check of the merged tree in a worktree,
  because two merges had changed the package after the branch last took `main` in:
  format,
  check and Clippy were clean,
  the whole `source_guard` module passed 219 of 219,
  and the modules the two merges and the narrowing touched passed.
  Planted shapes showed what the control missed on `main`:
  a constant of a published type,
  a member whose body opens on a later line,
  a gate that names the test inside a negation,
  a re-export under another name,
  a bare re-export read from the wrong module,
  and two functions qualified twice that collapsed into one wrongly keyed entry.
  The reader sees each now,
  with a control per shape that failed first.
  Ten names it newly saw are narrowed,
  because nothing outside their module reads them;
  one alias with no reader is deleted;
  and one line is added for an alias the binary's integration target names.
  Both rounds' picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The round on the two unwired sentences is merged (`e772d8ad66`).
  Both sentences show the name through `shown_so_it_cannot_rewrite_the_line` and stand in the guard's list;
  `CommandWord::of` keeps the kernel's bytes and escapes nothing,
  so a name is escaped once,
  at the sentence.
  A sentence that shows a name through a local of another spelling is not seen by the guard:
  the round planted one,
  measured two wider scans over the release sources and found each reports the program's own words,
  and the guard's top doc now says what a reader checks.
  `SPEC.md` says which lines the rule covers and what a record written as JSON does with a name.
  No process that serves more than one command reads the identity file
  (`main.rs` chooses the service and the handler's modes before the entry runs;
  one run of a serving process said the sentence zero times,
  with no commit through it,
  because a fixture has no manager),
  and the README says the sentence is said once for a command.
  The README's bullet on other targets says one thing,
  resting on the sources' imports and on one dated check that cannot be repeated on this machine;
  a clause that called two dependencies unix-only was false and is gone.
  The whole `source_guard` module passed 213 of 213 after its last merge of `main`,
  and its picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  Of the `0.6.1-alpha.49` behaviour pass,
  all four rounds are merged,
  and the whole gate on `main` at `3aab810dd9` passed
  (unit `gpc-gate-wrapper-next-alpha-on-3aab810dd9-20261127T142758-0500`,
  started 2026-11-27T14:27:58-05:00,
  `gate-exit-status=0`):
  `Summary [ 883.520s] 12261 tests run: 12261 passed (6 slow), 51 skipped`
  and "Clippy passed beside the test run.".
  No file of the wrapper package changed between that commit and the release.
  `0.6.1-alpha.50` is released
  (release commit `3458afd18e`,
  published 2026-11-27T14:58:49-05:00,
  unpacked at `.cache/wrapper-release-0.6.1-alpha.50/`,
  never locked or installed).
  Its install proof over alpha.49 passed in a disposable container at 2026-11-27T15:01:26-05:00,
  on the gate's own image:
  "starts that failed at the EXEC step:
  0",
  no unit at the start limit,
  none left failed,
  and "THE PROOF PASSED:
  the instance the older release loaded started as this release,
  and the pair of commits each landed once".
  Two Claude Sonnet reads of it run,
  same-vendor second passes:
  of the four merges that change behaviour since 2026-11-27T15:02:11-05:00,
  writing `~/temp/agent/git-policy-cli-alpha-50-release-review-behaviour-2026-11-27/keep/review.md`,
  and of the two merges on guards and reach since 2026-11-27T15:02:41-05:00,
  writing `~/temp/agent/git-policy-cli-alpha-50-release-review-reach-and-guards-2026-11-27/keep/review.md`.
  A merge that lands after that commit and changes only the package's TypeScript,
- the linter:
  The choices the alpha.22 fix round's brief did not decide
  wait in `doc/handover/questions-for-the-owner.md`.
  Four picks of the linter round on `file-uses` lines and characters a reader cannot see
  wait in `doc/handover/questions-for-the-owner.md` with the veto open.
- rounds for the wrapper's `0.6.1-alpha.50` reads:
  Three picks of the round on a shell alias wait in `doc/handover/questions-for-the-owner.md`.
  The next gate on `main` is the first to run issue 6625's controls in the container.
  The hunk round's and the surface-control round's picks
  wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The two-unwired-sentences round's picks
  wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  A merge that lands after the gated commit and changes only the package's TypeScript,
  its guide or test code leaves the gate standing,
  as one did for alpha.49 ("Releases of 2026-11-27"),
  with the tip checks run again on the pinned tree;
  issue 6612's branch changes only `bin/` and `e2e/`.
- rounds for `cargo-shadow`:
  The ten failures of 2026-11-27 came after the reexec had ended,
  no measurement reproduced them,
  and their words were not kept:
  issue 6628 holds that,
  with nothing to build until a red run's output is kept whole.
  One residue of the asking again:
  a host whose manager died and left its socket file or its directory behind is still asked five times.
  The round's commit closed issue 6629 by its `Closes` line at the merge,
  and the coordinator reopened the issue for that residue.
  What made an ask fail on 2026-11-27 is not measured (issue 6623).
  The `populated` read's picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  A control shows that a scope whose directory was never made is read as collected,
  with its unit left loaded in silence;
  the fix belongs to the retire gate and is not made
  (`doc/handover/questions-for-the-owner.md`).
  A `populated` read that answers "No such file" while the group's directory is there is no longer read as the manager's collection,
  which is a changed decision with the owner's veto open
  (`doc/handover/questions-for-the-owner.md`).
  `doc/planning/cargo-shadow-a-scope-unit-a-killed-shadow-leaves.md` now holds a second road:
  the option that stops the unit a journal recorded cannot reach it,
  the two sweeps can,
  and the ranking stays as it was,
  with its first pair closer.
- a follow-up round for `mpm-shadow`:
  The pick is that the row declares the `rustfmt` component;
  it waits in `doc/handover/questions-for-the-owner.md` with the veto open (issue 6630).
  Two behaviours stay as they are,
  each held by a control and said in the README,
  and each waits for the owner:
  a move of `current` that fails once ends the run 1 and leaves the older version until `prune --toolchain`,
  and a recovery that puts an unfinished copy back points `current` at the newest finished version.
  What the sentences round chose beyond its brief waits in `doc/handover/questions-for-the-owner.md`.
  The choices of the round on the Rust row wait in `doc/handover/questions-for-the-owner.md` too.
  The follow-up round on the report's sentences left two wording choices for the owner,
  and they wait in `doc/handover/questions-for-the-owner.md`.
  A name of another boot is ended whatever its start time says,
  which is a changed decision with the owner's veto open
  (`doc/handover/questions-for-the-owner.md`).
  The round on a hidden name nobody can judge left proposals and a wording pick,
  which wait in `doc/handover/questions-for-the-owner.md`.
  The warning's wording for a lock that is satisfied already has no test:
  the recovery round could not build that state,
  and worded it to promise nothing about the copy.
  Where the words of two `mpm-shadow` rounds could not both hold,
  the round that joined them chose,
  and both choices wait in `doc/handover/questions-for-the-owner.md` with veto open:
  a finished copy of an older version stays hidden only when the lock vouches for it,
  and one no lock vouches for is removed;
  the copy a kill moved aside waits while a name of this boot whose installer cannot be judged may hold the version,
  and does not wait for a name in an earlier build's shape,
  which nothing ever removes.

A run of a built wrapper executable on the host still needs a fixture runtime directory and no bus address
(a release binary in a pass,
a binary-target test):
`env --unset=DBUS_SESSION_BUS_ADDRESS XDG_RUNTIME_DIR=<a fixture folder>`.

The reads of the wrapper's `0.6.1-alpha.51`,
of the linter's `0.5.2-alpha.23` and of `mpm-shadow`'s `0.2.7-alpha.18` returned and were read whole
("What the wrapper's alpha.51 reads found",
"What the linter's alpha.23 read found",
"What `mpm-shadow`'s alpha.18 read found").
Each was a same-vendor second pass and not an independent review.
The coordinator rewrites what lasts from each into this file before scratch expires.

The alpha.50 behaviour read's merge finding is taken as a fix,
by the owner's request of 2026-11-10 that wrong text land less often,
and its round returns the pick and the other way for the owner.

Each point of the `mpm-shadow` alpha.15 read is closed,
or open on purpose with its entry in `doc/handover/questions-for-the-owner.md`:
one decision no claim named,
a recovery that puts an unfinished copy back now also points `current` at the newest finished version,
which the round holds with a control and writes into the README,
and which waits for the owner in `doc/handover/questions-for-the-owner.md`.

The rules on which names are judged,
held or removed stay as the round that joined two `mpm-shadow` rounds picked them,
with the owner's veto open and the read's costs written beside them
(`doc/handover/questions-for-the-owner.md`).

The `cargo-shadow` alpha.16 read's picks,
the coordinator's own with the veto open:
the read counts the transports the ask can take,
shown from systemd's source at both versions;
an ask the manager answered with a refusal ends the asking,
if a real reexec shows no such answer inside its window;
the records name every state in which the span is spent;
and each of the three edits gets a control shown red by it.

A state outside the `cargo-shadow` alpha.14 read's merges stays open:
a scope that does not exist yet may be read as collected and its unit left loaded in silence.

The coordinator's weighing:
all seven are taken,
by the two rounds the `mpm-shadow` alpha.17 release's passage names;
their picks are the coordinator's with the veto open,
and are written into `doc/handover/questions-for-the-owner.md` when the rounds' reports are read.

The raw characters in the linter's other messages wait for the owner,
because a command a person copies must hold the name raw
(`doc/handover/questions-for-the-owner.md`).

The control `the_home_says_whether_it_holds_a_record` holds the rule for an empty folder,
an unrelated file,
a record and a record being written;
the two reads of `0.6.1-alpha.49` say a folder that holds only an add's claim also counts as holding no record,
and no control shows that case.

The shell-alias round adds no refusal for an alias the wrapper cannot follow,
which is the open choice of `doc/planning/cli-git-slot-refusals-and-shell-aliases-they-cannot-read.md`.

Tip checks run in a worktree no merge moves:
on 2026-11-27 three Clippy arms run in the main checkout exited 0 while Rust merges landed under them,
and stood for no commit.

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

`main`'s tip is checked after a batch of merges and not by each merge:
the merge routine runs `git merge-tree` and compiles nothing.
That check never ran the package's TypeScript side,
and it does from now on,
until the owner chooses where those checks run (issue 6584):
`mise run //package/git-policy/cli:native:ts-checks` runs them each in a capped scope,
and `lint:oxlint` is run beside it.
A tip check that a merge lands under is void and is run again.

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

#### What the wrapper's alpha.51 reads found

Both are Claude Sonnet's,
same-vendor second passes and not independent reviews
(`~/temp/agent/git-policy-cli-alpha-51-release-review-behaviour-2026-11-27/keep/review.md`
and `~/temp/agent/git-policy-cli-alpha-51-release-review-no-change-merges-2026-11-27/keep/review.md`,
scratch).
The behaviour read ran alpha.50,
alpha.51 and a byte-identical copy of alpha.51 beside real Git 2.56.1,
and found two High and six Low points;
the no-change read found seven Low points.
No session on this machine runs alpha.51:
the session wrapper is the build of `main` at `ee0cf2268b` of 2026-11-22 and the lock row is alpha.44,
both older than the merge that brought the two High points.

- High,
  both from the merge for issue 6624 (`461d441ef7`),
  filed as issue 6641 with the reproductions.
  With `alias.g = !git`,
  options typed before the alias name are dropped once the words after it name a place:
  `git --git-dir=<main>/.git --work-tree=<main> g -C <wt1> reset --hard`,
  typed in another linked worktree,
  is refused by alpha.50 and run by alpha.51,
  and the main checkout loses its unstaged edit and the staged new file is gone from disk.
  And from a linked worktree Git hands the alias's shell the caller's `GIT_DIR`,
  so a nested `-C` moves the files and not the repository:
  `git g -C <wt1> reset --hard` typed in a dirty worktree overwrites an untracked file of `wt1` with the caller's committed text,
  where alpha.50 refused because it judged the dirty caller.
  Of 110 rows,
  22 moved from run to refused as the merge meant,
  7 false refusals went,
  and 12 moved from refused to run with a change of state.
  The merge's one behaviour control compares verdicts through `plan_invocation`,
  which never starts Git.
- The merge pass's copies bound (`85be73acf1`) does what it says:
  in 1,672 merges four moved from resolved to conflict,
  none the other way,
  and no resolved byte differs.
- The `final-newline` hint (`0f69dbfe93`) was shown at an add,
  a direct check and a manual push.
  At an add refused at `error` it promises a commit that cannot take the file,
  and at a push it still says to stage a file whose commit is made
  (Low 3 and Low 4).
- The two sentences of `e772d8ad66` stand in a module nothing in the release calls;
  neither binary holds their text.
- The no-change read:
  of the 6,789 Rust lines `beb61ec46b` changed every one is a doc comment,
  the code is byte-identical in 471 of 471 files,
  and every changed Markdown file renders to the same HTML;
  the ten names `35fdc1797c` narrowed are used nowhere outside their new reach;
  no changed linter code is in the built wrapper;
  and the released binaries differ in nine of 53 compared commands,
  each by the version or by the `final-newline` hint.
- Weighed by the coordinator.
  Five rounds take what earned one ("Rounds running").
  Dismissed:
  the behaviour read's Low 2,
  the alias spellings `SPEC.md` names as the guard's boundary,
  documented and unchanged;
  and the no-change read's Low 2,
  that the internal-error notice prints line numbers every source edit moves.
  Its Low 7,
  the lone words the linter's fix left in doc comments,
  is the linter's and is issue 6642.
- The first of the five rounds is merged (`570402952b`):
  the three lint findings in `check_outcome.rs` are gone,
  cleared by what each rule asks for and by no suppression,
  and the package's findings on `main` are the thirteen files over the line budget and nothing else.
  The crate's one doc test compiled and passed when run alone;
  its assertion is a unit test the gate runs now,
  shown red by a plant,
  and the guide says a doc fence holds TypeScript pseudocode because no task runs doc tests.
  The round found that `src/native/policy/policy_forbidden_strings.rs` opens a doc fence at line 23 that no line closes,
  which no rule of the linter reports;
  it is noted on issue 6642 for the round that puts the wrapper's doc comments right.
- The `final-newline` round is merged (`dce0e58a56`).
  At an add,
  at `warn` and at `error` alike,
  the hint leads with `fix -- <path>` and then `git add -- <path>`,
  and says a commit corrects the file only when it takes the whole file;
  a landed commit and a manual push say to edit the file and commit the correction;
  the direct check's hint is unchanged.
  Its controls were shown red by two plants,
  and the read-only one derives its answer from `decide_selection`.
  Its open points wait in `doc/handover/questions-for-the-owner.md`.
- The round on the two failures and three records is merged (`efae1c2767`),
  after its branch passed format,
  check,
  clippy and its modules on the tree that held the `final-newline` merge,
  and check and the `source_guard` module again on the tree that held the surface control's.
  It changes no release code.
  Two controls hold the two failures the policy stage's lines lead,
  each handing `forward_action` the text a real stage rendered,
  because no state reaches either branch through `plan_invocation` with a finding printed;
  a third reaches the margin of the copies bound,
  red when the bound reads the smaller count.
  The README states the copies bound for one region and not for the file,
  and `SPEC.md` says a plain `git worktree prune` behind an unplaced alias is not refused.
- The round on the surface control and three guards is merged (`ee6dc2cd2f`),
  and changes no release code.
  The reader of module lines refuses a `mod` statement at a file's first column that it cannot place,
  where it used to step over one;
  the surface reader reads the public members of a type a `pub use` carries,
  through the root's globs,
  so the list of published names gained fifteen lines,
  `GuardedCommand::ALL` among them,
  and holds 12,415;
  and three guards read spellings they could not:
  a direct `libc::syscall` in the commit path's files,
  a `Command::new` of any literal that ends in `systemctl`,
  and a write to `libc::STDERR_FILENO` in the note lane.
  Each new control was red first,
  and the `source_guard` module went from 225 tests to 231.
  A name behind a feature gate still counts as published,
  and the list's header names the three.
  What it chose waits in `doc/handover/questions-for-the-owner.md`.
- Issue 6641's round is merged (`229d5310eb`) and the issue is closed.
  A bare `!git` alias is judged in the one state its nested Git acts in:
  the directory Git runs the shell in,
  the `GIT_DIR` Git gives that shell,
  asked of Git itself,
  the options the caller typed before the alias name without their `-C` chain,
  and then the alias's words.
  All twelve rows the issue names reproduced before any change and now stop;
  of 111 rows seventeen moved against alpha.51,
  none to a run that changes state,
  and the build runs nothing that changes state which alpha.50 or alpha.51 refused.
  Two controls drive the built executable with real Git behind it,
  two go through `plan_invocation`,
  one holds the reading of the exported `GIT_DIR` in seven shapes,
  and one holds the slot refusals behind the same spellings;
  each was red under one of two plants.
  The round judged the one state,
  where its brief had read `SPEC.md` as both places measured:
  three commands that change nothing in real Git are refused for it,
  and the refusal does not say that files and index come from two places.
  Both wait in `doc/handover/questions-for-the-owner.md`.
- All five rounds are merged.
  The gate on `main` at `229d5310eb`,
  started at 2026-11-27T22:06:58-05:00,
  ran no test:
  in its last step the compiler of the test build was ended by `(signal: 9, SIGKILL: kill)` while the machine was out of memory ("Memory"),
  and the unit failed at 22:48:48 with no compile error printed.
  That is a build the machine killed and no verdict on `main`.
  The gate runs again from a command of the coordinating session that waits until `MemAvailable` plus `SwapFree` reads 40,000,000 kB on three readings a minute apart,
  for six hours at most;
  a Claude Code restart stops that command,
  and the next session starts it again.
  The measure is the coordinating session's own,
  for this gate alone,
  with the owner's veto open (`doc/handover/questions-for-the-owner.md`).
  Nothing in a release's dispatch reads the container gate:
  the registry's dispatch gate is the btrfs controls,
  which an alpha skips.
  So `0.6.1-alpha.52` is cut from the first green gate whose commit still matches `main` under the package's paths at dispatch,
  and no merge waits for it.
  Parts A and C of this file's history changed three files the wrapper's tests read
  (three lines added to the README,
  a module doc comment,
  and the comment header of the list of published names)
  and no wrapper test ran on them before their merge,
  because the machine had no memory for the build:
  the next gate is their first run.

#### What the linter's alpha.23 read found

A Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/monochromatic-lint-alpha-23-release-review-2026-11-27/keep/review.md`,
scratch):
no High,
one Medium and eight Low points.
Over 16,081 lists and 48,243 runs no list moved between read and refused,
between exit statuses or in its set of findings against alpha.22,
with a comparison first shown able to see a move.

- Medium:
  a long run of whitespace after `file-uses` in a six-word line costs time and memory with the square of the run.
  At 64,000 characters alpha.23 took 3.3 s and 2.17 GiB where alpha.22 took 11 MiB,
  so the reader's four-mebibyte bound no longer bounds the cost
  (`readings_of_a_written_folder`,
  `layout_allow_entries.rs`).
- Low:
  five sentences that promise an edit the next run refuses or finds dead again;
  a README sentence the list reader contradicts;
  two points of the alpha.22 read neither fixed nor listed as left;
  and three controls that would stay green under a named edit.
- One round took all nine.
  The coordinator's pick,
  with the veto open,
  stays:
  the read,
  fix and release loop of this package ends at the first read that finds no High and no Medium.
- That round is merged (`de1bd36d08`).
  Each point was reproduced on the released binary first.
  The names a gap could have left are offered to the file system one at a time and none is kept,
  and no name is offered whose first folder is longer than the package folder's longest name.
  On a line of 64,033 characters the branch took 1.10 times alpha.22's wall time (range 0.73 to 1.21)
  and 0.97 times its peak memory,
  where alpha.23 took 981 times and 180 times,
  over three rounds of three arms in rotating order.
  Every sentence a dead line or a refusal prints is true for the edit it names,
  and the three controls the read named were each shown red.
  Over 16,081 lists and 48,243 runs no list moved between read and refused against alpha.23;
  2,370 differ in message text alone.
  U+2800 stays as written and the README says why:
  the set a message spells is Unicode's `Cf`,
  `Cc` and `Default_Ignorable_Code_Point`,
  and none of them names it.
  The README dates that measurement 2026-11-28,
  the day in UTC of evidence written at 2026-11-27T21:03:10-05:00;
  the merge of issue 6642's branch carries the correction.
  The round's first release build ran outside a capped scope,
  by its own report.
- Issue 6642's round is merged (`cdf69930fd`),
  after its branch passed the package's `lint`,
  `lint:rustfmt` and `test` tasks (1196 of 1196) on the tree that holds the other round,
  with the README's measurement day corrected to the host's own.
  The released fix replaced one space with a break and touched nothing else,
  so a word that ended the old line became a line of its own,
  written at the paragraph's opening depth.
  Now a fragment of one or two words that this pass's own break cut,
  and that ends at no break-point character,
  joins the line that continues its clause while the joined line stays under 120 characters;
  a line the fix writes takes the indent of the paragraph's first continuation line;
  and no line the author wrote is rewritten.
  Over the wrapper's source as it stood before the merge that brought the lone words,
  the released fix wrote 834 doc lines of one or two words and the branch 732,
  every one of which ends where the rule wants a line to end.
  The words of all 4,141 files are unchanged,
  by a comparison first shown to see one changed word.
  Six controls were each shown red.
  The wrapper's own doc comments are not repaired:
  that is a round of the wrapper's once a linter holding the fix is released.
  Its proposal,
  `doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
  measured 199,796 breaks at no boundary in paragraphs the rule reports nothing for,
  in 8,623 files,
  and ranks first the reporting of the 158,514 a removal fix can join;
  its three questions wait in `doc/handover/questions-for-the-owner.md`.
- `rust/avoidable-closure` runs on no host run here:
  its Dylint driver is absent and the linter does not download one,
  so every count of findings taken on this machine leaves that rule out.
  Started through `mise run //:lint:monochromatic-lint` it is skipped the same way,
  measured on `package/mpm-shadow/cli` at 2026-11-27T20:15:12-05:00.
  It is issue 6643,
  whose round built nothing and whose proposal is merged (`e63f6d9226`):
  `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`.
  Nobody in this repository provides the driver:
  the plan holds the nightly,
  `cargo-dylint` and `dylint-link`,
  and no step builds a driver;
  this machine has no `~/.dylint_drivers`,
  and its `cargo` on `PATH` is the repository's shadow and no rustup proxy.
  A skip is a warning of the rule's own code at the package's `Cargo.toml`,
  merged into the findings,
  so neither code nor position tells it from a finding;
  it exits 0 unless `--max-warnings` is passed.
  In a container the round built a driver on its fifth attempt
  (it needed `llvm-tools-preview` beside `rustc-dev`,
  and a `TMPDIR` the image's `git` could write),
  and the rule then found six findings in `package/wg/cli-quicker-exempt` in about two seconds.
  It ranks first that the toolchain install builds the driver,
  which waits on the owner's pause of that install (issue 6316),
  and says a code of its own for a skip is worth taking with whatever else is picked.
  The issue stays open,
  and the choice is the owner's (`doc/handover/questions-for-the-owner.md`).
- `0.5.2-alpha.24` is release commit `40710212ed`
  (dispatched 2026-11-27T21:42:43-05:00,
  release run 46099406752 and publish run 46099490612;
  the follower's log holds `PUBLISHED`),
  locked (`071793f045`) and installed here,
  where `.cache/toolchain-bin/monochromatic-lint --version` says `0.5.2-alpha.24`.
  The install named no unresolved bin and no removed link,
  the exposure folder held 174 names before and after,
  and the session's `git` link was put back at 2026-11-27T22:03:21-05:00.
  Its second pass runs ("Rounds running").
  The gate reviews of the two proposals are back ("What the gate reviews of the linter's two proposals found").
  The wrapper's own doc comments are repaired by a round of the wrapper's ("Rounds running").

#### What the gate reviews of the linter's two proposals found

Two Claude Sonnet reads,
same vendor and not independent,
each of a proposal on `main` before it is put to the owner.
Neither proposal is ready:
each read breaks the ranking its question would carry.
Each is revised by a round ("Rounds running"),
read again,
and only then asked.
The reads are scratch that expires a day after its last change,
`~/temp/agent/gate-review-of-the-line-break-proposal-2026-11-27/keep/review.md`
and `~/temp/agent/gate-review-of-the-dylint-driver-proposal-2026-11-27/keep/review.md`,
so what they found is written here.

The line-break proposal
(`doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
issue 6642):
five High,
eight Medium,
three Low.

- Its "no boundary" is a test on six punctuation characters.
  `AGENTS.md` rule MD1 names no set,
  rule MD8 permits a hard wrap between inline spans and is never cited,
  and `AGENTS.md`'s own breaks before `and` and before a parenthesis would be findings.
- The test does not climb emphasis delimiters,
  so a sentence that ends inside bold counts as no boundary where the rule itself breaks after it:
  the first option and the existing rule cannot both be satisfied.
- The third option's stated gain is false:
  the join only looks at what a break the same pass inserted cut off,
  so the issue's own lone `the` stays.
  The coordinating session saw both on the installed `0.5.2-alpha.24` at 2026-11-27T22:36:55-05:00:
  a paragraph `Aaa bbb, ccc ddd eee` / `the` / `fff ggg hhh.` keeps its lone word,
  and `worktree_clone_add_route/mod.rs` on `main` holds one with no finding.
- "Returns to green" has no baseline:
  `main` already fails on `rust/max-lines` findings no fix clears,
  and the wrapper's commit policy runs one Markdown rule.
  The options leave out a first release at warn severity,
  a list that only shrinks,
  and the rule's own history,
  the owner's "Put the semantic line break findings on hold."
  among it (`doc/planning/repository-linter-stays-at-zero.md`).
- Part of the first option's 41,282 unfixable breaks are ones MD1's own limit of 120 forces,
  and the option that reports a break only when its whole clause fits under 120 is missing.
- The counts do not add up:
  221,518 less 9,860 is 211,658,
  not 199,796.
  The removal model is not the shipped guards.
  "One `--fix` pass clears them" is not shown.
  The class the issue names,
  a short line that itself ends at no boundary,
  was never counted.
  The examples are the first files in path order,
  four of them cut short.
  The harness,
  its output and its file list live only in scratch.
- Of the ranking's two pairs,
  the first holds in direction and not in its stated reason,
  and the second does not hold as written.
- Three points are about shipped code and wait for the second pass of `0.5.2-alpha.24`,
  to be routed with it:
  the join's test has the same blind spot for a fragment that ends inside bold,
  the README's "The fix never rewrites a line the author wrote."
  is no longer true of a joined line,
  and its sentence on "the deepest indent of a rustdoc run" looks reversed against `run_margin`.

The Dylint driver proposal
(`doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`,
issue 6643):
four High,
six Medium,
six Low.
What a skipped rule returns holds against the source.

- Three toolchain names are in play on this machine and no option says which directory a provider fills:
  the floating default has no `rustc-dev` and no `llvm-tools`,
  the dated one has both,
  and Dylint drops `RUSTUP_TOOLCHAIN` and files the driver under what `rustup show active-toolchain` says.
  Whether the nightly is pinned is the owner's question,
  left open by `doc/decision/rust-closure-lint-dylint.md`.
- "No rustup-proxy `cargo` on `PATH`" is false,
  and one option's disqualification rests on it.
  Read by the coordinating session on 2026-11-27 between 22:38:12 and 22:43:10 -05:00:
  `type -a cargo` lists the repository's two and then `/usr/bin/cargo`,
  a link to `/usr/bin/rustup`.
  `doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md` said the same in two places and now carries a correction beside each.
- An exit status for any skip would be red with a perfect driver:
  of 39 lock directories nine are exempt by name,
  and six of the thirty left carry prerequisite manifests.
- CI cannot start the linter at all:
  the last log of that step that could be fetched,
  run 41274002049 of 2026-11-01,
  ends `monochromatic-lint: command not found`,
  and the workflow on `main` installs `file-enforcer mpm-shadow cargo-shadow` before it runs the lint task.
  Open issue 6267 already records it,
  so no issue was filed.
  Every provider option delivers nothing to a pull request until that is fixed.
- The clippy clone happens only for a dated toolchain name.
  The installer's environment file is written per worktree and CI's steps do not read it.
  A code of its own for a skip conflicts with the amendment of 2026-11-07 as much as an exit status does.
  A published driver carries an absolute path into its builder's toolchain.
  The record of 2026-10-30 already ranked an installer-owned warm below the linter-side probe.
  Six options were not weighed,
  the baseline among them:
  the owner warms the driver by hand.
- Of the ranking's six pairs two do not hold as reasoned,
  and a disclosure of a skip is a complement to any provider,
  not an alternative ranked below one.

#### What `mpm-shadow`'s alpha.18 read found

A Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/mpm-shadow-alpha-18-release-review-2026-11-27/keep/review.md`,
scratch),
with a real `rustup 1.29.3` in a container:
no High,
two Medium and seven Low points.
It demonstrated that the Rust row's programs come from the pinned toolchain
(`rustc` through the exposure folder prints the pin's `5f4effd57 2026-11-13` where alpha.17 printed an older toolchain's),
and that the removal deletes symbolic links no row declares and leaves files,
directories and hard links.

- M-1:
  the removal deletes another install's in-flight temporary link,
  and a second remover fails on a link the first already removed,
  so two installs at once can exit 1 where alpha.17's both passed;
  the folder is right afterwards and a rerun passes.
- M-2:
  the exposure folder's `mpm-shadow` link is deleted when the running binary is no installed copy,
  although an installed one exists and the link resolves.
  This repository's plan declares the row,
  so a repository scope keeps it:
  the three installs made in the main checkout since alpha.18 each named no removed link,
  with 174 names in the folder before and after.
- Low:
  the directory rule accepts the channel,
  a hyphen and any text that is no date,
  so `mine-two` answers for `mine` and a second host triple answers for the pin;
  a removal made before a later failure is never named;
  an ordinary failed install's message lost the true sentence that the next run publishes without downloading;
  "a later run removes it" does not hold across process namespaces;
  a stale doc comment;
  three controls that stay green under named edits;
  and `list` starts `rustup` three times with no bound on the wait.
- Three rounds take them,
  by area ("Rounds running").
- The round on the failed install's message is merged (`27b745b6dd`).
  All six messages of an ordinary install's failed publish,
  where the read counted five,
  end with one sentence:
  a later install of that version publishes the kept copy without downloading while the version's name holds no finished copy,
  and removes it as a second copy once the name holds one;
  an install in another process namespace leaves it;
  and `prune --toolchain` prints its name.
  Each message was built on alpha.17,
  on alpha.18 and on the branch,
  with the install that follows.
  The other namespace was a real one,
  a rootless container sharing one scratch folder:
  the install inside printed nothing and left the copy,
  `prune --toolchain` inside printed its `left` line,
  and an install outside then removed it.
  The recovery's flag that no test read is now a value of three kinds,
  held by controls for the two kinds a fixture can build.
  No control holds the message that needs two failed renames in one publish,
  nor either recovery arm inside a run's warning.
  The package's task passed 934 of 934 after the merge of `main`.
  What the round chose waits in `doc/handover/questions-for-the-owner.md`.
- The round on the Rust row is merged (`b16e1fdca2`).
  The three wrong directories the read named reproduced on this machine's own `rustup 1.29.3`,
  with two of six links,
  and then all of them,
  taken from a neighbour.
  The row's directory is now the parent of the path `rustup which --toolchain <channel> rustc` prints,
  one `rustup` start for each such row in an install,
  and the name rule is gone:
  rustup's own source shows the host triple cannot be worked out by another program.
  `list` reads one listing,
  the targets being its `rust-std-<triple>` components,
  so it starts `rustup` twice for such a row where alpha.18 started it three times.
  No deadline ships for `list`:
  the measurement says what an answer costs and not where a late one is cut off,
  and the README says so.
  A control holds the failing listing's line,
  red under the plant the read named.
  The package's task passed 938 of 938.
- The round on the exposure folder's removal ended 2026-11-27T21:30:28-05:00 with its report read whole,
  and its branch was joined with `main` by a round of its own,
  merged as `79c2c4abc7`:
  the Rust row's merge had changed `exposure.rs` under it,
  and the one conflict was resolved with both intents kept.
  The joined tree's package task passed 949 of 949.
  No control drives a postinstall step on a `rustup-toolchain` row,
  so the two call sites the join took over rest on the compiler;
  that gap is older than both rounds,
  and the second pass of `0.2.7-alpha.19` is asked to drive it.
  Both races of M-1,
  M-2 and the unnamed removal reproduced with the released binary.
  On the branch a temporary link's name carries the install that holds it,
  by the four facts the hidden names already carry,
  so a rebuild leaves the link of an install that runs or cannot be judged,
  and removes and names one whose install ended;
  the `mpm-shadow` link stays while it resolves into an installed copy,
  whatever binary runs the rebuild;
  a link another run took first is no failure;
  and a run that fails at a later removal still names the ones it made.
  Eleven controls,
  six plants,
  and the package's task 945 of 945 on its own tree.
  Its three questions wait in `doc/handover/questions-for-the-owner.md`.

`0.2.7-alpha.19` holds the three rounds.
Its release commit is `1cd66ac8df`
(run 46101889021,
dispatched at 2026-11-27T22:14:33-05:00),
made on a `main` that a records commit had moved eighteen seconds before;
how the run sent its commit on a moved `main` is not yet read from its log.
The lock is `ba118ea127`,
merged as `68bac882d7`.
`mise run toolchain:install` ran twice in the main checkout,
ending at 22:30:36 and 22:30:41,
each `0 installed, 67 unchanged, 0 failed` with no unresolved bin and no removed link,
the exposure folder held 174 names before and after each,
and the session's `git` link was put back at 22:30:41.
Its second pass runs ("Rounds running").
`git worktree remove` of the lock's worktree,
slow under load,
removed the worktree and then failed with "is not a working tree" (issue 6648);
nothing was lost.

#### What `cargo-shadow`'s alpha.16 round did, and its alpha.17

The round on the four Low points of the alpha.16 read is merged (`05e0758cb7`).
It reproduced each with the released binary in containers:
the asking again is spent in 11 of 24 states,
five asks and about 0.86 s a call,
each now named in the README with its cost.
An ask the manager answers with a refusal of its own ends the asking:
three asks and a median of 56.5 ms with `app.slice` masked in a container's own manager,
where alpha.16 made five asks and took 895 ms.
Because a failed start that names the unit can be a transport that died,
a refused start is followed by one more ask,
which ends the asking only if the manager answers it.
The read of the host counts the transports systemd takes at versions 255 and 262,
quoted from systemd's own source.
Ten controls were each shown red,
and 48 calls begun inside 48 real reexecs of a container's manager all took the delegated route.
The package's task passed 978 of 978 after the merge of `main`.
`0.1.5-alpha.17` is release commit `2ff68958b1`
(dispatched 2026-11-27T20:26:16-05:00 from `05e0758cb7`,
release run 46093112468 and publish run 46093197842;
the follower's log holds `PUBLISHED`),
locked (`7f21ae4ef9`,
which reached `main` without a merge commit:
"How work is run side by side")
and installed here,
where `.cache/toolchain-bin/cargo-shadow --version` says `0.1.5-alpha.17`;
the install named no unresolved bin and no removed link,
and the session's `git` link was put back at 2026-11-27T20:35:23-05:00.
Its second pass,
a Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/cargo-shadow-alpha-17-release-review-2026-11-27/keep/review.md`,
scratch),
was read whole.
It ran the two released binaries and a byte-identical copy of alpha.17 in containers whose own manager it could reexec,
mask and stop,
and found no High,
two Medium and three Low points.

- Medium 1:
  after a failed start,
  any answer to the next ask ends the asking,
  a path among them,
  which is then thrown away
  (`control_group_route_discovery.rs`,
  the arm that answers `may_be_put_again: false` for an answered confirm ask).
  A unit load that fails once,
  a start whose reply is lost and a start refused once each took the weaker path in every call of alpha.17,
  where alpha.16 delegated in every call,
  each state built with a stand-in in the ask path.
  With a real reexec aligned to the start,
  2 of 24 calls of the alpha.17 binaries took the weaker path and none of alpha.16's 12;
  with the reexec fired from outside,
  none of 12.
  The round's own calls in real reexecs never sent a start from the branch binary,
  because the recipe's layout keeps `app.slice` active,
  so they did not exercise this arm.
- Medium 2:
  with a stop job queued on `app.slice`,
  the first ask answers the path,
  no start waits behind the stop,
  and the scope's own start is refused as destructive:
  10 of 10 calls of the alpha.17 binaries failed (exit 1,
  or 101 in shadow mode),
  where alpha.16 waited about 3.7 s and ran in 7 of 7.
- Low:
  three edits no control would catch
  (the arm after a failed confirm ask and the path it discards,
  the sleep at the call site,
  an address beside a runtime directory);
  five record sentences that are wrong or imprecise;
  and a reexec that begins after the discovery fails the whole call in both releases,
  which no record says.
- Borne out:
  one `systemctl` program a call where alpha.16 started two;
  three asks where a masked slice took five and an answer that is no path took ten;
  the span spent in the eleven named states;
  and of 240 compared files 21 differ,
  each by the version or by the missing `start` row.
- `0.1.5-alpha.17` is the copy installed on this machine,
  so a build begun here while `app.slice` has a stop job queued fails until the next release.
- Weighed by the coordinator:
  every point earns a fix,
  in two rounds by area ("Rounds running"),
  one on the discovery's two arms with the control that holds them,
  one on the records and the two other controls.
  Issue 6645 states the two Medium points.
What the round chose that its brief did not decide waits in `doc/handover/questions-for-the-owner.md`.

Of the two rounds on the read of `0.1.5-alpha.17`,
the one on its Low points is merged (`bf4457e7bf`):
two new controls,
each shown red by the edit the read names,
five record sentences corrected against their sources,
and the package's task at 980 of 980 on its merged tree once the machine had memory again
(two runs inside the out-of-memory stretch failed one and two controls it does not touch,
and each passed alone).
It wrote the read's third Low point into the README and built nothing:
a reexec of the manager that begins after the route is discovered fails the whole call,
and whether a scope's creation is then tried again is the owner's choice
(`doc/handover/questions-for-the-owner.md`).
The round on the two Medium points runs ("Rounds running");
its branch is joined with `main` in one tree,
because both rounds change the same two records.

### Releases of 2026-11-27

Alphas of the wrapper and of the tools on its path are published unasked
(`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
"How far the permission to publish reaches").
A release is dispatched when `main` equals `origin/main`,
which the dispatch task checks itself,
and `main` may move while it runs
("How work is run side by side").
After every `mise run toolchain:install` in the main checkout the session's `git` link points at the toolchain launcher,
and it is put back on the build of 2026-11-22
(last at 2026-11-27T20:35:23-05:00).

Before a release note is written,
`git merge-base --is-ancestor <merge> <the last release commit>` says what the release before already held.

In `mpm-shadow`,
the pick not to repeat a start refused as busy is before the owner again
(`doc/handover/questions-for-the-owner.md`).

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

On `main` at `6037c4e0f8`,
measured with the installed linter at 2026-11-27T20:12 -05:00,
the wrapper has no layout finding and 13 files over the budget,
down from 264 before the splits:
`foundation/lines_written_by_a_thread.rs`,
`health/doctor.rs`,
`health/doctor_filesystem.rs`,
`worktree/mod.rs`,
and nine files of `invocation/`.

- `foundation/lines_written_by_a_thread.rs` waits for the owner,
  with its loop.
- The rest wait for the move of group-folder files into child folders,
  round 7 of the plan's order,
  which follows rounds 4 to 6:
  the files of `invocation/`,
  `health/doctor.rs` and `health/doctor_filesystem.rs`,
  where a new sibling file needs a `group-file` line,
  and `worktree/mod.rs` (339 lines of module declarations).

A brief names a file over the budget only from the linter's listing of `main` taken when the brief is written.

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

- The issues these rounds filed are on GitHub,
  and each is closed by a merged round except those with a round ("Rounds that ended,
  and what each left") and these,
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
- The release gate runs the Rust suite and Clippy,
  a round runs the Rust modules it touched,
  and the coordinator's tip check ran `native:check`,
  Clippy and `source_guard`;
  where the TypeScript checks should run is a proposal for the owner (issue 6584).
- Issue 6586's round makes it impossible for the whole test process to reach this account's own service manager,
  and writes a proposal on whether the release should refuse to provision from a program that is not an installed wrapper.
- That the planted controls a round repairs still compile,
  and that a campaign notices each of them,
  is owed to the next `native:mutation:planted` run.
- A round's session cannot read `/var/log/audit/audit.log`,
  where a denial's record is and the kernel journal's is not:
  when a round reports a refusal only one program meets,
  the coordinator reads that log.
- The reload proof's control arm cannot pass:
  it needs a release from before the remedy of 2026-11-12,
  and its choices are
  `doc/planning/git-wrapper-reload-proofs-control-arm-without-a-pre-remedy-release.md`,
  ranked C > A > B there.
- Two proposals wait for the owner:
  `doc/planning/cli-git-notice-of-a-stopped-step-keeps-promising-a-stop.md`
  and `doc/planning/git-policy-cli-a-step-whose-watch-cannot-start.md`.
- A round's `edit` tool wrote a planning file into the main checkout and not its worktree
  (issue 6586's round,
  twice seen untracked there,
  gone later).
  The branch that fences a run's writes is held (issue 6378).
- `pnpm-shadow` and `search-shadow` print a refused start with the system's own answer;
  neither is released,
  because the standing answer on alphas does not name them
  (`doc/handover/questions-for-the-owner.md`).
- No real gate has run on the gate's reader of a container's control group yet:
  the next whole gate is its first.
- A wait loop is started with `run_in_background: true`,
  and a step that depends on a merge or a commit is issued after that command's notification,
  never beside it.
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
- Issue 6578 is reopened for one `cargo-shadow-<id>.scope` unit that a round's last whole-suite run left,
  which no single module reproduced.
  The coordinator stopped the twelve units of that shape that had been left.
- Worktrees of ended,
  merged rounds are not all removed.
  A removal of eleven on 2026-11-27 removed none:
  five were held by orphaned shells,
  three are dirty,
  and three are refused with "validation failed,
  cannot remove working tree:
  '.git' does not exist".
- A second writer makes issue 6339's shape:
  a build in a worktree whose add was interrupted writes that worktree's own `cargo-shadow` stamp,
  and the owed ignored-state copy can then never finish
  (a comment on that issue has the round's lines).
- The Slint round (`f1957916b0`,
  the owner's "Finish the move (Recommended)",
  `doc/decision/rust-dependency-catalog.md`) left three points open:
  the IDE's two `wrap_keys` fixtures need re-measuring for parley 0.13.0 (issue 6523),
  the IDE needs one container run before it is called green,
  and the three regenerated locks were written by `cargo fetch`,
  not through `cargo-shadow`'s trust policy.
- `source_guard/standards_review_boundary_tests.rs` now treats the declaration of a module
  whose name begins with a service-only module's name as a declaration,
  not a mention (`3fd13e17fe`);
  the round marked it for the owner to weigh.
- Two rounds split the four byte-identical copies of `src/shadow_identity.rs` in different ways.
  A file that several crates must keep identical belongs to one round.

### Next

- Merge each round when it has ended and its report is read,
  by the merge bar in "Standing directions".
- The last whole container gate ran on `main` at `6037c4e0f8`,
  started 2026-11-27T20:13:13-05:00:
  12321 of 12321 passed,
  51 skipped,
  exit status 0,
  the first whole gate to run issue 6625's twelve controls
  (log `~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-6037c4e0f8-20261127T201313-0500.log`,
  scratch).
  The next runs on the tip that holds the wrapper's five rounds,
  before `0.6.1-alpha.52`.
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
  A merge is made by hand in the main checkout,
  one branch after another
  ("How work is run side by side"),
  since the cleanup of 2026-11-27T15:26:37-05:00 deleted the coordinator's scratch merge script:
  the branch's tip equals its tip on origin and `main` is not behind origin;
  `git merge-tree --write-tree --name-only main <branch>` shows no conflict;
  `git merge --no-ff --no-edit <branch>`;
  then the branch's tip is an ancestor of `HEAD` and `HEAD^2` is that tip,
  before the merge is called landed.
  A branch whose package changed on `main` since the branch last took `main` in is merged with `origin/main` in a worktree first,
  and the package's format check,
  compile,
  Clippy and whole test task run there:
  on 2026-11-27 three `mpm-shadow` rounds merged with no textual conflict,
  and the merged tree did not compile.
  After the last of a run of merges,
  origin holds the tip before anything is built on it:
  when the commit service's push of `main` is refused with `GH009`,
  it does not try again,
  and one `git push origin main` clears it
  (`doc/troubleshooting/github-lfs-integrity-rejection.md`).
  A release commit of the linter counts as such a change for the wrapper:
  it writes the new version into `Cargo.toml` and `Cargo.lock` of `package/git-policy/cli` and of its fuzz sidecar,
  so a wrapper branch that took `main` in before it is checked again with it.
  That last check is not ceremony:
  on 2026-11-25T17:24:39-05:00 `git merge` returned 0 and no merge commit landed,
  while another session committed to `main` in the same second,
  and the script of that day printed the other session's commit as the merge.
  The wrapper removes the worktrees that hold a branch once it has landed
  (its README,
  "A merge that lands a branch also removes the worktrees that hold it"),
  and the merged lock branches of 2026-11-27 were gone from origin before the coordinator asked for it,
  by a step that was not read.
  A merged branch is then deleted by hand,
  locally and on origin:
  deleting issue 6620's branch locally left origin's copy
  (2026-11-27T15:19:09-05:00).
  Since issue 6620's merge the package's unit tests give every fixture's `git` the standing Git and no session place,
  so the TypeScript checks at `main`'s tip start nothing on the account's manager:
  at `bc24b2e688` `test:unit` and `lint:types` exited 0 on the pinned tree,
  the journal held no line for a fixture repository,
  and the shared link was as before.
- The wrapper's gate has tasks of its own,
  which the coordinator's gate of 2026-11-27 did not use and the next one does:
  `mise run //package/git-policy/cli:native:gate:main-tip` starts it on `main`'s tip in a capped unit,
  and `native:gate:verdict -- <log path> <deadline seconds>` reads its verdict.
- The `forbidden-strings` workflow has failed or been cancelled on every push to `main`
  since run 44656950451 of 2026-11-20T01:54:13Z,
  which is issue 6632.
  The workflow hands the scanner every changed entry as a file,
  and each of the 139 tracked links under `uses/` folders points at a folder,
  so each prints `read error: Is a directory`;
  run 46079689494 scanned 8149 entries and printed those 139 lines and no other finding.
  Its round is merged (`fc1c84b58e`),
  and run 46083398424 on that commit was green,
  the first since 2026-11-20:
  it left out the 139 folder links,
  said so,
  scanned 8017 entries and found nothing.
  The issue is closed.
  The step reads each changed entry's mode from `git diff --raw`
  and leaves out a gitlink and a link whose target on disk is a folder;
  it became two steps,
  because the linter caps a shell step's length and the old one stood at the cap.
  One thing is narrower than before:
  the old step checked a folder link's own name against the name rules before its read failed,
  and the new one does not.
  The released scanner's `--name-path` takes a link's target text under the link's name,
  so the workflow hands it both with no release of the scanner:
  issue 6633,
  whose round is merged (`af1647ef36`) and whose issue is closed.
  A third step,
  `Scan the entries left out of the file scan`,
  gives the scanner each left-out entry's own name,
  and for a link its stored target text,
  read from Git's object for the link and never through the link.
  Run 46086113778 on that commit was green in all three steps;
  its delta held no link,
  so what shows the step on real links is the round's run of the workflow's own step bodies
  with the released `0.6.2` binary on the delta `82d6a89596..60ecd08eef`:
  139 of 8158 changed entries left out,
  8019 files scanned,
  the 139 scanned by name and target text,
  every step exit 0,
  and a planted rule for `../` gave 139 findings there.
  The record is `doc/troubleshooting/forbidden-strings-scan-step-treats-tracked-folder-links-as-files.md`.
  The workflow cancels a run in progress when a later push arrives
  (`concurrency` with `cancel-in-progress: true`),
  so a run that must be read is waited for before the next push to `main`.
  Its open choices wait in `doc/handover/questions-for-the-owner.md`,
  beside what a positional link means inside the scanner.
  What the record's limits left was issue 6640:
  a changed link to a file had its stored target text scanned by nothing,
  and the whole-tree arm inspected no link and no gitlink.
  Its round is merged (`bed3564da4`),
  with both inspections in the workflow and the scanner unchanged at `0.6.2`:
  one step writes a record for each link and gitlink,
  a delta's from its changed entries and a whole-tree run's from the index,
  and the next hands the scanner each one's name and each link's stored target text in one call.
  The module holds 34 cases,
  each control red under one of seven plants,
  and on this repository's tree the whole-tree arm scans 139 links,
  clean,
  after a planted rule found 139.
  The issue is closed,
  on push run 46098402786 at `b16e1fdca2`:
  every step passed,
  the two new ones among them,
  and its delta of 33 regular files held no link.
  So that run shows the steps run and pass,
  and the scanning of links rests on the module's cases and on the round's own whole-tree run.
- `cargo-shadow`'s `src/lib.rs` holds 218 `mod` declarations and no function,
  and is the package's one `rust/max-lines` finding.
  Its round is merged (`60ecd08eef`),
  under the owner's "Keep 300,
  split everything"
  (`doc/decision/rust-files-keep-the-300-line-budget-after-the-vertical-reformat.md`):
  the file holds 201 code lines,
  because 52 test files are declared by the module they test,
  as the crate's newest modules already were,
  and a module that brings tests costs the file one line where it cost three.
  The crate's 731 public paths,
  and the release binary's help and version,
  are byte-identical before and after;
  the same 919 unit test names are there,
  527 under a new module path;
  and the package's task ran 968 and passed 968.
  Two tests that start their own binary with a filter naming their old path matched nothing after the move,
  failed in the first run,
  and name the new path now.
  A round that starts in this package from the alpha.16 read declares a test file the same way.
- The store fixture that tests of several packages build with `createSopsFixture`
  joined a tool's output when the tool exited,
  before its pipe had closed,
  so a store was now and then written empty or cut:
  issue 6410.
  Its round is merged (`25dec7b462`),
  with the branch of 2026-11-18 merged into it first;
  that branch and its review branch are deleted,
  and issue 2031 is closed by it.
  The runner joins at `close`,
  stops the tool's whole process group at a deadline,
  and names the failure.
  Measured by the round at 2026-11-27T18:35:53-05:00,
  with eight children in flight and 80 waits per arm:
  joined at `exit` the whole document arrived in 0 of 80 waits,
  joined at `close` in 80 of 80.
  The default deadline of 2,000 ms then failed four `sops --encrypt` calls falsely
  when the driver's unit suite ran under `CPUQuota=400%`,
  so a second round sized it from a measurement and is merged (`e948095a53`);
  the issue is closed.
  With the deadline lifted by a loader that rewrote the fixture module in memory,
  one run of the driver's suite per cap logged every fixture call:
  at `CPUQuota=400%`,
  36 calls,
  median 104.2 ms,
  slowest 4,597.5 ms;
  at `CPUQuota=1000%`,
  36 calls,
  median 39.3 ms,
  slowest 513.7 ms.
  The default is 15,000 ms,
  the deadline the reader gives the same `sops` child,
  and the suite then passed whole at `CPUQuota=400%` with the shipped default
  (228 `[PASS]` lines and no `[FAIL]`).
  A true hang costs 15 s once where it cost 2 s.
  Why 32 test files run under a 400% cap:
  the repository's own test runner sizes its workers from `availableParallelism()`,
  which reads the affinity mask and not the control group's `cpu.max`;
  the cure and its cost are in `doc/troubleshooting/subagent-driver-suite-fails-under-host-load.md`.
  What both reports left waits in `doc/handover/questions-for-the-owner.md`.
- The driver package's unit test task ran its tests against the built copy and did not build first:
  issue 6635,
  whose round is merged (`8e4a0505bf`) and whose issue is closed.
  The task declares `depends = ["build"]`,
  as `package/zip-writer/core` and other packages already do,
  so mise builds before the first test file starts;
  a test asks mise for the task's dependency list and was red before the declaration.
  Shown before the change,
  with no edit:
  a built copy older than a source file ended the task with status 1,
  two files failing to load while 72 ran against the stale copy and passed;
  with no built copy 75 files failed to load.
  After it both states print 229 `[PASS]` lines and no `[FAIL]`.
  The cost is the build's own 5.053 s per run.
  `depends` and not a `run` list,
  because a `run` entry is handed the words after `--`,
  which name test files here;
  the root `mise.toml` says never to use `depends`,
  and that sentence against the tree is a question for the owner.
  The round counted 96 more packages whose unit tests import a built copy their task does not build,
  and the root aggregate `test:unit` builds nothing:
  issue 6637 holds the list,
  and none was changed.
- Rounds wrote edits into the main checkout on 2026-11-27:
  the round of issue 6632 by its own report,
  and the round of issue 6625 as the coordinating session found
  (the wrapper's `SPEC.md`,
  one of its test files and a planning document).
  Each brief said to read a file "on `main`",
  the round opened the main checkout's copy,
  and its next edit landed on the path it had opened.
  The coordinating session saved the files' differences as one patch in the round's `keep/`,
  put the main checkout's copies back with `git restore --no-enforce-worktree -- <paths>`
  (the way the wrapper's refusal of `git checkout -- <path>` names,
  once a patch is saved),
  and told the round,
  which applied the patch in its worktree.
  Its first note said the restore was done before it was;
  a second note corrected it.
  The gate did not build the stray test:
  its image is built from the gate's own worktree,
  and the probe's name is nowhere in the gate's log.
  A brief names a file to read as the worktree's copy,
  never as "on `main`";
  the brief rules say so now,
  and every live round was sent a note.
  Issue 6636 asked for such an edit to be refused;
  its round is merged (`6037c4e0f8`) and the issue is closed.
  The driver hands every run's Pi a second extension that refuses an `edit` or `write` call naming the main checkout,
  outside `.claude/worktrees/`,
  and answers with the worktree's copy of the path;
  a Pi the driver did not start loads nothing,
  and a run with no worktree is handed the word `off`.
  A run's record keeps the path of every `edit` and `write` call whole.
  The round's tests use a stand-in Pi,
  so the coordinator showed it with a real one after the merge:
  a probe round started 2026-11-27T20:11:05-05:00 had its edit and its write of two ignored files under the main checkout's `.cache/` refused
  (the one file still held its old word and the other did not exist),
  and its write in its own worktree succeeded.
  A shell command that writes into the main checkout is not refused,
  which the package's README and the brief rules say.
  The round's three open points wait in `doc/handover/questions-for-the-owner.md`.
- Worktrees and branches of ended rounds were cleared at 2026-11-27T20:15 -05:00 and after:
  110 local branches and 9 of origin's whose tip is on `main` are deleted,
  and the worktrees whose branch was merged by a merge commit,
  that hold nothing uncommitted and that no process names or stands in,
  are removed.
  Kept,
  each for its reason:
  six whose tip is not on `main`
  (`clone-rung-partial-clone-of-a-dirty-source`,
  `fence-faster-size-reader`,
  `fence-size-read-beside-source-check`,
  `integrate-worktree-removal-held-commit-control-failed-in-two-gates`,
  `pi-prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked`
  and the review worktree of the driver's queue of runs);
  and six whose branch is merged but that hold uncommitted paths,
  from 4 to 25,522 of them,
  which nobody has read
  (the three `pi-break-wrapper-file-loops-*`,
  `pi-split-cargo-shadow-cli-lean-files-and-crate-root-over-the-line-budget`,
  the second part of `pi-split-mpm-shadow-cli-toolchain-files-over-the-line-budget-second-wave`,
  and `pi-split-wrapper-merge-files-over-the-line-budget-4`).
- A merge of a branch whose worktree still stands makes the wrapper hand that worktree's removal to the commit service,
  and says so in a notice;
  check that the folder,
  the local branch and origin's branch are gone afterwards.
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

On 2026-11-27,
with 23 rounds started between 21:11 and 21:42 -05:00,
the first builds of those rounds ran together.
At 21:48:24 `MemAvailable` read 1549 MiB,
swap held 50,404 of 66,219 MB,
and the memory stall read `full avg10=46.85`,
with `rustc` holding 28.8 GiB resident and seven guests each holding about 24 MiB of swap.
At 21:49:53 `MemAvailable` read 5833 MiB and the stall `full avg10=3.13`.
`systemd-oomd` killed nothing;
its limits are 90% of swap used and a memory stall of 60% for 30 s.
One start was lost in those minutes:
the driver's guard that asks which `git` a session would run waited 10 s for the wrapper's `--help` and refused the round,
which started from a list of its own afterwards (issue 6646).

The machine ran out of memory a second time that night,
for longer.
Eleven `run-p*.scope` units of the rounds ended with the result `oom-kill` between 22:34:15 and 22:48:09.
`systemd-oomd` wrote "Marked ... `mvm-libvirt-session.service` for killing" at least fourteen times between 22:47:44 and 22:48:10,
with memory used at 66,220,068,864 of 66,220,068,864 bytes and swap used up to 65,982,111,744 of 66,219,667,456;
the unit stayed active,
started 20:03:51 with no restart,
and its seven guests were running at 22:49:20.
Why a marked unit lived is not known.
The wrapper gate's compiler was ended by signal 9 in those minutes.
At 22:51:42 `MemAvailable` read 0 kB and `SwapFree` 2,267,792 of 64,667,644 kB,
with eleven `rustc` processes and nineteen rounds;
at 22:57:32 they read 7,961,472 kB and 15,256,480 kB.
Whether the kernel's own killer or a unit's limit ended each build was not read:
the kernel's log gave this account nothing.
What held the memory was read at 23:20:27,
in a third such stretch:
the unit that holds the guests,
`mvm-libvirt-session.service`,
held 29,937,127,424 bytes of memory and 50,694,000,640 of swap with ten guests running,
and the whole of `agent-work.slice`,
where the rounds' builds run,
held 15,762,833,408 and 1,350,725,632.
Each guest is at `mvm`'s default of 8,192 MiB,
which the owner kept until the crate split (`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`).
That reading counts what was left:
it was taken seconds after twelve builds had been killed,
with three compilers alive.
Over its life the builds' slice has peaked at 55,702,958,080 bytes of memory and 56,149,524,480 of swap,
and the guests' unit,
since it started at 20:03:51,
at 33,109,938,176 and 52,823,150,592
(`MemoryPeak` and `MemorySwapPeak`,
read at 23:25:24),
so neither is shown to be the larger at the moment the memory ran out.
What both readings of the night show is that the guests' unit holds about 50 GB of the 64.7 GB of swap the whole time,
whether a guest is in use or not,
and a build holds its peak for minutes.
The builds are what the machine kills when the swap is full.
The coordinating session first wrote here that the fourteen rounds compiling the wrapper were the load,
and then that the guests were the largest holder;
the first was read off the process list and the second off a reading taken after the kills,
and the peaks correct both.
Between 23:15:47 and 23:19:55 twelve more scopes ended `oom-kill`,
and at 23:19:36 `SwapFree` read 55,744 kB of 64,667,644 with `MemAvailable` at 0.
From 23:06 to 23:20 the stall's `full avg300` stood between 30 and 54:
every task that could run was stalled on memory for a third to a half of that time.
By the owner's criteria for a level of agents at once the stall fails no level
(`doc/handover/agents-at-once-on-this-machine.md`,
"What counts as the ceiling"),
and those criteria do not speak of a killed build.
What the coordinating session did:
it started nothing that compiles the wrapper,
wrote the live rounds one note ("Rounds running"),
put the gate's rerun behind a memory reading ("What the wrapper's alpha.51 reads found"),
and started a unit that writes one reading a minute for six hours,
`memory-readings-once-a-minute-on-the-night-of-2026-11-27`,
to `~/temp/agent/memory-exhausted-on-the-night-of-2026-11-27/keep/readings-once-a-minute.txt`.
It stopped and froze no round.
Whether rounds that compile the wrapper get a bound of their own is the owner's question
(`doc/handover/questions-for-the-owner.md`).
The record is `doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`.

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
`SPEC.md` gives the switch two conditions,
the cancel command,
which step 17 built and no grammar admits,
and a release whose printed sentences name only commands,
records and states it has;
and the snapshot batch's request budget has no value in release code.
So nothing calls `enter_the_removal` yet and `source_guard`'s
`removal_command_path_writes_nothing_tests.rs` holds the cut.
`doctor` still answers the cancel command's check as unknown
(`health/doctor_zfs_route/later_step_checks.rs` says step 17 builds it).
Nothing merged between that time and 2026-11-27T21:11-05:00 did any part of that wiring,
by a read-only digest of the route against `git log`;
the rounds that do started then ("Rounds running").

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
  No release code gathers the facts those checks read;
  the add route calls the delegation check itself
  (`worktree_clone_add_route/add_grants.rs`),
  so that one is reached.
- Step 19's unit texts and its immutable entry's verdict are built ahead (`72a0be07ef`).
  The migration is no program:
  it is `doc/runbook/zfs-checkout-migration-and-boot-setup.md`,
  marked not run,
  not approved and not final,
  and it has not run on this host.
  The owner answered every choice
  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`),
  and the chosen design merged as `1ef4afefa8`.
- Step 20 merged unwired (`a7502138a4`);
  writing `sanoid.conf` is the owner's act.
  Step 21's watch comparison merged and nothing calls it
  (`worktree_clone_watch_rebinding/`);
  the skip of the separate ignored-file copy after a clone fill is not built.
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
the rounds of the list of 2026-11-27T21:11-05:00 ("Rounds running"),
each read and merged as it ends.
They were briefed from a read-only digest of what is left of the route,
scratch,
`~/temp/agent/zfs-route-remaining-pieces-digest-2026-11-27/keep/digest.md`;
what of it must last is written here.

The switch itself is one of those rounds,
`zfs-removal-command-path-runs-the-whole-removal-with-force-still-refused`.
It merges after the management grammar's round.
Its brief takes 65,536 bytes of packed request input from the measurement in
`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
an agent's draft that left each of its values to the owner.
The owner accepted that one on 2026-11-27,
through the question tool
(`doc/decision/cli-git-zfs-removal-batch-request-budget-is-65536-bytes.md`),
so the round's branch merges with it;
the draft's other values stay unset.
`--force` stays refused in that round:
the forced arm's waits are unset,
and the owner's decision is that it does not ship before they are measured.
The rounds on the private propagation step,
on the body of `shutdown-unmount` and on the lost-event merge device each build the piece with its guest arm and add no call from a root mode,
because those calls are step 19's wiring,
where the step's "Depends on" still holds
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).

Held,
each with what it waits for.
The orders are the coordinator's own unless a record is named:

- printing the ZFS report from `doctor`:
  the three `doctor` rounds,
  whose gatherers and checks it composes;
- the origin cleanup look's calls from `git worktree add`,
  `list`,
  `prune` and `repair`:
  the removal's switch,
  because a blocked-cleanup record exists only after a removal,
  and to keep that round's files apart;
- offering `--force`:
  the switch,
  the two forced-arm rounds,
  and the waits' values,
  by the owner's decision on step 17;
- step 21's wiring,
  the watch comparison after a fill and the skip of the separate ignored-file copy:
  step 19,
  which step 21 depends on;
- the values of step 22 in code,
  the mount and unmount deadlines and the forced waits:
  the owner's acceptance,
  by the step 22 record's own sentence;
- the lost-event numbers and the nudge's switch:
  both lost-event rounds and a guest run of the design's measurements,
  by the lost-event decision's choice 17;
- Sanoid 2.3.0's coverage arm in a guest:
  a way to put Sanoid 2.3.0 into a guest,
  which plan step 23's template does not hold.

The host's own acts wait for more than the switch.
The checkout migration waits for removal to exist
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
"After removal exists (Recommended)"),
which the coordinator reads as the switch and the grammar released and installed;
the units,
the immutable entry,
`snapdir=disabled` and the zedlet wait for step 19's wiring and its guest proof;
`sanoid.conf` waits for step 20's wiring and the Sanoid arm;
and step 22's storage half runs after the migration.

Outside steps 17 to 23,
by `SPEC.md`'s list under "What this release does not yet do" for the add route,
and needed by none of the rounds:
the `.gitattributes` fence's own write;
the check after a restore;
finishing an add with Git's own checkout when its proof or its restore fails;
and the warning above the measured count of changed paths.

The lines of other records that `main` had passed are corrected,
by a records-only round merged as `24de7bc3e6`;
what it left for a decision waits in `doc/handover/questions-for-the-owner.md`.

Rounds keep their files under `~/temp/agent/<round name>-<date>/`,
which expires a day after its last change ("Scratch expires after a day");
a round's durable findings land in its branch's records.

#### Accepted designs that wait for their build steps

- Private propagation after a replacement mount,
  for step 19:
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- The triggers a lost last event waits for:
  `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.
  The design and its decision give parts to plan steps 10,
  14,
  18,
  19 and 22
  (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
  choice 17),
  where the plan's own text names steps 7 and 22;
  none of the parts given to steps 10 and 14 was built with those steps.
  Two rounds build the wrapper's side and the handler's side ("Rounds running").
  That design's own worktree is gone,
  by `git worktree list` at 2026-11-27T21:46:38-05:00.
- The forced removal's two kept refusals,
  of a locked holder and of a taskless namespace:
  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.
  The owner answered "Keep the refusal (Recommended)" to each on 2026-11-26T00:05-05:00
  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
  "The two refusals the built arm keeps"),
  so each waits for a guest arm and not for the owner.
  Two rounds measure them ("Rounds running"):
  the stop of a locked holder's own user unit,
  and the walk that finds a taskless namespace on a kernel that has `listns`.

Each one's acceptance is the section "The design is accepted" of its decision record.

#### Later steps' pieces built ahead, unwired

Pieces that need only work on `main` are merged as code no person's command reaches,
so their steps wait only for their wiring.
What is still built ahead:
step 17's cancel command (`df15f27a7a`) and forced arm (`64a71a4c9b`,
`3ebc78c3dc`),
the settling command,
the removal's finish pass,
the origin cleanup look and the snapshot batch's callers,
each reached by no command;
step 18's `doctor` lines (`838b7f5e71`,
`d463965dcd`,
`2bf72fb230`);
step 19's unit texts and immutable entry (`72a0be07ef`);
step 20's `sanoid.conf` writing (`a7502138a4`);
and step 21's watch comparison.
Step 16's deferred-destroy checks,
step 15's deferred-origin check and step 13's managed lock are wired,
though no command reaches them while the cut stands.
Step 12's move refusal and unmounted report are wired and in use
(`7cf926dc56`;
`3ea4fa16c3` is the merge that built them ahead).

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
("Rounds that ended,
and what each left").

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
and everything under `package/git-policy/cli` was the same.
Both are fixed (`beb61ec46b`):
4,062 findings in 480 files,
with `bin/` and `e2e/` left as they were while the round of issue 6612 worked there;
that round fixed the two findings one file kept.
The installed linter applies that fix itself with `--fix`,
which changes whitespace only,
so the scratch script of 2026-11-23 is not needed.
On 2026-11-27 it took 76 findings out of `package/mpm-shadow/cli` (`ee8871b190`)
and 11 out of `package/cargo-shadow/cli` (`f6b4dc0947`),
every one in a Rust doc comment,
and 778 out of five Markdown files of `package/pi-shared/subagent-driver` (`e6026dbd46`),
whose six Markdown files render to the same text as before,
in a window when no round worked in those packages:
in a worktree per package,
each changed file was equal to its old text once whitespace and comment markers were taken off,
`cargo fmt --check`,
`cargo check` and Clippy exited 0,
and the whole test task passed,
910 of 910 and 968 of 968.
The wrapper's fix was checked the same way,
and three ways more:
of 6,789 changed Rust lines none began with anything but a doc marker;
the linter reported no layout finding and no processing failure,
as before;
and the `source_guard` module (225 of 225) and the two modules whose tests read `SPEC.md` passed.
The same count,
of changed lines that are no doc comment,
is zero for the two packages' merges too.
`bin/` and `e2e/` need no sweep:
measured on `main` after issue 6612's merge,
the package holds no `markdown/semantic-line-breaks` finding.
The driver package's build,
`lint` and `lint:types` exited 0 in its worktree,
and its unit suite printed 228 `[PASS]` lines beside one failure that is not the fix's:
the store fixture of issue 6410,
which the Next list names.
`cargo-shadow`'s `src/lib.rs` is over the line budget,
305 code lines against 300,
and the Next list names its round.

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
