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
  Verified on the registry the same day:
   `logger-core` latest 0.4.6 and `fs-path-core` latest 0.2.2,
   each with a provenance attestation.
  These successful trusted publishes keep npm from expiring the trusted-publisher setup (2-day rule).
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
- `npm-release.yml` (changesets) failed in `select-mode` on every run:
   34 public `translation-repair-*` packages depend on ignored packages.
  Fixed in `33b6a407a5` (one `@monochromatic-dev/translation-repair-*` glob in `.changeset/config.json`);
   run 40864949257 passed `select-mode` and skipped version,
    pack,
    and publish (the local publish plan is empty).
  `doc/decision/npm-publishing.md` now records `npm-publish.yml` as the release path the user directed,
   and the npm runbook's repository owner is corrected to `Aquaticats`.
- [labwc-config issue 3](https://github.com/Aquaticat/labwc-config/issues/3) already lists the final names
   (`logger-core`,
    `test-harness-core`,
    `or-throw-core`).
- `package/pnpm-override/shim-readable-stream/lib/*.js` match the blanket `*.js` ignore,
   which `doc/decision/gitignore-negations.md` already names.
  The move commit `592d78b143` dropped them (deleted at the old path, never added at the new one);
   restored byte-identical in `5cb4a779ff`.
  Only `git commit --no-only -- <files>` from the repo root committed them:
   a directory pathspec silently committed nothing and explicit paths failed in the guard's `git add --all`
   (issue 1807);
   running from a subdirectory was refused (issue 1808).
- `file-enforcer/core`:
   the TS7006 error is gone (fixed elsewhere; `lint:types` passes);
   `55afe67909` makes the toolchain tests read the nightly and node versions from the plan,
   so daily toolchain bumps no longer break them.
- `dependency-catalog/cli-rust-update` `lint:types` passes now (fixed by its owning session).
- The 80 stale `src/*.js` files naming old packages were a one-off `tsc` emit from 2026-10-03
   (`noEmit` is set, so no build regenerates them; the earlier "regenerate on the next build" note was wrong).
  Moved to `~/temp/agent/package-category-elimination/stale-js-backup/`;
   `test:unit` passes for every affected package that has the task.
- `~/.local/bin/spawn-claude` and `spawn-pi` now link to
   `package/claude-code-plugin/source/src/cli/spawn-claude.ts` and `package/pi-plugin/spawn/dist/final/node/cli.mjs`;
   both print their help.
- Fork parity fixes:
  - `conf`: the fork was fine;
     autofix pass `3da31bdc3a` had corrupted the vendored upstream oracle `conf.fuzz/upstream-conf/`
     (arrow `set` became a `function`, losing `this`).
    Restored byte-exact in `4b2565bd3c` and `090879d2ff` (corrective commit comment posted on `4b2565bd3c`).
    User approved an oxlint ignore for that directory:
     `78c8068b76` (verified: oxlint reports no files for it, still lints its sibling `src/`).
    First coverage baseline frozen: `975ab54da3`.
  - `p-memoize`: upstream 9.0.0 (floating `>=9.0.0`) changed the clear message and the `cache: false` ordering;
     ported in `21bff0aa78`.
    User chose a full 9.0.0 port next:
     done in `ab6bf76143` (fork) and `aed44ff3c3` (sidecar, coverage baseline refrozen for the added functions only),
     README fix `55ca780397`.
    Rerun confirms fork `test:unit`, sidecar parity, and the coverage gate (8 files) pass;
     the sidecar's `coverage-report.ts` lint errors are fixed.
    `test:mutation` was not run (the subagent hand-mutated `dist` with 12 mutants, all caught).
    Open notes:
     `isCacheEnabled` is now `cache !== false` like upstream, so a falsy non-`false` `cache` fails, with no test pinning it;
     `mimicBestEffort` swallows its caught error (`void error`) because the package has no logger dependency.
  - `p-map`: the lockfile moved the floating oracle from 7.0.8 to 7.1.1 (`755685da03`, 2026-10-14);
     the 7.1.x engines were ported in `56aa14e30f` through `7c45cfd072`
     (parity and coverage gates pass, confirmed by a rerun).
    User chose to port the new 7.1 APIs too (`pMapSettled`, `pMapKeyed`, `pMapSettledKeyed`, `pMapStop`, `preserveOrder`),
     mirroring upstream's shapes.
    Done: `cc54c0acc6` (`pMapStop`, `pMapSettled`, `preserveOrder`), `89fdcb55c1` (keyed variants),
     the stack-trace link, sidecar parity tests, refrozen coverage baseline (`dca2e1f455`),
     unit suites (`a035dcc4f2`, `638225a63b`, `9c4daa4605`), fork README `9df5ebfe0e`, sidecar README `ab896e888e`.
    Rerun confirms fork `test:unit`, sidecar parity, and the coverage gate (10 files) pass.
    The subagent once stopped on `EDQUOT`
     (8.9G of leaked `native-binary-*` git-policy/cli test fixtures filled the user's `/tmp` tmpfs quota, issue #1809).
    Upstream's ava suite was hand-adapted per feature rather than ported wholesale;
     the differential fuzz against installed upstream is the parity guarantee for the new APIs.
  - `test:mutation` for `p-map` and `p-memoize` (user asked 2026-10-29):
     first attempt failed because `package/mutation-test/cli/dist` was built 2026-10-20 and still scanned
     `package/cli/mutation-test/src/container`; rebuilt (doc example fixed in `5ae7a23939`).
    Running as systemd user units `mutation-p-memoize` and `mutation-p-map`,
     logs in `~/temp/agent/package-category-elimination/mutation-*.log`,
     reports land in each package's `dist/mutation-report.json`; survivors still need triage.
  - A scan of `package/**/dist` for old category paths found stale builds;
     rebuilt `test-harness/core-matrix` (stack-frame filter fragments), `wg/cli-quicker` (build hint),
     `file-manager/variant-electron`, `doodle-widget/app`; all now clean.
    Remaining hits are old mutation reports, fuzz artifacts, and deliberate doc examples
     (`/repo/package/module/x` in oxlint plugins; translation-repair's pre-split history path).
  - `jsonc-edit/core-ts.fuzz` coverage: `6d66763505` refroze four source-explained deltas
     (removed `rebuildEntry` callbacks and the JSON.parse fast path)
     and made the driver reach `emitPlainJson` again;
     the gate passes (17 files).
    Nothing outside tests constructs the public `plainJson` value kind any more.
  - Each fuzz sidecar's `coverage-report.ts` still reports 3 `prefer-readonly-parameter-type` lint errors (pre-existing);
     the porting subagents fix theirs if cheap.
- `0c5e7be0ee`:
   ten fuzz sidecars built `SOURCE_MARKER` from `${sep}`-joined old segments (`package/module/<x>-fork/src`),
   a form the move rewrite could not see, so every `fuzz:coverage` gate projected nothing.
  After the fix, eight of ten gates pass.
  `conf.fuzz` lacked a baseline (now frozen);
   `jsonc-edit/core-ts.fuzz` reports covered-function drops in four files (`edit-comment.ts` 12 < 14, and so on),
   and a subagent is diagnosing whether source changes or lost driver reach explain them.
- Searches for moved paths so far covered literal paths, quoted segment lists (`'package', 'dev-script'`),
   escaped separators, and `${sep}` joins;
   test fixtures that use made-up paths such as `/repo/package/module/x` are left as they are.
- `56178d85ed` is mislabeled;
   corrective commit comment posted.

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
