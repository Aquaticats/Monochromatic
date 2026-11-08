# cargo-shadow lean build folders

Open work and current state of the lean-build-folders plan
(`doc/planning/lean-build-dirs.md`,
owner decision of 2026-11-07,
"A to F, B allowed (Recommended)"),
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

## Incident in the build

At 2026-11-07T23:19 an early debug build of `cargo-shadow lean run --only units` was started in a disposable fixture
repository without an empty `CARGO_HOME`.
It read the real `~/.cargo/config.toml`,
swept the real host build base in real mode for a few seconds,
and ended when its output pipe closed
(`| head` after twenty lines: `eprintln!` panicked on the closed pipe).
It removed older-compiler units from one real profile directory,
`/var/mnt/encrypted/cargo-build/00/030bc0f0fac3f9/release`,
under the build lock and a process scan,
which is what the first run does anyway;
the profile directory had no leftover trash.
The fix is in the released binary:
output ignores a closed pipe,
and a stopped run's leftover trash is finished by the next one.

## First run

Filled in by the run itself;
see the final report of the session that built this and the journal of the units
`lean-first-run-home` and `lean-first-run-trees`
(`journalctl --user -u <unit>`).
