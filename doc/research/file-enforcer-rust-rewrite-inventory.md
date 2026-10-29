# file-enforcer rewrite: required outcomes, consumer needs, and known flaws

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 by a read-only research agent;
file reads and source citations only,
nothing was executed.

Repo-relative paths, rooted at `/var/home/user/Monochromatic`. Read-only research; nothing was run or edited.
Framing per the user's correction: the TypeScript tool is evidence of what this repo needs, not a design to port.
Sections 1 to 3 state needs as outcomes. Section 4 lists existing Rust. Section 5 lists flaws to avoid. The appendix
records current-implementation facts (sizes, flags) only so scope trade-offs can be argued with numbers.

## 0. Prior decisions that bear on this rewrite

- `doc/decision/monorepo-manager-all-rust.md` (Status, Decision) already accepted, 2026-09-16, an all-Rust single-file
  tool placeholder-named `meow` that absorbs file-enforcer, with OpenTofu-shaped HCL configuration ("Yes to HCL", 2026-09-17).
  The present request (clap plus JSONC) departs from the HCL part. The design ranked formats
  `HCL > TOML > TOML+CEL > JSONC > YAML` (`doc/planning/monorepo-manager-from-scratch-design.md:1603`).
- The user's earlier position (`:579-590`): the config language "does not need to be Turing-complete" and the old
  config's shape "reflects time constraints, not requirements". Same doc, `:1556-1585`, split the old config into 27
  units: 13 general features, 3 repository tasks, 5 plain data, 6 retired.
- Same decision: the root `mise.toml` stays generated until Mise is removed; meow does not own mpm's lifecycle.
- Question for the user: is this standalone `file-enforcer` or the `meow` file-enforcement part, and does JSONC replace the HCL decision?

## 1. Required outcomes: which files must end with which content

Each item: the file(s), what content they must have, from which inputs, what logic that takes, and whether a JSONC data
config can express it. "Overwrite" = the tool owns the whole file. "Splice" = the tool owns only some keys and must keep
hand-written comments and ordering. Source of each: `file-enforcer.config.ts` (root, 2,592 lines, the only config:
`git ls-files` finds one), line numbers in parentheses.

### 1.1 Verbatim or lightly templated copies (JSONC can express)

- `LICENSE` = byte copy of `LICENSES/LGPL-3.0-or-later.txt` (`:2515`).
- `CLAUDE.md` = fixed 49-line preamble (agent policy prose, `:2523-2571`) + `AGENTS.md` verbatim (`:2572`).
  Preamble belongs in a data file, not code. Rule: header text + file include.
- `.cache/forbidden-strings.rules.txt` = 7-line header + `forbidden-strings.append.txt` + `forbidden-strings.append.local.txt`
  concatenated, mode 0600 (`:896-916`).
- `forbidden-strings.append.local.txt` = comment-only seed created only if absent, mode 0600 (`:869-891`).
  Rule: create-if-absent with mode.
- Skill mirror: every `.agents/skills/*/*.md` (12 files in 10 dirs) copied byte-identically to `.claude/skills/` and
  `.factory/skills/`, with a per-destination `.agents-mirror-manifest.json` mapping canonical path to SHA-256, and stale
  mirrored files deleted only if the previous manifest owned them (`:1295-1358`, validation `:1019-1220`).
  Logic: directory mirror with ownership manifest, path-traversal checks, sorted keys. Expressible as one built-in
  "mirror" rule; the pruning safety logic is code, the rule is data.

### 1.2 Fan-out over packages (needs globs, parsing, per-item derivation)

- Per-package license texts: for each of 223 `package/*/*/package.json` and the two-level `package/*/*/Cargo.toml` manifests, read the license string,
  map it to required text ids by substring (LGPL-3.0-or-later implies GPL-3.0-or-later too, CC-BY-SA-4.0 separate),
  write `<pkg>/LICENSES/<id>.txt` copied from root `LICENSES/`, and delete known ids that are no longer required
  (`:116-119`, `:318-336`, `:549-581`). Logic: JSON field and TOML field readers, a small lookup table, fan-out,
  prune of tool-owned files. Expressible as data if the engine offers a field reader and prune.
- `package/config/pnpr/config.yaml` (215 lines): a YAML template with the sorted list of publishable workspace package
  names inserted. A package is publishable if: not under `package/test-fixture/`, its `package.json` has a string
  `version`, it exposes a loadable entry (after dropping `./ts` export subpaths, with `main`/`module` ignored whenever
  `exports` exists, `bin` string or object, entries that are TypeScript source do not count, using
  `isTypeScriptSourcePath` from `package/config/pnpr`), and it is not in a reviewed-exclusion map with reasons
  (`:2056-2499`). Also: every name must start with `@monochromatic-dev/` and use only npm name characters; a stale
  exclusion entry is an error. Logic: this is real code (nested `exports` map walk). Needs a Rust built-in, or a
  mini expression language. A JSONC data config cannot express it.
- Cargo manifests (26 crates, each its own workspace root): keys the tool owns inside hand-written `Cargo.toml`
  (`:1466-2054`). Outcome per crate: `edition = "2024"`; `license` fixed if the crate already has one; `publish = false`
  if present; six `[lints.clippy]` keys (`deny`/`allow`); `repository` fixed and `readme = "README.md"` if repository
  exists; `homepage` = GitHub tree base + crate directory; release/dev `[profile]` keys from a directory-to-preset table
  (10 directories, 6 presets); Dylint `check-cfg` for 2 directories. Whole blocks (`[lints.clippy]`, `[workspace]`,
  `[lints.rust.unexpected_cfgs]`) appended only when absent. Rule shape: "if table exists, set key to value" (guard) and
  "if path absent, append this text". Splice: comments, key order, unowned keys must survive. Logic: path template
  (homepage), lookup tables, guards. Expressible as JSONC data with a path templating and guard primitive.
- Rust dependency catalog: `rust-catalog.toml` (143 lines) is the one source of dependency versions; each crate's
  `[workspace.dependencies]` table is generated from the catalog entries the crate uses; use sites say
  `workspace = true`; literal versions left by `cargo add` are adopted into the catalog as `>=` floors (conflicts reported,
  named catalogs for exceptions); unused catalog entries pruned; internal path-dependency versions rewritten; a
  `default-features = false` quirk for Cargo before 1.99 (`doc/troubleshooting/cargo-1-98-inherited-default-features.md`);
  a repo without the catalog is left alone. Splice on `Cargo.toml`, read-write on the catalog. Design:
  `doc/decision/rust-dependency-catalog.md`. Logic: a multi-pass algorithm with global conflict detection. A built-in,
  not configurable data. All problems collected first, then written, so a failure writes nothing.

### 1.3 Outputs with environment or host dependence (most likely scope cuts)

- `mise.toml` = comment header + `mise.no-env.toml` verbatim + a fixed `[env]` text (80 lines) with `_.path` listing four
  fixed `.cache/*` dirs, `node_modules/.bin`, and every existing `package/*/node_modules/.bin`, sorted and deduped
  (`:691-781`). Depends on installed state, so the output differs per machine (issue #335, `:719`). Retires with Mise.
- Hand-authored `package/*/*/mise.toml` (246) and `mise.no-env.toml`: only normalize `[tasks."x"]` header quoting in place
  (`:2506`, globs `:680-685`). Retires with Mise.
- `.browserslistrc.resolved.local.json` = JSON array of browser targets resolved from `.browserslistrc` (production
  section) with config and stats discovery off (`:936-967`). Needs the browserslist query engine plus caniuse data.
  Consumer: `package/config/rolldown/src/browserslist-targets.ts`. No Rust equivalent in the repo.
- JetBrains Harper language-server policy written into the user's IDE state under
  `~/.config/JetBrains/<IntelliJIdea|IdeaIC><newest version>/options/` (two XML files with embedded JSON; disable three
  prose rules globally, exclude AGENTS.md and CLAUDE.md, add a second server scoped to those two files)
  (`:1396-1464`). Outside the repo, per-user, host-specific.
- `.cache/forbidden-strings.rules.txt` post-step: run the repo-built scanner `compile-rules` if its binary exists, skip with
  a warning if absent, fail the run on scanner failure; delete the retired `forbidden-strings.local.txt` (`:819-923`).
- Guard: `CONTEXT.md` must not exist at the root (`:593-611`, rule DPL). Rule: "path must be absent".

### 1.4 Toolchain artifacts (generated by a different part of the package)

- `toolchain.lock.json`, `mpm.packages.toml`, `mpm.managers.toml` are rendered from a typed plan (61 tools) kept as
  TypeScript data modules. Consumers need stable JSON for `jq` (see 2.3). These are outputs of an installer/updater concern
  (section 3), not of the config.

### 1.5 Verdict on a JSONC data config

- Expressible with a few engine primitives (file include, text template, glob, JSON/TOML field read, sort/dedupe, create-if-absent
  with mode, delete tool-owned stale files, guard-then-set TOML key, append block if absent, directory mirror): LICENSE,
  CLAUDE.md, rules file and seed, skill mirror, package licenses, Cargo manifest enforcement, mise.toml (if kept).
- Needs a Rust built-in whose parameters are data: Rust catalog sync, pnpr config generator, LSP4IJ policy, scanner post-step.
- Cannot be data and has no Rust equivalent: browserslist resolution.
- Nothing in the repo needs a Turing-complete config language. The only genuinely algorithmic items are the pnpr
  entry-point predicate and the catalog sync; both are better as named Rust features.

## 2. What CI and humans need

### 2.1 Regeneration

- Humans run `mise run sync:files` (alias `file-enforcer`, `mise.no-env.toml:1063`), or after dependency install via
  `prepare:pnpm` (`task.ts:354-361`, `mise.no-env.toml:997`), which runs `node file-enforcer.config.ts` directly.
  Dev loop: `watch:sync:files` (`mise.no-env.toml:1068`) re-runs on source change and warns when a generated file is edited.
- Needed: one command, one config path, deterministic output, exit non-zero on real failure.

### 2.2 Drift check

- Needed: "do committed generated files equal what the sources produce?" Today CI cannot use the tool's own `--check`;
  `.github/workflows/advisory-core.yml:92-105` runs `task-run prepare:pnpm` and then
  `git diff --exit-code -- rust-catalog.toml 'package/**/Cargo.toml'`, with a comment that `--check` after a sync "could never fail".
  The need: a check that does not require a prior sync, covers every output, and has a stable message with the fix command.
- `current-baseline.yml:23` and `advisory-core.yml:91` call `task-run prepare:pnpm`; `pnpr-publish.yml:15` points maintainers
  at the exclusion map.

### 2.3 Consumers of generated content (contracts a rewrite must keep or consciously change)

- `toolchain.lock.json`: read with `jq` by `Dockerfile:24-49`, `package/test-harness/core-matrix/src/runtime.ts:13-36`, and Containerfiles
  (`package/music-player/desktop-app/Containerfile:14,65,94`, `package/terminal/app/Containerfile*`,
  `package/nested-wayland-session/cli/Containerfile`, `package/mutation-test/cli/runtime/Containerfile`,
  `package/forbidden-strings/core/mutants.Containerfile`); parsed by
  `package/dependency-catalog/cli-rust-update/src/toolchain.ts:28` and `package/translation-repair/typography/src/semantic-wrap-binary*.ts`.
  `Dockerfile:49` also bootstraps via `node package/file-enforcer/core/src/toolchain/sync.ts` (a Rust replacement must still install Node and pnpm).
- `bin-paths` JSON (`exposureDir`, `pathDirs`, `bins`): run by `package/ide/app/src/language/toolchain.rs:15,65` and copied by
  `package/ide/app/bin/inspect-language-toolchain.ts:41-86`.
- `.cache/forbidden-strings.rules.txt`: path baked into generated `mise.toml` `FORBIDDEN_STRINGS_RULES` and `cli-git.config.jsonc:17`.
- `package/config/pnpr/config.yaml`: `package/config/pnpr/Containerfile`, `src/publish-plan.ts`, `src/publish-missing-versions.ts`.
- `.browserslistrc.resolved.local.json`: `package/config/rolldown/src/browserslist-targets.ts`.
- TypeScript importers of the package: root config; `package/vm-builder/cli/src/{import,build-and-import}.ts` (only `exec`, 7 call
  lines each, `package.json:16`); `package/test-fixture/file-enforcer-perf` (benchmark of the old tool; retires). Nothing else
  imports it; `ensurePackage`, `startWatching`, `editTomlKey`, `evaluatePredicate` have no consumer outside the package
  (`rg` over `*.ts`, `*.rs`, `*.md` excluding `package/file-enforcer/` and `doc/`).
- Git hooks: `.git/hooks/{post-checkout,post-commit,post-merge,pre-push}` do not mention file-enforcer; no husky or lefthook.

## 3. Duties in the package that are not file enforcement

Separate concerns that were bundled by accident (evidence in section 5):
- Toolchain installer, status, updater (`toolchain/`): installs 61 tools over 7 routes, probes versions, queries
  GitHub/npm/PyPI/crates.io/Adoptium/conda/go.dev/nodejs.org/ziglang.org/Android, rewrites its own TypeScript data modules.
  Its file-generation slice is about 800 lines (lock, mpm manifests, plan rendering). Entry tasks `toolchain:sync|status|update`
  plus `:global` (`mise.no-env.toml:540-562`, `task.ts:335-347`). This is the repo bootstrap, so it must exist somewhere, not necessarily here.
- OS package installer (`ensurePackage`) with a 22.6k-line Repology-derived index and a PostgreSQL container pipeline
  (`generator/`, 282 lines): zero consumers.
- General process runner `exec` plus platform predicate: one consumer (vm-builder), already planned to replace it with a private runner
  (`doc/decision/monorepo-manager-all-rust.md`, Consequences).
- Rust catalog update, trust and verify are already separate TypeScript tools: `package/dependency-catalog/cli-rust-update`
  (`rust:update`, `rust:verify`), `cli-tighten` (`catalog:tighten`), `core-rust` (parser). The file-enforcer part is only rendering.
- Watch mode: a foreground re-run loop with desktop notification, not a daemon; `package/watch-restart/cli` exists separately.
- Test infrastructure living in `src/`: podman matrix runner, fuzz budget fixture, mutation runs.

## 4. Existing Rust that overlaps

- No first-party crate uses `toml_edit`, `jsonc-parser`, `hcl-*`, `kdl` (searched every `Cargo.toml` under `package/` and `rust-catalog.toml`: no hits).
  `toml_edit` is not in `rust-catalog.toml`. The TOML splice engine is TypeScript (`package/toml-edit/core`, about 9.3k lines).
- `monochromatic-jsonc-edit` (`package/jsonc-edit/core-rust`, 0.1.1): zero dependencies (`Cargo.toml:41`), comment-preserving
  parse, edit, canonical emit; used by `package/git-policy/cli` (`Cargo.toml:53`), `package/linter/monochromatic-lint` (`:59`), two fuzz crates.
  Two JSONC configs already exist and are read by Rust tools: `cli-git.config.jsonc`, `monochromatic-lint.config.jsonc`.
  Design doc: `jsonc-parser` 0.33.2 behind a wrapper passed 16 of 16 conformance cases (`from-scratch-design.md:2504-2518`);
  caveats in `doc/troubleshooting/jsonc-parser-json5-defaults.md`.
- Catalog floors in `rust-catalog.toml:38-106`: `clap >=4`, `serde >=1`, `serde_json >=1`, `toml >=1.1`, `anyhow >=1`,
  `ignore >=0.4`, `globset >=0.4`, `notify >=8.2.0`, `notify-debouncer-mini >=0.7`, `rayon >=1`, `sha2 >=0.10`, `tokio >=1.50`, `twox-hash >=2.1.4`.
- `clap` derive in `package/forbidden-strings/cli`, `ide/app`, `linter/monochromatic-lint`, `music-player/desktop-app`,
  `nested-wayland-session/cli`; plain `clap` in `mpm-shadow/cli`, `search-shadow/cli`. `toml` only in `ide/app/Cargo.toml:70`.
- No Rust code parses `rust-catalog.toml` or edits `Cargo.toml`; both parsers are TypeScript
  (`package/dependency-catalog/core-rust`, `package/file-enforcer/core/src/cargo/catalog`).
- Bootstrap hazard: generated `Cargo.toml` content (lints, `[workspace]`, `[workspace.dependencies]`) would apply to the new crate itself, so the old
  tool must keep producing manifests until the new one lands (`doc/planning/monorepo-manager-implementation-plan.md:186-187`).

## 5. Known flaws / do not repeat

Each flaw has evidence in the repo.

1. Incremental cache that is unsound and does nothing.
   `doc/troubleshooting/file-enforcer-staleness.md`: Defect 2, a skipped builder misses inputs read outside the tracked API and keeps stale output with exit 0;
   Defect 1, the root config has no lazy builder, all fifteen writes are eager, so the manifest never records and wall time is the same
   (0.929 s, 0.924 s, 0.926 s, and 0.920 s after an input edit); the manifest held 117 entries, all from test runs under `/tmp`; one absent watched path
   (`CONTEXT.md`) aborts every recording; recorded dependency sets are both too small and too large. It still costs about 3.2k non-test lines plus about
   2.5k test lines (io staleness tests 890, root `staleness-*` 1,585). `TODO.md` "Config execution model" says warm and cold cost the same.
   Lesson: an input set must come from the filesystem or the declared rule, never from cooperative tracking; do not add a cache until a measured need.
2. Fragile cross-process lock and manifest file.
   `io/staleness-manifest-lock.ts`, `-lock-owner.ts`, `-lock-owner-publish.ts`, `-lock-recovery.ts`, `-temp-orphan.ts` (about 1k lines), commit
   `fcdf928c72` "remove manifest temp files orphaned by dead writers", `35386af393` "let write callers isolate the staleness manifest" (tests polluted the real
   manifest because the default path walks up to the nearest `node_modules`). A tool that rewrites files in one pass needs no shared mutable state on disk.
3. Scope creep into an unrelated installer, with boundary churn.
   `toolchain/` is 15,689 of about 33.6k non-test lines excluding the generated package index (47 percent); `package/` plus `data/` add an OS package installer nobody calls and a 22,607-line
   generated index (`data/packages.generated.ts`) built by a PostgreSQL container pipeline. Duties already moved out once: commit `c7f312c91e`
   "the Rust mpm shadow owns the stale-crate prune, so sync stops pruning" removed, the same day (2026-10-29), what `2565250e26` had just added. README still documents `exec` forms the
   implementation rejects (`doc/handover/monorepo-manager.md:436`). Keep file enforcement separate from installing software.
4. Config is executable TypeScript with global mutable state.
   `tracker.ts` module-global read/write/glob sets, `AsyncLocalStorage` capture (`sourceCaptureStorage`), config imported via top-level await (`cli.ts:62`).
   Consequences: the config reads inputs through plain `node:fs` in six places (`glob`, `lstat`, `readFile`, `nanoSpawn`) that the tracker cannot see
   (staleness doc "Cause"); the 10-way `Promise.all` at `:2513` races on shared state; hidden ordering via a global read cache
   (`:2048` "runs after the manifest engine so it reads that step's output through the cache"); lint and type burden
   (`doc/troubleshooting/file-enforcer-watch-effect-boundaries.md`, task `lint:config:types` with `tsconfig.repository.json`).
   Static data would make the input set and output set known without running code.
5. Two entry points with different behavior.
   `node file-enforcer.config.ts` (used by `prepare:pnpm`, `mise.no-env.toml:997`, `task.ts:360`) skips `cli.ts`, so no `--check`, `--watch`, or active-config path.
   `cli.ts:44-47` silently drops any unknown `--flag`. No `--config`, `--verbose`, `--quiet`, `--dry-run` (`TODO.md` "CLI"). CI therefore
   re-implements drift detection with `git diff` (`advisory-core.yml:92-105`). One CLI with explicit flags and a real `--check`.
6. Non-deterministic or machine-dependent generated files.
   `mise.toml` `_.path` comes from globbing installed `node_modules` (`:722-745`, issue #335); an unsorted glob moved entries across filesystems
   (`doc/troubleshooting/node-fs-glob-mise-path-order.md`, fixed by sorting). Generated content must depend only on committed inputs.
7. A sync run with side effects beyond files in the repo.
   Spawns the scanner and fails the whole run if it fails (`:819-839`), edits JetBrains settings in the user's home (`:1396`), loads a JS library at
   runtime (`:637`), chmods files. These need to be separate, opt-in steps with separate exit codes.
8. Three TOML engines, one XML engine, hand-rolled glob syntax.
   `toml-edit-core` (TS splice), `toml-eslint-parser` (catalog locate/splice, `cargo/catalog/locate.ts`), a text normalizer for mise headers
   (`mise/manage-mise-manifests.ts`); `tiny-readdir-glob` in builders versus `node:fs` `glob` in the config; own mirror-glob syntax
   (`io/glob-mirror.ts`); Bun glob dotfile bug forced a custom `expandGlob` (`doc/troubleshooting/bun-fs-glob-dotfiles.md:167-170`).
   Pick one TOML editor and one glob engine.
9. Bloat in public surface and tests.
   `src/index.ts` exports about 190 names, many `@internal` solely so built-artifact tests can reach them; the config uses about 12 names.
   README claims "All production source files are under 100 lines" (`README.md:496`) while 22 of 146 non-test source files exceed 400 lines.
   About 19k test lines plus fuzz and container-mutation suites; the mutation job failed with a package-path `ENOENT`
   (`doc/troubleshooting/depot-toml-mutation-completion.md:106-107`).
10. Policy prose embedded in code: the 49-line CLAUDE.md preamble and 80-line mise `[env]` block live as template literals in the config (`:2523-2571`, `:695-774`).
    Keep content in content files; config names the rule.

## 6. Open questions for the user (scope)

- Keep or drop: the Mise outputs (1.3), browserslist output, LSP4IJ Harper policy, scanner post-step, root `LICENSE` and package licenses.
- Where do toolchain sync/status/update and `bin-paths` live (separate Rust tool, `meow`, or stay TypeScript)?
- Is the OS package installer and Repology index dropped outright (no consumer)?
- Rust catalog sync: port as a built-in, or leave in TypeScript for now?
- Must the new tool also edit the old TypeScript tool's outputs byte-identically, or may output formatting change (design doc lists "Byte-identical output" as an open topic, `:1300`)?
- JSONC or HCL (decision conflict), and the schema/validation story for the JSONC file (comments are allowed; jsonc-edit crate exists).

## Appendix: current implementation facts (reference only)

- `package/file-enforcer/core/src` non-test lines: toolchain 15,689 (of which plan data 1,723, container runner 673); data 22,732 (22,607 generated);
  io 5,307 (staleness 3,200); cargo 4,384 (catalog 3,821); package 1,820; pipeline 1,520; root files 1,174; watch 2,194; jetbrains 1,270; platform 105; mise 90.
  Tests and fixtures about 19,004 lines. `generator/`: Containerfile 105, entrypoint.sh 74, extract.sql 80, repology.conf 23.
- CLI: `cli.ts` accepts `--watch`, `--check`, one positional config path. `toolchain/{sync,status,update,generate}.ts` accept `--global`; `update.ts` also `--check`.
- Root config call-site counts: `overwrite` 8, `overwriteEach` 1, `overwriteIfNotExists` 1, `cat` 10, `getTomlProperty` 1, `addWatchedPaths` 1, one call each of
  `manageCargoManifests`, `manageRustCatalog`, `manageMiseManifests`, `manageLsp4ijServerSettings`.
- Dependencies: `chokidar`, `dot-prop`, `nano-spawn`, `tiny-readdir-glob`, `toml-eslint-parser`, `@lezer/xml`, plus workspace `toml-edit-core`,
  `dependency-catalog-core-rust`. 55 commits touch `package/file-enforcer/core` in the available history.
- Prior sizing in the design doc: "49 modules and 3,547 code lines" to port plus 56 test files with 11,126 lines (`from-scratch-design.md:1275-1284`);
  that count predates the toolchain growth recorded here.
