# MPM toolchain management adoption

Status:
 accepted 2026-10-26 on the maintainer's direct instruction
 ("Migrate from Mise to <https://mpm.run> for the toolchain management part").
This decision is in force.

## Decision

Meta Package Manager (`mpm`,
 the CLI behind [mpm.run](https://mpm.run),
 [kdeldycke/meta-package-manager](https://github.com/kdeldyde/meta-package-manager)) becomes the owner of
 toolchain management:
 the tool manifest,
 machine inventory,
 outdated detection,
 upgrade coordination,
 release-age cooldown,
 and SBOM export.
Mise keeps only task running,
 `[env]`/`[vars]`,
 and secret injection;
 its `[tools]`,
 `mise.lock`,
 tool backends,
 and tool settings are removed.

Exact-pin install mechanics that MPM's adapters cannot express stay with focused repository providers
 driven by the typed tool plan,
 because MPM restore does not forward per-entry versions to every manager
 (`doc/troubleshooting/mpm-manager-extensibility.md`).
This is the "MPM plus focused native providers" composition the vet recommended,
 not a two-executable substitution.

## Authorizing request and evidence base

- Authorizing request:
 the maintainer instruction quoted in Status,
 2026-10-26.
 It is an action request that adopts MPM for this slice,
 following the `yuku-toolchain.md` precedent where a direct instruction
 waived further technology selection.
- Primary vet:
 [`doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`](../audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md),
 which found MPM plus focused providers a plausible architecture
 but left adoption conditional on a disposable pilot.
 The verification work in this migration is that pilot.
- Fresh capability confirmation:
 2026-10-26 source audit of `meta-package-manager` `v8.2.0`
 (commit `f095931a12eddc91a9f69104d4046ce998ec6887`),
 which supersedes the vet's `7.6.1` evidence.
 The `rustup` and `bin` adapters the vet recorded as unreleased 8.0 work are now released,
 and `bin` still declares no `install` operation.
 Research record:
 [`doc/research/mpm-run-capabilities.md`](../research/mpm-run-capabilities.md).
- Governing skill:
 `choosing-technology`,
 commit `37117e36397e30233062828581a5e38217da9458`,
 SHA-256 `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.
- Hard constraints:
 unchanged from the vet's frozen set
 (complete responsibility coverage or explicit retirement,
 inspectable open-source local execution,
 preserved task behavior,
 preserved pins and intentional floating requests,
 reviewable artifact provenance,
 supported local and CI platforms).
- Frozen soft criteria and weights:
 unchanged from the vet
 (separation of responsibilities 5;
 repository fit,
 human auditability,
 maintenance evidence,
 operational simplicity 1 each).
 No scoring run selects MPM;
 the maintainer instruction does.

## Adopted component

- `meta-package-manager` `8.2.0`,
 installed as the `mpm` CLI,
 consumed as an external executable whose lifecycle this repository does not own
 (`doc/decision/monorepo-manager-all-rust.md` records the same boundary for the future monorepo manager).
- Bootstrap:
 `mpm` is itself a prerequisite with a documented install path
 (standalone release binary or `uv tool install meta-package-manager`),
 pinned in the tool plan like any other tool.

## Ownership split

### Meta Package Manager owns

- The restore manifest in `mpm`'s own TOML dump format
 (`mpm.packages.toml`,
 generated from the typed tool plan).
- Machine inventory (`mpm installed`),
 outdated detection (`mpm outdated`),
 upgrade coordination (`mpm upgrade`),
 the release-age cooldown that replaces Mise's `minimum_release_age`,
 and SPDX/CycloneDX SBOM export.
- Package installs its managers can perform:
 the `pnpm`,
 `uv`,
 and `rustup` managers,
 through `mpm restore` and `mpm install`.
- For the standalone tools:
 the upstream `bin` CLI installs each from its source spec
 (the maintainer's chosen route),
 and MPM's `bin` manager then inventories,
 upgrades and removes them
 (`https://mpm.run/managers/bin/` documents that split;
 the manager itself declares no `install`).

### Focused repository providers own

The install mechanics MPM cannot express,
 invoked by the toolchain sync command from the same typed tool plan:

- `cargo-binstall` installs for the `cargo:` tools
 (feature flags,
 OS filters,
 the `monochromatic-git-policy-cli` postinstall link).
- `rustup component add` and `rustup target add`,
 which the MPM `rustup` adapter deliberately does not model
 (its packages are toolchains only).
- The Android SDK cmdline-tools and `sdkmanager` component pins,
 exporting `ANDROID_HOME` through the tool env file
 instead of Mise's vfox plugin.
- The `archive` fallback,
 downloading locked artifacts with their recorded SHA-256,
 only for sources the `bin` CLI cannot fetch.

## Maintainer confirmations (2026-10-26)

The maintainer confirmed the migration forks after the capability research landed:

- Cutover:
 remove `[tools]` and `mise.lock` entirely.
 The cutover is a hard switch that follows a successful full sync;
 the earlier draft's "anything the new system fails to provision is missing
 until fixed" assumption was retracted on the maintainer's correction.
- This machine:
 converge the maintainer's machine onto the new toolchain as the final
 verification step.
- Old installs:
 leave the Mise installs in place;
 pruning is a later decision.
- Version policy:
 keep today's semantics (floating requests with committed resolutions).
- Archive tools:
 through MPM's `bin` manager
 (`https://mpm.run/managers/bin/`):
 the upstream `bin` CLI installs from source specs,
 and MPM's manager inventories and upgrades them,
 since the manager itself declares no `install` operation.
- Cargo tools:
 repository provider via `cargo-binstall`.
- npm and pipx tools:
 MPM's `pnpm` and `uvx` manager ids,
 with boundary installs at the locked versions wherever an adapter drops the
 version parameter.
- npm "latest" semantics (maintainer 2026-10-26):
 `toolchain:update` follows the registry's `latest` dist-tag,
 so a publisher rollback can move a tool down in the update diff;
 the alternative ("highest published version") was offered and declined.
- Update flow (2026-10-26):
 `toolchain:update`,
 the `pnpm update` analogue,
 is implemented and verified in a disposable tree.
 The committed lock still holds the migration-time resolutions
 (`--check` reported `16 updated` that day);
 the maintainer triggers the real re-resolution.
- Rust pieces:
 MPM's `rustup` adapter plus direct component and target calls.
- Android SDK:
 repository provider via the Android CLI's `android sdk install`
 (the `android` command in cmdline-tools),
 not a raw `sdkmanager` invocation.

The maintainer's "no pinning" wording is recorded as:
 declarations carry no `@version` pin syntax and keep floating requests,
 while `toolchain.lock.json` fixes the installed versions
 (the `mise install` against `mise.lock` semantics),
 so explicit pins install exactly.
 Sync semantics follow pnpm (maintainer 2026-10-26):
 the sync installs the locked versions the way `pnpm install` honors
 `pnpm-lock.yaml`,
 and re-resolving versions is a separate update action.

### Mise keeps

- Task discovery and execution (`mise run`),
 task templates,
 and monorepo task namespacing.
- `[env]`,
 `[vars]`,
 and `.env.local.json` secret injection through rops.
- `mise watch` call sites until the watching row of
 `doc/planning/mise-removal-coverage.md` selects its owner.

## Tool plan and generated artifacts

`file-enforcer` owns the typed toolchain policy
 (the `doc/planning/mise-removal-coverage.md` file-enforcer provisioning row),
 in `package/file-enforcer/core/src/toolchain/`:

- `toolchain.lock.json` (generated,
 committed):
 every tool with its stable id,
 floating-or-pinned request,
 resolved version,
 route,
 provider options,
 and per-platform artifact provenance.
 It replaces `mise.lock` as the pinned version record CI reads.
- `mpm.packages.toml` (generated,
 committed):
 the `mpm`-routed slice in `mpm dump` format,
 so `mpm restore` provisions it.
- `mpm.managers.toml` (committed):
 trusted local manager definitions for pinned manager-backed tools,
 where a definition can express the pin
 (`doc/troubleshooting/mpm-manager-extensibility.md`).

The sync entry point is `toolchain:sync`
 (with `bootstrap` and `prepare:android` kept as thin aliases so existing callers keep working).

## Migration

1. Land the tool plan,
 generators,
 providers,
 and `toolchain:sync` while Mise still declares `[tools]`.
2. Seed `toolchain.lock.json` from the current `mise.lock` resolutions,
 preserving every pin,
 floating request,
 URL and checksum.
3. Provision through `toolchain:sync` in a disposable home on Linux x64
 and run representative TypeScript,
 Rust,
 and lint tasks against the provisioned tools.
4. Remove `[tools]` and tool-install settings from `mise.no-env.toml`,
 regenerate `mise.toml` with file-enforcer,
 retire `mise.lock`,
 and repoint the `bootstrap` task.
5. Repoint CI version reads
 (`task-cli-setup` inputs and the `mise.lock` scrapers in release workflows)
 at `toolchain.lock.json`.
6. Update `AGENTS.md`,
 `CLAUDE.md`,
 `README.md`,
 and the `doc/planning/mise-removal-coverage.md` ownership rows.

## Machine-global scope (2026-10-28)

The maintainer extended this decision to the machine-global toolchain
 (the `[tools]` slice of `~/.config/mise/config.toml`,
 58 declarations):
 "Migrate my global toolchain management off mise to mpm.run too".
 The ownership split,
 version policy,
 and cutover semantics are unchanged;
 only the file locations differ
 (`doc/planning/mpm-global-toolchain-migration.md` carries the shapes):

- The typed plan,
 `toolchain.lock.json`,
 and the mpm manifests live machine-locally in
 `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`,
 so no personal inventory is committed here.
 The repository still owns the tooling:
 providers,
 resolvers,
 and the `toolchain:*:global` entry points.
- Payloads install under
 `${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain-global`,
 exposed through one farm directory on `PATH`
 (a `_.path` entry in the global Mise config).
- Old Mise installs stay as the rollback,
 pruned in a later decision.

## Pruning the repository's own crates (2026-10-29)

The maintainer decided that for anything provisioned that originates from the repository itself,
 meaning a package this repository owns,
 stale versions are deleted immediately after a new version is provisioned.
This settles the "pruning is a later decision" note for those packages only;
 third-party tools and the old Mise installs stay parked as the rollback.

- Scope:
 plan entries on the `cargo-binstall` route whose crate name starts with `monochromatic-`
 (`monochromatic-lint`, `monochromatic-git-policy-cli`, `monochromatic-search-shadow`,
 `monochromatic-nested-wayland-session`).
 Both the repository and the machine-global scope apply it,
 because they share one cargo home.
- What is deleted:
 `cargo binstall` replaces the installed binary in place,
 so the stale copies are the earlier `.crate` archives in `registry/cache/<index>/`
 and the extracted trees in `registry/src/<index>/` under the cargo home
 (the shipped `cargo` manager's plain `cargo install`, which `mpm.packages.toml` routes these crates to,
 and local builds fill both).
 The same pass covers every `monochromatic-*` crate found in the registry,
 not only the plan crates:
 a plan crate keeps its locked version,
 any other own crate (a library dependency such as `monochromatic-jsonc-edit`) keeps its newest cached version.
 A crate with a longer name that shares the prefix is a different crate and is judged on its own versions.
- Why the sync owns it:
 mpm 8.2.0 has no cleanup for `cargo` (its `cleanup` exists for `pnpm` and `uv` only)
 and removes no previous version on install or upgrade,
 so a plain `mpm restore` or `mpm upgrade` leaves these copies behind.
 The repo's manager definitions in `mpm.managers.toml` declare only fixed `install` and `remove` argument lists.
- When:
 in `toolchain:sync`,
 right after `installEntry` succeeds for that crate.
 A failed install deletes nothing,
 and a crate already at its locked version is not touched,
 so copies left by earlier builds go at the crate's next version change.
- Where:
 `package/file-enforcer/core/src/toolchain/prune.ts`.
- Not covered,
 and open for the maintainer:
 stale versions of third-party tools under `<toolchainRoot>/<id>/<version>`
 and their `downloads/` archives.

## Exit and rollback

- Every step lands as its own commit series,
 so `git revert` restores the Mise-owned toolchain
 (`[tools]`,
 `mise.lock`,
 and the CI scrapers) without touching task or secret behavior.
- `mpm` removal is `mpm`-side:
 delete the manifest,
 manager definitions and sync entry,
 then reinstall the tools through any manager.
- The exercised rollback path is recorded in
 `doc/troubleshooting/mpm-toolchain-migration.md` alongside any migration workarounds.

## Revisited when

- MPM forwards restore versions to npm,
 pipx or Cargo installs
 (then the provider routes for those tools collapse into `mpm restore`).
- MPM's `bin` adapter grows an `install` operation
 (then verified standalone archives can restore through `mpm` too).
- The monorepo manager (meow) takes over task execution,
 at which point Mise removal becomes the
 `doc/planning/mise-removal-coverage.md` endpoint gate.
- The user separately resolves the floating-versus-exact-lock preference recorded there;
 this migration preserves today's semantics
 (floating requests with committed resolutions).

## Rejected alternatives

- Keeping Mise as the toolchain owner:
 rejected by the maintainer
 (documentation gap,
 2026-09-16;
 scope concentration,
 2026-09-29),
 recorded in `doc/planning/mise-removal-coverage.md`.
- MPM as a drop-in replacement with no repository providers:
 rejected because the research record shows MPM drops restore versions for npm,
 pipx,
 Cargo,
 and config-defined managers,
 and cannot express Rust components and targets,
 Cargo features,
 postinstall links,
 per-tool OS filters,
 release-tag pins,
 or Android SDK components.
- Keeping Mise for tools while MPM observes beside it
 (the composition the research record suggests as conservative):
 rejected because the authorizing request migrates toolchain management off Mise.
 MPM still observes the machine through its managers under this decision's design.
- A fork of MPM carrying full Python manager classes:
 immediate control,
 but this repository then owns rebases,
 security updates,
 and the complete adapter test surface
 (`doc/troubleshooting/mpm-manager-extensibility.md`).
- pnpm 12.4+ runtime and pipeline ownership for Node,
 Bun and CLIs:
 still experimental upstream,
 and it would rebuild the scope concentration the maintainer rejected.
- metapac,
 pacdef,
 Topgrade:
 screened out in the vet's candidate ledger.

## Evidence limits

- The provisioning verification runs on Linux x64.
 macOS and Windows provisioning parity
 (the tool acquisition parity gate in `doc/planning/mise-removal-coverage.md`)
 is not exercised by this decision and remains open.
- MPM's own release attestation is not verified here;
 the adopted version is pinned by the tool plan and installed from its published release artifact
 (same posture as the vet's checksum-plus-isolation evidence).
