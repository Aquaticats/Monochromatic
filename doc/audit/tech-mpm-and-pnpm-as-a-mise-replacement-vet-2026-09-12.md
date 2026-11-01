# MPM and pnpm as a Mise replacement vet report

Status:
complete fit assessment;
Meta Package Manager plus pnpm is plausible but the composition remains incomplete and unvalidated.

Lifecycle phase:
serious alternative;
hard-gate confirmation and finalist validation remain pending.

Subject:
MPM and pnpm as a Mise replacement.

Decision scope:
assess whether Meta Package Manager 7.6.1 plus pnpm 12.4.1 can replace Mise for repository tasks and
developer-tool provisioning in Monochromatic.

Start date:
2026-09-12.

Last updated:
2026-09-12.

Governing skill commit:
`a05818ad70a40e5769a36de669697ba109891b31`.

Governing skill SHA-256:
`393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
`5ddc05149175ef46eadbdc5171969e02c48cf1dd1d25570419fa8e7cfb0c5ab3`.

Active audit owner:
Pi session `01a097c8-535a-72ce-9dbf-26985664148b`.

Prior compatible report:
none.
The existing [`mise-removal-coverage.md`](../planning/mise-removal-coverage.md) is a responsibility ledger,
not a technology vet report.

## Result

A migration away from Mise is realistic,
and **Meta Package Manager plus pnpm is a plausible organizing architecture** when ecosystem-native installers and
repository-owned adapters are part of the design.
The evidence does not support treating the names as a self-contained pair of executables.

pnpm 12.4.1 is a plausible owner for workspace dependency installation and task orchestration.
Meta Package Manager 7.6.1 can own machine-package inventory and coordinate package managers already present at the
start of an invocation.
Existing watcher commands can run as pnpm scripts.
Focused repository code can supply child environments and SOPS handling.

The important distinction is ownership:
MPM can orchestrate package providers,
but it does not replace their installation mechanics or provide one universal artifact lock.
Its restore format does not enforce recorded versions for several relevant default adapters.
Provider-native rustup,
Cargo,
SDKMAN,
Android `sdkmanager`,
and verified standalone-binary operations therefore remain part of the architecture.
MPM can invoke several of them through adapters;
an outer owner needs direct commands only where the stable adapter is absent or cannot express required options.

The proposal still needs defined and validated owners for:

- initial pnpm and package-manager bootstrap;
- Rust toolchains,
  components,
  and targets;
- JDK,
  Android SDK and NDK,
  Zig,
  and standalone binaries;
- task child environments,
  root and package environment precedence,
  and optional shell activation;
- SOPS decryption,
  child-only secret injection,
  redaction,
  and interactive editing;
- CI setup,
  editor paths,
  repository-root discovery,
  agent command policy,
  and generated configuration migration.

The evidence supports continuing with an **MPM plus pnpm pilot backed by focused native providers**.
It does not yet support adoption or clean-machine cutover.

## Context and hard constraints

The incumbent is Mise `2026.9.5` on Linux x64.
The repository currently resolves pnpm `12.3.4`.
`README.md:338-340` names Fedora as the development target and WSL2 as the Windows path.
Selected GitHub workflows also execute on macOS and Windows.

The current responsibility ledger measured:

- 178 active Mise configuration files containing 9,422 lines;
- 1,811 effective tasks;
- 37 root tool declarations and 38 lock entries;
- nine GitHub workflow files that reference Mise;
- 101 active package README files that reference Mise;
- 26 decision documents and nine runbooks that reference Mise.

Sources:
[`mise-removal-coverage.md`](../planning/mise-removal-coverage.md),
`mise.no-env.toml`,
active package `mise.toml` files,
and a fresh `mise tasks --all --hidden --json` count on 2026-09-12.

### Frozen hard constraints

- Complete responsibility coverage or an explicit retirement decision.
- Inspectable open-source local execution.
- Preserve current task failure,
  filtering,
  working-directory,
  and argument behavior.
- Preserve explicit pins where the current configuration has them,
  preserve intentional floating requests until separately changed,
  and retain reviewable artifact provenance.
- Work on repository-supported local and CI platforms.

### Components and overlays

Every candidate is an inspectable open-source local technology.

Applicable overlays:

- incumbent dependency replacement;
- high-trust developer-machine and CI execution;
- secret-handling boundary;
- native or prebuilt artifact boundary;
- multi-platform claims.

### Frozen soft criteria

No candidate reached scoring because the candidate composition is not complete.
If a later pilot produces validated finalists,
use these criteria without changing them in response to candidate results:

- separation of responsibilities,
  weight 5 because avoiding an over-broad owner is the stated motivation;
- repository fit,
  weight 1;
- human auditability,
  weight 1;
- maintenance evidence,
  weight 1;
- operational simplicity,
  weight 1.

Hard constraints remain outside arithmetic.

### Unresolved preferences

These preferences change the eventual architecture but do not change this report's fit result:

- retain or retire automatic directory-entry environment activation;
- accept generated `package.json` and `pnpm-workspace.yaml` task adapters from canonical TypeScript;
- retain floating tool requests or replace them with exact locks and reviewed update automation.

## Discovery protocol

### Frozen query schedule

Official ecosystem and registry queries:

- `site:mpm.run mpm.run task runner documentation pnpm monorepo`
- `site:pnpm.io pnpm runtime Node Deno Bun official docs`
- `site:pnpm.io/12.x pnpm cargo enabled python enabled documentation`
- exact PyPI metadata for `meta-package-manager`;
- exact GitHub release metadata for `kdeldycke/meta-package-manager` and `pnpm/pnpm`.

Repository-host queries:

- `mpm.run GitHub task runner monorepo package manager`
- `site:github.com "mpm.run"`
- GitHub repository metadata and release APIs for each discovered candidate.

Broader web queries:

- `"mpm" "pnpm" task runner run monorepo`
- `npm mpm.run package task runner`
- `open source meta package manager declarative package manifest restore cross-platform alternative to
  meta-package-manager mpm`
- `GitHub meta package manager wrapper multiple package managers install restore manifest cross platform`
- `package manager inventory restore all package managers mpm alternative Topgrade pacdef metapac`

Repository queries:

- active Mise configuration,
  tool,
  task,
  environment,
  workflow,
  source,
  README,
  decision,
  and runbook references;
- prior Mise removal plans and pnpm 12.4 investigation;
- existing repository-owned generators and provisioning code.

### Query outcomes

The initial web round identified Meta Package Manager,
pnpm,
`repo-run`,
and unrelated task runners.
The taxonomy expansion round identified `metapac`,
`pacdef`,
and Topgrade.
No later term added another candidate category.

The named-stack fit search is saturated across its official site,
package registry,
source host,
and this repository.
This was not a saturated selection of every task runner or every tool-version manager.
The report therefore assesses the proposed pair and records adjacent alternatives,
but does not select an entire replacement stack.

## Candidate ledger

### Meta Package Manager 7.6.1 plus pnpm 12.4.1

Discovery sources:
user proposal,
[Meta Package Manager documentation][mpm-home],
[pnpm documentation][pnpm-run],
and upstream source.

Base category:
inspectable open-source local technology.

Overlays:
all report overlays.

Screening result:
**serious alternative with unresolved composition**.
pnpm and MPM have credible boundaries,
but native providers and repository-owned adapters must be named before hard-gate confirmation.
Stable Meta Package Manager cannot reproduce several current pins through its default restore adapters,
so those entries need direct provider commands or a different owner.

### pnpm 12.4.1 plus repository-owned orchestration and focused native providers

Discovery source:
existing repository plan,
pnpm 12.4 documentation,
and pnpm source.

Base category:
inspectable open-source local technologies plus repository-owned TypeScript.

Overlays:
all report overlays.

Screening result:
**serious migration architecture,
not a validated candidate stack**.
Providers for non-pnpm toolchains,
environments,
and secrets remain unselected.
Existing direct watcher commands are plausible owners,
but their process-lifecycle parity remains unverified under pnpm.
No adoption recommendation is possible.

### Mise 2026.9.5 plus pnpm 12.3.4

Discovery source:
incumbent repository configuration.

Base category:
inspectable open-source local technologies.

Overlays:
incumbent and high-trust execution.

Screening result:
**excluded from the requested endpoint**.
It is the current rollback baseline,
but retaining it does not achieve migration away from Mise.

### metapac

Discovery source:
GitHub and the broader meta-package-manager search.

Base category:
inspectable open-source local technology.

Overlays:
incumbent replacement,
high-trust execution,
and multi-platform claims.

Screening result:
**not promoted for this proposal**.
Its own README describes a declarative wrapper over underlying package managers.
It is closer than Meta Package Manager to desired-state package declarations,
but substituting it would not answer the task,
environment,
or secret boundaries and would depart from the user's named stack.

### pacdef

Discovery source:
Meta Package Manager's comparable-tool catalogue and GitHub.

Screening result:
**hard-gate exit**.
The repository was archived on 2025-08-05 and points users to metapac.

### Topgrade

Discovery source:
Meta Package Manager integration and the broader search.

Screening result:
**category mismatch**.
Topgrade detects package managers and upgrades existing installations;
it is not a declarative clean-machine provisioner or project task runner.

## Meta Package Manager evidence

### Scope and execution model

Meta Package Manager's official homepage describes `mpm` as a wrapper around package managers.
Its primary operations inventory,
search,
install,
remove,
upgrade,
dump,
restore,
and export machine packages.
The current catalogue reports 143 actively supported manager adapters.
The stable 7.6.1 source contains 42 Python manager modules,
42 manager classes,
and 24 bundled TOML manager definitions.

This breadth is coherent for machine package inventory,
but it does not address the project's task and environment surfaces.
It replaces a multi-domain abstraction with a package-manager-only abstraction that spans many backends.
That separation may satisfy the user's concern even though MPM itself has a substantial package-manager surface.

Evidence:
[official homepage][mpm-home],
[manager catalogue][mpm-managers],
and stable source measurements.

### MPM delegates to managers available at invocation start

Before an operation,
MPM detects selected managers by invoking each manager's version probe.
It then fans operations out across available managers.
A single restore invocation has no dependency level that installs a manager and then begins using that newly available
manager later in the same invocation.

This leaves initial bootstrap and cross-manager ordering outside one MPM restore.
A staged workflow can still use an available platform manager to install another manager,
then invoke MPM again after detection changes.
That composition needs an outer owner such as file-enforcer and a disposable clean-machine proof.
The source audit establishes the single-invocation limitation,
not that staged bootstrapping is impossible.

Evidence:
[concurrency documentation][mpm-concurrency] and
`meta_package_manager/cli_snapshots.py:400-476` at stable tag `v7.6.1`.

### MPM can invoke many native providers

Stable MPM 7.6.1 has built-in adapters that invoke Cargo,
uv,
SDKMAN,
pnpm,
stew,
and platform package managers.
The wrapper does not eliminate those provider CLIs;
it standardizes operations over them.

The distinction in this assessment is therefore not `MPM` versus `direct` for every operation.
It is:

- MPM invocation where a stable adapter preserves required package identity,
  version,
  feature,
  and platform semantics;
- an outer direct invocation where no stable adapter exists or its operation model loses required data.

For this repository,
stable MPM has no rustup adapter.
The current development branch's rustup adapter manages toolchains but deliberately excludes components,
while the repository also needs `clippy`,
`rust-src`,
`llvm-tools-preview`,
and Android compilation targets.
MPM has no built-in Android `sdkmanager` adapter,
and the existing setup also owns license-input handling.
The stable Cargo adapter invokes `cargo install`,
but does not forward restore versions or Cargo feature options.
SDKMAN and uv are positive cases:
their built-in adapters can pass requested versions.

### MPM is already extensible at two levels

Stable MPM 7.6.1 can synthesize a new first-class manager from a trusted local
`[mpm.managers.<id>]` configuration block.
The definition can select binaries,
set fixed arguments and environment,
declare platform and privilege policy,
and parse one-line or flat JSON package listings.
A resource-bounded disposable fixture confirmed that a custom manager receives its own selector and participates in
`installed`.

For adapters needing dynamic version forwarding,
stateful or multi-line parsing,
multiple subprocesses,
or arbitrary logic,
MPM's documented full-power extension is a Python manager class contributed to or carried with MPM.
Stable 7.6.1 has no separately installed Python entry-point plugin loader,
and its TOML definition DSL rejects `{version}` as an operation placeholder.

This strengthens the proposal:
missing built-in adapters are not automatically permanent gaps.
The design choice is whether each gap fits a local definition,
justifies a full MPM class,
or stays clearer as a provider-specific command at the repository boundary.
[`mpm-manager-extensibility.md`](../troubleshooting/mpm-manager-extensibility.md) contains the source trace
and runtime fixture.

### Restore is machine-scoped and version support is uneven

`mpm dump` writes installed package versions into a TOML manifest.
`mpm restore` reconstructs a versioned `Specifier` for every entry and asks each selected manager to install it.
That format looks lock-like,
but manager adapters can explicitly ignore the version.

For relevant stable 7.6.1 adapters:

- pnpm installation is decorated with `version_not_implemented` and runs
  `pnpm add --global <package>` without the recorded version;
- pipx installation is decorated the same way;
- the Cargo bundled definition runs `cargo install <package>` without `--version`;
- the stew bundled definition runs `stew install <package>` without a tag or version.

The decorator logs a warning and allows the underlying manager to choose a version.
Custom TOML manager definitions cannot forward a restore entry's version through a `{version}` placeholder because
that placeholder is intentionally unsupported.

This is a hard failure only if MPM restore is assigned ownership of one of those pinned paths.
The proposed architecture can instead assign npm tools to pnpm,
Python tools to MPM's version-aware uv adapter,
and pinned Cargo tools to direct Cargo or cargo-binstall commands.
SDKMAN's built-in adapter also passes an explicit version to `sdk install`.
Those alternatives require a declaration-level mapping and runtime proof.

Evidence:

- `meta_package_manager/cli_snapshots.py:400-476`;
- `meta_package_manager/capabilities.py:268-286`;
- `meta_package_manager/definitions.py:402-404` and `1034-1052`;
- `docs/overrides.md:140-144`;
- `meta_package_manager/managers/pnpm.py:215-225`;
- `meta_package_manager/managers/pipx.py:269-280`;
- `meta_package_manager/managers/cargo.toml:41-43`;
- `meta_package_manager/managers/stew.toml:29-31`;
- `meta_package_manager/managers/uv.py:356-367`;
- `meta_package_manager/managers/sdkman.py:179-187`.

### Stable release differs from current documentation

The latest stable release is 7.6.1,
commit `d0404ee2f93f95dac9329e41e47511ef6da4132b`,
published 2026-08-11.
Current documentation includes unreleased 8.0 material.
In particular,
the current `rustup` and `bin` pages mark those adapters as 8.0.0 development additions.
Neither adapter exists in the 7.6.1 source tree.

This matters because Rust toolchains and standalone binaries are required Mise responsibilities.
Assessing current website feature headings without reading their changelog sections overstates stable coverage.

Evidence:
[latest GitHub release][mpm-release],
[current rustup page][mpm-rustup],
[current bin page][mpm-bin],
and the cloned stable tree.

### Platform and artifact surface

MPM publishes standalone Linux,
macOS,
and Windows binaries for x64 and arm64.
The Linux x64 7.6.1 asset is 29,313,224 bytes with GitHub-reported SHA-256
`de58308be281c46cd515ce01af4932ea3b4b5fcbec13d211d987cf463e4014be`.
A sibling attestation asset is published.

The stable source is GPL-2.0-or-later,
contains 32,906 Python lines across its production package,
and declares eight runtime dependencies on Python 3.10 or newer.
These measurements describe audit surface,
not a hard-gate failure.

Evidence:
GitHub release API,
`pyproject.toml:135-287`,
and local stable-source measurements.

### Security model

MPM's normal execution path passes manager commands as argument vectors rather than setting subprocess `shell=True`.
Backend-specific adapters can still invoke a shell explicitly.
The stable SDKMAN adapter uses `bash -c` because `sdk` is a shell function,
and its source quotes the init path before interpolation.
A selected-adapter audit must therefore inspect exceptions rather than inherit the general no-shell claim.

MPM's normal configuration is per-user and is not automatically discovered from the current repository.
An explicitly supplied configuration can redirect binaries and run arbitrary commands,
so project use would need the same trusted-configuration treatment as task files.

Evidence:
[security model][mpm-security] and `meta_package_manager/managers/sdkman.py:41-54,112-120`.

### Maintenance and validation boundary

The project is actively releasing and its latest stable release includes Linux,
macOS,
and Windows artifacts.
The repository is not archived.
A full maintenance sample and upstream CI execution were not completed because the proposed stack does not yet assign
every declaration and runtime boundary to an owner.

No MPM binary was downloaded or executed.
There is no consumer-boundary validation result.

## pnpm evidence

### Task orchestration is a plausible replacement boundary

pnpm 12 workspace tasks are `package.json` scripts.
`pnpm -r run <script>` builds a dependency-aware task graph,
limits concurrency,
skips absent scripts without severing graph edges,
continues independent work with `--no-bail`,
and exposes a stable JSON dry-run graph.

This maps well to the repository's package-scoped build,
test,
and lint families.
It does not directly consume TypeScript task declarations,
so the existing file-enforcer boundary would need to generate package scripts and
`pnpm-workspace.yaml` task relationships.

Evidence:
[pnpm task orchestration][pnpm-tasks] and
[`pnpm-12-4-polyglot-workspace.md`](../troubleshooting/pnpm-12-4-polyglot-workspace.md).

### Pipeline is promising but experimental

`pnpm pipeline` 12.4 performs a frozen install,
affected-project selection,
task graph execution,
result caching,
and aggregate failure reporting.
The official page marks its configuration and output experimental.
The repository's installed pnpm 12.3.4 does not contain this command.

A migration can use stable recursive `pnpm run` first and evaluate pipeline caching separately.
Full removal should not depend on pipeline until a disposable pilot passes the repository's parity gates.

Evidence:
[pipeline documentation][pnpm-pipeline] and the repository troubleshooting report.

### Runtime and ecosystem reach has boundaries

pnpm can install exact Node,
Deno,
and Bun runtimes.
pnpm 12 is itself available as a native executable,
so Node is not required after pnpm is bootstrapped.

Experimental 12.4 Cargo support installs and verifies locked crate sources,
then Cargo compiles them.
It does not install Cargo,
rustup,
components,
or targets.
Experimental Python support builds project environments from an existing Python interpreter;
it does not install that interpreter.

Evidence:
[pnpm runtime documentation][pnpm-runtime],
[pnpm installation documentation][pnpm-install],
[pnpm Cargo documentation][pnpm-cargo],
and [pnpm Python documentation][pnpm-python].

### Current repository version

The current checkout resolves pnpm 12.3.4,
while the assessed feature release is 12.4.1 at commit
`19eb39448649c926bc63b0e9fa16f0e340701460`.
No pnpm 12.4 executable was run against this repository.
The existing source audit records the relevant 12.4.1 implementation paths.

## Current tool declaration mapping

Every one of the 37 current root tool declarations has a plausible provider family.
That makes full removal realistic in principle,
but the mapping also shows why `mpm + pnpm` is an architecture rather than two self-sufficient executables.

### pnpm-centered entries

Seven declarations can move to pnpm itself or the pnpm workspace:

- `pnpm` through a pinned standalone pnpm bootstrap;
- `node` and `bun` through `pnpm runtime`;
- `npm:typescript-language-server`,
  `npm:socket`,
  `npm:wrangler`,
  and `npm:pagefind` through workspace dependencies and scripts.

### Cargo-installed CLI entries

Eleven declarations are Cargo-installed CLI tools:

- `cargo:fd-find`;
- `cargo:fastmod`;
- `cargo:cargo-fuzz`;
- `cargo-binstall`;
- `cargo:timeout-cli`;
- `cargo:cargo-nextest`;
- `cargo:apple-codesign`;
- `cargo:coreutils`;
- `cargo:slint-lsp`;
- `cargo:slint-viewer`;
- `cargo:cargo-ndk`.

MPM 7.6.1 can front Cargo installation,
but its default adapter does not pass restore versions or feature options.
Direct Cargo or cargo-binstall commands generated from the typed tool plan are the stronger current owner for pinned or
option-bearing entries.

### Other native provider entries

- `pipx:slopo` can move to a version-aware uv tool install,
  which MPM can front after uv exists.
- `rust` needs direct rustup ownership for the nightly channel,
  components,
  and Android targets.
- `java` needs a JDK provider;
  stable MPM can front SDKMAN on its supported platforms and pass a requested version.
- `android-sdk` needs a Google SDK bootstrap plus the existing direct `sdkmanager` component installation.
- 15 tools remain platform-package or standalone-binary entries:
  `ripgrep`,
  `harper-cli`,
  `dprint`,
  `uv`,
  `hyperfine`,
  `caddy`,
  `watchexec`,
  `sops`,
  `age`,
  `cmake`,
  `zig`,
  `llama.cpp`,
  `opentofu`,
  `hcloud`,
  and `betterleaks`.

MPM can normalize operations for available platform managers and some standalone installers.
The typed plan still needs per-platform package IDs,
version semantics,
artifact checksums,
and manager bootstrap order.

## Responsibility coverage ledger

### Workspace dependencies

Proposed owner:
pnpm.

Coverage:
strong current fit for npm dependencies;
experimental additional Cargo and Python dependency installation in 12.4.

Gate:
disposable 12.4 install and lockfile pilot.

### Task discovery and execution

Proposed owner:
pnpm recursive run,
with generated package scripts and workspace task relationships.

Coverage:
plausible but unvalidated.

Gate:
preserve namespaced partial invocation,
arguments containing spaces,
working directories,
sequential and parallel groups,
optional package tasks,
failure tolerance,
and aggregate exit status.

### Tool acquisition and locking

Proposed owner:
not fully selected.
pnpm can own Node and Bun runtimes plus npm-distributed CLIs.
Ecosystem-native installers remain necessary for Rust,
JDK,
Android,
Zig,
and standalone binaries.

MPM contribution:
unified invocation and inventory for managers available at invocation start.
A staged outer workflow may change that available set between invocations.

Gap:
stable MPM does not enforce several recorded versions,
does not order manager bootstrap inside one restore,
and does not carry provider-specific component and feature options in its snapshot format.

### Environment and PATH

Proposed owner:
repository-owned task environment setup plus pnpm's workspace-bin PATH behavior.
Whether a separate launcher is necessary depends on the environment inventory and shell-activation decision.

Coverage:
partial.
pnpm exposes workspace bins and Python project environments to scripts.
It does not reproduce Mise's tool-generated environment,
root and package precedence,
or optional shell activation.

### Secrets

Proposed owner:
unselected narrow SOPS and age child-environment launcher.

Coverage from MPM and pnpm:
none for decryption,
redaction,
and child-only secret injection.

### Watching

Proposed owner:
direct package watcher processes launched as pnpm scripts.

Coverage:
plausible.
The repository already invokes rolldown,
TypeScript,
Node,
watch-restart,
and file-enforcer watch modes directly.
`pnpm pipeline --watch` polls repository revisions and is not the relevant local source-file mechanism.

Gate:
verify restart behavior,
argument forwarding,
signal propagation,
child cleanup,
and aggregate watcher failure under pnpm.

### CI,
editors,
root discovery,
and agent policy

Proposed owner:
generated adapters after task,
tool,
and environment interfaces stabilize.

Coverage:
migration work remains.
MPM does not supply a GitHub Action equivalent to the current `jdx/mise-action` setup or
stable project-local tool paths.

## Fit and hard-gate outcomes

### MPM plus pnpm with native providers and repository-owned generation

Outcome:
serious alternative,
not yet hard-gate confirmed or validated.

Reason:
the declaration mapping has a plausible provider family for every current tool,
pnpm has a credible task boundary,
and MPM can provide cross-manager inventory and orchestration.
The exact composition,
bootstrap order,
environment and secret owners,
and platform parity remain unverified.

### MPM plus pnpm without underlying providers or repository adapters

Outcome:
category mismatch.

Reason:
neither tool claims to replace every underlying installer,
secret runtime,
or project-specific environment policy.
This is a warning against an overly literal interpretation,
not a rejection of the user's architecture.

### Keep Mise

Outcome:
valid rollback baseline but excluded from the requested endpoint.

### metapac

Outcome:
not promoted.

Reason:
package desired-state management does not answer task,
environment,
and secret responsibilities,
and switching to it is outside the user's named direction.

### pacdef

Outcome:
fail because upstream is archived.

### Topgrade

Outcome:
category mismatch.

## Scoring and sensitivity

No candidate composition was validated across the full replacement boundary,
so scores and sensitivity calculations are not applicable.
Publishing provisional numbers would allow soft evidence to hide unresolved ownership and parity questions.

## Pros and cons of the proposed pair

### Pros

- pnpm already owns the workspace dependency graph and is the natural home for package scripts.
- pnpm 12.4's task graph has machine-readable inspection and explicit dependency relationships.
- pnpm 12 can bootstrap Node and Bun runtimes after a standalone pnpm bootstrap.
- MPM offers one inventory,
dump,
restore,
SBOM,
and update interface over many machine package managers.
- MPM does not auto-load repository configuration by default.

### Cons

- MPM is not a project environment manager or one universal artifact-lock owner.
- One MPM restore detects its usable managers before restoring packages,
  so bootstrap ordering requires staged invocations.
- MPM's stable restore path ignores recorded versions for pnpm,
pipx,
Cargo,
and stew installations.
- Stable MPM lacks its documented development-branch rustup and bin adapters.
- Neither tool owns Mise's SOPS injection and output redaction.
- pnpm pipeline,
Cargo installation,
and Python installation are new 12.4 experimental surfaces.
- MPM has a substantial cross-manager surface,
  though it stays within the package-management responsibility.

## Migration implication

The smallest evidence-backed next experiment is a disposable two-part pilot while Mise remains the adapter and rollback
path:

- translate representative TypeScript,
  Rust,
  watch,
  and failure-preserving task paths to pnpm 12.4.1;
- exercise an MPM manifest over representative platform,
  Cargo,
  uv,
  and JDK packages,
  including a staged manager bootstrap and explicit pin checks.

Build the canonical tool coverage manifest at the same time,
assigning every one of the 37 current declarations to:

- pnpm runtime;
- pnpm workspace dependency;
- rustup or Cargo tooling;
- JDK and Android tooling;
- platform package manager;
- verified standalone-binary provider;
- intentional retirement.

Use that assignment to decide which providers MPM should coordinate and which need direct commands.
MPM earns a required architecture role only if the pilot shows that its shared inventory,
preview,
failure aggregation,
and cross-platform manifest reduce more complexity than its adapter exceptions add.

## Recommendation boundary

No complete stack is recommended for adoption from this report.

The realism answer is conditional:

- **realistic in principle** as MPM plus pnpm,
  with ecosystem-native providers and repository-owned adapters understood as parts of those boundaries;
- **not a direct configuration translation** because Mise's environment,
  secret,
  component,
  and activation semantics need separate implementations or retirement decisions;
- **not yet evidenced** for a clean-machine cutover,
  because MPM orchestration,
  pnpm 12.4,
  and every remaining provider still need disposable consumer-boundary validation.

## Evidence limits

- MPM was screened from official documentation and stable source,
but not executed.
- pnpm 12.4.1 source was audited previously,
but its executable was not run against this repository.
- macOS and Windows replacement paths were not exercised.
- no secret,
watch,
Android,
desktop,
or clean-machine parity test ran.
- current MPM documentation includes unreleased behavior;
this report separates stable 7.6.1 from development 8.0 material where relevant.

## Evidence records

### Repository context probe

Candidate:
incumbent Mise 2026.9.5 and pnpm 12.3.4.

Claim:
current versions and task/configuration surface.

Commands:
`mise --version`,
`pnpm --version`,
`mise tasks --all --hidden --json`,
`find`,
`rg`,
and `wc` from `/var/home/user/Monochromatic` on Linux x64.

Environment:
current developer checkout;
no credentials requested;
read-only repository inspection intended.

Result:
exit status 0 for cited probes.
The command tool imposed 30-second or 60-second ceilings;
exact elapsed time was not captured.

### MPM stable source audit

Candidate:
Meta Package Manager 7.6.1,
commit `d0404ee2f93f95dac9329e41e47511ef6da4132b`.

Clone:
`~/temp/agent/meta-package-manager-2026-09-12` from
`https://github.com/kdeldycke/meta-package-manager` using `gh repo clone --depth 1 --branch v7.6.1`.

Claim:
restore and relevant default manager adapters do not by themselves form an exact-version project toolchain contract.

Files:
`meta_package_manager/cli_snapshots.py`,
`meta_package_manager/capabilities.py`,
`meta_package_manager/managers/pnpm.py`,
`meta_package_manager/managers/pipx.py`,
`meta_package_manager/managers/cargo.toml`,
and `meta_package_manager/managers/stew.toml`.

Status:
serious alternative with declaration mapping and runtime validation pending.
No third-party candidate command was executed.

### pnpm source audit

Candidate:
pnpm 12.4.1,
commit `19eb39448649c926bc63b0e9fa16f0e340701460`.

Clone:
`~/temp/agent/pnpm-2026-09-11` from `https://github.com/pnpm/pnpm`.

Claim:
pipeline task bodies come from package scripts;
Cargo support installs dependencies but not the Rust toolchain;
Python support requires an interpreter.

Files and excerpts:
recorded in
[`pnpm-12-4-polyglot-workspace.md`](../troubleshooting/pnpm-12-4-polyglot-workspace.md).

Status:
serious component,
consumer-boundary validation pending.

[mpm-home]: https://mpm.run/
[mpm-managers]: https://mpm.run/managers/
[mpm-concurrency]: https://mpm.run/concurrency/
[mpm-security]: https://mpm.run/security/
[mpm-release]: https://github.com/kdeldycke/meta-package-manager/releases/tag/v7.6.1
[mpm-rustup]: https://mpm.run/managers/rustup/
[mpm-bin]: https://mpm.run/managers/bin/
[pnpm-run]: https://pnpm.io/cli/run
[pnpm-tasks]: https://pnpm.io/workspace-task-orchestration
[pnpm-pipeline]: https://pnpm.io/cli/pipeline
[pnpm-runtime]: https://pnpm.io/cli/runtime
[pnpm-install]: https://pnpm.io/installation
[pnpm-cargo]: https://pnpm.io/cargo
[pnpm-python]: https://pnpm.io/python
