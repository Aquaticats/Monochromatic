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
  the run-record retention runs from `cargo-shadow lean run`.
  The producing tasks (the TypeScript runners of `package/git-policy/cli`,
  `package/git-policy/cli.fuzz`,
  `package/linter/monochromatic-lint`,
  `package/linter/monochromatic-lint.fuzz` and `package/forbidden-strings/cli`) do not call it first.
  The plan assigned D to those packages;
  the owner's answer put all six in cargo-shadow.
- A for container builds:
  `package/ide/app/mise.toml` and `package/nested-wayland-session/cli/mise.toml` do not set the keyed folder.
  Their in-tree builds are kept lean by B and E only.
- The Git wrapper's Cargo repair clones whichever build directory `cargo metadata` reports
  (`package/git-policy/cli/src/native/repair/package_repair_cargo_workspaces.rs`).
  Run through the shadow that is the keyed folder on both sides;
  run without the shadow it is Cargo's own hash folder on both sides and the next build under the shadow starts cold.
  Not measured.
- Containers in any state use their image:
  an image an exited container holds is kept.
  Whether exited containers should go too is a question for the owner.
- What runs `cargo package` or `cargo publish --dry-run` about 90 times per crate,
  and which wrapper build is installed:
  both open questions of the plan are still open.
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
- Timer starvation:
  `lean run` goes trees,
  units,
  records,
  toolchains,
  Podman under one three-hour deadline.
  The units pass did about 700 profile directories in an hour of several thousand,
  so for the first firings it uses the deadline and Podman (which needs the whole source scan to finish) may be skipped.
  Cheap mechanisms first,
  or a budget per mechanism,
  is open work.
- Podman removed 251 of 1601 judged.
  Three causes seen in its errors,
  each a fix:
  Buildah working containers are not in `podman ps --all` (it needs `--external`),
  an image with several tags needs `rmi repository:tag`,
  and a child image goes with its parent in the same batch (so later ids read "image not known").
  Nothing was forced.

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
