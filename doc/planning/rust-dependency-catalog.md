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

Pending;
 see the open questions.

## Open questions

Recorded once the precedent research returns and the owner answers.
