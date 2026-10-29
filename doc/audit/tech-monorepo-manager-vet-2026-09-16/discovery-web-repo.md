# Monorepo manager discovery: web and repository

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/done-postcss` is now `package/done/variant-postcss`;
> `package/desktop-daemon/hall-monitor` is now `package/hall-monitor/daemon`;
> `package/webapp-productivity/done` is now `package/done/app`;
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`;
> `package/webapp-productivity/rss` is now `package/rss/app`;
> `package/webapp-productivity/wc` is now `package/wc/app`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `@monochromatic-dev/dev-script-file-enforcer` is now `@monochromatic-dev/file-enforcer-core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Run date: 2026-09-16.
Scope: monorepo managers that run arbitrary command tasks across workspace packages,
plus components with a persistent watch or daemon process and an inspection RPC, IPC, or HTTP API.
Excluded categories: templates, bundlers, CI services, libraries.

Screening is discovery-depth only.
A "survivor" passed the category screen from result content;
no hard gate, maintenance check, or source audit was run.
Capability claims marked "recall" were not observed in any result and need verification.

## Query ledger

Provider behavior common to every query:

- Provider: Linkup (`mcp__linkup__linkup-search`); it did not fail, so WebSearch was not used.
- Parameters: `depth: fast`, `maxResults: 20`; no domain, date, or image filters.
- Pagination: Linkup exposes no page or offset parameter.
  To test whether more results existed, W1 to W4 were re-requested with `maxResults: 60`.
  Each returned exactly 20 results;
  for W3 a URL diff against the 20-result call showed 0 new URLs,
  and W1, W2, W4 returned the same URLs in the same order.
  W5 to W8 were not re-requested because the cap was already established.
  The "continue until two consecutive pages add no new survivor" rule therefore could not be applied;
  every query stopped after one page.

### W1

- Query: `monorepo build tool daemon watch mode API inspect running tasks`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1 (re-request at maxResults 60 returned the same 20)
- Result count: 20
- New candidates: Turborepo, Nx, moon, mise, Lerna, pnpm workspaces, npm workspaces, Yarn workspaces, Task (go-task),
- Melos,
  devenv, Hermit, Develocity, an unnamed remote execution platform and an unnamed Nix environment manager (tool.news
  listing),
  `monorepo-watch`, RepoOps, `concurrently`, Bazel (plus Nx MCP server and Bazel MCP server mentions)
- New screening survivors: Turborepo, Nx, moon, Bazel, pnpm workspaces and pipeline, Lerna (weak), Task, Melos (weak)
- Stop reason: provider has no pagination; single capped page

### W2

- Query: `build system daemon status command show running actions`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1 (re-request at maxResults 60 returned the same 20)
- Result count: 20
- New candidates: Gradle, pitchfork, `git fsmonitor--daemon`, `dmypy`, systemd, `start-stop-daemon`, moadim, MCPProxy,
  Bubbaloop, Aerospike daemon (moon reappeared with `moon daemon status`)
- New screening survivors: Gradle, pitchfork (component), systemd (component, weak)
- Stop reason: provider has no pagination; single capped page

### W3

- Query: `Bazel alternative monorepo build system 2026`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1 (re-request at maxResults 60 returned the same 20 URLs, verified by diff)
- Result count: 20
- New candidates: Buck2, Buck (v1), Pants, Mill, Rush, Earthly, Aspect (Aspect Build), BuildBuddy, EngFlow, Gazelle,
  Sapling, Sourcegraph, Aviator, Endor Labs, Railway
- New screening survivors: Buck2, Pants, Mill, Rush, Earthly (pending maintenance check), Aspect CLI (component,
- unverified)
- Stop reason: provider has no pagination; single capped page

### W4

- Query: `Nx alternative monorepo task runner`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1 (re-request at maxResults 60 returned the same 20)
- Result count: 20
- New candidates: Nx Cloud, Nx Console (VS Code extension mention), monorepo.tools
- New screening survivors: none
- Stop reason: provider has no pagination; single capped page

### W5

- Query: `Turborepo alternative`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1
- Result count: 20
- New candidates: Angular, Svelte, npm (alternativeto listing), DevZero, Buildkite, Storybook, Bun, Vercel, Pika Pack,
  esbuild, Webpacker, Broccoli, `turbo devtools` (feature), topcodetools listing of 26 unnamed tools
- New screening survivors: none
- Stop reason: provider has no pagination; single capped page

### W6

- Query: `task runner watch mode HTTP API status`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1
- Result count: 20
- New candidates: Apache Druid Tasks API, `doowb/composer`, Watchtower HTTP API, Taskcluster, Kiro, Control Plane Task
- Runner,
  `bvaughn/task-runner`, Anchor Browser, Browser Use, RunningHub, Nowledge Mem watcher, Apify HTTP Request Runner,
  EntryScape Taskrunner, Node.js test runner `--watch`
- New screening survivors: none
- Stop reason: provider has no pagination; single capped page

### W7

- Query: `monorepo tool code generation keep generated files in sync`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1
- Result count: 20
- New candidates: `nx sync` (Nx feature), `@hey-api/openapi-ts`, watchdog (Python), pre-commit, Buf,
  GraphQL Code Generator, OpenAPI Generator and openapi-typescript (mentioned), `sync-monorepo-packages`, Spago,
  Symplify MonorepoBuilder, Tainted, Versio, Layer-pack, Builder (FormidableLabs), tomono, `shopsys/monorepo-tools`,
  Semaphore CI, CODEOWNERS generator, Write Guard
- New screening survivors: none
- Stop reason: provider has no pagination; single capped page

### W8

- Query: `incremental build system file watcher keeps outputs up to date continuously`
- Provider: Linkup, depth fast
- Filters: none
- Pages: 1
- Result count: 20
- New candidates: MSBuild, JetBrains Rider incremental build, IncrediBuild, CocoIndex (with LanceDB), GNU direvent,
  `inotify-hookable`, `inotifywait` (Gradle reappeared with file system watching and continuous build)
- New screening survivors: none
- Stop reason: provider has no pagination; single capped page

### Saturation status

- New survivors by query: W1 8, W2 3, W3 6, W4 0, W5 0, W6 0, W7 0, W8 0.
- Per-query page saturation: not testable (provider cap of 20, no paging).
- Schedule-level signal: the last five queries in schedule order added no new survivor,
  but that is ordering evidence, not a two-empty-page stop under the frozen rule.

## Candidates

### Monorepo manager survivors

#### Turborepo

- URL: https://turborepo.dev (also https://turbo.build/repo/docs/reference/watch)
- Discovered by: W1, W3, W4, W5
- Category guess: monorepo manager
- Screening note: survivor.
  Runs `package.json` scripts as tasks across workspace packages with caching.
  Results show `turbo watch` restarting dependent tasks, a "daemon mode for background caching",
  `turbo devtools` that hot-reloads package and task graphs,
  `--dry=json`, `--summarize` run reports,
  and experimental OpenTelemetry run metrics.
  No result showed a documented API for inspecting what a running `turbo watch` is doing.
  Non-JS projects still need a `package.json` wrapper (nx.dev comparison page).
- Taxonomy terms: daemon, watch mode, persistent tasks, devtools, package graph, task graph, dry run JSON,
  run summary, OTLP run metrics, boundaries, remote cache

#### Nx

- URL: https://nx.dev
- Discovered by: W1, W3, W4, W5, W7
- Category guess: monorepo manager
- Screening note: survivor.
  Task runner with caching, affected selection, plugins, and generators;
  results mention an Nx MCP server for agents, `nx graph`, task sandboxing, inferred tasks,
  and `nx sync`, which keeps TypeScript project references synchronized and checks them before `build`, `serve`, or
  `dev`.
  Nx Cloud is a separate hosted layer and is excluded.
- Taxonomy terms: sync generators (`nx sync`), generators, inferred tasks, project graph, affected, task sandboxing,
  MCP server, agent skills, distributed task execution, self-healing CI

#### moon (moonrepo)

- URL: https://moonrepo.dev (daemon status page https://moonrepo.dev/docs/commands/daemon/status)
- Discovered by: W1, W2, W3, W4, W5
- Category guess: monorepo manager
- Screening note: strongest observed match for the watch plus inspection requirement.
  moon v2 documents `moon daemon status`, which reports PID, a Unix socket or Windows named pipe IPC endpoint,
  uptime, PID file, and log file.
  Whether the IPC protocol is documented for third-party inspection was not observed.
  Polyglot task runner with toolchain management (proto).
- Taxonomy terms: daemon, IPC socket, named pipe, daemon guide, cache inspection, project:task targets, toolchain

#### Bazel

- URL: https://bazel.build
- Discovered by: W1, W3, W4, W5
- Category guess: monorepo manager (hermetic build system)
- Screening note: survivor; already assessed in `doc/research/bazel-migration-dx.md`.
  Results mention a Bazel MCP server with `bazel query`,
  Build Event Protocol output converted to OpenTelemetry traces (EngFlow Munich meetup link),
  and Gazelle BUILD file generation.
- Taxonomy terms: hermetic builds, remote execution, Build Event Protocol (BEP), MCP server, BUILD file generation

#### Buck2

- URL: https://buck2.build
- Discovered by: W3, W4
- Category guess: monorepo manager (hermetic build system)
- Screening note: survivor; results only place it with Bazel and Pants for polyglot hermetic builds.
- Taxonomy terms: hermetic

#### Pants

- URL: https://www.pantsbuild.org
- Discovered by: W3
- Category guess: monorepo manager (build system)
- Screening note: survivor; described as Python-oriented with automatic dependency inference.
- Taxonomy terms: dependency inference

#### Gradle

- URL: https://gradle.org (daemon doc https://docs.gradle.org/current/userguide/gradle_daemon.html)
- Discovered by: W2, W3, W8
- Category guess: monorepo manager (multi-project build system)
- Screening note: survivor; already used in this repository for two Kotlin packages (see R3).
  Long-lived daemon reached over a local socket, `gradle --status` lists daemons as IDLE or BUSY,
  file system watching keeps a virtual file system current between builds,
  and continuous build reruns tasks when declared inputs change.
  `gradle --status` shows daemon state, not per-task activity.
- Taxonomy terms: daemon, local socket, daemon status, continuous build, file system watching, virtual file system
- (VFS),
  incremental build, up-to-date check, multi-project builds

#### Mill

- URL: https://mill-build.org
- Discovered by: W3 (Sourcegraph list)
- Category guess: monorepo manager (JVM build tool)
- Screening note: survivor; only named in a list recommending it for JVM-heavy teams.
- Taxonomy terms: none observed

#### Rush

- URL: https://rushjs.io
- Discovered by: W3, W5, W7
- Category guess: monorepo manager
- Screening note: survivor; "build orchestrator" at the center of Rush Stack for large TypeScript monorepos.
- Taxonomy terms: build orchestrator

#### Earthly

- URL: https://earthly.dev
- Discovered by: W3 (Sourcegraph list)
- Category guess: monorepo manager (containerized build tool)
- Screening note: survivor pending maintenance check.
  Recall, not observed: Earthly's company announced it was ending Earthly development;
  verify project status before any further work.
- Taxonomy terms: none observed

#### Task (go-task)

- URL: https://taskfile.dev
- Discovered by: W1
- Category guess: monorepo manager (task runner)
- Screening note: survivor; a Python monorepo article uses `Taskfile.yml` includes to build packages in dependency order
  with outputs cached in `.task/`.
- Taxonomy terms: includes, checksum cache

#### pnpm workspaces and pnpm pipeline

- URL: https://pnpm.io
- Discovered by: W1, W3, W4
- Category guess: monorepo manager (package-manager workspace runner)
- Screening note: survivor already under assessment in
  `doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`; no watch or inspection feature observed.
- Taxonomy terms: recursive run, pipeline

#### Lerna

- URL: https://lerna.js.org
- Discovered by: W1, W3, W4, W7
- Category guess: monorepo manager
- Screening note: weak survivor; results state Lerna v6+ delegates task running and caching to Nx
  and mainly adds versioning and publishing.
- Taxonomy terms: versioning, publishing

#### Melos

- URL: https://melos.invertase.dev
- Discovered by: W1, W7
- Category guess: monorepo manager
- Screening note: weak survivor; Dart and Flutter workspace script execution and versioning, ecosystem mismatch.
- Taxonomy terms: none observed

### Component survivors

#### pitchfork

- URL: https://pitchfork.jdx.dev
- Discovered by: W2
- Category guess: component (daemon supervisor)
- Screening note: survivor; `pitchfork status [--json] <ID>` reports PID and running, stopped, or failed state.
  Same author as mise, which matters given the documentation-quality motivation in
  `doc/planning/mise-removal-coverage.md`.
  No file-watch rebuild behavior observed.
- Taxonomy terms: daemon supervisor, JSON status

#### systemd user units

- URL: https://systemd.io
- Discovered by: W2
- Category guess: component (process supervisor)
- Screening note: weak survivor; `systemctl status` and `systemctl show` expose unit state, logs, and restart policy.
  Linux-only.
  Recall, not observed: `.path` units can trigger services on file changes, and state is also reachable over D-Bus.
- Taxonomy terms: service supervisor, unit status

#### Aspect CLI

- URL: https://aspect.build
- Discovered by: W3
- Category guess: component (Bazel front end)
- Screening note: unverified survivor; results only show Aspect as a Bazel ecosystem company.
  Recall, not observed: Aspect CLI wraps Bazel and adds watch-style workflows.
- Taxonomy terms: none observed

### Screened out

- mise (https://mise.jdx.dev), W1: incumbent that the migration is leaving; monorepo tasks, task templates, task stubs,
- `mise watch`.
- npm workspaces and Yarn workspaces, W1: same package-manager workspace runner category as pnpm, which this repository
- uses.
- devenv (https://devenv.sh), W1: Nix development environment with services; environment manager, no inspection API
- observed.
- Hermit (https://cashapp.github.io/hermit), W1: tool installer, not a task runner.
- Develocity, W1: hosted build acceleration and analytics service.
- Unnamed remote execution platform and unnamed Nix environment manager, W1: tool.news snippets omitted names.
- `monorepo-watch` (https://github.com/utkarshk384/monorepo-watch), W1: beta watcher package, no inspection API
- observed.
- RepoOps, W1: hosted capture daemon.
- `concurrently`, W1: process launcher, no inspection API.
- Nx MCP server and Bazel MCP server, W1: features of Nx and Bazel, recorded under those entries.
- `git fsmonitor--daemon`, W2: file watch daemon with Simple IPC, serves only Git.
- `dmypy`, W2: mypy type-checker daemon with `dmypy status`; tool-specific.
- `start-stop-daemon`, W2: exit-code status only.
- moadim, W2: HTTP-API daemon for scheduled agent routines; category mismatch.
- MCPProxy, W2: MCP proxy daemon; category mismatch.
- Bubbaloop, W2: robotics agent runtime; category mismatch.
- Aerospike daemon, W2: database; category mismatch.
- Buck (v1), W3: superseded by Buck2.
- BuildBuddy and EngFlow, W3: remote execution, caching, and build-event services.
- Gazelle, W3 and W7: Bazel BUILD file generator.
- Sapling, W3: version control.
- Sourcegraph, W3: code intelligence.
- Aviator, W3 and W7: merge queue service.
- Endor Labs, W3: dependency security scanning.
- Railway, W3: deployment platform.
- Nx Cloud, W4: hosted CI layer.
- Angular, Svelte, Storybook, W5: frameworks and UI workshop.
- DevZero, Buildkite, W5: cloud development environment and CI service.
- Bun, W5: runtime and package manager.
- Vercel, W5: hosting platform.
- Pika Pack, esbuild, Webpacker, Broccoli, Layer-pack, W5 and W7: bundlers, packagers, and build plugins.
- Apache Druid, Watchtower, Nowledge Mem, EntryScape, Apify, RunningHub, Anchor Browser, Browser Use, Control Plane,
- W6: unrelated service APIs.
- Taskcluster, Semaphore CI, W6 and W7: CI services.
- Kiro, JetBrains Rider, W6 and W8: IDEs.
- `doowb/composer`, `bvaughn/task-runner`, watchdog, W6 and W7: libraries.
- Node.js test runner `--watch`, W6: test runner.
- `@hey-api/openapi-ts`, Buf, GraphQL Code Generator, OpenAPI Generator, openapi-typescript, W7: code generators.
- pre-commit, W7: Git hook framework.
- `sync-monorepo-packages`, W7: `package.json` field synchronizer; a file-enforcer peer, not a task runner.
- Spago, Symplify MonorepoBuilder, Tainted, Versio, tomono, `shopsys/monorepo-tools`, CODEOWNERS generator, Write
- Guard, W7:
  language-specific package, split, affected-detection, versioning, migration, or ownership tools.
- Builder (FormidableLabs), W7: shares scripts across Node projects; not an orchestrator.
- MSBuild, W8: .NET project build engine with Inputs and Outputs incremental checks; no watch daemon observed.
- IncrediBuild, W8: commercial distributed build accelerator with a Build Monitor.
- CocoIndex, W8: incremental data-indexing framework with a live watcher (`cocoindex update -L`); category mismatch.
- GNU direvent, `inotify-hookable`, `inotifywait`, W8: file watch command runners, no inspection API observed.

### Follow-up hints (recall, not observed in any result)

These may matter for the watch plus inspection requirement and need source verification:

- Turborepo: `turbo daemon status` and an experimental `turbo query` GraphQL interface.
- Nx: an Nx daemon and `nx watch`.
- Buck2: `buck2 status` and `buck2 log whatup` for in-flight actions.
- Pants: `pantsd` daemon and `--loop` rerun mode.
- Bazel: `ibazel` (bazel-watcher) and the Build Event Service stream.
- Gradle: Tooling API for programmatic build observation.
- Mill and Task: `--watch` flags.
- Watchman: file watch daemon with a JSON socket API; not surfaced by any query.
- process-compose: process orchestrator with a REST API; not surfaced by any query.

## Repository findings

### R1: candidate ledger in the MPM and pnpm vet report

Source: `doc/audit/tech-mpm-and-pnpm-as-a-mise-replacement-vet-2026-09-12.md`, "Candidate ledger" and "Query outcomes".

- Meta Package Manager 7.6.1 plus pnpm 12.4.1:
  serious alternative with unresolved composition;
  native providers and repository-owned adapters must be named before hard-gate confirmation,
  and stable MPM cannot reproduce several current pins through default restore adapters.
- pnpm 12.4.1 plus repository-owned orchestration and focused native providers:
  serious migration architecture, not a validated candidate stack;
  non-pnpm toolchain, environment, and secret providers unselected;
  watcher process-lifecycle parity under pnpm unverified; no adoption recommendation.
- Mise 2026.9.5 plus pnpm 12.3.4:
  excluded from the requested endpoint; rollback baseline only.
- metapac:
  not promoted; declarative wrapper over package managers that does not answer task, environment, or secret boundaries.
- pacdef:
  hard-gate exit; repository archived 2025-08-05, points to metapac.
- Topgrade:
  category mismatch; upgrades existing installations, not a declarative provisioner or task runner.
- Named in "Query outcomes" but without a ledger entry: `repo-run` and "unrelated task runners" (not individually
- named).
  The report states its search was not a saturated selection of every task runner.

### R2: candidates and fallbacks in the keep-mise audit and the removal coverage plan

`doc/audit/mise-keep-vs-build-own.md` (2026-06-02):

- mise: kept; no alternative monorepo manager or task runner is named.
- Build our own toolchain and monorepo manager: rejected as disproportionate.
- rustup: named as the provider for a boundary reconcile task, not a replacement candidate.
- No other tool is named as a candidate or fallback.

`doc/planning/mise-removal-coverage.md`:

- Hard requirement fallback order: existing tool on the market, then plug file-enforcer and watch plus RPC into an
- existing tool,
  then build a Bazel replacement.
- Bazel: assessed as a broader alternative (`doc/research/bazel-migration-dx.md`).
- `@monochromatic-dev/dev-script-file-enforcer`: canonical TypeScript configuration compiler and toolchain-convergence
- orchestrator.
- pnpm 12.4+ (`pnpm pipeline`, runtimes, Cargo and Python installs, shims): primary surface candidate for tools and
- tasks.
- Meta Package Manager 7.6.1: serious orchestration candidate for machine packages, not selected.
- Nadle: named fallback, together with "a focused repository task runner", if pnpm cannot preserve task semantics.
- Thin Mise adapters: allowed during migration.
- Watching: "direct watcher processes or a focused task runner", unselected.
- Focused native providers: rustup, Android `sdkmanager`, a verified standalone-binary installer, file-enforcer OS
- package dispatch.
- Secrets: SOPS and age retained as primitives; child-environment launcher unselected.
- CI: `jdx/mise-action` current owner; replacement blocked on selections.

### R3: tool-name search

Command run as specified; it matched 99 files.
Zero matches for `moonrepo`, `rushjs`, `pantsbuild`, `please.build`, and `process-compose`.
No `turbo.json`, `nx.json`, `moon.yml`, `rush.json`, Bazel, Pants, Taskfile, `process-compose`, Tiltfile, Buck, or Mill
configuration exists;
only `package/linter/kotlin/settings.gradle.kts` and `package/music-player/android-app/settings.gradle.kts` matched a
config-file search.

- Gradle (67 files): actually used, as a per-package build tool invoked from Mise tasks, not as a monorepo manager.
  Used in `package/linter/kotlin/` (`build.gradle.kts`, `gradlew`, wrapper properties, `mise.toml` publication tasks)
  and `package/music-player/android-app/` (`build.gradle.kts`, `gradlew`, `gradle.properties`, `mise.toml` that shells
  to the committed wrapper).
  Root `mise.toml` and `mise.no-env.toml` only mention Gradle in JDK and Android SDK comments.
  The remaining matches are documentation: Kotlin and Android decision and vet reports under `doc/decision/`,
  troubleshooting docs such as `doc/troubleshooting/intellij-gradle-kotlin-dsl-symlink.md` and
  `doc/troubleshooting/kotlin-2-2-10-parallel-gradle-cache.md`,
  `doc/research/bazel-migration-dx.md` (no Gradle interop documented for Bazel), and `README.md`.
  One false positive: `package/cli/forbidden-strings/data/betterleaks-default-config.toml`.
- Turborepo or `turbo` (13 files): mentioned only.
  Real mentions: `doc/audit/file-enforcer.md` (build-graph reconciler tier; registry-present but never surfaced),
  `doc/audit/tech-monorepo-manager-vet-2026-09-16.md` (query schedule), `doc/decision/workspace-ts-source-imports.md`
  and
  `doc/planning/issue-486-workspace-ts-source-imports.md` (just-in-time internal packages precedent),
  `doc/research/typescript-monorepo-cross-package-imports.md` (docs pinned at a commit),
  `doc/troubleshooting/oxlint.md`.
  False positives: `gpt-3.5-turbo` and a `turbo` model-speed signal in `package/pi-shared/model-selection/` and
  `package/pi-plugin/auto-mode/README.md`,
  `libjpeg-turbo` in `package/dev-script/file-enforcer/src/data/packages.generated.ts`, and the betterleaks config.
- Nx (22 files): mentioned only.
  Real mentions: `doc/audit/file-enforcer.md`, `doc/audit/tech-monorepo-manager-vet-2026-09-16.md`,
  `doc/decision/workspace-ts-source-imports.md`, `doc/planning/issue-486-workspace-ts-source-imports.md`,
  `doc/research/typescript-monorepo-cross-package-imports.md` (27 matches), `doc/troubleshooting/oxlint.md`.
  False positives: `\nx` escapes in unit tests, `nx` normal-vector variables (`deps-cube`, `typeface/aquaticat`,
  `doodle-widget`),
  the `nx-libs` OS package entry, and base64 blobs in HTML files.
- wireit (2 files): mentioned only, in `doc/audit/file-enforcer.md` (build-graph reconciler tier) and the monorepo
- manager vet report.
- Nadle (2 files): mentioned only, as the fallback in `doc/planning/mise-removal-coverage.md` and in the vet report.
- lage (1 file) and Buck2 (1 file): mentioned only, in `doc/audit/tech-monorepo-manager-vet-2026-09-16.md`.
- Tilt (6 files): one real mention in the vet report;
  the rest are false positives (camera tilt in `deps-cube`, "platform-tilt" in
  `doc/decision/ios-iphone-x-vet-report/vet-onsen.md`,
  glyph tilt in `package/webapp-productivity/wc/src/favicon.ts`).
- Outside the regex: `doc/audit/file-enforcer.md` also names `moon`, `make`, `ninja`, `redo`, and `tup`
  as registry-absent or build-graph peers of file-enforcer.

### R4: `mise watch` use

178 `mise.toml` files exist under `package/`.

- Active packages invoking `mise watch` (6 files):
  `package/kwin/key-helper/mise.toml`, `package/desktop-daemon/hall-monitor/mise.toml`,
  `package/ssg/aquati.cat/mise.toml`,
  `package/webapp-productivity/done/mise.toml`, `package/webapp-productivity/done-postcss/mise.toml`,
  `package/webapp-productivity/rss/mise.toml`.
- Paused packages invoking `mise watch` (5 files):
  `package-paused/webapp-content/messages-demo/mise.toml`, `package-paused/webapp-search/exa-search/mise.toml`,
  `package-paused/dev-script/inference-canary-viewer/mise.toml`, `package-paused/desktop-daemon/editord/mise.toml`,
  `package-paused/webapp-forge/server/mise.toml`.
- Root `mise.toml` and `mise.no-env.toml`: only the comment `# Required by mise watch` above `watchexec = "latest"`.
- Other watch paths in root `mise.toml`: `watch:*` task templates fan out unbounded to package tasks,
  `rolldown --watch`, `tsc --watch`, `node --watch` per test file, and `watch:sync:files` running file-enforcer
  `--watch`.
- Parallel system: `package/dev-script/watch-restart/` replaces `watchexec` in editord's dev loop
  after documented SIGINT-hang and signal-propagation failures through `watchexec`, `mise`, `sh`, `node`.
- None of these paths exposes an inspection RPC or IPC channel in the lines read.

## New taxonomy terms

- daemon
- daemon status
- IPC socket
- named pipe
- local socket
- Simple IPC
- watch mode
- persistent tasks
- continuous build
- file system watching
- virtual file system (VFS)
- incremental build
- up-to-date check
- devtools (live graph view)
- package graph
- task graph
- project graph
- dry run JSON
- run summary
- OpenTelemetry run metrics (OTLP)
- Build Event Protocol (BEP)
- BEP to OpenTelemetry traces
- MCP server (build tool)
- agent skills
- sync generators (`nx sync`)
- generators
- inferred tasks
- BUILD file generation
- dependency inference
- affected
- task sandboxing
- hermetic builds
- remote execution
- remote cache
- distributed task execution
- build orchestrator
- multi-project builds
- daemon supervisor
- JSON status and exit-code contract
- HTTP API mode
- readiness success pattern (long-running task detection)
- build monitor
- live update mode (watcher that reconciles target state)
- checker daemon
- non-hermetic task runner
- task templates
- task stubs
- codegen
- code intelligence layer
