# cli-git native content policies

## Purpose and how to respond

This branch makes three content policies work in the native wrapper
for `git add` and for the direct `git cli-git check` and `git cli-git fix` commands:
the built-in `final-newline`,
`security/forbidden-strings`
and `mono/forbidden-root-context`.
`markdown/autofix` stays unported
and keeps refusing with exit 2 whenever it is enabled and the lifecycle has candidates.
`mono/dependent-version-bump` no longer refuses: on `feat/cli-git-native-dependent-version` it is wired
(see `doc/handover/cli-git-native-dependent-version.md`, section "Wiring into the native executable").

Paths starting with `src/` or `bin/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Work is recorded slice by slice as it lands.
Respond by merging the branch into `main` and by vetoing any item under "Choices open to veto".

## Status

Complete on this branch, pending review and merge.
Done:
the two candidate sources,
the coded failure outcome,
`final-newline` and `mono/forbidden-root-context` for `git add` and `git cli-git check`,
`security/forbidden-strings` with the `rulesFile` option,
`git cli-git fix` with final-newline corrections,
three fuzz targets,
and the differential runs.
The final gate,
the fuzz smoke campaign and the mutation campaigns are recorded in their own sections, Final gate and Mutation testing.

## Differential harness

### How the incumbent runs with its optional plugins

The incumbent is the built TypeScript wrapper of the main checkout,
`package/git-policy/cli/dist/final/node/index.mjs` (build output of the TypeScript wrapper, gone since 2026-10-13)
(built 2026-09-26 15:43, after the last source commit under `package/git-policy/cli/src`,
`cf2bf70d2` at 15:39 the same day).
It is run as `node <that file> <arguments>` with `PATH=/usr/bin:/bin`,
so its real Git is `/usr/bin/git`.

Each disposable repository lives under a `mktemp`-style directory below `${HOME}/temp/agent/`.
The directory above the repository holds a `node_modules/@monochromatic-dev/git-policy-cli` link
to the main checkout's package,
so the incumbent's `cli-git.config.ts` can import `@monochromatic-dev/git-policy-cli/ts`
when `git cli-git trust --yes` bundles it.
Every run gets a disposable `HOME`,
`GIT_CONFIG_NOSYSTEM=1`,
`GIT_CONFIG_GLOBAL=/dev/null`,
fixed author and committer identities,
and a disposable `FORBIDDEN_STRINGS_CACHE_DIR`;
`FORBIDDEN_STRINGS_RULES` is unset unless a case sets it.

### Probe result

A first probe on 2026-10-06 confirmed the harness before any Rust was written:
`git cli-git trust --yes` trusted the configuration,
`git cli-git check -- a.txt b.txt` reported one `final-newline` warning and one forbidden-strings error and exited 1,
`git add -- b.txt` staged with a `final-newline` warning,
and `git add -- a.txt` exited 1 with a redacted forbidden-strings finding and left `a.txt` unstaged.

## Slices

### Slice 1, candidate sources

Commit `3741ea3e8`.
Two sources the candidate layer did not provide,
both staged on a private copy of the real index:

- What `git add` would stage
  (`src/native/candidate_prediction.rs`, `CandidateRequest::Add`).
  The real index path comes from `git rev-parse --path-format=absolute --git-path index`,
  so `GIT_INDEX_FILE`,
  a linked worktree's own index and `--git-dir` are honored.
  The index is copied with its access and modification times
  (`src/native/candidate_private_index.rs`),
  as `index-file-timestamps.ts` does for issue #544,
  into a fresh `0700` directory named `cli-git-add-policy-<pid>-<nanoseconds>-<count>` beside it,
  which is removed when the prepared candidates are dropped.
  `git ls-files --stage -z --full-name -- :/` lists the copy before and after
  `git <prefix> add <region>` runs on it,
  and the candidates are the paths whose complete record lists differ
  (`src/native/candidate_stage.rs`, `staged_delta`),
  in the incumbent's order:
  paths the index held first,
  in index order,
  then new paths.
  `git diff-index --cached` against `HEAD`,
  limited to the changed paths with `:(top,literal)` pathspecs 2,048 at a time,
  decides additions and deletions;
  a removed path `HEAD` lacks is dropped,
  as `add-staged-delta.ts` drops it.
- The worktree files a direct command selects (`CandidateRequest::Direct`).
  `git add --all -- <pathspecs>` runs on the copy (`--all` is `:/`),
  then every stage-0 entry `ls-files --stage` lists for the same pathspecs is a candidate,
  as `direct-check-facts.ts` and `listPrivateIndexPaths` define it.

Both read bytes through the existing store and its one object reader.

The probe that chose the delta shape:
during a merge conflict `git write-tree` exits 128,
so a design that writes the index before the replay as a tree would refuse every `git add`
while any conflict remains;
the stage-record comparison does not
(`a_conflict_elsewhere_does_not_stop_the_prediction`).

The add listings put `--no-literal-pathspecs` after the caller's prefix.
Measured on the host:
with `GIT_LITERAL_PATHSPECS=1`,
`git ls-files --stage -- :/` lists nothing and exits 0,
which would have made every add look unchanged;
with `--no-literal-pathspecs` it lists the index.
The replay itself keeps the caller's mode
(`literal_pathspec_mode_cannot_empty_the_prediction`).

A new cause,
`CandidateFailure::PrivateIndexUnavailable`,
reports a copy that could not be made;
`candidate_failure_code` maps it to `content-unavailable`.

Process count,
by the counting stand-in the candidate layer's controls use:
6 Git processes for 1,
20 and 200 new files predicted and read twice,
5 for the same direct scopes,
and one more `diff-index` at 2,049 changed paths;
the positive control,
one `git cat-file blob` per candidate through the same stand-in,
counted 1,
20 and 200
(`candidate_prediction_count_tests.rs`).

Gate on that tree,
`GIT_POLICY_NATIVE_IMAGE_TAG=content-policies`:
544 unit tests,
37 binary-level tests,
1 public-interface consumer test,
Clippy passed,
exit status 0;
test image `72ada9f0761ed4a786466f3311af30fdc993ac326af249b9646dba0b89e1e315`,
base `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`,
evidence `package/git-policy/cli/target/verification/native-plPDWH`.
The snapshot was copied before the next edit started.

### Slice 2a, the failure outcome carries its code

Commit `34b533adb`.
`PolicyOutcome::Failed` is now `Failed { code, message }`,
and `run_policy_stage` puts that code into the `engine-failure` event.
The three repository facts the command policies read keep `content-unavailable`.
`failed_check_ends_the_pass_with_an_engine_failure` runs both codes through the stage.

### Slice 2b, lifecycle content and the first two policies

Commit `30ce6cab5`.

- `policy_checks::CandidateSource` is renamed by role to `policy_content::LifecycleContent`
  (`None` or `Requested(CandidateRequest)`);
  `candidate_version::CandidateSource` keeps its name.
- `RepositoryFacts::candidates(request)` prepares candidates,
  so the facts seam stays the only way a check asks Git.
  `ContentState` prepares them when the first content policy reads,
  shares one version with every later content policy,
  and remembers a failure so it is reported once and not retried.
- `wrapped_command::prepare_guarded_command` requests the add's candidates for `git add` in a worktree;
  `management::plan_management` requests the scope's.
  A direct command prepares before any policy runs,
  as the incumbent does,
  and reports a scope Git refuses as one `transaction-failed` event with its trigger,
  the incumbent's code for the same failure.
- `final-newline` (`src/native/policy_final_newline.rs`) and `mono/forbidden-root-context`
  (`src/native/policy_root_context.rs`) check the candidates with the incumbent's codes and messages.
- Superseded on `feat/cli-git-native-dependent-version`: `mono/dependent-version-bump` is wired
  and no longer answers `Unavailable`.
- At this commit, `markdown/autofix` and `mono/dependent-version-bump` answered `Unavailable`
  wherever the lifecycle has candidates,
  naming "the native Markdown linter" and "planning dependent version bumps".
  `security/forbidden-strings` and a `direct-fix` correction did the same at this commit,
  until their own slices.

Gate on that tree,
`GIT_POLICY_NATIVE_IMAGE_TAG=content-policies`:
553 unit tests,
37 binary-level tests,
1 public-interface consumer test,
Clippy passed,
exit status 0;
test image `7bbeab59bab540a512ceb7b631901de62cfcbe9183f511001bdfb6848141846c`,
same base,
evidence `package/git-policy/cli/target/verification/native-BGz5rN`.
The unit count equals the host count of that commit (543 passed and 10 that need Git 2.56),
so the snapshot held the committed tree.

### Slice 3, forbidden-strings

Commit `b7bed7111`.

- `security/forbidden-strings` (`src/native/policy_forbidden_strings.rs`) scans the lifecycle's candidates
  through the linked scanner (`scanner_run::scan_version`).
  Rules load once per invocation,
  and only when at least one candidate is scannable,
  so an add that only deletes never reads a rules file.
  A load failure is remembered and reported to every later scan of the invocation.
- Matches keep the incumbent's words
  (`scanner-output.ts`:
  "Forbidden string matched at line N (rule R)."
  and "... in pathname segment N (rule R).")
  and the scanner's masked display path.
  Every result is checked for a failed matcher or a pathname with a line break before any match is reported;
  either ends the policy as `policy-incomplete`.
- `rulesFile` (`src/native/config_rules_file.rs`) is a new option of the policy.
  A value is refused when it is empty,
  starts with `/`,
  starts with a drive letter and a colon,
  holds a backslash or NUL,
  or has an empty,
  `.` or `..` component;
  the configuration error names the key and the reason.
  A value that names a missing file is not a configuration error:
  loading reports it as `policy-incomplete` when the policy runs.
- `scanner_selection::rules_source(rules_file, variable, root)` is the one place that picks the file:
  the configured name,
  else `FORBIDDEN_STRINGS_RULES` from the invocation's environment,
  else `forbidden-strings.local.txt` in the top level.
  Only the default file may be missing beside the built-in rules.
- `src/native/panic_notice.rs` is the executable's panic hook,
  installed first thing in `main.rs`.
  It prints the source location and never the message,
  which can hold scanned bytes.
  `a_caught_panic_shows_only_the_notice_in_its_own_process` runs a caught panic in a child process with the hook
  and,
  as its positive control,
  with the default hook,
  which does print the payload.

Gate on that tree:
564 unit tests,
38 binary-level tests,
1 public-interface consumer test,
Clippy passed,
exit status 0;
test image `3635275302f050430f5c0ebde7fff86162aa87b3abb22b19db145a19a9717793`,
evidence `package/git-policy/cli/target/verification/native-akNbC8`.

### Slice 4, direct fix

Commit `ed3f10514`.

- A finding with `fix_available` ends its stage as `StageEnd::Proposed`
  (exit status 1),
  whatever its severity and even under keep-going,
  as `patchProposed` ends the incumbent's stage (`policy-stage.ts`).
- `final-newline` proposes a full-content correction on `direct-fix` only,
  where the incumbent's `canApplyPatches` is true;
  its finding then says `"fix":"available"`.
- `ContentState` keeps the proposals of a pass,
  applies them after the pass over the prepared objects,
  and every later read,
  the scan included,
  sees the corrected bytes
  (`scanner_run::CandidateBytes` is the reader the scan pass now takes).
  A file whose correction restores its original bytes stops counting as corrected,
  so two states are equal exactly when every file holds the same bytes.
- `src/native/direct_fix.rs` gives `policy_convergence::converge` its passes,
  reports only the last pass,
  and appends one `fix-summary`
  (trigger,
  changed passes,
  corrected paths in byte order)
  only when files changed and the last pass exits 0.
  A cycle or the pass limit reports one engine failure and nothing else,
  as `fixCycleFailure` and `fixPassLimitFailure` do;
  the cycle message is the incumbent's direct-fix wording.
- `src/native/direct_fix_install.rs` follows `direct-fix-install.ts`:
  every corrected file must still hold the bytes the fix read,
  each gets a backup copy and a new file beside it named `.cli-git-direct-fix-<pid>-<nanoseconds>-<count>`,
  the new files are renamed over the originals with mode `0644` or `0755`,
  and the real index must be byte-identical afterwards.
  Any failure restores every replaced file from its backup
  and reports one `transaction-failed` event.
  A backup that cannot be restored is left in place and the message says so.
- Only `final-newline`,
  `markdown/autofix` and `mono/dependent-version-bump` run on `direct-fix`
  (`policy_trigger.rs`, as in the incumbent),
  so a fix never runs forbidden-strings or root-context.

Gate on that tree:
578 unit tests,
39 binary-level tests,
1 public-interface consumer test,
Clippy passed,
exit status 0;
test image `355d6966e8532a7b3cf762a3309b06d5f9df70334fc317b10fb0d270ae858d7a`,
evidence `package/git-policy/cli/target/verification/native-d97U92`.

Commit `e4725f72c` then made the private index directory builder compile without an unused `mut` on Windows
(`native:clippy:windows` denied it)
and added `native:build`,
which builds the executable on the host for the differential runs.

### Fuzz targets

Commit `e37dad30a`,
in `package/git-policy/cli.fuzz`:

- `stage_listing`:
  `parse_stage_records` over raw bytes,
  which must be refused as malformed or as an unsupported mode,
  or accepted only when the bytes are exactly Git's rendering of the returned records;
  and `staged_delta` over two generated listings in index order,
  compared with a delta recomputed from sorted maps.
- `rules_file`:
  `check_rules_file` over text inputs and over values built from path words;
  the answer must be the refusal restated from the value's words,
  and an accepted value joined to a root must stay below it with only ordinary components.
- `final_newline`:
  `normalized_final_newline` over raw bytes and generated text;
  left alone only when empty,
  holding NUL,
  not UTF-8 or already canonical,
  and a replacement keeps the content,
  ends with one LF and is itself left alone.

The `wrapper_controls` target's fixed facts now refuse to prepare candidates,
so its invariant that no unchecked `git add` is forwarded still holds and now goes through the content policy.

The repository's installed wrapper corrected the final newline of the new seed files when they were committed
(its `final-newline` exclusions do not cover this package's seeds),
so every new seed ends with one LF.
The names `seed/final_newline/missing` and `seed/final_newline/extra` were kept,
although those two files are now canonical;
the targets still reach both cases through their generated view.

Generator controls:
21 passed (`cargo test --lib` through `mise run //package/git-policy/cli.fuzz:test`).
Planted defects:
14 planted,
14 noticed,
including the three new ones
(a changed path left out of the staged delta,
a `rulesFile` value with `..` accepted,
extra final line feeds kept);
evidence `package/git-policy/cli.fuzz/target/verification/planted-2Ml8kS`.

Smoke campaign,
`GIT_POLICY_NATIVE_IMAGE_TAG=content-policies mise run //package/git-policy/cli.fuzz:smoke`
on `e37dad30a`'s fuzz package
and `ed3f10514`'s subject
(the later `e4725f72c` changes only the private index directory builder, which no target calls):
the controls and Clippy passed in the container,
then each of the eight targets ran 30 seconds under AddressSanitizer and exited 0
with no artifact.
Executions:
`stage_listing` 227,141,
`rules_file` 1,991,661,
`final_newline` 2,653,607;
the five existing targets between 13,916 (`config_schema`) and 290,050 (`global_arguments`).
Compiler `rustc 1.100.0-nightly (1303417c4 2026-09-21)`,
evidence `package/git-policy/cli.fuzz/target/verification/campaign-id61LK`.
A first attempt ended with exit status 1 right after copying the sources and printed no error;
the second attempt is the one recorded.

## Final gate

On `e4725f72c`,
the tree every later section measures,
`GIT_POLICY_NATIVE_IMAGE_TAG=content-policies mise run //package/git-policy/cli:native:test:container`:
578 unit tests,
39 binary-level tests,
1 public-interface consumer test,
Clippy with warnings denied,
exit status 0.
Test image `30a8e6112b2a3723be78c4b379dfd130b44baa960df6525eb8b7c2d9b05589f7`,
base `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`,
real Git 2.56.0 in the image,
evidence `package/git-policy/cli/target/verification/native-qNO9Nv`
(`passed.json` has tests and Clippy both true).
`main` changed nothing under `package/git-policy/` since the branch point `8ee6cbb39`,
so no merge was needed.
`native:clippy:windows` also passed on that tree.

On the host,
whose Git is 2.55.0,
the same suites leave 10 unit and 4 binary-level tests failing.
Each compares with Git 2.56.0's option tables or version string,
and every one lives in a file this branch did not change
(`git diff main` over those test files and their subjects is empty);
they were not run on `main` here.

## Mutation testing

### Campaigns

All campaigns ran `mise run //package/git-policy/cli:native:mutation:scoped` against the final gate image
`30a8e6112b2a3723be78c4b379dfd130b44baa960df6525eb8b7c2d9b05589f7`
(each `manifest.json` records it as `baseImage`),
with cargo-mutants 27.1.0,
its 300-second build bound and 90-second test bound,
and only the two exclusions the runner already has
(`+=` to `*=` and `-=` to `/=`).
Each noticed all five planted guard removals before mutating.
The tags `content-policies-b`,
`-c` and `-d` are further names of the same gate image made with `podman tag`,
because the runner names its campaign image by the tag;
the candidates branch did the same.
No source or test changed between the final gate and the campaigns.

Every changed source file was mutated
except `src/native/lib.rs`,
for which cargo-mutants lists no mutant,
and `src/native/policy_test_support.rs`,
which is compiled for tests only.

### Results

Across the four scopes cargo-mutants generated 411 mutants:
275 caught on the first run,
13 timed out and were caught on the rerun,
123 did not compile,
and none was missed.

- Candidate sources
  (`candidate_stage.rs`,
  `candidate_prediction.rs`,
  `candidate_private_index.rs`,
  `candidate_store.rs`,
  `candidate_error.rs`),
  evidence `package/git-policy/cli/target/verification/native-mutation-vipZW3`:
  107 mutants,
  64 caught,
  30 did not compile,
  13 timed out,
  none missed;
  cargo-mutants exited 3 for the timeouts.
  The unmutated baseline took 79 seconds to test against the 90-second bound;
  the timeouts took 90 to 101 seconds,
  while the host's load average was between 66 and 96 from other sessions' containers.
  An earlier attempt,
  `native-mutation-UbDlh4`,
  stopped when the unmutated baseline itself reached 90 seconds.
- Timeout rerun,
  once,
  of the three files that had them
  (`candidate_error.rs`,
  `candidate_store.rs`,
  `candidate_stage.rs`),
  evidence `native-mutation-XBlW6j`:
  81 mutants,
  61 caught,
  20 did not compile,
  no timeout,
  none missed;
  unmutated baseline 10 seconds.
  All 13 earlier timeouts are among the caught,
  matched line by line against the first run's `timeout.txt`.
  Before the rerun,
  one of them,
  `run_listing` replaced with `Ok(vec![])`,
  was planted by hand in a throwaway worktree:
  pinned to two host CPUs its unit tests failed 58 tests in 7.86 seconds,
  against 10.02 seconds for the unmutated tests on the same CPUs,
  so it was slow under contention,
  not hanging.
- Policies and scanner
  (`policy_final_newline.rs`,
  `policy_content.rs`,
  `policy_root_context.rs`,
  `policy_forbidden_strings.rs`,
  `scanner_failure_code.rs`,
  `scanner_run.rs`,
  `scanner_selection.rs`),
  evidence `native-mutation-kymPWy`:
  110 mutants,
  70 caught,
  40 did not compile.
  Its first attempt,
  `native-mutation-X1HSTd`,
  stopped on Podman's `database is locked` in the first planted control
  and was retried once after two minutes.
- Engine,
  direct fix and option check
  (`policy_engine.rs`,
  `policy_checks.rs`,
  `direct_fix.rs`,
  `direct_fix_install.rs`,
  `panic_notice.rs`,
  `main.rs`,
  `config_rules_file.rs`),
  evidence `native-mutation-oQyd93`:
  95 mutants,
  74 caught,
  21 did not compile.
- Lifecycles and configuration
  (`config_policies.rs`,
  `config_schema.rs`,
  `entry.rs`,
  `management.rs`,
  `wrapped_command.rs`,
  `repository_facts.rs`),
  evidence `native-mutation-caNqhs`:
  99 mutants,
  67 caught,
  32 did not compile.

The mutants that did not compile replace a function body with a value the return type cannot take:
`Default::default()` or a list or `Ok` of it for types without a default,
and `Rc<[u8]>` built from a leaked `Vec`,
which does not convert;
`unviable.txt` in each evidence directory lists them.

## Refusal frontier

### Before, on `main` at `8ee6cbb39`

- `git add` in a worktree refused with exit status 2 whenever any content policy was on,
  which by default is always,
  because `final-newline` is built in:
  "cli-git: predicting what git add would stage, which the policy final-newline needs,
  is not implemented in this native development executable, so git add was not run.
  Run it with the installed cli-git."
- `git cli-git check` and `git cli-git fix` refused the same way whenever a content policy was selected,
  needing "reading the worktree files that git cli-git check and fix select".

### After, at `e4725f72c`

- `git add`,
  `git cli-git check` and `git cli-git fix` run `final-newline`,
  `mono/forbidden-root-context` and `security/forbidden-strings` and refuse nothing for them.
- Whenever `markdown/autofix` or `mono/dependent-version-bump` was enabled at this commit
  (the dependent-version refusal is superseded on `feat/cli-git-native-dependent-version`),
  and the lifecycle has candidates,
  the command still refuses with exit status 2 and the existing one-line notice,
  for example
  "cli-git: the native Markdown linter, which the policy markdown/autofix needs,
  is not implemented in this native development executable, so git add was not run.
  Run it with the installed cli-git."
  and "planning dependent version bumps, which the policy mono/dependent-version-bump needs, ...".
- Controls:
  `binary_frontier_tests.rs`,
  `add_runs_ported_content_policies_and_refuses_unported_ones`
  (each unported policy refuses under its own name and nothing is staged;
  the positive control escapes `mono/dependent-version-bump` and the add goes through);
  `binary_policy_tests.rs`,
  `direct_fix_corrects_only_worktree_files`
  (a listed `markdown/autofix` refuses the fix and no file changes).
- Commit transactions,
  manual-push listings and post-commit stay outside this branch and keep their own refusals.

## Differential results

The driver is `${HOME}/temp/agent/content-policies-20261006/differential.ts`;
its last full record is `differential-all.txt` beside it,
from the executable built at `e4725f72c`.
Each command runs on its own pair of fresh repositories built the same way:
one with the incumbent's `cli-git.config.ts`
(`mono/forbidden-root-context` at error,
`security/forbidden-strings` at error with `builtinRules: false`
and the release scanner from this worktree as its executable),
one with the equivalent `cli-git.config.jsonc`,
each committed before the case's own setup.
The native executable runs by path through a `git` link outside `PATH`;
neither wrapper is installed and neither touched this repository.
Compared per command:
exit status,
standard output,
standard error,
`git ls-files --stage` of the index,
a digest and mode of every worktree file,
and whether the real index bytes changed.
Directory names that differ between the two copies are normalized before comparing.

44 commands over 17 cases ran;
32 matched in every field.

### Cases that matched

- Clean file:
  add,
  check and fix all exit 0 with no output.
- Missing final newline:
  add stages with one warning on standard error,
  check reports the warning on standard output,
  `fix -- file.txt` and `fix --all` print one `fix-summary`,
  correct the worktree file and leave the index unchanged.
- Secret staged with a clean worktree copy:
  add exits 0
  (Git would stage the clean bytes),
  check exits 0.
- Clean staged copy with a secret in the worktree:
  add exits 1 with the redacted match and stages nothing,
  check exits 1 with the same match.
- Forbidden root context:
  add and check exit 1 with `root-context-forbidden`.
- Rules from `FORBIDDEN_STRINGS_RULES`,
  from the default `forbidden-strings.local.txt`,
  and from configuration
  (native `rulesFile: "rules/private.txt"`,
  incumbent `FORBIDDEN_STRINGS_RULES` naming the same file):
  identical matches.
- Symbolic link,
  staged deletion and intent-to-add entry:
  add,
  check and fix agree in every field.
- `git add --patch` with closed standard input and `git add --pathspec-from-file=-`:
  both wrappers forward and Git stages nothing.
- From a subdirectory without keep-going:
  check and add stop on `require-root` identically.

### Differences

All are intentional;
none was fixed.

- Missing rules file
  (`FORBIDDEN_STRINGS_RULES` naming a file that does not exist),
  add and check:
  both exit 2,
  the incumbent with `plugin-threw` "Forbidden-strings scanner exited with infrastructure status 2.",
  the native wrapper with `policy-incomplete` "cli-git could not load the forbidden-strings rules,
  so no file was scanned: read rules <path>: No such file or directory (os error 2).".
  Failure codes follow their cause;
  `plugin-threw` named plugins,
  which the native wrapper does not have
  (`doc/planning/cli-git-rust-open-decisions.md`,
  section "Engine failure codes that lose their source",
  and this delegation's failure-code item).
- A pathname that is not UTF-8
  (`caf` 0xE9 `.txt`):
  the incumbent fails,
  printing "The encoded data was not valid for encoding utf-8"
  (exit 1 for add,
  `transaction-failed` with exit 2 for check and fix);
  the native wrapper checks,
  stages and fixes the file and renders its name with a replacement character.
  The plan forbids requiring UTF-8
  (open decisions,
  section "Non-UTF-8 paths in events and journals");
  how an event carries the exact name is still the owner's open question,
  so no `pathBytes` field was added.
  With `--all` the native add-explicit policy rejects the bulk add before any content is read,
  while the incumbent fails on the name first;
  both exit 1.
- From a subdirectory with keep-going,
  check names the files `file.txt` and `secret.txt` in the incumbent
  and `sub/file.txt` and `sub/secret.txt` in the native wrapper.
  `SPEC.md` line 362 makes event paths repository-relative.
- From a subdirectory,
  `fix -- file.txt`:
  the incumbent fails with `transaction-failed` "Deleted candidate lacks HEAD entry: file.txt";
  the native wrapper corrects `sub/file.txt` and reports it.
  The incumbent confuses the two relative forms.
- `git add -- missing.txt`
  (a pathspec that matches nothing):
  the incumbent exits 1 with Git's message as plain text;
  the native wrapper exits 2 with `content-unavailable`
  "cli-git could not predict what git add stages: git add failed:
  fatal: pathspec 'missing.txt' did not match any files".
  A failure before Git runs exits 2 under the spec's exit contract
  (open decisions,
  "A wrapper failure before Git runs").
- `git cli-git check` and `fix` with that pathspec:
  same exit status and code,
  `transaction-failed`;
  the message wording differs
  ("git add --all -- missing.txt failed: ..." against
  "cli-git could not read the selected worktree files: git add failed: ...").
  The native message does not repeat the command's arguments;
  Git's own message still quotes the pathspec.

## Choices open to veto

- Candidates for `git add` are prepared lazily,
  by the first content policy that reads;
  a direct command prepares before any policy runs,
  as the incumbent does.
- A failed `git add` replay is reported with Git's standard error in the message.
- A direct command's scope that Git refuses is `transaction-failed`,
  the incumbent's code,
  not `content-unavailable`.
- Corrections are applied in memory as full contents,
  not staged into the private index as patches;
  a fix pass starts no Git process for them.
- A state equal to the one just before it counts as settled
  (`policy_convergence::converge`, unchanged here);
  the incumbent calls that a cycle.
  No shipped correction can produce it.
- `rulesFile` is checked by its words only;
  a symbolic link inside the repository that points elsewhere is followed.
- Installation failure messages name no file;
  the incumbent's name the path.
- The direct fix does not take the landing lock the incumbent takes around installation.
- `git add --patch`,
  `--interactive`,
  `--edit` and `--pathspec-from-file=-` are replayed with standard input closed,
  as the incumbent replays them,
  so the policies see what such an add stages without input,
  which is nothing.
- No `pathBytes` field:
  a name that is not UTF-8 is rendered with replacement characters,
  as the foundation renders it.

## What remains

- `markdown/autofix` needs the native Markdown linter;
  it keeps refusing.
- `mono/dependent-version-bump` needs native dependent-version planning,
  which another branch ports;
  it kept refusing at this commit (superseded on `feat/cli-git-native-dependent-version`, which wires it),
  and this branch did not touch those modules.
- Manual-push listings,
  commit transactions and post-commit candidates are outside this branch.

## Manifests and lockfiles

- `package/git-policy/cli/Cargo.toml` and both lockfiles are unchanged.
- `package/git-policy/cli.fuzz/Cargo.toml` gained three `[[bin]]` entries for the new targets.
  file-enforcer manages it through the `package/*/*/Cargo.toml` glob;
  `mise run sync:files` in this worktree left it unchanged.
  The same run rewrote the root `mise.toml` (`_.path` entries) and `package/config/pnpr/config.yaml`
  (published package count 128 to 131) from what is on disk here;
  neither is this branch's change,
  so both were restored and nothing from that run was committed.
- `package/git-policy/cli/mise.toml` gained `native:build`;
  `package/git-policy/cli.fuzz/mise.toml` describes `test:planted` without a count.

## Not verified

- Nothing ran on macOS or Windows;
  `native:clippy:windows` only type-checks and lints the library for `x86_64-pc-windows-gnu`.
- The differential runs used the host's Git 2.55.0 behind both wrappers;
  the gate image's Git 2.56.0 ran only the Rust controls.
- `git add --patch` was compared for an untracked file only,
  where Git answers "No changes.";
  a tracked file with changes and closed standard input was not compared.
- A second process changing a file or the index while `git cli-git fix` installs was provoked only inside the
  installer's unit controls,
  by calling its two phases directly;
  no concurrent process raced a real fix.
- The incumbent's landing lock around direct-fix installation has no native counterpart and no control.
- The differential driver and its records live in scratch
  (`${HOME}/temp/agent/content-policies-20261006/`),
  not in the repository.
