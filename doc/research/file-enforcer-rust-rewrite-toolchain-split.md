# Toolchain split: Rust file-enforcer and the mpm wrapper

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 by a read-only research agent;
file reads, `jq`, `rg` and `ls` only,
nothing was built or installed.

Read-only research, 2026-10-29. Paths are repo-relative unless absolute. "Fact" = read or measured; "Inference" = mine.
The working tree had an uncommitted one-line change in `plan.artifacts.data.ts` and I read that version (concurrent work, untouched).

## 1. What mpm-shadow does today

Crate `monochromatic-mpm-shadow` 0.1.1 (`package/mpm-shadow/cli/Cargo.toml:2`), binary `mpm-shadow`, 1,966 lines including tests.
Its only dependency is `clap` (`Cargo.toml:53`), builder API.

### Two modes chosen by argv[0]

- Fact: `src/main.rs:44-68`. Started under the file name `mpm` it runs `run_shim` (`src/shim.rs:145`); any other name runs `run_cli` (`src/cli.rs`).
- Shadow mode: finds the real `mpm` on PATH skipping itself (`src/real_binary.rs:65`, `src/shim.rs:109`), runs it as a child with arguments
  and streams untouched (`src/shim.rs:78`), then prunes (`src/shim.rs:127`), exits with mpm's status (`src/shim.rs:58`).
  Exit codes 126 (found, not startable), 127 (none on PATH), 2 (bad `mpm-shadow` command line) (`src/constants.rs:31-46`).
- Command mode: `mpm-shadow prune`, `--help`, `--version` only (`src/cli.rs:46-111`).
  `subcommand_required(true)` so a bare `mpm-shadow` prints help.
- Fact: in shadow mode every argument goes to real mpm byte for byte (`README.md:12-14`). New verbs therefore can only live under the
  `mpm-shadow` name; `mpm sync` would reach real mpm (inference from that contract).

### What it reads and writes

- Reads: `PATH`, `CARGO_HOME`/`HOME` (`src/cargo_home.rs:33`), cargo's install record `<cargo home>/.crates.toml` (`src/installed.rs:38-51`),
  the registry `registry/cache/<index>/*.crate` and `registry/src/<index>/*` (`src/registry.rs:118`).
- Writes: deletes stale `monochromatic-*` crate archives and extracted trees (`src/prune.rs:108-117`). Nothing else.
- Fact: it does not read `toolchain.lock.json`, `mpm.packages.toml` or `mpm.managers.toml`. It has no network code, no TOML or JSON parser, no
  plan awareness. The plan reaches it only as a plan entry (`plan.tools.data.ts:388-407`, route `cargo-binstall`, postinstall link `mpm-shadow` as
  `mpm` in `.cache/mpm-shadow-bin`, Linux only).

### The boundary in `doc/decision/mpm-toolchain-adoption.md`

- mpm owns (`:92-112`): `mpm.packages.toml` restore manifest "generated from the typed tool plan", inventory (`mpm installed`), outdated
  (`mpm outdated`), upgrade, release-age cooldown, SBOM; installs through its `pnpm`, `uv`, `rustup` managers; `bin` manager inventories standalone tools.
- Repository providers own (`:114-131`): `cargo-binstall` installs (features, OS filters, postinstall link), `rustup component/target add`,
  Android SDK, the `archive` fallback with SHA-256. "Invoked by the toolchain sync command from the same typed tool plan."
- file-enforcer owns (`:201-223`): "the typed toolchain policy ... in `package/file-enforcer/core/src/toolchain/`", generated committed
  `toolchain.lock.json` and `mpm.packages.toml`, committed `mpm.managers.toml` (but the renderer generates it, `generate.ts:61-62`).
- The shadow owns (`:282-350`): only the prune rule for repository-owned crates after every mpm run.
- Machine-global scope (`:254-280`): plan, lock and manifests live in `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`; payloads in
  `${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain-global`.
- Inference: the decision assigns policy plus install to the TypeScript package and does not say where sync lives once that package is Rust.
  `doc/planning/file-enforcer-rust-rewrite.md:133-143` records the user rejecting every single-owner option and the split question as open.
- Fact, not what the decision implies: the TS sync almost never calls mpm. The only `mpm` invocation in `src/toolchain` is
  `mpm --rustup restore mpm.packages.toml` (`providers.rust-android.ts:72-83`), with a direct `rustup toolchain install` fallback. Every other route
  runs its own provider (`sync.ts:92-103`, `providers.cargo.ts:36-98` (args at `:50`) runs `cargo binstall` directly). `status.observe.ts` never calls `mpm installed`.

## 2. Responsibilities in `package/file-enforcer/core/src/toolchain`

Total 15,911 lines excluding `*.test.ts` (includes one 42-line `.md`). Line counts below are per file group and sum to that total.
Fact: every entry point reads `ALL_TOOLS` (the TS plan, `plan.ts:12-19`) or `loadPlanFromDir` for `--global`, never `toolchain.lock.json`.
`sync.ts:274`, `status.ts`, `bin-paths.ts` and `update.ts` import `./plan.ts`; the only use of `paths.lockFile` is as a write target (`generate.ts:225`,
`rg lockFile` over `src`). So the lock is write-only to this package; the docs' "sync converges to the lock" is true only because the lock is rendered from the plan the sync reads.

### R1. Plan data and schema: 3,197 lines. Class (a) input, plus (d)

- `plan.tools.data.ts` 1,040 (63 entries with rationale comments), `plan.artifacts.data.ts` 683 (32 tools carry artifact rows), `types.ts` 267, `plan.ts` 130.
- `plan.load.ts` 305, `plan.load.guards.ts` 424, `plan.load.guards.options.ts` 348: dynamic `import()` of a scope's TS data modules plus hand
  validation (`plan.load.ts:216`). Exists only for the machine-global scope, whose plan is TS data in `~/.config/monochromatic-dev/toolchain/`
  (fact: those two `.data.ts` files plus lock and manifests are what is there; global lock has 59 entries, includes `cargo-git` and `go-install` routes).
- (d) Inference: `plan.load*` (1,077 lines) is a TypeScript-specific schema check; with data in JSON(C) read by serde it has no counterpart.
- Straddle: the same type is edited by hand (declaration fields) and rewritten by machine (`version`, `artifacts`).

### R2. Lock and mpm manifest rendering: 779 lines. Class (a)

- `lock.ts` 102, `generate.ts` 319, `mpm-manifest.ts` 358. Outputs: `toolchain.lock.json` (1,652 lines), `mpm.packages.toml` (45), `mpm.managers.toml` (205).
- Fact: the lock record is the plan entry field for field (`lockToolFromPlanEntry` `lock.ts:27` copies request, version, route, manager, packageId, bins, os, env, options,
  artifacts; the only additions are `schemaVersion: 1` and `generatedFrom`, `toolchainLockFromTools` `lock.ts:57`). The render is a projection, not a transformation.
- `mpm.packages.toml`: group tools by `manager`, sorted section names, `packageId = "<version>"` (`mpm-manifest.ts:150-193`). Pure function of the lock.
- `mpm.managers.toml`: one `[mpm.overrides.cargo-binstall-<slug>]` per `cargo-binstall` route entry (18 now), baking `--version` into `binstall`
  arguments (`mpm-manifest.ts:306-358`). Pure function of the lock.
- (d) duplication, fact: `installArgs` (`mpm-manifest.ts`) and the argument builder in `providers.cargo.ts:36-98` (args at `:50`) render the same `cargo binstall --version ... --features ...`
  command twice. Only consumer of `mpm.managers.toml` outside docs: the documented `mpm --config mpm.managers.toml --cargo-binstall-monochromatic-mpm-shadow install`
  (`doc/decision/mpm-toolchain-adoption.md:349`); `rg` finds no repo code that passes `--config`.
- Fact: `mpm.packages.toml` has one code consumer, the rustup restore at `providers.rust-android.ts:72-83`; the `[cargo]`, `[pnpm]`, `[uvx]`, `[conda]` sections are
  read by no repo code (humans running `mpm restore`).

### R3. Host primitives and path algebra: 1,602 lines. Class (b)

- `paths.ts` 623 (repo scope and `globalToolchainPaths`, `paths.ts:198`, `:284`), `layout.ts` 517 (per-route bin search, `hostPlatformId` `:38`), `fs-probe.ts` 55,
  `exec.ts` 407 (`run`, `downloadVerified`, `extractArchive`, `atomicSymlink`).
- Trust checks: not found. Fact: no code in the directory verifies ownership or permission bits of the plan, the manifests or the config dir. `chmod` appears only for exec bits
  (`exec.ts:388-401`, mode 0o755) and the container readability heal (`container-runner.ts:621-654`). The `owner-writable only (MPM's trust gate)` text is a generated header
  comment (`mpm-manifest.ts:295,316`); `generate.ts` writes with default mode and does not enforce it. The trust is mpm's own gate on `mpm.managers.toml` (per that header).
  Inference: `loadPlanFromDir` runs the user's global `plan.tools.data.ts` as code (`plan.load.ts:216`, dynamic `import()`), so that file's `0600` mode is the only trust control.
  With a data file instead, that code-execution surface disappears; the Rust writer of `mpm.managers.toml` should create it with a restrictive mode.
- Dead code candidates (d): `clearExposure` (`exposure.ts:453`) and `symlinkOrCopy` (`exec.ts:354`) have no non-test caller (rg over `src`, excluding `*.test.ts`).

### R4. Install and sync: 3,189 lines. Class (b)

- `sync.ts` 464, `sync.order.ts` 115 (staging `bin, uv, mpm, rustup, rust, cargo-binstall, go, node, pnpm`, `sync.order.ts:18`), `providers.*` 2,610:
  archive 421, bin 509, cargo 200, manager (go/pnpm/uvx) 355, rust-android 377, environment 217 (GitHub token, `bin` CLI path), context 111, parse 374, barrel 46.
- 10 declared routes (`types.ts:36`, values listed there); 7 used by the repo lock (bin-sourced 20, cargo-binstall 18, mpm-pnpm 11, archive 10, mpm-uvx 2, android-sdk 1, rustup-toolchain 1;
  measured with `jq` on `toolchain.lock.json`). Global lock uses also `cargo-git` 1 and `go-install` 1.
- Contract: lock-constrained, idempotent per tool, probes first (`sync.ts:164`), warns when a shared install would be replaced (`sync.ts:126`).
- Touches the host and network: `bin install`, `cargo binstall`, `rustup`, `uv tool install` (`providers.manager.ts:305,333`), `pnpm add --global` (`providers.manager.ts:194-195`), `go install`, downloads with SHA-256.

### R5. Status and observe: 1,044 lines. Class (b), partly (d)

- `status.ts` 376, `status.observe.ts` 472 (one probe per route, `ROUTE_OBSERVERS` `:459`), `status.parse.ts` 196 (`uv tool list`, `pnpm list --global` parsers).
- Inference: this reimplements machine inventory that the decision gives to `mpm installed` (`mpm-toolchain-adoption.md:96`). mpm's `bin` manager inventories absolute paths
  and not locked versions (`doc/planning/mpm-toolchain-migration.md` shape notes), so it may not cover `bin-sourced`; unverified here.

### R6. Exposure, env file, bin-paths: 836 lines. Class (b), with (d)

- `exposure.ts` 461: rebuild `.cache/toolchain-bin` symlink farm (`:66`), postinstall links (`:201`, e.g. `git` into `.cache/cli-git-bin`, `mpm` into `.cache/mpm-shadow-bin`),
  postinstall commands (`:294`, `search-shadow link .cache/search-shadow-bin`). Straddle: links live under the worktree, but the global scope links into `$HOME/.local/bin`.
- `exposure.env.ts` 143: writes gitignored `.cache/toolchain.env.json` (`ANDROID_HOME`, `JAVA_HOME`, `RUSTUP_TOOLCHAIN`). (d) Fact: `readToolEnvFile` (`:128`) has no caller
  and `rg` finds no consumer of `toolchain.env.json` outside `src/toolchain` and docs. Written, read by nothing in the repo.
- `bin-paths.ts` 232: prints `{exposureDir, pathDirs, bins}` JSON (`:88`, `:197`) from the plan plus what exists on disk. Needs only Node builtins by design
  (`bin-paths.lint-exemption.md`, 42 lines).

### R7. Update: 4,243 lines. Class (b) and straddle

- Orchestration 1,342: `update.ts` 190, `update.resolve.ts` 301, `update.apply.ts` 253, `update.render.ts` 464, `update.report.ts` 134.
- Resolvers 2,901 (network): github 457, rust 403, shared 366, github-release 272, android 272, go 265, registry (npm/PyPI/crates.io) 204, node 199, conda 191, adoptium 115, zig 103, barrel 54.
  Sources: api.github.com, registry.npmjs.org, PyPI, crates.io, nodejs.org, go.dev, static.rust-lang.org, ziglang.org, api.adoptium.net, conda, dl.google.com.
- Straddle, fact: update does network resolution then rewrites committed files in two steps. It regenerates the text of `plan.artifacts.data.ts` and patches `version`
  literals in `plan.tools.data.ts` by text substitution (`update.apply.ts:148-250`, `update.render.ts:59`, `:374`), then re-renders the lock and manifests through
  `writeArtifacts` (`update.apply.ts` end). `update.render.ts` (464) exists only because the plan is TypeScript source; it disappears with a JSON plan.
- `--check` reports without writing and exits non-zero on movement (`update.ts:64-190`).

### R8. Container test runner: 979 lines. Class (c)

- `container-runner.ts` 673 (bounded `podman run --memory=2g --cpus=2`, capture cap 32 MiB, per-cell timeout, readability heal), `mise.container-test.ts` 173, barrel `index.ts` 133.
- Fact: `container-runner.ts` is shared with `src/package/mise.container-test.ts:37` (the `ensurePackage` matrix) and has no toolchain logic.
  `package/test-harness/core-matrix/src/container.ts:398` already carries `--memory=2g`, so a bounded runner partly exists there (overlap not compared in detail).

### Other (d) duplication and obsolescence

- `toolPlanInstallableNames` (`plan.ts:97`) only feeds `src/package/mise.generate-index.ts:236`, the `ensurePackage` index; the research inventory
  lists `ensurePackage` as zero-consumer (`doc/research/file-enforcer-rust-rewrite-inventory.md:153-160`).
- Mise task wrappers exist twice: `mise.no-env.toml:540-562` (with `:global` variants) and `task.ts:335-347` (no `:global` variants), plus the generated `mise.toml:542-562`.
- `Containerfile`s hardcode a copy of `.tools.rust.version` (`package/terminal/app/Containerfile:53-58` says "kept in sync with that key"): a manual duplicate of the lock.

## 3. Consumers and contracts

### `toolchain.lock.json` (strict JSON, read with jq, TS and Rust)

- jq, hard cost to change shape:
  - `Dockerfile:24-45` (`.tools.node.artifacts["linux-x64"].url/.sha256`, same for `pnpm`).
  - `package/mutation-test/cli/runtime/Containerfile:24-34` (node and pnpm artifacts).
  - `package/test-harness/core-matrix/src/runtime.ts:24-25` (`.tools.node.artifacts[$p]`, platform id from the key set).
  - `.github/actions/task-cli-setup/action.yml:42,46` (`.tools.node.version`, `.tools.pnpm.version`).
  - `.github/workflows/music-player-release.yml` (30 hits: `.tools.rust.version`, `.tools."cargo-binstall".version`, `.tools."cargo:cargo-about".version`,
    `.tools."android-sdk".options.androidPackages`, `.tools.java.artifacts[$platform]`), `kotlin-linter-publish.yml` (5), `fs-id.yml` (1).
  - `package/music-player/desktop-app/Containerfile.windows-gnu:48` quotes a `jq` read of `.tools.rust.version`.
- TypeScript parsers: `package/dependency-catalog/cli-rust-update/src/toolchain.ts:41-62` (`.tools.rust.version`), `package/ide/app/bin/inspect-language-toolchain.ts:46,87`
  (copies the lock into a fixture), the lock test `package/translation-repair/typography/src/semantic-wrap-binary.unit.test.ts:269-285`
  (pins `cargo:monochromatic-lint` version, `semantic-wrap-binary-constants.ts:14-23`).
- Prose-only references (no parse): `terminal/app/Containerfile*`, `nested-wayland-session/cli/Containerfile`, `forbidden-strings/core/mutants.Containerfile`,
  `music-player/desktop-app/Containerfile` (the build contexts cannot reach the lock; they hardcode `ARG RUST_TOOLCHAIN=nightly-2026-10-29`).
- Keys relied on: `tools.<id>.version`, `.artifacts.<platform>.{url,sha256}`, `.options.androidPackages`. The keys `request`, `route`, `bins`, `env`, `manager`, `packageId`, `os` have no external reader.
- Cost rule (inference): the tool-id keys (`node`, `pnpm`, `rust`, `cargo-binstall`, `cargo:cargo-about`, `java`, `android-sdk`, `cargo:monochromatic-lint`) and
  `version`/`artifacts`/`options.androidPackages` are the frozen surface; everything else can change freely. JSONC would break every jq reader (jq rejects comments).

### `mpm.packages.toml` and `mpm.managers.toml`

- Read only by mpm. `mpm --rustup restore mpm.packages.toml` (`providers.rust-android.ts:72-83`) and the documented shadow install via `--config mpm.managers.toml`.
- Format is mpm's own TOML dump / override schema (mpm 8.2.0). Changing the content is free; changing the format needs an mpm upgrade.

### Global copies (`~/.config/monochromatic-dev/toolchain/`)

- Fact on this machine: holds `plan.tools.data.ts`, `plan.artifacts.data.ts`, `toolchain.lock.json`, `mpm.packages.toml`, `mpm.managers.toml` (listing only).
- Consumers: `sync.ts --global`, `status.ts --global`, `update.ts --global`, `generate.ts --global` (`loadPlanFromDir`), mise tasks `toolchain:*:global` (`mise.no-env.toml:552-562`).
  Payloads: `~/.local/share/monochromatic-dev/toolchain-global`, farm on PATH via the global Mise config `_.path` (outside the repo).
  Moving the plan format changes a hand-edited, `0600` file with comments the user owns (`plan.tools.data.ts` there is `-rw-------`).

### `bin-paths` JSON and the exposure directory

- Consumer: `package/ide/app/src/language/toolchain.rs:15,65` (`ENTRY_POINT = "package/file-enforcer/core/src/toolchain/bin-paths.ts"`), run as `node bin-paths.ts`
  in the language-server sandbox: no network, project read-only, cleared environment, 10 s limit (`toolchain.rs:67-69`, `toolchain/command.rs:1-13`).
  Parser `toolchain/parse.rs:30-90` requires `exposureDir` (relative), `pathDirs` array, `bins` object. Contract text is pinned in `doc/planning/mpm-toolchain-migration.md`.
- Consumers of the directory itself: `file-enforcer.config.ts:725-741` generates the `mise.toml` `_.path` list `.cache/search-shadow-bin`, `.cache/mpm-shadow-bin`,
  `.cache/toolchain-bin`, `.cache/cli-git-bin`, then `node_modules/.bin`. `package/translation-repair/typography/src/semantic-wrap-binary.ts:38-50` finds `monochromatic-lint` by PATH.
  Error messages in `ide/app/bin/nested-compositor.ts:66,75`, `semantic-wrap-binary-error.ts:14-17` tell users to run `mise run toolchain:sync`.

### Mise tasks

- `toolchain:sync|status|update` plus `:global`, `bootstrap`, `prepare:android` aliases (`mise.no-env.toml:540-562,950-955,1025`; `task.ts:335-352`). All run `node .../toolchain/<x>.ts`.
  `prepare` fans out `prepare:android` (`mise.no-env.toml:958`). Repointing them is a one-line change each; the argument grammar is `--global` and `--check`.

## 4. Where the plan data lives

### Key fact first

- The lock is write-only to the package (`paths.lockFile` is used only at `generate.ts:225`; see section 2 intro). Sync, status, update and bin-paths read the TS plan. So "is the render needed" reduces to
  "does anyone need a second copy of the plan": today the lock is that copy, consumed only by jq, TS and Rust readers outside the package (section 3).

### Fields the plan has (all from `types.ts`)

- Declaration fields (hand-written intent): `id`, `request` (`{track:'latest'}` 55, `{pinned}` 6, `{channel}` 2), `route` (10 values), `manager`, `packageId`, `bins[]`,
  `os[]` (4 entries have it), `env{}` (3 entries), `options{}` (`rustComponents`, `rustTargets`, `cargoFeatures`, `cargoGit{url}`, `assetPattern`, `allowBuilds`,
  `androidPackages`, `postinstall` link or run, `binAliases`, `bootstrap{requiresTool, distribution}`).
- Resolution fields (machine-written by update): `version`, `artifacts{platformId:{url,sha256}}` (10 platform ids seen; Linux only for most tools by
  `doc/decision/mise-lock-linux-only-stopgap.md`).
- Fact: `ToolchainLockTool` equals `ToolPlanEntry` minus `id` (`ToolPlanEntry` `types.ts:196`, `ToolchainLockTool` `types.ts:236`), so the lock already carries every declaration field. Resolvers and providers use only fields the lock has.

### Can TypeScript data modules be replaced?

- Yes (inference): the plan has no logic, only data and comments. The two `.data.ts` files are already import-free by lint exemption design; the global scope proves
  the loader needs only exported literals. The cost is a one-time conversion of 63 entries and the global file; the comments (rationale per pin) need a home.

### Is the plan-to-lock render needed?

- Option A, lock is the only data file. Declaration fields are hand-edited in the lock, `version`/`artifacts` are written by update, and an optional `note` string per tool keeps
  rationale (JSON has no comments). The render step disappears; file-enforcer keeps only lock to TOML projections plus a lock schema check. Cost: the pnpm model's
  intent/resolution split is by convention, not by file; every hand edit touches a 1,652-line generated-looking file.
- Option B, declarations in JSONC (comments, ids in order, no version/artifacts) plus the lock as resolved output. The lock must still carry the declaration fields, because jq readers use
  `options.androidPackages` (`music-player-release.yml`) and `route`/`bins` serve sync. So B has a real render-and-compare: a file-enforcer rule kind that projects declaration fields into the
  lock and fails `check` on drift (the `pnpm install --frozen-lockfile` model), while mpm-shadow `update` writes only `version` and `artifacts`. Hazard: two writers of one file (file-enforcer for
  declaration fields, update for resolution fields); both must preserve the other's fields and key order. Cost: two files, and the lock must stay strict JSON for jq.
- Option C, keep TS plan: not importable by Rust. Rejected by premise.
- Ranking: B > A > C. B keeps comments and the pnpm-style intent/resolution split the maintainer endorsed (`mpm-toolchain-adoption.md:185-188`), and it wins despite the render-and-compare and
  the two-writer hazard because rationale comments (e.g. pin reasons in `plan.tools.data.ts`) have no other home in strict JSON. A is simpler (one file) but loses comments or forces a `note` field.
  C cannot be read by Rust. If the user values one file over comments, A overtakes B.
- Both read by both tools: mpm-shadow (sync, status, update) needs the lock only (all fields present). file-enforcer needs the lock only (manifests). Declarations are read by
  update alone (inference: nothing else needs `track/pinned/channel` beyond what the lock's `request` already holds, so even B's declarations file could be the lock's `request` plus `id` plus options).
- Format of the declaration file: JSONC fits the Rust rewrite (jsonc-edit crate, `file-enforcer-rust-rewrite.md`); TOML would match mpm but the rewrite chose JSONC; the lock stays JSON for jq.

## 5. Proposed split

Legend: owner -> reason. "Q" = open question for the user.

- Plan declarations and lock schema -> committed data (JSONC declarations or lock itself), shared Rust schema code. Reason: Rust cannot import `.data.ts`; lock already has all fields.
  Q: Option A or B above. Q: does the schema live in a small shared crate (both binaries parse the same lock), or does each parse the JSON it needs?
- Lock render (`lock.ts`, `generate.ts` lock half, 102 + part of 319) -> Option A: drop as a render, update writes the lock, `check` validates the schema. Option B: file-enforcer rule kind projecting
  declarations into the lock's declaration fields (two-writer hazard, section 4). Reason: the render is a pure projection either way.
- `mpm.packages.toml` render (`mpm-manifest.ts:150-193`) -> file-enforcer rule kind (pure function of the lock, deterministic, committed). Q: still committed, given its only code reader is the rustup restore?
- `mpm.managers.toml` render (`mpm-manifest.ts:306-358`) -> file-enforcer rule kind, or drop and let mpm-shadow materialize it when installing the shadow. Reason: single documented consumer; command arguments duplicate `providers.cargo.ts`.
  Q: keep it committed?
- Global-scope copies (lock-to-manifest render in the config dir) -> same rule kind pointed at an out-of-repo path, via the planned per-user opt-in config and separate subcommand
  (`file-enforcer-rust-rewrite.md`, "Out-of-repo rules"). Q: or mpm-shadow writes them as part of `update --global`.
- Install/sync (R4: `sync*`, `providers.*`, plus `exec.ts`, `layout.ts`, `paths.ts`) -> mpm-shadow subcommand under the `mpm-shadow` name (`mpm-shadow sync [--global]`). Reason: host and network action beside the tool that
  already shadows mpm and knows cargo's home; the `mpm` name stays pass-through.
  Q: should shadow mode also intercept `mpm restore` to apply locked versions (mpm drops versions for cargo, npm, pipx, `mpm-toolchain-adoption.md:389-399`)? Q: do the 10 routes shrink (drop `rustup-components` step-route, `cargo-git`, `go-install` if global-only)?
- Status (R5) -> mpm-shadow `status`, reading the lock; reuse `mpm installed` for manager-backed routes where it reports locked versions, keep native probes for `bin-sourced`/`archive`.
  Q: confirm `mpm installed --output-format json` covers route observers (unverified).
- Update (R7 resolvers and orchestration) -> mpm-shadow `update [--check] [--global]`: network, then rewrite lock rows (and nothing else if Option A/B). Reason: acts on network and the lock; the TS text patching is gone. Under Option B it must preserve declaration fields file-enforcer owns.
  Q: per-resolver port (11 families, 2,901 lines) or lean on `mpm outdated` / mpm's release-age cooldown for npm, PyPI, crates.io and keep native resolvers only for GitHub, rust, go, zig, node, java, android, conda.
- Exposure, postinstall links/commands, env file (R6) -> mpm-shadow `sync` phases. Q: drop `toolchain.env.json` or wire a reader (no reader exists today).
- bin-paths -> either `mpm-shadow bin-paths` or fold into the IDE: the IDE is Rust and the output is the lock's `bins` plus link existence in `.cache/toolchain-bin` (inference). Reason to fold: the sandbox
  cannot rely on a binary inside the not-yet-provisioned toolchain, which is the same bootstrap problem as below, whereas `node` is already needed. Cost: IDE `ENTRY_POINT` and `parse.rs` contract change.
  Q: which?
- Container runner (R8) -> test harness package (`package/test-harness/core-matrix`), merged with its existing `--memory=2g` container code. Reason: no toolchain logic; used by `ensurePackage` tests too.
- `toolPlanInstallableNames`, `ensurePackage` coupling -> drop.
- Dead code: `clearExposure`, `symlinkOrCopy`, `readToolEnvFile`, `plan.load*` -> drop.
- Mise task aliases and Containerfile `ARG RUST_TOOLCHAIN` copies -> repoint to the new binary; Q: have file-enforcer write the Containerfile default from the lock to remove the manual duplicate.

### Bootstrap order (flagged)

- Fact, today: a fresh machine needs git, mise (`README.md:171-195`), and an ambient Node that can run TypeScript, because `mise run toolchain:sync` is `node .../sync.ts` (`mise.no-env.toml:540-542`).
  sync imports workspace packages (`@monochromatic-dev/caught-value-core/ts`, `logger-core/ts`, `nano-spawn`), so `pnpm install` must run before it.
  README does not say how Node or pnpm get there. The Dockerfile does it by hand: `jq` reads node and pnpm URLs and SHA-256 from the lock, `curl`, `sha256sum --check`, symlinks
  (`Dockerfile:24-45`), then `pnpm install --frozen-lockfile` (`:45`), then `node package/file-enforcer/core/src/toolchain/sync.ts` (`:49`), then `mise run prepare` (`:51`).
  CI uses the same lock-by-jq idea for Node and pnpm only (`task-cli-setup/action.yml:42-46`).
- Fact: `Dockerfile:49` is `export $(cat .env.local | xargs) && node ... sync.ts`, and `README.md:165-169` says `bin`-sourced resolution reads `GITHUB_TOKEN` or the `gh` login and can exhaust the API quota mid-sync
  (`providers.environment.ts:52`). A token is an effective rung-0 input for containers and CI.
- Fact, inside the sync: `bin` (archive + SHA-256) -> `uv` -> `mpm` (PyPI wheel via `uv tool install`, `plan.tools.data.ts:991`) -> `rustup`/`rust` -> `cargo-binstall` -> `node` -> `pnpm` -> the rest, where
  `cargo:monochromatic-mpm-shadow` sits at `plan.tools.data.ts:388` as an ordinary `cargo-binstall` route. So the shadow is installed by the sync that would now be its subcommand.
- The new circle (inference): if `sync` becomes `mpm-shadow sync`, the first run needs an `mpm-shadow` binary that nothing has installed. Its release archives are Linux only, tar.gz, x86_64 and aarch64 (`package/mpm-shadow/cli/Cargo.toml:27` comment);
  a fresh macOS or Windows machine must compile it, and compiling needs rustup, which the sync installs. The lock has `artifacts: {}` for the shadow crate (`plan.tools.data.ts:407`), so no SHA-256 is recorded for a download.
- Required rung 0 (inference): download one prebuilt `mpm-shadow` per platform, verify the SHA-256 recorded in the lock, run `mpm-shadow sync`. That means (1) giving the shadow crate `artifacts` rows (the `archive` route) and a
  Linux/macOS/Windows release matrix, and (2) a documented manual step, since the repo forbids bash and PowerShell scripts (AGENTS.md rule SCR). Dockerfile and CI can keep the inline `jq` plus `curl` pattern.
- file-enforcer is not on the install critical path under this split (inference): sync reads the committed lock and needs no render. file-enforcer is needed only to regenerate or `check` the manifests, so the bootstrap does not have
  to include it. It would still be a plan entry (`cargo:` or `archive`) installed by `mpm-shadow sync`.
- Mise stays ambient: `mise run` is how users launch it today. Dropping the Node requirement is a gain of the Rust sync; mise still needs installing first until the tasks move (`mpm-toolchain-adoption.md:190-199`).
- Q: acceptable that a fresh machine without a published `mpm-shadow` build for its platform has no path except `cargo install monochromatic-mpm-shadow`?

## Consequences to record if the split is accepted (inference)

- Amend `doc/decision/mpm-toolchain-adoption.md`: `:203-205` (file-enforcer owns the typed policy), `:289-292` (the shadow as "single owner of the rule" for pruning only), and the shadow's README and
  `Cargo.toml:18` description, which today say it only runs mpm and prunes.
- Amend `doc/planning/mpm-toolchain-migration.md` (pins the `bin-paths` command consumers run) and `doc/planning/file-enforcer-rust-rewrite.md:133-143` (open toolchain question).
- Format: lines in this report exceed 120 characters (MD1); reflow before pasting into `doc/`.
