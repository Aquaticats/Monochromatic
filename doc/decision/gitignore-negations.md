# Eliminate .gitignore negations at the source

## Status

Accepted,
 2026-07-11;
 implemented in full,
 2026-07-12.
The stale-negation cleanup landed first
 (its hunks shipped inside commit `078d1bcd3`;
 a scope-note comment on that commit documents them).
The three restructures then landed:
 seeds in #364 (commit `e02ae3974`),
 plugin bundles in #365 (commit `d6a858fb7`),
 embedded baseline in #366 (commit `44c4ab9bc`),
 repo de-rooting in #367 (commit `58995afff`).
The only remaining `.gitignore` negations are the two KWin ones
 discussed in the closing section.

## Goal

`.gitignore` negations (`!` re-includes) exist only where a broad ignore pattern
 collides with something the repo genuinely tracks.
Every negation is therefore a symptom:
 either the broad pattern is too broad,
 or the tracked thing lives in a place shaped like scratch output.
This decision removes the collisions themselves instead of patching them with re-includes.

## Landed cleanup

Four negation lines protected nothing and were deleted outright:

- `!*.config.js`:
  zero tracked `*.config.js` files existed;
  the only on-disk match sat inside an already-ignored zig cache under `target/`.
- The `.vscode` whitelist
  (`!/.vscode/` plus five file re-includes):
  no `.vscode` config file was ever committed in repo history,
  and `.zed/settings.json` is ignored,
  so the practiced policy is that editor config is machine-local.
  Replaced by a plain `/.vscode` (root sandbox sentinel file) and `.vscode/` (any editor dir).
- `!/packages/rust-module/forbidden-regex.fuzz/corpus/*/seed-*`:
  the corpus directories exist but contain zero committed seeds.
  The seeds design below makes the pattern permanently unnecessary.
- `!/packages/claude-code-plugin/*/dist/final/*.js`:
  the committed bundles are `.mjs` files under `dist/final/node/`,
  so this pattern never matched anything.

## Design: fuzz seed corpora move to `seeds/<target>/`

Applies to `package/forbidden-strings/cli.fuzz` (1344 committed seeds)
 and `package/forbidden-strings/core.fuzz` (none yet).

Committed seeds move out of `corpus/<target>/` into a sibling `seeds/<target>/` directory
 that is tracked plainly.
`cargo fuzz run` accepts multiple corpus directories
 (verified against the installed cargo-fuzz 0.13.2:
 `cargo-fuzz run [OPTIONS] <TARGET> [CORPUS]...`);
libFuzzer reads every listed directory and writes new discoveries only to the first,
 so invocations become `cargo fuzz run <target> corpus/<target> seeds/<target>`.
`corpus/` then holds only fuzzer scratch output and is ignored with no re-include.

Consequences:

- The seeder (`src/bin/seed-from-tests.rs`) writes to `seeds/<target>/`.
- The mise `smoke` and `run` tasks pass both directories explicitly
  (passing explicit corpus args replaces cargo-fuzz's default `corpus/<target>`).
- `package/forbidden-strings/cli.fuzz/README.md` and `doc/handover/forbidden-strings-fuzzing.md`
  update their corpus-layout descriptions.

Rejected alternatives:

- Keep the negation:
  leaves seeds and scratch output mixed in one directory,
  distinguished only by a filename prefix.
- Force-add seeds past a plain `corpus/` ignore:
  tracked-but-ignored files are invisible to `rg` and every gitignore-respecting tool;
  the tracked shim files in `package/shim/readable-stream/lib/` already demonstrate this failure mode.

## Design: plugin bundles move out of `dist/`

The committed Claude Code plugin bundles leave `dist/` for a distinct tracked directory
 (provisional name `bundle/`,
 subject to the NCD check at implementation time).
Each plugin's tsdown config already spreads the shared base config,
 so the change per plugin is a one-line `outDir` override.

This retires the `!/packages/claude-code-plugin/*/dist/` carve-out
 and makes `dist/` mean the same thing everywhere in the repo:
 regenerable scratch output,
 always ignored.

The constraints in `doc/decision/dist-in-git.md` stand unchanged:
 Claude Code copies plugin sources into a per-user cache without any install step,
 so built bundles must remain committed.
Only their directory name changes;
 that decision is superseded solely in its gitignore-carve-out mechanism.

Reference updates required:

- `.claude-plugin/plugin.json` hook commands
  (`${CLAUDE_PLUGIN_ROOT}/dist/final/node/...`).
- `package.json` `bin`,
   `module`,
   `exports`,
   and `files` entries.
- The session-start-housekeeping plugin,
  which cleans stale dist artifacts.
- Any mise tasks referencing the old path,
  plus a one-time `git mv` of the tracked bundles.

Rejected alternatives:

- Keep the negation:
  workable (the existing comment documents it),
  but keeps the only spot in the repo where `dist/` content is load-bearing and committed.
- Copy step (build into `dist/`,
   copy the final bundle into a tracked directory):
  pays the same reference-update cost while duplicating every bundle,
  with drift risk between built and committed copies.

## Design: forbidden-strings de-roots its rule files

Three coupled changes:

- **The baseline ships inside the binary.**
  `mise.port-betterleaks.ts` regenerates the betterleaks-ported baseline
  into `package/forbidden-strings/cli/` instead of the repo root;
  `lib.rs` embeds it via `include_str!` and exports it as a public constant,
  so the crates.io binary carries it and
  `package/forbidden-strings/core.bench` consumes the constant
  instead of an `include_str!` with a fragile `../../../../` path.
- **The baseline is pure opt-in.**
  A new flag activates it;
  resolution without any rules file still errors.
  Rationale:
  nobody invokes a secret scanner with zero configuration and expects it to work,
  and silently adding baseline rules to existing users' scans would be a behavioral break
  on a published 0.1.9 CLI.
  The published resolution order
  (`--rules`,
   then `FORBIDDEN_STRINGS_RULES`,
   then `./forbidden-strings.local.txt`)
  is untouched.
- **The repo stops materializing rules at the root.**
  file-enforcer concatenates only `forbidden-strings.append.txt`
  and `forbidden-strings.append.local.txt`
  into a file under an already-ignored scratch directory
  (proposal:
   `.cache/`),
  and the generated root `mise.toml` `[env]` points `FORBIDDEN_STRINGS_RULES` at it.
  Root `forbidden-strings.local.example.txt` and root `forbidden-strings.local.txt` both disappear,
  deleting the `!/forbidden-strings.local.example.txt` negation
  and the `/forbidden-strings.local.txt` ignore line.

Implementation cautions:

- Env-var propagation into git-hook execution contexts must be verified with a real invocation;
  `scan-candidates.ts` in the git-policies packages falls back to the cwd default when the env var is absent.
- The self-skip path lists
  (`package/forbidden-strings/cli/src/lib.rs`
  and both `scan-candidates.ts` copies)
  reference the root example file and must follow the baseline into the package.
  The package-anchored skip entry for it already exists and currently points at a nonexistent path.

Rejected alternatives:

- Rename the example file out of the `*.local.*` convention
  (`baseline`,
   `example`,
   and `local-example` variants were considered):
  fixes the negation but keeps materialized rules at the repo root.
- Multi-source CLI (repeated `--rules` or a pathsep-joined env var):
  removes the intermediate file entirely,
  but changes the public interface of a published tool
  and forces missing-file semantics for fresh clones without an appendix.
- Always-on embedded baseline with an opt-out flag:
  silently changes scan results for existing users.
- Narrowing the `*.local.*` catch-all:
  dozens of live machine-local files across the repo rely on it.

## Remaining negations and their root causes

With the three restructures landed,
 the file contains no re-includes from the families above.
Two newer negations arrived independently
 (commits `dd6a8c973` and `078d1bcd3`)
 and are out of scope here:

- `!/packages/kwin/key-helper/kwin-script/contents/code/main.js`,
  colliding with the blanket `*.js` ignore.
- `!/packages/kwin/key-helper/kwin-script/metadata.json`,
  colliding with the blanket `metadata.json` ignore from the search-index family.

KWin's KPackage layout mandates both paths,
 so these cannot be renamed away.
Observation for any future pass:
 blanket basename ignores (`*.js`,
 `metadata.json`,
 `meta.json`)
 are the repo's remaining negation generators;
 eliminating these two at the source would mean scoping those blanket patterns instead,
 which is undecided and deliberately not designed here.

## Music player design runtime (2026-10-18)

Four more negations were added for the Design Component runtime files that 97 tracked music player design pages load
 (`support.js` in `design/` and `design/candidates/`, plus `artists.js` and `buckets.js` in `design/candidates/`).
The pages name the files by relative path (`./support.js`), so renaming them is not possible without editing every page.
The blanket `*.js` ignore hid them from `git status`, so a fresh clone rendered no page (issue 1639).
This is the same family of collision as the KWin pair: a blanket basename ignore meeting tracked source.
Scoping `*.js` remains undecided and was not designed here.
