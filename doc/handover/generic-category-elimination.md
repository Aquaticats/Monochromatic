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
- Type checks for every moved package:
   systemd unit `category-move-lint-types`,
   summary in `~/temp/agent/package-category-elimination/verify-types.summary.txt`
   (result pending at this update;
    known preexisting failure:
    `file-enforcer` TS7006 in `src/toolchain/update.ts`).

## Open follow-ups

- Record the `lint:types` results here and fix any move-caused failure.
- `cargo-publish.yml` misreads a moved manifest as a version change
   (its detect step reads the old path as empty),
  so each move push starts a doomed publish of an unchanged version
  (run 40845758673 after the `cli` move;
   run 40854963672 for this batch).
  A publish step that skips versions already on crates.io,
   as the guard's job does,
   would stop it.
- `PRESERVED_PREFIXES` in `package/git-policy/cli/src/native/policy_final_newline.rs`
   still lists `package/forbidden-strings/core.fuzz/seed/` (no such directory)
   and `package/test-fixture/toml-edit/src/` (no `.gitattributes` yet);
  giving the fixture a `-text` `.gitattributes` would let the list go.
- npm:
   `module-logger` and `module-fs-path` (and the published shims) were renamed;
   the user first-publishes the new names through `npm-publish.yml`
   (input is now a package path such as `logger/core`),
   then `npm deprecate` the old ones.
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
