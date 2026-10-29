# MPM toolchain migration: recorded workarounds and rollback path

## Scope

This file collects migration workarounds observed while toolchain management
moved from Mise `[tools]`,
`mise.lock`,
`mise install`,
and `mise upgrade`
to Meta Package Manager plus repository providers
(`doc/decision/mpm-toolchain-adoption.md`,
`doc/planning/mpm-toolchain-migration.md`).
Each workaround names the emitting tool and its verbatim message.
The file also records the rollback path the adoption decision requires here.
Live provisioning instructions live in `README.md`,
and ownership lives in `doc/planning/mise-removal-coverage.md`.

## Workarounds observed

Four workarounds were observed while the toolchain implementation landed on
2026-10-26
(`doc/planning/mpm-toolchain-migration.md`,
section `Implementation record (toolchain-core, 2026-10-26)`).
Each entry names the emitting tool and its verbatim message,
and later entries follow the same shape.

### `bin` first run prompts for a download directory and exits 0 on EOF

Surface:
 the first `bin` invocation during `toolchain:sync` staging on a machine with
 no `bin` configuration.

Emitting tool and message:
 `bin` (marcosnils/bin 0.29.4) prompts `Pick a default download dir:` and,
 on EOF,
 prints `⨯ Error loading config file Invalid input`
 while exiting 0 with nothing installed.
 An empty `$HOME/.bin/config.json` also silently shadows
 `$XDG_CONFIG_HOME/bin/config.json`,
 which the same resolution order explains
 (`BIN_CONFIG`,
 `$HOME/.bin/config.json`,
 `$XDG_CONFIG_HOME/bin/config.json`,
 `$HOME/.config/bin/config.json`).

Workaround:
 the sync pre-seeds `{"default_path": ..., "bins": {}}` at the configuration
 path `bin` resolves
 (`seedBinConfig` in `package/file-enforcer/core/src/toolchain/providers.ts:206`).
 Tradeoff:
 an existing configuration file is left alone,
 so an empty `$HOME/.bin/config.json` still wins and a human must remove it.

### `bin install` prompts for an asset on multi-file archives

Surface:
 `bin install` of a source whose release archive holds more than one file.

Emitting tool and message:
 `bin` (marcosnils/bin 0.29.4) prompts
 `Showing 7 assets out of 7. Select an option`
 and exits 1 on EOF.
 It also re-downloads the release asset on every invocation.

Workaround:
 one `bin install` per primary binary with `-n` fallback globs
 (`bin install <release-tag-url> -n '<asset>/**/<bin>'`),
 and the single-binary payload routing rule sends multi-binary or whole-tree
 payloads to the `archive` provider instead.
 Tradeoff:
 `bin` covers only single-binary payloads,
 and one archive holding several tools needs several `-n` fallbacks.

### MPM 8.2.0 drops restore versions for configuration-defined managers

Surface:
 `mpm restore` of a versioned entry through a trusted local manager definition.

Emitting tool and message:
 `mpm` 8.2.0 warns
 `Configuration-defined managers do not support version pinning. Letting the package manager choose the version.`
 on each such entry
 (`doc/troubleshooting/mpm-manager-extensibility.md`).

Workaround:
 only `--version` literals or package ids carrying the pin survive,
 so `mpm.managers.toml` bakes each `cargo:` pin into a `--version` argument
 and npm tools stay provider-owned at the repository boundary
 (`package/file-enforcer/core/src/toolchain/mpm-manifest.ts`).
 Definitions belong under `[mpm.overrides.<id>]` in 8.2.0;
 the `[mpm.managers.<id>]` path of the 7.6.1 record is historical.
 Tradeoff:
 one definition cannot forward each restore entry's version field,
 so the pin lives in the generated definition and in `toolchain.lock.json`.

### The taplo lock row carried a blake3 digest

Surface:
 seeding `toolchain.lock.json` artifact provenance from `mise.lock`.

Emitting tool and message:
 the `mise.lock` taplo row for `linux-x64` recorded
 `checksum = "blake3:4871fab0e60275a1eb46e7190726e144f56c9a9527f59b0d1da5a042baead8e2"`
 while its sibling rows recorded `sha256` digests.

Workaround:
 `toolchain.lock.json` records the SHA-256 of the same artifact bytes
 (computed 2026-10-26),
 so every artifact row is a checkable SHA-256
 (`package/file-enforcer/core/src/toolchain/plan.artifacts.ts:12`).
 Tradeoff:
 the digests no longer match `mise.lock` row for row,
 so comparing the two records needs the artifact bytes.

### Stale Mise shims shadow unprovisioned tools

Surface:
 running a repository task on a machine that still carries the old Mise shim
 directory on `PATH` before `toolchain:sync` has populated
 `.cache/toolchain-bin`
 (observed 2026-10-26 mid-sync,
 `mise run lint:monochromatic-lint`).

Emitting tool and message:
 `mise` shim resolution,
 which then exits non-zero:

```text
mise ERROR No version is set for shim: monochromatic-lint
Set a global default version with one of the following:
mise use -g cargo:monochromatic-lint@0.1.0
```

Workaround:
 run `mise run toolchain:sync` first,
 which links every declared binary into `.cache/toolchain-bin`;
 that directory precedes the shim directories in the generated `_.path`.
 Alternatively drop the retired shim directory from `PATH`.
 Tradeoff:
 until the sync completes,
 tasks on such machines resolve through the retired shims and fail with this
 diagnostic instead of a missing-binary error;
 the exposure directory is also rebuilt during a sync,
 so tasks racing a running sync can see the same window.

### `mpm` warns "Unrecognized Linux distribution" on Bazzite

Surface:
 any `mpm` invocation on Bazzite
 (a Fedora Kinoite derivative,
 `/etc/os-release` reports `NAME="Bazzite"`,
 observed 2026-10-26 with mpm 8.2.0).

Emitting tool and message:
 `mpm`,
 printed before the command output while still exiting 0:

```text
Unrecognized Linux distribution: Environment:
  sys.platform:          'linux'
```

Workaround:
 none is required;
 the toolchain's routes never rely on `mpm`'s platform-manager adapters,
 and `mpm installed`,
 `mpm outdated`,
 `mpm sbom`,
 and `mpm restore` all work.
 Tradeoff:
 `mpm`'s brew,
 dnf,
 and similar system adapters stay unavailable on unrecognized derivatives,
 so a future route that needs one must extend `mpm`'s platform map or
 call the manager directly.

### Leftover Mise plugins keep `mise self-update` in the toolchain pass

Surface:
 `mise self-update` or `mise plugins update` on a machine that still holds
 pre-cutover Mise plugins after the toolchain cutover
 (observed 2026-10-27 with the `cmake` vfox plugin set that the removed root
 `cmake = "latest"` declaration pulled in).

Emitting tool and message:
 `mise` 2026.10.20,
 which updates installed plugins unless `--no-plugins`:

```text
mise ERROR Failed to update plugins: cmake, vfox-cmake, vfox-jdx-vfox-cmake
```

 The plugin update's forced checkout then collides with the `git` wrapper's
 `linked-worktree-only` policy
 (`doc/troubleshooting/mise-plugin-update-linked-worktree-guard.md`).

Workaround:
 remove the unused plugins
 (`mise plugins uninstall --yes cmake vfox-cmake vfox-jdx-vfox-cmake`,
 applied 2026-10-27),
 or pass `mise self-update --no-plugins`.
 Tradeoff:
 the recorded rollback path reinstalls plugins on demand,
 and a worktree still declaring a plugin-backed tool
 (the pre-cutover `tr-*` worktrees carry `cmake = "latest"`)
 resurrects them,
 which brings the failure back while the exposed `git` wrapper predates
 `monochromatic-git-policy-cli` 0.2.9.

## Rollback path

The adoption decision requires the rollback path to be recorded here
(`doc/decision/mpm-toolchain-adoption.md`,
section `Exit and rollback`).

1. Each migration step lands as its own commit series,
   so `git revert` of the cutover commits restores the Mise-owned toolchain
   (`[tools]`,
   `mise.lock`,
   and the `mise.lock` CI scrapers)
   without touching task,
   `[env]`/`[vars]`,
   or secret behavior.
2. With `[tools]` and `mise.lock` back,
   `mise install` and `mise upgrade` provision tools again.
   `mise run` task behavior never changed across the migration,
   so tasks run in both states.
3. `mpm`-side removal is `mpm`-side:
   delete the restore manifest (`mpm.packages.toml`),
   the manager definitions (`mpm.managers.toml`),
   and the sync entry (`package/file-enforcer/core/src/toolchain/`),
   then reinstall the tools through any manager.

Exercise status:
not yet exercised as of 2026-10-26.
The removal gates of `doc/planning/mise-removal-coverage.md` require the
rollback path to be exercised before the Mise files are deleted at the
removal endpoint,
and this entry records the exercise when it runs.

## Verification

The migration's verification gates
(`doc/planning/mpm-toolchain-migration.md`,
section `Verification`)
are:

1. `mise run toolchain:status` clean on a machine after a sync.
2. A disposable home (`mktemp -d`,
   `HOME=` and `XDG_DATA_HOME=` overrides,
   container with 2 GiB and 2 CPUs)
   syncs the Linux x64 slice,
   then runs one TypeScript task,
   one Rust task,
   and one lint task through `mise run`.
3. `mpm installed` and `mpm outdated` list the manager-backed tools.
4. No synthetic host load,
   and every probe reports the command,
   working directory,
   and exit status.

## What does not work

- Restoring every entry through `mpm restore` alone:
  MPM drops restore versions for some managers
  and cannot express Rust components and targets,
  Cargo features,
  postinstall links,
  per-tool OS filters,
  release-tag pins,
  or Android SDK components
  (`doc/research/mpm-run-capabilities.md`,
  `doc/troubleshooting/mpm-manager-extensibility.md`).
  The confirmed per-family routes
  (`doc/planning/mpm-toolchain-migration.md`,
  section `Routing (user-confirmed 2026-10-26)`) cover those entries;
  that is the adopted design,
  not a migration workaround.
- Running the sync entry without Node:
  `node package/file-enforcer/core/src/toolchain/sync.ts` is TypeScript
  executed by an already-present `node`.
  Node and Mise stay the manually installed bootstrap prerequisites.
  The sync then stages `bin`
  (a standalone release binary,
  SHA-256 checked against the release `checksums.txt` values),
  `uv`,
  and `mpm`
  (the PyPI wheel `meta-package-manager` 8.2.0,
  SHA-256 checked against PyPI's published digest,
  installed with `uv tool install`).

## Upstream filing decision

Nothing to add upstream from this record.
The external-tool gap that shaped the composition
(restore version drops and the bounded TOML manager definitions)
carries its own audit and filing decision in
`doc/troubleshooting/mpm-manager-extensibility.md`.
This file records no new external-tool defect,
so there is no new-issue draft here.

## Related records

- `README.md`,
  section `Initial setup`:
  the live provisioning and inventory commands.
- `doc/planning/mise-removal-coverage.md`:
  ownership rows,
  parity gates,
  and the removal endpoint.
- `doc/planning/mpm-toolchain-migration.md`:
  the interface spec and verification gates.
- `doc/troubleshooting/mise-rust-components.md`,
  `doc/troubleshooting/mise-cargo-backend-needs-rust-installed.md`,
  and the other provisioning-half `doc/troubleshooting/mise-*.md` records:
  historical Mise-era tool provisioning,
  kept as written.
