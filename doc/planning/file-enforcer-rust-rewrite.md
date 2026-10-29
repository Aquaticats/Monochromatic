# File-enforcer Rust rewrite

## Status

Being grilled with the user, started 2026-10-29.
Nothing here is accepted yet;
decisions land in this file as the user answers them.

## Request

"Migrate to Rust + clap + config with jsonc for file-enforcer.
Ground up rewrite.
No need to preserve anything." (user, 2026-10-29)

## Constraints stated by the user

- "Consider the original file-enforcer bad in every way and don't treat it as Jesus." (user, 2026-10-29)
  The TypeScript package at `package/file-enforcer/core` is evidence only of which files the repo needs managed.
  Its builder API,
  subsystem boundaries,
  CLI shape,
  staleness manifest,
  locking,
  and config-as-TypeScript model are not requirements and are not a design to port.

## Decisions

### Standalone tool that meow can embed

Asked 2026-10-29 how the rewrite relates to meow,
whose accepted decision (`doc/decision/monorepo-manager-all-rust.md`, 2026-09-16)
puts file enforcement inside the single meow binary with an HCL config and daemon-owned watch mode.
The user chose "Standalone, meow can embed".

- The engine is a library crate and the CLI is a clap binary over it;
  meow may depend on the library later instead of rewriting it.
- The tool has its own watch mode until a meow daemon exists.
- The config is JSONC,
  following the cli-git and linter rewrites (`doc/decision/cli-git-rust-rewrite.md`),
  which also moved off HCL.
- Consequence to record when this is accepted:
  amend `doc/decision/monorepo-manager-all-rust.md` so meow links this crate and its file-enforcement config is JSONC.

Evidence at the time:
meow is plan-only with no code
(`doc/handover/monorepo-manager.md`, "Design research; no product code"),
last designed 2026-09-21,
and issue #1799 prefers an existing tool over building a monorepo manager.

### JSONC through the in-repo jsonc-edit crate

Asked 2026-10-29 which JSONC stack reads `file-enforcer.config.jsonc` and edits JSONC targets;
the user chose "In-repo jsonc-edit" over `jsonc-parser` (dprint) and the `config` crate (config-rs).

- `monochromatic-jsonc-edit` (`package/jsonc-edit/core-rust`) is the accepted foundation
  (`doc/decision/jsonc-edit-rust-foundation.md`)
  and the loader for `cli-git.config.jsonc` and `monochromatic-lint.config.jsonc`.
- The config is decoded by hand per field,
  as `package/git-policy/cli/src/native/config_parse.rs` does;
  parse errors map the crate's byte offset to `path:line:col`.
- Emit is canonical,
  so editing a hand-formatted JSONC target re-formats it.
- Missing features (serde,
  value spans) get added to the crate,
  not worked around.
- Why not config-rs (`config` 0.15.27, researched 2026-10-29):
  no JSONC format (only JSON5 through `FileFormat::Json5`),
  no line or column on `ConfigError`,
  environment keys lowercased,
  and read-only.
  Evidence:
  [`doc/research/file-enforcer-rust-rewrite-crates.md`](../research/file-enforcer-rust-rewrite-crates.md),
  whose claims come from reading source and docs,
  not from running the crates.

## Research

- [`doc/research/file-enforcer-rust-rewrite-meow-status.md`](../research/file-enforcer-rust-rewrite-meow-status.md):
  meow's state and its decisions about file enforcement.
- [`doc/research/file-enforcer-rust-rewrite-inventory.md`](../research/file-enforcer-rust-rewrite-inventory.md):
  every file the repo needs managed,
  stated as outcomes,
  plus the old tool's flaws.
- [`doc/research/file-enforcer-rust-rewrite-crates.md`](../research/file-enforcer-rust-rewrite-crates.md):
  clap,
  config-rs,
  JSONC,
  schema,
  and structured-edit crates.

### Config model: rule kinds with structured values

Asked 2026-10-29 how much logic the JSONC config carries;
the user chose "Rule kinds + structured values"
over string expressions and over named Rust rules with JSONC toggles.

- Rust implements a fixed set of rule kinds
  (concatenate,
  mirror a directory,
  set TOML keys where a table exists,
  append a block if absent,
  fan out over packages,
  plus named built-ins).
- JSONC values are literals or tagged objects such as `{"file": ...}`,
  `{"var": "dir"}`,
  and `{"concat": [...]}`;
  no string expression language.
- Every input and output path is known from the config without running anything,
  so `check` is exact and a JSON Schema covers the whole file.

### Rust catalog sync: a Rust library in dependency-catalog

Asked 2026-10-29 who renders each crate's `[workspace.dependencies]` from `rust-catalog.toml`;
the user chose "Rust lib in dependency-catalog".

- A new Rust library crate in `package/dependency-catalog` owns parsing and rendering the catalog.
- file-enforcer links it behind a `rust-catalog` rule kind,
  so file-enforcer stays the one writer of every `Cargo.toml`.
- The TypeScript update,
  verify,
  and tighten tools keep their own parser until they move.

### pnpr owns its publishable-package list

Asked 2026-10-29 who produces `package/config/pnpr/config.yaml`;
the user chose "pnpr owns it, own check".

- `package/config/pnpr` generates the file with its own write and check tasks,
  reusing its `isTypeScriptSourcePath`;
  file-enforcer is not involved.
- CI runs pnpr's check beside file-enforcer's.

### Toolchain: split between file-enforcer and the mpm wrapper

Asked 2026-10-29 where toolchain sync,
status,
update,
and `bin-paths` go.
The user rejected all three options (own TypeScript package,
separate Rust tool,
inside file-enforcer) and wrote:
"Some needs to be folded into Rust file-enforcer,
some needs to be folded into our mpm wrapper."

- The mpm wrapper is `package/mpm-shadow/cli` (Rust,
  clap).
- Research:
  [`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`](../research/file-enforcer-rust-rewrite-toolchain-split.md).

Then asked 2026-10-29 how to split;
the user chose "Split by side effect":

- file-enforcer renders,
  from committed inputs only,
  `mpm.packages.toml` and `mpm.managers.toml` from the lock,
  and the lock's declaration fields from the plan.
- mpm-shadow gains `sync`,
  `status`,
  and `update` (network resolution writing lock versions and artifacts),
  each with `--global` for the machine scope,
  plus the `.cache/toolchain-bin` exposure farm and postinstall links.
  Under the `mpm` name it stays a pass-through plus prune.
- The bounded container runner moves to `package/test-harness/core-matrix`.
- Dropped as unused:
  `clearExposure`,
  `symlinkOrCopy`,
  `.cache/toolchain.env.json` (no reader),
  and the TypeScript plan loaders.

### Toolchain plan: JSONC plan and JSON lock

Asked 2026-10-29 where the hand-written plan lives;
the user chose "JSONC plan + JSON lock" over making the lock the only file.

- `toolchain.plan.jsonc` holds intent
  (id,
  request,
  route,
  bins,
  os,
  options)
  with rationale comments.
- `toolchain.lock.json` stays strict JSON for its `jq` readers
  (`Dockerfile:24-45`,
  `.github/actions/task-cli-setup/action.yml:42-46`,
  `.github/workflows/music-player-release.yml`).
- A file-enforcer rule projects the plan's fields into the lock,
  and `check` fails on drift.

### Lock writer: a separate resolution file

Asked 2026-10-29 who writes `toolchain.lock.json`;
the user chose "Separate resolution file".

- `mpm-shadow update` writes only `toolchain.resolution.json`
  (strict JSON:
  per tool,
  version and artifacts with URL and SHA-256).
- file-enforcer renders `toolchain.lock.json` from `toolchain.plan.jsonc` plus `toolchain.resolution.json`
  and is the lock's only writer,
  so `check` rebuilds the whole lock from committed inputs.

### Bootstrap: prebuilt mpm-shadow and cargo install

Asked 2026-10-29 how a fresh machine gets its first `mpm-shadow`;
the user answered "A+B":
both a prebuilt archive and `cargo install`.

- `mpm-shadow` becomes an archive-route plan entry with release URLs and SHA-256 in the lock;
  `Dockerfile` and CI fetch it with `jq`,
  `curl`,
  and `sha256sum`,
  as they already do for Node and pnpm (`Dockerfile:24-45`).
- `cargo install monochromatic-mpm-shadow` is the other documented path,
  and the only one on platforms without a prebuilt archive.
- A runbook step covers people's machines,
  since `AGENTS.md` (`SCR`) forbids bash and PowerShell scripts.

### bin-paths: an mpm-shadow subcommand

Asked 2026-10-29;
the user chose "mpm-shadow bin-paths".
`mpm-shadow bin-paths` prints the same JSON (`exposureDir`,
`pathDirs`,
`bins`) that `package/ide/app/src/language/toolchain.rs` parses today,
so mpm-shadow stays the only owner of the exposure layout.

### Update: mpm where it can, ported lookups elsewhere

Asked 2026-10-29 (re-asked in plainer words after the user wrote "What is going on? I don't understand.")
how `mpm-shadow update` finds new versions;
the user chose "Ask mpm where it can" over porting every lookup (my recommendation).

- npm and PyPI tools:
  `mpm outdated --output-format json`.
- Ported to Rust:
  GitHub releases,
  nodejs.org,
  crates.io (mpm's `cargo` manager has no `outdated`,
  `doc/research/mpm-run-capabilities.md`),
  go,
  zig,
  Adoptium,
  Android,
  conda.
- Accepted limits:
  `mpm outdated` sees only tools installed on the machine running it,
  and reports no download hashes.
- Retracted from the first asking:
  my con that mpm "drops versions" described mpm's restore (install) behavior
  (`doc/decision/mpm-toolchain-adoption.md:389-399`),
  not version lookup.

### browserslist through browserslist-rs

Asked 2026-10-29 what happens to `.browserslistrc.resolved.local.json`;
the user chose "Rust rule kind (browserslist-rs)" over moving resolution into rolldown's config
(my recommendation) and over spawning Node.

- A `browserslist` rule kind resolves `.browserslistrc`'s production section with `browserslist-rs`
  (0.21.2,
  2026-09-16) and writes the same JSON array for `package/config/rolldown/src/browserslist-targets.ts`.
- Accepted tradeoff:
  the crate bundles its own browser data,
  versioned with the crate rather than the pnpm lock,
  so its targets can differ from what JavaScript tooling computes.
- `browserslist-rs` needs a choosing-technology vet before it enters `rust-catalog.toml`.

### Out-of-repo rules: per-user config, opt-in

Asked 2026-10-29 what happens to the Harper policy written into JetBrains state under `~/.config/JetBrains/`;
the user chose "Per-user config, opt-in".

- A per-user JSONC config under `~/.config` holds rules that write outside the repo,
  matching `doc/decision/monorepo-manager-per-user-config.md`.
- A separate subcommand applies it;
  the default `write` and `check` touch only repo files.

### No scanner step

Asked 2026-10-29 about the post-write `forbidden-strings compile-rules` run;
the user chose "Drop it".
file-enforcer writes `.cache/forbidden-strings.rules.txt` only,
and forbidden-strings compiles from that text when its cache is missing or stale
(`package/forbidden-strings/cli/src/runtime_cache/warning_tests.rs:23`).

### Crates: core-rust library and cli binary

Asked 2026-10-29 how to split crates so meow can embed the engine;
the user chose "core-rust lib + cli bin".

- `package/file-enforcer/core-rust`:
  library `monochromatic-file-enforcer`,
  the engine and rule kinds,
  no clap.
- `package/file-enforcer/cli`:
  binary `file-enforcer`,
  clap,
  JSON-lines output,
  exit codes.
- `package/file-enforcer/core` (TypeScript) is deleted at cutover.

### Config schema: generated and self-enforced

Asked 2026-10-29 whether the config gets a JSON Schema;
the user chose "Generated, self-enforced".

- The schema is derived from the Rust config types with `schemars`,
  which needs a choosing-technology vet.
- `file-enforcer.schema.json` is committed and kept current by a file-enforcer rule,
  so `check` fails when it is stale;
  the config's `"$schema"` key points at it.
- Unverified:
  JetBrains support for `$schema` in `.jsonc` files.

### mise task-header quoting: not normalized

Asked 2026-10-29;
the user chose "Drop it".
Both `[tasks.'x']` and `[tasks."x"]` stay as written,
since nothing reads the spelling and Mise is due to be removed.

### Root mise.toml PATH: existence-based, as today

Asked 2026-10-29 how `_.path` (`mise.toml:1262`) lists `package/*/*/node_modules/.bin` directories;
the user chose "Keep existence-based",
noting:
"The existence of option 1 means you don't understand how pnpm, mise, node_modules, path works."

- The rule globs the `node_modules/.bin` directories that exist and sorts them,
  as the TypeScript generator does (`file-enforcer.config.ts:722-745`),
  but skips empty ones:
  asked 2026-10-29,
  the user chose "Skip empty .bin" after the measurement found 18 of the 55 empty.
- The mechanism and counts are recorded in
  `doc/troubleshooting/pnpm-undeclared-workspace-links-survive-install.md`,
  "When a package gets node_modules/.bin",
  where the user chose to keep them instead of an `AGENTS.md` rule.
- Retracted:
  my option 1 derived the list from every package with a `package.json`.
  The user's correction:
  "You cannot meaningfully determine if a .bin will exist from just reading package.json."
  Whether a `.bin` exists depends on the resolved dependencies' bins and on how pnpm links them,
  which `package.json` does not determine,
  so the existing directories are the only reliable signal
  and option 1 would have named directories no install creates.
  Measured 2026-10-29 in the main checkout:
  223 `package/*/*/package.json`,
  205 `package/*/*/node_modules`,
  55 `package/*/*/node_modules/.bin`.
- Accepted tradeoff:
  issue #335 records the list changing between checkouts after filtered or stale installs,
  so `check` reports `mise.toml` drift on such a machine until a full `pnpm install` runs.

### Adopted from the request and repository rules

- Rust is approved for this tool by the request itself;
  no existing approval scope (meow, cli-git, the linter) covers a standalone file-enforcer.
- The CLI uses clap,
  as requested;
  repository precedent is clap derive in `package/forbidden-strings/cli` and others.
- Every Rust dependency comes from `rust-catalog.toml` through `workspace = true`
  (`doc/decision/rust-dependency-catalog.md`).
- Rust tests run through nextest (`RNQ` in `AGENTS.md`).
- Every line the tool writes is one JSON object (`schemaVersion`, `sequence`, `type`, `code`, `message`, `hint`),
  matching meow's accepted "Emit every line as a json, error or warning or not." (user, 2026-09-17)
  and the live cli-git event stream (`package/git-policy/cli/src/native/diagnostics.rs:223`).
  Adopted because the library is meant for meow to embed,
  then confirmed 2026-10-29 when the user chose "JSON lines always" over a TTY-dependent text renderer
  and a text default with a `--json` flag.
- Dropped outright under "No need to preserve anything",
  because nothing outside the package uses them:
  the OS package installer (`ensurePackage`),
  its 22,607-line Repology-derived index,
  and the PostgreSQL `generator/` pipeline;
  the staleness manifest,
  its cross-process lock,
  and orphan-temp sweeping
  (`doc/troubleshooting/file-enforcer-staleness.md` shows the cache never recorded for the root config
  and could keep stale output with exit 0);
  the `CONTEXT.md` guard,
  which cli-git's `mono/forbidden-root-context` policy already enforces.
- No incremental cache ships without a measured need.
- `check` renders every output in memory and compares,
  needing no prior write,
  so CI's `git diff --exit-code` workaround in `.github/workflows/advisory-core.yml:92-105` can go.
- One entry point:
  the clap binary,
  with unknown flags rejected
  (the old `node file-enforcer.config.ts` path skipped `cli.ts` and silently dropped unknown flags).
- Output need not be byte-identical to the TypeScript tool's;
  the first Rust run may reformat managed files once.
- `exec` leaves with the rewrite:
  `package/vm-builder/cli` gets its private runner,
  as already designed in `doc/decision/monorepo-manager-all-rust.md` ("Consequences").
- The root `mise.toml` stays generated from `mise.no-env.toml` until Mise is removed,
  as `doc/decision/monorepo-manager-all-rust.md` already decides.
- Long fixed text (the `CLAUDE.md` preamble,
  the `mise.toml` `[env]` block) lives in content files the config names,
  never inline in the config.
- The TypeScript file-enforcer keeps running until the Rust tool replaces it;
  concurrent edits in `package/file-enforcer/core` belong to other sessions.

## Open questions

Tracked in the grilling rounds;
each answered question moves into a decisions section with the user's words and date.
