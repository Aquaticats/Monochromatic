# A build that waits minutes on Cargo's package cache

What:
 a build or a test run of this repository stops with
`Blocking waiting for file lock on package cache` for minutes to tens of minutes,
and what removes that wait without changing what is built.
Why:
 on 2026-11-12 seven audits of the wrapper and this session's own runs met it,
with some forty agents building in one account,
and one audit's build slept about 34 minutes on it.
Measured:
 2026-11-12 on this host,
by the runs named below.

## What a person sees

- `cargo` prints `Blocking waiting for file lock on package cache` and does nothing else.
- A run started in that state waits for the whole time another build holds the lock;
  nothing in Cargo bounds it.
- A test that spawns `cargo` (several of the wrapper's repair controls do) hangs with the run,
  and the wrapper's unit-test watchdog ends it as a stuck test:
  `the unit-test run showed no sign of work for 120.2 s (no task able to run or waiting for the
  disk, no processor time, no bytes moved, and no process or task started): a test is stuck`.

## What holds it

Cargo takes one exclusive lock per `CARGO_HOME`, on `$CARGO_HOME/.package-cache`
(rust-lang/cargo issue 15094 names that file and the message;
its companion is `$CARGO_HOME/.package-cache-mutate`).
The lock is taken while Cargo touches the shared package cache: an index update,
a download, an unpack, and the registry reads around them.
It is shared by every Cargo process of the account,
whatever worktree and whatever `--target-dir` they build in,
so parallel agents serialize on it.

`--offline` and `--locked` do not avoid it:
Cargo still takes the lock for the cache it reads.
Measured here,
a `cargo check --locked --offline --lib` of `package/git-policy/cli` waited 300 s and was ended
by the run's own `timeout`,
and the debug build of that package (`mise run //package/git-policy/cli:native:build`) took
3,250 s wall while other agents built,
most of it in this wait:
its first minutes printed nothing but that line.

## What removes the wait

A `CARGO_HOME` per job that reaches the shared cargo home through links:

```sh
# one directory per job, outside the repository
mkdir --parents "${JOB_HOME}/cargo"
ln --symbolic --force "${HOME}/.cargo/registry" "${JOB_HOME}/cargo/registry"
ln --symbolic --force "${HOME}/.cargo/git" "${JOB_HOME}/cargo/git"
cp "${HOME}/.cargo/config.toml" "${JOB_HOME}/cargo/config.toml"    # the same build layout and mirrors
export CARGO_HOME="${JOB_HOME}/cargo"
```

The crates, their sources and the configuration are the same files,
so nothing is downloaded and nothing is built differently;
only the lock file is the job's own.

Measured here:
two controls of `package_repair_prewarm_job` that spawn real Cargo were aborted by the
unit-test watchdog at 240 s under the shared cargo home
(`the_filling_publishes_a_clean_copy_and_leaves_the_source_alone`,
`the_filling_puts_the_lock_back_as_checked_out`),
and the same test passed in 3.0 s with a cargo home of its own:
`1 test run: 1 passed, 7567 skipped`.
The whole 37-test selection then answered
`37 tests run: 37 passed, 7531 skipped` in 2.9 s,
where the same selection had taken 256.6 s and lost two tests to the watchdog.

## What does not help

- Deleting `$CARGO_HOME/.package-cache` while another Cargo runs:
  the lock is Cargo's own mutual exclusion,
  and removing the file it guards does not tell the other process anything.
- `cargo clean`: it deletes the build directory,
  which is not what is locked.
- A different `--target-dir`: the build directory lock is a second lock
  (`Blocking waiting for file lock on build directory`),
  and the package cache lock is taken before it either way.
- `--offline`: measured above.

## Where this is recorded

- This session's runs, 2026-11-12:
  the 37-test selection above, the 300 s `cargo check`, and the debug build's wait.
- `doc/planning/cli-git-worktree-add-package-repair-off-the-waiting-path.md`,
  which measured adds under the same host pressure.
