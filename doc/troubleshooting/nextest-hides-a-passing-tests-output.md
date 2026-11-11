# nextest 0.9.150 counts a test that reports it did not run as passed, and shows nothing it printed

## Symptom

A control that cannot do its work in this environment prints one line naming itself and the
reason,
then returns,
so the run is green and the summary counts it as covered:

```text
        PASS [   0.015s] (1/2) monochromatic-git-policy-cli worktree::worktree_snapshot::tests::btrfs_the_swap_puts_the_stage_at_the_destination
        PASS [   8.367s] (2/2) monochromatic-git-policy-cli::native_binary worktree_ladder::a_subvolume_worktree_is_removed_and_a_dirty_one_is_refused
────────────
     Summary [   8.370s] 2 tests run: 2 passed, 5432 skipped
```

Both of those controls skipped their work,
and the `stderr` they printed is not in the log:

```text
    BTRFS CONTROL NOT RUN: snapshot-swap (fixture root /run/user/1000/btrfs-negative-control/native-unit-1751841-snapshot-swap is on Ok(Other); the host task native:test:host:btrfs covers it)
```

`2 passed` is the whole of what a reader of that log sees.
This is how the wrapper's btrfs controls stayed unrun for weeks in the container gate while the
gate read as full coverage (`doc/handover/cli-git-rust-implementation.md`,
`### Gate and tooling`).
The `5338 skipped` count does not help:
nextest counts skipped as tests its filters and ignore markers removed,
never a test that ran,
passed,
and chose not to do its work.

## Root cause

nextest's embedded default profile sets the display of a passing test's output to `never`,
in the copy of `default-config.toml` the binary carries
([`nextest-runner/default-config.toml:52-53`][default-config]):

```toml
# When to display output for successful tests. Generally best left as "never".
success-output = "never"
```

`TestOutputDisplay` has four values,
and `Never` shows neither now nor at the end
([`nextest-runner/src/reporter/displayer/unit_output.rs:30-42`][enum]):

```rust
    /// Never shows captured output.
    Never,
```

For a finished test the reporter resolves the passing case to `success_output`
([`nextest-runner/src/reporter/displayer/unit_output.rs:99-114`][resolve]):

```rust
        if describe.is_success_for_output() {
            self.success_output(success_output)
```

and then refuses to load the recorded output at all when that value is neither immediate nor
final
([`nextest-runner/src/reporter/displayer/imp.rs:406-418`][load]):

```rust
        if is_immediate || is_final {
            LoadOutput::Load
        } else {
            LoadOutput::Skip
        }
```

`should_load_for_test_finished` is the caller for a test's own output
([`nextest-runner/src/reporter/displayer/imp.rs:356-369`][finished]).
So the line is not dropped late by a log filter:
nextest never reads it out of the run record,
and no amount of log searching finds it.

## Verification

Installed tool:
`cargo-nextest 0.9.150 (d225b1daf 2026-10-26)`,
from `cargo nextest --version`.
Source read:
nextest `2cf55ec204573ecbe1a921a93f77a1b1cc7244ab` (`nextest-runner` 0.129.0),
cloned with `gh repo clone nextest-rs/nextest -- --depth 1`.
The citations above are that commit's lines;
the installed 0.9.150 carries the same default,
which `cargo nextest run --help` shows for the flag:

```text
      --success-output <WHEN>
          When to display output for successful tests

          [env: NEXTEST_SUCCESS_OUTPUT=]
          [possible values: immediate, immediate-final, final, never]
```

A two-test crate that prints one line from a passing test and nothing from another:

```rust
// src/lib.rs of a package with no dependencies
#[test]
fn prints_and_passes() {
    eprintln!("BTRFS CONTROL NOT RUN: probe-control (fixture root /x is on Overlayfs)");
    assert_eq!(1 + 1, 2);
}

#[test]
fn says_nothing() {
    assert_eq!(2 + 2, 4);
}
```

Default reporting hides the line,
and `--success-output=final` shows it.
The first transcript below is that probe crate;
the second is the wrapper's own tree,
where the same two controls ran green while printing the marker.

```text
$ cargo nextest run
        PASS [   0.004s] (1/2) probe says_nothing
        PASS [   0.004s] (2/2) probe prints_and_passes
     Summary [   0.004s] 2 tests run: 2 passed, 0 skipped

$ cargo nextest run --success-output=final
        PASS [   0.006s] (1/2) probe prints_and_passes
  stdout ───

    running 1 test
    test prints_and_passes ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

  stderr ───
    BTRFS CONTROL NOT RUN: probe-control (fixture root /x is on Overlayfs)
```

One caveat of the second arm:
every passing test's harness output comes with it,
which for the wrapper's own suite is a few lines per test,
so the log grows with the number of tests in the run,
not with the number of skips.

## Verified workarounds

Show the captured output of passing tests,
at the end of the run:

```sh
cargo nextest run --success-output=final
```

Or for every run of a package,
in its `.config/nextest.toml`,
which keeps the flag out of each caller:

```toml
[profile.gate]
success-output = "final"
```

Tradeoffs:
the log grows by every passing test's own output,
there is no way to ask for "only tests that printed something",
and a reader still has to search the log for the marker the control printed.
Set the marker as a fixed prefix at the skip site so that search is one `rg`.

## One line, two meanings, and the names a green run has to hold

This document's marker has a second reading that a gate must not miss:
a control that needs a filesystem that is not btrfs prints the same line on a btrfs host,
because there it is the control that has nothing to check.
`off_btrfs_the_ioctls_report_a_missing_feature`
(`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl_tests.rs:116-126`)
prints it when `filesystem_kind` answers btrfs,
so the one green btrfs host run of the suite holds exactly one such line and means the opposite of a skip.
A gate that refuses on every `BTRFS CONTROL NOT RUN` line therefore refuses every release on a btrfs host;
the release gate of the wrapper refuses only the lines whose own reason is not
`needs a filesystem that is not btrfs`
(`package/git-policy/cli/bin/native-release-host-controls.ts`).

A run's exit status alone is also no evidence that a control ran.
The release gate therefore requires the passing status line of three controls by name,
and nextest prints those lines only at a status level a person's own
`~/.config/nextest/config.toml` can silence:
with `NEXTEST_STATUS_LEVEL=none` the same one-test run printed only its summary:

```text
     Summary [   0.012s] 1 test run: 1 passed, 6092 skipped
```

so the gate sets `NEXTEST_STATUS_LEVEL=pass` and `NEXTEST_FINAL_STATUS_LEVEL=pass`
for the run rather than depending on that file.
With them the same run prints the name it checked:

```text
        PASS [   0.009s] (1/1) monochromatic-git-policy-cli subvolume::fs_btrfs_ioctl::tests::a_path_without_a_name_cannot_be_split
```

## What does not work

- Failing the control instead of skipping it.
  The gate has no btrfs filesystem,
  so a control that fails there turns every gate run red and
  the run cannot distinguish a real btrfs regression from the container's own filesystem.
  The control must be able to state "not here",
  but that statement has to be visible.
- `--no-capture` (`--nocapture`).
  It does show the line,
  and nextest documents that it also runs the tests serially,
  because it passes the test binary's standard output straight through,
  which costs the parallelism the gate's `--no-fail-fast` run relies on.
- Reading the `skipped` count in the summary.
  It counts what filters and `#[ignore]` removed.
  A control that ran,
  passed and did nothing is inside `passed`,
  as the symptom's summary shows.
- `junit.store-success-output = true`.
  It does store the output,
  but into the JUnit report nextest writes when `junit.path` is set,
  which the gate does not produce or read.
- Searching the log for the marker without one of the two options above.
  The text is never in the stream to search,
  per the root cause.

## Upstream filing decision

Do not file.
The audit trail:

1.  Upstream's fault?
    No.
    The default is documented in the shipped configuration and deliberate
    ("Generally best left as `never`").
    Nothing behaves other than as documented.
2.  Can upstream fix it?
    Yes in principle,
    and it is not a defect:
    a "show output of tests that printed only to stderr" option would be a feature request.
3.  Supporting this use case?
    Not stated.
    The option list has no "only tests with output" value.
4.  Would the repository welcome a contribution?
    Not checked:
    no filing is proposed,
    so the contribution policy was not read.
5.  Will they fix it?
    Not checked for the same reason.
6.  Prototyped fix?
    None,
    and none is wanted at this boundary:
    the consumer-side option already exists and is applied in this repository
    (`package/git-policy/cli/bin/native-test-containerfile.ts`,
    `runArguments`).

The workaround sits at our boundary and needs nothing from upstream.

[default-config]: https://github.com/nextest-rs/nextest/blob/2cf55ec204573ecbe1a921a93f77a1b1cc7244ab/nextest-runner/default-config.toml#L50-L53
[enum]: https://github.com/nextest-rs/nextest/blob/2cf55ec204573ecbe1a921a93f77a1b1cc7244ab/nextest-runner/src/reporter/displayer/unit_output.rs#L26-L42
[resolve]: https://github.com/nextest-rs/nextest/blob/2cf55ec204573ecbe1a921a93f77a1b1cc7244ab/nextest-runner/src/reporter/displayer/unit_output.rs#L99-L114
[load]: https://github.com/nextest-rs/nextest/blob/2cf55ec204573ecbe1a921a93f77a1b1cc7244ab/nextest-runner/src/reporter/displayer/imp.rs#L406-L418
[finished]: https://github.com/nextest-rs/nextest/blob/2cf55ec204573ecbe1a921a93f77a1b1cc7244ab/nextest-runner/src/reporter/displayer/imp.rs#L356-L369
