# Planning: shared Rust dependency catalog

Status:
 accepted 2026-10-29;
 the decision record is `doc/decision/rust-dependency-catalog.md`.
Three veto lists asked as multi-select questions came back without any answer;
 the owner then accepted every item explicitly in the two acceptance questions.
Implementation in progress.
Tracks [issue #1800](https://github.com/Aquaticat/Monochromatic/issues/1800).
Builds on `doc/planning/cargo-toml-file-enforcer.md` and respects `.out-of-scope/cargo-workspace.md`.

## Goal

One declarative source of truth for Rust dependency requirements across the independent Cargo workspace roots,
with a one-place update workflow,
a CI check that fails on drift,
documented per-crate exceptions,
and stale-entry detection,
without a repository-root Cargo workspace and without centralizing profiles.

## Proposed design (summary for acceptance)

Each item links to its decision section for evidence and rejected alternatives.

### Catalog file

- `rust-catalog.toml` at the repository root,
   in Cargo dependency syntax,
   with `[catalog]` and `[catalogs.<name>]` tables (pnpm's names).
- Registry entries are `>=` floors;
   named-catalog entries are bounded (caret or exact);
   git entries carry `git` plus `rev`.
- Supply-chain keys with pnpm's names and the repo's values:
   `minimumReleaseAge = 1440`,
   `minimumReleaseAgeExclude = []`,
   `minimumReleaseAgeStrict = true`,
   `minimumReleaseAgeIgnoreMissingTime = false`,
   `trustPolicy = "no-downgrade"`,
   `trustPolicyExclude`,
   `trustPolicyIgnoreAfter`,
   `trustPolicyExcludePrune`.
- No version-family grouping (pnpm has none).

### Crate manifests

- Each crate keeps its own `[workspace]` root and profiles.
- `file-enforcer` renders a generated `[workspace.dependencies]` table per crate holding only the catalog entries that
   crate uses;
   use sites say `workspace = true` plus crate-local `features`,
   `optional`,
   `default-features`,
   in every dependency table including `dev-dependencies`,
   `build-dependencies`,
   and `target.<cfg>.*`.
- Strict membership:
   every registry and git dependency comes from the catalog.
- A crate picks a named catalog in `[package.metadata.rust-catalog.catalogs]`.
- Internal crates stay out of the catalog and are depended on by `path`;
   the sync writes `version = "=<target package.version>"` (the `workspace:*` counterpart).

### `file-enforcer` sync and check

- Adopt on sync:
   a literal left by `cargo add` moves into the catalog as a `>=` floor and the use site becomes `workspace = true`;
   a literal conflicting with an existing entry fails instead.
- Prune catalog entries no crate uses.
- Check-only mode for CI with named diagnostics and a list of affected crates,
   run in `.github/workflows/advisory-core.yml` after `task-run prepare:pnpm`.

### Recursive update and tightening

- A mise task with inline TypeScript runs `cargo update` in every crate root (the `pnpm -r --no-save update`
   counterpart),
   with dry run,
   then enforces release age (strict, from the sparse index `pubtime`)
   and the trust policy (heuristic levels from the crates.io API,
   cached per `name@version`),
   refusing and reporting violations.
- `catalog-tighten` gains a Cargo source that raises `>=` floors to the highest version locked across consuming crates;
   it writes no lockfiles.
- The CI check re-verifies release age and trust for every locked registry version.

### Out of scope here

- Lockfile drift reporting and transitive policy:
   [issue #1801](https://github.com/Aquaticat/Monochromatic/issues/1801).
- Change intents for Rust crates:
   [issue #1802](https://github.com/Aquaticat/Monochromatic/issues/1802).
- `package/deps-update/cli`:
   treated as unusable;
   audit tracked in [issue #1803](https://github.com/Aquaticat/Monochromatic/issues/1803).

## Measured facts (2026-10-29)

Measured with a read-only scratch script that parses every first-party `Cargo.toml` through
`@monochromatic-dev/toml-edit-core` and lists every entry in `dependencies`,
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
   (`i-slint-backend-testing = "=1.18.1"` in `package/ide/app` and `package/music-player/desktop-app`;
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
   `"1.50"` in `package/ide/app`,
   `"1"` elsewhere;
   feature sets differ per crate;
   `optional = true` in `package/music-player/truepeak-core`.
- `zbus`:
   `"5.19.0"` in `package/nested-wayland-session/cli`,
   `"5"` in `package/music-player/desktop-app`.
- `windows`:
   `0.58` in `package/file-manager/app`,
   `0.62` in `package/music-player/desktop-app`;
   different majors and different feature lists.
- `slint`:
   all `1.18.1`,
   feature sets differ
   (`package/ide/app` is GPU-only with `default-features = false`).
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
   three git dependencies on one `rev` in `package/ide/app`.
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
 (`package/file-enforcer/core/src/cargo/`) with the catalog and a check-only mode.
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
 adopt on sync (`pnpm add --save-catalog`);
 prune unused entries (`catalogPrune`);
 the recursive no-save update plus tightening (see "Recursive no-save update");
 the CI drift check;
 named diagnostics;
 the affected-crate list.
Retracted after selection:
 internal crates referenced by name (see "Internal dependencies").
Follow-on:
 changesets on update.
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

### Tightening, after `catalog:tighten`

Owner direction:
 also look at the repo's `catalog:tighten` task
 (`package/dependency-catalog/cli-tighten/`).
It raises `>=x.y.z` floors in the JS catalog to the installed version,
 reports every entry as `TIGHT`, `OK`, `SKIP`, `UNDCL`, or `MISS`,
 and rewrites only the version token.
Its portability plan (`doc/handover/catalog-tighten-portability.md`) covers JS package managers only.
For Cargo,
 the build always uses `Cargo.lock`,
 so the lockfile is the resolved-version source
 (the JS tool avoids lockfiles because `node_modules` can be deleted while the lockfile stays).
Owner:
 extend `catalog-tighten` with a Cargo source,
 because it already owns floor tightening;
 `rust-catalog.toml` then has two writers (the `file-enforcer` sync and tighten),
 as `pnpm-workspace.yaml` already has pnpm plus tighten.

Floor rule:
 the highest version resolved across the consuming crates' lockfiles.
Why
 (the owner pointed out the earlier question should not have existed):
 pnpm's catalogs page lists "Maintain unique versions" as the first advantage,
 and `catalog-tighten`'s resolver (`src/version-resolve.ts`, `readInstalledVersion`) states
 "a catalog pins one version across all importers, so the first on-disk hit is the active version".
Under that one-version premise a lowest-resolved floor would contradict the catalog's purpose.

Probe (throwaway crate, repo cargo, offline):
 with `itoa` locked at 1.0.11 and the requirement raised from `1.0.4` to `1.0.15`,
 `cargo metadata --locked` failed ("cannot update the lock file ... because --locked was passed"),
 and a plain `cargo metadata` re-resolved only `itoa`,
 to 1.0.18 (the newest cached release, not the new floor),
 while `memchr` stayed at 2.6.4.
So a raised floor moves lower-locked crates to whatever is newest,
 not to the floor,
 unless tighten also updates their lockfiles.

Tighten writes no lockfiles:
 the owner rules lock writes impure for a tightener.
Lockfiles move only through the recursive update.

### Requirement operator

`>=` floors,
 like the JS catalog,
 so `catalog-tighten`'s existing `>=`-only rule applies unchanged.
Owner rationale:
 the breakage risk of crossing a major is intentional;
 falling behind upstream is the failure to surface.

Known consequence:
 `i-slint-backend-testing` must stay an exact pin equal to the resolved `slint`
 (its README requires `=` matching `slint`, noted in `package/ide/app/Cargo.toml`).
A recursive update moves `slint` to its newest release while the exact pin stays,
 so `package/ide/app` and `package/music-player/desktop-app` stop compiling their tests until the pin is raised
 by hand.
The nine `ra_ap_*` exact pins share one version and move together,
 so they do not split.

Git dependencies (the Helix crates,
 `portable-pty`) are catalog entries with `git` and `rev`;
 tighten skips them like every non-`>=` entry.

### Recursive no-save update

Support the counterpart of `pnpm -r --no-save update`
 ("Don't update the ranges in `package.json`", `docs/cli/update.md`):
 a task that runs `cargo update` in every crate root.
`cargo update` never edits manifests,
 so it is no-save by construction.
With `>=` floors it moves every lockfile to the newest releases,
 and a following `catalog-tighten` run raises the catalog floors to the highest locked versions.
This replaces the earlier "update task writes catalog entries" item:
 a saving update is the recursive update followed by tighten.

### Internal dependencies

Internal crates stay out of the catalog.
Owner direction:
 depend on them the way JS packages use `workspace:*`.
This retracts the kept "internal crates by name" item,
 which mapped pnpm's `workspace:` catalog entries (v12.2.0) rather than the repo's `workspace:*` use.

### Affected crates and changesets

The check lists the crates whose generated tables a catalog edit changes,
 for a CI that builds only affected crates.
Changesets on update wait for Rust releases to adopt change intents
 ([issue #1802](https://github.com/Aquaticat/Monochromatic/issues/1802));
 today Rust crates have no `package.json`,
 `.changeset/config.json` names only npm packages,
 and `.github/workflows/cargo-publish.yml` publishes when a crate's `Cargo.toml` `version` is bumped by hand.

### Derived from precedent (veto open)

Adopted without a separate question because pnpm's docs or the repo's JS setup decide them;
 each is offered to the owner for veto.

- Internal path dependencies:
   the sync writes `version = "=<target package.version>"`.
   Evidence:
   every internal JS reference uses `workspace:*`
   (1537 matches of `"workspace:*"` and no other `workspace:` form across `package/*/*/package.json`),
   and pnpm publishes `workspace:*` as the exact version (`docs/workspaces.md`, "Publishing workspace packages").
- Adopt on sync rewrites `cargo add`'s caret literal into a `>=` floor.
   Evidence:
   the JS catalog's entries are `>=` floors although `pnpm-workspace.yaml` sets `savePrefix: ''`,
   and the owner chose `>=` floors for Rust.
- Table names follow pnpm:
   `[catalog]` for the default catalog and `[catalogs.<name>]` for named ones.
- Named-catalog entries hold bounded requirements (caret or exact),
   and tighten skips them as it skips every non-`>=` entry.
   Evidence:
   pnpm's own named-catalog example is `react17: ^17.0.2`;
   a `>=0.58` entry would be pulled to the newest release by the first recursive update.
- The drift check runs in `.github/workflows/advisory-core.yml` after `task-run prepare:pnpm`.
   Evidence:
   that workflow runs on `push` and `pull_request` to `main`,
   and its setup action runs `pnpm install --frozen-lockfile`
   (`.github/actions/task-cli-setup/action.yml`),
   the JS catalog's drift gate.
- [Issue #1801](https://github.com/Aquaticat/Monochromatic/issues/1801) narrows:
   with `>=` floors and the recursive update,
   direct dependencies converge whenever the update runs;
   what remains is a cross-lockfile drift report and transitive dependencies.

### Release age

The repo sets `minimumReleaseAge: 1440` in `pnpm-workspace.yaml`.
On the repo's cargo (`1.101.0-nightly`),
 `cargo -Zunstable-options generate-lockfile --help` lists
 `--publish-time <yyyy-mm-ddThh:mm:ssZ>`,
 "Latest publish time allowed for registry packages (unstable)"
 (tracking issue [rust-lang/cargo#16271]);
 `cargo update --help` lists no such flag.
So a release age can bound a full re-resolution but not a selective `cargo update <spec>`.
The index `pubtime` field it relies on was stabilized in Rust 1.94.0.

The crates.io sparse index also records `pubtime` per version
 (`https://index.crates.io/an/yh/anyhow`, last line:
 `{"name":"anyhow","vers":"1.0.104","pubtime":"2026-07-18T20:59:37Z"}`),
 so the update can check every newly locked version against the release age with stable `cargo update`,
 independent of the unstable flag.

The repo's JS policy is strict:
 `minimumReleaseAge: 1440`,
 `minimumReleaseAgeExclude` (scope globs with rationale comments),
 `minimumReleaseAgeStrict: true`,
 `minimumReleaseAgeIgnoreMissingTime: false`.
`package/deps-update/cli` (`mise run deps:update`) describes itself as running
 `pnpm update --recursive --no-save` and reporting strict refusals
 (`doc/troubleshooting/pnpm-update-no-save-strict-release-age.md`).
Owner direction (2026-10-29):
 the owner did not know this package existed;
 treat it as unusable and build nothing on it.
Its report format may still inform the Rust update's diagnostics as a description,
 not as code to extend.
The owner of the Rust recursive update is therefore open.

Correction (2026-10-29,
 after the owner accepted "strict release age from `pubtime`, refusing and reporting"):
 that item misread pnpm.
pnpm's `minimumReleaseAge` "defines the minimum number of minutes that must pass after a version is published before
 pnpm will install it",
 so resolution picks the newest version old enough;
 `minimumReleaseAgeStrict` "controls how pnpm behaves when no version of a dependency satisfies the
 `minimumReleaseAge` constraint within the requested range":
 `true` fails,
 `false` falls back to a younger version
 (`docs/settings/dependency-resolution.md`).
Adopted under rule `EPR`:
 the update runs `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>` in each
 crate root,
 a resolver-level constraint with pnpm's semantics,
 and a full re-resolution matches the update now that selective updates are omitted.
Packages in `minimumReleaseAgeExclude` are then moved to their newest release with `cargo update <name>`,
 and every newly locked version is checked against `pubtime`;
 a young version that is not excluded fails the run.
With strict off,
 a failed constrained resolution falls back to an unconstrained one.
The sparse index carries `pubtime` back to old releases
 (`anyhow` 0.0.0 shows `2019-10-05T19:52:26Z`),
 so `minimumReleaseAgeIgnoreMissingTime` rarely applies.
`minimumReleaseAgeExcludePrune` (pnpm default `false`) joins the kept settings as the sibling of
 `trustPolicyExcludePrune`.

### Trust policy

Owner correction:
 crates.io does carry trust evidence,
 so a pnpm-style `trustPolicy` can work through heuristics.
pnpm's `trustPolicy: no-downgrade` (set in this repo's `pnpm-workspace.yaml`) fails "if a package's trust level has
 decreased compared to previous releases",
 judged by publish date,
 ignoring prereleases for a stable install;
 `trustPolicyExclude`,
 `trustPolicyIgnoreAfter`,
 `trustPolicyExcludePrune`,
 and `trustLockfile` refine it (`docs/settings/dependency-resolution.md`).

Per-version evidence from `https://crates.io/api/v1/crates/<name>/versions`:

- `trustpub_data`:
   set when the version was published through Trusted Publishing,
   with `provider`,
   `repository`,
   `run_id`,
   and `sha`
   (present on the newest `libc` and `cc` releases,
   `rust-lang/libc` and `rust-lang/cc-rs`;
   null on the newest `anyhow`,
   `tokio`,
   and `clap`,
   which carry a `published_by` user instead).
- `published_by` and `audit_actions`:
   the publishing account.
- `repository`:
   the crate's declared repository URL for that version.

The lockfiles hold 1388 distinct registry `name@version` pairs across 1024 crate names
 (measured 2026-10-29),
 so whole-lockfile verification needs cached API results:
 the crates.io data-access page allows "a maximum of 1 request per second" with an identifying user-agent,
 while "No rate limits are required to use data from the sparse crate index".

Decided (owner, 2026-10-29):

- Levels:
   Trusted Publishing (`trustpub_data` present) is highest;
   among token publishes,
   an account that published an earlier release ranks above a first-time publisher.
   A release ranking below any earlier-published stable release fails (`no-downgrade`).
- A trusted-publishing release whose `trustpub_data.repository` differs from earlier trusted releases is a downgrade.
- A release whose declared `repository` differs from the previous release fails unless excluded.
- Kept settings:
   `trustPolicyExclude`,
   `trustPolicyIgnoreAfter`,
   `trustPolicyExcludePrune`,
   and re-verification of every locked entry in the CI check (pnpm's default `trustLockfile: false`).

### Owner of the recursive update

A mise task written in TypeScript (owner decision).
The repo's task rule allows two forms:
 inline logic through `shell = "node --input-type=module-typescript -e"`,
 or a package bin the task runs;
 which form is open.

### Update selection features

Omitted (owner decision):
 patterns and negation,
 crate filter,
 direct-only,
 dependency-kind filters,
 interactive picker.
Kept:
 release age and dry run.

[rust-lang/cargo#16271]: https://github.com/rust-lang/cargo/issues/16271

### Named catalog selection

A crate selects a named catalog in its own manifest,
 under `[package.metadata.rust-catalog.catalogs]`
 (for example `windows = "windows-legacy"`).
Why:
 pnpm selects at the consumer (`catalog:<name>`),
 and the selection moves with the crate.
Cargo's manifest reference says the `package.metadata` table "is completely ignored by Cargo and will not be warned
 about".

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

### Out-of-scope formatting left in the worktree (2026-10-29)

While linting the catalog code,
 a package-wide `mise run //package/file-enforcer/core:format:oxlint` reformatted files outside this work.
Owner decisions:

- The 25 files under `package/file-enforcer/core/src/toolchain/` and `src/package/` that were unmodified
   before this session stay modified (formatting only);
   their diff is saved at `${HOME}/temp/agent/rust-catalog/autofix-outside-scope.patch`.
- `src/toolchain/plan.artifacts.ts` and `src/toolchain/plan.tools.ts` already held another session's uncommitted
   work;
   they now also carry this autofix's formatting and are left as they are.
Later autofix runs snapshot `src/` first and copy every file outside the catalog work back byte for byte.

The autofix also changed meaning in one of the 25 files:
 `src/toolchain/update.ts` had a typed arrow `const fetcher: ... = async (url, init) => ...`,
 rewritten as `async function fetcher(url, init)`,
 so `mise run //package/file-enforcer/core:lint:types` fails with TS7006 at `update.ts:632` and `:633`.
Told this,
 the owner still chose to leave all 25 files as they are.
Type-checking the catalog code therefore reads that task's output for errors in `src/cargo/catalog/`,
`src/check-mode.ts`,
`src/cli.ts`,
and `src/index.ts`,
 which report none.

### Cargo 1.98 regression from the migration (2026-10-29)

The migration commit `fd0bd03540` broke cargo 1.98.0 (the `localhost/monochromatic/ide` image),
 blocking the IDE 0.1.3 release:
 Edition 2024 member sites with `default-features = false` over a generated entry that kept default features
 fail with "`default-features = false` cannot override workspace's `default-features`".
Host nightly 1.101 accepts them because rust-lang/cargo#17126 (cargo 1.99) allows the override,
 so the earlier `memchr` probe in "Open questions" passed on the wrong toolchain to catch it.
Full trace: `doc/troubleshooting/cargo-1-98-inherited-default-features.md`.

Fix, commit `d1477a13ba`:
 a crate's generated entry carries `default-features = false` when every catalog site of that dependency in the crate
 writes it,
 and sites that disagree fail the sync with a named issue
 (`package/file-enforcer/core/src/cargo/catalog/default-features.ts`).
Verified with `cargo metadata --locked` for all seven affected crates on host nightly and on cargo 1.98.0,
 with the pre-fix ide manifest still failing on 1.98.0 as the control.

Owner correction (2026-10-29):
 "we should always use nightly rust for everything!"
The 1.98 cargo came from `package/terminal/app/Containerfile` installing floating `--default-toolchain stable`
 (the IDE image is built `FROM localhost/monochromatic/terminal`);
 `nested-wayland-session` and the music-player Linux image did the same.
Commit `8652ba7d7f` makes each of them install `ARG RUST_TOOLCHAIN`,
 defaulting to the `toolchain.lock.json` pin (`nightly-2026-10-29`),
 the pattern `package/music-player/desktop-app/Containerfile.windows-gnu` already used
 (its default had drifted to `nightly-2026-10-21` and was bumped).
So the catalog targets the pinned nightly only;
 the `default-features = false` rendering stays,
 since it is also what pre-2024 editions need on every cargo.
The `ARG` defaults are synced by hand,
 which is how the windows-gnu default drifted.
`package/forbidden-strings/core/mutants.Containerfile` still uses `FROM docker.io/library/rust:1`;
 it sits in another session's uncommitted rename batch,
 which was asked to switch it when the batch lands.

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

- Verified 2026-10-29 (repo cargo, throwaway crate and throwaway worktree):
   `memchr = ">=2.6.4"` in a crate's own `[workspace.dependencies]`,
   used as `{ workspace = true, default-features = false, features = ["alloc"] }`,
   resolved with only `alloc`,
   and `cargo package` kept `version = ">=2.6.4"`;
   `cargo update --dry-run` ran on the host in `package/music-player/desktop-app` and
   `package/file-manager/variant-qt` with only the registry index
   (no Podman environment).
- Name of the mise task and of its package
   (decided:
   a package bin under `package/dev-script/` run by a thin mise task).
- Decided (owner, 2026-10-29):
   the trust cache is local and gitignored,
   `.cache/rust-catalog-update/crates-io/<name>.json`,
   holding per version only `num`,
   `created_at`,
   the publisher id,
   the trusted-publishing repository,
   and the declared repository;
   CI restores and saves it with the pinned `actions/cache/restore` and `actions/cache/save` steps
   that `.github/workflows/music-player-release.yml` already uses,
   keyed on a hash of every `Cargo.lock` with a prefix restore key.
  Measured 2026-10-29:
   the 24 lockfiles hold 1024 registry crate names with 46,593 published versions
   (local sparse-index cache; `clap` has the most, 462);
   `https://crates.io/api/v1/crates/clap/versions` returns all 462 in one response (`meta.next_page` is null),
   so a cold verification is 1024 requests,
   about 17 minutes at 1 request per second.
  GitHub removes cache entries "that have not been accessed in over 7 days",
   so a cold run recurs after a quiet week.
  Rejected:
   a committed evidence file (about 4 to 5 MB estimated, regenerated on every update,
   and CI would check the committed copy instead of crates.io)
   and no cache (17 minutes on every run).
- Re-probed 2026-10-29 on `cargo 1.101.0-nightly (3960fa69b 2026-10-12)`:
   `cargo generate-lockfile --help` lists `--publish-time` and `cargo update --help` does not,
   with or without `-Zunstable-options`,
   so the update keeps full re-resolution through `generate-lockfile`.

## Status and next actions (2026-10-29)

Landed:
 `module-rust-catalog` (catalog parser and the shared `Cargo.lock` reader),
 the `file-enforcer` sync and `--check` mode with the per-crate `default-features` rendering,
 the migration of all 24 manifests,
 the Cargo source of `catalog-tighten`,
 `package/dependency-catalog/cli-rust-update` (update, dry run, CI `--verify`, exclude pruning, tests, README),
 the pinned-nightly build images (`8652ba7d7f`),
 and the contributor runbook `doc/runbook/rust-dependencies.md`.

The first real dry runs (systemd units, throwaway worktree) failed in their first crate
 with "the `-Z` flag is only accepted on the nightly channel":
 the host's systemd user manager exports `RUSTUP_TOOLCHAIN=1.98.1`
 (`systemctl --user show-environment`; rustup's own default is nightly,
 and neither `~/.config/environment.d`, the shell profiles, nor the global mise config set it,
 so its origin is still unknown).
Commit `8620661b05` first let an inherited `RUSTUP_TOOLCHAIN` win, which kept the stale stable pin;
 the follow-up makes the `toolchain.lock.json` nightly pin win and logs a differing inherited value.
Both failures restored every lockfile, as designed.

The third run resolved all 24 crates on the pinned nightly and refused 6 first-publisher releases
 ([issue #1806](https://github.com/Aquaticats/Monochromatic/issues/1806));
 the owner approved all six (`trustPolicyExclude`, commit `f722a78ea5`)
 and amended the trust policy (commit `81fbb6ae68`):
 a listed individual crate owner counts as a known publisher,
 and only a change between two declared repositories fails.
Re-checked against crates.io, only `wayland-server@0.31.14` (published through a team owner) would still be refused.
The fourth run passed:
 161 newly locked versions of 161 crates,
 every check clean,
 every lockfile restored (dry run).
The host's `RUSTUP_TOOLCHAIN=1.98.1` stays on purpose as a canary for tools that do not pin the toolchain
 (owner decision, [issue #1805](https://github.com/Aquaticats/Monochromatic/issues/1805)).
The warm crates.io history cache (161 crates) is in the main checkout's gitignored `.cache/rust-catalog-update/`.

Landed after the package moves (the update package is now `package/dependency-catalog/cli-rust-update`,
 file-enforcer is `package/file-enforcer/core`):

- `rust:update` and `rust:verify` tasks in `mise.no-env.toml` and in root `task.ts`
   (`task-run` reads `task.ts`, not `mise.toml`), commit `328e2421b2`;
   `pnpm-lock.yaml` with the importer landed in the other session's batch.
- CI steps in `.github/workflows/advisory-core.yml` after `task-run prepare:pnpm` (commit `ed7a8036c2`).
   `prepare:pnpm` already runs the file-enforcer sync (`node file-enforcer.config.ts`),
   so the drift step checks with `git diff --exit-code` that the sync left every crate manifest and `rust-catalog.toml`
   unchanged;
   a `file-enforcer --check` at that point would compare against files the sync just rewrote and could never fail.
   Control in a throwaway worktree: clean tree exit 0, an edited generated entry exit 1 naming the file.
   `rust:verify` runs between `actions/cache/restore` and `actions/cache/save` of `.cache/rust-catalog-update/`.

The first full `rust:verify` (1388 versions of 1024 crates) got HTTP 429 from crates.io after 657 API requests
 spaced exactly 1000 ms apart.
crates.io's data-access policy lists the API last, after the sparse index, static.crates.io,
 and the database dump (`https://static.crates.io/db-dump.tar.gz`, 2,103,407,089 bytes on 2026-10-29, rebuilt daily,
 no rate limit),
 and allows the API "a maximum of 1 request per second".
Owner decision (2026-10-29): fill the trust cache from the dump (commit `d6954e5a7f`).
Measured for the design:

- Dump layout (dump `2026-10-29-020020`, crates.io commit `55d3c7bf0d5d0ffc2fc6217d7ce0f97cc798ae02`):
   tar order `crates.csv`, `crate_owners.csv`, `versions.csv`, then `dependencies.csv` (1.8 GB)
   and `version_downloads.csv` (0.96 GB).
   Public: `versions.{crate_id,num,created_at,published_by,repository}`, `crate_owners`, `users`,
   and `crates.trustpub_only` (134 locked crates).
   Private: `versions.trustpub_data` and every `trustpub_*` table (`dump-db.toml` at that commit).
- Across 785 API histories (37,822 releases, 1,470 Trusted Publishing) no Trusted Publishing release has a publisher;
   the earliest is 2025-07-12T16:49:05Z,
   so a crate with a publisher-less release since 2025-07-01 is left to the API.
- On the real dump: 1024 of 1024 locked crates found, 883 histories written, 141 left to the API,
   22 seconds including gunzip, stopping after `versions.csv`.
- Dump against API on 660 crates and 23,813 releases: identical version sets, times, and publishers;
   9 old `dashmap` releases list a repository in the dump and none in the API,
   which the declared-to-declared repository rule does not count.
- The tar and CSV readers are hand-written (owner chose this over vetted libraries).
- The API now spaces requests 1100 ms apart and retries HTTP 429 after `Retry-After`
   or a doubling 30-second backoff, at most five times.

Cold full verify (2026-10-29, systemd unit `rust-verify-cold`, empty cache):
 914 seconds and 187 crates.io API requests for 1388 locked versions of 1024 crates,
 inside the 25-minute CI step budget.
It exited 1 with 41 trust findings on versions already in committed lockfiles
 (the earlier dry runs checked only newly locked versions, and the earlier warm full run stopped at HTTP 429):
 22 trust downgrades (mostly a first-time publisher after a known one)
 and 19 repository changes (organization moves such as `RazrFalcon` to `linebender`,
 backports published after a newer line such as `heapless@0.7.17` after `0.8.0`,
 and `http` to `https` for `transpose`).
By publish age: 38 older than 30 days, 29 older than 180 days, 20 older than 365 days, 11 older than 730 days.
The list with dates is in `${HOME}/temp/agent/rust-catalog/cold-verify.log`.
The API-rich cache was restored over the dump-filled one afterwards.

pnpm precedent read for this (pnpm `pnpm11/resolving/npm-resolver/src/trustChecks.ts`
 and `pnpm11/docs/settings/dependency-resolution.md`, 2026-10-29):
 the trust check runs inside version picking (`pickWithoutTrustDowngrade`),
 and, separately, `trustLockfile` (added in v11.3.0, default `false`) makes `pnpm install`
 "re-apply `minimumReleaseAge` and `trustPolicy` to every entry in the loaded lockfile";
 its docs reserve `trustLockfile: true` for "closed-source projects where every commit comes from a trusted author".
 (Correction: the first question to the owner said pnpm never re-checks locked versions.
 That was wrong, and the owner re-decided with the corrected fact.)
 History is ordered by publish date, not semver ("Trust checks are based solely on publish date, not semver");
 `trustPolicyIgnoreAfter` skips versions published more than that many minutes ago;
 and a picked version that fails is set aside and the picker retries the next candidate (up to 1000 times)
 instead of failing the install.
`rust-catalog-update` does not yet set aside and repick; it restores every lockfile instead.

Owner decisions (2026-10-29, question tool):

- Verify scope: "New versions only", kept after the correction:
   the owner "never had and never will accept PRs containing a lockfile change",
   so every committed lockfile comes from the owner's own sessions,
   which is the trusted-base case pnpm's `trustLockfile: true` exists for.
   `--verify --base <rev>` checks only registry versions that some `Cargo.lock` holds and no `Cargo.lock` held at `<rev>`
   (a union across lockfiles, as the update's own new-version check uses; a crate whose lockfile is new counts in full).
   Commits auto-push straight to `main`, so a merge base with `main` would always be empty there;
   CI passes the push's previous commit (`github.event.before`) as the base,
   and fetches that one commit.
   Landed in `161cb34c54`; verified locally (0 versions against `HEAD`, 691 versions and 13 findings against a base from before 2026-09-01).
   Known limit: after a force-push `github.event.before` may be unreachable, so the step fails
   (`continue-on-error` keeps the workflow green).
- `trustPolicyIgnoreAfter = 525600` (365 days) in `rust-catalog.toml`.
- Toolchain plan data: "Add a *.data.ts pattern".
   `plan.tools.ts` and `plan.artifacts.ts` are data, not code ("the 300 line limit doesn't apply to non-code"),
   so they become `plan.tools.data.ts` and `plan.artifacts.data.ts`,
   a `**/*.data.ts` override in `package/config/oxlint/src/overrides.ts` turns `eslint/max-lines` off,
   and neither file is split.
   The renderer emits one property per line and fixes the generated file's other findings.

Adopted from pnpm without asking (its documented behavior settles it):
 when a newly resolved version fails the trust check, the update sets it aside and re-resolves that crate
 with the version excluded, up to a bound, before failing.
 Queued after the verify scope change.

Done since the cold run:
 the cold verify (log `${HOME}/temp/agent/rust-catalog/cold-verify.log`) and the restore of the API-rich cache;
 the verify scope and ignore-after (`161cb34c54`);
 the `**/*.data.ts` lint override in `package/config/oxlint`;
 in `package/file-enforcer/core`, the `fetcher` type fix (`a6b16edc2a`),
 the zero-width-space and readonly-tuple fixes,
 and every non-toolchain test importing the built entry (`53525c50c6`),
 which took the package from 452 to 360 lint errors.
#1800's acceptance criteria are met, so it closes with this status update;
 the items below are tracked here and in the session task list.

Remaining:

1.  Confirm the new CI verify step on a real `advisory-core` run:
     its log should show `comparing lockfiles with <sha>` and `verifying N newly locked versions`.
2.  Set aside a trust-failing version and re-resolve (adopted from pnpm), `Refs #1800`:
     split `checkTrust` into a structured core returning the refused `name@version` with its problems;
     candidates are older releases of that crate, caret-compatible with the refused version,
     published before the release-age cutoff, not yanked (from the sparse index, a per-run `yanked(name)` on `Registry`,
     not the cached history format), not already set aside, and passing the trust policy against the full history;
     try them newest first with `cargo update --package <name>@<version> --precise <candidate>` in each crate that locks it;
     re-run both checks after each round (transitive versions move), with bounded rounds and bounded attempts;
     keep the original finding when nothing works; report each set-aside as a warning in dry and real runs.
     Needs a small tested version comparator (prerelease and `+build` metadata, for example `0.11.1+wasi-snapshot-preview1`),
     a fake cargo runner that understands `--precise`, and a test that fails with the set-aside removed.
     Check `cargo update --help` and the Cargo Book for how `--precise` treats yanked versions first.
3.  Rename the toolchain plan data to `plan.tools.data.ts` and `plan.artifacts.data.ts`
     (renderer, loader, patcher, tests, and the global copy in `~/.config/monochromatic-dev/toolchain/`),
     make the renderer's output lint-clean, then tell the `monochromatic-db` session, which runs `toolchain:update`.
4.  The other `src/toolchain/*` lint findings (357 errors and 267 warnings after the rename `71ff15994f`).
     `providers.ts` (1557 lines) and `update.resolvers.ts` (1521 lines) are code, so they still split into sibling modules.
     In progress (2026-10-29): three helper agents each own one disjoint cluster, edit only it, never change a signature
     used outside it, and leave changes uncommitted for review:
     the update cluster (`update.resolvers.ts`, `update.render.ts`, `update.ts`, their tests),
     the providers cluster (`providers.ts`, `container-runner.ts`, `exec.ts`, `mise.container-test.ts`, provider tests),
     and the sync/status cluster (`status.ts`, `sync.ts`, `layout.ts`, `exposure.ts`, `mpm-manifest.ts`, `bin-paths.ts`,
     `generate.ts`, their tests, `src/package/mise.container-test.ts`).
     This session keeps `types.ts`, `paths.ts`, `plan.ts`, `plan.load.ts`, `lock.ts`, `src/index.ts`,
     the toolchain tests' `require-eventual-artifact` imports, and any cross-cluster signature change the agents report.
     `lock.ts` and `plan.ts` are done (0 findings).
     All three clusters landed (`e5cc3750e4` providers, `a11dc18d42` sync/status, `fca18e62d3` update,
      plus `66f…` making the bin list row scan linear after the owner rejected a quadratic replacement:
      "quadratic here isn't acceptable");
      117 findings remained afterwards, all cross-cluster or in this session's files.
     Owner decisions (2026-10-29):
      lock absence is written by omitting the key ("Omit absent keys"), so `manager`, `packageId`, `os`,
      and `bootstrap.requiresTool` become optional and the plan data drops its `null` literals;
      none of the three outside readers uses those fields (checked: core-matrix reads only `.tools.node.artifacts`);
      `bin-paths.ts` keeps Node built-ins only and takes one justified scoped disable for `prefer-caught-value-text`;
      AGENTS.md MXL stays as written ("Leave MXL as is").
     Former open question for the owner:
      `types.ts` mirrors `toolchain.lock.json`, which holds 120 JSON `null` values
      (30 `manager`, 30 `packageId`, 59 `os`, 1 `requiresTool`)
      and is read outside file-enforcer (`package/test-harness/core-matrix`, `package/ide/app/bin/inspect-language-toolchain.ts`,
      `package/translation-repair/typography`);
      `no-nullish-union` either becomes optional fields with omitted lock keys (a format change for those readers)
      or keeps `null` in the on-disk mirror type behind a documented scoped disable.
     `paths.ts`'s `isMainModule` uses `realpathSync` at every toolchain entry point's top level,
      so its `no-sync` fix changes callers in all three clusters.
5.  Open, not fixed:
     the overwrite, TOML-write, JetBrains, and Mise-manifest suites time out on
     `node_modules/.cache/file-enforcer/staleness-manifest.json.lock` when the whole suite runs in parallel,
     and `src/io/write.unit.test.ts` passes alone; there is no baseline from before the import change,
     and tests sharing one real lock path is itself a defect to propose to the owner.
     The toolchain plan and generator tests still expect `nightly-2026-10-28` and node `26.12.1`.
