# Planning: file-enforcer for Cargo.toml duplication

Status:
 approved,
 implementation in progress.
 Sibling of `doc/planning/mise-toml-file-enforcer.md` (same duplication and
 drift problem,
 different file family).

## The problem

The repo has fifteen `Cargo.toml` files and no Cargo workspace.
 Every crate is its own workspace root,
 several with an explicit empty
 `[workspace]` table (`package/music-player/truepeak-core/Cargo.toml`,
 `package/music-player/android-app/rust/Cargo.toml`).
 The no-workspace architecture is deliberate:
 the Android crate is a Gradle island,
 standalone crates keep their own
 `target/` and `Cargo.lock`,
 and path dependencies do not require workspace
 membership.

Because there is no workspace,
 Cargo's native de-duplication mechanisms
 (`[workspace.package]` field inheritance,
`[workspace.dependencies]`) cannot
 span these islands without collapsing them into one workspace,
 which the
 architecture forbids.
 So the repo's generic generator,
 file-enforcer,
 becomes the canonical source
 instead.
 It already depends on `@monochromatic-dev/toml-edit-core` (a
 comment-preserving TOML editor) and ships `getTomlProperty` plus
 `overwriteTomlKey`,
 so property-level edits that preserve each crate's bespoke
 rationale comments are feasible today.

## Decisions

### Goal

Canonical source of truth.
 The generator rewrites each `Cargo.toml` to match a canonical definition on
 every run,
 so drift self-heals and cannot be reintroduced by hand.

### Mechanism

Comment-preserving property edits.
 `parseTomlEdit` then guarded `tomlSet` then `tomlStringify`,
 in splice mode,
 so
 every comment and unowned key survives byte-identically.
 One measured caveat drives the design:
 re-setting an already-correct value reformats it
 (`["derive"]` becomes `[ "derive", ]`),
 so the engine compares the current
 effective value against canonical and only writes on a real difference.
 Consequence:
 already-canonical files stay byte-identical,
 and the first run's diff is small.

`tomlSet`'s create path inserts a dotted top-level key before the first table
 header,
 not a `[table]` block.
 So `tomlSet` is used only to update or add keys inside tables that already
 exist;
 blocks that must be materialized where absent (`[lints.clippy]`,
 `[workspace]`) are appended as canonical block text instead.

### Owned scope

- `package.edition`:
   canonical `2024` for every crate.
   The Android crate migrates from `2021` to `2024`;
   there is no per-crate edition override.
- `package.license`:
   `LGPL-3.0-or-later`,
   present-seeded (owned only where already declared).
- `package.publish`:
   `false`,
   present-seeded.
- `[lints.clippy]`:
   canonical `disallowed_methods = "deny"`,
  `implicit_return = "deny"`,
   `needless_return = "allow"`,
   `shadow_reuse = "deny"`,
   `shadow_same = "deny"`,
   `shadow_unrelated = "deny"`.
   Universal baseline,
   inserted where the block is absent.
- `[workspace]`:
   enforced no-workspace invariant.
   An empty `[workspace]` table is inserted on every crate that lacks one,
   so no
   future ancestor `Cargo.toml` can absorb a crate.
   Existing tables (including the cargo-fuzz `members = ["."]` form) are left
   untouched.
- Shared dependencies:
   every dependency whose exact requirement is shared by two or more crates and
   has a single fleet-wide form.
   The one same-name conflict,
  `tokio` (non-optional in two crates,
  `optional`
   in `truepeak-core`),
   is excluded,
   which removes the need for per-dependency
   membership lists.
   `image`,
  `gtk4`,
   and `windows` carry genuinely divergent per-crate
   requirements and are never owned.
   Path dependencies are excluded (their path is location-relative).
- Published-crate metadata (the four crates carrying `package.repository`):
   `repository`,
  `readme`,
   and `homepage`.
   `homepage` is derived from the crate's path
   (`https://github.com/Aquaticat/Monochromatic/tree/main/<dir>`).

### Not owned

- `package.version`:
   hand-authored.
   Version is a per-crate release act with real outliers (`0.1.9`,
  `0.1.1`,
   `0.0.0`) and independent release cadences;
   the shared `0.1.0` is coincidental,
   not intentional lockstep.
- `package.include`,
  `categories`,
  `keywords`:
   `include` carries a bespoke inner comment in `forbidden-strings` and rarely
   changes;
  `categories` and `keywords` are genuinely per-crate.
- `[lib]` and `[[bin]]`:
   names are per-crate.

### Insertion policy

Universal-baseline blocks (`[lints.clippy]`,
`[workspace]`) are inserted where
 missing.
 Everything else is present-seeded:
 owned only in crates that already declare it,
 so the internal fuzz and bench
 crates that deliberately omit `license` or a release profile are left as they
 are.

### Consequence of the aggressive dependency scope

Because ownership is self-healing,
 a shared dependency can no longer be diverged
 in a single crate by editing its `Cargo.toml`;
 the next run reverts it.
 To diverge legitimately,
 remove that dependency from the canonical registry.

## Placement

- Generic engine (`manageCargoManifests`,
   compare-before-set,
   guarded keyed
  enforcement,
   block insertion):
   `package/file-enforcer/core/src/cargo/`,
   exported from the package
   index,
   with unit tests.
- Repo-specific canonical spec (edition,
   lint keys,
   dependency registry,
   profile
  presets,
   metadata):
   inline in `file-enforcer.config.ts`,
   which is exempt from the max-lines rule
   (`**/*.config.*`) and is the established home for repo file-enforcer policy
   (mirroring `manageLsp4ijServerSettings`'s data).
- Discovery:
   bounded-depth globs `package/*/*/Cargo.toml` and `package/*/*/*/Cargo.toml`
   (covering the two-level crates and the three-level Android crate) so
   traversal never descends into the deep gitignored `target/` trees (the
   terminal crate's `target/` vendors many manifests);
   a `target/` and
   `node_modules/` fragment filter stays as a safety net.

## Follow-on work

- `implicit_return = "deny"` becomes active fleet-wide.
   It surfaces under `cargo clippy` (builds and runs are unaffected) and will
   report every function that returns through a trailing expression.
   The linter crate already uses explicit `return` widely,
   so the direction is
   established,
   but a migration pass to explicit returns is a separate tracked
   task.
- `shadow_reuse`,
  `shadow_same`,
  and `shadow_unrelated = "deny"` (commit `53c1e01a9`) report every
  name-rebinding shadow across the fleet.
  The complete fallout ledger,
  one entry per finding,
  lives in [issue #604](https://github.com/Aquaticat/Monochromatic/issues/604),
  along with the Slint generated-code remedy.
- `file-manager` and `file-manager-gtk-sticky` gain
   `disallowed_methods = "deny"`,
   so `Result::unwrap` (banned by the root
   `clippy.toml`) becomes a hard clippy error there and may surface new findings.
