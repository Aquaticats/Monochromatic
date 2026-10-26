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

Empty state as of 2026-10-26:
no migration workaround has been observed yet.
These docs were written against the command grammar in
`doc/planning/mpm-toolchain-migration.md` while the toolchain implementation
was still landing in `package/dev-script/file-enforcer/src/toolchain/`,
so there was no emitting tool and no diagnostic to record.

Fill one entry per observed workaround as the implementation and cutover land,
in this shape:

- Surface:
  the command and working directory that produced the message.
- Emitting tool and message:
  the exact tool name and the verbatim diagnostic.
- Workaround:
  the runnable step that got past it,
  with the semantics that shifted.

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
   and the sync entry (`package/dev-script/file-enforcer/src/toolchain/`),
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
  `node package/dev-script/file-enforcer/src/toolchain/sync.ts` is TypeScript
  executed by an already-present `node`.
  Node and Mise stay the manually installed bootstrap prerequisites,
  and the sync then ensures `mpm` itself
  (a standalone release binary recorded in `toolchain.lock.json`,
  SHA-256 checked).

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
