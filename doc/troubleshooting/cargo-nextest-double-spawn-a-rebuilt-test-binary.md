# cargo-nextest 0.9.150: a test binary rebuilt under a running suite fails every remaining test with `[double-spawn] failed to exec`

## Symptom

`mise run //package/git-policy/cli:native:test:host` runs `cargo nextest run --locked --offline --lib`,
which builds the test binary and then runs 6,271 tests from that one binary.
When another command rebuilds the same crate into the same target directory while the suite is running,
almost every test from that moment on fails in 3 to 5 ms with one line per test
(3,633 of the 3,635 that followed,
in the run quoted here;
the two that passed had already been spawned):

```text
FAIL [   0.004s] (6271/6271) monochromatic-git-policy-cli worktree_copying::worktree_copy_vanished_entry::tests::an_absent_entry_is_no_metadata_and_no_failure
  stderr ───
    error: [double-spawn] failed to exec `"/var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-10/b8/ea46e1a0aff3a2/debug/build/monochromatic-git-policy-cli/dd3f7a5e87b058fc/out/git_policy_cli-dd3f7a5e87b058fc" "--exact" "worktree_copying::worktree_copy_vanished_entry::tests::an_absent_entry_is_no_metadata_and_no_failure" "--nocapture"`, current_dir: `/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-merge-commit-landing-in-a-busy-checkout/package/git-policy/cli`

    Caused by:
      No such file or directory (os error 2)

    (test failed with exit code 70)
```

The run ends with a summary that looks like a catastrophic regression:

```text
     Summary [ 134.132s] 6271 tests run: 2638 passed, 3633 failed, 19 skipped
```

and the task exits 100.
A test that failed this way printed nothing of its own:
no assertion,
no output,
only the `failed to exec` line,
and it failed in milliseconds where the same test takes seconds when it actually runs.

The trigger is not the code under test.
It is two commands that build one crate in one target directory at the same time:
a suite,
and any filtered run,
`cargo test`,
`cargo clippy`,
or `cargo build` of the same crate.
This repository's build directory is shared by every worktree of a checkout
(`CARGO_TARGET_DIR` under `/var/mnt/encrypted/cargo-build`),
so a build in one worktree rewrites the binary another worktree's suite is running.

## Root cause

The chain is nextest's double-spawn,
and it is the reason the failure names a path rather than a test.

`cargo nextest --version` here is `0.9.150 (d225b1daf 2026-10-26)`.
The citations are from the tag `cargo-nextest-0.9.150`,
commit `d225b1daf362c1ca8e5e048132618c1a81c57a28`,
and each names that commit's own file and lines.

1. Nextest resolves **its own** executable once,
   when it first needs the double-spawn stub,
   and keeps that path for the whole run
   ([`nextest-runner/src/double_spawn.rs` at 115 to 152](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/nextest-runner/src/double_spawn.rs#L115-L152),
   `try_enable` and `get_current_exe`):

   ```rust
   let current_exe = get_current_exe().map_or_else(/* warn and disable */, Some);
   ```

   ```rust
   static PROC_SELF_EXE: &str = "/proc/self/exe";
   ```

2. Every test process is then spawned through that stub rather than directly
   ([`nextest-runner/src/test_command.rs` at 304 to 321](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/nextest-runner/src/test_command.rs#L304-L321)):

   ```rust
   if let Some(current_exe) = double_spawn.current_exe() {
       let mut cmd = std::process::Command::new(current_exe);
       cmd.args([DoubleSpawnInfo::SUBCOMMAND_NAME, "--", program.as_str()]);
       cmd.arg(shell_words::join(args));
       cmd
   } else {
       let mut cmd = std::process::Command::new(program);
   ```

   `program` is the **test binary's path**,
   resolved when nextest built the test list,
   and `args` is what the test binary is to be run with (`--exact <name> --nocapture` in the message).

3. The stub is the same executable,
   entered as `cargo-nextest __double-spawn -- <program> <args>`,
   and it `exec`s that program
   ([`cargo-nextest/src/double_spawn.rs` at 21 to 37](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/cargo-nextest/src/double_spawn.rs#L21-L37)):

   ```rust
   let mut command = std::process::Command::new(&self.program);
   // Note: exec only returns an error -- in the success case it never returns.
   let err = command.args(args).exec();
   Err(ExpectedError::DoubleSpawnExecError { command: Box::new(command), current_dir: std::env::current_dir(), err })
   ```

4. Only that failure produces the line in the symptom
   ([`cargo-nextest/src/errors.rs` at 1320 to 1334](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/cargo-nextest/src/errors.rs#L1320-L1334)):

   ```rust
   error!("[double-spawn] failed to exec `{command:?}`, current_dir: `{current_dir_str}`");
   ```

   and the stub exits with `NextestExitCode::DOUBLE_SPAWN_ERROR`
   ([`cargo-nextest/src/errors.rs` at 655 to 657](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/cargo-nextest/src/errors.rs#L655-L657)),
   which is `70`
   ([`nextest-metadata/src/exit_codes.rs` at 23](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/nextest-metadata/src/exit_codes.rs#L23)):

   ```rust
   pub const DOUBLE_SPAWN_ERROR: i32 = 70;
   ```

So the run holds one path to its test binary for its whole life,
and its first use of that path is inside a fresh stub process for every single test.
`cargo` replaces the binary at that path when it rebuilds the unit:
the file is written again in place,
and a rebuild whose fingerprint directories change leaves the old
`out/` path gone for good.
Whichever happens,
the stub's `exec` is the first call that notices,
and it notices once per test.

What this record first read as,
and why that was wrong:
3,633 failures in one run looked like a regression from the change being tested.
Three things disproved it.
Every failure was `failed to exec` at 3 to 5 ms with no test output,
the failures began at the 2,637th test and never stopped,
and the binary the message names was written again at 2026-11-10T19:48:10.470058170-05:00,
24 s after that run's log was last written (2026-11-10T19:47:46.147258172-05:00),
because a filtered run of one test had rebuilt the same crate while the suite ran.

## Verification

- Under test:
  `cargo-nextest 0.9.150 (d225b1daf 2026-10-26)`,
  the version `mise exec -- cargo nextest --version` prints in this checkout's `package/git-policy/cli`.
- The harness is two commands in one target directory,
  the first long enough for the second to overlap it:

  ```bash
  # terminal 1: the suite
  mise run //package/git-policy/cli:native:test:host -- --no-fail-fast
  # terminal 2, 20 s later: any rebuild of the same crate
  mise run //package/git-policy/cli:native:test:host -- a_merge_in_progress_copies_its_entries_and_flushes_neither_copy
  ```

- Failing catalog:
  the run above.
  3,633 of 6,271 tests failed,
  all with the `failed to exec` line,
  and the summary was
  `Summary [ 134.132s] 6271 tests run: 2638 passed, 3633 failed, 19 skipped`,
  task exit 100.
  The failing path is a test binary (`out/git_policy_cli-<hash>`) and the arguments are a test's own
  (`--exact <test name> --nocapture`),
  which is how the stub's command line can be told from any other.
- Clean catalog,
  same checkout and same task:
  the filtered run alone,
  `Summary [   1.154s] 25 tests run: 25 passed, 6265 skipped`;
  the single control alone,
  `Summary [   0.059s] 1 test run: 1 passed, 6289 skipped`;
  and the whole suite with nothing else building in its target directory (quoted under
  "A whole suite in the same target directory,
  run alone").
- A second failing shape to look for in a log:
  when the replaced binary is the `cargo-nextest` stub itself rather than a test binary,
  the suite fails before any test starts and the message names `/proc/self/exe`'s target instead.
  That shape was not reproduced here and is stated as untested.

### A whole suite in the same target directory, run alone

`mise run //package/git-policy/cli:native:test:host -- --no-fail-fast`
with no other cargo command in the shared target directory:

```text
     Summary [ 386.629s] 6271 tests run: 6271 passed (1 slow), 19 skipped
```

That run took 386.6 s against 134.1 s for the broken one,
and it exited 0.
Both numbers are the same suite,
the same crate and the same target directory;
the only difference is that nothing rebuilt the binary while it ran.

## Verified workarounds

1. **Give a suite a target directory of its own.**
   `CARGO_TARGET_DIR=$HOME/temp/agent/<run>/target mise run //package/git-policy/cli:native:test:host`
   compiles the crate and its dependencies once in that directory,
   and no other worktree's build can rewrite what it runs.
   The same crate built twice costs the compile twice,
   so this is the workaround to pick when two agents must build in the same window,
   and not the default.
   Does not change what is tested.
2. **Use the container gate for a suite that others may build beside.**
   `mise run //package/git-policy/cli:native:test:container` compiles inside its own image,
   so a host build cannot touch its binaries;
   its limits (2 GiB,
   two CPUs,
   512 PIDs by default) also keep the run from loading the host.
   The tradeoff is that it tests the image's Git and toolchain,
   not the host's.
3. **Serialize:
   let a suite finish before the next build starts.**
   This is what the repository's own tasks assume,
   and it needs no flag:
   a rebuild after the suite ends leaves the next suite with a current list and a current path.
   A killed suite leaves nothing behind,
   because the plan lives only in that process.
4. **`NEXTEST_DOUBLE_SPAWN=0`** removes the stub
   ([`cargo-nextest/src/dispatch/core/base.rs` at 372 to 377](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/cargo-nextest/src/dispatch/core/base.rs#L372-L377)):

   ```rust
   if std::env::var("NEXTEST_DOUBLE_SPAWN").as_deref() == Ok("0") {
       DoubleSpawnInfo::disabled()
   ```

   With it,
   nextest spawns the test binary itself,
   so the same replaced path fails as a spawn error of the test process rather than of the stub.
   It does **not** make a replaced path exist,
   and it gives up the `SIGTSTP` protection the double-spawn was added for
   ([`nextest-runner/src/double_spawn.rs` at 11 to 30](https://github.com/nextest-rs/nextest/blob/d225b1daf362c1ca8e5e048132618c1a81c57a28/nextest-runner/src/double_spawn.rs#L11-L30)),
   so treat it as a way to make a diagnosis clearer,
   not as a fix.

## What does not work

- `--no-fail-fast` was set in the failing run and only made the report longer:
  it keeps the run going,
  which is how 3,633 identical failures were collected instead of the first one.
  It prevents nothing.
- Re-running the failing tests inside the same suite,
  or `--retries`,
  changes nothing:
  the path is wrong for every attempt.
- Reading the failures as flaky tests is wrong,
  and expensive:
  they have no test output at all,
  they fail in milliseconds,
  and they stop the moment the build ends.
- Running the suite in a linked worktree does not isolate it,
  because this repository's worktrees share one `CARGO_TARGET_DIR`
  (`doc/planning/cargo-shadow-lean-build-folders.md` and `mise.toml`).

## Upstream filing artifact

Nothing filed,
and nothing to add as a bug report:

- nextest's behaviour is a named,
  diagnostic failure with its own exit code and the exact command it could not
  run.
  The cause is a file replaced underneath a plan that was made before the replacement,
  which nextest cannot prevent.
- A duplicate search found no matching report
  (`gh search issues --repo nextest-rs/nextest "double-spawn failed to exec"`,
  one closed-issue hit about setup scripts,
  unrelated).
- The 6-constraint check therefore finds no defect to report,
  and publishing anything external needs the owner's authorisation,
  which this record does not have.

If it is ever reported,
the minimal shape is:
build the test binary,
start `cargo nextest run`,
replace the binary at the path nextest lists with another build of the same crate,
and every remaining test
fails with exit code 70 and `[double-spawn] failed to exec`.
