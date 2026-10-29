# Mise removal coverage

Status:
 the replacement is chosen,
 its design is complete,
 and no code exists yet.
Full removal is the endpoint,
and it happens only when meow supports the full platform matrix,
at which point Mise is removed entirely rather than kept for provisioning.
Keeping Mise only for tool provisioning and locking is a transitional checkpoint,
not the target state.
Earlier drafts made TypeScript the canonical authored format for replacement configuration.
On 2026-09-16 the user stated that the monorepo manager's configuration does not have to be TypeScript,
so TypeScript is no longer a requirement for that owner.
Whether generated tool-native adapter files are acceptable remains unresolved.

Later on 2026-09-16 the monorepo manager narrowed to one remaining route:
a single-file all-Rust tool with file-enforcer rewritten in Rust,
because a single file carrying Node is too big.
The user accepted it the same day
([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md)),
and the replacement owners were chosen on 2026-09-17:

- Configuration is OpenTofu-shaped HCL,
   parsed by a patched `hcl-edit` under meow's own evaluator
   ([`doc/decision/monorepo-manager-hcl-front-end.md`](../decision/monorepo-manager-hcl-front-end.md)).
- Rules that write outside a repository move to a per-user configuration,
   with repository proposals the user accepts
   ([`doc/decision/monorepo-manager-per-user-config.md`](../decision/monorepo-manager-per-user-config.md)).
- Content hashing is XXH3-128
   ([`doc/decision/monorepo-manager-cache-key-hash.md`](../decision/monorepo-manager-cache-key-hash.md)).
- The root `mise.toml` stays generated from `mise.no-env.toml` until Mise is removed,
   and meow never generates it.
- Every line meow writes is a JSON object,
   so consumers of these tasks parse rather than scrape.

Sections that name file-enforcer the canonical TypeScript configuration compiler,
or that require provisioning Node before file-enforcer can execute,
describe today's TypeScript implementation,
which keeps running until meow replaces it.

## Motivation

On 2026-09-16 the user stated that Mise must be migrated away from because it is not documented well enough,
with no ETA.
Replacement candidates should therefore be judged partly on documentation quality for the features this repository
would actually consume.
Bazel was assessed as a broader alternative the same day;
see `doc/research/bazel-migration-dx.md`.

On 2026-09-29 the user stated a further reason:
 Mise is trying to do too much at once.
Its standing as a version manager is perfect,
so it could just fan out to other tools for everything else,
instead of absorbing their work into one binary.
Doing too much at once at once produces software whose behaviors are unpredictable,
so the breadth itself is a defect for this repository,
independent of the documentation gap.
One current example of that breadth:
 Mise embeds aube for `npm:` version resolution and installs
 ([jdx/mise#11149](https://github.com/jdx/mise/pull/11149),
 released in v2026.7.12)
rather than fanning out to an existing Node package manager.

## Monorepo manager hard requirements

Stated by the user on 2026-09-16:

- Whatever monorepo manager is chosen must provide every functionality of `@monochromatic-dev/dev-script-file-enforcer`.
  Cargo manifest enforcement and JetBrains LSP4IJ settings management are pluggable by definition.
  OS package provisioning and its Repology-derived package index may be replaced by Meta Package Manager.
- It must provide the watch plus RPC pattern:
   a watch mode that keeps everything up to date,
   plus an RPC or IPC mechanism through which people can inspect and control what the running watch process is doing.
  The user added the control requirement on 2026-09-16,
   correcting an earlier inference that observation alone was enough.
- Wildly underdocumented candidates are culled early,
   including documentation whose structure keeps readers from finding consumed behavior;
   moon is excluded on that basis.
  Any confusion or frustration while reading or trying to read a candidate's documentation culls it immediately,
   with no second chances.
- Result on 2026-09-16:
   `doc/audit/tech-monorepo-manager-vet-2026-09-16.md` screened 732 entries and re-screened 21 after the user counted
   external wrappers as plugging in;
   no candidate passed,
   so neither an existing tool nor a plug-in host qualifies.
- Fallback order:
   first an existing tool on the market;
   if none qualifies,
   plug these functionalities into an existing tool;
   if that is too hard,
   build a Bazel replacement.

Recorded inferences,
open to user correction:

- Inspection means at least observing current and recent activity.
- Control means at least starting or rerunning work and cancelling running work through the same channel.
- The inspection interface should be documented,
  because insufficient documentation is the stated reason for leaving Mise.

## Trigger and correction

The initial assessment recommended narrowing Mise before assigning every consumed responsibility to an owner.
That skipped the question that determines whether removal is viable:
what supplies each capability after Mise is gone?

This ledger records current owners,
proposed boundaries,
selection status,
parity evidence required before cutover,
and behavior that may be retired intentionally.
It is not a technology selection or adoption decision.

## Measured scope

Measurements used tracked files plus `mise tasks --all --hidden --json` from the repository root.

- 178 active Mise configuration files contain 9,422 newline-delimited lines.
- Root `mise.toml` declares 37 tools;
  `mise.lock` contains 38 tool entries that must be reconciled during a provisioning audit.
- Mise reports 1,811 effective tasks from 169 source files.
  This count includes inherited templates and is configuration-surface evidence,
  not a count of independent migration units.
- 9 of 14 tracked GitHub workflow files reference Mise.
- 160 active `package.json` files contain one script entry,
  the placeholder in `package/figma/to-penpot/package.json`.
- Effective task metadata contains 39 tasks with declared dependencies,
  133 with usage specifications,
  and no tasks with sources,
  outputs,
  post-dependencies,
  wait-for dependencies,
  or task-local tools.
- Task metadata reports no `raw` or `interactive` task properties,
  but `mise run --raw secrets:edit` is a documented CLI override and remains an interactive contract.

## Existing TypeScript configuration compiler

`file-enforcer.config.ts` and `@monochromatic-dev/dev-script-file-enforcer` already provide the canonical TypeScript
configuration layer proposed by this plan.
The package executes direct async TypeScript,
tracks source and destination files,
skips byte-identical writes,
persists staleness metadata,
protects generated destinations in watch mode,
and supports whole-file generation plus structured JSON,
TOML,
and XML edits.
It already generates root `mise.toml`,
keeps `CLAUDE.md` synchronized,
manages Cargo manifest fields,
patches JetBrains settings,
and exposes platform-aware command dispatch and OS package installation.

Replacement design should extend this boundary rather than introduce a second configuration generator.
Domain-specific typed plans should live in sibling TypeScript modules imported by `file-enforcer.config.ts`,
while required YAML,
TOML,
JSON,
XML,
and lockfiles remain generated or tool-owned adapters.
Field ownership must be explicit when an external tool also mutates a native manifest.

File-enforcer can also own toolchain convergence as the TypeScript policy and orchestration boundary.
Its current `ensurePackage` only checks whether a binary responds,
then installs an unversioned OS package;
its package records contain no version,
checksum,
provenance,
component,
or platform-artifact contract.
A toolchain extension should delegate resolution,
download verification,
and installation mechanics to focused providers such as pnpm,
rustup,
Android `sdkmanager`,
and a verified standalone-binary installer.
File-enforcer should compare desired and observed state,
order providers,
and report drift rather than reimplement every provider backend.

## pnpm 12.4+ candidate surface

pnpm 12.4.0 adds experimental Cargo and Python dependency installation plus `pnpm pipeline`.
It can now coordinate npm,
Cargo,
and Python dependency graphs in one install;
manage Node,
Deno,
and Bun runtimes;
execute project-aware shims;
and schedule workspace scripts with dependencies,
concurrency,
affected selection,
caching,
and aggregate failure reporting.

Update 2026-10-28:
 the session resolves pnpm 12.14.0 from a Mise-managed install,
and `toolchain.lock.json` records 12.16.1 for its `pnpm` entry with `track: latest`,
so the 12.4 additions are locally established.
`pnpm pipeline --help`,
`pnpm tasks --help`,
and `pnpm run --help` on 12.14.0 show pipeline runs,
concurrency-group status,
`--dry-run --json`,
`--stream`,
`--aggregate-output`,
`--reverse`,
`--sort`,
and the `[since]` affected-selection filters.
Cargo support installs and vendors dependencies but does not install Rust or compile crates.
Python support requires an existing interpreter.
Pipeline commands execute scripts selected from project manifests,
not TypeScript declarations directly.
`https://pnpm.io/cli/pipeline`,
fetched 2026-10-28,
still warns "pnpm pipeline is experimental.
Its configuration and output may change.";
12.5.0 added `concurrencyGroups`,
12.6.0 added task `priority` and `pnpm tasks status`,
and 12.12.0 added signed remote-cache result sharing.
Both 12.14.0 and 12.16.1 accept `--no-bail` and `--resume-from` while `pnpm run --help` lists neither,
because `pnpm/crates/cli/src/cli_args/cli_command.rs:321-337` marks the flags `hide = true`.
`doc/troubleshooting/pnpm-12-4-polyglot-workspace.md` records the source trace and current limitations.
`doc/troubleshooting/pnpm-recursive-no-bail-semantics.md` records the task-orchestration documentation trace.

## Meta Package Manager candidate surface

Meta Package Manager 7.6.1 plus pnpm 12.4.1 is a plausible replacement architecture when focused native providers and
repository-owned adapters are understood as parts of the design.
pnpm can own workspace tasks and dependencies.
Meta Package Manager can own machine-package inventory and coordinate package managers available at invocation start.
Existing direct watcher commands can run as pnpm scripts.

This is not a two-executable substitution.
One MPM restore does not install a package manager and then begin using it later in the same invocation,
so file-enforcer or another outer owner must stage bootstrap operations.
MPM's TOML dump records installed versions,
but stable default adapters can ignore those versions during restore.
The pnpm and pipx adapters explicitly warn and let the manager choose;
the Cargo and stew definitions also omit a version from their install commands.
Custom TOML manager definitions cannot forward a restore entry's version through a `{version}` placeholder.
MPM can invoke its stable Cargo,
uv,
SDKMAN,
pnpm,
stew,
and platform-manager adapters.
Provider-native commands still exist underneath that interface.
The outer workflow needs direct pnpm,
Cargo,
rustup,
Android `sdkmanager`,
or standalone-binary calls only where MPM lacks a stable adapter or cannot express a required pin,
component,
feature,
license-input,
or platform option.

The stable release also lacks the rustup and bin adapters currently documented as unreleased 8.0 additions.
Missing built-ins are not automatically permanent gaps:
MPM 7.6.1 can register first-class managers from trusted local TOML definitions,
and full-power Python manager classes can implement version pinning,
stateful parsing,
and arbitrary provider logic when contributed to or carried with MPM.
The TOML path is intentionally bounded,
and stable MPM has no separately installed Python entry-point plugin loader.
`doc/troubleshooting/mpm-manager-extensibility.md` records the source path and disposable runtime fixture.

These constraints do not make staged composition impossible,
but they keep the architecture unselected until a declaration-level mapping and disposable pilot pass.
`doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md` records the official documentation,
stable source audit,
candidate ledger,
and conditional fit result.

## Coverage ledger

### Tool acquisition, versions, and bootstrap prerequisite

Current owner:
 Mise `[tools]`,
backends,
shims,
`mise.lock`,
`mise install`,
and `mise upgrade` until the cutover in `doc/planning/mpm-toolchain-migration.md` removes them;
the `Owner, selected 2026-10-26` field describes the owner after cutover.

Evidence:
 `mise.toml:27-211`,
`mise.lock`,
`.github/workflows/npm-release.yml`,
and package Containerfiles.
The declarations span runtime toolchains,
language-package backends,
release artifacts,
Rust components,
JDK and Android tooling,
and platform-specific binaries.

Owner,
selected 2026-10-26
 ([`doc/decision/mpm-toolchain-adoption.md`](../decision/mpm-toolchain-adoption.md)):
 Meta Package Manager owns the restore manifest (`mpm.packages.toml`),
machine inventory (`mpm installed`),
outdated detection (`mpm outdated`),
upgrade coordination (`mpm upgrade`),
the release-age cooldown,
and SBOM export (`mpm sbom`).
file-enforcer's typed tool plan
(`package/dev-script/file-enforcer/src/toolchain/`) owns policy,
convergence order,
and drift reporting.
The maintainer confirmed the install route per tool family
(`doc/planning/mpm-toolchain-migration.md`,
section `Routing (user-confirmed 2026-10-26)`):
single-binary payloads install with the upstream `bin` CLI
(`bin install <source spec>`,
GitHub shorthand,
release-tag URLs,
host sources),
and MPM's `bin` manager owns their inventory and upgrades afterwards
(the manager declares no `install` operation by design);
multi-binary or whole-tree payloads (`age`,
`cmake`,
`llama.cpp`,
`uv`,
`taplo`,
the JDK,
Node,
`zig`,
`bubblewrap`) and sources `bin` cannot fetch
go through the checksummed `archive` provider.
npm CLI tools route through MPM's `pnpm` manager id
and `pipx:slopo` through MPM's `uvx` manager id
(the one wrapping `uv tool install`).
`cargo:` tools stay a repository provider over `cargo-binstall`.
Rust stays MPM's `rustup` manager
(toolchain ids carry the exact channel)
plus direct `rustup component add` and `rustup target add` calls.
Android SDK components stay a repository provider
running the Android CLI shipped in cmdline-tools
(`android sdk install <package>[@<version>]...`).
Declarations carry no `@version` pin syntax and keep floating requests;
`toolchain.lock.json` fixes what `toolchain:sync` installs
(the `mise install` against `mise.lock` semantics),
so the explicit pins still install exactly.
The sync installs the locked versions the way `pnpm install` honors
`pnpm-lock.yaml`,
and re-resolving versions is a separate update action
(the `pnpm update` analogue).
`mpm.managers.toml` entries are inventory and pin descriptions
for the `cargo:` family only;
no install depends on `mpm restore` except the `rustup` slice.
`toolchain.lock.json` replaces `mise.lock` as the pinned version record.
The sync entry point is `mise run toolchain:sync`
(also `node package/dev-script/file-enforcer/src/toolchain/sync.ts`),
`mise run toolchain:status` reports drift,
and `node package/dev-script/file-enforcer/src/toolchain/bin-paths.ts` reports the exposure directory.
One manually installed bootstrap prerequisite remains:
Node executes the sync entry and Mise wraps it as a task;
the sync then stages `bin` (a standalone release binary,
SHA-256 checked against the release `checksums.txt` values),
`uv`,
and `mpm` (the PyPI wheel `meta-package-manager` 8.2.0,
SHA-256 checked against PyPI's published digest,
installed with `uv tool install`) before the other tools.

Selection status:
 selected 2026-10-26 on the maintainer's direct instruction,
with the disposable provisioning verification in `doc/planning/mpm-toolchain-migration.md` as the adoption pilot.
The maintainer confirmed the per-family routes on 2026-10-26
(`doc/decision/mpm-toolchain-adoption.md`,
section `Maintainer confirmations (2026-10-26)`).
`mpm restore` covers only the managers whose adapters honor the recorded version;
the sync installs the rest from `toolchain.lock.json`
(`doc/troubleshooting/mpm-manager-extensibility.md`).
macOS and Windows provisioning parity stays unverified.

Parity gate:
 a disposable clean home on every supported operating system installs the committed versions,
checks supported artifact integrity and provenance,
applies tool options and components,
exposes executables to shells and child processes,
and performs explicit updates without unrelated installations.

Intentionally retired behavior:
 none selected.
Floating `latest` requests keep today's semantics
(floating requests with committed resolutions),
confirmed 2026-10-26 in `doc/decision/mpm-toolchain-adoption.md`,
section `Maintainer confirmations (2026-10-26)`.

### Task discovery and execution

Current owner:
 Mise monorepo discovery,
task templates,
target-path names,
arguments,
aliases,
working directories,
dependency execution,
and output handling.

Evidence:
 `mise.no-env.toml`,
active package `mise.toml` files,
and `doc/decision/mise-task-node-invocations.md`.
`mise.no-env.toml:282-343` already implements repository-owned discovery filtering,
bounded fanout,
and failure collection inside Node scripts that invoke Mise recursively.

Owner,
selected 2026-09-17:
 meow.
Tasks are declared in one root `meow.hcl`,
packages are selected by tags meow infers from manifests and `package` blocks adjust,
specialization is most-specific-tag-wins with mandatory `overridden` and `override` markers,
and a command is an argv list with no shell
("Configuration authoring" in `monorepo-manager-from-scratch-design.md`).
Execution reaches a task through `meow run //package/<path>:<task>`,
which holds the terminal,
forwards the task's bytes,
and exits with its code;
`meow status`,
`meow pause`,
`meow resume`,
`meow end`,
and `meow priority` control work already scheduled
("User interface").
Cross-package edges come from the native manifests,
and freshness comes from hashing a task's declared reads.

Still open on 2026-09-17,
so this entry cannot be marked verified:
which task names meow builds in,
how a package changes one,
whether built-ins share a namespace with user-defined tasks,
and what makes one task run before another
("Open questions" in the design).

Superseded proposals:
 pnpm 12.4+ workspace orchestration with `pnpm pipeline`,
file-enforcer compiling TypeScript task definitions into `pnpm-workspace.yaml` and manifest scripts,
and Nadle as the fallback.

Selection status:
 blocked on a disposable pnpm 12.4+ pilot,
source audit,
and representative runtime validation.
The pipeline feature is experimental and task execution still depends on project manifest scripts.

Update 2026-09-16:
pnpm exited the monorepo manager vet at the documentation gate.
`https://pnpm.io/cli/recursive` says "Even if `--no-bail` is used,
 all tasks will finish",
while `https://pnpm.io/workspace-task-orchestration` says that after a task fails,
dependent tasks are skipped and only independent ready tasks continue under `--no-bail`.
Nadle,
the fallback,
also exited at the documentation gate:
`https://nadle.dev/docs/getting-started/features/` configures `CopyTask` with `to`,
while `https://nadle.dev/docs/guides/file-operation-tasks/` uses `into`.
See `doc/audit/tech-monorepo-manager-vet-2026-09-16.md`.

Update 2026-10-28:
 the failure-semantics question that update left open is now measured,
and `doc/troubleshooting/pnpm-recursive-no-bail-semantics.md` holds the probe and source trace.
Both pnpm 12.14.0 and 12.16.1 skip a failed task's dependents under `--no-bail`,
continue independent ready tasks,
and exit non-zero
(`pnpm/crates/workspace-task-scheduler/src/tests.rs:880`),
so the `workspace-task-orchestration` sentence is correct
and the `cli/recursive` sentence "all tasks will finish" is false as written.
Parity-gate items "intentional failure tolerance",
"tests continuing after build failure where required",
"retained failure causes",
and "correct aggregate exit status" hold across independent projects,
but a dependent of a failure never runs under any `--no-bail` spelling.

Task edges and package edges also disagree today.
`rg -n '^\s*depends\b' -g 'mise*.toml' package/` matches 178 lines on 2026-10-28,
and edges name other packages and the root project directly:
`package/translation-repair/translate-stage/mise.toml:10`
depends on 13 `//package/translation-repair/*:build:js:node` tasks
while `package/translation-repair/translate-stage/package.json` omits
`@monochromatic-dev/translation-repair-glossary` from `dependencies`.
pnpm `dependsOn` entries take only `name` for the same project
and `^name` for the same-named task in each workspace dependency
(`https://pnpm.io/workspace-task-orchestration`,
fetched 2026-10-28),
so exact task-edge parity needs package-graph reshaping or repository-owned scheduling.
Missing scripts pass through as skipped tasks,
and `pnpm -r run --dry-run --json test` prints the resolved graph with `missingScript` flags,
which covers "missing optional package builds" and machine-readable listing at the graph level.
No disposable pnpm pilot is recorded in `doc/`;
every gap recorded here is documentation,
configuration,
or probe evidence.

Update 2026-10-29:
 the disposable pilot is now recorded in
[`pnpm-task-execution-pilot.md`](pnpm-task-execution-pilot.md),
covering graph reshaping,
declaration compilation,
paths with spaces,
missing scripts,
and pipeline failure tolerance.
It closes the "no pilot" gap with a conditional ranking,
and the remaining open item is whether the user accepts three semantic shifts
for a pnpm-owned task slice.

Update 2026-10-29 (Nadle):
 [`nadle-task-execution-pilot.md`](nadle-task-execution-pilot.md)
pilots Nadle 0.6.1 against the same gates and compares both with meow.
Nadle covers exact task edges,
typed in-process bodies,
caching,
and machine-readable output,
but its name grammar bans the colon task vocabulary,
so the contract choice is now a three-way flagged decision.

Update 2026-10-29 (task-slice comparison):
 [`task-slice-comparison.md`](task-slice-comparison.md)
joins the measured Poe the Poet 0.49.0,
Buck2 2026-10-14,
Pants 2.33.1,
and projen 0.104.19 pilots
with pnpm 12.16,
Nadle 0.6.1,
meow,
and keeping Mise.
Only Poe the Poet and projen keep the colon task names;
the documented-drift class that culled Nadle reproduces in every candidate,
so it is recorded per candidate rather than used as a cull.
Please and Tilt pilots are in flight,
then the contract question is asked with every candidate.

Parity gate:
 preserve package namespaces,
partial invocation,
paths containing spaces,
sequential and parallel groups,
missing optional package builds,
intentional failure tolerance,
tests continuing after build failure where required,
retained failure causes,
machine-readable task listing,
and correct aggregate exit status.
`package/dev-script/task-util/src/build-and-test.unit.test.ts` is part of the acceptance boundary.

Intentionally retired behavior:
 no task feature is selected for retirement.
Unused Mise task features do not need replacement unless a current caller outside task metadata is found.

### Watching and process lifecycle

Current owner:
 several different paths.
Some packages invoke `mise watch`,
root `watch:*` tasks fan out through Mise,
and file-enforcer owns its own `--watch` mode.

Evidence:
 `package/kwin/key-helper/mise.toml`,
`package/ssg/aquati.cat/mise.toml`,
`package/webapp-productivity/done/mise.toml`,
and root watch templates.

Proposed owner:
 direct watcher processes or a focused task runner,
selected per current behavior rather than by blanket translation.

Selection status:
 unselected.

Parity gate:
 verify watched roots,
ignore rules,
restart versus parallel behavior,
argument forwarding,
signal delivery,
child cleanup,
and terminal restoration.

Intentionally retired behavior:
 none selected.

### Environment, PATH, and shell activation

Current owner:
 Mise tool environments,
root and package `[env]` sections,
`[vars]`,
configuration precedence,
and optional shell activation.
`file-enforcer.config.ts:652-720` generates root PATH entries from installed workspace bins.

Evidence:
 `mise.toml:1241-1325` and package environment sections.
The active project has four top-level `[env]` sections and six `[vars]` sections.
The root enter hook is commented out,
so its bootstrap-on-entry description does not prove current automatic execution.

Proposed owner:
 unselected explicit environment launcher for tasks and CI.
Automatic directory-entry activation is a separate preference and receives a separate owner only if retained.

Selection status:
 blocked on deciding whether ambient shell activation is required.

Parity gate:
 preserve root and package precedence,
working-directory behavior,
PATH ordering,
workspace executable discovery,
tool-produced variables,
and clean-clone operation before workspace dependencies exist.
Test direct shell,
IDE,
task,
and CI consumers separately.

Intentionally retired behavior:
 automatic shell activation is undecided,
not implicitly retired.

### Secrets, redaction, and interactive editing

Current owner:
 Mise loads optional encrypted `.env.local.json` through its rops integration,
marks values for output redaction,
and injects them into commands.
`@monochromatic-dev/module-env-local-json` reads the same store directly for
consumers launched outside mise and owns decryption,
discovery,
and the identity chain.
SOPS and age own editing and cryptographic primitives.

Evidence:
 `mise.no-env.toml:1212-1233` for the `secrets:edit` and `secrets:migrate`
tasks,
`file-enforcer.config.ts:652-668` for the generated `[env]` `_.file` loader,
and `package/module/env-local-json/src/` for the direct reader,
identity resolver,
atomic writer,
and edit CLI.

Proposed owner:
 retain SOPS and age as primitives;
`@monochromatic-dev/module-env-local-json` owns file reading,
decryption,
identity resolution,
and the write and edit subcommands.
A narrow child-environment launcher for mise-run command injection,
output redaction,
and command echoing is still unselected.
Interactive terminal passthrough is owned by the edit CLI.

Selection status:
 reading,
decryption,
identity resolution,
and interactive editing are selected and implemented in the module
(2026-10-18).
Child-only injection and output redaction for task commands remain in Mise and
are unselected.
The identity location is decided:
it stays at `~/.config/mise/age.txt`,
and the shared chain also honors `MISE_CONFIG_DIR` and `XDG_CONFIG_HOME`.

Parity gate:
 disposable canary secrets prove optional-file absence,
invalid ciphertext failure,
missing identity failure,
child-only injection,
stdout and stderr redaction,
error redaction,
interactive editor access,
signal handling,
and absence of plaintext files.
For the direct reader specifically:
requested keys only,
no `process.env` mutation,
explicit-path hard failure,
repository-root and bounded-walk discovery,
plaintext rejection before identity lookup,
and no value in any log or error.
The reader tests generate fixtures with real `sops` and `age` and run in an
Ubuntu container as root and as a non-root user.
Never use real credentials for this gate.

Intentionally retired behavior:
 none selected.

### CI integration

Current owner:
 `jdx/mise-action`,
Mise caches,
job-scoped `mise install`,
`mise exec`,
and `mise run`.
Six workflows disable automatic installation to prevent unrelated root tools from being installed.

Evidence:
 `.github/workflows/npm-release.yml:35-83` and the other Mise-using workflows.
The npm release workflow also writes resolved Node and pnpm directories to `$GITHUB_PATH` for later third-party actions.

Proposed owner:
 the selected tool provisioner plus the selected task and environment interfaces.
CI should request only each job's tools.

Selection status:
 blocked by those selections.

Pin source:
 `toolchain.lock.json`
 (generated by file-enforcer from the typed tool plan)
 replaces `mise.lock` as the pinned-version record every workflow and container build reads.
 The cutover repoints `.github/actions/task-cli-setup/action.yml`,
 the `awk` Rust scrapers in `.github/workflows/music-player-release.yml`,
 `.github/workflows/kotlin-linter-publish.yml`,
 and `package/music-player/desktop-app/Containerfile.windows-gnu`
 (`doc/planning/mpm-toolchain-migration.md`,
 section `Cutover`).

Parity gate:
 all nine workflows preserve cache keys,
release-age policy,
locked versions,
PATH across steps,
job permissions,
secret masking,
and the same user-facing task behavior as local execution.

Intentionally retired behavior:
 none selected.

### Repository-root discovery

Current owner:
 `MISE_MONOREPO` in `@monochromatic-dev/module-fs-path` and its consumers.

Evidence:
 `package/module/fs-path/src/root-marker.ts` plus consumers in application,
CLI,
build,
test,
and VM code.
The package already exports `PNPM_WORKSPACE` and `GIT_REPOSITORY`,
but those markers represent different boundaries.

Proposed owner:
 select the marker separately for each caller's domain.
A package-workspace boundary and a Git-checkout boundary are not interchangeable.

Selection status:
 caller classification pending.

Parity gate:
 cover nested workspaces,
Git worktrees whose `.git` is a file,
source archives,
missing markers,
nearest-marker behavior,
and cached lookup behavior.

Intentionally retired behavior:
 the Mise-specific marker can retire only after every caller has a semantically correct replacement.

### Editors, VM images, and containers

Current owner:
 Mise shims and install paths appear in IntelliJ configuration;
the development VM installs Mise and seeds its global configuration;
Containerfiles use Mise for selected runtime setup.

Evidence:
 `.idea/dprintProjectConfig.xml`,
`.idea/opentofu_settings.xml`,
`.idea/misc.xml`,
`package/config/dotfiles/`,
`package/dev-script/vm-builder/Containerfile`,
and `package/cli/mutation-test/runtime/Containerfile`.

Proposed owner:
 selected tool provisioner paths or stable project-local wrappers.
Existing Containerfiles and VM-builder code remain machine-setup owners,
not a new task runner.

Selection status:
 the tool provider is selected:
 the toolchain system of `doc/decision/mpm-toolchain-adoption.md` (2026-10-26),
 which installs into manager-native homes plus the toolchain root and exposes
 executables through `.cache/toolchain-bin`.
 The `.idea/` path repoints and the Containerfile pin repoints land with the
 consumer migration in `doc/planning/mpm-toolchain-migration.md`,
 section `Consumers`.
 The user-global `~/.config/mise/config.toml` `[tools]` slice moved to the same
 system on 2026-10-28
 (`doc/planning/mpm-global-toolchain-migration.md`);
 the VM-builder and Containerfile seeding of a global Mise config still needs
 to follow that shape.

Parity gate:
 editor linting,
formatting,
JDK selection,
OpenTofu integration,
VM construction,
and container runtime paths work without user-specific hardcoded locations.

Intentionally retired behavior:
 Mise schema and IDE-plugin configuration may retire after all `mise.toml` files retire.

### File-enforcer provisioning policy

Current owner:
 file-enforcer generates root `mise.toml`,
queries `mise registry`,
and excludes Mise-installable tools from other package-manager provisioning paths.

Evidence:
 `file-enforcer.config.ts`,
`package/dev-script/file-enforcer/src/package/manager-defs.ts`,
and
`package/dev-script/file-enforcer/src/package/mise.generate-index.ts`.

Proposed owner:
 file-enforcer is the canonical TypeScript configuration compiler
and toolchain-convergence orchestrator for the replacement architecture.
It should generate the selected provisioning,
workspace-task,
manifest,
CI,
and editor adapters from typed domain modules.
Focused providers retain artifact resolution and installation mechanics;
file-enforcer should not become the task or secret runtime.

Selection status:
 existing generation,
structured editing,
staleness,
watch protection,
Cargo enforcement,
JetBrains integration,
and platform package dispatch are implemented.
New pnpm and replacement-tool plans remain undesigned and unverified.

Parity gate:
 generated files remain deterministic,
required system packages are not filtered out accidentally,
and no tool is omitted because a retired registry formerly claimed ownership.

Intentionally retired behavior:
 Mise-registry filtering retires with the `[tools]` cutover.
 The replacement classification rule is the `route` field of `toolchain.lock.json`
 (`bin-sourced`,
 `archive`,
 `cargo-binstall`,
 `mpm-pnpm`,
 `mpm-uvx`,
 `rustup-toolchain`,
 `rustup-components`,
 `android-sdk`),
 selected in `doc/decision/mpm-toolchain-adoption.md` (2026-10-26).

### Agent policy, guardrails, filters, tests, and documentation

Current owner:
 repository policy requires `mise run`;
agent guardrails recognize its command form;
output filters classify Mise warnings;
fixtures require a Mise executable;
README and troubleshooting documents teach Mise commands.

Evidence:
 `AGENTS.md:844-879`,
`package/agent-harness-shared/shell-command-analyzer/`,
`package/pi-plugin/guardrail/`,
`package/claude-code-plugin/source/src/handler/bash-output-filter/`,
and Mise-specific test fixtures.

Proposed owner:
 selected task command grammar,
tool diagnostics,
and migration documentation.
Historical troubleshooting evidence remains historical and should not be rewritten as current instructions.

Selection status:
 blocked by final interfaces.

Parity gate:
 command analysis permits the new safe forms,
blocks their unsafe variants,
filters only diagnostics proven safe to suppress,
and all user-facing instructions name executable commands that pass at the consumer boundary.

Intentionally retired behavior:
 Mise-specific noise transforms and command exceptions should retire when no active producer remains.

### Upstream features not consumed by tracked project configuration

Searches of tracked root and package configuration and source found no active `[bootstrap.*]`,
`mise bootstrap`,
Mise MCP,
Mise OCI,
or Mise dependency-management invocation.
The root task named `bootstrap` is a project task that runs `mise install` and `mise upgrade`,
not Mise's machine-bootstrap subsystem.

Proposed owner:
 none.
Unused upstream capabilities require no replacement.

Parity gate:
 repeat the search across the final tracked tree
and inspect user-global configuration separately
before claiming machine-wide removal.

Intentionally retired behavior:
 these unconsumed project capabilities may remain absent.

## Safe migration sequence

1. Validate file-enforcer as the canonical TypeScript compiler and pilot pnpm 12.4+ in a disposable workspace while Mise
remains installed.
2. Select the remaining task,
tool,
and environment boundaries from the pilot evidence.
3. Move executable logic behind ordinary process interfaces that do not require Mise discovery,
templates,
or environment variables.
Keep thin Mise adapters so existing callers continue working.
4. Run representative TypeScript,
Rust,
Android,
desktop,
watch,
deployment,
and secret paths through both interfaces in disposable fixtures.
5. Migrate CI,
editor paths,
root discovery,
file-enforcer policy,
agent tooling,
and documentation only after their new owners pass their parity gates.
6. Select and validate tool provisioning early,
but cut it over after task and environment consumers no longer require Mise semantics.
7. Remove Mise configuration and installation only after a clean-machine exercise passes without it.

Every step is an independently useful checkpoint.
Keeping Mise solely for tool acquisition,
version selection,
and locking is an acceptable transition state while the final provisioning replacement is validated.

## Removal gates

Full removal remains blocked until all conditions hold:

- Every ledger entry has a selected owner or an explicitly accepted retirement decision.
- Every selected external technology completes the choosing-technology process.
- Clean disposable environments pass on every supported operating system and CI runner.
- Local and CI invocations use one verified task contract.
- Secret and interactive terminal tests pass with synthetic credentials.
- Tool integrity,
provenance,
version locking,
and update behavior meet or exceed the current contract.
- IDE,
VM,
container,
agent,
and source-level consumers no longer depend on active Mise behavior.
- A broad search finds no active Mise invocation outside preserved historical evidence.
- The rollback path has been exercised before deleting the Mise files.

## Unresolved preferences

The following user choices can change the architecture and candidate set:

- Keep or retire automatic directory-entry environment activation.
- Accept pnpm's package,
runtime,
shim,
and pipeline breadth as one workspace boundary,
or treat it as the same scope-concentration problem motivating Mise removal.
- Prefer one focused cross-language tool manager
  or several ecosystem-native installers
  for capabilities pnpm does not cover.
- Accept generated TOML,
  YAML,
  JSON,
  or other tool-native adapters from canonical TypeScript,
  or require every replacement tool to consume TypeScript directly.
  For the monorepo manager this is settled:
  its configuration does not have to be TypeScript.
- Preserve floating tool requests or replace them with reviewed update automation and exact locks.

Full removal of the Mise binary is the accepted endpoint assumption for candidate discovery.
