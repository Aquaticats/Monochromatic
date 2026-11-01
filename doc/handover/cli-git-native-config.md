# cli-git native repository configuration

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Note,
 2026-10-14:
 the retirement of the TypeScript wrapper deleted the task `native:config:differential`
 and the `bin/differential-*.mjs` files that this record names.
The record is `doc/handover/cli-git-native-retire.md`;
the incumbent that this record compares against is the commit `f4fc1fa213ece65672e4314983a75266837892f9`.

## Purpose and how to respond

This branch expresses the repository's own cli-git configuration in the native wrapper's format
and proves that the native wrapper decides like the incumbent TypeScript wrapper under it.
It answers the work-queue item
"This repository's translated configuration must list all four optional policies and set `rulesFile`"
in `doc/handover/cli-git-rust-implementation.md`.

Branch `feat/cli-git-native-config`,
worktree `.claude/worktrees/cli-git-lane-config`,
created from the transactions head `13f60ef1b1`.
Paths starting with `src/`, `bin/` or `mise.toml` are relative to `package/git-policy/cli/`;
every other path is repository-relative.

Respond by merging the branch into the integration branch.
Three things stay open for the people who own them:
the two policies the native wrapper cannot evaluate yet (see `Blocking rows`),
the per-worktree rules file (see `Rules file in a linked worktree` and `Choices open to veto`),
and the failures the base already has (see `Gate results`).

## Status

The configuration, the differential check, and the control that binds the shipped file are committed.
The last differential run against the committed head showed 36 equal rows, 36 pending rows and 0 different rows.
Commits on this branch after `13f60ef1b1`:

- `fab9c47b79` adds `cli-git.config.jsonc`.
- `108a326616` adds the differential check and its mise task.
- `31bad9874a` adds the configuration control and the gate snapshot entry for the shipped file.

The container gate and Clippy results are in `Gate results`.

## The native format and how the incumbent maps onto it

The native wrapper reads one file, `cli-git.config.jsonc`, at the top level Git reports
(`src/native/config_file.rs:56`).
The incumbent's `cli-git.config.ts` and `cli-git.config.mjs` are never read or run natively;
when one sits beside the JSONC file, only `git cli-git check` reports it (`src/native/config_file.rs:69`).
The top-level keys are `policies`, `hooks`, `indexLock` and `landing` (`src/native/config_parse.rs:52`).

The four optional policies are `markdown/autofix`, `mono/forbidden-root-context`, `mono/dependent-version-bump`
and `security/forbidden-strings`.
Each one is off unless the file lists it (`off_unless_listed` rows in `src/native/policy_registry.rs:184` to `:208`),
and an empty file and no file behave identically.
This is the user decision of 2026-10-05
(`doc/handover/cli-git-rust-implementation.md:1255` to `:1260`).

The mapping from `cli-git.config.ts` to `cli-git.config.jsonc`, setting by setting:

- `markdown/autofix` at `warn` with `exclude: ['package/ssg/']` (`cli-git.config.ts:35`)
  becomes `["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }]`.
  The rule list is the incumbent's default, now stated.
  The native option set is `rules` and `exclude` only (`src/native/config_policies.rs:278`).
- `mono/forbidden-root-context` at `error` (`cli-git.config.ts:41`) becomes `"error"`.
- `mono/dependent-version-bump` is absent from `cli-git.config.ts`
  but ran at its default severity, `error`, through the registered repository plugin.
  It is listed at `"error"` explicitly
  (`doc/handover/cli-git-rust-implementation.md:1255` to `:1260` names this consequence).
- `security/forbidden-strings` at `error` (`cli-git.config.ts:42`) becomes
  `["error", { "builtinRules": true, "rulesFile": ".cache/forbidden-strings.rules.txt" }]`.
  `builtinRules` is the schema default, stated as the incumbent states it.
  `rulesFile` is new in the native format (`src/native/config_policies.rs:153`, `src/native/config_schema.rs:130`):
  the scanner reads the file it names relative to the top level,
  else the `FORBIDDEN_STRINGS_RULES` variable, else `forbidden-strings.local.txt`
  (`src/native/scanner_selection.rs:115`; user decision at `doc/handover/cli-git-rust-implementation.md:1293`).
  The value is the path the root `mise.toml` gives `FORBIDDEN_STRINGS_RULES` (line 1319),
  so both wrappers read one file.
- The incumbent's `executable` option has no counterpart:
  the scanner is linked into the native wrapper and the option set is `builtinRules` and `rulesFile` only
  (`src/native/config_schema.rs:117` to `:130`).
- `plugins` and `trust` (including `children: true`) have no counterpart:
  executable configuration and its consent protocol are retired
  (`doc/planning/cli-git-rust-behavior-ledger.md:878` to `:895`; `src/native/config_parse.rs:54` to `:60`).
- The five built-in policies keep their defaults and need no lines.

## Decisions and their reasons

- The JSONC file is added beside `cli-git.config.ts`, and the TypeScript file stays.
  The incumbent still reads it, and the differential check needs both.
  The native `check` reports the TypeScript file as ignored (see `Facts the integrator must know`).
  Deleting the TypeScript file belongs to the cutover, after the incumbent is retired.
- `rulesFile` names `.cache/forbidden-strings.rules.txt` rather than relying on the variable.
  The user chose the configured option first so that a wrapper started without the variable does not silently
  scan with the built-in rules only.
- The differential check compares byte-for-byte output of direct commands (`cli-git check`, `cli-git fix`) and `add`.
  `git commit` is still refused natively, so a commit cannot be compared.
  `--no-enforce-*` controls are not used anywhere;
  the policy filter `--policy <id>` isolates one policy at a time instead.
- A row where the native wrapper refuses because it does not implement an enabled policy counts as pending,
  never as equal.
  `--require-all-equal` turns pending rows into failures; use it once both policies are ported.
- The control that binds the shipped file is a Rust unit test over the real loader
  (`src/native/config_repository_tests.rs`), not a copy of the file in a test.
  It needs the file in the container snapshot,
  so `bin/test-native-container.mjs` copies `cli-git.config.jsonc` into the image next to `clippy.toml`.
  The inline translation in `src/native/config_parse_acceptance_tests.rs:102` is a separate parser control
  and can drift;
  the new control is the one tied to the file the repository ships.
- Candidate sets build the forbidden-string hit from fragments so no source file here carries a matching literal.
  The fixture rules file is a copy of the committed `forbidden-strings.append.txt`;
  the generated cache also carries embargoed local rules and is never read.

## Evidence

The differential check builds, per candidate set, one disposable repository for each wrapper.
Each holds the real `cli-git.config.ts`, the real `cli-git.config.jsonc`, a baseline commit made by real Git,
the scanner at the path the incumbent configuration names, and the rules file where `rulesFile` names it.
The incumbent runs from source (`src/index.ts`) with `FORBIDDEN_STRINGS_RULES` set;
the native wrapper runs without that variable, so `rulesFile` is what is exercised.
Twelve probes run per set: `check` with every policy, `check --policy <id>` and `fix --policy <id>` for each of the five
content policies, and `add`.

Candidate sets:

- `clean-control`, a file that passes every policy.
- `missing-final-newline`, a file lacking its final newline.
- `forbidden-string-from-rules-file`, a line that only the appendix rule `ncd-reserved-shortcodes` matches.
- `forbidden-root-context`, a new root `CONTEXT.md`.
- `dependent-version-change`, a hand-bumped workspace package with a publishable dependent.
- `markdown-autofix-touch`, a Markdown file with an image link to an LFS-tracked file.

Each probe `check --policy <id>` also asserts that the incumbent reports the named set's policy and no other,
so a set that stops triggering fails the run instead of passing as equal.

Result against the committed head with Git 2.55.0 (the host's, `/usr/bin/git`):
72 rows, 36 equal, 36 pending, 0 different.
Every `check --policy` and `fix --policy` row for `final-newline`, `security/forbidden-strings`
and `mono/forbidden-root-context` is equal in all six sets.
That includes the forbidden-string finding from the `rulesFile` path, the missing-final-newline finding and its fix
(the file bytes, the index and `git status` after the fix match), and the root-context finding.

Positive controls (the check can fail):

- Removing the `mono/dependent-version-bump` line makes the `dependent-version-change` set fail on its
  `check` and `fix` rows (native stays silent where the incumbent reports `dependent-version-stale`).
- Removing the `mono/forbidden-root-context` line makes `forbidden-root-context` fail.
- Removing the `markdown/autofix` line makes `markdown-autofix-touch` fail.
- Removing `rulesFile` makes `forbidden-string-from-rules-file` fail
  (the native scan then misses the appendix-only rule).
- Each removal was made in the working tree, the check run, and the file restored; the tree was clean afterwards.

The Rust control (`src/native/config_repository_tests.rs`, four tests) passes on the shipped file
and fails when any one line is removed:
the `markdown/autofix` line (two tests fail), the `mono/dependent-version-bump` line,
the `mono/forbidden-root-context` line, the `security/forbidden-strings` line (two tests fail),
or only `rulesFile`.
Each failure names the policy that is missing.

## Blocking rows

The 36 pending rows are exactly the rows where the native wrapper refuses:
`ShippedChecks` returns an unavailable outcome for `markdown/autofix` and `mono/dependent-version-bump`
whenever the lifecycle requested candidates (`src/native/policy_checks.rs:378` and `:427` to `:432`).
Because the translated configuration lists both, every native `git add`, `git cli-git check` without a narrowing
`--policy`, and `fix` for those two policies exits 2 with
"the native Markdown linter, which the policy markdown/autofix needs, is not implemented ..."
or "planning dependent version bumps, which the policy mono/dependent-version-bump needs, is not implemented ...",
even for a clean tree.
The incumbent exits 0 for the clean set and for `add` of every set except the forbidden-string and root-context sets,
where it exits 1 with the finding on stderr.
The blocker is therefore the two unported policies (the open items in the main handover),
not the configuration.

What each unported policy must produce, taken from the incumbent in the same fixtures:

- `markdown/autofix` on `doc.md`: one `finding` event, `policyId` `markdown/autofix`, `severity` `warn`,
  code `markdown/autofix/markdown-autofix`,
  message `monochromatic-lint --fix (markdown/lfs-image-url) rewrites doc.md.`,
  exit 0 on `check` and on `add` (the event goes to stderr on `add`).
- `mono/dependent-version-bump` on a hand-bumped `package/module/a/package.json`
  (an illustrative fixture path; no package has existed there):
  exit 1, one `finding` event at `error`,
  code `mono/dependent-version-bump/dependent-version-stale` on `package/module/b/package.json`
  (an illustrative fixture path; no package has existed there),
  `fix` `available`; `fix --policy mono/dependent-version-bump` exits 0, prints a `fix-summary` naming that manifest
  and rewrites its version.
  The incumbent skips this policy for `add` (`src/optional/repository-policy/dependent-version-bump-policy.ts`,
  the pre-forward guard in `findDependentBumps`), while the native stub refuses `add` for it;
  the port must reproduce the skip.

Once both are ported, run the differential with `--require-all-equal`.

## Rules file in a linked worktree

`rulesFile` is relative to the top level of the worktree Git runs in.
The file is generated by `mise run file-enforcer` into `.cache/`, which is ignored (`.gitignore:68`),
so a worktree that nobody generated it in has none.
This worktree has none: Claude Code's worktree isolation created it, and nothing copied `.cache` into it.

What the ignored-state copy does (by the ledger section `Linked-worktree ignored-state copy`,
`doc/planning/cli-git-rust-behavior-ledger.md:3371` onward):
it runs for `git worktree add` and `git worktree move`, and copies every path Git reports as ignored when the source
is a linked worktree or a bare repository.
The incumbent bypasses a main-worktree source;
the native executable (branch `feat/cli-git-native-rules-cache`, `doc/handover/cli-git-native-rules-cache.md`)
copies exactly one path from a main-checkout source, the configured `rulesFile`, and only when Git ignores it and the
checkout has it.
So a worktree created through the native wrapper from the main checkout or from a linked worktree gets
`.cache/forbidden-strings.rules.txt`, and a source without that file copies nothing,
so the missing-file error below stands.
A worktree created by another tool, or with `--no-worktree-copy`, still has no `.cache`.
The copy runs in the executable as measured by the binary tests in `src/native/binary_worktree_copy_tests.rs`.

Measured with both wrappers in a disposable repository that has the configuration and no `.cache`:
the native `git cli-git check --policy security/forbidden-strings --all` exits 2 with an `engine-failure` event,
code `policy-incomplete`,
message "cli-git could not load the forbidden-strings rules, so no file was scanned: read rules
<root>/.cache/forbidden-strings.rules.txt: No such file or directory (os error 2).";
the incumbent exits 2 with `plugin-threw` and "Forbidden-strings scanner exited with infrastructure status 2."
when its variable names a missing file.
Both fail closed.

Where the incumbent's variable pointed was observed in this session's shell, in the incumbent's debug output while
running in this worktree: it resolved to `/var/home/user/Monochromatic/.cache/forbidden-strings.rules.txt`,
the main checkout's file.
That shell was started from the main checkout.
A shell with mise activated inside a worktree resolves `{{config_root}}` to that worktree and would see no file,
as the native wrapper does.
So the incumbent passed in this worktree only because its variable pointed at the main checkout;
under the native wrapper with `rulesFile`,
every forbidden-strings run in a worktree without its own `.cache` fails closed and, through the full configuration,
blocks `git add` and the commit.

## Choices open to veto

- Default taken: keep `rulesFile` at `.cache/forbidden-strings.rules.txt`, relative to the worktree,
  and add no mechanism that creates or finds the file.
  The reason is the user decision of 2026-10-05 that the configured option comes first
  (`doc/handover/cli-git-rust-implementation.md:1293`) and that a missing explicit file is an error,
  so a worktree never scans with fewer rules than the repository names.
  The native error names the missing path.
- Chosen by the user on 2026-10-12: the ignored-state copy brings `.cache/forbidden-strings.rules.txt`
  when the native wrapper creates a worktree, from a main checkout and from a linked worktree.
  It cannot help a worktree created by another tool.
- Option: file-enforcer runs when a worktree is created.
  It would cover every creator, but it makes worktree creation depend on the generator and its scaffolding of the
  embargoed local appendix.
- Option: each worktree runs `mise run file-enforcer` first.
  This needs no code and is what `doc/runbook/forbidden-strings-out-of-band-rules.md` already tells contributors to do.
  The veto, if any, is to make one of the other options mandatory before the cutover.

## Facts the integrator must know

- The incumbent's decisions for the five content policies are unchanged by this branch;
  nothing outside the configuration file, the new `bin/` modules, one `mise.toml` task and one test module changed.
- `git cli-git check` (native) prints a `configuration-warning` event, code `legacy-config-ignored`, as its first event
  while `cli-git.config.ts` sits beside the JSONC file.
  The differential removes that one event, renumbers the events after it, and fails if it is absent or repeated.
  Delete `cli-git.config.ts` at the cutover and this notice disappears.
- Native `check` and `fix` events match the incumbent's field for field, in the same key order, for the three ported
  policies; the differential compares bytes, not parsed values.
- Files outside this lane that changed: `src/native/config_file.rs` gains a test-module registration at its end,
  and `bin/test-native-container.mjs` gains one snapshot entry and one `COPY` line for the shipped file.
  No configuration-loading fix was needed.
- The main handover item "This repository's translated configuration must list all four optional policies and set
  `rulesFile`" can be ticked on merge; this branch does not edit that file.
- The generated root `mise.toml` keeps `FORBIDDEN_STRINGS_RULES`; the standalone scanner and CI still use it.
- The `lint:oxlint:paths` task aborts in Oxlint's default formatter for some files that have warnings;
  `doc/troubleshooting/oxlint-default-formatter-render-panic.md` has the reproduction and the workaround.
  The new `bin/differential-*.mjs` files lint with zero warnings and zero errors.

## Gate results

Container gate at `31bad9874a` (image `6a81db2ba3034977061a1ab45ebc1b9300a085ce8fffee2a988b4def6deb0022`,
Git 2.56.0 base `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`):

- The four tests of `src/native/config_repository_tests.rs` pass in the image,
  so the snapshot entry and the `COPY` line deliver the shipped file.
- The library target ran 1337 passed, 1 failed, 4 ignored.
  The one failure, `commit_transaction::tests::a_selected_change_prepares_a_commit_outside_head`,
  is not part of this branch:
  the transactions doc records no container gate at `2ce0f83d0b`, the commit that added it,
  and in the image Git has no identity and the test's `git` child stops with
  "fatal: unable to auto-detect email address (got 'user@91eb299ae250.(none)')" (exit 128).
  The same test passes on the host through the `native:test:host` task with the test name as its filter,
  where Git has an identity.
  It comes from the slice 9 skeleton commit `2ce0f83d0b`;
  its owner must give the test an identity in the child environment.
- Because `cargo test` stops after the first failing target, the remaining targets were run in the same image with
  `--no-fail-fast`: the binary target passed 50 tests and the consumer target 1.
- `mise run //package/git-policy/cli:native:clippy` fails on the base before this branch's files are linted:
  an unused import `git_text` in `src/native/commit_convergence_tests.rs:14`,
  a collapsible `if` in `src/native/commit_transaction.rs:148`,
  and two missing `return` statements in `src/native/commit_transaction_tests.rs:64` and `:139`.
  None of the four diagnostics is in a file this branch changed.

## Trailing gates

No trailing gate is skipped for lack of time; the following were not applicable or not run, with the reason:

- Mutation testing: not run.
  The only change to a Rust file that has production code is the `#[cfg(test)]` module declaration at the end of
  `src/native/config_file.rs` (lines 367 to 370),
  and the added `src/native/config_repository_tests.rs` is a test module.
  `mise run //package/git-policy/cli:native:mutation:list -- --file src/native/config_file.rs` lists sixteen mutants,
  the last at line 323, all in code this branch did not touch;
  the same listing for `src/native/config_repository_tests.rs` is empty.
  This branch adds no mutant, so a campaign over it would repeat one over unchanged code.
- Git 2.56.0: the differential ran only on the host's Git 2.55.0.
  The 2.56.0 image ran the configuration tests, which do not use Git, and the existing suite.
  A run of the differential inside the interop image (`mise run //package/git-policy/cli:native:interop`)
  would cover 2.56.0 and was not done.
  Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
  The incumbent that such a run needs.
  The results are in `doc/handover/cli-git-native-interop.md`.
- Windows, fuzz targets, planted controls and release-artifact gates: this branch adds no code to them.
  The new Rust test reads a file through `std::fs` and a relative path only.

The differential's pending rows are the open work and are listed under `Blocking rows`.

## Rerun commands

Build the native wrapper once, then run the differential from the repository root:

```sh
# doc/handover/cli-git-native-config.md: build the native wrapper the differential starts by path
mise run //package/git-policy/cli:native:build
mise run //package/git-policy/cli:native:config:differential
mise run //package/git-policy/cli:native:config:differential -- --scenario dependent-version-change
mise run //package/git-policy/cli:native:config:differential -- --require-all-equal
```

The options are `--native <path>` (default `target/debug/cli-git-native`), `--scanner <path>`
(default: this checkout's release scanner, else the main checkout's), `--scenario <name>` and `--require-all-equal`.
The incumbent's Markdown policy needs `monochromatic-lint`, found with `mise which monochromatic-lint`.

Run the control that binds the shipped file:

```sh
# doc/handover/cli-git-native-config.md: the four tests of the shipped configuration
mise run //package/git-policy/cli:native:test:host -- repository_tests
```

Run the container gate and Clippy (heavy; through the slot limiter):

```sh
# doc/handover/cli-git-native-config.md: the gate and the lint, through the host slot limiter
export GIT_POLICY_NATIVE_IMAGE_TAG=config
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:test:container
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:clippy
```
