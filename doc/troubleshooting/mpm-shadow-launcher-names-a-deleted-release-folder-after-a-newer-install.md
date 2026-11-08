# mpm-shadow 0.2.7-alpha.2 launcher exits 127 in every other tree after a newer release of a repository tool is installed

Recorded 2026-11-07T13:05:00-05:00.

Status:
fixed in `monochromatic-mpm-shadow` 0.2.7-alpha.4 (the launcher follows `current` since 0.2.7-alpha.3 and rewrites its stale record since alpha.4)
and `monochromatic-lint` 0.5.2-alpha.2 (the closure check fails instead of skipping);
the lock has named them since commits `ee2d870060` and `5cd86b210f`
(`doc/planning/one-release-each-batch-2026-11.md`,
section "Release record:
`monochromatic-mpm-shadow` 0.2.7-alpha.3 and 0.2.7-alpha.4").
A tree whose launcher binary predates alpha.4 keeps failing until `mise run toolchain:install` runs in it.
The two follow-ups this file left open are built on branch `feat/mpm-shadow-launchers-from-newest-install`
(2026-11-08):
a launcher is a copy of the newest installed `mpm-shadow`,
not of whichever copy ran `install`
(the owner's pick with the veto open),
and
the older-lock sentence names the lock file it compared.
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
(`package/mpm-shadow/cli/src/toolchain/layout.rs`,
`find_tool_bin`,
which searches `<tool dir>/current` and descends into the archive folder).
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
(printed `PATH` entries,
2026-11-07).
Mise and the inherited `PATH` are not the cause,
and in a freshly made worktree the test run of `//package/mpm-shadow/cli:test` ended with 581 passed and exit 0,
printing only the older-lock warning.

`mise run toolchain:install` does repair a stale record:
in the disposable worktree it printed `0 installed, 66 unchanged, 0 failed`
and rewrote `real` to the 0.1.5-alpha.1 folder.
The agent's report that it did not clear the failure was not reproduced.

## Second symptom: `rust/avoidable-closure` skipped with exit 0

The agent of the linter package reported on 2026-11-07 that `mise run lint:monochromatic-lint` silently skipped
`rust/avoidable-closure` in a worktree whose lock named an older `cargo-shadow` than main had installed.
The cause is the same dangling launcher:
`dylint_probe` in `package/linter/monochromatic-lint/src/closure_cargo.rs` ran `cargo dylint --version` with standard error discarded,
treated every non-zero status as "cargo-dylint is not installed",
and `run_closure_step_at` in `closure_step.rs` turned that into
`rust/avoidable-closure skipped all N package(s): ` followed by `` `cargo dylint --version` failed (exit status: 127); install cargo-dylint and dylint-link with mise install ``.
That is a skip with exit 0 and a wrong remedy.
The red run of the new test printed exactly that text for status 126 before the change.
Now a `cargo` that starts and exits 126 or 127 fails the run with a processing failure that quotes the `cargo` standard error,
which for the launcher fix above includes the tool,
both versions and `` `mpm install` in <tree> ``.
A `cargo` missing from the machine and a `cargo` answering 101 (no `cargo-dylint`) stay documented skips.
The tests are `a_cargo_that_cannot_start_its_program_fails_the_run_instead_of_skipping` (`closure_step_tests.rs`)
and `a_cargo_that_exits_126_or_127_is_broken_and_any_other_failure_is_absent` (`closure_cargo_tests.rs`).

## Verification

Under test:
the mpm-shadow of this checkout after the fix,
with tests that build the layout above in a disposable directory
(`package/mpm-shadow/cli/tests/launcher.rs`,
`MovedArchive`):

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

### Third layer: the shadow behind the launcher took the launcher for the real tool

With the first fix alone,
a launcher whose record was stale ran `cargo-shadow`,
which looked for the real `cargo` on `PATH`.
Its launcher skip (`leads_back_to` in `package/cargo-shadow/cli/src/real_binary.rs`) canonicalizes the record's `real` path;
the deleted folder cannot be resolved,
so the launcher was not skipped,
was chosen as the real `cargo`,
started `cargo-shadow` again,
and the recursion guard stopped `mise run //package/mpm-shadow/cli:lint` with
`cargo-shadow: stopped at once, before doing anything, because cargo-shadow was started by cargo-shadow.`
(measured 2026-11-07 in the disposable worktree with a copy of the fixed launcher and a record that named the deleted 0.1.4 folder).
The launcher now rewrites its own record to the program it started
(`refresh_stale_record` in `launcher.rs`;
tests `a_launcher_rewrites_a_stale_record_to_the_program_it_started_so_a_shadow_behind_it_can_see_where_it_leads`
and `a_record_that_cannot_be_rewritten_is_a_warning_and_the_program_still_runs`),
so the record is true again before any shadow reads it.
`pnpm-shadow` has the same skip rule and is covered by the same rewrite;
only `cargo-shadow` was run in this measurement.
Corrected 2026-11-07:
`search-shadow` does not have that rule (it had no launcher-record rule and no guard),
and its links are not launchers;
see `doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`.

## Limits

- A launcher is a hard link of the binary that last ran `install` in its tree,
  so a tree whose launcher predates the fix still fails until `mise run toolchain:install` runs in it.
  Running it also gives the tree a fixed launcher.
- A launcher never resolves the working directory or `PATH`,
  so a process started from another checkout's shell runs that checkout's launcher.
  Both run the same newest copy.
  The brief's wish that a worktree run its own locked version cannot hold for the repository's own tools:
  older versions are deleted by the owner's rule,
  so the worktree runs the newest copy and the warning names the locked version,
  the version that runs and the command.
- 174 records in the same scan name another tree's lock file
  (a snapshot of `.cache` into a new worktree where `install` has not run);
  that is a separate condition and is not changed here.

## Recurrence 2026-11-07T21:50:00-05:00: `git` fails in a systemd user unit through the machine-global launcher

Status:
repaired on this machine with `mise run toolchain:install:global -- --frozen-lockfile`;
the launcher now names the program it starts and never tries a path that does not exist
(`monochromatic-mpm-shadow` 0.2.7-alpha.6,
`package/mpm-shadow/cli/src/toolchain/launcher.rs`).

### Symptom

A Node process of the toolchain,
started in a `systemd-run --user` unit,
ran `git` and got
`mpm-shadow: could not start ~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-git-policy-cli/current/monochromatic-git-policy-cli-0.6.0-x86_64-unknown-linux-gnu/monochromatic-git-policy-cli: No such file or directory (os error 2)`,
also after `mpm update -g cargo:monochromatic-git-policy-cli` moved the global lock from 0.6.0 to 0.6.1-alpha.1.

### Route (measured 2026-11-07)

The parent process does not matter.
The reproduction run with `sh -c 'command -v git; git -C <repository> rev-parse --short HEAD'` in the same kind of unit failed with the same text,
so the brief's note that `sh` works was not reproduced.
What differs between an interactive shell and a unit is `PATH`:
the shell of a mise-activated checkout has that checkout's `.cache/monochromatic-git-policy-cli-bin/git` first (a repository-scope launcher,
rebuilt by every `toolchain:install` in that tree),
and the unit's `PATH` has none of those,
so `git` is `~/.local/bin/git`.
That is a link to `~/.local/share/monochromatic-dev/toolchain-global/launchers/monochromatic-git-policy-cli`,
the machine-global launcher.

### Where 0.6.0 came from

The `real` field of `toolchain-global/launchers/monochromatic-git-policy-cli.launch`,
written at 11:30 when 0.6.0 was current.
Nothing rewrote it afterwards:
`mpm update -g` only publishes the global lock (`update_loaded` in `src/toolchain/update.rs` ends in `publish_lock`),
and launchers and records are made only by `install` (`rebuild_exposure`,
then `ensure_launcher`,
in `src/toolchain/exposure.rs` and `launcher.rs`).
The launcher binary was a copy of `mpm-shadow` 0.2.7-alpha.2 (the copy alone,
with no record beside it,
printed `mpm-shadow 0.2.7-alpha.2` for `--version`),
older than the follow-`current` rule of 0.2.7-alpha.3,
so it started the recorded path as it was.
The warning and the path disagreed because the warning compared the lock with the version behind `current`,
and the start used the recorded folder.
After the update the warning disappeared (lock and `current` agreed) and the start still failed.
The binary was old because `ensure_launcher` places a copy of the running installer,
not of the newest installed `mpm-shadow`:
the global install that made it ran as alpha.2.

A current build (a copy of the repository launcher) given the same record followed `current`,
ran 0.6.1-alpha.1 and rewrote the record (measured in a scratch directory).
So the defect on this route is a launcher binary frozen at an old version,
which no later code can change;
the code defects left in the current launcher were the ones below.

### Fix in the launcher

- The older-lock warning of a launcher ends with `; starting <path>`,
  the program it is about to start (`warning_naming_program`),
  never the path a stale record named.
- A launcher checks that its program is a file before it starts it.
  When it is not,
  it prints `mpm-shadow: not starting <path>: no file exists there` and the one-sentence advice,
  exits 127,
  and prints no warning that says it starts something.
  A file that vanishes between the check and the start still reaches the `could not start` line and the same advice.
- Red tests (all failed before the change):
  `a_launcher_whose_recorded_version_was_deleted_runs_the_version_current_names`,
  `a_global_launcher_whose_recorded_version_was_deleted_names_the_path_it_starts_and_the_global_command`,
  `a_launcher_never_prints_the_older_lock_warning_for_a_program_it_does_not_start`,
  `a_launcher_that_finds_no_program_under_current_names_both_versions_and_the_one_command`,
  `a_missing_real_program_exits_127_naming_it_and_never_writes_standard_output`,
  `a_release_archive_tool_warns_on_every_run_when_the_lock_pins_an_older_version`,
  `an_older_lock_prints_the_warning_on_standard_error_and_leaves_standard_output_byte_identical`
  (`tests/launcher.rs`) and `the_warning_names_the_program_under_current_when_the_recorded_folder_was_deleted` (`launcher_tests.rs`).
  The global-scope test is a guard for the route:
  only its named path was red.

### Repair on this machine

`mise run toolchain:install:global -- --frozen-lockfile` ended with `0 installed, 62 unchanged, 0 failed`,
replaced the global launcher with a 0.2.7-alpha.5 copy and rewrote its record to the 0.6.1-alpha.1 folder.
The reproduction then printed `OK <short hash>`.

### Same class, still open (not changed here)

- Launcher binaries older than the follow-`current` rule survive wherever `install` has not run since:
  a scan of every `.cache/toolchain-launchers/*.launch` in the main checkout and its linked worktrees on 2026-11-07 found 925 records,
  261 of which named a file that no longer exists;
  each of those trees is repaired by `mpm install` in it.
  The global `cli-git-native` launcher (binary of 2026-11-03) still has a record that resolves,
  and keeps the old binary until the next `install -g`.
- `leads_back_to` in `package/cargo-shadow/cli/src/real_binary.rs` (and its siblings in `pnpm-shadow` and `search-shadow`) canonicalizes a record's `real`;
  an unresolvable one is treated as "not a launcher".
  The launcher's own record rewrite covers the current binaries.
- Paths composed from a version string in `layout.rs` (`tool_install_dir`,
  the Android `cmdline-tools/<version>` search directory) are reached only through `is_executable` or `exists` checks,
  and `prune_cargo.rs` builds its paths from directory listings;
  none starts a program without a check.

### A related warning that looked stale

The same session saw `locked at 0.5.2-alpha.5 ... but 0.5.2-alpha.6 runs` in a worktree whose committed lock said alpha.6.
A launcher compares the version behind `current` with the lock named in its own record,
never with the lock of the directory it runs in.
A shell begun in another checkout puts that checkout's `.cache/*-bin` first,
so its launcher answers with that checkout's lock
(measured:
from a worktree whose lock pinned alpha.5,
the main checkout's launcher printed no warning and the worktree's own launcher printed it).
This is the documented limit under "Limits" above and not a stale record:
the records of the worktree launchers name the worktree's lock file.
The warning did not say which lock it read,
which made this hard to see;
it now names the file
(`older_lock_message_for` in `package/mpm-shadow/cli/src/toolchain/cargo_installs.rs`),
so the same message from two launchers is told apart by the path.

## Follow-up built 2026-11-08: launchers are copies of the newest installed mpm-shadow

Branch `feat/mpm-shadow-launchers-from-newest-install`,
package `monochromatic-mpm-shadow`
(no release dispatched from this branch;
the behaviour reaches a machine only once a release carrying it is installed).

### The rule

`ensure_launcher` no longer copies the running `mpm-shadow`.
`launcher_source` in `package/mpm-shadow/cli/src/toolchain/launcher_source.rs`
finds the install directory the running program belongs to
(the nearest ancestor that holds a `current` link,
which covers both `<root>/<tool id>/<version>/<archive folder>/<bin>` and `<root>/cargo/<crate>/<version>/bin/<bin>`),
then copies the program under that `current` link in whichever of the two toolchain roots names the newest version.
A running program that belongs to no install (a build folder,
a copy fetched to a scratch path)
is still copied as it is,
through `/proc/self/exe` when its file was deleted,
so the bootstrap case keeps working.

This also removes the dependency on the running file:
an `mpm-shadow` that installs a newer one has just deleted its own file,
and only a shadow that belongs to no install needs `/proc` now.

### What it fixes, and what it does not

- Fixed:
  a launcher placed from now on carries the code of the newest installed shadow,
  which follows `current` and rewrites a record naming a deleted folder on its first run.
  A dangling record therefore stops breaking the tool,
  and heals the first time that launcher runs.
- Not fixed:
  the launcher binaries already on disk.
  A census of this machine on 2026-11-08 read 1132 launcher binaries
  and found 891 without either the record-rewriting sentence or the both-versions sentence
  (`could not update the launcher record`,
  `no program named`),
  so those trees still fail on a dangling record until `mpm install` runs in them,
  exactly as before.
  145 carried all three markers of the current release,
  88 the record rewrite without the file check,
  8 only the both-versions sentence.
- Not fixed:
  the dangling records themselves.
  A record still names the version-carrying folder below `current`
  (`the_program_a_record_names_is_the_version_folder_a_newer_install_deletes` in `published_archive_tests.rs`),
  so installing a newer version of any repository-published tool deletes a folder that every other tree's record names.
  The census found 1078 records of which 284 named a file that no longer exists
  (2026-11-07:
  925 records,
  261 dangling):
  `mpm-shadow` 121,
  `cargo-shadow` 37,
  `cli-git-native` 37,
  `monochromatic-lint` 35,
  `pnpm-shadow` 27,
  `search-shadow` 27.
  All 284 are followable:
  the program of the same name exists under the nearest `current` link,
  so one run of each launcher with the current code clears its record.
  207 records name another tree's lock file,
  the separate condition under "Limits".
- The census read each `.launch` record and checked whether its `real` path exists,
  and classified each launcher binary by the sentences its code holds
  (the binaries carry no version literal).
  It ran read-only:
  no launcher was started,
  because starting one rewrites its record.

### Tests

New,
in `launcher_source_tests.rs`:
`the_newest_install_in_the_same_root_is_the_source`,
`a_newer_install_in_the_other_root_is_the_source`,
`an_older_install_in_the_other_root_does_not_replace_the_one_that_runs`,
`the_root_whose_current_link_names_no_program_falls_back_to_the_other_one`,
`a_deleted_note_on_the_running_program_leaves_its_file_name_readable`,
`a_running_program_that_belongs_to_no_install_is_the_source`,
and the moved `a_file_that_exists_is_used_as_it_is_and_a_deleted_one_is_read_through_proc`.
`a_launcher_is_a_copy_of_the_newest_installed_mpm_shadow` in `launcher_tests.rs`
starts a copy of the test binary from an old version folder while `current` names a newer one,
first in the tree's own root and then in the machine-global root,
and asserts the launcher's bytes are the newest install's.
Red before the change:
the launcher was the running copy's bytes.
The two controls it needs are unchanged and green:
`a_launcher_is_made_with_its_record_beside_it` and
`a_launcher_is_made_from_the_running_binary_after_its_file_was_deleted`.

## Upstream filing decision

Not applicable:
`mpm-shadow` is this repository's own tool,
so there is no third party to report to.
