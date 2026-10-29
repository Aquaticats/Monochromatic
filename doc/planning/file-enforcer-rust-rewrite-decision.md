# Rust file-enforcer rewrite: decision record draft

## Status

Draft for acceptance,
written 2026-10-29 after the user confirmed a shared understanding of the design
("Shared understanding",
answering the closing question of the grilling).
Not accepted:
on the user's acceptance this file moves to `doc/decision/file-enforcer-rust-rewrite.md`,
and the amendments under "Consequences" are applied.
Every decision's question,
answer,
evidence,
and history is in [`file-enforcer-rust-rewrite.md`](file-enforcer-rust-rewrite.md);
this record states the result.

## Context

- Request:
  "Migrate to Rust + clap + config with jsonc for file-enforcer.
  Ground up rewrite.
  No need to preserve anything." (user, 2026-10-29)
- Constraint:
  "Consider the original file-enforcer bad in every way and don't treat it as Jesus." (user, 2026-10-29)
- `doc/decision/monorepo-manager-all-rust.md` put file enforcement inside meow with an HCL config;
  meow is plan-only with no code.
- Evidence:
  the research and review reports under `doc/research/file-enforcer-rust-rewrite-*.md`,
  including three outside reviews per round (two rounds),
  a source-level mise study,
  and a podman prototype of the toolchain files (branch `prototype/fe-mpm-files`).

## Decision

### Shape

- Standalone tool that meow can embed later:
  `package/file-enforcer/core-rust` (library `monochromatic-file-enforcer`,
  engine and rule kinds,
  no clap) and `package/file-enforcer/cli` (binary `file-enforcer`,
  clap).
- Config `file-enforcer.config.jsonc`,
  read with the in-repo `monochromatic-jsonc-edit` and decoded by hand.
- Fixed Rust rule kinds with structured JSONC values (literals and tagged objects such as `{"file": ...}`,
  `{"var": "dir"}`,
  `{"concat": [...]}`);
  no string expression language.
- Every line the tool writes is one JSON object,
  including clap's help,
  version,
  and parse errors.
- Exit codes:
  0 clean,
  1 drift or rule failure,
  2 bad command line or config.

### Execution

- One auto-started server per worktree serializes every write
  over an owner-only Unix socket with newline-delimited JSON-RPC 2.0,
  and exits when idle;
  `user` commands use their own server keyed by the per-user config path.
- Commands are clients:
  `write`,
  `check` (`--expected-dir <dir>` writes paired sparse actual and expected trees for standard diff tools),
  `watch` (subscribes to the server,
  which watches inputs,
  outputs,
  and glob roots,
  and warns on hand edits to generated files),
  `explain <path>`,
  `user write`,
  `user check`.
- Each run plans every rule against a virtual view of planned outputs,
  validates the whole run,
  revalidates the snapshot it read,
  then publishes file by file through hidden staging names;
  atomicity is per file,
  and a partial run reconverges on the next run.
- Server lifecycle contracts:
  a startup lock separate from the socket,
  verified stale-socket recovery,
  a handshake on protocol version,
  build identity,
  and worktree identity,
  exit when the worktree root disappears,
  a verified owner-only fallback when `$XDG_RUNTIME_DIR` is unusable,
  and detached standard streams.
- Mirror and export paths are never followed through symlinks,
  and containment is rechecked at mutation time.

### Outputs file-enforcer writes

- `CLAUDE.md` (Claude-only preamble from a content file,
  then `AGENTS.md`).
- Root `LICENSE` and every package's `LICENSES/` copies,
  pruning only the tool-owned ids (CC-BY-SA-4.0,
  GPL-3.0-or-later,
  LGPL-3.0-or-later).
- `.claude/skills` and `.factory/skills` as copies of `.agents/skills`,
  with the targets owned outright.
- Per-crate `Cargo.toml` keys (edition,
  license,
  publish,
  lints,
  repository,
  homepage,
  readme,
  profiles,
  Dylint check-cfg),
  and `rust-catalog.toml` plus `[workspace.dependencies]` through a new dependency-catalog Rust library.
- `.browserslistrc.resolved.local.json` through `browserslist-rs`.
- `.config/mise/conf.d/10-bin.local.toml` (gitignored):
  the existing non-empty `package/*/*/node_modules/.bin` directories.
- Through `user write`:
  the JetBrains Harper LSP4IJ policy under `~/.config/JetBrains`.

### Outputs that stop being generated

- Root `mise.toml`:
  `mise.no-env.toml` becomes the hand-written `mise.toml`,
  and the static environment moves to the committed `.config/mise/conf.d/20-env.toml`.
- `.cache/forbidden-strings.rules.txt` and its seed:
  forbidden-strings and cli-git read a comma-separated list,
  each entry anchored with `{{config_root}}`,
  where a trailing `?` marks an entry that may be absent;
  cli-git's JSONC array uses `{"path": ..., "optional": true}`.
- `mpm.packages.toml` and `mpm.managers.toml`,
  in both scopes.
- Also gone:
  the scanner post-step,
  mise task-header quoting,
  the `CONTEXT.md` guard (cli-git enforces it),
  the OS package installer and its Repology pipeline,
  the staleness cache and its lock,
  and the `exec` export.

### Schema

- Committed `file-enforcer.schema.json`,
  derived with `schemars`,
  with a cargo golden test that also checks the hand-written decoder against a coverage-complete sample corpus,
  mapped in `.idea/jsonSchemas.xml`,
  with `"$schema"` kept for VS Code.

### Toolchain (mpm-shadow)

- People edit `toolchain.plan.jsonc`;
  mpm-shadow is the only writer and checker of `toolchain.lock.json`,
  which stays strict JSON for its `jq` readers,
  carries a digest of its contents,
  and keeps its frozen surface (tool ids,
  `version`,
  `artifacts.<platform>.{url,sha256}`,
  `options.androidPackages`).
- Commands:
  `sync`,
  `status`,
  `update [--check]`,
  `lock` (offline),
  `lock --check` (the toolchain drift gate in CI beside `file-enforcer check`),
  and `bin-paths`,
  each with `--global`.
- Lookups:
  first versions of new npm and PyPI tools from the registries,
  later ones from `mpm outdated`,
  native lookups for the rest including Rust channels;
  `update --check` never installs.
- `sync` and `status` refuse when plan and lock disagree;
  offline `lock` rejects resolution-affecting edits;
  rustup channels and cargo-binstall tools install directly from the lock;
  postinstall links and commands run in sync.
- Mutations revalidate the plan and lock revisions they read before publishing.
- Bootstrap:
  a prebuilt `mpm-shadow` archive with its SHA-256 in the lock,
  or `cargo install monochromatic-mpm-shadow` after a verified rustup install.
- file-enforcer has no toolchain role.

### Elsewhere

- pnpr generates and checks its own `package/config/pnpr/config.yaml`.
- The bounded container runner moves to `package/test-harness/core-matrix`.
- `package/vm-builder/cli` gets a private process runner.
- The IDE launcher finds `mpm-shadow` by explicit path for `bin-paths`.

### Cutover

- One switch,
  after everything is built,
  published,
  fetched by a clean container,
  and compared with the TypeScript tool's output on this repo;
  output need not be byte-identical.
- The switch commit repoints tasks,
  CI (checking committed outputs before any regeneration,
  then local outputs separately),
  the `Dockerfile`,
  the IDE launcher and its tests,
  the scanner and cli-git configs,
  the two fixtures that still depend on the old package,
  and deletes `package/file-enforcer/core`.
- The global plan is converted directly,
  keeping a 0600 backup beside the original until the converted global sync is verified.

## Consequences

- Amend `doc/decision/monorepo-manager-all-rust.md`:
  meow links `monochromatic-file-enforcer`,
  its file-enforcement config is JSONC,
  and the root `mise.toml` is no longer generated.
- Amend `doc/decision/mpm-toolchain-adoption.md`:
  no generated restore manifest (`:92-112`),
  and mpm-shadow,
  not file-enforcer,
  owns the toolchain policy and lock (`:201-223`).
- Gates before implementation:
  a live forbidden-strings run on throwaway rules files (including a missing optional entry),
  `browserslist-rs` on the real `.browserslistrc` feeding the rolldown consumer,
  `mpm outdated` probes varying installed,
  locked,
  and registry versions (the reading rule stays pending until then),
  and choosing-technology vets for `toml_edit`,
  `browserslist-rs`,
  `schemars`,
  and an XML editor.
- Follow-ups:
  process rules in issue #1817.

## Rejected

- Building it as meow's first slice,
  or standalone with no embedding path.
- The `config` crate (config-rs):
  no JSONC format,
  no error positions.
- String expressions or Rust-only named rules for the config.
- A text renderer beside JSON lines.
- No coordination,
  or flock instead of the server,
  for file-enforcer writes.
- A three-file toolchain model (plan,
  resolution,
  lock).
- file-enforcer rendering or checking the mpm files:
  the prototype showed it fails on a machine without file-enforcer and that the files mislead installs.
- Replacing `CLAUDE.md` with an `@AGENTS.md` import or the skill copies with symlinks:
  closed-source tool behavior is not a contract.
