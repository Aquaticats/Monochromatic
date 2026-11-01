# cli-git native Markdown autofix

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose and how to respond

This branch makes the native wrapper run `markdown/autofix` for `git add`,
`git cli-git check` and `git cli-git fix`
through the native linter executable `monochromatic-lint`,
and adds the `pathBytes` event field the owner adopted for file names that are not UTF-8.
`markdown/autofix` no longer refuses;
`mono/dependent-version-bump` still does.

Paths starting with `src/` or `bin/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Inspect the sections "Differences",
"Choices open to veto"
and "Not verified".
Respond by merging the branch into `main`
and by vetoing any item under "Choices open to veto".

## Status

The implementation is complete on this branch.
The final gated commit is recorded in "Final gated commit";
the gate, the scoped mutation campaign and the fuzz smoke run pass on it.
The earlier records (merge commit `db2857189` and the first campaigns) are kept below for history.
The one surviving mutant that a test should kill (`markdown_exclude.rs:95:39`) is killed by commit `2e10ab6fb2`,
and the two base64 `|` to `^` survivors are equivalent mutants.
`main` changed `package/git-policy/cli/Cargo.lock` and `package/git-policy/cli.fuzz/Cargo.lock`
(the `monochromatic-jsonc-edit` 0.1.1 release, `f313a8933`) after the branch point `6ed268522`,
so `main` was merged in as `b89b38252`;
the merge had no conflict.

## What changed

### The policy

`src/native/policy_markdown.rs` checks the lifecycle's candidates in candidate order,
as the installed wrapper's `markdownLintPolicy` does
(`package/git-policy/markdown-lint/src/rewrite-candidates.ts`
(deleted 2026-10-13: now `package/git-policy/cli/src/native/policy_markdown.rs`)):

- Deleted candidates,
  symbolic links and submodules are skipped,
  and so is every pathname whose extension is not exactly `md` or `mdx`.
- A candidate the `exclude` patterns name is skipped before its bytes are read
  and before any linter is needed (`src/native/markdown_exclude.rs`,
  the `ignore` crate's gitignore matcher with the linter's walk over ancestor directories).
- A candidate whose bytes are not UTF-8 is skipped.
- Every other candidate goes through the linter.
  A changed source is the finding `markdown-autofix`,
  "monochromatic-lint --fix (markdown/lfs-image-url) rewrites `<path>`.",
  the installed wrapper's words;
  on `git cli-git fix` only it carries a full-content correction,
  which the existing fix loop applies and installs.
  Each finding the fixes leave is `markdown-violation`,
  "`<rule>` at `<path>:<line>:<column>`: `<message>`".
- The first candidate the linter cannot check ends the policy with one `engine-failure` event,
  code `policy-incomplete`,
  whose `path` names that candidate.
  The message states the cause and both remedies:
  fix it,
  or pass `--no-enforce-markdown/autofix` for one command.

### Finding the linter

`src/native/markdown_linter_lookup.rs` takes the first absolute `PATH` entry of the invocation's environment
that holds an executable regular file named `monochromatic-lint`
(with `.exe` on Windows), following symbolic links.
Relative and empty entries are skipped.
The lookup runs once per invocation,
when the first candidate needs the linter,
so a command without a Markdown candidate never needs one.
A missing linter is the settled engine failure:
exit status 2 and nothing forwarded
(`doc/planning/cli-git-rust-open-decisions.md`, "Markdown policy without the linter").

The evidence for `PATH`:
the installed wrapper ran a command chosen by repository configuration,
which the native configuration refuses (`src/native/config_policies.rs`, the retired `command` key);
the plan makes the choice installation wiring and forbids repository JSONC from making it
(`doc/planning/cli-git-rust-implementation.md`, lines 181 to 189);
the cutover installs the linter through mise as `cargo:monochromatic-lint`
(`doc/planning/unified-linter.md`, "Distribution and consumers"),
and mise exposes installed tools on `PATH`,
through `mise activate` or its shims directory
(<https://mise.jdx.dev/dev-tools/shims.html>, read on 2026-10-06);
the scanner is linked in and needs no lookup,
and real Git is found on `PATH` the same way (`src/native/real_git.rs`).
"Beside the wrapper's own executable" was rejected:
mise installs each `cargo:` tool in its own directory,
so the linter is not beside the `git` launcher.

### Running the linter

`src/native/markdown_linter.rs` starts the linter from the repository's top level as
`monochromatic-lint --config=<file> --stdin --stdin-filename=<path> --fix`,
with the invocation's whole environment.
Each option is one `--name=value` argument,
so a pathname that starts with `-` stays a value;
the pathname is passed as Git's exact bytes.
The configuration (`src/native/markdown_linter_config.rs`) is one block that selects `**/*.md` and `**/*.mdx`
and the policy's rules at severity `warn`,
with the policy's `exclude` patterns as the rule's own `exclude` option,
as the installed wrapper passed them (`--lfs-image-exclude`).
It is built as a JSONC value and printed by `monochromatic-jsonc-edit`'s emitter,
and written once per invocation to an owner-only file made by `tempfile`'s exclusive temporary-file call
in the system's temporary directory;
the file is removed when the invocation's checks are dropped,
which is before a forwarded command replaces the process.

`src/native/bounded_child.rs` runs the child with its standard input read from an anonymous temporary file
and both outputs written to anonymous temporary files,
and polls it with `try_wait`,
pausing 1 to 16 milliseconds between looks.
A child still running at its bound is killed and collected,
and one whose output file grows past its cap is killed at the next look.
Files need no reader or writer threads,
a child that never reads its input cannot block,
and a grandchild holding an output open cannot delay the wrapper.
The executable's bounds (`LINTER_LIMITS`) are 60 seconds,
64 MiB of standard output (the fixed source)
and 4 MiB of standard error (the findings).
Measured on this host,
one linter run took 11 milliseconds warm and 227 milliseconds cold.

### Reading the linter's output

`src/native/markdown_linter_output.rs` reads one run against the linter's contract
(`package/linter/monochromatic-lint/README.md`; `src/run_stdin.rs`, `src/run_output.rs`, `src/run_failure.rs`).
With every rule at `warn`,
a finding the fixes leave is a record on standard error and exit status 0,
so every other status means the run cannot be trusted:

- Exit status 0 is usable only with a UTF-8 fixed source,
  which is not empty for a non-empty input
  (the linter refuses to empty a file, so an empty output means it never read the input),
  and a standard error of LF-terminated records,
  each a `warn` of a selected rule with a whole line and column in its first label.
  A repeated field,
  an unselected code,
  a lone line feed
  and a missing final line feed are each refused with the line they are on.
- Exit status 2 quotes the linter's explanation:
  its `monochromatic-lint:` lines and the messages of its records,
  at most 400 characters,
  never the records' file names.
- Any other status,
  a signal,
  a timeout,
  an output past its cap,
  a program that cannot start
  and a configuration that cannot be written each have their own sentence.

All of these end the policy as `policy-incomplete`.
A candidate whose bytes cannot be read stays `content-unavailable`,
and so does a location answer without a top level.
Probes of the linter built from this worktree,
run exactly as the policy runs it,
confirmed the contract:
a rewrite and an exact URL exit 0,
a stale object URL exits 0 with one `warn` record,
an excluded path is unchanged,
`README.MD` and a non-UTF-8 input exit 2,
an empty input exits 0 with empty output,
and a Latin-1 name and a name starting with `-` are rewritten.

### pathBytes

`src/native/event_path.rs` gives every event path two forms:
`path`, text with each invalid sequence replaced by U+FFFD as before,
and,
only for a name that is not UTF-8,
`pathBytes` with the exact bytes as standard padded base64 (RFC 4648 section 4).
It applies to `finding` and `engine-failure` events.
A `fix-summary` whose changed paths include such a name adds `changedPathBytes`,
the base64 of every changed path,
entry for entry in the order of `changedPaths`;
it is absent while every name is UTF-8,
and no entry is ever JSON `null`.
`SPEC.md` documents the three fields under schema version 1,
which allows new optional fields.
The forbidden-strings finding keeps the scanner's masked display path and never gets `pathBytes`:
the exact bytes would undo the masking of a matched segment.

The encoding is base64,
not the one-code-point-per-byte form of the transaction journals (`SPEC.md`, `captured.json`),
for these reasons:
the adopted option is Option C of the decision brief,
whose text says "base64 as in ripgrep's format" and shows `Y2Fm6S50eHQ=` for `caf` 0xE9 `.txt`
(`doc/planning/cli-git-rust-open-decisions.md`, "Non-UTF-8 paths in events and journals");
ripgrep's JSON output carries a path that is not UTF-8 as base64 bytes (cited by the brief),
and Go's JSON encoding carries byte slices the same way
("[]byte encodes as a base64-encoded string", <https://pkg.go.dev/encoding/json>, read on 2026-10-06);
every consumer of these events reads JSON and can decode base64
(`Buffer.from(value, 'base64')` in the TypeScript end-to-end fixture's runtime, `base64.b64decode` in Python);
and a base64 value cannot be mistaken for a name,
while a Latin-1 string such as `café.txt` reads as the different UTF-8 name `café.txt`.
The journal form stays right for the journals,
which only the wrapper itself reads back.

`PolicyFinding.path`,
`FindingEvent.path`,
`PolicyEvent::EngineFailure.path`
and `PolicyEvent::FixSummary.changed_paths` now hold `EventPath`,
and `PolicyOutcome::Failed` gained `path`;
a branch that builds these types must adopt them when it merges.

### Refusal frontier

Before,
on `main` at `6ed268522`,
`git add`,
`git cli-git check` and `git cli-git fix` refused with exit status 2
whenever `markdown/autofix` was on and the lifecycle had candidates:
"cli-git: the native Markdown linter, which the policy markdown/autofix needs,
is not implemented in this native development executable, so git add was not run."

After,
`markdown/autofix` refuses nothing;
`MARKDOWN_AUTOFIX_NEEDS` is gone.
`mono/dependent-version-bump` still refuses the same way.
Controls:
`binary_frontier_tests.rs`, `add_runs_ported_content_policies_and_refuses_unported_ones`
(the dependent-version policy refuses,
`markdown/autofix` at error lets an add with no Markdown file through,
and the escaped refusal is the positive control);
`binary_markdown_tests.rs` for the policy itself.
The tests that used `markdown/autofix` as the example of an unported policy now use the dependent-version policy.

### Gate image

`bin/native-linter-snapshot.mjs` copies the linter crate and `monochromatic-jsonc-edit`,
vendors the linter's own lockfile from the host's Cargo cache
(`vendorLockedDependencies` in `bin/native-scanner-snapshot.mjs` now takes a manifest),
and adds a first image stage that builds the linter
with the development profile and no debug information,
on the same base as the test image.
The test image copies only the executable to `/opt/monochromatic-lint/monochromatic-lint`
and names it in `GIT_POLICY_NATIVE_TEST_LINTER`,
which tests read and the wrapper never does.
The image stays mount-free and network-free.
A host binary is not copied in,
because the host's C library is newer than the image's.
On a fresh host the linter's locked crates must be in the Cargo cache first
(`mise run //package/linter/monochromatic-lint:dependencies:fetch`).

Tests that need the real linter link that exact file into a directory on the `PATH` they build,
and fail when the variable is unset,
so an installed linter can never stand in for it.

### Manifests and lockfiles

- `package/git-policy/cli/Cargo.toml` gained `ignore = "0.4"` and `tempfile = "3"`,
  both already locked through the scanner (`ignore` 0.4.25, `tempfile` 3.27.0 through `gix-tempfile`).
- `package/git-policy/cli/Cargo.lock` gained exactly those two dependency lines.
  `native:lock` (`cargo generate-lockfile --offline`) would have rewritten 608 lines to newer versions;
  the new task `native:lock:members` (`cargo update --offline --workspace`) records a changed dependency list
  and moves no other locked version.
- `package/git-policy/cli.fuzz/Cargo.toml` gained three `[[bin]]` entries;
  its lockfile followed through `lock:subject` with the same two lines.
- `mise run sync:files` left both manifests unchanged.
  It rewrote the root `mise.toml` and `package/config/pnpr/config.yaml` from what is on disk here,
  as it did for the content-policies branch;
  neither is this branch's change,
  so both were restored and nothing from that run was committed.

## What to inspect

- `src/native/policy_markdown.rs` and its tests,
  for which candidates reach the linter and what is reported.
- `src/native/markdown_linter_output.rs`,
  for what output is trusted.
- `src/native/event_path.rs`, `src/native/policy_events.rs` and the three `SPEC.md` paragraphs,
  for the event fields.
- The choices in the Choices open to veto section.

## Differential results

### Harness

The driver is `${HOME}/temp/agent/markdown-autofix-20261006/tools/differential.ts`;
its last full record is `differential-final.txt` beside it.
The incumbent is `package/git-policy/cli/dist/final/node/index.mjs`
(build output of the TypeScript wrapper, gone since 2026-10-13)
of the main checkout,
run as `node <file>` with `PATH=/usr/bin:/bin`,
its configuration trusted with `git cli-git trust --yes`,
and its Markdown command set to the main checkout's `node package/cli/markdown-lint/src/cli.ts` by absolute path;
run by hand first,
that command rewrote the fixture's link and printed `[]`.
The native executable is the debug build of this branch,
run by path through a `git` link outside `PATH`,
with a `PATH` holding a link to the linter built from this worktree,
none,
or a stand-in that exits 3.
Both sides use `rules: ["lfs-image-url"]` and `exclude: ["package/ssg/"]` at `warn`
unless a case says otherwise,
and each command runs on its own pair of fresh repositories
with `.lfsconfig`, `.gitattributes` and one tracked image,
committed before the case's setup.
Compared per command:
exit status,
standard output,
standard error,
`git ls-files --stage` of the index,
a digest and mode of every worktree file,
and whether the real index bytes changed.
Only the two copies' directory names are normalized.

39 commands over 13 cases ran;
25 matched in every field.

Rerun on `db2857189`, with the host debug builds of this commit's native executable and linter
(`${HOME}/temp/agent/native-gate-20261010/differential-all.txt`, driver `differential.ts` beside it):
39 commands, 25 matched in every field, 30 differing fields,
and the same differing commands and reasons as the record above;
the only text that differs from `differential-final.txt` is the root directory's name.
The run's differing fields are the intentional differences listed under "Differences",
so the run shows the comparison can report a difference.

### Cases that matched

- An LFS image link that must be rewritten:
  add warns with `markdown-autofix` and stages the bytes unchanged,
  check reports it,
  `fix -- <path>` and `fix --all` rewrite the worktree file and print one `fix-summary`.
- A link already rewritten,
  a path under `package/ssg/`,
  and a non-Markdown file with an image link:
  nothing is reported.
- Staged with a link to rewrite and a clean worktree copy:
  the add stages the clean bytes silently,
  check and fix report nothing.
  The reverse:
  add and check report the worktree bytes,
  fix rewrites them.
- A stale object URL:
  `markdown-violation` with the same words on both sides.
- Severity `error`:
  add exits 1 and stages nothing,
  check exits 1.
- A missing linter with an add of a non-Markdown file:
  both stage it.

### Differences

All are intentional;
none was fixed.

- Missing linter and a linter that exits 3,
  add, check and fix (6 commands):
  both exit 2 and change nothing.
  The incumbent reports `plugin-threw`
  ("markdown-lint could not be started." and "markdown-lint exited with infrastructure status 3: ...");
  the native wrapper reports `policy-incomplete`,
  a message with the cause and both remedies,
  and a `path` naming the candidate.
  Failure codes follow their cause (`doc/handover/cli-git-rust-implementation.md`, "User decisions 2026-10-05");
  `plugin-threw` named plugins, which the native wrapper does not have.
- A file name that is not UTF-8,
  `pkg/caf` 0xE9 `.md`
  (3 commands):
  the incumbent fails with "The encoded data was not valid for encoding utf-8";
  the native wrapper reports the rewrite with `path` `pkg/caf�.md` and `pathBytes` `cGtnL2NhZukubWQ=`,
  stages the file on add,
  and rewrites it on fix with `changedPathBytes`.
  The plan forbids requiring UTF-8,
  and the field is the adopted decision.
- `rules: []`
  (3 commands):
  the incumbent passes no `--rule`,
  so its linter runs every rule it has
  and rewrites the heading punctuation and the bare URL as well
  ("monochromatic-lint --fix () rewrites ...");
  the native configuration refuses the empty list with `config-invalid`
  ("must name at least one rule; set the policy's severity to "off" to disable it.",
  `src/native/config_policies.rs`),
  a refusal that predates this branch.
  The incumbent's empty list widening to every rule is an incumbent defect:
  its own option comment says prose rules never rewrite a commit unasked.
- `pkg/UPPER.MD`
  (2 commands):
  the incumbent selects the name (`isMarkdownPath` lowercases, `rewrite-candidates.ts`, lines 82 to 88)
  and its linter then refuses it
  ("Standard input path must end in .md or .mdx"),
  so every add and check of such a file exits 2;
  the native wrapper uses the linter's own extension rule and leaves the file alone.
  The linter built from this worktree refuses `README.MD` the same way (exit 2, probe above).

## Gate

Run on merge commit `db2857189`, the merge of main's tip `248a33eeb` into the branch.
The source snapshot was the worktree's committed tree; this evidence file is not part of it.

Command: `mise run //package/git-policy/cli:native:test:container`,
in the bounded container of `bin/test-native-container.mjs` (2 GiB, 2 CPUs, no network, 128 processes).

- Base image: `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a` (audited Git 2.56.0).
- Gate image: `localhost/git-policy-native-test:development`,
  ID `81a68a436612e0255669f2c1babfa140c29a7317b926b594c552cb91828b1d60`.
- Evidence: `package/git-policy/cli/target/verification/native-TzvKOX` (`passed.json`: tests true, Clippy true).
- Unit tests (`git_policy_cli` library): 633 passed, 0 failed.
- Binary tests (`native_binary`): 43 passed, 0 failed.
- Candidate consumer test (`native_candidates`): 1 passed, 0 failed.
- The executable's own unit target: 0 tests.
- Clippy: `cargo clippy --locked --offline --all-targets -- -D warnings` exits 0 in the same image.
- Log: `${HOME}/temp/agent/native-gate-20261010/gate.log`, exit 0.

The linter built for the gate is the one in the image's first stage;
the host's linter was built separately for the differential.

Host builds, used by the differential only:
`mise run //package/linter/monochromatic-lint:build:debug` and `mise run //package/git-policy/cli:native:build`,
both exit 0 at `db2857189`.

## Mutation testing

Run on `db2857189` through the scoped task,
`mise run //package/git-policy/cli:native:mutation:scoped -- --file <glob>`,
one lane per group of changed modules, the four lanes in parallel,
each in its own container (2 GiB, 2 CPUs, 128 processes, no network, `--timeout 90`, `--build-timeout 300`).
The exclusions are the two stalling replacement kinds of `doc/troubleshooting/cargo-mutants-timeout-exit-status.md`
(`replace += with *=`, `replace -= with /=`), which every runner passes.
The lanes ran on the gate image; the driver is `${HOME}/temp/agent/native-gate-20261010/lanes.ts`.

Scopes (all non-test modules changed since the branch point `6ed268522`, except `lib.rs`, which only declares modules):

- Lane a, image `localhost/git-policy-native-mutation:autofix-20261010-a`
  (`ID 55b809a039e17a13e430707d8dc73ede251c59cee2c7f98d39e7ddfc421a46f6`):
  `bounded_child`, `markdown_linter`, `markdown_linter_config`, `markdown_linter_lookup`,
  `markdown_linter_output`, `markdown_exclude`, `policy_markdown`.
- Lane b, image `localhost/git-policy-native-mutation:autofix-20261010-b`
  (`ID 1c8c7d9bd0a6996e7bd77129272f234bfcb51e82b303811fd4368e5371991638`):
  `event_path`, `policy_events`, `policy_engine`, `direct_fix`, `policy_root_context`.
- Lane c, image `localhost/git-policy-native-mutation:autofix-20261010-c`
  (`ID e4e0315cf21c21ee29792965cfd760b98d0fe4d9e23cb4f8c8635117877e13b1`):
  `policy_final_newline`, `policy_content`, `policy_forbidden_strings`, `policy_checks`.
- Lane d, image `localhost/git-policy-native-mutation:autofix-20261010-d`
  (`ID 00739bb7c904c12b667db16b8588de32ca4bd727b679fc3ccf93bdd5d3814c22`):
  `real_git_candidate`, `entry`, `management`, `wrapped_command`.

First pass (the campaigns as run, before any rerun):

- Lane a: 105 mutants; 74 caught, 29 unviable, 2 timed out, 0 missed.
- Lane b: 95 mutants; 80 caught, 11 unviable, 2 timed out, 2 missed.
- Lane c: 96 mutants; 51 caught, 42 unviable, 3 timed out, 0 missed.
- Lane d: 98 mutants; 79 caught, 16 unviable, 3 timed out, 0 missed.
- Total 394 mutants: 284 caught, 98 unviable, 10 timed out, 2 missed.
  Every lane exited nonzero (cargo-mutants exit 3 on timeouts, or the task's failure on missed mutants).

Rerun of the ten timed-out mutants:
A "rerun-once" rule is not written in `doc/troubleshooting/cargo-mutants-timeout-exit-status.md`,
which covers only the exit status and the exclusions.
The rerun was applied here as a diagnostic, not as a pass:
each timed-out mutant was rerun once, alone, in its lane's image,
scoped to its file with `--examine-re` on its file and position
(`${HOME}/temp/agent/native-gate-20261010/rerun.ts`, logs under `${HOME}/temp/agent/native-gate-20261010/rerun/`).
Nine of the ten were caught on the rerun, each within the 90-second bound.
The lanes' parallel load plausibly caused the first timeouts; this was not measured.

- `markdown_linter_output.rs:86:30` (`==` to `!=` in `member`): caught.
- `policy_events.rs:408:62` (`+` to `*` in `render_policy_events`): caught.
- `event_path.rs:128:9` (`EventPath::bytes` to `Vec::leak(Vec::new())`): caught.
- `policy_final_newline.rs:82:40`, `:102:18` and `:152:67`: caught.
- `management.rs:207:24` (`delete !` in `selected_policies`): caught.
- `real_git_candidate.rs:32:51` and `:117:21`: caught
  (the file's rerun tested 5 mutants, including siblings at those positions).
- `markdown_exclude.rs:95:39` (`+` to `*` in `ExcludeMatcher::excludes`): missed on the rerun.

One rerun attempt of `markdown_exclude.rs` failed its unmutated baseline:
`forwarding::version_is_identical_to_native_git` hit its 5-second bound (`binary_support.rs:144`).
The log is kept as `rerun/a-markdown_exclude-attempt1-baseline-flake.log`; the second attempt ran clean.

Totals after the rerun, for all 394 mutants:
293 caught, 98 unviable, 3 missed, 0 timed out.

Surviving mutants:

- `event_path.rs:173:33` and `event_path.rs:173:42`, `|` replaced by `^` in `base64_standard`:
  equivalent mutants.
  The three shifted operands (`<< 16`, `<< 8`, and the unshifted byte) occupy disjoint bit ranges,
  so `^` and `|` give the same result for every input.
  No test can kill them; they are recorded, not accepted as a gap.
- `markdown_exclude.rs:95:39`, `index + 1` replaced by `index * 1` in `ExcludeMatcher::excludes`:
  a real survivor.
  The mutant makes `is_file` always false, so the last path segment is matched as a directory,
  which changes only the result for a directory-only pattern (one ending in `/`) that names a file.
  It survived both passes, so no test checked that such a pattern does not exclude a file of the same name.
  Killed by the test `a_directory_only_pattern_never_excludes_a_file_of_that_name` in
  `src/native/markdown_exclude_tests.rs`, committed as `2e10ab6fb2`.
  Shown by hand: with the mutant applied the test fails (exit 101);
  restored, the file is identical to the commit.

Evidence:

- Lane logs: `${HOME}/temp/agent/native-gate-20261010/mutation-{a,b,c,d}.log`, driver output `lanes.out`.
- Campaign evidence directories: `package/git-policy/cli/target/verification/native-mutation-42VFBA` (a),
  `native-mutation-aLKSBy` (b), `native-mutation-39v6uq` (c), `native-mutation-G2TQli` (d).
- Rerun logs and output: `${HOME}/temp/agent/native-gate-20261010/rerun/` and `rerun.out`, `rerun-exclude.out`.

## Fuzzing

Command: `mise run //package/git-policy/cli.fuzz:smoke`, exit 0 at `db2857189`.
It runs the generator controls, Clippy, then each registered target for 30 seconds in a mount-free bounded container
(image `localhost/git-policy-cli-fuzz-run:development`,
ID `ca537f8362885a8205fa3d36024640ba6fd08445eacb52ea8d899537defde224`).
Log: `${HOME}/temp/agent/native-gate-20261010/fuzz-smoke.log`.

- Generator controls: 26 passed, 0 failed.
- Targets, all exit 0 (executions in 30 seconds each):
  `global_arguments` 1,073,716; `config_loading` 442,734; `config_schema` 57,815;
  `wrapper_controls` 67,028; `batch_reply` 124,249; `stage_listing` 140,393;
  `rules_file` 5,931,102; `final_newline` 11,706,373;
  `linter_output` 5,716; `event_path` 38,547; `linter_config` 60,396.

The three targets new on this branch are `linter_output`, `event_path` and `linter_config`.
`linter_output` ran the fewest executions, so its coverage is the thinnest;
no coverage figure was measured.
No crash artifact was written: the run reported no failure.

Not run: `test:planted`, the planted-defect controls for the fuzz generators,
is not part of the smoke task and was not run here.

## Final gated commit

Gated commit: `2e10ab6fb2`, the test commit on top of the merge `2e3fe1e91a`
(main's tip `8e0fc1252a`, the linter cutover).
The gate, the mutation campaign and the fuzz smoke run all read the clean working tree at `2e10ab6fb2`.

### Merge

- Nine native source files conflicted, because main restyled rustdoc comments (`928c633874`).
  Each conflict hunk was resolved to the branch side;
  then `mise run format:monochromatic-lint -- package/git-policy/cli/src/native` (the fixer main's commit used)
  rewrapped the branch's doc comments.
  `git diff HEAD` against the branch's previous commit shows only comment lines changed.
- The installed `git` wrapper refused `git add` with `config-changed`
  ("Tracked TypeScript source bytes changed") until `git cli-git trust --yes` ran in this worktree
  (`doc/troubleshooting/cli-git-trusted-snapshot-stale-policy-code.md`).
  The trust output said its authority covers "current and future descendant repositories beneath this exact
  canonical root", and that new descendant configs "auto-enroll exact bundles without another prompt and run
  with full account permissions".
  This is a broader grant than this worktree; the owner should review it.

### Gate

- `mise run //package/git-policy/cli:native:test:container` exits 0.
- Gate image `localhost/git-policy-native-test:development`,
  ID `1a0dcbafb56d3edb7225d2cd8f7f0964525b73285c24278df7193149e2374888`.
- Evidence: `package/git-policy/cli/target/verification/native-FQBB9J` (`passed.json`: tests true, Clippy true).
- Unit tests: 634 passed, 0 failed (633 before the new test).
- Binary tests (`native_binary`): 43 passed, 0 failed.
- Candidate consumer test (`native_candidates`): 1 passed, 0 failed.
- Clippy `-D warnings` in the same image: exit 0.
- Log: `${HOME}/temp/agent/native-gate-20261010/gate-2.log`.

### Mutation

Scoped campaign over the twenty changed non-test modules, in four lanes
(`lanes2.ts` in `${HOME}/temp/agent/native-gate-20261010`); the modules are those of the earlier lanes.

First pass, per lane:

- Lane a (7 modules): 105 mutants; 70 caught, 29 unviable, 6 timed out.
- Lane b (5 modules): the four-lane run failed its unmutated baseline
  (`forwarding::version_is_identical_to_native_git` hit its 5-second bound, the flake recorded above).
  Rerun alone, the lane tested 95 mutants: 81 caught, 11 unviable, 1 timed out, 2 missed.
- Lane c (4 modules): 96 mutants; 48 caught, 42 unviable, 6 timed out.
- Lane d (4 modules): 98 mutants; 74 caught, 16 unviable, 8 timed out.

Total first pass: 394 mutants; 273 caught, 98 unviable, 21 timed out, 2 missed.
The 21 timeouts are 11 more than the earlier pass's 10, under the same bounds;
the parallel load plausibly caused them, which was not measured.

Rerun of each timed-out file once, alone, scoped to the timed-out positions (`rerun3.ts`, `rerun4.ts`):
all 21 timed-out mutants were caught on the rerun.
The reruns are in `${HOME}/temp/agent/native-gate-20261010/rerun3/` and `rerun4/`.

Final: 394 mutants; 294 caught, 98 unviable, 2 missed, 0 timed out.

The two missed mutants are equivalent.
They are now at `event_path.rs:237`, earlier at line 173, because the merge's rustdoc rewrap moved the lines:

- `event_path.rs:237:33` (`|` to `^` in `base64_standard`):
  the OR joins operands in disjoint bit ranges, so XOR gives the same value.
- `event_path.rs:237:42` (the other `|` to `^` in the same expression): the same disjoint bit ranges make XOR equal OR.

The earlier real survivor, `markdown_exclude.rs:95:39`, is no longer missed: it is among the 70 caught in lane a,
and it is killed by the new test, as recorded under "Mutation testing".

Images, merge result:

- `localhost/git-policy-native-mutation:merge-2e10ab-a`,
  ID `279de82a8b91a9dbd7684cdeda179d5e4873de66bba3c52535a36a13ad10a195`.
- `localhost/git-policy-native-mutation:merge-2e10ab-b`,
  ID `659bf965d08643e98bc373edb14aaa3b5f9ca85af230587acf5bfa15abe55b18`.
- `localhost/git-policy-native-mutation:merge-2e10ab-c`,
  ID `5c3af7c561bcfb037267db2123bfd5e43d74b0b6af8fc15e1af1b73419d33249`.
- `localhost/git-policy-native-mutation:merge-2e10ab-d`,
  ID `8cc1cb87661107ea05fc22a17ce96eb0ac824a10c9a960c43fb60385d950adad`.

### Fuzz

- `mise run //package/git-policy/cli.fuzz:smoke` exits 0 on `2e10ab6fb2`.
- Image `localhost/git-policy-cli-fuzz-run:development`,
  ID `f201194efa0fafa2f092068ae57d1ef8bbf02ed639ded8fb0fe1653bed9d05a0`.
- Generator controls: 26 passed, 0 failed.
- Targets, all exit 0, 0 artifacts written:
  `global_arguments` 865,699; `config_loading` 338,746; `config_schema` 41,823;
  `wrapper_controls` 43,324; `batch_reply` 191,286; `stage_listing` 122,450;
  `rules_file` 1,036,677; `final_newline` 7,971,020; `linter_output` 27,024;
  `event_path` 36,222; `linter_config` 187,080 executions.
- Log: `${HOME}/temp/agent/native-gate-20261010/fuzz-smoke-2.log`.

### Not rerun on the merge result

- The differential was not rerun. Main's cutover changed the incumbent's configuration and wrapper,
  so the harness needs a check against the new incumbent before its counts mean anything;
  its last record is the pre-merge one in the Differential results section.
- The `forwarding::version_is_identical_to_native_git` 5-second bound failed its unmutated baseline twice:
  in lane b of the four-lane run on the merge result,
  and in an isolated rerun of `markdown_exclude.rs` before the merge.
  Each passed on the next isolated run. It is a timing test under a bound and is not fixed here.

## Choices open to veto

Each is this branch's choice,
with no decision behind it unless one is named.

- The linter is found on `PATH` only:
  there is no override variable,
  and relative or empty entries are skipped,
  where a shell would search them.
- Every rule runs at `warn` in the one-rule configuration,
  so remaining findings arrive with exit status 0
  and any non-zero status means `policy-incomplete`;
  this reads the delegation's "a non-zero status ... as `policy-incomplete`" together with the incumbent,
  which reported remaining findings as `markdown-violation`.
- The bounds are 60 seconds,
  64 MiB of standard output and 4 MiB of standard error per candidate.
- The child's streams are anonymous temporary files instead of pipes.
- Only `.md` and `.mdx` in exact case are Markdown,
  by the platform's extension rule:
  `README.MD` and a file named `.md` are not checked.
- Exclusion runs in the wrapper and again in the rule's options,
  as in the installed wrapper.
  The wrapper's patterns are relative to the top level;
  the linter's are relative to the nearest `.lfsconfig`.
- The linter gets the invocation's whole environment.
- An engine failure about one candidate names it in `path`, and `pathBytes` when needed.
- The autofix message reads "monochromatic-lint --fix (markdown/<rule>, ...) rewrites <path>.",
  the words of the incumbent's source (`src/optional/markdown-lint/rewrite-candidates.ts`);
  the integration branch changed it from the installed wrapper's older "markdown-lint --fix (<rule>)" after the
  repository configuration differential showed the difference.
- `changedPathBytes` lists every changed path once any of them is not UTF-8.
- `pathBytes` is base64,
  for the reasons under "pathBytes".
- `native:lock:members` is a new task for a lockfile change that must not move other versions.

## What remains

- `mono/dependent-version-bump` still refuses;
  its planner is on another branch.
- Commit transactions,
  post-commit and manual-push lifecycles are not ported,
  so the Markdown policy runs only for `git add` and the direct commands here.
- This repository's translated `cli-git.config.jsonc` must list `markdown/autofix`
  with `exclude: ["package/ssg/"]` at cutover.

## Not verified

- Nothing ran on macOS or Windows;
  `native:clippy:windows` only type-checks and lints the library.
  A Windows mise shim named `monochromatic-lint.cmd` would not be found:
  the lookup tries `monochromatic-lint.exe` only.
- A repository whose `.lfsconfig` is below the top level,
  where the wrapper's and the linter's exclusion bases differ,
  was not run.
- Five operating-system failure branches of `src/native/bounded_child.rs`
  (writing or rewinding the input file, duplicating a file handle, `try_wait` failing, reading an output back)
  and a metadata failure while watching an output have no control;
  nor do the two arms that only a defect in cli-git can reach
  (`ProcessLinter::configuration_path` and `MarkdownState::excludes` before preparation),
  or `ExcludeMatcher`'s whole-set build failure after every line compiled.
- The gate builds the linter with the development profile;
  the release artifact the cutover installs was not run.
- The differential ran on the host's Git 2.55.0 behind both wrappers;
  the gate image's Git 2.56.0 ran only the Rust controls.
- The differential record above was first made before `main`'s `monochromatic-jsonc-edit` 0.1.1 was merged;
  its rerun on `db2857189` uses binaries built after that merge and gives the same result.
