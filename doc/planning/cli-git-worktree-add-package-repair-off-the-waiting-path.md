# What a worktree add's package repair is for, and what an add waits for

What:
 the steps the package repair runs after a new worktree is filled,
 what would be wrong in the new worktree without each one,
 what a build does when it starts before a step has run,
 and the change that takes a clean off the add's waiting path where it deletes nothing.
Why:
 The owner's bound is "60s is fine for adding worktrees",
 and the repair was the largest part of an add that passed it:
 34 `cargo clean` runs, 32.8 to 47.7 s of workspace resolutions in each of three real adds,
 every one of them deleting nothing.
Future sessions read this before moving any step of the repair,
and before measuring an add again.
Status:
 measured on 2026-11-12 from the wrapper's own timing records of this repository,
 from a btrfs fixture of it,
 and from one `strace` of each arm;
 changed on branch `perf/git-policy-cli-worktree-add-does-not-wait-for-its-package-repair`.

## The steps, what each is for, and what recorded the reason

An add fills the new worktree,
copies the ignored files,
and then repairs the package managers of that worktree
(`src/native/worktree_copying/worktree_copy_after_lock.rs`,
`src/native/repair/package_repair.rs`).
Three lanes run beside each other,
and the add waits for the slowest:

- pnpm,
  `pnpm install --offline --frozen-lockfile` in the worktree.
  Without it a copied `node_modules` keeps links that climb out of the tree by the old number of
  levels,
  so an entry resolves into the source checkout or not at all
  (`README.md`,
  "the target is left alone";
  the positive control of
  `binary_package_repair_tests::real_pnpm_relinks_a_global_virtual_store_install`
  shows a verbatim copy of an install holding a dead entry).
  The reason is recorded:
  the owner widened the wrapper's pnpm exemption to every package manager on 2026-11-03
  (`doc/planning/cli-git-main-worktree-ignored-copy.md`,
  "The wrapper may launch pnpm itself for worktree copying"),
  and `SPEC.md`,
  "Package-manager repair of a new worktree",
  says a worktree whose copied links were never relinked resolves the source's packages.
- toolchain,
  `mpm-shadow install --frozen-lockfile` in the worktree.
  Without it the launchers under `.cache/toolchain-launchers` name another checkout's
  `toolchain.lock.json`,
  so the worktree runs the source's tool versions.
  Recorded:
  the owner chose "Install online during the add" on 2026-11-03
  (`doc/planning/cli-git-main-worktree-ignored-copy.md`),
  told that an add can then download.
- Cargo, which had four steps before this change:
  1. `cargo metadata --locked --offline --no-deps` for each Cargo workspace among the tracked
     `Cargo.toml` files,
     timed as `cargoMetadataWorktree`.
     Without it the wrapper does not know a workspace's build and target directories,
     and `doc/decision/cargo-build-dir-host-cache.md` allows no reader of the build folder's layout
     except the `cargo-shadow lean` sweep,
     so the question has to be put to Cargo (`SPEC.md`,
     "Package-manager repair of a new worktree").
  2. The same command in the source checkout's counterpart workspace,
     timed as `cargoMetadataSource`.
     Without it the repair cannot tell whether the source holds build state to clone,
     nor where.
     Same recorded reason.
  3. `cargo clean --offline --package <member>` for every member of the workspace,
     timed as `cargoClean`.
     This is the step this change moves.
     Without it, where the copy carried build state of the worktree's own crates, Cargo treats those
     artifacts as fresh and a build reuses them at the wrong paths:
     measured below.
     The reason is recorded:
     the owner chose "Reuse deps,
     rebuild own",
     done with "Run cargo clean -p",
     on 2026-11-03
     (`doc/planning/cli-git-main-worktree-ignored-copy.md`,
     summed at `doc/handover/cli-git-rust-implementation.md`,
     "Worktree add and package managers").
  4. The clone of the source's external build directory into the worktree's,
     and the clean of that copy.
     Without it the worktree is correct and its first build only compiles more.
     It is already the commit service's work
     (`doc/planning/cli-git-worktree-add-package-repair.md`,
     "The change"),
     queued as a `prewarm` record.
- The lock the add's clean displaces,
  read before it and put back byte for byte after,
  and kept under the Git directory when a concurrent write makes the two ambiguous.
  Without it the clean rewrites a `Cargo.lock` that no longer matches its manifest,
  or creates one that is missing,
  in a worktree nobody has yet.
  Recorded:
  `SPEC.md`,
  "A committed `Cargo.lock` that disagrees with its manifest".

No step of the repair has no recorded reason.

## What a build does when it starts before a step has run

Each probe is a throwaway Cargo workspace under `${HOME}/temp/agent/pkgrepair-fingerprint`,
built with its own `CARGO_HOME`:

- Before `cargo metadata`:
  nothing is written but the workspace's `target/` directory,
  which is not even made (a copy without `target/` gained none from the read),
  so a build started at once compiles everything and is correct.
- Before the clone of an external build directory, where the worktree has no build directory of
  its own:
  a copy of a built workspace, `target/` included,
  whose Cargo home keys the build directory by workspace path,
  recompiled its crate on the copy's first build and the binary named the copy's own path.
  Correct, and only slower.
- Before `cargo clean`, where the copy carried build state of the worktree's own crates
  (`target/` inside the worktree, Cargo's default layout):
  `cargo build` in the copy answered `Finished dev profile in 0.15s` without compiling,
  and the binary still printed the source checkout's `CARGO_MANIFEST_DIR`.
  Wrong, and wrong silently.
  This is why the clean cannot be moved after the add returns on that layout.
- Before the pnpm relink: an entry of the copied `node_modules` resolves into the source checkout
  or not at all;
  the control cited above holds the dead entry.
- Before the toolchain install: the worktree runs the tool versions its launchers name,
  which are the source's.

## What the add waited for, measured

### Real adds of this repository

The wrapper writes one line per add to
`<git-common-dir>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`.
Three adds of release 0.6.1-alpha.12 on 2026-11-12 between 03:41 and 03:43:

- `totalMs` 39,694, 48,606 and 62,634 ms;
- `cargoClean` 32,823 ms over 34 calls, 47,692 over 34 and 44,376 over 34,
  which is the largest single item of the repair in all three;
- `cargoMetadataWorktree` 838, 3,131 and 1,597 ms over 35 calls;
- `cargoMetadataSource` 2,376, 1,081 and 3,253 ms over 35 calls;
- `repairCargo`, the whole Cargo lane, 9,768, 14,835 and 12,849 ms;
- `relinkHandOver` 7, 16 and 41 ms, and `cargoPrewarmHandOver` 106, 10 and 14 ms.

### The fixture

`${HOME}/temp/agent/pkgrepair-repair-off-the-path`:
a btrfs subvolume holding a clone of this repository (36 tracked workspaces,
each with the `target/` directory a developer's checkout holds,
one of them a tiny workspace the fixture built so its external build directory exists),
snapshotted once per add.
Every arm has its own `HOME`, `XDG_RUNTIME_DIR`, `CARGO_HOME` and `RUSTUP_HOME`,
the tool caches reached through links so `--offline` finds the crates,
a `DBUS_SESSION_BUS_ADDRESS` naming a socket that is not there (so no arm reaches a commit service),
and the same Git configuration.
Four arms, four rounds, the order rotating:

- the merge commit's wrapper (`wrapper-before`, a debug build of `451f7d330b`),
- the same binary a second time, to bound the spread of one binary against itself,
- this branch's wrapper (`wrapper-after`),
- native `git worktree add`.

Wall clock, which cannot judge this change:

- `before` median 26,938 ms, MAD 11,563 ms, largest 103,055 ms;
- `before` again median 12,480 ms, MAD 4,284 ms, largest 51,479 ms,
  and the two runs of that one binary differ by 51,576, 21,112, 13,361 and 996 ms round by round;
- `after` median 10,298 ms, MAD 3,611 ms, largest 15,406 ms;
- native median 9,285 ms, MAD 3,090 ms, largest 13,587 ms.

The wrapper's own record of the same adds, which is what judges it:

- `before`: 36 `cargo clean` calls in every add,
  `cargoClean` 249,188 ms of summed work over the four adds (median 33,420 ms in one add),
  and the Cargo lane (`repairCargo`) 12,042 ms at the median and 67,840 ms at the largest;
- `after`: no `cargo clean` call at all,
  and the Cargo lane 460 ms at the median and 2,289 ms at the largest;
- in both arms the add read its own 36 workspaces and their 36 source counterparts,
  at a median of 764 and 664 ms of work before and 618 and 602 ms after.

### What the add starts, from `strace --follow-forks`

One add per arm, traced, counting the Cargo runs the add starts and the flushes it makes:

- `before`: 72 `cargo metadata` starts and 36 `cargo clean` starts
  (144 and 72 `execve` lines, because the rustup proxy execs the real Cargo),
  403 `execve` calls in all, and 19 flush calls;
- `after`: 72 `cargo metadata` starts and none for a clean,
  187 `execve` calls, and one flush call.

## The change

- A `cargo clean` no longer runs in the add where it deletes nothing
  (`src/native/repair/package_repair_cargo.rs`,
  `plan_build_state`).
  A copied `target/` is no longer a reason to clean,
  and on this host,
  whose Cargo keys the build directory by workspace path outside the tree,
  that removed every clean an add ran.
  The measurement behind it is the first and third probes above:
  a worktree with no build directory of its own rebuilds its own crates by itself,
  and a copy that carries build state in `target/` does not.
- Where the copy did carry build state,
  the add moves that state aside in one rename before it answers
  (`stage_the_worktrees_own_state` in `package_repair_prewarm_job.rs`)
  and adds the workspace to the same `prewarm` record,
  as an entry whose `stagedIn` names where the state waits.
  The staging name lies under the repository's Git directory
  (`monochromatic-git-policy-cli-prewarm-staging/<hex of the build directory>`),
  never in the worktree,
  because a name beside the build directory would show in `git status` and a `git add -A` would
  commit a copy of `target/`.
  The filling cleans that copy and publishes it back with the rename that never replaces,
  exactly as it already filled an external build directory.
  The add runs no `cargo clean` on this path at all,
  which is what puts it inside its bound:
  the clean's quiet window is six hundred seconds and its ceiling eighteen minutes.
  A build that starts at once finds no build directory and compiles everything itself,
  which is correct and slower than a build that waits for the filling.
- Where the state cannot be moved aside (a staging name another writer holds,
  a directory that cannot be renamed),
  the add cleans it where it stands as every release before this one did:
  the tool's own window applies there,
  and that path is why the clean's code and its lock-keeping stay.
- The source's build directory is read only where the worktree's own is absent.
  A worktree that holds build state needs its own members moved aside and nothing cloned,
  so the read was a workspace resolution the plan never used.
- A filling whose process the service was killed after publishing gives its names back:
  the staging directory and the marker beside a build directory that a build now holds are removed
  by the next filling of that workspace,
  which used to answer `Filled::Kept` and never look at those names again
  (`src/native/repair/package_repair_prewarm_job.rs`,
  `give_back_what_a_killed_filling_left`).
- The line the add prints for work it hands over now names where its progress shows:
  the queue's own states,
  `<git-common-dir>/monochromatic-git-policy-cli/work/{pending,claimed,done,failed}`,
  and the commit service's journal.
  A line that says work happens after the command returns without saying where to look is not a line
  a person can act on.

What stays on the waiting path, and why:

- One `cargo metadata` read per workspace, and their source counterparts where a fill is possible.
  A clean has to be decided before a build can start,
  so the layout question cannot move after the add,
  and moving the source read into the commit service would make the service resolve every
  workspace of every add for an answer the add already has
  (it is the same reads, so no work is saved, only moved into the queue commits share).
  Measured: 1,382 ms of work at the median across 72 reads after the change.
- The relink and prewarm hand-overs, which are a record and a wake.

## The two findings of the 2026-11-11 audit

The audit is
`${HOME}/temp/agent/kimi-trials/kimi-audit-alpha-12-package-repair-on-add-and-commit`.

### A clean reached outside the worktree

`plan_build_state` checked that a copied `target` does not leave the worktree only when Cargo's
build directory is inside the worktree.
With an external build directory,
a copied `target` link plus a source with no build directory chose `CleanOnly`,
and `cargo clean --package` deleted the files the link names.
That breaks `SPEC.md` ("Nothing is cloned or cleaned when ... a clean would reach a directory outside
the worktree") and `README.md` ("the target is left alone").

Reproduced here against the pre-change code by putting the audit's own test into a worktree of that
commit and running it:
the shared artifact was gone (`survived=false`) and the test failed at its assertion.
The change closes it by removing that arm:
nothing is cleaned where the worktree has no build directory of its own,
so no clean follows a copied link at all,
and the in-tree arm keeps both link guards.
The control is
`binary_package_repair_tests::an_external_build_directory_never_cleans_through_a_linked_target`
and
`package_repair_cargo_tests::a_target_link_that_leaves_the_worktree_is_never_cleaned_with_an_external_build_directory`.

### The clean of an already copied `target/` ran inside the add

The second audit (`alpha-12-merge-discarding-commands-lfs-and-package-repair`, finding F08) showed the
`CleanOnly` plan still running `cargo clean` inside the add under the tool's own window:
with a stand-in `cargo` whose `clean` never ends, `git worktree add` exited zero after 615.7 and 635.3
seconds.
That is the arm this change takes off the add's path, as "The change" describes:
the add moves the carried state aside in one rename and hands the clean to the filling.

The audit's own probe (`reproductions/repair_probe.ts hang-cargo`) run against the wrapper built from
this worktree, on the same fixture it makes (a repository whose copy carries `target/`, a `cargo` whose
`clean` never ends, and no commit service reachable):

- `add: 20.5 s exit=0`, and
- the stand-in `cargo` record holds one `metadata` run and no `clean` at all.

What the first build in such a worktree does:
it finds no build directory, because the add moved the copy's state aside, so Cargo compiles
everything itself, including the dependencies the copy carried.
That is correct, and slower than a build that starts after the filling has cleaned and published that
state back.
The record stays queued, so a service that runs later still cleans and publishes it.

Two properties of this repository's filesystem shaped that step:

- A worktree of this repository is a btrfs subvolume, and btrfs refuses to rename a directory out of a
  subvolume (`EXDEV`, measured here while trying to stage the state under the repository's Git
  directory), so the state is staged beside the build directory it is taken from.
- That name stands inside the worktree, so the add names the three staging patterns in the repository's
  own `info/exclude` (`*.monochromatic-prewarm/`, `*.monochromatic-prewarm.target/`,
  `*.monochromatic-prewarm.owner`), once, before the rename:
  without it the person's `git status` would show the wrapper's staging and a `git add -A` could commit
  a copy of `target/`.

### The no-service fallback exceeds the add's bound

When no commit service answers,
the add relinks the worktree inline
(`package_repair.rs`,
`relink_here`),
and that wait is the tool's own:
a 600-second quiet window and an 18-minute ceiling
(`src/native/service/wait_ceilings.rs`).
This is intended behaviour (`SPEC.md`,
"the add relinks the worktree itself when no service answers"),
and it is not the add's 60-second bound.

Measured here on a fixture of its own:
a repository of a lockfile and an ignored `node_modules`,
a `pnpm` first on the add's search path that sleeps 62 seconds and exits zero,
and a `DBUS_SESSION_BUS_ADDRESS` that names no socket,
so the add's relink record could reach no service.
The add took 72,850 ms,
its record says `repairPnpm` 62,122 ms and `relinkHandOver` 6 ms,
and the line it printed while it waited was:
"`<pnpm> install --offline --frozen-lockfile is still running in <worktree> for the repair of the new
worktree; it is stopped when it shows no sign of work for 600 seconds, or after 18 minutes at the
latest.`"
So an add whose `pnpm` is slow reaches 72 s here,
and the two bounds that end that tool are the tool's, not the add's.

The options, and what each costs:

- Give the inline fallback the add's remaining budget as its quiet window and ceiling.
  Pros: the add is bounded as the owner asked, and a slow `pnpm` becomes a printed line naming the
  command to run, which the tool already prints when its own bounds end it.
  Cons: the owner's 600-second window is one number for every tool run, and a second, smaller number
  for this path asks for a decision about which one wins;
  the tool's line would name a bound no other run has.
- Leave the relink to the service alone and print the command instead of running it inline.
  Pros: the add never waits for pnpm at all, and the wait for a service is already bounded
  (`HAND_OVER_GREETING_WAIT`).
  Cons: a session with no service manager (a container),
  which is where the inline fallback does its work today,
  would leave every new worktree with the source's links until a person runs the command.
  `SPEC.md` says such a worktree resolves the source's packages without an error,
  so this is a loss of correctness-of-location rather than a broken build.
- Leave it as it is and record the exception.
  Pros: no change.
  Cons: an add's bound is stated as 60 s and this path does not hold it.

Ranking: the first, then the second, then the third.
The first keeps what the fallback is for and bounds it;
the second is the stronger bound at the cost of a worktree whose links name the source checkout;
the third keeps a stated bound that is not held.
None of them is taken here:
the relink's own failures and retries are another session's work
(`fix/git-policy-cli-relink-of-a-new-worktree-does-not-fail-on-pnpm-and-is-not-given-up`),
and changing the wait under it would collide.

## The controls

Each control failed before the change and passes after it.
The failing lines are quoted from the runs at the commits named beside them.

- `package_repair_cargo_tests::a_worktree_with_no_build_state_of_its_own_is_never_cleaned`
  (`src/native/repair/package_repair_cargo_tests.rs`):
  `a worktree whose build directory is absent holds nothing a clean would reach, so it is not
  cleaned: CleanOnly`.
- `package_repair_cargo_tests::a_worktree_with_no_build_directory_of_its_own_runs_no_clean`:
  `the add starts no clean where the worktree holds no build directory: ["clean --offline --package
  tiny | .../worktree/package/tiny"] of metadata --locked --offline --no-deps --format-version 1 |
  .../source/package/tiny`.
  Its positive control,
  `a_build_directory_the_copy_carried_is_cleaned_before_the_add_answers`,
  records a clean for the same stand-in Cargo when the worktree holds build state.
- `package_repair_cargo_tests::a_target_link_that_leaves_the_worktree_is_never_cleaned_with_an_external_build_directory`:
  `a clean must never reach outside the worktree it was started for: CleanOnly`.
- `package_repair_prewarm_job_tests::a_filling_killed_after_it_published_leaves_nothing_of_its_own`:
  `the staging directory the killed filling left is given back: .../worktree-build.monochromatic-prewarm`
  (the staging name the entry gives now lies under the Git directory,
  and the control follows it).
- `package_repair_prewarm_job_tests::a_staged_state_the_add_moved_aside_is_cleaned_and_published`,
  `a_build_that_got_there_first_keeps_its_own_and_the_staged_state_is_given_back`,
  `a_staging_name_another_writer_holds_is_never_renamed_over`,
  and `a_worktree_with_no_build_state_has_nothing_to_move_aside`.
- `binary_package_repair_tests::a_build_started_at_once_is_correct_and_the_add_starts_no_clean`,
  run against the merge commit's code:
  `the add starts no clean for a workspace whose build directory the worktree does not have; it ran:
  ... clean --offline --package other | .../deeper/at-once/other`.
  After the change the same control builds the new worktree at once,
  the worktree's own crates rebuild, and the binary names the worktree's own paths.
- `binary_package_repair_tests::an_external_build_directory_never_cleans_through_a_linked_target`,
  run against the merge commit's code:
  `the add's cargo clean deleted the source artifact through the copied target link`.

## What this does not cover

- The in-tree layout's kill window.
  Where the worktree holds build state of its own,
  the clean still runs on the add's path,
  and a kill in the middle of it leaves some members cleaned and others not,
  with nothing recording that the clean was interrupted.
  Closing it means handing that clean to the service in the shape the prewarm filling already has
  (take the state aside under a name this job proves it owns,
  clean it there,
  publish it with a rename that never replaces),
  which is a change of the filling and not of this one.
- A bound on the metadata reads.
  They run under the repair tool's own 600-second window,
  because the fallback where no service answers keeps that window for its tools.
- The service's own cost.
  Moving the source read into the service would add 36 workspace resolutions per add to the queue
  that commits share,
  which is why it stays in the add.
