# Audit of /tmp uses (issue 1810)

Issue: [#1810](https://github.com/Aquaticats/Monochromatic/issues/1810).
Rule: `XTD` in `AGENTS.md` (never use `/tmp`; the device behind it is unknowable).
The user chose one pass (not one layer per run) on 2026-10-29.

## Decisions from the user (2026-10-29)

- Mise: the repository is migrating away from it altogether.
  Running tasks through `mise run` stays fine, and new mise tasks are fine too.
  A `TMPDIR` entry in the root `mise.toml` `[env]` is allowed but only redirects runs started through mise, so code
  fixes come first.
- Session identification is the agent's job: the git wrapper session was found from the session registry
  (`~/.claude/sessions/*.json` names, `~/.claude/projects/-var-home-user-Monochromatic/<sessionId>.jsonl`), it is
  `monochromatic-ff`.
- Tests that get slower when their fixtures leave tmpfs stay failing; the fix is to make the code faster.
  Filed as [#1812](https://github.com/Aquaticats/Monochromatic/issues/1812):
  `file-enforcer/core/src/staleness-concurrency-regression.unit.test.ts` passes in 330 ms on tmpfs and fails at the
  10-second harness limit on the user cache disk (one `fsync` stalled for 4.7 s at load average 46; median 1.2 ms).
- Production tools that make working directories keep them under `node_modules/.monochromatic/<tool>` (each tool picks
  its own name), not under a shared helper and not in `$XDG_RUNTIME_DIR`.
- `pi-plugin/morph-compact` moves to `package-paused` (done, `64046360d2`).

## Ownership

- Session `monochromatic-ff` owns every Rust use in `package/git-policy/cli`: the production trio
  (`markdown_linter.rs`, `transaction_git.rs`, `manual_push_probe.rs`) and the 12 test files that mention
  `temp_dir()` or `/tmp`.
  Its work is finished on branch `worktree-agent-ab12997169a0f5b11` (pushed to origin, not on main: native Git refuses
  the merge while other sessions hold staged changes in the shared checkout).
  On that branch `mise run //package/git-policy/cli:native:clippy` is clean with and without the ban.
  It messages `monochromatic-80` when the branch reaches main.
- The root `clippy.toml` ban is uncommitted until that message arrives, because every crate sets
  `disallowed_methods = "deny"` and the ban would fail that crate's clippy on main.
  Content: `std::env::temp_dir`, `tempfile::tempdir`, `tempfile::tempfile`, `TempDir::new`, `NamedTempFile::new`.
- The TypeScript container gate runners in `package/git-policy/cli/bin/*.mjs` (they copy the source snapshot into
  `/tmp/cli-git-native-test-*`) belong to the non-Rust part of this issue, which is this session's.
- `package/file-enforcer/core/src/toolchain/*` and `.projenrc.ts` hold another session's uncommitted work: never stage or
  rewrite them. The codemod skips every file with uncommitted changes.
- The git wrapper refuses `git checkout HEAD -- <path>` in the main worktree (policy `linked-worktree-only`); undo a
  codemod edit with a plain edit instead of the bypass flag.

## Done and committed

Rust (unit-test scratch root in the cargo build directory, `CARGO_TARGET_TMPDIR` for integration tests, a guard test
that the root is outside the system temporary directory, clippy clean under the planned ban unless noted):

- `search-shadow/cli` `eacb935ed4`; `forbidden-strings/cli` `65f8d0dfe2` (243 of 244 pass; the failing
  `path_scan::prefix_tests::navigation_markers_after_prefix_are_not_names` is in untouched files, no baseline run);
  `file-manager/app`, `file-manager/variant-gtk-sticky`, `music-player/truepeak-core`, `wg/cli-quicker-exempt`,
  `nested-wayland-session/cli` `45cec40b27`; `music-player/desktop-app` `438bc434e4` (1593 of 1594 pass; the failing
  `ui_lists_render_tests` test is in an untouched file, no baseline run); `ide/app` `5e32ff2d84` (822 pass through the
  `test` task; its `lint:clippy` stops at `implicit_return` errors in `build.rs` line 307, so clippy could not confirm the
  ban there); `linter/monochromatic-lint` and `monochromatic-lint-closure` `da7dfae188`.
- Production Rust: `nested-wayland-session/cli` `PrivateBus` uses `$XDG_RUNTIME_DIR` and fails with a message when it is
  unset. Socket fixtures there and in `music-player/desktop-app` use it too (a Unix socket path holds at most 107 bytes).

TypeScript:

- `test-harness/core` `scratchRoot()` (`cb30dfba68`, `a3caf474d4`, canonical-path fix in `e655dcad15`): node-only,
  imported as `@monochromatic-dev/test-harness-core/ts/scratch-root.ts`, root `monochromatic-dev/test-scratch` under
  `XDG_CACHE_HOME` or `~/.cache` (outside every checkout), `MONOCHROMATIC_TEST_SCRATCH` overrides it.
- 158 test files in 45 packages `e655dcad15`; 21 fixture and verification helpers `8ca06f2b17`;
  3 paused `webapp-forge/server` test files (unverified, outside the workspace).
  Failures after the change were compared with `HEAD`: 8 test files fail identically at `HEAD` (translation-repair,
  wg/cli-quicker, env-local-json), 3 translation-repair packages and `pi-plugin/openai-fast` fail `lint:types` on errors in
  files this change does not touch, and 12 more failures were environment effects of a systemd user unit (no `mise`
  activation) or the `/home` to `/var/home` symlink, fixed in `scratchRoot`.
  The one real regression is `staleness-concurrency-regression` (#1812, left failing on purpose).
- Production tools under `node_modules/.monochromatic/<tool>`: `git-clone-size/cli` and `deps-update/cli` `af0ab873d9`
  (new `temp-dir.unit.test.ts` passes); `mvm/cli`, `config/pnpr`, `open-code-review-issue/cli`,
  `agent-harness-shared/search-fetch`, `pi-plugin/ask-user-question`, `mutation-test/cli` (each with
  `src/scratch-parent.ts`; `lint:types` clean, tests only where a test imports the changed file); `pi-plugin/bash-poke`
  spool and `pi-plugin/morph-compact` (file tier; its one-shot socket uses `$XDG_RUNTIME_DIR`).
- `morph-compact` move: `64046360d2` (36 renames, lockfile importer removed with `pnpm install --lockfile-only`, root
  `mise.toml` `PATH` line removed by hand because `sync:files` runs file-enforcer core with another session's
  uncommitted changes; the generator globs `package/*/*/node_modules/.bin`, so it produces the same result).

## Not done

- The `scratchRoot` codemod must skip files that compare against `tmpdir()` on purpose
  (`git-clone-size/cli/src/temp-dir.unit.test.ts`, `test-harness/core/src/scratch-root.unit.test.ts`,
  `test_scratch.rs` guard tests); the first one was rewritten by mistake and restored by hand.
- TypeScript uses still on `tmpdir()`: `ide/app/task.ts` and `ide/app/bin/*.ts` (inspection scripts that make
  disposable projects, plus a nested-compositor socket path in `inspect-bundle.ts`),
  `ide/app/bin/runtime-publish.test.ts` (package lacks the harness dependency), `toml-edit/core.fuzz/task.ts`,
  `monorepo-task/cli/src/run.ts`, `wg/cli-quicker/src/tempdir.ts` (holds a private key, runs as root; needs a
  directory root can write), `test-fixture/file-enforcer-perf/src/{run-e2e,perf.config,bench-in-container}.ts`,
  `translation-repair/slice-reading/src/image-ocr.ts`, `test-harness/core/src/rejection-fixture-process.ts`,
  `test-harness/core-diagnostic/.../assertion-source.unit.test.ts`,
  `translation-repair/source-scan/.../temp-dirs-scratch.unit.test.ts`,
  `translation-repair/adjudication/.../corpus-source.unit.test.ts` and `corpus-commit-probe.unit.test.ts`,
  `logger/core.fuzz/**`, `package-paused/webapp-forge/stress/.../force-push.ts`,
  `package-paused/desktop-daemon/editord/.../token-file.ts`.
- `.mjs` scripts: `music-player/design/*-test.mjs`, `forbidden-strings/cli/bin/*.mjs`,
  `git-policy/cli/bin/*.mjs` (mine), `git-policy/cli.fuzz/bin/*.mjs`, `linter/monochromatic-lint*/bin/*.mjs`.
- Other shapes: `/tmp` string literals, `TMPDIR` reads, `os.tmpdir()` as a member call.
- Config files with `/tmp`: `mise.toml` of `music-player/desktop-app`, `linter/monochromatic-lint`,
  `test-fixture/file-enforcer-perf`, `translation-repair/coverage-census`, `upstream-fork/conf`, and
  `upstream-fork/deepmerge-ts.fuzz/container/stryker.config.json` (check whether the path is inside a container);
  `doc/audit/*.evidence.json` and `doc/planning/depot-*.json` are historical records: keep their text.
- TypeScript guard: an oxlint restriction on `tmpdir` from `node:os` (`oxlint.config.ts` is the config), with the three
  intentional comparison tests as documented exceptions. Show it failing on a real violation before fixing (rule `GFP`).
- Rust guard: land the `clippy.toml` ban after `monochromatic-ff` reports its branch on main.
- Tests that import none of the changed production files (`pnpr`, `open-code-review-issue`, `ask-user-question`,
  `mutation-test`) have only the type check as evidence.

## Stays, with the reason

- `nested-wayland-session/cli` protocol tests and doc comments: `"/tmp/a b.png"`, `"/tmp/hello.txt"`, `"/tmp/frames"` are
  string data passed to parsers; nothing touches the filesystem.
- `wg/cli-quicker-exempt/src/application_targets_tests.rs`: `/tmp/.mount_*` strings are AppImage mount-point samples.
- `ide/app/src/language/confine/project.rs` (`REPLACED_LOCATIONS`) and `recipe.rs` line 217: `/tmp` is the mount point
  inside the language-server sandbox (bubblewrap), not host scratch; the matching test strings stay.
- `search-shadow/cli/mise.toml` container commands.

## Findings

- Cargo's current layout puts test executables under `<cargo-build-dir>/<profile>/build/<package>/<hash>/out/`, so three
  levels above the executable is the package build directory, not the target directory.
- Clippy stops at the first failing target kind, so a first sweep's violation count is a lower bound. A sweep script that
  assumes a `lint:clippy` task misreports crates without one as clean.
- A systemd user unit lacks the shell's `mise` activation and `PATH`: bus and `oxlint` tests failed there and passed from a
  shell. Run verification from a shell.
- Moving fixtures from tmpfs to disk makes `fsync`-heavy tests slow under load; see #1812.
- The git wrapper's push lock was held by another session for over ten minutes, so several commits are saved locally but
  not pushed yet; the wrapper reports "The commit is already saved".
- Scratch files from this work are in `~/temp/agent` with the `i1810-` prefix (scripts: `i1810-codemod-ts.ts`,
  `i1810-verify-ts.ts`, `i1810-scratch-parent.ts`, `i1810-verify-prod.ts`).

## Next

1. Finish the remaining TypeScript files above, the `.mjs` scripts and `git-policy/cli/bin`, verifying each package.
2. Add the oxlint restriction, then land the `clippy.toml` ban once `monochromatic-ff` reports.
3. Handle the config files above.
4. Commit with `Refs #1810`, and `Closes #1810` on the last commit.
