# Planning: shared Rust dependency catalog

Status:
 proposal,
 design decisions in progress with the owner.
Tracks [issue #1800](https://github.com/Aquaticat/Monochromatic/issues/1800).
Builds on `doc/planning/cargo-toml-file-enforcer.md` and respects `.out-of-scope/cargo-workspace.md`.

## Goal

One declarative source of truth for Rust dependency requirements across the independent Cargo workspace roots,
with a one-place update workflow,
a CI check that fails on drift,
documented per-crate exceptions,
and stale-entry detection,
without a repository-root Cargo workspace and without centralizing profiles.

## Measured facts (2026-10-29)

Measured with a read-only scratch script that parses every first-party `Cargo.toml` through
`@monochromatic-dev/module-toml-edit` and lists every entry in `dependencies`,
`dev-dependencies`,
`build-dependencies`,
and their `target.<cfg>.*` variants.

### Fleet size

- 24 first-party manifests,
   each its own workspace root.
- 101 distinct dependency names,
   194 entries.
- 35 names appear in two or more crates;
   66 appear in exactly one.
- No manifest declares `[workspace.dependencies]` or `[patch]`.

### What the current mechanism covers and misses

`CARGO_SHARED_DEPENDENCIES` in `file-enforcer.config.ts` owns 19 entries,
 each as a whole value (version plus features).
 Its guard paths are `dependencies.<name>` and `build-dependencies.<name>` only, so it never sees:

- `dev-dependencies`
   (`i-slint-backend-testing = "=1.18.1"` in `package/desktop-app/ide` and `package/music-player/desktop-app`;
   `tempfile`).
- `target.<cfg>.dependencies`
   (`ignore` and `libc` and `tempfile` in `package/git-policy/cli` under `cfg(unix)`;
   `libc` in `package/music-player/desktop-app`;
   `zbus` under `cfg(target_os = "linux")`).

Because it owns features,
 any crate needing a different feature set must leave the catalog entirely
 (`tokio`, `slint`, `image` today).
`gtk4` is excluded with a "divergent" note,
 but both crates now declare the identical `{ version = "0.11", features = ["v4_12"] }`.

### Divergent requirements today

- `tokio`:
   `"1.50"` in `package/desktop-app/ide`,
   `"1"` elsewhere;
   feature sets differ per crate;
   `optional = true` in `package/music-player/truepeak-core`.
- `zbus`:
   `"5.19.0"` in `package/cli/nested-wayland-session`,
   `"5"` in `package/music-player/desktop-app`.
- `windows`:
   `0.58` in `package/desktop-app/file-manager`,
   `0.62` in `package/music-player/desktop-app`;
   different majors and different feature lists.
- `slint`:
   all `1.18.1`,
   feature sets differ
   (`package/desktop-app/ide` is GPU-only with `default-features = false`).
- `image`:
   both `0.25`,
   features differ.

### Families that must move together

- Slint:
   `slint`,
   `slint-build`,
   `i-slint-backend-winit`,
   `i-slint-backend-testing` (exact `=` pin required by its README),
   all `1.18.1`.
- rust-analyzer:
   nine `ra_ap_*` crates,
   all `=0.0.336`,
   across `package/linter/monochromatic-lint` and its fuzz sidecar.
- Helix:
   three git dependencies on one `rev` in `package/desktop-app/ide`.
- CXX-Qt:
   `cxx-qt`,
   `cxx-qt-lib`,
   `cxx-qt-build`,
   all `0.9.1`.

### Path dependencies carrying a published version

`forbidden-regex` (`0.1.1`),
`monochromatic-jsonc-edit` (`0.1.1`),
and `forbidden-strings` (`0.5.0`) are depended on by path plus `version`,
 so `cargo publish` can resolve them.
All three match the target crate's `package.version` today;
 nothing enforces it.

### Resolved versions drift across lockfiles

Each crate has its own `Cargo.lock`,
 so identical requirements still resolve differently.
Sampled resolved versions:
`anyhow` 1.0.102, 1.0.103, 1.0.104;
`serde` 1.0.228, 1.0.229;
`clap` 4.6.1, 4.6.7;
`libc` 0.2.186 to 0.2.191;
`zbus` 5.16.0, 5.19.0.

### CI today

No workflow runs `file-enforcer` or checks its output.
`.depot/workflows/current-baseline.yml` runs only on `workflow_dispatch`.
Lightweight checks on `pull_request` exist (`.github/workflows/forbidden-strings.yml`, `fs-id.yml`);
 `advisory-core.yml` also runs on `push`.
Auto-push lands commits on `main` directly,
 so a drift check must run on `push` to catch most changes.
No Renovate or Dependabot configuration exists.
Installed git hooks are `post-checkout`,
`post-commit`,
`post-merge`,
and a Git LFS `pre-push`;
 there is no `pre-commit` gate.
No crate declares `package.rust-version`.

## Ecosystem research (2026-10-29)

Primary sources gathered by a research subagent;
 probes marked as such were run on the repo's cargo
 (`.cache/toolchain-bin/cargo`, `cargo 1.101.0-nightly (3960fa69b 2026-10-12)`).

### Cargo cannot share requirements across workspace roots

- A crate inherits only from a workspace it is a member of;
   membership brings that root's single `Cargo.lock` and root-only `[profile.*]`.
   [Cargo workspaces reference][cargo-workspaces].
- Nested workspaces ([rust-lang/cargo#5042]) is open,
   needs an RFC,
   and nobody is working on it.
   Remote version definitions ([rust-lang/cargo#15435]) is open with no accepted design.
- A registry dependency without `version`,
   `path`,
   `git`,
   or `workspace` is a hard error;
   unknown keys in a dependency table only warn.
   So every manifest must carry literal versions somewhere;
   a catalog can only be a source that a generator renders.

### Each crate may carry its own `[workspace.dependencies]`

Every crate here is already its own workspace root,
 so it can declare `[workspace.dependencies]` and reference it with `workspace = true`.
The use site may add only `features` (additive),
`optional`,
and `default-features`.
`[workspace.dependencies]` entries cannot themselves be `optional`.

Probe (throwaway crate,
 `memchr = "2.7.2"` and `itoa = "=1.0.18"` in its own `[workspace.dependencies]`,
 use site `memchr = { workspace = true, default-features = false, features = ["alloc"] }`):

- `cargo tree --offline --edges features` listed `memchr feature "alloc"` only,
   while `itoa` (defaults kept) listed `itoa feature "default"`,
   so a member-side `default-features = false` override works on edition 2024.
- `cargo package --offline --no-verify` produced a normalized `Cargo.toml` with literal
   `version = "2.7.2"`,
   `default-features = false`,
   `features = ["alloc"]`,
   and the exact pin `version = "=1.0.18"`,
   so published crates lose nothing.

### Tools

- No tool was found that syncs one catalog across independent Cargo roots
   (searched crates.io,
   GitHub,
   web;
   "not found",
   not "does not exist").
   `cargo-autoinherit`,
   `cargo-hakari`,
   `cargo-workspaces`,
   and `cargo-rail` all work within one workspace at a time.
- Cargo's `unused_workspace_dependencies` lint flags a `[workspace.dependencies]` entry no member inherits;
   stabilized for Rust 1.100 ([rust-lang/cargo#17298]),
   present on the repo's nightly.
   Fleet-wide staleness (an entry no crate uses) still has to live in the generator.
- `cargo-deny` `[bans.workspace-dependencies]` flags literal declarations beside an inherited one,
   per workspace root.
- Renovate's cargo manager extracts `[workspace.dependencies]` and can group updates across roots with `groupName`;
   propagating a catalog bump needs the generator to run (`postUpgradeTasks`, admin-allowed).
   Dependabot has open bugs for many roots in one repo ([dependabot-core#11861]) and for files that are both package and
   workspace root ([dependabot-core#16037]).

### Precedent

Gradle version catalogs (`libs.versions.toml`),
Maven `dependencyManagement`,
pnpm catalogs,
Bazel `crate_universe`,
and Buck2 Reindeer all keep identity plus version centrally and leave variation (features, classifiers, optional) at
 the consumer.
pnpm alone offers enforcement modes (`catalogMode`: `manual`, `prefer`, `strict`),
named catalogs for alternative versions,
and pruning of unused entries (`catalogPrune`).
Gradle's `[versions]` section shares one version across several libraries,
 the shape the Slint and rust-analyzer families need.

[cargo-workspaces]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[rust-lang/cargo#5042]: https://github.com/rust-lang/cargo/issues/5042
[rust-lang/cargo#15435]: https://github.com/rust-lang/cargo/issues/15435
[rust-lang/cargo#17298]: https://github.com/rust-lang/cargo/pull/17298
[dependabot-core#11861]: https://github.com/dependabot/dependabot-core/issues/11861
[dependabot-core#16037]: https://github.com/dependabot/dependabot-core/issues/16037

## Decisions

Owner answers of 2026-10-29.
An earlier round of answers was voided by the owner because the questions did not state why each option ranked where
 it did;
 only answers given with stated rankings count.

### Owner of the sync and the check

Extend `file-enforcer`'s existing Cargo engine
 (`package/dev-script/file-enforcer/src/cargo/`) with the catalog and a check-only mode.
Why:
 `Cargo.toml` keeps one owner (edition, lints, `[workspace]`, profiles, and dependencies);
 a new Rust CLI built on `toml_edit` (the crate Cargo itself depends on) would split ownership for a lighter CI setup;
 contributing upstream (for example to `cargo-rail`, which unifies each workspace separately) waits on another roadmap.
Consequence:
 the CI check needs Node and a pnpm install.

### Manifest shape

Each crate's own root manifest gets a generated `[workspace.dependencies]` table holding only the catalog entries it
 uses,
 and its use sites reference them with `workspace = true` plus crate-local `features`,
`optional`,
and `default-features`.
This is not a root workspace:
 every crate stays its own workspace root.
Why:
 use sites become a catalog reference instead of a literal requirement (the issue's own complaint),
 and Cargo rejects a hand-written `version` beside `workspace = true` between check runs.
Consequence:
 features are additive under `workspace = true`,
 so catalog entries carry versions and no features.

### Membership

Strict:
 every registry and git dependency references the catalog;
 a literal requirement fails the check unless it is a documented exception.
Path dependencies stay out of the catalog.
Why:
 one place then covers every version,
 matching the JS rule that every external dependency uses `catalog:`.

### Catalog format

A standalone TOML file in Cargo dependency syntax,
 not named `Cargo.toml`,
 read and validated by `file-enforcer` on every sync and in the CI check.
Why:
 it uses the syntax Rust contributors already write and is a plain data file like `pnpm-workspace.yaml`;
 a standalone TypeScript module would add edit-time typing but put Rust versions in TypeScript syntax,
 and inline data in `file-enforcer.config.ts` fails the issue's "discoverable" requirement.

### Lockfile alignment

Out of scope for #1800,
 tracked in [issue #1801](https://github.com/Aquaticat/Monochromatic/issues/1801).
Why:
 aligning resolved versions also covers transitive dependencies and legitimate per-crate older resolutions,
 a separate policy that does not change the catalog's design.

### Follow pnpm's full catalog feature set

Owner direction:
 model the Rust catalog on everything pnpm catalogs offer,
 not only the subset this repo's JS side uses today,
 and ask which features to omit.
The mapping is in the section "pnpm feature map".

### Exceptions

pnpm-style named catalogs:
 alternative versions live in `[catalogs.<name>]` tables of the catalog file.
Where a crate selects a named catalog is still open
 (pnpm selects at the consumer with `catalog:<name>`).

### Catalog name and location

`rust-catalog.toml` at the repository root,
 beside `pnpm-workspace.yaml`.
Why:
 it names the content in the repo's own term ("catalog");
 `cargo-catalog.toml` reads like a cargo subcommand binary,
 and `Cargo.catalog.toml` looks like a manifest variant.
Renovate's cargo manager matches only files named `Cargo.toml`,
 so it will not misread this file.

### Feature selection

Kept:
 named catalogs;
 internal crates referenced by name (pnpm `workspace:` entries);
 adopt on sync (`pnpm add --save-catalog`);
 prune unused entries (`catalogPrune`);
 an update task writing new versions into catalog entries (`pnpm update`);
 the CI drift check;
 named diagnostics.
Omitted:
 path entries (`file:`, `link:`),
 because internal crates by name cover every path dependency today;
 the `catalogs.default` synonym;
 the `catalogMode` setting,
 because strict is decided;
 overrides through the catalog (Cargo has none in manifests; lockfile concerns go to #1801);
 delegated catalog files;
 cross-repository catalogs;
 query and run commands.
Pending confirmation:
 the affected-crate list and changesets on update
 (the question's labels were ambiguous).

### Tightening, after `catalog:tighten`

Owner direction:
 also look at the repo's `catalog:tighten` task
 (`package/dev-script/catalog-tighten/`).
It raises `>=x.y.z` floors in the JS catalog to the installed version,
 reports every entry as `TIGHT`, `OK`, `SKIP`, `UNDCL`, or `MISS`,
 and rewrites only the version token.
Its portability plan (`doc/handover/catalog-tighten-portability.md`) covers JS package managers only.
For Cargo,
 the build always uses `Cargo.lock`,
 so the lockfile is the resolved-version source
 (the JS tool avoids lockfiles because `node_modules` can be deleted while the lockfile stays).
Owner and floor rule are open.

## pnpm feature map

Read from the pnpm docs source (`pnpm/pnpm.io` `docs/catalogs.md`,
`docs/workspaces.md`,
`docs/settings/_catalogMode.mdx`,
`docs/settings/_catalogPrune.mdx`,
`docs/cli/update.md`,
`docs/cli/add.md`,
`docs/config-includes.md`,
`docs/config-dependencies.md`,
`docs/settings/dependency-resolution.md`,
`docs/filtering.md`,
`docs/errors.md`),
 cloned 2026-10-29.
Each item gives the pnpm feature,
 then its Rust mapping.

### Reference forms

- Default catalog and `catalog:` references:
   `[dependencies]` in `rust-catalog.toml`,
   use sites `{ workspace = true }` through the generated per-crate table.
- Named catalogs (`catalogs.<name>`, `catalog:<name>`):
   `[catalogs.<name>]` tables;
   pnpm's consumer-side selection needs a Cargo-sanctioned place such as `[package.metadata]`,
   because unknown keys in a dependency table warn.
   `catalogs.default` as a synonym of the default catalog,
   with an error when both are defined.
- `catalog:` in `dependencies`,
`devDependencies`,
`optionalDependencies`:
   Cargo `dependencies`,
   `dev-dependencies`,
   `build-dependencies`,
   and `target.<cfg>.*`;
   optionality is a use-site key.
   `peerDependencies` has no Cargo counterpart.
- `workspace:` ranges in the catalog (pnpm v12.2.0),
   replaced on publish (`workspace:*` to exact,
   `workspace:^` to caret,
   `workspace:~` to tilde):
   an internal crate named in the catalog renders a per-consumer relative `path` plus a `version` derived from the
   target's `package.version`;
   `cargo publish` already drops the path and keeps the version.
   Aliases (`workspace:foo@*`) map to Cargo's `package` rename key.
- `file:` and `link:` entries relative to the catalog file (pnpm v12.6.0):
   a `path` relative to the repository root,
   rendered relative to each crate.
- Location specifiers are never moved into a catalog automatically (pnpm v12.3.0),
   because one path cannot mean the same directory for every project.

### Workflow

- `catalogMode` (`manual`, `prefer`, `strict`) governing `pnpm add`:
   a mode setting in `rust-catalog.toml`;
   adding happens through `cargo add` (literal) followed by the sync adopting the literal into the catalog
   (`pnpm add --save-catalog` and `--save-catalog-name`).
- `catalogPrune` (default `false`):
   the sync removes entries no crate references.
- `pnpm update` writes new ranges to the catalog entry instead of `package.json`:
   an update task that rewrites catalog entries.
- `pnpm update --changeset`:
   no Rust crate uses changesets (`.changeset/` mentions none).
- Migration codemod (`pnx codemod pnpm/catalog`):
   the first sync adopts every existing literal requirement.

### Checks

- Frozen install fails when the lockfile's recorded catalogs differ
   (`ERR_PNPM_LOCKFILE_CONFIG_MISMATCH`):
   the CI drift check.
- `ERR_PNPM_CATALOG_ENTRY_NOT_FOUND_FOR_SPEC` and
   `ERR_PNPM_CATALOG_ENTRY_INVALID_RECURSIVE_DEFINITION`:
   named diagnostics in the check.
- Filtering counts a project as changed when a catalog version it takes changes:
   the check can list crates affected by a catalog edit.

### Not directly mappable

- `overrides` referencing the catalog,
   and convergence overrides:
   Cargo has no transitive version override in manifests;
   only the lockfile can,
   which belongs to [issue #1801](https://github.com/Aquaticat/Monochromatic/issues/1801).
- Config includes (catalogs in delegated files with their own owners),
   config dependencies and `updateConfig` hooks (catalogs shared across repositories),
   `pnpm config get catalog.<name>`,
   and `pnx <tool>@catalog:`.

### Version families

pnpm has no grouping construct:
 a family repeats its version on every entry,
 as this repo's JS catalog does for the `@deepseek-ai/dsh-*` packages.
Following pnpm,
 the Rust catalog adds none.

## Rejected

- Repository-root Cargo workspace:
   out of scope (`.out-of-scope/cargo-workspace.md`).
- Catalog as a real `Cargo.toml` virtual manifest:
   every cargo command errors on a workspace with no members,
   and `file-enforcer` discovery would treat it as a crate.
- Version-only inline rewrite:
   leaves literal requirements in every manifest.
- Opt-in and shared-only membership:
   opt-in never catches drift outside the catalog;
   shared-only leaves single-use versions unaudited.

## Open questions

- Which pnpm features to omit
   (asked as a feature list).
- Where a crate selects a named catalog.
- Whether strict membership now covers internal crates through `workspace:` entries,
   which pnpm allows but does not require.
- Where the CI check runs.
