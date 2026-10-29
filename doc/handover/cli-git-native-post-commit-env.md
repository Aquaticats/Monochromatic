# Native post-commit environment fix (evidence)

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Worktree: `.claude/worktrees/cli-git-lane-post-commit-env`.
Branch: `feat/cli-git-native-post-commit-env`, based on `main` at `e0b93ffaca`.
Image tag: `post-commit-env`.

## The defect

With the published `monochromatic-git-policy-cli` 0.1.0, `git commit ... -- doc/x.md` in a repository with the
repository's `cli-git.config.jsonc` landed the commit and then exited 2.
Standard error held an engine-failure event `policy-incomplete` (trigger `post-commit`, policy `markdown/autofix`,
message "could not find monochromatic-lint in any absolute directory on PATH"),
then a `commit-landed` event with outcome `post-commit-blocked`, so auto-push was blocked.
The pre-commit stage found the linter.

## Cause

`run_staged_gate` in `package/git-policy/cli/src/native/post_commit.rs` built its checks with `shipped_checks(...)`
and nothing else.
`shipped_checks` gives the Markdown policy `executable_linter(&[])`, which has no PATH,
so every Markdown candidate stayed unchecked.
`entry.rs` and `management.rs` assign the environment-derived settings after building their checks;
the post-commit gate did not, and it received only the child-environment overlay, never the invocation's environment.

The same omission hid three more settings from the post-commit stage:

- The forbidden-strings options of the configuration (`rulesFile`, `builtinRules`), so a configured private rules file
  was ignored after the landing.
- The `FORBIDDEN_STRINGS_RULES` variable.
- The Markdown options of the configuration (`rules`, `exclude`), so an excluded Markdown file was still linted.

## Change

Commit `b9fab0f843`,
`fix(git-policy-cli): give the post-commit stage the invocation environment and configured options`.

- `PostCommitRequest` has a new field `environment`, the invocation's whole environment.
  `commit_command.rs` passes `command.environment`.
- `run_staged_gate` sets `scanner_settings.options`, `markdown.options`, `scanner_settings.rules_variable` and
  `markdown.linter` on its checks, from the request, as `entry.rs` and `management.rs` do.
- The crate version is 0.1.1 in `package/git-policy/cli/Cargo.toml` and `Cargo.lock`
  (`native:lock:members`).
  `file-enforcer.config.ts` manages only the shared dependency and profile blocks of Cargo manifests,
  not `version`, so the manifest is edited directly.
  The install example in `package/git-policy/cli/README.md` names 0.1.1.
  The `"cargo:monochromatic-git-policy-cli"` pin in the root mise files belongs to the branch
  `feat/cli-git-native-install` and is not touched here.

## Other builders of the checks

Every non-test caller of `shipped_checks(` under `src/native`:

- `entry.rs`: sets the rules variable and the linter; the configuration options are set by `wrapped_command.rs`
  after the configuration loads.
  The manual-push lifecycle in `wrapped_command.rs` runs on these checks.
  Unchanged.
- `management.rs` (direct `check` and `fix`): sets the rules variable and the linter; the options are set from the
  loaded configuration.
  Unchanged.
- `commit_transaction.rs` `private_checks`: moves the scanner settings and the Markdown state of the checks it
  rebuilds from, so the commit's pre-forward and convergence passes keep them.
  Unchanged.
- `post_commit.rs`: fixed here.
- `policy_checks.rs` is the definition and its default has an empty environment by design.

The allowed worktree directories (`Vec::new()` in the post-commit gate) are read only by `linked-worktree-only`,
which declares the `pre-forward` trigger alone, so the post-commit stage does not need them.

## Tests

Unit tests in `src/native/post_commit_tests.rs`, one per setting:

- `the_linter_is_found_on_the_invocations_path`
- `the_markdown_options_reach_the_post_commit_policy` (the unchecked case is also its positive control)
- `the_configured_rules_file_reaches_the_post_commit_scanner`
- `the_rules_variable_reaches_the_post_commit_scanner`

Binary test in `src/native/binary_markdown_tests.rs`:
`the_post_commit_stage_finds_the_linter_on_the_invocations_path` commits a Markdown file through the built
executable, in a disposable repository, with a fake `monochromatic-lint` on PATH.
The fake returns its input and records the commit `main` names on each run.
The test requires exit 0, empty standard error, and a last recorded run equal to the landed commit,
which only the post-commit stage can produce.

Red before the fix: the binary test failed with exit 2 and the exact event sequence of the report
(`policy-incomplete` at `post-commit`, then `commit-landed` with `post-commit-blocked`).
With the four assignments removed from the fixed file, all four unit tests and the binary test fail;
with each planted control applied alone, its named test fails (checked on the host, Git 2.55.0).

Planted controls added to `bin/native-planted-controls.mjs` (additive): one per assignment, named
`post-commit stage ignores ...`, each `from` string occurring once in `post_commit.rs`.
The planted phase of the mutation campaign was not run (see Mutation).

## Gates

Final gated head: `b9fab0f843afc31df5a1f858be61e2d5a813f17b`; later commits only add this document.

- Container gate (`native:test:container`, tag `post-commit-env`), image id
  `b15ea7a8690b6438c88d3533f9a7c405768948b500c0c748c0a44f27937b23fe`.
  Library: 1657 passed, 0 failed, 6 ignored (helper processes and interop probes).
  Binary (`native_binary`): 116 passed, 0 failed, 1 ignored (the landing-crash child).
  Candidate consumer: 1 passed.
  The real-linter Markdown binary test passed in this image.
- `native:clippy` on the host: finished with `-D warnings`, no diagnostics.

## Mutation

Command, through the slot limiter, against the image above:

```text
# package/git-policy/cli
NATIVE_MUTATION_SKIP_PLANTED=1 GIT_POLICY_NATIVE_IMAGE_TAG=post-commit-env \
  node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:mutation:scoped -- \
  --file src/native/post_commit.rs --file src/native/commit_command.rs --timeout 900
```

Result: 20 mutants, 12 caught, 8 unviable, 0 missed, 0 timeouts.
`NATIVE_MUTATION_SKIP_PLANTED=1` skipped the planted-control phase, which takes hours under the CPU hold;
the four new controls were instead applied one at a time on the host, and each failed its named test.

## Rerun commands

```text
# package/git-policy/cli
mise run //package/git-policy/cli:native:test:binary:host the_post_commit_stage_finds
mise run //package/git-policy/cli:native:test:host post_commit::
GIT_POLICY_NATIVE_IMAGE_TAG=post-commit-env mise run //package/git-policy/cli:native:test:container
mise run //package/git-policy/cli:native:clippy
```

## Facts for the integrator

- Publishing 0.1.1 and moving the mise pin from 0.1.0 are outside this branch.
- `cargo fmt` (`native:format`) rewrites most of `src/native`, so the crate is not rustfmt-clean at `main`;
  this branch does not run it.
