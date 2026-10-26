# MPM toolchain management adoption

Status:
 accepted 2026-10-26 on the maintainer's direct instruction
 ("Migrate from Mise to https://mpm.run for the toolchain management part").
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
- Restore of the entries whose managers honor the manifest as written.

### Focused repository providers own

Every exact-pin or option-bearing install,
 invoked by the toolchain sync command from the same typed tool plan:

- `cargo-binstall` installs for the `cargo:` tools
 (feature flags,
 OS filters,
 the `monochromatic-git-policy-cli` postinstall link).
- Verified vendor-archive downloads
 (Node,
 pnpm,
 Bun,
 and the standalone binaries Mise resolved through aqua and GitHub releases),
 with the per-platform URL and SHA-256 the plan records,
 reusing the provenance `mise.lock` carries today.
- `rustup` components and Android targets,
 which the MPM `rustup` adapter deliberately does not model
 (its packages are toolchains only).
- Android SDK cmdline-tools and `sdkmanager` components.
- The Adoptium Temurin archive for the JDK,
 matching the CI path in `doc/decision/mise-free-ci.md`.

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
 in `package/dev-script/file-enforcer/src/toolchain/`:

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
