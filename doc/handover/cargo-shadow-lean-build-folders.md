# cargo-shadow lean build folders

Open work and current state of the lean-build-folders plan
(`doc/planning/lean-build-dirs.md`,
owner decision of 2026-11-07,
"A to F,
B allowed (Recommended)"),
built in `package/cargo-shadow/cli` and released as cargo-shadow 0.1.5-alpha.5.
Written 2026-11-07T23:55:00-05:00;
the first run was still going,
so the numbers of what it freed are in the section "First run" only as far as they were known.
Updated 2026-11-08T15:42:45-05:00:
what the run takes next (the cheap mechanisms first,
the units pass last),
the three Podman refusals the first run met,
and what of each is still unmeasured.
Both changes are in the sources only,
so nothing of this is installed or released yet.
Updated 2026-11-08T16:18:47-05:00:
D is wired to its five producers through a new scoped entry point (`lean records`),
and the Cargo repair without the shadow and the two open questions of the plan are measured,
with their answers below.
All of this was in the sources only at that time.
The version installed then was 0.1.5-alpha.7,
which lists only `lean run` and `lean timer-install`,
so a producing task stopped at its first step until the release of 0.1.5-alpha.8,
which `toolchain.lock.json` now locks and `mise run toolchain:install` linked in.
Updated 2026-11-08T18:18:16-05:00:
the name list of mechanism D now names every program of this repository that writes a record,
the retention is wired on the seventeen sibling producers that write records of their own,
and the process scan classifies what it cannot read instead of calling it another user's.
The installed 0.1.5-alpha.8 has `lean records`,
so the wiring works today;
the widened name list is in the sources only and needs a release and `mise run toolchain:install`
before the tasks sweep the names it adds.

## State

- Updated 2026-11-10T17:30:00-05:00 (branch `fix/cargo-shadow-lean-read-only-trees-and-run-deadline`,
  not released and not installed):
  the two long passes continue after the position each records,
  a claimed entry holding a directory this user may not write is now removed whole and says so,
  the Git wrapper's fixtures no longer leave read-only directories in a build tree,
  and `RUN_DEADLINE_SECONDS` is set from a measurement of this machine.
- The six mechanisms,
  the dated-nightly retirement and the Podman cleanup are merged to `main` (merge commit
  "Merge branch 'feat/cargo-shadow-lean-build-folders'") and released:
  GitHub release `monochromatic-cargo-shadow-v0.1.5-alpha.5`,
  built from the release commit `a018a85ade`.
  The crates.io check was skipped by the release tooling (the alpha label is not published there).
  `toolchain.lock.json` locks it (only that row moved) and `mise run toolchain:install` linked it into
  `.cache/toolchain-bin/cargo-shadow`.
- The decision record `doc/decision/cargo-build-dir-host-cache.md` has the dated amendment of 2026-11-07T22:19:43-05:00
  that lets the sweep read fingerprint files.
  No other decision record was edited.
- Gates at release time:
  `mise run //package/cargo-shadow/cli:lint` (cargo check) clean,
  `lint:clippy` clean,
  `test` 785 passed and 18 skipped (the package has no container suite),
  and `monochromatic-lint` clean on the lean sources.
- Each guard was shown to fail when removed:
  a held build lock,
  a process using an entry,
  a swapped entry's identity (before the move and before the unlink),
  a pinned toolchain,
  an image a container uses,
  and a workspace that still exists.

## How to run it

```sh
# doc/handover/cargo-shadow-lean-build-folders.md
.cache/toolchain-bin/cargo-shadow lean run --dry-run
.cache/toolchain-bin/cargo-shadow lean run --only trees,units
.cache/toolchain-bin/cargo-shadow lean timer-install
```

Run a long pass in its own named systemd user unit with `CPUQuota`,
`MemoryMax`,
`TasksMax` and `IOWeight` of at most 10,
and give `--deadline-minutes`:
nothing in the tool waits without end,
and what a deadline cuts off waits for the next run.
Always point `CARGO_HOME` at an empty directory when running the binary against a fixture:
with the real one it reads the real `build.build-dir` and sweeps the real host build base.

## Open

- D at its producers:
  wired on 2026-11-08 in the five producing packages,
  and not yet released.
  The retention still runs from `cargo-shadow lean run` over every checkout,
  and also from `cargo-shadow lean records <directory>...` over the directories one producing task names
  (`package/cargo-shadow/cli/src/lean_command.rs`,
  section "Keeping the build folders lean" of the crate README).
  Each of these tasks runs that first,
  over its own `target/verification`:
  the original eight:
  `native:test:container` and `native:mutation` of `package/git-policy/cli`,
  `smoke` and `test:planted` of `package/git-policy/cli.fuzz`,
  `test:container` of `package/linter/monochromatic-lint`,
  `smoke` and `test:planted` of `package/linter/monochromatic-lint.fuzz`,
  and `test:container` and `test:mutation:container` of `package/forbidden-strings/cli`;
  and the seventeen added on 2026-11-08T18:18:16-05:00:
  `test:git-image`,
  `native:mutation:scoped` and `native:mutation:planted` of `package/git-policy/cli`,
  `build` of both fuzz packages,
  `mutation:rust-style`,
  `mutation:processors` and `mutation:coverage` of `package/linter/monochromatic-lint`,
  `lint:clippy:container`,
  `test:release:container`,
  `test:guards:container`,
  `test:guards:observe:container`,
  `test:mutation:startup:container` and `test:mutation:pathname:container` of `package/forbidden-strings/cli`,
  `lint:clippy:container` and `smoke:embedding:container` of `package/forbidden-strings/cli.fuzz`,
  and `test:mutation` of `package/forbidden-strings/core`.
  The last package is not in the plan's list of producing packages and writes `mutation-*` records anyway.
  What a task a wired task starts must not do is sweep a second time,
  and none of the seventeen is started by a wired task:
  `native:mutation:parallel` starts units of `native:mutation:scoped` and `native:mutation:planted` and writes no record itself,
  so it is not wired and each unit sweeps once;
  the twenty `mutation:*` tasks of the linter and the scanner that depend on a wired `test:container` are not wired either.
  `package/cargo-shadow/cli/src/lean_records_wiring_tests.rs` holds the ledger of the sixteen programs that write records,
  with each script and prefix read from the program,
  and fails from a task file alone when a producing task stops sweeping;
  the ledger is kept by hand,
  so a program that starts writing records is a ledger line in the same change;
  `--list` runs are recorded as writing nothing.
  The plan assigned D to those packages;
  the owner's answer put all six mechanisms in cargo-shadow,
  so the tasks call its retention rather than reimplementing it.
  Why a scoped entry point and not the whole pass:
  a machine-wide dry run of the records mechanism alone measured 398 s on this host on 2026-11-08
  (359 checkouts,
  119 verification directories read,
  nothing due for removal),
  which is too much in front of every producing task.
  Not measured:
  what the scoped sweep costs on a package whose records are due,
  and what each of these tasks now takes.
  The name list is no longer narrower than the producers:
  `RECORD_PREFIXES` in `lean_run_records.rs` carries nineteen prefixes,
  each beside the program that writes it,
  and `every_name_a_producer_in_this_repository_writes_is_a_run_record` fails when a name a program writes is missing.
  Two prefixes of the nineteen are written by no current program:
  `container-` (five records of 2026-10-04) and `windows-` (three of 2026-10-05),
  which the scanner's snapshot producer wrote before it took a name per run,
  so they go with the rest.
  What is still never swept,
  measured 2026-11-08 in `package/linter/monochromatic-lint/target/verification`
  (33 entries and 32 MiB):
  eighteen logs a person or an agent redirected there by hand
  (`gate-*`,
  `hand-mutations-*`,
  `stack-repro-*`,
  `workspace-*`;
  520 KiB together),
  one record a hand renamed `processor-survivors-mutation-w00nRd` (17.9 MB),
  a cache directory `nextest-cache` (9.8 MB),
  and the two rendered files (`processors-handover.html`,
  `processors-manifest.json`).
  No program here writes those names,
  so no list can recognise them;
  the README says so beside the list,
  and the record a hand renamed is the coordinator's call,
  not the retention's.
  The wiring works against the installed 0.1.5-alpha.8,
  which has `lean records`;
  the widened name list needs a release and `mise run toolchain:install` before it sweeps anything.
  No producing task was run while making this change.
- A for container builds:
  `package/ide/app/mise.toml` and `package/nested-wayland-session/cli/mise.toml` do not set the keyed folder.
  Their in-tree builds are kept lean by B and E only.
- The Git wrapper's Cargo repair clones whichever build directory `cargo metadata` reports
  (`package/git-policy/cli/src/native/repair/package_repair_cargo_workspaces.rs`).
  Measured 2026-11-08 in this worktree's `package/cargo-shadow/cli`,
  with `cargo metadata --no-deps --locked --offline`:
  with the shadow on `PATH` the reported `build_directory` is the keyed folder
  `/var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-08/9f/66d64bfdfa2aab`
  (929 MiB,
  its ledger naming this worktree),
  and with the shadow off `PATH` (the real cargo of `.cache/toolchain-bin`) it is Cargo's own hash folder
  `/var/mnt/encrypted/cargo-build/fb/b51766dd97b6cf`,
  which held 2.0 KiB (a `CACHEDIR.TAG` and an empty `debug/`).
  Both sides of a worktree add are read with the same `cargo` from the person's `PATH`
  (`read_workspace` for the worktree,
  `cargoMetadataSource` for the source),
  so without the shadow the repair clones a hash folder into a hash folder.
  For this workspace the hash side held 2 KiB,
  which is nothing to carry,
  and only the keyed side (929 MiB) is what a shadow build reads;
  another workspace's hash folder could hold real state,
  which would then be carried,
  but the shadow still reads the keyed folder.
  So what the repair cannot do without the shadow is carry the source checkout's keyed state,
  and no build state at all in the case measured.
  The measurement left that 2 KiB hash folder in the real build base;
  nothing was built into it and nothing else was touched.
  Re-measured read-only 2026-11-08T18:18:16-05:00,
  and it is still there:
  `/var/mnt/encrypted/cargo-build/fb/b51766dd97b6cf` is 1,994 bytes apparent (8 KiB of blocks) and holds
  `CACHEDIR.TAG` (177 bytes),
  `debug/.cargo-build-lock` (0 bytes) and `.rustc_info.json` (1,817 bytes,
  naming `/var/home/user/.rustup/toolchains/nightly-2026-11-08-x86_64-unknown-linux-gnu`),
  with no fingerprint,
  no dep-info and no compiled file,
  so no build reached a unit.
  The directory,
  its tag and its lock are stamped 2026-11-08T15:56:02 and `.rustc_info.json` 2026-11-08T16:06:46,
  so more than one cargo invocation used it.
  It is one of a series:
  eight folders of the same three entries and 1,953 to 1,995 bytes were made between 15:53:21 and 16:23:19 that day
  (`85/524c90af92bb3d`,
  `85/726a528cfb25ad`,
  `c0/7b9d5f99c8eb7a`,
  `e9/09a5f691e5af1d`,
  `e9/6a8972d5d8e733`,
  `20/a8ca6a487f0070`,
  `64/0be5ee5f2ae620`,
  `64/d0b0a2854e1dde`),
  and one real build of 1.3 GB beside them (`93/347f88e48dce9e`).
  The command this bullet names is not its maker:
  measured 2026-11-08T18:18:16-05:00 in a disposable `CARGO_HOME` whose `config.toml` carries the same
  `build-dir = <base>/{workspace-path-hash}`,
  `cargo metadata --no-deps --locked --offline` creates no build folder at all
  (it reports the path),
  and neither metadata with dependencies,
  `cargo check` nor `cargo tree` reach the layout (they stop at resolution).
  A `debug/.cargo-build-lock` says a command took that profile directory's build lock,
  which metadata never does,
  so the maker was another real-cargo invocation of the same toolchain,
  and which one is not established.
  The workspace it belongs to is not established either:
  the folder holds no dep-info or fingerprint,
  and the hash is not invertible;
  matching it again would need that workspace's exact path.
  Nothing was created or removed under `/var/mnt/encrypted/cargo-build` while measuring this,
  and the folder was not deleted.
  A change would need the repair to ask for the keyed folder,
  which means the Git wrapper carrying cargo-shadow's keying rule;
  that crosses the boundary that keeps the rule in cargo-shadow,
  so it is named here for the coordinator and the owner and not proposed as this branch's work.
  The alternative,
  carrying whichever of the two build folders holds newer state,
  keeps the rule where it is and still needs an owner decision.
- Containers in any state use their image:
  an image an exited container holds is kept.
  Whether exited containers should go too is a question for the owner.
- What runs `cargo package` or `cargo publish --dry-run` about 90 times per crate
  (the plan's first open question):
  measured,
  and the "about 90 times per crate" figure is not reproducible as a run count.
  What a run leaves is exact:
  `cargo package` verifies by unpacking the crate and building it.
  Measured 2026-11-08 in a disposable crate with `CARGO_BUILD_BUILD_DIR=<base>/{workspace-path-hash}`,
  the same shape as this host's `~/.cargo/config.toml`:
  it compiled
  `<base>/75/4a9477468c27be/package/tiny-probe-0.1.0`,
  so the unpacked copy lives inside the workspace's own build tree and no new tree is made.
  The hash-shaped trees of the host build base hold 2,131 `package/tmp-crate` directories
  and 6,763 unpacked crate directories over 45 distinct crate versions
  (`find /var/mnt/encrypted/cargo-build -maxdepth 4 -mindepth 4 -type d -path '*/package/*'`);
  25 of those versions are `monochromatic-git-policy-cli`,
  which has 70 distinct versions in Git history and 69 release tags.
  The order-of-magnitude figure is therefore somewhere between the versions of a crate that were packaged
  and the trees those copies reached;
  which of the two the "about 90" counted is not established,
  and neither is a loop that runs 90 times.
  The copies reach many trees because the Cargo repair clones a workspace's build tree into each new worktree,
  which is how 25 versions became 4,165 `monochromatic-git-policy-cli-*` directories.
  Who runs it,
  counted in the files rather than observed in a log:
  `.github/workflows/cargo-publish.yml` has 16 `run: cargo package` steps and 16 `run: cargo publish --dry-run` steps,
  one of each per crate job,
  and every one of the dry runs there passes `--no-verify`,
  so the workflow does not verify twice;
  `package/crate-release/cli` runs one `cargo publish --dry-run --locked --allow-dirty` per release,
  without `--no-verify`,
  so that one does verify;
  and each crate has a manual `package` and `publish:dry-run` task.
  Whether it should stop at the source is the owner's call,
  and it has two verifies to judge:
  the workflow's `cargo package` step and the release preflight's `cargo publish --dry-run`.
  Both compile the packaged crate,
  which is the check that a `--no-verify` would remove.
  What can stop at the source without losing either check is the copy:
  the build trees these runs fill are cloned into every new worktree by the Cargo repair,
  and nothing skips `package/` in a tree that is cloned.
- Which wrapper build an add uses
  (the plan's second open question):
  the installed and locked one,
  `monochromatic-git-policy-cli` 0.6.1-alpha.4
  (`toolchain.lock.json`,
  route `archive`;
  the launcher record `.cache/toolchain-launchers/monochromatic-git-policy-cli.launch` names
  `.../toolchain/cargo-monochromatic-git-policy-cli/current/monochromatic-git-policy-cli-0.6.1-alpha.4-x86_64-unknown-linux-gnu/monochromatic-git-policy-cli`,
  which is there and executable).
  That build carries the whole copy rule:
  the skip of `target/**/incremental`,
  `target/verification` and the caches entered at version 0.5.14
  (`5e773a62f4`,
  2026-11-02)
  and first shipped in `monochromatic-git-policy-cli-v0.5.18`.
  The brief's 49.5 GB per add is within a percent of its listed total minus incremental state only (49.2 GB),
  which is what a build before 0.5.18 would have written;
  a build carrying this rule would also have left out `target/verification` (6.2 GiB),
  so about 43 GB.
  That subtraction compares two measurements of different things and settles nothing on its own.
  Which build ran for that measurement is still not established;
  what a new add uses is 0.6.1-alpha.4.
- The proof of "not in use" has a stated limit,
  and the scoped sweep inherits it unchanged.
  `scan_processes` cannot read the entries of every process,
  and `blocks` consults only the uses it found
  (`package/cargo-shadow/cli/src/lean_proc_scan.rs`),
  so a record such a process holds can be removed.
  Until 2026-11-08T18:18:16-05:00 every one of those was counted as "unreadable (other users)",
  which was wrong twice:
  the sweep of one package's directory printed
  "426 processes read,
  681 unreadable (other users),
  58 in other mount namespaces judged by identity,
  0 uses under the root",
  and most of those were kernel threads.
  The scan now reads `/proc/<pid>/status`,
  which every process shows,
  and the removal's proof line names three parts:
  kernel threads (`Kthread: 1`) and ended processes (`State: Z`),
  which hold no file;
  the uninspectable processes of another user that `other_uids_can_reach` shows the path lets in;
  and the uninspectable ones of this user or of root,
  whose descriptors the kernel hides from a plain user.
  Measured on this host 2026-11-08T17:59:37-05:00 over 1028 processes:
  372 were read and 656 had no readable working directory,
  and the scan called all 656 "other users",
  although 11 of them ran as this user.
  The same count at 18:10 over 1071 processes,
  this time reading `/proc/<pid>/status`:
  385 read,
  608 kernel threads,
  2 ended processes,
  and 76 uninspectable,
  of which 65 of this user or of root
  and 11 of other uids (avahi,
  dbus,
  systemd-oomd and other daemons).
  The figures move with the machine,
  so these are two readings minutes apart and not one rate.
  A real sweep of a disposable directory printed,
  on the same build:
  "366 processes read,
  630 kernel threads and ended processes (which hold no file),
  74 uninspectable (0 of another user reaching a root,
  74 of this user or of root that no plain user may look into),
  59 in other mount namespaces judged by identity,
  0 uses under the roots".
  What is closed:
  a kernel thread and an ended process were counted as possible holders and are not.
  What is not,
  and cannot be by this tool:
  a live process the kernel hides still could hold the record.
  Closing that needs `CAP_SYS_PTRACE` or root,
  which cargo-shadow never takes.
  The nearest thing available,
  refusing every removal while an uninspectable process of another user could reach the entry,
  was measured and not taken:
  the host build base sits on a `0777` mount with a `uid 70` daemon always running,
  so it would stop the sweep of every tree there
  (a first attempt did stop the idle-tree tests,
  whose fixtures live in that base,
  blocked by pid 3032).
  That reachability fact is still measured and named in the proof line,
  so whether a record held by another user's process may go stays the owner's call,
  now with the count and the kind of each process beside it.
- The timer:
  `cargo-shadow lean timer-install` writes `cargo-shadow-lean.service` and `cargo-shadow-lean.timer` to
  `~/.config/systemd/user` and enables the timer.
  It fires once a day at 04:30 local time and at no other moment (`Persistent=false`);
  the service has no `[Install]` section.
  See "First run" for whether it was installed.

- Deviation from the owner's words,
  veto open:
  the owner allowed reading fingerprint files "for that sweep only" (B).
  The amendment of `doc/decision/cargo-build-dir-host-cache.md` extends the same reader to the size caps (E),
  which evict by the same units and times.
  That extension is a coordinator reading,
  not the owner's sentence.
- Legacy trees of live workspaces stay for good.
  A moved every shadow-run build to `<base>/toolchain/<toolchain>/...`,
  so the old `<base>/<xx>/<hash>` tree of a live worktree has no ledger and a dependency lead that exists,
  and mechanism C keeps it.
  B and E shrink it to the cap;
  nothing removes it.
  Most of the 6884 trees the first pass kept are these.
  This needs an owner decision:
  retire a legacy tree after the idle period once a keyed folder exists for the same workspace.
- Every host workspace's next shadow build starts cold at once
  (the keyed folder is new),
  not per toolchain change as the plan framed it.
- The run's order and the three Podman refusals are fixed in the sources,
  not released and not installed (2026-11-08,
  branch `fix/cargo-shadow-lean-pass-order-and-podman-refusals`):
  `lean run` takes the cheapest mechanism first and the units pass last (run records,
  toolchains,
  Podman,
  idle trees,
  then the older-compiler sweep and the size caps).
  The order rests on the first run's timestamps:
  2.5 minutes for records and toolchains together,
  20.5 minutes for Podman,
  29 minutes for the idle trees,
  and hours for the units pass,
  which that run stopped 40 minutes in after about 700 of several thousand profile directories.
  A budget per mechanism is still open work;
  what replaces the starvation is the position each long pass records
  (see the point below on where a run continues).
  Until a release and `mise run toolchain:install`,
  the timer keeps running the installed 0.1.5-alpha.7 and its old order.
- Where a run continues,
  in the sources only (2026-11-10,
  branch `fix/cargo-shadow-lean-read-only-trees-and-run-deadline`):
  the idle-trees pass and the units pass record the last tree and the last profile directory they judged
  in `lean-progress` under `$XDG_STATE_HOME/cargo-shadow`,
  or under `$HOME/.local/state/cargo-shadow` when the variable is unset,
  written by an atomic rename,
  and the next run starts after that key.
  A pass that finishes its list leaves no key,
  so the run after it starts at the front;
  a missing,
  unreadable or unknown-shaped record means the front and never a failure,
  and the run says which it found.
  A dry run writes nothing.
  Both keys are paths,
  not counts,
  so a tree the same pass removed still orders the list.
  Why this matters:
  the journal of the run that ended 2026-11-10T07:37:24-05:00 shows the idle-trees pass spending the whole bound
  (`idle trees: 33804 trees read; removed 278 (511.49 GiB by disk blocks); kept 14572; skipped 18954` at 07:30:33,
  the deadline having passed inside the `b3/` and `b4/` prefixes),
  after which the units pass reached no profile directory at all
  (`older-compiler sweep: 46529 profile directories read; removed 0 units ...` and
  `the deadline passed before every profile directory was reached`).
  Every run started those two passes at the front,
  so the trees and profile directories past the bound were never reached.
  Measured times,
  `RUN_DEADLINE_SECONDS` and the end time are in the README section
  "The order a run takes its mechanisms in";
  the numbers are also below under "Measured 2026-11-10".
- The read-only directory that stopped a removal,
  answered in the same branch (issue 6282):
  a directory inside a claimed entry that this user owns and may not write (`0500`,
  `0555`,
  `0000`) gets this user's read,
  write and search before the walk removes what it holds,
  the removal's line names every directory whose mode it changed,
  a directory of another user is left as it is and named,
  and a tree that could not be emptied whole is reported as partly removed,
  never as skipped.
  The 25 read-only directories in the base's trash on 2026-11-10 all came from this repository's own fixtures
  (`native-unit-*-rollback-read-only`,
  `remove-old-read-only`,
  `migrate-unreadable-dir`,
  `sync-unreadable-dir`),
  whose teardown now gives the permission back before it removes the tree
  (commit of the same day,
  `fix(git-policy-cli)`).
  Measured on disposable fixture roots:
  with the teardown as it was,
  the four controls fail at `remove only the fixture: Os { code: 13 }`
  and leave seven such directories behind;
  with it,
  `find <fixture root> -type d ! -perm -u=w` prints nothing.
- Podman removed 251 of 1,601 judged on the first run.
  The three refusals its errors showed are answered in the same branch:
  the container list asks for `--external`,
  because a Buildah working container holds its image and is invisible to `podman ps --all`;
  an image is asked for by every `repository:tag` it carries,
  because `podman rmi <id>` refuses an image with more than one tag;
  and a refusal that says `image not known` counts the image as gone.
  Measured 2026-11-08 in a throwaway storage root (`~/temp/agent/podman-probe`,
  podman 6.1.3,
  buildah 1.45.1,
  no real storage touched):
  `ps --all --external` lists both the external and the ordinary containers,
  `rmi <id>` refuses a two-tag image while `rmi <tag> <tag>` untags both and deletes it,
  and removing an image also deletes an ancestor that only that image referenced,
  which is the direction the first run's `image not known` errors came from.
  Unmeasured:
  how many of the 1,601 a fixed run removes,
  and the time a per-image `podman rmi` costs against the old batches of 25.
  Nothing is forced,
  and no image,
  volume,
  container,
  build folder or timer was touched while fixing this.

## Timer off until the moved checkouts are checked, recorded 2026-11-08T12:43:37-05:00

`cargo-shadow-lean.timer` was paused for the 860 move.
When the coordinator enabled it again at 2026-11-08T12:43:37-05:00 it fired at once,
although its calendar time is 04:30 and it has `Persistent=false`;
why is not established.
The run printed `lean run: 339 checkouts, removing`,
and the coordinator stopped it after 44 seconds,
because the checkouts had moved behind a link that morning and no lean run had been checked against that.
The journal holds no removal line for that run,
and removal lines stream as they happen since 0.1.5-alpha.6;
the service ended with `Result=signal`.

The timer is disabled again.
Before enabling it,
read the dry run `cargo-shadow-lean-dry-run-after-860-move` (`--dry-run --only trees`,
in the user journal),
confirm it keeps every live checkout's folders,
and wait until the cut-over stages from `c22-verify-final-checksum` to `c28-ide-through-link` are recorded done.

Enabled again at 2026-11-08T16:19:00-05:00,
before those stages finished,
after the owner questioned the wait.
The stages read the repository,
and nothing in the move's sources names the build folders,
so the wait had no reason behind it.
The dry run read 6,923 trees,
would remove 18 (26.31 GiB) and kept 6,905:
each of the 18 names a source that is gone or names no workspace,
and each is past its idle limit.
This time enabling it started no run;
the next is 04:30.
It still runs the installed 0.1.5-alpha.7,
without the fixes merged in `a5b31ab7de`,
until the next release.

## Incident in the build

At 2026-11-07T23:19 an early debug build of `cargo-shadow lean run --only units` was started in a disposable fixture
repository without an empty `CARGO_HOME`.
It read the real `~/.cargo/config.toml`,
swept the real host build base in real mode for a few seconds,
and ended when its output pipe closed
(`| head` after twenty lines:
`eprintln!` panicked on the closed pipe).
It removed older-compiler units from one real profile directory,
`/var/mnt/encrypted/cargo-build/00/030bc0f0fac3f9/release`,
under the build lock and a process scan,
which is what the first run does anyway;
the profile directory had no leftover trash.
The fix is in the released binary:
output ignores a closed pipe,
and a stopped run's leftover trash is finished by the next one.

## Measured 2026-11-10: what a pass over every tree and profile directory costs

What the journal holds for the two runs that went furthest
(`journalctl --user --unit=cargo-shadow-lean.service`,
`-o short-iso`,
read 2026-11-10):

- The run that ended 2026-11-10T07:37:24-05:00 (pid 1251356,
  bound `3 * 3_600` s,
  started 04:30:00,
  deadline 07:30:00,
  the unit's own timeout 08:00:00):
  run records 04:30:00 to 04:30:24 (24 s for 34 verification directories,
  nothing due),
  toolchains in the same second (7 dated nightlies judged,
  none retired),
  Podman 04:30:24 to 04:35:58 (334 s:
  56 images removed,
  172 gone with another image of the run,
  2044 kept),
  idle trees 04:35:58 to 07:30:33 (10,475 s,
  of which the last 33 s were the summary after the deadline),
  then the units pass 07:30:33 to 07:38:05 (452 s,
  no profile directory judged:
  listing them was all it had time for).
  The idle-trees pass judged 14,850 of 33,804 trees (278 removed for 511.49 GiB of disk blocks,
  14,572 kept),
  which is 0.71 s per tree including the removal of those 511 GiB,
  and it never reached the trees whose shard sorts after `b4/`.
- The run that ended 2026-11-09T08:00:00-05:00 (pid 2686726,
  same bound,
  started 04:30:00):
  run records 04:30:00 to 04:33:13 (193 s for 25 verification directories,
  116 records removed),
  toolchains to 04:33:19 (6 s,
  1 nightly retired for 2.73 GiB),
  Podman to 05:05:43 (1,944 s),
  idle trees from 05:05:43 with no summary line at all:
  the unit's `TimeoutStartSec` killed it at 08:00:00,
  30 minutes after the run's own deadline,
  while a single tree's deletion was still going.
  systemd recorded `2min 22s CPU time over 3h 30min wall clock`,
  `516.9M` memory peak.
  So the trees pass alone was over 2 h 54 min here and did not finish.

A planning pass over the real base,
2026-11-10,
`./target/release/cargo-shadow lean run --dry-run`,
in scopes with `CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=256` and `IOWeight=1`,
while other sessions held the host between 70 and 148 on the load average:

- `--only units --deadline-minutes 0`,
  which lists and judges nothing:
  `listed 66115 profile directories in 466 s`,
  471.94 s of wall clock and 1.97 s of user time.
  The same listing inside the pass below took 880 s for 68,755,
  so it is almost purely waiting for the disk,
  and the list grew by 2,640 profile directories in the 74 minutes between the two.
- `--only units --deadline-minutes 30`:
  `listed 68755 profile directories in 880 s`,
  then 1,782 of them judged in the 921 s the 30-minute bound left,
  which is 0.52 s each,
  and the dry run also sized the 5,521 units it would remove (16.57 GiB).
  A pass over 68,755 profile directories therefore costs about 9.9 h today.
- `--only trees --deadline-minutes 45`:
  stopped after 65 minutes still inside the fixed read of every tree's recorded workspace,
  having judged nothing.
  That read has no deadline of its own,
  and on that host it was longer than the whole bound;
  the pass now says how long it took (`read the recorded workspace of N trees in X s`),
  and the answer a quiet host gives is not measured yet.
  On the quiet 04:30 runs the journal covers,
  the same work fits inside the measured 0.71 s per tree,
  so the fixed read there is a fraction of the pass.

So a whole pass over the trees and the profile directories costs about 17 h today
(6.6 h + 9.9 h + the 6 minutes the cheap mechanisms took),
against the 3 h the bound allowed:

`RUN_DEADLINE_SECONDS` is 5 h (`package/cargo-shadow/cli/src/lean_policy.rs`),
the service's `TimeoutStartSec` is 5.5 h,
and both reach the installed unit only through a release and `cargo-shadow lean timer-install`
(the unit installed on 2026-11-10 still holds 12,600 s),
and a run from the timer's 04:30 ends at 09:30,
or 10:00 at worst when one large tree's deletion outlasts the run's own bound
(the run that ended 2026-11-09T08:00:00-05:00 was killed by exactly that,
30 minutes past its deadline).
No bound a person would accept makes one run cover seventeen hours,
so the bound stays below a whole pass and the recorded positions carry the rest:
the trees pass at 6.6 h needs a second night,
and that second night's last 3.4 h are the units pass's.
Measured costs of a longer run to a person:
`2min 22s CPU time over 3h 30min wall clock` and a 517 MiB peak in that 2026-11-09 run,
`IOWeight=5` against the default 100 on a disk whose scheduler is BFQ,
which honours that weight,
`IOSchedulingClass=idle` and `Nice=15` in the unit,
and a commit of fourteen files in this repository took 2.69 s while this planning pass was reading the same volume
under a load average near 100
(`doc/troubleshooting/git-fsmonitor-daemon-start-wait.md` has the one measured cause of a slow commit here,
and it is not this run).
Not measured:
the fixed workspace read on a quiet host,
what the units pass costs when its units are deleted rather than sized in a dry run,
and a commit timed beside a lean pass at the unit's own `IOWeight=5` against the same commit with nothing running,
because a throwaway repository cannot be put on the build volume,
so the one commit timing above is not a controlled comparison.

## First run

Released versions:
0.1.5-alpha.5 had its removal log lines printed only when a pass ended,
0.1.5-alpha.6 streams them,
and 0.1.5-alpha.7 (installed and locked) also refuses a dependency token that the read cut in two as a lead
(the sample dry run showed cut paths such as `/home/u` read as gone sources).
The passes below ran with alpha.5 (records,
toolchains) and alpha.6 and alpha.7 (Podman,
trees,
units);
`doc/planning/lean-build-dirs.md` is not edited for this (a concurrent reflow of that file was uncommitted in the main
checkout).
Run from 2026-11-07T23:52:28-05:00 to 2026-11-08T01:24:39-05:00,
each pass in its own systemd user unit with `IOWeight=5`,
and the units stopped before the session closed.
The log lines with the proof of each removal are in `~/temp/agent/lean`
(`first-home.err`,
`first-podman.err`,
`first-trees.err`,
`first-units.err`).

- Run records:
  493 records removed (7.33 GiB by disk blocks) in 120 verification directories of all checkouts,
  none in use.
- Toolchains:
  12 of 17 dated nightlies retired (28.39 GiB by disk blocks;
  `du` of `~/.rustup/toolchains` 52.95 GB before,
  22.47 GB after),
  by `rustup toolchain uninstall`;
  five stayed because a checkout's lock or rustup's settings pin them
  (2026-10-29,
  2026-10-30,
  2026-10-31,
  2026-11-01,
  2026-11-07).
- Podman:
  251 images and 27 volumes removed,
  2744 images and 5 volumes kept.
  Podman itself refused many plain `rmi` calls (image held by a Buildah working container that `podman ps` does not list,
  an image with several tags,
  or an image already gone with its child),
  and nothing was forced.
  The failures are in `first-podman.err`'s summary.
  Judged on 5604 image rows and 20 containers in any state.
- Idle trees:
  7069 trees read,
  185 removed (406.18 GiB by disk blocks),
  6884 kept,
  none skipped,
  in about 26 minutes.
- Units and incremental state:
  stopped at 2026-11-08T01:24:39-05:00,
  after the first 785,496 units and 1,018 incremental directories
  (the first 700-odd profile directories of several thousand),
  because a pass of this size takes many hours at disk weight 5.
  The rest waits for the daily timer.
  A pass stopped by `systemctl stop` can leave one profile directory with a `.cargo-shadow-trash` directory;
  the next run deletes it under the profile's lock.
- Free space (`df`,
  before at 2026-11-07T23:52:28-05:00,
  after at 2026-11-08T01:24:39-05:00):
  `/var/mnt/encrypted` (the 860 drive,
  which also holds the repository move's 150 GiB margin) 976.30 GB to 1113.87 GB
  available,
  +137.56 GB (128.1 GiB),
  142.25 GB less used;
  `/var/home` (the filesystem of the home directory,
  rustup and Podman storage) 303.03 GB to 303.66 GB available,
  +0.63 GB (not the freed amount:
  30.5 GB of toolchains alone went by `du`;
  other sessions write there,
  and snapshot-shared extents free nothing until their last reference goes);
  `/` is composefs,
  50 MiB,
  always 0 B free,
  and no tool here can change it.
- Timer:
  installed and enabled on 2026-11-08 before 01:00;
  `systemctl --user list-timers cargo-shadow-lean.timer` shows the next fire at 2026-11-08T04:30:00-05:00.
  `systemd-analyze --user verify` of both units is clean.
