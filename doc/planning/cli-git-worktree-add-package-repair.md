# The package repair a worktree add runs, and what it costs the add

What:
 the three repairs an add runs once the new worktree is filled,
 the measured time each takes,
 and the two changes that stop the add waiting for the Cargo repair's clone and for the relinking
 of pnpm's and the toolchain's copied state.
Why:
 a traced add spent a third of its wall time in the repair,
 almost all of it inside one workspace's build-directory clone,
 while the two other repairs finished first,
 and a second trace showed 49.6 s of `pnpm install` and 11.8 s of `mpm-shadow install`
 (which runs `pnpm list --global` eleven times) inside the add.
Status:
 measured on 2026-11-11 from the owner's trace of 2026-11-10T18:00:41Z,
 changed on branch `fix/git-policy-cli-worktree-add-package-repair-off-the-waiting-path`
 and on branch `perf/git-policy-cli-worktree-add-package-tools-off-the-waiting-path`.
Future sessions read this before moving any of the three repairs,
and before measuring an add again.

## What the add runs after the copy

An add that took the snapshot rung fills the worktree,
copies the ignored files,
and then repairs the package managers of the new worktree
(`package/git-policy/cli/src/native/worktree_copying/worktree_copy_after_lock.rs:63-152`,
named in `doc/planning/cli-git-worktree-add-route-findings-on-main.md:256-263`).
The repairs are three waits of one task (`package/git-policy/cli/src/native/repair/package_repair.rs`),
and only the Cargo plan is the add's work:
the pnpm relink and the toolchain install are the commit service's
(section "The pnpm relink and the toolchain install are the service's work"),
and the Cargo clone and clean are the service's too (section "The change"):

- pnpm:
  `pnpm install --offline --frozen-lockfile` in the worktree
  (`package_repair_pnpm.rs:40`).
- toolchain:
  `mpm-shadow install --frozen-lockfile` in the worktree,
  which rewrites the launchers that still name another checkout's lock
  (`package_repair_toolchain.rs:5-6`,
  `:338`).
- Cargo:
  for every Cargo workspace of the worktree,
  `cargo metadata --locked --offline --no-deps` on the workspace and on its source counterpart,
  then `cargo clean --offline --package <member>` in the worktree
  (`package_repair_cargo.rs:173-180`,
  `package_repair_cargo_workspaces.rs:266`).

The `cargo clean` calls that the trace shows are the wrapper's own,
not `cargo-shadow`'s:
`cargo-shadow` treats `clean` as a command that resolves nothing and passes it through
(`package/cargo-shadow/cli/src/verb_tests.rs:39-50`),
while every `cargo` the repair starts is the toolchain's shadow first
(`.../toolchain-global/exposure/cargo`) and the real `cargo` after it.
The 105 `cargo` execve entries of the trace are 70 metadata runs and 35 clean runs,
each of them the shadow and the real binary.

## The time split of one traced add

The trace is `${HOME}/temp/agent/worktree-add-traces/add-2026-11-10T18-00-41-418Z.strace`,
`strace -f` of one `git worktree add --detach <dest> HEAD` through the installed wrapper
`0.6.1-alpha.8` under a host with about fifty agents working:
5,752,264 lines,
2,290,094 completed system calls,
490.5 s of wall time (`13:00:41.475` to `13:08:51.942`).
The untraced add earlier the same day took 79 s,
so the trace's own overhead is in every figure below.

The phases,
split at the events that start and end each one:

- Registration,
  12.7 s (`13:00:41.475` to `13:00:54.163`):
  2.0 s of system-call time.
- Filling the tree,
  272.4 s (`13:00:54.163` to `13:05:26.593`):
  114.4 s of system-call time,
  of which `unlinkat` 38.3 s over 37,677 calls,
  `statx` 32.0 s over 750,542 calls,
  `renameat2` 14.5 s over 515 calls,
  `getdents64` 7.3 s.
- After the fill,
  37.2 s (`13:05:26.593` to `13:06:03.832`):
  19.7 s of system-call time,
  of which one `fsync` of 9.1 s and one `recvfrom` of 5.4 s.
- The repairs,
  167.8 s (`13:06:03.832` to `13:08:51.654`):
  44.6 s of system-call time.
- The tail,
  7.9 s (`13:08:51.654` to `13:08:51.942`).

The three repairs ran beside each other and the add waited for the slowest:

- `pnpm install` 49.6 s (`13:06:03.832` to `13:06:53.401`).
- `mpm-shadow install` 11.8 s (`13:06:03.847` to `13:06:15.652`).
- The Cargo repair 167.8 s (`13:06:03.878` to `13:08:51.654`),
  so it alone set the repair's length:
  34 percent of the add.

Inside the Cargo repair,
one blocking-pool thread cloned one workspace's build directory from the main checkout's
`/var/mnt/encrypted/cargo-build/ae/b0b48f8399eb6a` into the worktree's `e6/e1a60d246f5b3a`:
3,215 `ioctl` calls with `FICLONE` (12.3 s blocked,
longest 6.7 s),
14.5 s in `openat`,
12.8 s in `statx`,
4,543 `chmod` calls,
spanning `13:06:51.252` to `13:08:42.797`,
which is 111.5 s,
or 66 percent of the Cargo repair and 23 percent of the add.
The other 34 workspaces were cleaned without a clone,
because the source checkout had no build directory for them.

The same shape is in the wrapper's own timing records,
which are not traced and so carry no trace overhead:
`doc/handover/cli-git-rust-implementation.md:4008-4037` reads 192 records of 2026-11-08 and finds
`cargoCloneTree` 66,305 ms over 34 clones,
`cargoClean` 34,029 ms over 34 workspaces,
`cargoRepairWorkspace` 108,436 ms over 35 workspaces,
and `repairPnpm` 6,440 ms,
with the repair 89 percent of the add at the median.

## What the repair exists to guarantee

The Cargo repair exists because Cargo bakes absolute paths into what it builds,
so artifacts built at the source checkout's path are wrong for the worktree's own crates,
while registry and git dependencies hold nothing path-specific and can be reused.
The owner chose that on 2026-11-03 as "Reuse deps,
rebuild own",
done with "Run cargo clean -p"
(`doc/handover/cli-git-rust-implementation.md:10195-10197`,
`doc/planning/lean-build-dirs.md:990-992`).
The toolchain repair exists because mpm-shadow's launcher records hold absolute paths,
and the owner accepted an online install during the add
(`doc/handover/cli-git-rust-implementation.md:10198`,
`doc/troubleshooting/mpm-shadow-record-copied-into-a-new-worktree-retires-the-source-links.md:142`).
The owner's speed bound is "60s is fine for adding worktrees"
(`doc/handover/cli-git-rust-implementation.md:10212-10215`),
with no target to stop at,
and dozens of agents at once are normal operation.

The repair is correct without its clone:
a worktree whose build directory is absent rebuilds its dependencies,
which is what 34 of the 35 workspaces already did.
So the clone adds speed to the worktree's first build and costs the add its longest step,
and the two are separable:
the add stays correct when the clone happens after it returns.

Two limits the build directory's layout puts on any replacement:

- `doc/decision/cargo-build-dir-host-cache.md:401-430` allows no reader of the build directory's
  layout except the `cargo-shadow lean` sweep,
  so no code may decide which entries belong to the worktree's own crates.
  The clean has to be Cargo's own command.
- `cargo metadata` reports the build and target directories of a workspace,
  so the wrapper reads no configuration,
  and `doc/decision/cargo-build-dir-host-cache.md:179-186` records that the hash keys the workspace
  root,
  which is why a new worktree starts with no build state at all.

A shared build directory is not a hazard here:
`plan_build_state` refuses a clean when the source and the worktree name one build directory
(`package_repair_cargo.rs:139-141`),
and the trace's cleans touched only the worktree's own hash
(`e6/e1a60d246f5b3a`),
never the main checkout's (`ae/b0b48f8399eb6a`).
So no add deleted an artifact another worktree still used,
and nothing of this change addresses that.

## The change

The Cargo repair no longer clones and cleans a workspace whose build state lives only in the
source checkout.
It writes one `prewarm` work item naming each such workspace (its `cargo metadata` directories and
its members) and wakes the commit service,
exactly as a snapshot's trash is handed over
(`worktree_trash_hand_over.rs:111-191`):

- `package/git-policy/cli/src/native/repair/package_repair_prewarm.rs`:
  the record,
its payload,
and the hand-over.
- `package/git-policy/cli/src/native/repair/package_repair_prewarm_job.rs`:
  the service's filling.
  Per workspace:
clone the source's build directory into `<destination>.monochromatic-prewarm`,
  run `cargo clean --package <members>` against that copy with `CARGO_BUILD_BUILD_DIR` and
  `CARGO_TARGET_DIR` pointing at directories of the job's own,
  then publish it with `renameat2(RENAME_NOREPLACE)`.
  A destination that exists is a build that got there first,
  and then nothing is published and the staging copy is removed.
- `WorkKind::Prewarm` in `work_item_kinds_and_their_names.rs`,
  drained before the trashes in `service_work_drain_pending_claims.rs`.

What stays on the add's waiting path is the plan:
`cargo metadata` for each workspace and its source counterpart,
which is the 70 metadata runs of the trace,
and the clean of a workspace whose build state the worktree already has.

## Measured before and after, on one fixture

The fixture is `${HOME}/temp/agent/prewarm-fixture/repo`:
one Git repository,
three Cargo workspaces (`package/alpha`,
`package/beta`,
`package/gamma`,
each a `[workspace]` with itself as its only member and depending on `serde` and `serde_json`),
and one pnpm project with a lockfile and a `node_modules`.
All three workspaces were built in the source checkout first,
so the repair had three external build directories to fill:
`d9/f195893300fe47`,
`9f/009d21f7be5cf2`,
`95/00340344269ece`.
Each arm is one `git worktree add --detach <dest> HEAD` through a binary built from this repository
with `cargo build --locked --offline --bin monochromatic-git-policy-cli` (debug profile),
run under `strace -f -c` with a fresh destination.
The before binary is the parent commit's tree,
extracted with `git archive` and built the same way.
The fixture is not a btrfs subvolume,
so every add took Git's checkout route and copied 27 ignored
entries,
in both arms.

The Cargo repair's own steps,
from the add's timing record
(`<git-common-dir>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`):

- Before,
  three runs:
  `cargoCloneTree` 12,758 ms,
12,780 ms and 3,932 ms over three clones,
  `cargoClean` 6,513 ms,
5,233 ms and 2,665 ms over three workspaces,
  `repairCargo` 7,641 ms,
6,923 ms and 2,662 ms.
- After,
  three runs:
  no clone and no clean at all,
  `cargoPrewarmHandOver` 102 ms,
106 ms and 72 ms,
  `repairCargo` 875 ms,
633 ms and 760 ms.

The system calls of the same six adds,
from `strace -f -c`:

- Before:
  110,621 calls in 20.18 s of system-call time,
  84,401 in 9.80 s,
  and 59,008 in 11.37 s.
- After:
  18,458 calls in 2.69 s,
  18,450 in 2.27 s,
  and 18,938 in 4.10 s.

So the add no longer makes the clone's and the clean's 40,000 to 92,000 system calls,
and the Cargo repair's own time falls from 2.7 to 7.6 s to 0.6 to 0.9 s.

Wall time cannot be attributed to this change on this host,
and this document claims no wall-time
saving from it:
the six adds took 48.18 s,
54.14 s and 13.28 s before and 4.33 s,
2.52 s and 38.62 s after,
and the spread comes from steps this change does not touch.
Two after runs show it:
`copyIgnored` was 1,225 ms in one and 22,757 ms in the other,
and the second add's 38.62 s carries a 760 ms Cargo repair.
Host load,
not the repair,
decides those seconds,
and one pair of runs says nothing about them (`NQH`,
`QNB`).

What this fixture cannot show is the filling itself:
no commit service can be started for a repository on this account
(the hand-over reports that,
and the record stays),
so no staging directory was made in either arm.
The filling is proven instead by
`repair::package_repair_prewarm_job::tests::the_filling_publishes_a_clean_copy_and_leaves_the_source_alone`
which runs the job's own code against a real Cargo workspace:
the copy is published under the worktree's build directory name,
the worktree's own crate is cleaned out of it,
and the source checkout's build directory is untouched.
That control checks both incremental settings,
because the container gate's image sets `CARGO_INCREMENTAL=0`
(`package/git-policy/cli/bin/native-test-containerfile.ts:110`)
and a developer's host has it on,
and a crate's state lives in different places under the two:
the first release of the control read `debug/incremental` alone,
passed on a host,
and failed the release gate on `main` at `cd9b8132f8`.

The records and counts of the six runs are in `${HOME}/temp/agent/prewarm-fixture/measure`.

## What the review of the hand-over found

Kimi K3.1 Pro reviewed the merged change at `aa7f601c6d` and wrote five findings
(`${HOME}/temp/agent/kimi-reviews/worktree-add-package-repair-handover-1/review.md`).
Each one was reproduced against this package's own code before anything changed.
The controls that show them fail on `fc750a295d` and pass on the commits after it.

### The service's clean rewrote `Cargo.lock`

Reproduced with the job's own `prewarm_awaiting` against a real Cargo workspace.
`cargo clean` resolves the worktree's workspace even though `CARGO_BUILD_BUILD_DIR` and
`CARGO_TARGET_DIR` name directories of this job's own,
so a lock that no longer matches its manifest is rewritten and a missing one is created,
after the add has returned and the person may be working in the worktree.
The first fix read the lock before the clean and put it back exactly as checked out,
through the same `package_repair_cargo_lock.rs` the add's own clean uses.
The second review showed the windows a put-back cannot close ("What the second review of the filling
found").
What stands now is `--locked`:
the clean refuses a lock it would have to change,
the worktree is never written,
and the report names the lock,
the workspace and the add's own clean command
(`repair::package_repair_prewarm_job::tests::the_filling_never_writes_the_worktrees_lock`).

### A directory at the staging name was removed without proof of ownership

`fill_one` ran `remove_dir_all` on `<destination>.monochromatic-prewarm` whenever the destination
was absent,
so a directory at that name that this job did not make was deleted.
The reviewer's own path to it,
a build directory named `<other>.monochromatic-prewarm`,
cannot happen:
`doc/decision/cargo-build-dir-host-cache.md` hashes the workspace root into
`<cache>/<two hex>/<fourteen hex>`,
and no name with a suffix can be one of those hashes.
The deletion itself is the defect,
so the names are now claimed by a marker this job writes exclusively before it clones,
and only names its own marker claims are removed.
A directory no marker claims is somebody's work:
the filling leaves it and stages under names of this run's own instead
(`repair::package_repair_prewarm_job::tests::a_directory_at_the_staging_name_that_this_job_did_not_make_is_never_removed`).

### The clean's target directory was left behind

`Filled::Published` returned before removing `<staging>.target`,
so a directory of private build state stayed behind for every build directory successfully filled.
The target directory is now given back whatever the filling came to
(`repair::package_repair_prewarm_job::tests::a_published_filling_leaves_no_target_directory_of_its_own`).

### The worktree-local Cargo fallback looked in the wrong directory

The fallback after the search path is the worktree's own `.cache/toolchain-bin/cargo`,
which sits at the worktree root,
and the filling looked for it under the record's first workspace root.
A record whose workspace is nested,
such as `package/example`,
found nothing there and failed the whole filling.
The lookup now starts from the record's repository,
which is the worktree root
(`repair::package_repair_prewarm_job::tests::the_worktrees_own_cargo_link_is_found_from_the_worktree_root_for_a_nested_workspace`).

### A worktree added again at one path was answered as already known

The record's object name was a digest of the worktree's path alone,
and the queue answers an id it already knows as known in every state.
A worktree removed and added again at the same path was therefore queued as known and never filled,
and a filling that failed could never be asked for again:
the service settles a failure into `failed` and reads no record back.
The object name is now a digest of the worktree's path and the millisecond the add asked,
so each hand-over is its own record.
A second record for one destination costs nothing when it runs:
the destination-exists check answers `Filled::Kept` before anything is cloned
(`repair::package_repair_prewarm::tests::a_worktree_added_again_at_the_same_path_gets_its_own_filling`).

### What the filling still cannot make safe

Codex reviewed the branch as well,
and named the windows that remain after these fixes.
Each one costs at most work this job made,
never a file a person wrote:

- Two fillings of one destination can run at once when a worktree is removed and added again at the
  same path while its first record is still queued.
  The second takes the names the first is working in,
  the loser reports a failure,
  and only directories this job made are removed.
- A fill that answers `Filled::Kept` because a build got there first leaves a killed fill's staging
  directory and its target where they are.
- A fill that ran under names of its own,
  because somebody else's file or directory held the usual ones,
  strands its staging copy if it is killed,
  since no later fill learns that name.

A per-directory claim would close the stranded staging copy,
and it costs more than it buys while one service instance claims a record once.
The two lock windows this list named are closed now,
because the filling's clean never writes the lock ("What the second review of the filling found",
`repair::package_repair_prewarm_job.rs`).

## What the second review of the filling found

Kimi K3.1 Pro reviewed the merged change at `5686c7bb29` and wrote five findings about the service's
filling (`${HOME}/temp/agent/kimi-reviews/package-repair-review-findings-round-133/review.md`).
The review's own reproductions were run here at `ee545033ad` and three of them failed:
a reader-less named pipe at the marker name stayed blocked for a second,
a person's `Cargo.lock` written while the clean ran came back as the checked-out bytes,
and a foreign file planted at the marker name was gone
(`${HOME}/temp/agent/pi-runs/prewarm-fix-run/base-evidence/review-reproductions-at-ee545033ad.log`).
This package's own controls for all five findings fail on that commit's product code and pass on
`1885e5cb6e`
(`${HOME}/temp/agent/pi-runs/prewarm-fix-run/base-evidence/this-branchs-controls-at-the-base-product.log`).

### A foreign named pipe at the marker name held the filling for ever

`marker_claims` read the marker with `std::fs::read`,
which waits inside `open` for a writer that never comes when the name holds a named pipe,
so one object anyone can create at a path this job already tolerates held the filling for ever,
against "nothing should ever wait forever or indefinitely".
The read now goes through `private_storage::read_regular_file`,
whose open is `O_NONBLOCK | O_NOFOLLOW` and whose descriptor has to prove a regular file,
so a pipe,
a socket,
a directory or a link is somebody else's and answers at once
(`repair::package_repair_prewarm_job::tests::a_named_pipe_at_the_marker_name_never_holds_the_filling`).

### The clean wrote the worktree's `Cargo.lock`

`cargo clean` resolves the worktree's workspace,
so it rewrote a lock that no longer matched its manifest and created one that was missing.
The put-back compared only the bytes read before the clean with the bytes found after it,
which cannot tell Cargo's write from a person's own:
a `Cargo.lock` written while the clean ran came back as the bytes read before it
(`checked out` where the control wrote the person's update),
and a kill between the rewrite and the put-back left the rewritten bytes,
because the original sat in memory.
No restore can tell the two writers apart,
so the filling no longer writes the lock at all:
`clean_arguments_of_the_copy` passes `--locked`,
and `cargo clean --locked` exits 101 in Cargo's own words
("cannot update the lock file ... because --locked was passed to prevent this")
with the file untouched,
which was measured for a matching,
a stale and an absent lock.
The report names the lock,
the workspace,
`cargo update --offline --workspace` and the clean the add would have run
(`what_to_do_next` in `package_repair_prewarm_job.rs`).
The add's own clean keeps its put-back,
because it runs where the worktree was just made and nobody has it yet
(`SPEC.md`,
"A committed `Cargo.lock` that disagrees with its manifest").
The cost is the filling itself where a lock disagrees with its manifest:
at `1885e5cb6e` the full resolution of all 35 committed workspaces
(`cargo metadata --locked --offline`) succeeds,
so nothing here costs that today,
and the shape that does cost it is a manifest edited ahead of its lock
(`repair::package_repair_prewarm_job::tests::the_filling_never_writes_the_worktrees_lock`,
`the_filling_never_writes_a_lock_a_person_updates_while_the_clean_runs`).

### A foreign file that won the marker race was removed

`write_marker` answered every `AlreadyExists` as success and `give_back` removes the marker
unconditionally,
so a file another command created between this run's look at the name and this run's exclusive
write was deleted,
although this run never wrote it.
The exclusive write now answers whether this run created the marker (`TheMarkersAnswer`),
and a name another writer holds makes this run stage under names of its own,
so it has no marker to remove
(`repair::package_repair_prewarm_job::tests::a_file_that_wins_the_race_to_the_marker_name_is_never_removed`).

### Two hand-overs in one millisecond shared one object name

The record's object name is a digest of the worktree's path and the moment the add asked,
and the moment was a millisecond,
so a second hand-over of one path inside one millisecond took the first one's name;
the queue answers an id it already knows as known in every state,
so its filling was never queued.
`iso_time::now_iso_never_the_same_twice` keeps the last millisecond it wrote and steps past it,
so a stamp is a moment this process has not written before and never one before the clock
(`repair::package_repair_prewarm::tests::every_hand_over_of_one_path_gets_an_object_name_of_its_own`,
whose control counted 24 object names for 1000 hand-overs of one path before the change).

## The pnpm relink and the toolchain install are the service's work

What:
 the two repairs that rewrite a copied layout in place,
 what the add still does for them,
 and what one measured pair of adds says they cost.
Why:
 neither run reads the worktree's files.
`pnpm install --offline --frozen-lockfile` relinks `node_modules` for the new depth,
`mpm-shadow install --frozen-lockfile` rebuilds the toolchain links and launchers for the new
 checkout,
and each is a whole program run through the shadow of the tool it repairs,
so an add that waits for them queues behind every other add on the same disk.
The owner's trace of the installed release `0.6.1-alpha.8`
(2026-11-10T18:00:41Z,
section "The time split of one traced add" of this document)
puts 49.6 s in `pnpm install`,
11.8 s in `mpm-shadow install`,
and eleven `pnpm list --global --depth=0` runs inside that install,
all on the add's waiting path.

### What the add does now

The add still makes both decisions,
because a worktree that needs nothing must leave no record for the service,
and it makes them from cheap reads:

- pnpm:
  a lockfile and a `node_modules` in the worktree
  (the `need_of` function of `package_repair_pnpm.rs`).
- toolchain:
  a launcher record under `.cache/toolchain-launchers` that names another lock,
  read as a job of the process's blocking pool
  (the `need_within_the_budget` function of `package_repair_toolchain.rs`).

It then writes one `relink` work item (`package_repair_relink.rs`),
whose payload is `{"root": "<worktree>", "tools": ["pnpm", "toolchain"]}`,
under the repository's common directory,
keyed by a digest of the worktree's path,
so two worktrees of one commit get one record each.
Where a commit service answers,
the add wakes it and prints one line naming what was handed over.
The service runs `relink` items before a prewarm and before a trash
(`service_work_drain_pending_claims.rs`),
runs each tool's own command in the worktree,
with the same quiet window and ceiling the add gave it
(`package_repair_relink_job.rs`),
and leaves one line per attempt in its journal.

What stays on the waiting path,
and why:

- The two decisions above,
  which are a few `stat` calls and one read of the launcher directory,
  and are what keep a worktree that needs nothing from leaving a record.
- The record write and the wake,
  timed as `relinkHandOver` in the add's own record.
- The two tools themselves,
  when no commit service can be reached,
  as in a session with no user service manager such as a container.
  A worktree whose copied `node_modules` was never relinked
  resolves the source checkout's packages without an error,
  and one whose launchers were never rebuilt runs the source checkout's tool versions,
  so in that one case the add does what every release before this one did,
  and the line it prints names the command that does the same work by hand.
  The record stays,
  so a service that starts later repeats a relink that changes nothing.

### Measured on a fixture, before and after

The fixture is `${HOME}/temp/agent/git-policy-add-package-tools`:
one Git repository of four tracked files,
with `pnpm-lock.yaml` and a `node_modules` of two packages (6.4 MB),
`.cache` ignored,
and no tracked `Cargo.toml`,
so the Cargo lane of the repair only lists tracked manifests.
The fixture is not a btrfs subvolume,
so every add took Git's checkout route and copied 2,339 ignored entries.
The driver is `measure.ts` in that folder:
one `git worktree add --detach <destination> HEAD` per arm,
alternating before and after,
a fresh destination each time,
the fixture's own `HOME` and `XDG_RUNTIME_DIR`,
a `git` of the arm on `PATH`,
and the add's stderr and its own timing record kept per run.

- Before:
  a binary built from the branch's first commit,
  extracted with `git archive`,
  and built with
  `cargo build --locked --offline --release --bin monochromatic-git-policy-cli`.
- After:
  this branch's binary,
  built with `mise run //package/git-policy/cli:native:build:release`.
- The service:
  this branch's binary in its own mode,
  `monochromatic-git-policy-cli service run --journal <folder> --path <socket>`,
  with the fixture's `HOME` and runtime directory,
  and no session bus.

The adds,
from the driver's log and each add's timing record
(the record keeps milliseconds;
these are printed to the second):

- Before,
  `2026-11-10T23:11:12Z` to `2026-11-10T23:13:23Z`,
  exit 0:
  `repairPnpm` 1,130 ms,
  `repairToolchain` 13 ms,
  `repairCargo` 16 ms.
- After,
  `2026-11-10T23:11:07Z` to `2026-11-10T23:11:12Z`,
  exit 0:
  `relinkHandOver` 28 ms,
  no `repairPnpm` and no `repairToolchain` at all,
  and the relink record was in `pending` when the add returned,
  and gone at the driver's next look.
- Before,
  `2026-11-10T23:15:09Z` to `2026-11-10T23:17:17Z`,
  exit 0:
  `repairPnpm` 72,426 ms,
  `repairToolchain` 1 ms,
  `repairCargo` 2 ms.
- After,
  `2026-11-10T23:13:23Z` to `2026-11-10T23:15:09Z`,
  exit 0:
  `relinkHandOver` 216 ms,
  no `repairPnpm` and no `repairToolchain`.
- After,
  `2026-11-10T23:17:17Z` to `2026-11-10T23:18:37Z`,
  exit 0:
  `relinkHandOver` 226 ms,
  no `repairPnpm` and no `repairToolchain`.

So the add's own work for the two tools falls from 1,130 ms and 72,426 ms
to a record and a wake of 28 ms in one run and 216 and 226 ms in the other two,
and the two tools run in the service instead.
The two before adds differ by 71 s in the same step,
which is this host and not the change,
as rules `NQH` and `QNB` say,
so no wall-time saving is claimed here:
the whole-add times of the same runs were 131 s and 128 s before,
and 5 s and 106 s and 80 s after,
and `copyIgnored` alone was 127,658 ms in one of them.
The first before add of the run is left out:
it took a recovering rung of its own,
which its record names as `claimStage` and `releaseClaim`,
copied nothing,
and so started no repair at all.

What this fixture cannot show:

- The toolchain install itself:
  `mpm-shadow install` in a worktree writes into the account's tool store,
  which is not a disposable thing to run extra installs against
  while other checkouts read it,
  so the fixture repository carries no `.cache/toolchain-launchers`,
  and the toolchain lane only reads there.
  The 11.8 s and the eleven `pnpm list` calls are therefore the owner's trace
  of the installed release,
  and the service's run of the same command is proven by
  `repair::package_repair_relink_job::tests::both_tools_run_in_the_worktree_in_the_records_order`
  and
  `repair::package_repair_relink_job::tests::a_tool_that_fails_is_named_and_the_next_still_runs`.
- The pnpm relink at this repository's size:
  the fixture's `node_modules` holds two packages,
  so its 1,130 ms is a small checkout's cost,
  not the 49.6 s the owner's monorepo trace shows.
- The time the service takes to relink one worktree:
  the fixture's service was too busy with other worktrees to read its journal here,
  and the driver only observed that the record left `pending`.

The runs,
their stderr and their timing records are in
`${HOME}/temp/agent/git-policy-add-package-tools/measure`.
