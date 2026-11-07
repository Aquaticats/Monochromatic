# mpm-shadow 0.2.7-alpha.2 launcher exits 127 in every other tree after a newer release of a repository tool is installed

Recorded 2026-11-07T13:05:00-05:00.

Status:
the launcher follows the tool's `current` link when the folder its record names is gone,
in the release of `monochromatic-mpm-shadow` that follows 0.2.7-alpha.2
(the version and its release record are filled in `doc/planning/one-release-each-batch-2026-11.md`,
section "Release record: `monochromatic-mpm-shadow` launcher follows current").
A tree whose launcher binary predates that release keeps failing until `mise run toolchain:install` runs in it.
The tool is `package/mpm-shadow/cli`,
which this repository owns,
so there is no upstream to file against.

## Symptom

An agent worked in a linked worktree whose lock said `cargo:monochromatic-cargo-shadow` 0.1.4.
After the main checkout locked and installed 0.1.5-alpha.1,
the agent reported that host tests in the worktree "stopped working",
and `mise run toolchain:install` in the worktree seemed not to clear it.
The error text was not kept;
the same failure was reproduced on 2026-11-07 in a disposable worktree
(a worktree of the parent of commit `c6177b875b`,
whose launcher record was put in the state the pre-release worktrees had)
and found naturally in `cargo-shadow-keep-caller-limits`:

```text
$ mise run //package/mpm-shadow/cli:lint
[//package/mpm-shadow/cli:lint] $ cargo check
mpm-shadow: warning: cargo:monochromatic-cargo-shadow is locked at 0.1.4 in this tree but 0.1.5-alpha.1 runs, because older repository tools are never kept once a newer one is installed; upgrade this tree's lock with `mpm update cargo:monochromatic-cargo-shadow` or merge main
mpm-shadow: could not start ~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-cargo-shadow/current/monochromatic-cargo-shadow-0.1.4-x86_64-unknown-linux-gnu/cargo-shadow: No such file or directory (os error 2)
[//package/mpm-shadow/cli:lint] ERROR task failed
```

The exit status of `.cache/cargo-shadow-bin/cargo` is 127.
The first line is correct and the second line names a folder of a version the first line says does not run.
The same failure hits the `mpm` link when `mpm-shadow` moved:
a scan of every `.cache/toolchain-launchers/*.launch` record in the main checkout and its linked worktrees on 2026-11-07
found 853 records,
of which 102 for `mpm-shadow` and 20 for `cargo-shadow` named a file that no longer exists.

## Root cause

The launcher is a hard link of `mpm-shadow` with a record beside it.
The record's `real` field is the program found at install time through the tool's `current` link
(`package/mpm-shadow/cli/src/toolchain/layout.rs`, `find_tool_bin`, which searches `<tool dir>/current` and descends into the archive folder).
For a crate installed by `cargo binstall` the path is `current/bin/<name>`,
which has no version in it.
For a release archive of this repository it is `current/<archive folder>/<name>`,
and the archive folder is named after the version:

```json
{"toolId":"cargo:monochromatic-cargo-shadow","lockFile":"<worktree>/toolchain.lock.json","real":"~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-cargo-shadow/current/monochromatic-cargo-shadow-0.1.4-x86_64-unknown-linux-gnu/cargo-shadow","scope":"repo"}
```

Installing 0.1.5-alpha.1 in the main checkout deleted the 0.1.4 version directory
(older repository tools are never kept),
and moved `current`.
The link `current` stayed valid,
but every other tree's record still named the 0.1.4 folder,
and `run_launcher` in `package/mpm-shadow/cli/src/toolchain/launcher.rs` executed that path as recorded.

An earlier reading was wrong:
the brief for this fix suspected that a shell started in the main checkout runs the main checkout's `cargo` launcher inside a worktree.
That is true of a shell without mise
(`which -a cargo` inside the worktree printed the main checkout's `.cache/cargo-shadow-bin/cargo` first),
but main's launcher runs the same newest copy and works;
under `mise run` and `mise exec` the worktree's own `.cache/*-bin` directories come first
(printed `PATH` entries, 2026-11-07).
Mise and the inherited `PATH` are not the cause,
and in a freshly made worktree the test run of `//package/mpm-shadow/cli:test` ended with 581 passed and exit 0,
printing only the older-lock warning.

`mise run toolchain:install` does repair a stale record:
in the disposable worktree it printed `0 installed, 66 unchanged, 0 failed`
and rewrote `real` to the 0.1.5-alpha.1 folder.
The agent's report that it did not clear the failure was not reproduced.

## Verification

Under test:
the mpm-shadow of this checkout after the fix,
with tests that build the layout above in a disposable directory
(`package/mpm-shadow/cli/tests/launcher.rs`, `MovedArchive`):

- `a_launcher_whose_recorded_version_was_deleted_runs_the_version_current_names`:
  stdout equals the new version's program run directly,
  stderr is the older-lock warning followed by the program's own stderr,
  exit 0.
- `a_launcher_with_a_stale_record_and_a_matching_lock_prints_nothing_extra`.
- `a_launcher_that_finds_no_program_under_current_names_both_versions_and_the_one_command`:
  exit 127,
  and the message names the tool,
  `locked at 0.1.4`,
  `0.1.5`
  and `` `mpm install` in <tree> ``.
- Unit tests of `prepare_launch` in `launcher_tests.rs`:
  a deleted program is replaced by the one under `current`,
  an existing program is never replaced,
  and a program missing under `current` stays as recorded.

Before the fix the first three failed
(exit 127 with the message above),
and the two controls of the unit tests passed.

## Fix

`package/mpm-shadow/cli/src/toolchain/launcher_current_link.rs`:
`program_to_run` returns the recorded program when it exists,
otherwise the program of the same file name found under the `current` link above it,
and `missing_program_advice` is the sentence printed after `could not start` when nothing is found.
The README section "The launcher" of `package/mpm-shadow/cli` describes both.

## Limits

- A launcher is a hard link of the binary that last ran `install` in its tree,
  so a tree whose launcher predates the fix still fails until `mise run toolchain:install` runs in it.
  Running it also gives the tree a fixed launcher.
- A launcher never resolves the working directory or `PATH`,
  so a process started from another checkout's shell runs that checkout's launcher.
  Both run the same newest copy.
  The brief's wish that a worktree run its own locked version cannot hold for the repository's own tools:
  older versions are deleted by the owner's rule,
  so the worktree runs the newest copy and the warning names the locked version, the version that runs and the command.
- 174 records in the same scan name another tree's lock file
  (a snapshot of `.cache` into a new worktree where `install` has not run);
  that is a separate condition and is not changed here.

## Upstream filing decision

Not applicable:
`mpm-shadow` is this repository's own tool,
so there is no third party to report to.
