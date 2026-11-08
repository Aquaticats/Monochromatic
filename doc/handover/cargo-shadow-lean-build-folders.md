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
All of this is in the sources only.
Until cargo-shadow is released and installed,
a producing task stops at its first step,
because the installed 0.1.5-alpha.7 lists only `lean run` and `lean timer-install`.

## State

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
  and now also from `cargo-shadow lean records <directory>...` over the directories one producing task names
  (`package/cargo-shadow/cli/src/lean_command.rs`,
  section "Keeping the build folders lean" of the crate README).
  Each of these tasks runs that first,
  over its own `target/verification`:
  `native:test:container` and `native:mutation` of `package/git-policy/cli`,
  `smoke` and `test:planted` of `package/git-policy/cli.fuzz`,
  `test:container` of `package/linter/monochromatic-lint`,
  `smoke` and `test:planted` of `package/linter/monochromatic-lint.fuzz`,
  and `test:container` and `test:mutation:container` of `package/forbidden-strings/cli`.
  The plan assigned D to those packages;
  the owner's answer put all six mechanisms in cargo-shadow,
  so the tasks call its retention rather than reimplementing it.
  What that leaves uncovered,
  each measured from the task files rather than assumed:
  the linter's `test:markdown`,
  `test:processors`,
  `test:processors:slice`,
  `test:inferred-constants` and `test:rust-style` write records without depending on `test:container`,
  and so does `mutation:rust-style`;
  `package/git-policy/cli`'s `native:mutation:scoped` and `native:mutation:planted` write
  `native-mutation-*` records of their own,
  as do the units `native:mutation:parallel` starts;
  most of `package/forbidden-strings/cli`'s other container tasks write records too.
  They wait for the coordinator to say whether the sweep belongs on each of them or in the shared runners,
  because a producing task that a wired task starts must not sweep a second time.
  Why a scoped entry point and not the whole pass:
  a machine-wide dry run of the records mechanism alone measured 398 s on this host on 2026-11-08
  (359 checkouts,
  119 verification directories read,
  nothing due for removal),
  which is too much in front of every producing task.
  Not measured:
  what the scoped sweep costs on a package whose records are due,
  and what each of these tasks now takes.
  A second finding,
  from the main checkout's five `target/verification` directories on 2026-11-08,
  is that the retention's name list is narrower than what the producers write.
  `lean_run_records.rs` removes only names starting `campaign-`,
  `native-`,
  `git-image-`,
  `mutation-` or `planted-`.
  So `processors-mutation-*` and `coverage-*` in `package/linter/monochromatic-lint`,
  and every record of `package/forbidden-strings/cli`
  (`container-`,
  `clippy-`,
  `guard-`,
  `pathname-mutation-`,
  `startup-mutation-`,
  `release-test-`,
  `test-`,
  `windows-`),
  are never swept,
  by the daily run or by these tasks.
  Widening that list changes which real records are removed,
  so it is left to the coordinator and the owner.
  A producing task that runs before the release and install stops at its first step
  with `error: unrecognized subcommand 'records'`;
  no producing task was run while making this change.
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
  route `archive`,
  launcher record `.cache/toolchain-launchers/monochromatic-git-policy-cli.launch`).
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
  `scan_processes` counts a process whose `/proc/<pid>/cwd` cannot be read as unreadable and does not inspect it
  (`package/cargo-shadow/cli/src/lean_proc_scan.rs`),
  and `blocks` consults only the uses it found,
  so a record a process of another user holds can be removed.
  The sweep of one package's directory printed this line on 2026-11-08:
  426 processes read,
  681 unreadable (other users),
  58 in other mount namespaces judged by identity,
  0 uses under the root.
  The removal messages name that line as the proof,
  so a reader can see the uncertainty;
  whether a record held by another user's process may go is the owner's call.
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
  today the deadline cuts the units pass,
  and the next run starts it again at the front of the profile list.
  Until a release and `mise run toolchain:install`,
  the timer keeps running the installed 0.1.5-alpha.7 and its old order.
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
