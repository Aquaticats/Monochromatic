# Handover: generic package category elimination

Updated 2026-10-29 by session `monochromatic-db`.
Decisions and mappings live in
[the plan](../planning/generic-category-elimination.md);
this file records the outcome and what is still open.

## Goal

Eliminate the `package/` categories `cli`,
 `module`,
 `shim`,
 `stub`,
 `build-tool`,
 `dev-script`,
 `desktop-app`,
 `desktop-daemon`,
 `rust-module`,
 `webapp-productivity`,
 and `git`,
moving every member under a product category,
 with npm names following paths (`AGENTS.md` rule SGD).

## Outcome

All eleven categories are gone from `package/`.

- `cli` batch:
   `113633b259` (merge `4fc48e3bb1`),
   with `rust-module/forbidden-regex` and `module/wg-allowedips`.
- Every remaining category:
   `592d78b143`,
   published in merge `ae249a3f7e`
   (the merge resolves `package/git-policy/cli.fuzz/Cargo.toml`
    and fixes references the move script missed:
    the toml-edit fuzz CI-scope near-miss inputs,
    `package/shim/<name>` and `package/cli/<name>` placeholders,
    one `task-util` path,
    the paused `webapp-forge` seed imports).
- The emptied old category directories held only ignored build output and were deleted.
- Mappings:
   `~/temp/agent/package-category-elimination/combined-moves.json`
   (plus the `cli` list in `cli-batch.ts`).

User decisions taken during execution
 (recorded in the plan):
 `kv-store/{core,memoize}` instead of `cache/*`
 (`.gitignore` ignores every `cache/` directory;
  negations rejected);
 fork leaves drop `-fork`;
 the guard's final-newline autofix honors git's `-text` attribute
 (`--no-enforce-final-newline` rejected).

## Guard releases cut for this work

- 0.2.10:
   skip forbidden-strings scans of content a parent tree already holds.
- 0.2.11:
   released by the issue 1804 session;
   carries the new-path scanner and seed lists.
- 0.2.12 (`d2c2d04bbc`,
   change `96962173be`):
   final-newline keeps files whose `text` attribute is unset.
  `package/jsonc-edit/core-rust.fuzz/.gitattributes` marks `seed/** -text`;
   its three newline-less seeds are byte-identical after the move
   (blob IDs match the pre-move commit).
- `cargo-publish.yml` wait-step fix:
   `306db1a809`
   (the exact `=x` pin broke the index lookup).

## Verified

- Before the commit:
   `//:prepare:pnpm:install`,
   `//:sync:files`,
   file-enforcer `--check` from `package/file-enforcer/core`,
   `cargo metadata --locked --offline` for the moved crates.
- `claude-cpu-watch.service`:
   `~/.config/systemd/user/claude-cpu-watch.service` links to `package/claude-cpu-cap/daemon/systemd/`;
   running from the new `src/cpu-watch.ts`.
- Type checks for every moved package
   (`~/temp/agent/package-category-elimination/verify-types.summary.txt`):
   88 pass,
   8 have no `lint:types` task,
   3 failed.
  `file-enforcer/core`:
   the preexisting TS7006 in `src/toolchain/update.ts` (user defers it).
  `dependency-catalog/cli-rust-update`:
   `registry.unit.test.ts` lacks `fetchDump`,
   from `monochromatic-fb`'s in-progress registry change, not the move.
  `upstream-fork/deepmerge-ts.fuzz`:
   its gitignored `*.local.*` files still imported `module-test`
   (the move script reads tracked files only);
   rewritten in place.
- Gitignored files the move script could not reach:
   81 compiled `.js` files and a source map beside moved sources still name old packages
   and regenerate on the next build of each package;
   `file-manager/variant-qt/.qmlls.ini` was pointed at the new build directory.

## Follow-ups done after the batches

- `75fd6a4b49`:
   the `forbidden-strings` publish job's detect step skips a push whose version is already on crates.io,
   so a package move no longer runs a doomed publish and release
   (run 40845758673 after the `cli` move failed that way;
    the `jsonc-edit` and `nested-wayland-session` jobs already skipped).
  Probed against the live index:
   0.5.0 skips,
   0.5.99 publishes.
- `065c567802`:
   `package/test-fixture/toml-edit/.gitattributes` declares `src/** -text`.
- `68527f4eb8`:
   the guard's final-newline policy no longer lists any package path;
   seeds and fixtures are exempt only through `-text`.
  New test `the_toml_edit_fixture_declaration_preserves_its_sources` fails with the declaration emptied and passes with it;
   the 21 final-newline tests and clippy pass.
  Installed guards pick this up with the next release;
   until then 0.2.12 still lists the two paths,
   which is harmless.

## Unit tests after the moves

`test:unit` for every moved package
 (`~/temp/agent/package-category-elimination/verify-tests.summary.txt`):
 60 passed,
 10 failed on the first run.
- Stale pre-move `dist/` (tests import built output),
   passing after a rebuild:
   `env-local-json/core`,
   `test-harness/core`,
   `dependency-catalog/cli-cube`,
   `monorepo-task/core-exec`,
   `upstream-fork/matcher.fuzz` (after building `upstream-fork/matcher`).
- Move-caused,
   fixed:
   `fs-path/core` root-discovery suffix,
   and `test-harness/core-diagnostic` (`96886d4f54`:
    the frame filter matched installed harness copies by bare old names such as `module-test/dist/`,
    which carry no npm scope,
    so the move script missed them).
- Not from the move,
   left open:
   `upstream-fork/{conf,p-map,p-memoize}.fuzz` parity checks fail on the pre-move commit `113633b259^` too
   (same p-memoize message,
    "The cache Map can't be cleared");
   `file-enforcer/core` `generate.unit.test.ts` expects `nightly-2026-10-28`,
    which the toolchain update replaced.

## Open follow-ups

- npm rename done 2026-10-29
   (the user ran `~/temp/agent/npm-first-publish/publish-renamed.sh`,
    then `finish-renamed.sh` after its stage 3 needed `--allow-publish`):
   `@monochromatic-dev/logger-core` 0.4.5 and `@monochromatic-dev/fs-path-core` 0.2.1 are published;
   `module-logger` and `module-fs-path` are deprecated ("Renamed to …", verified on the registry);
   the npm login was removed (`npm whoami` fails).
  `npm-publish.yml` is registered as both packages' trusted publisher (`--allow-publish`).
- First trusted-publisher releases (user:
   "just do new releases through npm-publish.yml now"):
   `logger-core` 0.4.6 and `fs-path-core` 0.2.2 (`66e9826bab`, version bump and changelog only).
  Runs 40863033623 and 40863038503 succeeded (pack and publish);
   npm printed `+ @monochromatic-dev/logger-core@0.4.6` and `+ @monochromatic-dev/fs-path-core@0.2.2`,
   a signed provenance statement for each (Sigstore log indexes 3565726298 and 3565721940),
   and "Your package is being processed".
  At this update the registry did not list 0.4.6 or 0.2.2 yet;
   a check was waiting up to 15 minutes.
  Next:
   confirm both versions and their provenance on the registry.
  A successful trusted publish also keeps npm from expiring the trusted-publisher setup (2-day rule).
- Three CI breakages surfaced on the way,
   each fixed and pushed:
  - `09b320f9c5`:
     nine files built old paths from separate segments
     (`join(..., 'package', 'dev-script', 'task-cli', ...)`),
     which the move script cannot see;
     the task runner re-invoked itself at the old path,
     so every CI `task-run` failed with `MODULE_NOT_FOUND`.
    Search used:
     an old category name quoted and followed by another quoted segment.
  - `9b5e5bf46a`:
     `fs-path/core` had no `task.ts`,
     which `task-run` requires;
     it had been unbuildable in CI since the workflows dropped mise (`38cb13b4d1`),
     not because of the move.
  - `c5cfe8e3a3`:
     the root `task.ts` still named `npm:module:pack` and `npm:module:publish`
     (renamed only in `mise.no-env.toml`),
     and ran `pnpm pack --recursive` and `pnpm publish --recursive`,
     which would pack and publish every workspace package.
    Both now match `mise.no-env.toml`:
     pack `./package/$NPM_PACKAGE`,
     publish that one tarball with provenance.
- `npm-release.yml` (changesets) fails in its `select-mode` step on every run,
   including before the moves,
   so it never reaches publishing;
   `doc/decision/npm-publishing.md` and `doc/runbook/publish-npm-package-first-time.md` still describe it as the release path.
- [labwc-config issue 3](https://github.com/Aquaticat/labwc-config/issues/3) already lists the final names
   (`logger-core`,
    `test-harness-core`,
    `or-throw-core`).
- `package/pnpm-override/shim-readable-stream/lib/*.js` are tracked but match the blanket `*.js` ignore,
   which `doc/decision/gitignore-negations.md` already names;
  explicit file pathspecs fail on them,
   so stage that directory instead.
- `56178d85ed` is mislabeled;
   corrective commit comment posted.
- `~/.local/bin/spawn-pi` and `spawn-claude` were broken symlinks before this work
   (old `packages/` layout and a plugin cache).

## Tooling

`~/temp/agent/package-category-elimination/`:
 `cli-batch.ts` (move and rewrite),
 `module-pathspec.ts` (path lists from `git status`,
  excluding other sessions' files),
 `batch-scan.ts` (diff lines no rewrite explains),
 `leftover-scan.ts` (old names left in a tree),
 `seed-precheck.ts`,
 `merge-probe.ts`,
 `verify-types.ts`.
