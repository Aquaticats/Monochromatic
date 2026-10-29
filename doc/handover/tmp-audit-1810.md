# Audit of /tmp uses (issue 1810)

Issue: [#1810](https://github.com/Aquaticats/Monochromatic/issues/1810).
Rule: `XTD` in `AGENTS.md` (never use `/tmp`; the device behind it is unknowable).
The user chose one pass (not one layer per run) on 2026-10-29.

## Constraints from the user (2026-10-29)

- Mise: the repository is migrating away from it altogether.
  First message: "don't rely more on it".
  Corrected twice: running tasks through `mise run` stays fine in this session, and new mise tasks are fine too.
  A `TMPDIR` entry in the root `mise.toml` `[env]` is therefore allowed, but it only redirects runs started through
  mise, so code fixes come first.
- Session identification is the agent's job: the git wrapper session was found from the session registry
  (`~/.claude/sessions/*.json` names, `~/.claude/projects/-var-home-user-Monochromatic/<sessionId>.jsonl`), it is
  `monochromatic-ff`.

## Ownership

- Session `monochromatic-ff` owns every Rust use in `package/git-policy/cli`: the production trio
  (`markdown_linter.rs`, `transaction_git.rs`, `manual_push_probe.rs`) and the 12 test files that mention
  `temp_dir()` or `/tmp`.
  Its work is finished on branch `worktree-agent-ab12997169a0f5b11` (pushed to origin, not on main: native Git refuses
  the merge while other sessions hold staged changes in the shared checkout).
  On that branch `mise run //package/git-policy/cli:native:clippy` is clean with and without the five-method ban.
  It messages `monochromatic-80` when the branch reaches main.
- The root `clippy.toml` ban (below) is uncommitted until that message arrives, because every crate sets
  `disallowed_methods = "deny"` and the ban would fail that crate's clippy on main.
- The TypeScript container gate runner in `package/git-policy/cli/bin/` (`test-native-container.mjs`,
  `native-verification-process.mjs`, and the `*-native-container.mjs` siblings) copies the source snapshot into
  `/tmp/cli-git-native-test-*`.
  It belongs to the non-Rust part of this issue, which is this session's.
- `package/file-enforcer/core/src/toolchain/*` and `.projenrc.ts` hold another session's uncommitted work: never
  stage or rewrite them.
  The TypeScript codemod skips every file with uncommitted changes.

## Done and committed

Rust, each with the unit-test scratch root in the cargo build directory (`test_scratch.rs`, three levels above the test
executable) or `CARGO_TARGET_TMPDIR` for integration tests, a guard test that the root is outside the system temporary
directory, and clippy clean under the planned ban unless noted:

- `search-shadow/cli`: `eacb935ed4`; 49 tests pass.
- `forbidden-strings/cli`: `65f8d0dfe2`; 243 of 244 pass.
  The failing test `path_scan::prefix_tests::navigation_markers_after_prefix_are_not_names` asserts UNC prefix
  redaction, uses no scratch directory, and lives in files this change does not modify; no baseline run was made.
- `file-manager/app`, `file-manager/variant-gtk-sticky`, `music-player/truepeak-core`, `wg/cli-quicker-exempt`,
  `nested-wayland-session/cli`: `45cec40b27`.
  `wg/cli-quicker-exempt` ran clippy only (its lifecycle tests are privileged).
- `music-player/desktop-app`: `438bc434e4`; 1593 of 1594 pass.
  The failing test `ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis` is in an
  unmodified file and uses no scratch directory; no baseline run was made.
- `ide/app`: `5e32ff2d84`; 822 tests pass with 2 skipped through the `test` task.
  The `lint:clippy` task stops at two `implicit_return` errors in `build.rs` line 307 (a file this change does not
  modify), so clippy could not confirm the ban for this crate.
- `linter/monochromatic-lint` and `linter/monochromatic-lint-closure`: `da7dfae188`; 696 and 9 plus 24 tests pass.
  The two integration test roots declare the shared file with `#[path = "test_scratch.rs"]`.

TypeScript:

- `test-harness/core`: `scratchRoot()` in `src/scratch-root.ts` (`cb30dfba68`, location corrected in `a3caf474d4`).
  Import it as `@monochromatic-dev/test-harness-core/ts/scratch-root.ts`; it is node-only and not re-exported from the
  index because the neutral build must load in browsers.
  The root is `monochromatic-dev/test-scratch` under `XDG_CACHE_HOME` or `~/.cache`, outside every checkout so that a
  tool walking upward never finds a repository or project configuration; `MONOCHROMATIC_TEST_SCRATCH` overrides it.
  Four unit tests pass and `lint:types` is clean.

## Decisions taken without asking

- Production `PrivateBus` in `nested-wayland-session/cli/src/private_bus.rs` uses `$XDG_RUNTIME_DIR` and fails with a
  message when it is unset (the CLI already needs it for the Wayland socket).
- Socket fixtures (`nested-wayland-session/cli` control socket tests, `music-player/desktop-app` notification bus)
  use `$XDG_RUNTIME_DIR` too: a Unix socket path holds at most 107 bytes and the build directory path is longer.
  `monochromatic-ff` hit the same limit in issue 1809.
- Systemd user units differ from a shell: 16 private-bus tests timed out inside unit `issue-1810-verify` and passed from
  a shell. Run bus tests from a shell.
- A control run of `dbus-daemon` used a probe directory in `/tmp` (removed straight away); it was the only deliberate
  `/tmp` write of this session.

## Uncommitted: TypeScript codemod (158 files)

`~/temp/agent/i1810-codemod-ts.ts` replaced `tmpdir()` with `scratchRoot()` in 158 definite test files
(`*.test.ts` families) of packages that depend on `@monochromatic-dev/test-harness-core`, fixing the `node:os` import and
adding the `scratch-root.ts` import.
Verification runs as unit `issue-1810-ts-verify` (script `~/temp/agent/i1810-verify-ts.ts`): `lint:types` per package,
then each changed file with `node <file>`.
Result file: `~/temp/agent/i1810-verify-ts-summary.json`.
Nothing of this layer is committed yet.
After the summary shows green (or each failure is explained as pre-existing by running the file at `HEAD`), commit with
explicit pathspecs per package group, `Refs #1810`.
The root `clippy.toml` edit is also uncommitted (see Ownership).

## Not done

- TypeScript non-test files that use `tmpdir()` (about 37 uses), in two groups:
  - Fixture helpers with non-test names (`fixture.ts`, `test-support.ts`, `test-fixtures.ts`, `*-fixture.ts`,
    `mise.verify-*.ts`, `verify-host.ts`, `disposable-host.ts`, `host-fixture-home.ts`,
    `setup-fixture.ts`, `integration-fixture`): same `scratchRoot()` rewrite, each package already depends on the harness.
  - Production code that creates short-lived private working directories on other people's machines:
    `mvm/cli/src/spawn.ts`, `mvm/cli/src/backend/hetzner/ssh.ts`, `deps-update/cli/src/scratch.ts`,
    `git-clone-size/cli/src/temp-dir.ts`, `config/pnpr/src/publish-steps.ts` (two uses, one holds an npm userconfig),
    `open-code-review-issue/cli/src/github-api.ts`, `agent-harness-shared/search-fetch/src/tool-output.ts` and
    `gh-process.ts` (a working directory), `pi-plugin/bash-poke/src/spool.ts` (a `tmp` parameter default),
    `pi-plugin/ask-user-question/src/answer-workspace.ts` and `tool-result.ts`,
    `pi-plugin/morph-compact/src/ipc-socket-unix.ts` (a socket; needs a short path, so `$XDG_RUNTIME_DIR`) and
    `ipc-file.ts`, `wg/cli-quicker/src/tempdir.ts`, `mutation-test/cli/src/...`, `electron-infra/core/src/wayland-constants.ts`.
    These must not import the test harness. Open design: a small runtime helper (candidate home:
    `fs-path/core`, 13 dependents among the affected packages, or per-tool directories under
    `$XDG_CACHE_HOME/monochromatic-dev/`) and the dependency edits plus lockfile regeneration it implies.
- Other TypeScript and JavaScript shapes not covered by `tmpdir()`: `/tmp` string literals, `TMPDIR` reads, `.mjs`
  scripts (`music-player/design/*-test.mjs`, `forbidden-strings/cli/bin/*.mjs`, `git-policy/cli.fuzz/bin/*.mjs`,
  `linter/monochromatic-lint*/bin/*.mjs`, `git-policy/cli/bin/*.mjs`), `package/ide/app/bin/*.ts`,
  `ide/app/bin/runtime-publish.test.ts` (package lacks the harness dependency), and three test files the codemod
  skipped for shape (`test-harness/core-diagnostic/.../assertion-source.unit.test.ts`,
  `translation-repair/source-scan/.../temp-dirs-scratch.unit.test.ts`,
  `translation-repair/adjudication/.../corpus-source.unit.test.ts` and `corpus-commit-probe.unit.test.ts`).
- Config files with `/tmp`: `mise.toml` files of `search-shadow/cli` (container commands, stay: they run inside a
  container image), `music-player/desktop-app`, `linter/monochromatic-lint`,
  `test-fixture/file-enforcer-perf`, `translation-repair/coverage-census`, `upstream-fork/conf`,
  `upstream-fork/deepmerge-ts.fuzz/container/stryker.config.json` (check whether the path is inside a container).
  `doc/audit/*.evidence.json` and `doc/planning/depot-*.json` are historical records: keep their text.
- TypeScript guard: an oxlint restriction on `tmpdir` from `node:os` (`oxlint.config.ts` is the config; check how its
  other restrictions are declared and whether `os.tmpdir()` as a member call can be caught), with the harness's own
  guard test as a documented exception. Show it failing on a real violation before fixing (rule `GFP`).
- Rust guard: land the `clippy.toml` ban after `monochromatic-ff` reports its branch on main, then run each crate's
  clippy once more.

## Stays, with the reason

- `nested-wayland-session/cli` protocol tests and doc comments: `"/tmp/a b.png"`, `"/tmp/hello.txt"`, `"/tmp/frames"`
  are string data passed to parsers; nothing touches the filesystem.
- `wg/cli-quicker-exempt/src/application_targets_tests.rs`: `/tmp/.mount_*` strings are AppImage mount-point samples.
- `ide/app/src/language/confine/project.rs` (`REPLACED_LOCATIONS`) and `recipe.rs` line 217: `/tmp` is the mount point
  inside the language-server sandbox (bubblewrap), not host scratch; the matching test strings stay.
- `search-shadow/cli/mise.toml` container commands.

## Findings

- Cargo's current layout puts test executables under `<cargo-build-dir>/<profile>/build/<package>/<hash>/out/`, so
  three levels above the executable is the package build directory, not the target directory.
  Scratch still lands on the disk-backed cargo build directory (btrfs on this host).
  `monochromatic-ff` uses the same heuristic in `git-policy/cli`.
- Clippy stops at the first failing target kind, so a first sweep's violation count is a lower bound.
  A sweep script that assumes a `lint:clippy` task misreports crates without one as clean:
  `linter/monochromatic-lint`, `terminal/app` and `ide/app` showed zero for that reason.
- The git wrapper's push lock was held by another session for over ten minutes, so several commits are saved locally but
  not pushed yet; the wrapper reports "The commit is already saved".
- Scratch files from this work are in `~/temp/agent` with the `i1810-` prefix. The first sweep script wrote
  `sweep-forbidden-strings-cli.log` and `sweep-summary.json` there, overwriting same-named older scratch files.

## Next

1. Read `~/temp/agent/i1810-verify-ts-summary.json`, resolve failures, commit the 158-file codemod.
2. Rewrite the fixture-named TypeScript helpers with the same codemod (widen its file filter).
3. Decide and build the runtime scratch helper for production TypeScript, then migrate those files.
4. Add the oxlint restriction, then land the `clippy.toml` ban once `monochromatic-ff` reports.
5. Handle `.mjs` scripts, the git-policy `bin/` runners and the config files above.
6. Commit with `Refs #1810`, and `Closes #1810` on the last commit.
