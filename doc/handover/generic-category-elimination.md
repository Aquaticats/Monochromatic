# Handover: generic package category elimination

Updated 2026-10-29 by session `monochromatic-db`.
Decisions,
 mappings,
 peer holds,
 and the execution queue live in
[the plan](../planning/generic-category-elimination.md);
this file records in-flight state the plan does not.

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
The user executes per category as soon as each mapping settles,
 coordinating with peer sessions.

## State right now

### `cli` batch landed

- `113633b259` (`refactor(*): move cli packages into product categories`,
   prepared as `ea175fbb12`,
   published in merge `4fc48e3bb1`)
  moved every `cli` package except `nested-wayland-session` (held by `monochromatic-ec`,
   since released) and `markdown-lint` (untracked debris),
  plus `rust-module/forbidden-regex{,.bench,.fuzz}` and `module/wg-allowedips`.
  It includes `pnpm-lock.yaml` from `monochromatic-fb`'s `pnpm install`.
- `monochromatic-fb` regenerated the root `mise.toml` PATH entries and `package/config/pnpr/config.yaml`
   in `328e2421b2` (`sync:files`);
  its file-enforcer `--check` passed right after.
- Guard 0.2.10 is installed (`e8e3769dfb`,
   `toolchain.lock.json` pins it);
  it skips scanning content a parent tree already holds,
   which let the renamed scanner data files through.

### Regression fixed:
 fuzz seed bytes

The installed guard's final-newline autofix exempts fuzz seeds by hardcoded path prefix
 (`PRESERVED_PREFIXES` in `package/git-policy/cli/src/native/policy_final_newline.rs`).
0.2.10 still carries the old prefixes,
 so the batch commit added or stripped a trailing newline in 28 seeds
 under `package/forbidden-strings/cli.fuzz/seed/`.
`73233b5e75` restored the pre-move bytes with `--no-enforce-final-newline`
 (script:
 `~/temp/agent/package-category-elimination/restore-seeds.ts`).
Until guard 0.2.11 is installed,
 any commit touching those seeds needs that flag,
 or the autofix changes them again.

### Uncommitted:
 `builtin-rules.txt` header rewrite

`package/forbidden-strings/cli/data/builtin-rules.txt` moved byte-identical,
 so its generated header still names the old paths.
The rewritten header sits in the main worktree,
 uncommitted:
 committing it changes the blob,
 the guard then scans the file,
 and its own rule text matches (`aws-amazon-bedrock-api-key-short-lived`,
 line 118),
 because 0.2.10 lists the scanner self-match files only at their old paths
 (`SCANNER_SELF_MATCH_PATHS` in `package/git-policy/cli/src/native/scanner_selection.rs`).
Copy at `~/temp/agent/package-category-elimination/builtin-rules.rewritten.txt`.

### `module` batch:
 applied in the main worktree,
 not committed

- Moves (`~/temp/agent/package-category-elimination/module-moves.json`):
   all of `module` and `rust-module`,
   plus `dev-script/file-enforcer` and the `dependency-catalog` tools
   (`monochromatic-fb` paused them for this window and wants the commit OID).
  User decisions:
   `kv-store/{core,memoize}` instead of `cache/*` (`.gitignore` ignores every `cache/` dir;
   negations rejected),
   fork leaves drop `-fork`.
- `npm-publish.yml` takes a package path;
   `mise.no-env.toml` tasks are `npm:package:{pack,publish}` with `pnpm --filter ./package/$NPM_PACKAGE`.
- Pre-commit checks pass (`module-precommit.ts`:
   install,
   `sync:files`,
   file-enforcer `--check` from `package/file-enforcer/core`,
   `cargo metadata`).
- Blocked on guard 0.2.12:
   three `jsonc-edit/core-rust.fuzz` seeds lack a final newline on purpose;
   the user rejected `--no-enforce-final-newline`,
   and chose that the final-newline policy honor `-text`.
  A subagent is implementing it;
   `package/jsonc-edit/core-rust.fuzz/.gitattributes` (new) marks `seed/** -text`.
  Then:
   release 0.2.12 (four-file bump),
   install,
   commit.
- Commit routine:
   `node module-pathspec.ts <repo>` (excludes other sessions' dirty files;
   extend its list when the scan flags one),
   `node batch-scan.ts <repo> module-moves.json module-scan.out.txt -- $(cat module-commit-pathspec.txt)`,
   then `git add --pathspec-from-file=module-add-pathspec.txt`
   and `git commit --file=module-commit-msg.txt --pathspec-from-file=module-commit-pathspec.txt`.
- This handover is excluded from the batch;
   after it lands,
   remove `package/rust-module/rust-linter-{core,pattern}` (untracked build output),
   run `lint:types` on moved packages,
   and message `monochromatic-fb`.

### Guard 0.2.11 (done)

Releasing 0.2.11 fixes both hardcoded path lists
 (the source on `main` already names the new paths).
`main`'s `package/git-policy/cli` now also contains another session's in-progress push-stacking work
 for [issue 1804](https://github.com/Aquaticats/Monochromatic/issues/1804),
so a release from `main` would ship it.
The user decides whether to release from `main` now,
 wait for the issue 1804 work to settle,
 or keep using the per-commit flags.
Release mechanics:
 bump `package/git-policy/cli/Cargo.toml`,
 both `Cargo.lock` files,
 and the `=<x>` pin in `package/git-policy/cli.fuzz/Cargo.toml`;
 push;
 `cargo-publish.yml` publishes on a version-changing push.
A manual dispatch defaults to a dry run:
 pass `--field dry-run=false`.

### Post-batch verification:
 running

`~/temp/agent/package-category-elimination/verify-cli-batch.ts`
 runs `//:prepare:pnpm:install`,
 file-enforcer `--check`,
 `cargo metadata --locked --offline` for the five moved crates,
 and `lint:types` for the moved npm packages plus `mcp/mvm`,
 `pi-plugin/ask-user-question`,
 `pi-plugin/morph-compact`,
 and `config/tofu`;
summary in `verify-cli-batch.summary.txt` beside it.
Known preexisting failure:
 `file-enforcer` TS7006 at `src/toolchain/update.ts:632-633` (user defers it).

## Next

1.   Read the verification summary;
     fix or record each failure.
2.   `git clean --dry-run -d -X package/cli/markdown-lint`,
     then remove it.
3.   Guard 0.2.11 per the user's decision,
     then commit the `builtin-rules.txt` header rewrite.
4.   Batch 2 (mapping:
      `~/temp/agent/package-category-elimination/batch2-moves.json`;
      add `desktop-app/ide`,
      `desktop-app/terminal`,
      and `cli/nested-wayland-session`,
      now released).
5.   The `module` batch waits for `monochromatic-fb` to message that its CI and file-enforcer cleanup are done.

## Tooling

- Move script:
   `~/temp/agent/package-category-elimination/cli-batch.ts`,
  `node cli-batch.ts <repoRoot> dry|list|apply [moves.json]`.
  It rewrites `package/<from>`,
   `../<from>`,
   bare `<from>`,
   and `@monochromatic-dev/<from with - >` forms with a sidecar-aware boundary,
  recomputes sibling-relative paths inside moved dirs,
  skips `doc/**/*.json{,l}` evidence and three planning docs,
  and gives historical Markdown (`doc/audit`,
   `doc/handover`,
   `HANDOVER*`,
   `CHANGELOG*`) one dated note instead of rewriting (user decision,
   `AGENTS.md` WR6).
- Before committing a batch,
   run `revert-scan.ts` and `addition-scan.ts` (same folder):
  `git commit -- <paths>` takes whole working-tree files,
   so another session's uncommitted edit inside a rewritten file would ride along,
   and `HEAD` can move under a staged batch while other sessions land commits.
- Pre-scan changed files with
  `package/forbidden-strings/cli/target/release/forbidden-strings --builtin-rules --rules .cache/forbidden-strings.rules.txt <files>`,
  plus a positive control on `builtin-rules.txt`.
- The guard now replays commits that lose a landing race and fast-forwards `main`;
  its output records `commit-replayed` with the published OID.

## Peer coordination

- `monochromatic-fb`:
   holds the `module` batch until it messages that its CI step and `file-enforcer` lint cleanup are done;
  `module/rust-catalog` and the catalog tools stay held.
  Told about the `plan.artifacts.ts` layout rewrite in `e8e3769dfb`.
- `monochromatic-ec`:
   released `desktop-app/ide`,
   `desktop-app/terminal`,
   and `cli/nested-wayland-session` after IDE 0.1.3 (`ide-v0.1.3`).
  When moving them,
   update `.github/workflows/ide-release.yml` and `//package/desktop-app/ide:<task>` addresses together with the dirs;
   `doc/handover/slint-ide-0x.md` still lists old paths (historical note rule applies).
- Translation-repair session:
   holds released;
   the katex shim and its `pnpm-workspace.yaml` `link:` override move in one commit,
   then tell it so it can annotate its ledger lines.

## Open follow-ups

- [Issue 1804](https://github.com/Aquaticats/Monochromatic/issues/1804):
   another session is implementing push stacking.
- `cargo-publish.yml` wait-step fix:
   `306db1a809`.
- `56178d85ed` is mislabeled;
   corrective commit comment posted.
- [labwc-config issue 3](https://github.com/Aquaticat/labwc-config/issues/3) tracks the renamed npm names there.
- After publishable renames:
   the user first-publishes new npm names,
   then `npm deprecate` the old ones.
