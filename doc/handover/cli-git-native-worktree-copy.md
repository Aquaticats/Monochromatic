# Native worktree-copy lane (slice 11)

> References below this note that no longer resolve in the tree.
> Each entry gives the value that identifier has now:
> `package/git-policy/cli/native-interop` is deleted (2026-11-01 `cd67a90f6c`) and the
> old-format interop harness has no replacement.
> `package/git-policy/cli/bin/native-planted-controls.mjs` is now
> `package/git-policy/cli/bin/native-planted-controls.ts` (2026-10-29 `418c4dcc54`).

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Note 2026-10-29:
the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now.
Read every `.mjs` file name here as the same
name ending in `.ts`;
the driver `old-format-fixtures` is `old-format-driver`.
Those folders are `package/git-policy/cli/bin`,
`package/git-policy/cli/native-interop` and `package/git-policy/cli.fuzz/bin`.

## Purpose

This lane ports the linked-worktree ignored-state copy from `package/git-policy/cli/src/worktree-copy/*.ts` to native
Rust modules under `package/git-policy/cli/src/native/worktree_copy*.rs`.
The port covers applicability and bypass,
created-worktree detection,
ignored-state selection,
copy-on-write staging,
installation with exact existing-entry
checks and rollback,
journals and the install log,
the settlement lock,
crash recovery,
the summary line and exit
codes,
and a small public entry point for the integrator.

The commit refusal is not wired to this code.
`Unported::CommitTransaction` stays in place.

## Changes

Commits on `feat/cli-git-native-worktree-copy`,
newest last in the lane:

- `13a443921f` ports the registry,
  selection,
  manifest,
  compare and stage modules.
- `e592c72fd3` ports the journal,
  install log,
  install,
  rollback,
  lock,
  recovery,
  transaction and lifecycle modules.
- `1c4333924d` words raw failures at the nearest owner,
  adds the seven planted controls and the incumbent-journal
  fixture test.
- `ee58aaade6` and `8405c79db6` satisfy the implicit-return,
  shadowing and collapsible-if lints.
- `d2176acbcb` covers the settlement waiter's live-holder line and release.
- `236be820ff` points the rollback planted control at the current comparison.

Shared files touched,
additively only:
`package/git-policy/cli/src/native/lib.rs` (module declarations) and
`package/git-policy/cli/bin/native-planted-controls.mjs` (seven appended entries).

## Public entry point

The integrator calls `worktree_copy::run_with_worktree_copy` once per forwarded command.
The caller supplies the
bypass facts (`--no-worktree-copy`,
short-circuit help or version,
whether the command creates or moves worktrees
after alias resolution),
the repository identity,
the effective working directory,
and a runner that starts real Git
with the lease token in its environment.
`exit_code` turns the returned `Settlement` into the process status,
and
`summary_line` renders the one-line summary.
The module docs state the contract.

## Gates

Container gate,
final commit:
`mise run //package/git-policy/cli:native:test:container` through the slot limiter.
The image tagged `localhost/git-policy-native-test:worktree-copy` has id
`53dd88eb33b88a928fdbb22305372796603375cd01078dd0c3f92cf3280e9a65`.
The gate failed.
The lib test target stopped at Clippy with four findings in files this lane does not own:
`src/native/invocation/entry_tests.rs:114`,
`src/native/lock/index_lock_wait_tests.rs:190` and `:201`,
and
`src/native/index_writer_coordination_tests.rs:192`.
The coordinator said these belong to the transactions branch.
The binary and interop targets therefore did not run in this gate.

Results before that failure,
in the same container image:

- Library unit tests:
  963 passed,
  2 ignored.
- Earlier container run (before the final commits) on the same tree apart from the rollback control fix:
  the binary
  target had 36 passed and 3 timed out at the 5-second bound under host load.
  Those timeouts are in the native binary
  frontier tests,
  not in worktree-copy code.

Host run:
`mise run //package/git-policy/cli:native:test:host -- worktree_copy` passed for the worktree-copy modules
(108 tests against Git 2.55.0 on the host).
The install-entry tests were rerun at the final commit and pass (6 of 6).

Clippy:
the worktree-copy files have zero findings.
The four container findings above are in other lanes' files.

## Mutation evidence

Container scoped run,
`mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/worktree_copy*.rs'`
(log `wc-mutation-2.log`),
started after the planted-control fix:

- 454 mutants:
  302 caught,
  45 unviable,
  45 missed,
  62 timed out.
- Planted controls:
  all seven worktree-copy controls were noticed,
  each by a named test.
  The campaign also noticed the
  nine controls from other lanes.
- The campaign exited 1 because cargo-mutants exits non-zero on any missed or timed-out mutant.

Host filtered run,
`cargo mutants --no-config --in-place --baseline run --timeout 120 --file 'src/native/worktree_copy*.rs'`
with the same two excluded kinds and the test filter `-- worktree_copy` (output `mut-host-all`):

- 176 mutants listed:
  121 caught,
  18 unviable,
  35 missed,
  2 timed out.
- The mutant count differs from the container run (176 versus 454).
  This is not explained yet.
  The filter reduces
  the test set per mutant,
  so the host run reports a different set of mutants and outcomes,
  and its results are not a
  substitute for the container run.

Causes seen in the timeouts:
several mutants make a blocking lock acquisition wait forever on a holder the test itself
owns (for example `validate_inherited_lease` returning `Ok(false)` in the lifecycle inherited-lease test).
These are
hangs in the tests rather than slow passes,
and the campaign counts them as timeouts.
The missed mutants cluster in
journal path and stage validation,
lock release and retirement branches,
and recovery notices.
Individual survivors
have not yet been proven equivalent or killed by new tests.

## Deviations

- Copy-on-write staging uses Linux `FICLONE` with a byte-copy fallback.
  Other platforms are not covered.
- Install,
  rollback,
  journal and lock modules are Unix only,
  because they depend on modes,
  links and owner checks.
- Rust io and JSON error texts differ from Node's.
  Tests assert the wording before the cause.
- Early modules (registry,
  selection,
  manifest,
  compare,
  stage) were written before their tests in some commits,
  so
  test-first order was not followed there.
- The reflink and non-reflink filesystem paths are not exercised on separate filesystems.
- The two-process settlement race is not tested.
- The alias and flag-position `--no-worktree-copy` consumer tests belong to the wrapper modules.
  This module receives
  the bypass facts only.
- The excluded mutant kinds (`replace += with *=`,
  `replace -= with /=`) are inherited from the repository runner.
- No fuzz target was added for worktree-copy input.

## Choices open to veto

- The lifecycle takes the bypass facts from the integrator instead of parsing argv itself.
- `run_with_worktree_copy` takes an `identity_after_lock` closure to refresh the repository identity after the lock is
  acquired.

## Trailing gates

Skipped or not yet run in this lane:

- Full container gate with the binary and interop targets.
  Blocked by the four Clippy findings in transactions-branch
  files.
  Rerun after merging the transactions branch:
  `GIT_POLICY_NATIVE_IMAGE_TAG=worktree-copy node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:test:container`
- Container scoped mutation over `src/native/worktree_copy*.rs`,
  to triage the 45 missed and 62 timed-out mutants.
  Mutation campaigns trail the cutover by coordinator decision on 2026-10-11,
  and the host unit was stopped as
  instructed.
  Rerun command:
  `GIT_POLICY_NATIVE_IMAGE_TAG=worktree-copy node ~/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/worktree_copy*.rs'`
- Host filtered mutation,
  for faster iteration on survivors:
  `cargo mutants --no-config --in-place --baseline run --timeout 120 --file 'src/native/worktree_copy*.rs' --exclude-re 'replace \+= with \*=' --exclude-re 'replace -= with /=' -- worktree_copy`
- Reflink and non-reflink filesystem tests,
  and the two-process settlement test.
- Fuzz smoke for worktree-copy input.

## A repository path of the copy is bytes, and so is every record that names one (2026-11-09T15:53:42-05:00)

Branch `fix/git-policy-cli-worktree-copy-ignored-names-keep-bytes`.

The copy kept a repository path as text.
The manifest walk read each name the filesystem listed with `to_string_lossy`, and the
journal, the install log and the stage claim carried the result, so a name that is not
UTF-8 became a name holding U+FFFD.

Measured on the real binary (`target/debug`, real Git 2.56.0, a fixture whose ignored
`heavy/` held `bad\xff.bin`, a name with a newline and a name with a tab, beside an
ignored file `top\x80.log`): `git worktree add` ended with status 2 and the bare text
`No such file or directory (os error 2)`, copied nothing, and left the worktree registered
but unfilled.
A second fixture, holding `heavy/bad\xff.bin` next to a file whose name is exactly the
lossy spelling, failed with `File exists (os error 17)` while staging.
So no name was silently renamed: the copy refused, but it could not carry the ignored
state of such a repository at all.

Every repository path is now the bytes the filesystem reported, from the selection through
the manifest, the stage, install, rollback and the records; a diagnostic renders them with
replacement characters only where a person reads one.
The order of a manifest is the UTF-16 order of the path's text, then its bytes, so a name
that is text keeps the order the incumbent gave it and a name that is not text has a
reproducible place.

The record rule, in `foundation/path_bytes_in_records.rs`:
the four stage roots keep the text field and the bytes field, always;
a list (`selectedRoots`, `intendedEntries`, an install log's `intended`) and a single
`relativePath` keep the text form only when the text spells every byte, and otherwise the
bytes form alone.
A record the previous release wrote holds text only and is read as it is; a record this
release writes for a name that is not text has no text field, so that release refuses it
(the same rule `root_fields` already used) instead of acting on a replacement spelling.
Reason for the difference: an install log's created-entry lines dominate the record size,
so a name that is text costs nothing extra.

Evidence, after the change:

- the same fixture copies every name exactly (`bad\xff.bin`, the newline name, the tab
  name, `top\x80.log`, and an ignored directory whose own name is `dir\xfe`), and the
  destination holds no replacement spelling;
- four cuts killed the wrapper with `SIGKILL` on its own process group and let an ordinary
  read-only command recover: on the staged journal (0 of 2,003 entries installed), after
  the awkward names were installed, and after the whole install before the cleanup.
  Each recovery reported `recovered ignored-state copies for 1 worktree transaction`,
  finished with 2,003 entries, missed none of the four names and left no replacement
  spelling;
- proof by removal: with the walk's bytes replaced by `to_string_lossy` again, the new
  manifest control fails with `manifest: WorktreeCopyError("No such file or directory
  (os error 2)")`.

Controls added: `a_name_that_is_not_text_is_one_entry_with_its_own_bytes`,
`an_ignored_root_whose_name_is_not_text_is_selected_byte_for_byte`,
`a_named_ignored_file_whose_name_is_not_text_is_found_by_its_bytes`,
`a_repository_path_is_joined_from_its_parent_and_a_name`,
`repository_path_order_keeps_the_incumbents_order_and_settles_equal_text_by_bytes`,
`a_list_and_an_entry_keep_their_bytes_without_losing_the_earlier_release`,
`a_batch_holding_a_name_that_is_not_text_round_trips_through_its_bytes`,
`a_journal_keeps_the_bytes_of_its_lists_and_an_earlier_release_refuses_one_that_has_no_text`
and `recovery_installs_an_ignored_name_that_is_not_text_under_its_own_name`.

Open, found while driving those cuts and not changed here:
a kill in the middle of one destination file leaves that file partly written, because the
install copies bytes when the destination worktree is its own btrfs subvolume (a hard link
across subvolumes is `EXDEV`), and the resumed copy refuses it as a differing entry:
`ignored-state copy would overwrite differing destination entry "heavy/zzz-pad-01016.txt"`,
measured 0 bytes at the destination against 13 in the source.
The copy then ends without finishing and removes its stage.
Nothing of the source is lost, and the person is told, but the worktree stays unfilled.

Commits: `87dbe215b0` (the bytes), `91ef3e7485` and `389919c65c` (the records' controls and
the corrected hexadecimal comparison).
