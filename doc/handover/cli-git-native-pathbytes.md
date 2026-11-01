# cli-git native `pathBytes` through the commit path

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose and how to respond

This records the trailing gate for the optional `pathBytes` field of the native JSONL events.
The owner adopted the field on 2026-10-06 (`doc/planning/cli-git-rust-open-decisions.md`,
section "Non-UTF-8 paths in events and journals", option C):
a file name that is not valid UTF-8 appears in an event as a readable `path` plus an optional `pathBytes`
with the exact bytes, and a reader can rely on "no `pathBytes` means `path` is the exact name".
The field only adds; escaping in place would change every name that contains a backslash.

The Markdown autofix lane had built the field for `git add`, `check` and `fix`.
This lane asked whether every path-carrying event keeps such a name exactly, from a real commit through the
journals to the event, and closed the gaps it found.

What to inspect:

- the matrix in section "What the commit path does with a bad name",
  which says what the output looks like on the code before this lane and today;
- the contract and its one exception in `package/git-policy/cli/SPEC.md`
  (section "Finding event" and the journal paragraph of section "Transaction directory and journal");
- the choices open to veto in section "Choices open to veto";
- the mutation survivors and their reasons in section "Mutation".

How to respond: veto any choice in "Choices open to veto" by naming it;
silence adopts them.
Merge the branch `feat/cli-git-native-pathbytes` when the gate evidence satisfies you.

## What existed before

`src/native/event_path.rs` already had `EventPath` (readable text with U+FFFD, plus the exact bytes only when
the name is not UTF-8) and standard padded base64.
Findings and engine failures printed `pathBytes`, and the fix summary printed `changedPathBytes`,
so `git add`, `check` and `fix` handled such a name.

The commit path did not.
The native copy of the retired TypeScript wrapper decoded every Git listing with strict UTF-8,
so any commit that named such a file stopped.
Measured on the base commit with the 7 red controls of commit `e613bda1cd` (4 failed, 3 passed):

- a commit of the file, with or without a policy finding, failed;
  with a finding the output was `error: pathspec 'bad\u{fffd}.txt' did not match any file(s) known to git`,
  and without one `git add --all -- bad\u{fffd}.txt failed: fatal: pathspec ... did not match any files`,
  because the exact bytes had been turned into replacement characters on the way to Git;
- a replay conflict over such a name failed;
- an untracked name beside a tracked commit, an `add` blocked at error severity, and the shape of names that are
  valid UTF-8 (including a backslash) already behaved correctly.

## What was added

Every path in the commit transaction is now exact bytes (`Vec<u8>`) from Git's listing to Git's next argument,
and only a message shown to a person is text.

- `src/native/path_bytes.rs`: `os_from_path_bytes` (the bytes as an operating-system string on Unix) and
  `path_message_text` (the readable text for messages).
- `src/native/event_path.rs`: `base64_decode_standard`, strict and canonical, for reading the journals back.
- The commit pipeline (`commit_transaction`, `commit_index`, `commit_convergence`, `commit_conclusion`,
  `commit_landing_loop`, `commit_revalidation`, `commit_preparation`, `commit_no_change`, `commit_worktree_records`,
  `private_index_stage`, `landing_attempt`, `landing_index`, `worktree_completion`, `replay_merge`, `replay_commit`):
  the strict UTF-8 refusals are gone; pathspecs are built as bytes, including `:(top,literal)` ones.
- The journals (`transaction_journal*`): `selectedPathspecBytes`, `committedPathBytes`, and `pathBytes` in each
  added-path object, written only when some path is not UTF-8 and checked on reading
  (the text must equal the readable text of the bytes, or the record is malformed).
- `candidate_store.rs`: the pushed-tree listing of a manual push (`added_raw_from_ls_tree`)
  read each path as lossy text,
  so a name that is not UTF-8 reached the content policies with replacement characters and no `pathBytes`;
  it now keeps the exact bytes.
  This gap was found by a sweep of every `from_utf8` and `from_utf8_lossy` call in `src/native` for file paths,
  and the other call sites handle object names, refs or messages.
- `policy_events.rs`: the replay conflict event carries `pathsBytes`, entry for entry with `paths`,
  when any conflicting name is not UTF-8.
- `SPEC.md`: the reader contract, the one exception, `pathsBytes`, and the journal fields.

Output for names that are valid UTF-8 is byte for byte what it was.
The regression control `names_that_are_utf8_keep_the_old_shape` runs a backslash name (`a\b.txt`) and a name that
holds U+FFFD itself (`caf\u{fffd}.txt`) through `add` and expects the old line, with no `pathBytes`.
The journal controls `names_that_are_utf8_get_no_byte_fields`
and `a_record_without_byte_fields_reads_its_text_as_the_exact_name`
hold the same for records.

## What the commit path does with a bad name

The 17 binary controls in `src/native/binary_event_path_commit_tests.rs` run the built `cli-git-native` in a throwaway
repository with the file name `bad` + byte `0xff` + `.txt`, unless the line says otherwise.
Today every one passes.

- A clean commit lands; `HEAD` holds the exact bytes and the output is empty.
- A commit with an autofix correction prints a fix summary with `changedPaths` and `changedPathBytes`
  (`YmFk/y50eHQ=`), and the corrected blob lands under the exact name.
- A blocking finding selected by `--include` (the finding has fix `none`) prints the finding with `pathBytes`.
- An `add` blocked at error severity prints the finding with `pathBytes`.
- A replay conflict prints `concurrent-commit/replay-conflict` with `paths` and `pathsBytes`.
- `commit -a`, a staged name in index mode, an amend, and a pre-commit hook that restages the file all land it.
- A no-change normalization names the file exactly.
- A clean replay of a commit that lost the landing race lands it, for the bad name and for `café.txt`.
- A commit killed at `ref-updated` (through `CLI_GIT_TEST_ONLY_PHASE_SIGNAL`) is completed by recovery
  from journals that hold the bytes.
- An untracked bad name does not stop a commit of other files.
- A Markdown file with an unchecked bad name and a failing linter reports the exact bytes.
- The `security/forbidden-strings` finding shows the scanner's masked display (`bad\xff.txt`) and has no
  `pathBytes`, by design (see "Choices open to veto").

22 test files were added or changed, and 44 test functions are new (27 unit and 17 binary).

## Positive control

The red controls of commit `e613bda1cd` were run on the base code in a throwaway worktree:
4 of 7 failed with the messages above and 3 passed, as listed in "What existed before".
The new control `a_pushed_tree_lists_a_name_that_is_not_utf8_exactly` fails on the code before the listing fix
(it listed `bad\u{fffd}.txt`) and passes after.
The 6 planted guard removals added to `bin/native-planted-controls.mjs` were each planted on the host and each
failed its named test.
The planted phase of the campaign was not run in a container (see "Trailing gates").

## Gate results

The gated source is commit `f23cb2f247` (the later commits of this branch change documents only).

- Container gate (`GIT_POLICY_NATIVE_IMAGE_TAG=pathbytes`), judged by the test result lines:
  1688 unit passed, 0 failed, 5 ignored; 135 binary passed, 0 failed, 1 ignored; 1 consumer passed.
  Image `sha256:37b69d10e5e7660ae2f3a345dddcd6c641af12a8c360ac4ecc466d30252faace`.
- `native:clippy` with `-D warnings`: finished clean on the same head.
  An earlier run found one shadowed binding in `landing_index.rs`, fixed in `7f6b8801d0`.
- Git is 2.56.0 on the host and in the image; the brief named 2.55.0.

## Mutation

The scope is the 21 changed non-test source files of `src/native`, 610 mutants,
run with `NATIVE_MUTATION_SKIP_PLANTED=1`
(the planted phase was skipped for speed; the 6 new controls were checked on the host instead).

- Pass 1 (image `sha256:4cda49b9c3c71c45fe060496a7c582b6a2c46bc45cd226bd70a70e2d1a85175c`, source `1a8b83ff9f`):
  610 mutants, 480 caught, 79 unviable, 25 missed, 26 timeouts.
  A first start failed its baseline on an existing control,
  `event_path::events_carry_the_exact_bytes_of_a_name_that_is_not_utf8`,
  which hit its 5-second command bound on a loaded host; the second start passed its baseline.
- Killing tests were added in `8d3b29aff7` for 13 of the 25 survivors
  (identity times, socket and symbolic link inspection, non-conflict error, oldest winning commit, stage-zero entry),
  and each was shown to fail on its mutant.
- Pass 2 (image `sha256:84ac61c07e79396f9101132cc9d7603b8b2aff6fef914a7508d50a980c454f34`, source `8d3b29aff7`)
  over the lines of the survivors and the timeouts: 73 mutants, 39 caught, 12 missed, 22 timeouts.
- Pass 3, the timeouts again at `--timeout 900`: 28 mutants, 26 caught, 2 timeouts.
- Pass 4, `candidate_store.rs` after the listing fix, only the mutants of `added_raw_from_ls_tree`
  (image `sha256:37b69d10e5e7660ae2f3a345dddcd6c641af12a8c360ac4ecc466d30252faace`): 7 mutants, 7 caught.
  The source of pass 1 differs from the final source only by tests, the listing fix and its test.

The 12 survivors left:

- `event_path.rs:237:33` and `237:42`, `|` to `^` in `base64_standard`: the three operands occupy disjoint bit
  ranges (a byte shifted by 16, by 8, and a byte), so `|` and `^` give the same value for every input.
  Equivalent.
- `worktree_completion.rs:243:47`, `|` to `^` in `open_nonblocking`: `O_NOFOLLOW` and `O_NONBLOCK` are distinct single
  bits, so `|` and `^` are equal.
  Equivalent.
- `landing_attempt.rs:584:48`, `==` to `!=` in `advance_target`: the flag only adds `--no-deref` to `git update-ref`
  (`landing_support.rs`, `compare_and_swap_target`).
  The ref it moves is `target_ref`, which `capture_invocation.rs` takes from `git symbolic-ref --quiet HEAD`
  (recursive by default; on a throwaway repository with `HEAD` to `refs/heads/alias` to `refs/heads/main` it printed
  `refs/heads/main`, and `--no-recurse` printed `refs/heads/alias`) or sets to `HEAD` when that command exits 1.
  So the ref is never symbolic, and `--no-deref` changes nothing either way.
  Equivalent.
  Code this lane did not change.
- `worktree_completion.rs:302`, `303`, `304`, `316`, `317`, `318`, `319` and `320`, `||` to `&&` in `inspect_or_fail`:
  the checks compare the file as opened with the file as listed, and the file after the read with the file before.
  each operand compares two system calls on one path, and with no writer between them every operand is false,
  so `||` and `&&` give the same answer.
  The guards exist for that race; no test can reach a differing operand without a hook between the calls,
  so these are justified as impossible to catch deterministically, not killed.
  Code this lane did not change.

The 2 timeouts left, `landing_attempt.rs:419:8` and `592:8` (`delete !` in `land_locked` and `advance_target`):
each makes a normal landing report a lost race with a replayable head, and the loop at
`commit_landing_loop.rs` (`loop {`, no attempt bound) replays and lands again without end.
The first run's logs show 5 and 4 binary controls failing for them
(`a_commit_of_all_tracked_changes_lands_them` among them) while the library suite passes,
and the binary target then never finishes.
Caught by failure and by non-termination; the harness reports it as a timeout.

The other 20 timeouts of pass 2 were caught at `--timeout 900`.

## Limits

- Messages show the readable text only.
  `-m`, `-F` and similar commit-argument values stay text (`commit_args.rs`), and pathspec removal compares text.
- A `--pathspec-from-file` source is read as text (`pathspec_file_source` returns `Option<String>`);
  no control feeds it a name that is not UTF-8.
- The replay and capture modules keep Latin-1 strings internally, consistently within themselves.
- Copying ignored state into a new linked worktree was not exercised with such a name.
- `os_from_path_bytes` has a lossy branch for platforms that are not Unix; the Windows target is not installed here,
  so `native:clippy:windows` was not run.

## Choices open to veto

- The replay conflict list is `pathsBytes`, a parallel array (as `changedPathBytes` is for the fix summary).
- The journals use parallel arrays for lists and a `pathBytes` field in each added-path object,
  and they stay byte for byte unchanged when every path is UTF-8.
- A record whose bytes do not read as the text beside them is malformed.
- `security/forbidden-strings` never carries `pathBytes`, because the exact bytes would undo its masking;
  this is the one exception to the reader rule, written in `SPEC.md`.
- Messages stay readable text with U+FFFD, and the exact bytes travel only in the fields.
- The planted phase was skipped in the campaign and the 6 new controls were checked on the host.

## Trailing gates

- The planted phase of the mutation campaign for the 6 new controls, in a container:
  `bin/mutate-native-container.mjs` runs it (through `bin/native-planted-controls.mjs`)
  when `NATIVE_MUTATION_SKIP_PLANTED` is not set.
- The Windows checks (`native:clippy:windows`), because the target is not installed.
- The race guards of `inspect_or_fail`, which need a concurrency hook.
- The long fuzz campaigns; no fuzz target was added for the journal fields.
- A merge gate rerun after the merge with `main`.

## Rerun commands

Run these from `.claude/worktrees/cli-git-lane-pathbytes` (or the merged checkout).

```sh
# doc/handover/cli-git-native-pathbytes.md
GIT_POLICY_NATIVE_IMAGE_TAG=pathbytes node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:test:container
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:clippy
mise run //package/git-policy/cli:native:test:binary:host -- event_path_commit
```

The scoped mutation campaign, with the files and the skipped planted phase named above:

```sh
# doc/handover/cli-git-native-pathbytes.md
NATIVE_MUTATION_SKIP_PLANTED=1 GIT_POLICY_NATIVE_IMAGE_TAG=pathbytes \
  node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:native:mutation:scoped -- \
  --file src/native/commit_conclusion.rs \
  --file src/native/commit_convergence.rs \
  --file src/native/commit_index.rs \
  --file src/native/commit_landing_loop.rs \
  --file src/native/commit_no_change.rs \
  --file src/native/commit_preparation.rs \
  --file src/native/commit_revalidation.rs \
  --file src/native/commit_transaction.rs \
  --file src/native/commit_worktree_records.rs \
  --file src/native/event_path.rs \
  --file src/native/landing_attempt.rs \
  --file src/native/landing_index.rs \
  --file src/native/path_bytes.rs \
  --file src/native/policy_events.rs \
  --file src/native/private_index_stage.rs \
  --file src/native/replay_commit.rs \
  --file src/native/replay_merge.rs \
  --file src/native/transaction_journal.rs \
  --file src/native/transaction_journal_encode.rs \
  --file src/native/transaction_journal_parse.rs \
  --file src/native/worktree_completion.rs
```
