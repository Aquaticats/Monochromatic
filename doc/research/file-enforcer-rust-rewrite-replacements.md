# Replacement options for each file-enforcer output

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 by a research agent with disposable fixtures under `${HOME}/temp/agent/fe-rust/fx-*`;
F marks a fact (read or measured), I an inference.


Scope: read-only research. No repo file was edited or committed. Fixtures live under `/var/home/user/temp/agent/fe-rust/fx-*`.
Doc sources saved under `/var/home/user/temp/agent/fe-rust/docs/`. Clones (depth 1, 2026-10-29): `cargo-2026-10-29` (06413ecf8c),
`mise-2026-10-29` (70d216e92b), `lsp4ij-2026-10-29` (116a18d1a4), all under `/var/home/user/temp/agent/`.
Tool versions used for probes: Claude Code 2.1.312, npm 11.20.0, pnpm 12.16.1, cargo 1.101.0-nightly (2026-10-12), mise 2026.10.20, git (system).
Marks: **F** = fact (read or measured), **I** = inference. Text-fragment links were composed from page text read via the `.md`/HTML fetch,
not confirmed in `agent-browser`.

Cross-cutting facts that change several answers:

- **F** `.claude/` and `.factory/skills/` are gitignored (`.gitignore:125-126`), so the skill mirrors are per-clone state, never committed.
  `.cache/`, `*.local.*` (so `.browserslistrc.resolved.local.json`, `forbidden-strings.append.local.txt`) are also ignored (`.gitignore:11,68`).
- **F** Tracked outputs: `CLAUDE.md`, `LICENSE`, `mise.toml`, `toolchain.lock.json`, `mpm.packages.toml`, `mpm.managers.toml`, 457 files under `package/**/LICENSES/`.
- **F** Git keeps a symlink as mode 120000 and checks it out as a symlink (fixture `fx-git`); with `core.symlinks=false` the checkout is a plain file
  holding the target path text (fixture: `CLAUDE.md` contained `AGENTS.md`). `git archive` keeps symlinks as symlinks.
- **F** No `REUSE.toml`, no `.reuse/`, no `reuse` invocation anywhere in the repo.
- **F** Precedent for check-only guards instead of generators: `cli-git.config.jsonc:9-11` runs `mono/forbidden-root-context` (the `CONTEXT.md` guard)
  in the Rust git-policy wrapper, which duplicates `file-enforcer.config.ts:593-611`.

## CLAUDE.md

1. Contents and readers
   - **F** 49-line preamble (`file-enforcer.config.ts:2523-2571`) + `AGENTS.md` verbatim (`:2572`); `diff` is one hunk `0a1,49` (`doc/audit/dsh-setup-gaps-2026-10-17.md:198-205`).
   - **F** The only recorded reason is a comment, `file-enforcer.config.ts:2520`: "Claude Code's @include is unreliable". It dates from the package's
     first commit (`fc314fb217`, 2026-02-20); no evidence is recorded. Not reproduced here (below).
   - **F** Readers: Claude Code. Pi 1.4.0 picks the first of `AGENTS.override.md, AGENTS.md, AGENTS.MD, CLAUDE.md, CLAUDE.MD` per directory
     (`doc/troubleshooting/pi-instruction-snapshots.md:1669`; Pi `docs/configuration.md:45`), so Pi never sees the CLAUDE.md preamble here.
     Codex takes `AGENTS.override.md`, then `AGENTS.md`, one file per directory ([Codex AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md#:~:text=Codex%20includes%20at%20most%20one%20file%20per%20directory)).
     Factory Droid reads `AGENTS.md` natively (list includes `CLAUDE.md` only as compatibility name, [Factory AGENTS.md](https://docs.factory.ai/cli/configuration/agents-md)).
   - **F** DSH loads both files and charges both against its byte cap (67,895 B pair vs 65,536 cap, `doc/audit/dsh-setup-gaps-2026-10-17.md:150-200`), so the copy costs context in that tool.
   - **F** The Harper LSP policy names both files (`file-enforcer.config.ts:1417-1442`); `README.md` of file-enforcer core states the copy rule (`package/file-enforcer/core/README.md:12`).
2. Status: tracked.
3. Replacements
   - **R1 hand-written `CLAUDE.md` = preamble + `@AGENTS.md`.** **F** Claude Code memory docs: "CLAUDE.md files can import additional files using `@path/to/import` syntax"
     ([memory#import-additional-files](https://code.claude.com/docs/en/memory#import-additional-files)); relative paths resolve from the importing file, depth four hops,
     in-tree imports need no approval dialog (only imports resolving outside the working directory do). **F** Probe: fixture `fx-claude/a` (`CLAUDE.md` = `@AGENTS.md`)
     made `claude -p` answer the codeword stored only in `AGENTS.md`; control `fx-claude/f` (same `AGENTS.md`, `CLAUDE.md` without the import) answered NONE, so the import is what delivered it. Breaks: nothing known; Pi/Codex/Droid still read `AGENTS.md`; the preamble stays Claude-only.
   - **R2 `CLAUDE.md` is a symlink to `AGENTS.md`.** **F** Docs: "If you don't need Claude-specific content, a symlink also works: `ln -s AGENTS.md CLAUDE.md`"
     ([memory#share-one-file-with-other-coding-tools](https://code.claude.com/docs/en/memory#share-one-file-with-other-coding-tools:~:text=If%20you%20don%27t%20need%20Claude-specific%20content%2C%20a%20symlink%20also%20works)).
     **F** Probe `fx-claude/b` worked. **F** Docs constraints: Edit/Write refuse to write through the symlink (they redirect to the target); Windows clones without `core.symlinks` get a one-line file.
     Breaks: the 49-line preamble disappears unless it moves into `AGENTS.md`. **I** Moving it there also delivers it to Pi/Codex/Droid, which today never receive it
     (decision needed: some preamble lines name Claude-only features, `spawn-claude`, `claude-code-bash` cgroup).
   - **R3 delete `CLAUDE.md`.** **F** Docs table: with an `AGENTS.md` and no `CLAUDE.md`/`CLAUDE.local.md` in the working directory or above, Claude reads `AGENTS.md`;
     needs Claude Code v2.1.277+, and sessions on Bedrock or telemetry-disabled read `CLAUDE.md` only before v2.1.281
     ([memory#agents-md](https://code.claude.com/docs/en/memory#agents-md)). A gitignored `CLAUDE.local.md` silently switches this off. `InstructionsLoaded` hooks do not fire for it.
     Breaks: preamble must move into `AGENTS.md`; the Harper policy loses a name; DSH double-charge disappears.
4. Recommendation: R1 if the preamble stays Claude-only (smallest change, no generation, no duplicate bytes for DSH-style loaders because the import is one file).
   R3 (or R2) if the owner wants one policy file for every agent; this is a content decision, not a mechanism one. The "unreliable include" premise did not reproduce on 2.1.312 in one fixture, so it is no longer a requirement (**I**).

## .claude/skills and .factory/skills mirrors

1. Contents and readers
   - **F** Byte copies of `.agents/skills/*/*.md` (12 files, 10 dirs) plus `.agents-mirror-manifest.json` (`file-enforcer.config.ts:1295-1358`). Nothing in the repo reads the mirrors
     (`rg` for `.claude/skills`, `.factory/skills`, `agents-mirror` outside the generator and docs: no hits). Only the agent tools read them.
2. Status: gitignored (`.gitignore:125` `/.claude/`, `:126` `/.factory/skills/`).
3. Replacements
   - **Factory Droid: drop the mirror.** **F** Droid's skill locations include "Compatibility: `<repo>/.agents/skills/**/SKILL.md`"
     ([Factory skills](https://docs.factory.ai/cli/configuration/skills#:~:text=Compatibility)). Not executed (no Droid API session); docs only.
   - **Pi, Codex: nothing needed.** **F** Pi: "Pi also supports the Agent Skills locations `~/.agents/skills/` and `.agents/skills/`" (Pi 1.4.0 `docs/skills.md`).
     Codex scans `.agents/skills` up to the repo root and "supports symlinked skill folders" ([Codex skills](https://developers.openai.com/codex/skills#:~:text=Codex%20supports%20symlinked%20skill%20folders)).
   - **Claude Code: symlinks, not copies.** **F** "a `<skill-name>` entry in the enterprise, personal, or project location can be a symlink to a directory elsewhere on disk"
     ([skills docs](https://code.claude.com/docs/en/skills#:~:text=can%20be%20a%20symlink%20to%20a%20directory%20elsewhere%20on%20disk)); the docs list no `.agents/skills` location. **F** Probes (`fx-claude`):
     per-skill symlink `.claude/skills/x -> ../../.agents/skills/x` loaded; whole-directory symlink `.claude/skills -> ../.agents/skills` loaded
     (not documented, works on 2.1.312, **I** may change); plain `.agents/skills` with no link: not seen. Live change detection covers symlink targets for `SKILL.md` text only per docs.
     Breaks: a committed link needs `.gitignore:125` (`/.claude/`) changed to `/.claude/*` plus a `!/.claude/skills` negation (a child of an ignored directory cannot be re-included), which cuts against `doc/decision/gitignore-negations.md` (Accepted 2026-07-11: negations are a symptom, only two KWin ones remain); the alternative is one uncommitted `ln -s` per clone. **F** Docs: a linked worktree with no `.claude/skills` at its root
     falls back to the main checkout's project skills on v2.1.277+ ([skills docs, worktree paragraph](https://code.claude.com/docs/en/skills#discovery-from-parent-and-nested-directories)),
     which already covers worktrees if the main checkout has the link.
4. Recommendation: drop `.factory/skills` outright; replace `.claude/skills` with one committed symlink (`.claude/skills -> ../.agents/skills`) or per-skill links,
   after the `.gitignore` change. This deletes the manifest, pruning, and path-traversal logic (the mirror rule). Prefer per-skill links if the owner wants a documented path.

## Root LICENSE and per-package LICENSES/<id>.txt

1. Contents and readers
   - **F** Root `LICENSE` is a byte copy of `LICENSES/LGPL-3.0-or-later.txt` (7,652 B, `file-enforcer.config.ts:2515`). Package copies: per manifest license string, GPL + LGPL texts (35,149 + 7,652 B)
     plus CC-BY-SA when named (`:116-119,318-336,549-581`). Known ids handled: CC-BY-SA-4.0, GPL-3.0-or-later, LGPL-3.0-or-later; MIT/Apache/OFL/markdownlint copies in 14 packages are hand-kept.
   - **F** Measured: 225 `package/*/*/package.json`, 152 `"private": true`, 73 not private; 202 have a `LICENSES/` dir; 26 `Cargo.toml` tracked.
     So most copies sit in packages that are never published (**I**: no distribution reader).
   - **F** Readers: `files` arrays (`"LICENSES"`, e.g. `package/file-enforcer/core/package.json:13-16`), Cargo `include = ["LICENSES/*.txt"]` (`package/forbidden-strings/cli/Cargo.toml:7-20`),
     publish paths `pnpm pack` (`task.ts:379-381`, `package/config/pnpr/src/publish-steps.ts:140`) and `cargo package` (`.github/workflows/cargo-publish.yml:161,457`).
   - Storage cost: **F** identical content hashes to four git blobs total; the cost is working-tree bytes (about 9.5 MB for 221 pairs, **I** from sizes) and 457 tracked paths.
2. Status: tracked (LICENSE, 457 package copies).
3. Replacements
   - **Symlinks for npm/pnpm packages: do not work.** **F** Fixture `fx-license`: `npm pack --dry-run` and `pnpm pack` omit symlinked files and a symlinked `LICENSES` directory
     (also a symlinked root `LICENSE`); the same layout with regular copies packs all files (positive control, pkg `d`). The npm-packlist README does not document this; the measurement is the evidence.
   - **Symlinks for crates: work.** **F** `cargo package --list` and the built `.crate` contained the symlink target's content for both a linked file and a linked `LICENSES` directory;
     cargo docs: "Symlinks are flattened to their target files" (`doc/book/src/commands/cargo-package.md:30` in the clone). So the ~26 Cargo packages can use committed links; **I** `cargo publish` uses the same packaging.
   - **Pack-time injection (build-time step in the consumer).** **I** The npm publish path packs through one place (`publish-steps.ts:140`, plus `task.ts:379`); a step that adds the texts to the tarball
     removes all 200+ copies. Costs: two packers to patch, and `pnpm pack` has no documented hook for outside files. Not tried.
   - **Drop copies for private packages.** **F** (152 private). Whether LGPL distribution duty applies to non-published packages is a legal call for the owner, not tested.
   - **REUSE/SPDX.** **F** REUSE 3.3: "A Project MUST include a License File for every license ... placed in the LICENSES/ directory in the root of the Project"; `LICENSE`/`COPYING` "SHOULD be in the root"
     ([spec 3.3](https://reuse.software/spec-3.3/#:~:text=Each%20License%20File%20MUST%20be%20placed%20in%20the%20LICENSES%2F%20directory%20in%20the%20root%20of%20the%20Project)). Per-package copies are not required by the spec for this repo as one Project; the repo does not run `reuse`.
   - **GitHub detection of root `LICENSE`.** **F** licensee issues #155 and #227 report symlinked license files undetected (both closed stale, no fix); #737 (scan `LICENSES/`) closed COMPLETED 2026-01-29, deployment on github.com not verified.
     So keep `LICENSE` a regular file (**I**). Current API shows `lgpl-3.0` for the repo (`gh api repos/Aquaticats/Monochromatic`).
4. Recommendation: keep root `LICENSE` as a plain committed file (one-time copy, change once a decade; no generator needed, a check is enough).
   Package copies: delete the generator; for the 26 Cargo packages use committed symlinks; for npm packages choose between pack-time injection and keeping copies for the 73 non-private ones only.
   Do not claim symlinks for npm; they ship nothing.

## Root mise.toml

1. Contents and readers
   - **F** Header + `mise.no-env.toml` verbatim + an 80-line `[env]` block (`file-enforcer.config.ts:691-781`): sops `_.file`, `_.path` of four `.cache/*` dirs, `node_modules/.bin`, then every existing
     `package/*/*/node_modules/.bin` (60 entries now), two Slint vars, `FORBIDDEN_STRINGS_RULES`. The bin list is machine-dependent (issue #335, `:719`).
   - **F** Readers: mise itself; `package/monorepo-task/core-exec/src/build-and-test-fixture-config.ts:23,13-20` reads both files in tests; docs/AGENTS (`WC2`).
   - **F** 58 commits touched `mise.toml` since 2026-10-01 (`git log --since`), mostly mirrored edits.
2. Status: tracked.
3. Replacements (mise 2026.10.20 docs in the clone, and probes in `fx-mise`)
   - **Merge by hierarchy instead of generation.** **F** Several files per directory are loaded and combined: `mise.toml`, `mise.local.toml`, `.config/mise/conf.d/*.toml`, `.mise/conf.d/*.toml`
     ([configuration](https://mise.jdx.dev/configuration.html#configuration-hierarchy); clone `docs/configuration.md:41-148`). **F** Probe: `.config/mise/conf.d/env.toml` `[env]` value was visible next to `mise.toml` tasks.
     So the static `[env]` text can be a committed `.config/mise/conf.d/env.toml` and `mise.no-env.toml` simply becomes `mise.toml`; nothing is generated.
   - **No general include key for `[env]`.** **F** `rg` of `docs/configuration.md` and `docs/tasks/` finds only task-level includes, no top-level include. **I** conf.d replaces the need.
   - **Glob in `_.path`: not expanded.** **F** Docs: `_.file` ("The path can be a glob pattern", `docs/environments/index.md:262`) and `_.source` (same sentence in its section) accept globs, `_.path` does not say so; probe: `_.path = ["package/*/*/node_modules/.bin"]` put the literal `*` string into PATH.
   - **Dynamic bin dirs via `_.source`.** **F** Probe: `_.source = "scripts/path.sh"` (a bash loop prepending existing `package/*/*/node_modules/.bin`) added the dirs; the `_.path` entry stayed first.
     Order within the globbed set reverses (prepend per iteration) and `_.source` costs a bash run per env computation (docs: "runs each time mise computes the environment"). It needs a bash script file, which the repo's SCR rule bans. A `.ts` task cannot change mise's env, so the TypeScript route is different: a task writes the bin list into a gitignored fragment (`.config/mise/conf.d/dyn/mise.local.toml` or `conf.d/x.local.toml`), which **F** probe `fx-mise` showed loads, and `*.local.*` already ignores both (`.gitignore:11`, `git check-ignore` confirmed). That is generation moved into the consumer, not elimination, but it removes the machine-dependent part from the committed file (answers issue #335).
   - **Drop the dynamic part.** **I** The 60 workspace bin dirs are only needed for tools that pnpm already puts on PATH for `pnpm run`; issue #335 already proposes deriving it from workspace metadata. Not tested.
   - **Mise is slated for removal** (`doc/decision/monorepo-manager-all-rust.md:68`), so the minimum is to stop generating and let the tool die with its config.
4. Recommendation: stop generating now. Rename `mise.no-env.toml` to `mise.toml`, commit the static `[env]` block as `.config/mise/conf.d/env.toml` (the `{{config_root}}` template already makes
   `FORBIDDEN_STRINGS_RULES` static), and either drop the workspace bin list or add it as a `.ts` task. Cost: 2 test files and docs that name `mise.no-env.toml` (rg: 18 non-doc files mention `mise.no-env`, incl. fuzz and kotlin package mise files) must be repointed.

## .cache/forbidden-strings.rules.txt and forbidden-strings.append.local.txt

1. Contents and readers
   - **F** Rules file = 7-line header + `forbidden-strings.append.txt` (9,503 B tracked) + `forbidden-strings.append.local.txt` (4,904 B, gitignored, mode 0600), mode 0600 (`file-enforcer.config.ts:896-916`);
     the post-step compiles the runtime cache (`:819-839`). The seed (`:869-891`) is a comment-only header created if absent.
   - **F** Readers: `FORBIDDEN_STRINGS_RULES` in generated `mise.toml:1350`; `cli-git.config.jsonc:19` (`rulesFile`); a fuzz seed `package/git-policy/cli.fuzz/seed/rules_file/repository-default`; docs `package/forbidden-strings/cli/README.md:135,174-176`.
     `AGENTS.md:35,44-45` names both appendix files directly. CI: `rg` over `.github` and `Dockerfile` finds no scanner rules-file use.
   - **F** Scanner: `--rules` takes one path (`package/forbidden-strings/cli/src/cli.rs:205-215`), env default `FORBIDDEN_STRINGS_RULES`, loader `fs::read_to_string` on a single path (`src/frx_load.rs:191`); no include, no directory, no list.
     Section names must be unique across the concatenation. Cache identity is the exact rules text (`cli.rs:72-74`), so a combined text must be materialized in memory anyway.
2. Status: rules file gitignored (`.gitignore:68`); seed gitignored (`:11`); `append.txt` tracked.
3. Replacements
   - **Scanner reads several files.** **F** needs a code change (accept repeated `--rules`, a path list in the env var, or an automatic sibling `*.local` lookup) in both the CLI and the git-policy `rulesFile` option
     (`package/git-policy/cli/src/native/config_rules_file.rs:140`, `config_schema.rs:140`). **I** Small: concatenate in memory before the existing compile step.
     Breaks: both consumers must learn the new input; the `0600` rule applies to the local file only.
   - **Drop the seed.** **I** Nothing reads the seed's text; its comment is documentation, so move it into `forbidden-strings.append.txt`'s header or the scanner README, and let the scanner treat the local file as optional.
   - **Keep the concatenation as a build-time step in the consumer.** **I** `forbidden-strings compile-rules` already exists; give it multiple inputs instead of making file-enforcer concatenate.
4. Recommendation: make the scanner the owner (multiple rules inputs, optional local file) and delete both generated pieces and the seed. This also removes the env var that only the generated `mise.toml` sets.

## Per-crate Cargo.toml keys and [workspace.dependencies]

1. Contents and readers
   - **F** 26 tracked `Cargo.toml`, each its own workspace root by decision (`doc/planning/cargo-toml-file-enforcer.md:11-30`, `.out-of-scope/cargo-workspace.md`). Tool-owned keys: `edition="2024"`, `license`, `publish=false`,
     six `[lints.clippy]` keys, `repository`, `homepage`, `readme`, profile keys by directory (10 dirs, 6 presets, `file-enforcer.config.ts:~1560-1700`), Dylint `check-cfg` for 2 dirs, and the whole `[workspace.dependencies]` table from `rust-catalog.toml` (143 lines).
   - **F** Readers: cargo, clippy, rust-analyzer, `cargo publish`; CI drift check `.github/workflows/advisory-core.yml:92-105`; `rust:update`/`rust:verify` read the catalog.
2. Status: tracked.
3. Replacements, each checked against the Cargo book in the clone (`doc/book/src/reference/config.md`, `manifest.md`) and by fixture `fx-cargo`
   - **`.cargo/config.toml` `[profile]`.** **F** Config profiles overlay the manifest key by key: `merge_config_profiles` "Takes the manifest profiles, and overlays the config profiles on-top" (`src/workspace/profiles.rs:1322-1345`);
     docs: config `[profile]` can "override settings specified in Cargo.toml". **F** Probe: a config `opt-level=2` beat the manifest's `opt-level=1`, and config `codegen-units=1` beat the manifest's `7`.
     So config cannot hold a default that a crate overrides; it only forces. **F** Config discovery is rooted at the current directory, not the manifest (`src/context/mod.rs:719`, docs "current directory and all parent directories");
     probe: `cargo build --manifest-path package/cat/crate/Cargo.toml` from the repo root did not read `package/cat/crate/.cargo/config.toml` (`codegen-units=7` stayed), from the crate dir it did.
     **I** Load-bearing presets (scanner `panic="unwind"`, `overflow-checks=true`, `.out-of-scope/cargo-workspace.md`) would silently differ by invocation directory and IDE launch directory. The repo root `.cargo/config.toml` already exists (tracked, `[env]` only).
   - **Config `include`.** **F** Stable since 1.93 ("Support for including extra configuration files via the `include` config key has been stabilized in 1.93.0", `doc/book/src/reference/unstable.md:2327-2331`);
     paths are relative to the including file, `.toml` only, `{ path, optional }` form. **F** Probe: a crate-level `.cargo/config.toml` `include = ["../../../../shared/profile-scanner.toml"]` worked (note it counts from the `.cargo/` directory).
     This lets a shared preset file be pulled in per crate without generation; it still needs one `.cargo/config.toml` per crate and has the cwd caveat. No `-Zconfig-include` is needed on this toolchain.
   - **`[lints.clippy]` via `build.rustflags`.** **F** No `[lints]` key exists in config (`rg` of `config.md`: none). **F** Probe: `build.rustflags = ["-Dclippy::implicit_return","-Aclippy::needless_return"]` made `cargo clippy` fail on an implicit return.
     Caveat **F**: `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS` env replace `build.rustflags` entirely (`config.md:614-618`), and the fuzz crates' tasks run under cargo-fuzz's injected `RUSTFLAGS` (`package/*.fuzz/mise.toml`), so lints would silently turn off there. Not recommended.
   - **Workspace inheritance across roots.** **F** Not available: nested workspaces issue rust-lang/cargo#5042 is still OPEN (updated 2026-08-04), remote version definitions #15435 OPEN. Cargo's own table for `[workspace.package]`, `[workspace.lints]`, `[workspace.dependencies]` needs one root;
     `.out-of-scope/cargo-workspace.md` rejects that for profile ownership, Gradle and Podman reasons.
   - **Check instead of write.** **I** `edition`, `license`, `repository`, `homepage`, `readme`, `publish` are one-line, change rarely, and have no machine reader besides `cargo publish`; a read-only policy (like `mono/*` in `cli-git.config.jsonc`) can fail on drift without a splice engine.
     For 2 Dylint crates, hand-write `[lints.rust.unexpected_cfgs]`.
   - **`[workspace.dependencies]`.** **F** The `>=` floors mean the table only centralizes version text; the pins live in each `Cargo.lock` (`rust-catalog.toml:1-12`, `doc/decision/rust-dependency-catalog.md`). **I** The table must remain a rendered projection of the catalog (or become `check`-only);
     there is no native cross-root mechanism.
4. Recommendation: keep profiles hand-written in each `Cargo.toml` (they carry bespoke rationale comments and are load-bearing); do not use config profiles for them. **F** A root `.cargo/config.toml` is found from any directory in the repo and supports `[profile.<name>.package.<crate>]`, but "Overrides cannot specify the `panic`, `lto`, or `rpath` settings" (`doc/book/src/reference/profiles.md:638`), so the scanner and fuzz presets (`panic`, `lto`) cannot be expressed per crate in a root config; only plain `[profile.release]` there would apply to every crate. Per-crate settings other than those three (`opt-level`, `codegen-units`, `overflow-checks`, `strip`) could live in one root file. Convert the simple metadata keys and lints to a check-only policy.
   Keep the catalog-to-`[workspace.dependencies]` projection as the one real generator in Rust, or reopen the single-workspace decision (owner's call).

## .browserslistrc.resolved.local.json

1. Contents and readers
   - **F** JSON array of 18 targets resolved from `.browserslistrc` with config and stats discovery off (`file-enforcer.config.ts:936-967`). Only reader: `package/config/rolldown/src/browserslist-targets.ts:905-946`
     (called at top level by `index.ts:14`, `index.node.ts:34`, `index.client.ts:15`).
   - **F** The consumer already falls back: when the file is absent it imports `browserslist` and resolves `undefined` queries with `path: process.cwd()` (`browserslist-targets.ts:930-944`).
   - **F** Rolldown 1.2.15 `transform.target` takes strings like `chrome58`, not queries (`node_modules/.pnpm/rolldown@1.2.15/.../binding-CVMT71K4.d.mts:1170-1181`), hence the conversion step.
2. Status: gitignored (`.gitignore:11`).
3. Replacements
   - **Delete the file and the file branch; always resolve live.** **F** Measured: `browserslist(undefined, {path: "package/config/rolldown"})` returned an array identical to the generated file (18 entries, same order).
     Breaks: none today; behavior differs only if a nearer `browserslist` config or `browserslist-stats.json` exists below the root (**I**), and the output now depends on installed `caniuse-lite` data at build time, as the file already did at sync time.
4. Recommendation: drop the output; delete `generatedFileExists`/`importJson` and the generator. Add `path` pointing at the repo root if per-package config drift is a worry.

## toolchain.lock.json, mpm.packages.toml, mpm.managers.toml

1. Contents and readers (details in `doc/research/file-enforcer-rust-rewrite-toolchain-split.md:141-186`)
   - **F** `toolchain.lock.json` (54,628 B): resolved versions, URLs, SHA-256 per tool. Readers by `rg`: `Dockerfile:24-49` (jq), `.github/actions/task-cli-setup/action.yml:42,46` (jq),
     `.github/workflows/music-player-release.yml` (30 hits), `kotlin-linter-publish.yml` (5), `fs-id.yml` (1), `package/test-harness/core-matrix/src/runtime.ts`, `package/mutation-test/cli/runtime/Containerfile`,
     `package/dependency-catalog/cli-rust-update/src/toolchain.ts`, IDE Rust tests (`package/ide/app/src/language/...`). It is a lockfile, so it is generated by nature; the question is who writes it.
   - **F** `mpm.packages.toml` (1,123 B): one reader, `mpm --rustup restore` in `package/file-enforcer/core/src/toolchain/providers.rust-android.ts:81-88`, which falls back to direct rustup when mpm fails.
     `mpm.managers.toml` (7,414 B, 18 cargo-binstall manager overrides): one reader, the documented shadow install `mpm --config mpm.managers.toml ... install monochromatic-mpm-shadow`
     (`doc/runbook/publish-crate-first-time.md:463`, `doc/decision/mpm-toolchain-adoption.md:348`). `rg` finds no other reader outside `package/file-enforcer/`.
2. Status: all tracked.
3. Replacements
   - **Lock written by the updater, no plan-to-lock render (Option A in the split doc).** **I** `ToolchainLockTool` already equals the plan entry minus `id`; the lock holds every field. Keeps jq readers working (JSONC would break jq).
   - **Drop `mpm.packages.toml`.** **I** Its only reader has a rustup fallback; or have the installer pass the channel directly.
   - **Hand-write `mpm.managers.toml` with the single entry actually used** (`cargo-binstall-monochromatic-mpm-shadow`) **I**, or let mpm-shadow materialize it. The other 17 override blocks have no reader.
4. Recommendation: keep `toolchain.lock.json` (owned by the updater, validated by `check`); drop `mpm.packages.toml`; shrink `mpm.managers.toml` to what the runbook uses. Frozen surface for the lock: tool ids, `version`, `artifacts.<platform>.{url,sha256}`, `options.androidPackages` (split doc :158-160).

## JetBrains Harper LSP4IJ policy (written under ~/.config/JetBrains)

1. Contents and readers
   - **F** Edits two XML files per IDE version under `~/.config/JetBrains/<IntelliJIdea|IdeaIC><newest>/options/` (`LanguageServersSettings.xml`, `UserDefinedLanguageServerSettings.xml`): three rules off, exclude patterns for `AGENTS.md`/`CLAUDE.md`,
     a second scoped server (`file-enforcer.config.ts:1396-1464`). Reader: LSP4IJ in the user's IDE. Outside the repo, per user.
2. Status: not in the repo (home directory).
3. Replacements, from LSP4IJ source (clone `lsp4ij-2026-10-29`)
   - **Project-level committed config: not for this.** **F** `ProjectLanguageServerSettings` is a project-scope `@State` (`settings/ProjectLanguageServerSettings.java:30-33`), but the class is documented for debug port, suspend and trace only,
     and the code reads it for trace (`LanguageServerWrapper.java:1960`). **F** Server definitions live in application-scope `UserDefinedLanguageServerSettings.xml` (`launching/UserDefinedLanguageServerSettings.java:38-41`) and `configurationContent`
     for the launched server is read from `GlobalLanguageServerSettings` (`LanguageServersRegistry.java:232-245`, `UserDefinedStreamConnectionProvider.java:50-57`). So the committed `.idea/` cannot carry the rule toggles or the scoped server.
   - **Harper's own file-level controls.** **F** Harper config is passed by the LSP client as JSON under `harper-ls`; the docs list `excludePatterns` and comment ignores (`harper:ignore`, `spellcheck:ignore`) inside comment blocks of code files
     ([Harper LS](https://writewithharper.com/docs/integrations/language-server)). No repo config file is documented (searched the page for config file, `harper.toml`: none). Markdown comment-ignore support is **unverified**. The repo already uses `# harper:ignore` at `mise.no-env.toml:1`.
   - **Template import.** **F** LSP4IJ can export/import a server as a template directory (`template.json`, `settings.json`) via the UI ([UserDefinedLanguageServer.md#exporting-templates](https://github.com/redhat-developer/lsp4ij/blob/main/docs/UserDefinedLanguageServer.md)).
     A template committed in the repo is a manual import, not an automatic project config.
4. Recommendation: remove it from file-enforcer. It is per-user, host-specific, and not expressible as project config. Keep it as an opt-in user-level script or a committed LSP4IJ template plus a runbook line.

## file-enforcer.schema.json (new)

1. Contents and readers
   - **F** Planned: schemars-derived schema committed and kept current by a file-enforcer rule; config points at it via `"$schema"` (`doc/planning/file-enforcer-rust-rewrite.md:474-485`).
     **F** The two existing JSONC configs have no schema (`rg schema` in `cli-git.config.jsonc`, `monochromatic-lint.config.jsonc`: none). **F** The planning doc flags JetBrains `$schema` support in `.jsonc` as unverified; the JetBrains JSON page
     does not document an instance-level `$schema` property (searched page text). The verified project route is a mapping in `.idea/jsonSchemas.xml` (already tracked, with `relativePathToSchema` entries, e.g. mise at `https://mise.jdx.dev/schema/mise.json`).
2. Status: new, planned committed.
3. Replacements
   - **Subcommand prints the schema; no committed file.** **I** `file-enforcer schema > path` on demand; the IDE mapping needs a real path, so this only helps CLI/editor-agnostic use.
   - **Golden test in the tool's own crate.** **I** The Rust test asserts the committed schema equals `schemars::schema_for!` output; cargo test is the generator-consumer check, no file-enforcer rule needed.
   - **Drop the schema.** **I** Both sibling JSONC configs run without one; error messages from the decoder can carry the same guidance. Risk: the planning doc already notes a schemars schema can drift from a hand-written decoder.
4. Recommendation: drop the self-enforcing rule. Choose between no schema (matches the siblings) and a committed schema guarded by a golden test in the crate; both avoid the tool generating its own input.

## Summary ranking of effort and risk (**I**)

- Drop outright: `.factory/skills`, `.browserslistrc.resolved.local.json`, Harper LSP4IJ policy, `mpm.packages.toml`, forbidden-strings seed, schema self-rule.
- Replace by native mechanism: `CLAUDE.md` (import or symlink), `.claude/skills` (symlink), root `mise.toml` (conf.d), rules file (scanner multi-input), crate package licenses for Cargo (symlinks).
- Keep generating (no native replacement verified): npm package licenses (symlinks ship nothing) unless pack-time injection is built, `[workspace.dependencies]`, `toolchain.lock.json`.
- Other facts: `.codex/` exists and is empty; `.pi/settings.json` carries no packages; `.factory/` holds only `skills/`. The `mise run ...` processes seen in `ps` belong to other sessions, not this research.
- Unverified: GitHub licensee on a symlinked root `LICENSE` today; Droid skill discovery by live session; Markdown `harper:ignore`; IDE launch directories for cargo config.

All fixture dirs and clones remain under `/var/home/user/temp/agent/fe-rust/` and `/var/home/user/temp/agent/{cargo,mise,lsp4ij}-2026-10-29/`. No background process was left running.
