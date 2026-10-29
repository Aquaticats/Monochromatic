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

- npm:
   `@monochromatic-dev/logger-core` 0.4.5 and `@monochromatic-dev/fs-path-core` 0.2.1 are published
   (the user ran `~/temp/agent/npm-first-publish/publish-renamed.sh`;
    fs-path-core went through npm staged publishing,
    shown as a `0.0.0-stage` placeholder until approved).
  Its stage 3 failed because `npm trust` also needs `--allow-publish`.
  Still to run:
   `bash ~/temp/agent/npm-first-publish/finish-renamed.sh`
   (trusted publisher for both,
    `npm deprecate` of `module-logger` and `module-fs-path`,
    `npm logout`;
    the npm login token stays on disk until it runs).
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
