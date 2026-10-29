# Scratch locations of cli-git

This note records where the native wrapper (`package/git-policy/cli`, binary `cli-git-native`, installed as `git`)
keeps its short-lived private files, and why none of them is in the system temporary directory.
It answers issue 1810 for this crate; issue 1809 is the incident behind it.

## Rule

The crate never calls `std::env::temp_dir`, `tempfile::tempdir`, `tempfile::tempfile`, `TempDir::new` or
`NamedTempFile::new`, and no code path falls back to `TMPDIR` or `/tmp`.
Which device backs the system temporary directory is unknowable: a small tmpfs, a per-user quota or a slow disk.
A full per-user quota there broke every session in issue 1809.
The code is in `src/native/scratch_location.rs`.

## Inventory

Five groups of scratch files existed.
Each is listed with its old and new location.

- The linter configuration `cli-git-markdown-*.jsonc` (`markdown_linter.rs`, `markdown_linter_config.rs`):
  was a named file in `temp_dir()`; is now a named file in the per-user scratch directory.
- The linter's input and output streams (`bounded_child.rs`): were anonymous files in the linter's scratch directory,
  which was `temp_dir()`; are now anonymous files in the per-user scratch directory.
- The probe directory `cli-git-push-facts-<uuid>` (`manual_push_probe.rs`): was a directory in `temp_dir()`;
  is now a directory in the per-user scratch directory.
- The push output file (`bounded_push.rs`) and the query output files (`bounded_query.rs`): were anonymous files made
  by `tempfile::tempfile()`, which uses `temp_dir()`; are now anonymous files in the per-user scratch directory.
- The input file `cli-git-input-<uuid>` (`transaction_git.rs`, `input_file`): was a file in `temp_dir()` that is
  unlinked as soon as it is open; is now a file in `<git-common-dir>/cli-git-scratch`.

## Two locations

The per-user scratch directory is `<state>/cli-git/scratch`.
`<state>` is an absolute `XDG_STATE_HOME`, otherwise `$HOME/.local/state`; a relative `XDG_STATE_HOME` is ignored as
the XDG rules say, and the lookup mirrors `cache_directory` of the Git version cache.
The directory and each missing parent are created with mode `0700`.
If neither variable gives a path, the command fails with a sentence that names `XDG_STATE_HOME` and `HOME` and says the
system temporary directory is never a fallback; the linter and the probe report it when a candidate or a push needs
the directory, not when they are constructed, because several checks are built with an empty environment as
placeholders and a command with no Markdown candidate must not touch the disk.

The repository scratch directory is `<git-common-dir>/cli-git-scratch`, created with mode `0700` beside the
other `cli-git*` private entries of the common directory.

### Why the input file uses the repository

Every request that carries input belongs to a commit transaction, so a repository exists.
The common directory is on the repository's own device, needs no environment variable (hooks and sandboxes may lack
`HOME`), and is collected with the repository.
`GitContext` gained `common_dir: Option<PathBuf>`, filled by `GitFacts::transaction_context` from the location the
invocation already asked for, at no extra process start.
A request with input and no `common_dir` fails with a sentence that calls the missing directory a defect in cli-git.
The contexts that only run queries (forwarding, the doctor, the coordination queries and the worktree copy)
hold `None`.

A transaction directory was not chosen.
The context a request runs with carries no transaction directory, and the common directory is known to every caller
that sends input.
`memfd_create` would remove the location question for this one site but is Linux specific, so a portable fallback
would be needed anyway.

### Why the others use the per-user directory

The linter is constructed before the repository is known, and its placeholders have an empty environment.
`PushGit` carries the working directory and the complete environment but not the common directory, so the repository
location would cost one more `rev-parse` start per push (or a field on the three construction sites and the nineteen
test constructions).
The direct `check` and `fix` commands run the same linter.
The environment is available to all of them, so one rule serves them: the per-user directory.

## Security properties

Kept:

- The input file is still created through `write_exclusive_scratch`: exclusive creation, no final link followed, mode
  `0600`, and unlinked at once.
- The linter configuration is still created by `tempfile::Builder` (exclusive creation, mode `0600`), and the
  anonymous files still have no name.
- The probe directory is still created by `create_private_directory` (mode `0700`), a path that already exists is never
  adopted, and `ScratchGuard` removes it on every path.

Changed, and stronger:

- The old parent was a shared directory that every account can write.
  The new parents are private: `prepare_user_scratch` and `prepare_repository_scratch` refuse a scratch path that is a
  link or a file, refuse one that belongs to another account, and tighten a looser mode of one this user owns to
  `0700`.
  Only the final path is checked; an ancestor that is a symbolic link, such as `/home` pointing at `/var/home`, is
  accepted, as every user-owned path under it already is.

Changed, and a limit:

- The probe's hook is executed from the scratch directory, so the filesystem under the state directory must allow
  execution; `/tmp` mounted `noexec` had the same limit.
- A command whose environment has neither an absolute `XDG_STATE_HOME` nor a non-empty `HOME` now fails where it used
  to succeed; the input file is not affected, because it does not read the environment.

## Collection of stale entries

Every preparation of either directory removes the entries last modified more than a day ago (`sweep_stale`).
A link is removed itself and never followed; a directory is removed with its contents; an entry dated in the future is
kept, so a wrong clock cannot delete live files.
A day is far longer than any named entry lives: the configuration file lives as long as one invocation (each linter
run is bounded to 60 seconds), the probe directory as long as one probe, and the input file for the moment between
its creation and its unlinking.

Which kinds can be left behind by a kill:

- The anonymous files have no name from the start, so a kill leaves nothing.
- The input file is named for the moment between creation and unlinking; a kill inside that moment leaves one file,
  which the next preparation in the same repository sweeps after a day.
- The configuration file and the probe directory are removed by their owners on normal and error paths; a kill leaves
  them, and the next preparation in the same state directory sweeps them after a day.

The bound of the leak is therefore one entry per kill, collected by the next run that uses the same directory.
A repository that is never used again keeps at most the one input file of a kill inside that moment.

## Controls

Each production site has a child-process control (`binary_markdown_tests.rs`, `binary_push_environment_tests.rs`) that
runs the built executable with `TMPDIR` naming a path that does not exist and checks that the command works, that
nothing creates that path, that the scratch entry exists in the new location while the child runs (the recorded
`--config` path, the probe directory seen by the transport shim, the common directory entry), and that the directory is
empty afterwards.
The auto-push after a commit has the same control.
Library controls cover the environment lookup, the privacy checks, the link refusal, the sweep and the missing-home
sentence (`scratch_location_tests.rs`), the input file's location and unlinking (`transaction_git_tests.rs`), the
linter (`markdown_linter_tests.rs`) and the probe (`manual_push_probe_tests.rs`).

The controls were shown failing before the change: with the old code the three binary controls failed with the
unusable `TMPDIR` named in the error (`private probe directory ... could not be created`, `could not create the
temporary linter configuration`, `pack-objects ... could not start`).
They were shown failing a second time by breaking the new location (renaming the scratch directory) and by dropping the
unlink of the input file, each restored afterwards.

## Tests

The unit tests use the fixture root under the target directory.
Contexts and environments that name no repository get a directory beside the fixtures (`git_context`,
`scratch_environment`).
`test_support_tests.rs` proves the fixture root is not the system temporary directory by comparing against `TMPDIR`,
`/tmp` and `/var/tmp` without calling `std::env::temp_dir`, so it needs no `#[expect(clippy::disallowed_methods)]`
and is clean with and without the lint configuration that bans it.
String arguments such as `-C /fixture/repo` in the rule tests name no directory the tests touch.

## Not done

- No change to `.github/workflows`, to the gate runner scripts, or to the TypeScript end-to-end fixtures.
- The root `clippy.toml` ban belongs to another change; the crate was checked against a local copy of it that was not
  committed.
