# Monorepo manager vet report

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `@monochromatic-dev/dev-script-file-enforcer` is now `@monochromatic-dev/file-enforcer-core`.

Status:
complete;
no candidate is recommendable.
What followed:
the user accepted a from-scratch all-Rust tool on 2026-09-16
([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md)),
designed in
[`doc/planning/monorepo-manager-from-scratch-design.md`](../planning/monorepo-manager-from-scratch-design.md).

Lifecycle phase:
discovery,
screening,
and the re-screen after counting external wrappers are complete;
no candidate reached serious-alternative status.

Subject:
Monorepo manager.

Decision scope:
select a monorepo manager for Monochromatic that provides every file-enforcer functionality
and a watch mode with an RPC or IPC inspection and control channel,
natively or through documented extension,
before plugging functionality into a tool or building a Bazel replacement.

Start date:
2026-09-16.

Last updated:
2026-09-16.

Governing skill commit:
`a05818ad70a40e5769a36de669697ba109891b31`.

Governing skill SHA-256:
`393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
`4e284171b98782db0a833a8c30858ec20900e33677c8f4e53fac6f6e44665995`.
Superseded fingerprints,
newest first:

- `8ce05b698e00ec017f52b92abf9df17175cc7fcb8f6678dd97120ae336b290aa`,
   before the user counted an external wrapper as plugging in;
   screening had completed with no survivor under it.
- `93965544ecfaae75f635465fbeb32fa6ab8f19696624723bf0f71456f5586470`,
   before the user made any documentation confusion an immediate cull;
   screening agents were running and received the stricter rule mid-run,
   and merged results apply it retroactively to every recorded signal.
- `39dc93902e80f8a713cf55d3a0d2b6ff24651344785f88c5a75ec6fabc5d36c6`,
   before the user clarified that documentation structure is part of the documentation gate;
- `8f69b0cf08360061800fa6b0d8197fb8c20e22171d23d191348bc3c296cc01d4`,
   before the user added the documentation gate;
- `cafbebe56ed9aa8f5ed2dc0f500fa351c79754fcdd56dd4632b38e7626fe0898`,
   before the user allowed Meta Package Manager to own OS package provisioning;
- `340bcda696599b81be2d3314af769ed3c774d3914ad724a32f148d9d975eec89`,
   before the user added the control requirement.

Active audit owner:
Claude Code session `e28ad59c-f3f5-46e9-8c8c-59a61c331610`.

Prior compatible report:
none.
Related but incompatible:
[`tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`](tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md)
covers tool provisioning and task execution without the file-enforcer or inspection requirements.
[`bazel-migration-dx.md`](../research/bazel-migration-dx.md) is a single-candidate research note.

## Context

Measured on 2026-09-16:

- Incumbent task runner:
   Mise `2026.9.5` on Linux x64,
   1,825 tasks,
   0 with `sources` or `outputs`.
- Incumbent generation and enforcement:
   `@monochromatic-dev/dev-script-file-enforcer`,
   workspace source.
- Workspace:
   176 `package.json`,
   19 `Cargo.toml`,
   Gradle Kotlin DSL builds under `package/music-player/android-app/` and `package/linter/kotlin/`,
   Zig `0.15.2`.
- One active author;
   GitHub Actions CI is being expanded.
- The user states insufficient Mise documentation as the reason to migrate.

Requirements source:
`doc/planning/mise-removal-coverage.md`,
section "Monorepo manager hard requirements".

## Requirement checklist

### File-enforcer functionality

Derived from exports in `package/dev-script/file-enforcer/src/index.ts` and the CLI in `src/cli.ts`,
not from the package README,
which predates the current source layout.

- FE01 Configuration that expresses the generation and enforcement rules,
   with author control over sequencing and parallelism.
   The user stated on 2026-09-16 that the configuration does not have to be TypeScript;
   file-enforcer's direct TypeScript execution is incumbent behavior,
   not a requirement.
- FE02 CLI discovers `file-enforcer.config.ts` upward from the working directory.
- FE03 File and glob reads with glob provenance:
   `cat`,
   `globResults`,
   `expandGlob`.
- FE04 In-memory read cache with targeted invalidation:
   `readCache`,
   `readCached`,
   `updateCache`,
   `invalidatePaths`.
- FE05 Writes that skip identical content and replace atomically:
   `overwrite`,
   `readExisting`,
   `src/io/write-atomic.ts`.
- FE06 Create-only atomic writes:
   `overwriteIfNotExists`.
- FE07 Glob mirroring with positional wildcard substitution:
   `overwriteEach`,
   `mirrorGlobPath`.
- FE08 Lazy content builders skipped when captured inputs are unchanged.
- FE09 Persistent staleness manifest with content hashes,
   glob expansions,
   destination metadata,
   and a cross-process lock with owner recovery
   (`src/io/staleness-*.ts`).
- FE10 JSON transforms:
   parse,
   format,
   flat merge,
   default merge,
   key omission,
   property path lookup.
- FE11 TOML reads and comment-preserving splice edits:
   `getTomlProperty`,
   `editTomlKey`,
   `overwriteTomlKey`.
- FE12 XML entry listing,
   lookup,
   replace-or-insert,
   and attribute escaping.
- FE13 Line deduplication:
   `dedup`.
- FE14 Cargo manifest enforcement plans:
   `manageCargoManifests`.
- FE15 JetBrains LSP4IJ settings management and options-directory discovery.
- FE16 Command execution with captured output:
   `exec`.
- FE17 Platform dispatch through ordered,
   nestable predicate commands with per-session cached evaluation:
   `exec(platformCommands)`,
   `evaluatePredicate`.
- FE18 OS package provisioning across apt,
   dnf,
   pacman,
   apk,
   zypper,
   brew,
   winget,
   scoop,
   and choco,
   with privilege detection:
   `ensurePackage`,
   `detectManager`,
   `canProvide`,
   `installPackage`.
- FE19 Package index generated from Repology plus hand-maintained overrides:
   `p`,
   `mergeOverrides`,
   `registerPackages`,
   `src/package/mise.generate-index.ts`.
- FE20 Read,
   write,
   glob,
   and timestamp tracking API:
   `trackRead`,
   `trackDest`,
   `trackGlob`,
   `captureTrackedSources`,
   `addWatchedPaths`.
- FE21 Watch mode rerunning the configuration when tracked sources change
   (`--watch`,
   `startWatching`).
- FE22 Protected destinations:
   external edits to managed outputs are reverted and reported through a desktop notification
   (`src/watch/watch.ts:28-29`,
   `src/watch/notify.ts`).
- FE23 Watch echo suppression from recorded write timestamps,
   debounced batched reruns,
   and watcher restart supervision
   (`src/watch/watch-rerun-queue.ts`,
   `src/watch/watch-supervisor.ts`).
- FE24 Tagged structured logging.

### Delegation of FE18 and FE19

The user allowed Meta Package Manager to replace FE18 and FE19 on 2026-09-16.
Meta Package Manager `v7.6.1` ships adapters for every manager FE18 dispatches to:
`apk.py`,
`apt.py`,
`dnf.py`,
`homebrew.py`,
`pacman.py`,
`scoop.py`,
`winget.py`,
`zypper.py`,
and `choco.toml`
(`gh api 'repos/kdeldycke/meta-package-manager/contents/meta_package_manager/managers?ref=v7.6.1'`).

Parity items not yet checked:
privilege escalation,
the binary-name to per-manager package-name mapping that FE19's index supplies,
such as `rg` to `ripgrep`,
and the check-before-install behavior.
Version enforcement gaps are recorded in
[`tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`](tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md);
FE18 records no versions either.

### Watch plus inspection pattern

- WR1 A persistent watch process keeps task outputs up to date after source changes.
- WR2 Another process can ask the running watch process what it is doing,
   covering at least current and recent activity,
   through a documented RPC or IPC channel.
- WR3 Through the same channel,
   another process can control the running watch process,
   at least starting or rerunning work and cancelling running work.

## Frozen hard constraints

- HC1 Every FE item is available natively,
   through a documented extension mechanism,
   or through an external wrapper,
   except FE18 and FE19,
   which may be delegated to Meta Package Manager.
- HC2 WR1 is available natively,
   through a documented extension mechanism,
   or through an external wrapper.
- HC3 WR2 and WR3 are available natively,
   through a documented extension mechanism,
   or through an external wrapper.
   On 2026-09-16 the user chose to count wrapping a tool from outside as plugging in.
- HC4 Inspectable open-source local execution.
- HC5 Works on repository-supported local and CI platforms:
   Linux x64 development,
   GitHub-hosted Linux,
   macOS,
   and Windows runners.
- HC6 Runs arbitrary command tasks for TypeScript,
   Rust,
   Kotlin with Gradle,
   and Zig workspace packages.
- HC7 Official documentation covers every consumed feature,
   and any confusion or frustration while reading or trying to read it culls the candidate immediately.
   The user added this gate on 2026-09-16:
   cull wildly underdocumented candidates early.
   The user then clarified with moon screenshots that incoherent documentation structure counts,
   not only missing pages,
   and finally directed an immediate cull on any confusion or frustration,
   with no second chances.

### HC7 screening test

Recorded operationalization,
open to user correction.
Official documentation means the project's own docs site,
reference pages,
or repository docs.
A feature named only in a changelog,
issue,
blog post,
or source code counts as undocumented.

Withdrawn extension-mechanism requirement:
the first screening pass also required a documented extension mechanism,
reasoning that a wrapper around a tool is building rather than plugging in.
The user rejected that reading on 2026-09-16 by counting wrappers as plugging in,
so candidates that exited only on it are re-screened.

Coverage:
a candidate exits when its official documentation lacks any of:

- a configuration reference;
- a CLI or API reference for the commands this repository would run;
- behavior-level documentation for each native surface it would supply for WR1,
   WR2,
   or WR3.

Confusion:
a candidate exits the moment the reader hits any confusion or frustration reading or trying to read its documentation.
Reading stops there;
the report records the triggering URL and what confused the reader.
Documentation must be readable without JavaScript and without strict bot blocking,
per the user's stated opinion on 2026-09-16.
Each relied-on page is fetched once with plain `curl --location` and a desktop Firefox user agent.
The candidate exits when that first request returns a bot challenge,
captcha,
or 403,
429,
or 202 block page,
or when the documentation text is absent from the returned HTML.
There is no browser retry;
an earlier browser-retry exception is withdrawn.

Every structure signal is a confusion trigger and therefore an exit:

- index or overview pages whose entries lack usable summaries,
   such as descriptions cut off mid-sentence or entries that show only a child count;
- navigation sections that list unrelated kinds of pages as flat siblings with no grouping by purpose;
- one consumed feature spread across several sections without a page that ties them together;
- current documentation that depends on a separately frozen version for consumed behavior;
- a page whose description of what a command does contradicts its own documented defaults,
   so following the description produces surprising behavior;
- consumed behavior that users can learn only by asking maintainers in chat,
   such as Discord,
   instead of from official documentation.

Other triggers include broken or missing pages,
not knowing where a feature is documented,
and needing sources outside official documentation to understand consumed behavior.
Documentation quality among candidates that pass remains a weight-5 soft criterion.

### Category fit for screening

Recorded 2026-09-16 after the GitHub pass counted general task runners as in scope,
which the user's words "monorepo manager" do not support.

- Monorepo manager:
   models a workspace of multiple projects or packages and runs tasks across them using relationships between projects.
- Component:
   a persistent watch process with an RPC,
   IPC,
   or HTTP interface that can compose with a monorepo manager.
- Category mismatch:
   a command runner without a workspace or project model,
   such as a Makefile-style recipe runner,
   unless it qualifies as a component.

Screening order:
category fit,
then the HC7 documentation gate,
then the remaining hard constraints.

### Screening exits needing a note

- Fabr:
   `https://fabr.build/` resolves only to `67.205.27.5`,
   and plain `curl` from the development machine timed out connecting on two further attempts on 2026-09-16,
   including a forced IPv4 attempt,
   while other documentation hosts loaded from the same machine.
  A remote fetcher loaded the page.
  The cause is unknown;
   the exit stands because a reader on this machine cannot open the documentation.

### User-directed exclusions

- moon:
   excluded on 2026-09-16 by the user's judgment that its documentation is wildly underdocumented
   and structurally hostile to readers.
  Evidence is the user's screenshots of `moonrepo.dev` on 2026-09-16:
  - the Commands overview shows summaries cut off mid-sentence,
     such as "will generate and serve a visual graph of",
     and the `daemon`,
     `docker`,
     `extension`,
     `query`,
     and `sync` entries show only an item count;
  - the Guides menu lists continuous integration,
     code owners,
     daemon,
     Docker integration,
     extensions,
     MCP integration,
     offline mode,
     remote caching,
     Renovate,
     terminal notifications,
     WASM plugins,
     webhooks,
     and VCS hooks as flat siblings;
  - a banner states that the documentation is for moon v2 and that v1 documentation is frozen elsewhere.
- moon,
   user-reported experience:
   deleting almost all caches required `moon clean` with a lifetime of 1 second,
   the behavior was unexpected,
   and the user had to ask the primary developer on Discord.
  Current source `moonrepo/moon` `website/docs/commands/clean.mdx`,
   last changed in `2ef35f144` on 2026-02-18,
   says the command "can be used to reset the workspace entirely",
   while its option list gives `--lifetime` a default of "7 days" without connecting the two.
  The same page labels `--all`,
   "Clean all cached items and reset state",
   as added in 2.0.0.

## Components and overlays

Base category:
inspectable open-source local technology.
A candidate with a hosted control plane receives a second managed-service classification for that component.

Overlays:

- incumbent dependency replacement;
- high-trust execution in developer-machine and CI boundaries;
- native,
  Wasm,
  prebuilt binary,
  or generated-code boundary;
- multi-platform claim.

## Tiering from the stated fallback order

The user's order is lexicographic and sits outside score arithmetic:

1. Tier A:
   every FE item except FE14,
   FE15,
   FE18,
   and FE19 is native,
   and WR1,
   WR2,
   and WR3 are native.
   FE14 and FE15 are repository-specific policies,
   which the user classifies as pluggable by definition;
   Tier A still requires a documented extension mechanism that can express them.
   FE18 and FE19 belong to Meta Package Manager when delegated.
2. Tier B:
   some requirement is met only by plugging functionality into the tool through a documented extension mechanism.
   Each plugged item records the extension point and what must be built.
3. Tier C:
   no tool qualifies and plugging is judged too hard;
   the user would consider building a Bazel replacement.

No fix difficulty is estimated for unbuilt plug-ins.

## Frozen soft criteria

Weights are 1 unless a stated preference sets them.

- Documentation quality for consumed features,
  weight 5,
  because insufficient documentation is the stated reason for leaving Mise.
- Native requirement coverage within a tier,
  weight 1.
- Watch inspection and control depth,
  weight 1.
- Cache correctness and affected-only execution,
  including input enforcement,
  weight 1.
- Community and ecosystem,
  weight 1.
- Maintenance evidence,
  weight 1.
- Polyglot repository fit,
  weight 1.
- Migration surface from Mise,
  weight 1.
- Human auditability,
  weight 1.

## Unresolved preferences

- Whether control must extend beyond starting,
  rerunning,
  and cancelling work.
- Whether an undocumented but inspectable internal protocol can satisfy WR2.
- Relative weight of community size;
  the user raised it without a magnitude.

## Discovery protocol

### Frozen query schedule

Registry and ecosystem index queries:

- npm registry search API,
   `text=keywords:monorepo`,
   popularity-weighted,
   250 per page.
- npm registry search API,
   `text=keywords:build-system`.
- npm registry search API,
   `text=keywords:task-runner`.
- crates.io API,
   `q=build system`,
   sorted by downloads,
   100 per page.
- crates.io API,
   `q=task runner`.
- crates.io API,
   `q=monorepo`.
- `https://monorepo.tools` tool comparison.
- `korfuri/awesome-monorepo` README build-tool sections.

Repository-host queries:

- `gh search repos --topic monorepo --sort stars`,
   100 per page.
- `gh search repos --topic build-system --sort stars`.
- `gh search repos --topic build-tool --sort stars`.
- `gh search repos --topic task-runner --sort stars`.
- `gh search repos "watch mode" daemon build --sort stars`.

Broader web queries:

- `monorepo build tool daemon watch mode API inspect running tasks`
- `build system daemon status command show running actions`
- `Bazel alternative monorepo build system 2026`
- `Nx alternative monorepo task runner`
- `Turborepo alternative`
- `task runner watch mode HTTP API status`
- `monorepo tool code generation keep generated files in sync`
- `incremental build system file watcher keeps outputs up to date continuously`

Repository queries:

- Mise configuration and `mise watch` use;
- file-enforcer source;
- prior reports and plans:
   `tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`,
   `mise-keep-vs-build-own.md`,
   `mise-removal-coverage.md`,
   `bazel-migration-dx.md`;
- `rg` for turbo,
   nx,
   moon,
   wireit,
   lage,
   rush,
   tilt,
   buck2,
   pants,
   gradle,
   nadle.

Expansion round,
frozen 2026-09-16 after collecting the "New taxonomy terms" sections of every discovery ledger.
Terms were de-duplicated into distinct concepts;
concepts already covered by the initial schedule were not repeated.
Later taxonomy terms are recorded without appending queries.

GitHub `search/repositories`,
sorted by stars,
100 per page:

- E1 `"build server protocol" in:name,description`
- E2 `"file watcher" daemon in:description`
- E3 `"task runner" daemon in:description`
- E4 `"json-rpc" task in:description`
- E5 `"process manager" api in:description`
- E6 `"continuous build" in:description`
- E7 `"remote execution" build in:description`
- E8 `"project graph" in:description`
- E9 `affected monorepo in:description`
- E10 `"persistent tasks" OR "long-running tasks" in:description`
- E11 `"live update" watch in:description`
- E12 `"sync generator" OR "code generation" monorepo in:description`

npm registry search API,
popularity-weighted,
250 per page:

- E13 `text=task runner daemon`
- E14 `text=monorepo watch daemon`
- E15 `text=build orchestrator`

crates.io API,
sorted by downloads,
100 per page:

- E16 `q=build orchestrator`
- E17 `q=file watcher daemon`

Exact repository lookups for tools the ledgers name but topic queries cannot return:

- E18 Buck2,
   Wireit,
   Watchman,
   just,
   Mage,
   batect,
   Dagger,
   watchexec,
   Tilt,
   process-compose,
   pueue,
   pitchfork,
   Nadle,
   Earthly,
   and Aspect CLI.
   Mise is the incumbent and Lage is already queued.

### Early WR evidence recorded before the candidate ledger

Collected 2026-09-16 while discovery ran;
each tool still needs discovery provenance and full screening.

- Bazel `8e90a0d`:
   no native watch command;
   `CommandServer` exposes `Run`,
   `Cancel`,
   `UpdateTerminalSize`,
   and `Ping`;
   one command at a time per output base.
  `ibazel` v0.33.0 reports outward only.
  Details in `doc/research/bazel-migration-dx.md`.
- Tilt v0.37.7,
   released 2026-08-15,
   repository pushed 2026-09-14:
   `internal/cli/` contains `trigger`,
   `get`,
   `describe`,
   `enable`,
   `disable`,
   `wait`,
   and `logs` commands.
  `tilt trigger` posts to the running Tilt process's HTTP API
   (`internal/cli/trigger.go:61-63`).
  A GitHub code search for `cancel update` returned no files;
   that search does not prove cancellation is absent.
- Gradle Tooling API documentation
   (<https://docs.gradle.org/current/userguide/tooling_api.html>)
   covers running builds,
   cancelling a running build,
   and progress events,
   but does not mention continuous build through the Tooling API
   or connecting to a build started by another process.
- Nx daemon reference
   (`nrwl/nx` `astro-docs/src/content/docs/reference/nx-daemon.mdoc:9-24,60,109-110`):
   the daemon watches workspace files to keep project graph data current,
   clients reach it through a Unix socket,
   `nx daemon` prints its process ID and log path,
   and the socket is described as a remote for code execution.
  The page documents no client protocol for outside tools and no task control through the daemon.

### Query ledger

Full per-query and per-page ledgers are appendices in
[`tech-monorepo-manager-vet-2026-09-16/`](tech-monorepo-manager-vet-2026-09-16/).

- Repository-host class,
   `discovery-github.md`:
   `topic:monorepo` (9,379 results),
   `topic:build-system` (1,125),
   and `topic:build-tool` (2,466) are blocked by the GitHub 1,000-result search cap,
   still adding candidates on their final pages.
  A `stars:>=50` narrowing added no new repositories.
  The unread tails hold repositories with 13 or fewer stars.
  On 2026-09-16 the user chose to skip enumerating them,
   so this class is recorded as not saturated by user decision;
   any recommendation rests on that waiver.
  `topic:task-runner` (972) was exhausted without saturating.
  `"watch mode" daemon build` returned 1 result and was exhausted.
  Topic queries cannot return Buck2,
   Wireit,
   Lage,
   Watchman,
   just,
   or Mise,
   which carry no GitHub topics.
- Web class,
   Linkup,
   `discovery-web-repo.md`:
   blocked for pagination;
   Linkup has no offset parameter and returned at most 20 results per query.
- Web class,
   DuckDuckGo HTML endpoint,
   `discovery-web-paged.md`:
   W1,
   W2,
   and W4 met the two-empty-page stop rule;
   W3 and W5 stopped early;
   W6 to W8 were not fetched.
  The subagent reports that the user stopped this run with "We already have enough from DDG".
  Mojeek and Bing returned bot challenges.
  The run added Lage.
- Repository class:
   results in `discovery-web-repo.md`,
   section "Repository findings".
- Registry class,
   `discovery-registry.md`:
   npm `keywords:monorepo` (4,145 results),
   `keywords:build-system` (48),
   `keywords:task-runner` (330),
   crates.io `task runner` (1,771),
   monorepo.tools `/compare`,
   and `korfuri/awesome-monorepo` were exhausted;
   crates.io `build system` and `monorepo` met the two-empty-page stop rule.
  The crates.io `build system` result is dominated by general libraries under download sorting.
  None of the npm keyword queries returned `nx`,
   `turbo`,
   `wireit`,
   `lage`,
   or `@moonrepo/cli`.
  375 lenient survivors;
   69 duplicate candidates already queued from other classes.

The discovery pass counted plain command runners as in scope;
section "Category fit for screening" corrects that for screening.

### Candidate ledger

Every candidate has its own subsection,
with evidence and final gate,
in the screening appendices of
[`tech-monorepo-manager-vet-2026-09-16/`](tech-monorepo-manager-vet-2026-09-16/):

- `screening-primary-chunk-1.md`,
   `screening-primary-chunk-2.md`,
   `screening-primary-chunk-3.md`:
   87 candidates from the GitHub and web classes.
- `screening-registry-chunk-1.md`,
   `screening-registry-chunk-2.md`,
   `screening-registry-chunk-3.md`:
   204 candidates from the registry class.
- `screening-github-runners-category.md` and `screening-registry-runners-category.md`:
   424 command runners checked for category fit.
- `screening-promoted-runners.md`:
   the 15 GitHub runners that passed category fit.
- `discovery-expansion.md`:
   the expansion round,
   17 new candidates,
   and the two registry components that passed category fit.

## Screening results

732 screening entries,
counting moon and Earthly twice because two discovery classes surfaced each:

- G1 category mismatch:
   536.
- G2 license or missing source:
   42.
- G3 documentation:
   151,
   most under the confusion rule.
- User exclusion:
   moon,
   twice.
- Duplicate skip:
   Earthly.
- Survivors:
   none.

Candidates that met every documentation coverage requirement and exited only on a confusion trigger:
Bazel,
Gradle,
Rush,
pnpm,
pitchfork,
and magus.
Confusion triggers spot-checked with plain `curl` from the development machine on 2026-09-16:

- Bazel:
   `https://bazel.build/docs/configurable-attributes` links "Configurable Build Attributes" to both
   `/configure/attributes` and `/docs/configurable-attributes`.
- pnpm:
   `https://pnpm.io/cli/recursive` and `https://pnpm.io/workspace-task-orchestration` describe `--no-bail` differently.
- oneRepo:
   `https://onerepo.tools/docs/config/` documents `lifecycles` under `taskConfig`
   while its example sets it under `tasks`.
- Nadle:
   `https://nadle.dev/docs/getting-started/features/` configures `CopyTask` with `to`,
   while `https://nadle.dev/docs/guides/file-operation-tasks/` uses `into`.
- Tilt:
   `https://docs.tilt.dev/file_changes.html` refers to a hard-coded list of ignored editor temp files without listing it
   and asks readers who hit an unlisted file to file a bug.

Several candidates documented pieces of inspection or control of a running process,
including Tilt,
pitchfork,
devtooie,
and Rush through `rush-serve-plugin`;
each exited on a confusion trigger before capability evidence was completed.
An earlier version of this sentence claimed no candidate documented both,
which the screening appendices do not support.

Leads recorded after the frozen schedule,
not queried or screened:
`neptaco/mcproc`,
the omni task runner named by `@omni-oss/task-bench`,
and PM2.

## First-pass result

Under fingerprint `8ce05b698e00ec017f52b92abf9df17175cc7fcb8f6678dd97120ae336b290aa`:
no serious alternative.
Every discovered candidate exited at a screening hard gate.
The user then counted external wrappers as plugging in,
which reopens candidates whose only recorded exit was the extension-mechanism requirement.

Evidence limits:

- The GitHub topic tails beyond the 1,000-result cap were skipped by user decision.
- The DuckDuckGo web run was stopped by the user after W1,
   W2,
   and W4 saturated.
- The confusion and no-JavaScript rules arrived while screening agents were running;
   agents applied them retroactively to their recorded evidence.
- Fabr's exit depends on a connection timeout from the development machine with unknown cause.
- GitHub returned HTTP 429 for rendered pages after heavy request volume;
   agents read repository Markdown through `gh api` instead,
   and no verdict depended on those responses.

The extension-mechanism requirement was this report's operationalization of HC3,
not a user statement.

## Re-screen after counting wrappers

`rescreen-list.md` classifies all 151 G3 exits.
20 rested only on the withdrawn requirement,
and devtooie is added because its only other failure was one HTTP 429 from GitHub after heavy request volume.
The other 131 stay out on another recorded G3 failure.

Correction:
an earlier summary in this session said Turborepo,
Lerna,
and Melos exited only on the extension-mechanism requirement.
Their sections also record G3 failures under the retroactive rules:

- Turborepo:
   `https://turborepo.dev/docs/reference/configuration` says the daemon still backs `turbo watch`,
   while `https://turborepo.dev/docs/reference/watch` never mentions the daemon.
- Lerna:
   command options are documented in per-command GitHub READMEs.
- Melos:
   documentation is served as raw MDX without JavaScript.

Candidates whose only confusion trigger sits in plugin documentation,
such as Yarn,
Pants,
and Please,
stay out because the user's confusion rule applies to any documentation read,
not only consumed features.

Re-screen results,
in `rescreen-result-1.md` and `rescreen-result-2.md`:
all 21 exited on documentation confusion triggers checked against live pages.
Examples:

- Watchman:
   `https://facebook.github.io/watchman/docs/install` says Linux binaries build on Ubuntu 20.04,
   while the release workflow now uses `ubuntu-24.04`,
   and keeps a section for macOS 10.6 and earlier.
- devtooie:
   `docs/configuration.md` says a control-API restart starts a stopped package,
   while `docs/package-lifecycle.md` says restart applies to running packages.
- dev-process-manager:
   Getting Started requires Node.js 18 or later,
   while published 4.1.0 declares `engines.node` `>=22`.
- zinoma:
   the README links to a nonexistent `variant.Paths.field.paths` anchor.

## Terminal result

No serious alternative,
under both the original constraints and the relaxed wrapper rule.
Every discovered candidate exited at a screening hard gate,
so no finalist validation,
scoring,
or ranking applies,
and no candidate is recommendable.

Under the user's fallback order,
plugging functionality into an existing tool needs a host that passes HC7,
and none did;
the remaining stated fallback is building a Bazel replacement.
This report does not propose or evaluate a custom implementation.
