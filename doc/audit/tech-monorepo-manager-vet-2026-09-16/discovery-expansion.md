# Monorepo manager discovery: frozen expansion schedule and screening

Executed 2026-09-16.
Read-only: nothing under `/var/home/user/Monochromatic` was modified; no candidate tool was installed or executed; no
browser was used.
Working files live under `scratchpad/exp/` (`gh/`, `npm/`, `crates/` raw pages plus `.txt` renderings, `readme/`,
`npm-readme/`, `pages/` curl captures, `notes.md` page-by-page screening notes).

## Method

- De-duplication set (`exp/known.json`, built by `exp/known.ts`):
  every GitHub repository, npm package, crate, URL, and entry name found in `screening-primary.json`,
  `screening-runners.json`, `screening-registry-primary.json`, `screening-registry-runners.json`,
  and the candidate sections of `discovery-github.md` (lines 169 to 1940, including "Excluded but close"),
  `discovery-registry.md` (181 to 624), `discovery-web-repo.md` (134 to 373), `discovery-web-paged.md` (175 to 219).
  After E12 page 2 the extractor was fixed to also read `- owner/repo (stars; qN pM): note` lines (GitHub repositories
  known: 644 to 986) and every cached page was re-rendered; all counts below use the rebuilt set.
  Names mentioned only in repository-findings or taxonomy sections (for example Wireit, Tilt, Mage) were not treated as
  known candidates.
- "New candidate": a result not in the de-duplication set and not returned by an earlier page of this schedule
- (schedule order E1 to E18).
  Rows recognised as known only by manual check are noted per page (for example `@visulima/task-runner`, `vite-plus`).
- "Screening survivor" (discovery): plausible G1 category fit with no obvious disqualifier.
  Close exclusions applied: affected-detection-only tools that run nothing, add-ons or plugins of another build tool,
  remote cache or execution servers, language-locked tools that run only one fixed kind of work (compiler check, hot
  reload),
  watchers or process supervisors lacking either file watching or an RPC, IPC, or HTTP interface, app-specific daemons
  (git status backends, code-intelligence indexes, agent orchestrators), repositories without source or returning HTTP
  404.
  Tiny, new, or unlicensed-looking repositories were kept when category fit was plausible; license and docs were left
  to G2 and G3.
- Screening input: name, description, topics; README heads (GitHub API raw README or npm registry metadata) for every
- ambiguous row.
- npm terminal views hid rows from vendor families with no tool-shaped member (MetaMask, 0x, Statsig, EthereumJS,
- InversifyJS, Kerebron, Salesforce templates, Wingify, Nomic Foundation, date-io, Tevm, Node-RED, Backstage,
- WordPress, Transcend MCP, Decap/Netlify CMS widgets, PouchDB, Webiny) and rows already SEEN; counts use every row in
- the saved page.
- Stop rule: stop after two consecutive complete pages with no new screening survivor, or when results are exhausted;
- GitHub's 1000-result cap was never reached before saturation.
- G3 no-JS fetches:
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0' <url>` via `exp/curl-page.ts`, which strips `<script>`/`<style>` before measuring visible text and looking for
- block markers.

## Query ledger

### E1

- Literal query: `"build server protocol" in:name,description`
- Provider: GitHub `search/repositories`; sort `stars`, order `desc`, `per_page=100`
- `total_count`: 29
- Per-page result count: p1 29
- Per-page new candidates: p1 26 (known: `build-server-protocol/build-server-protocol`, `616b2f/bsp.nvim`,
- `JetBrains/intellij-bsp`)
- Per-page new screening survivors: p1 0
- Stop reason: exhausted after page 1
- Screening: every result is a BSP spec, test kit, client, or server adapter binding an IDE to one build tool (Bazel,
- Gradle, Xcode, MSBuild, dotnet, rebar3, Maven).

### E2

- Literal query: `"file watcher" daemon in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 19
- Per-page result count: p1 19
- Per-page new candidates: p1 18 (known: `stark9000/arduino-fastbuild`)
- Per-page new screening survivors: p1 2 (`louisgv/nsfw.d`, `panord/fwatchd`)
- Stop reason: exhausted after page 1
- Close exclusions: `michaelglass/FsHotWatch` (F#-only compiler-warming daemon), `definev/fmon` (Dart VM Service
- client, no own interface), `eltaline/ecrond`, `adetunjii/fw-daemon`, `Turgut-Kalyon/wned`,
- `AndreyBarmaley/inotify-watcher` (watch-and-run daemons with no interface in README).

### E3

- Literal query: `"task runner" daemon in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 8
- Per-page result count: p1 8
- Per-page new candidates: p1 8
- Per-page new screening survivors: p1 0
- Stop reason: exhausted after page 1
- Close exclusions: `kuznetsss/tasksd` (JSON-RPC process-spawning daemon, no file watching), `ta3pks/taskd` (HTTP task
- daemon, README 404, no watching).

### E4

- Literal query: `"json-rpc" task in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 43
- Per-page result count: p1 43
- Per-page new candidates: p1 42 (1 seen on E3 p1: `kuznetsss/tasksd`)
- Per-page new screening survivors: p1 0
- Stop reason: exhausted after page 1
- Screening: A2A and MCP agent servers, Kanboard, Odoo, EvaTeam, TrueNAS API clients, JSON-RPC demos, `dvazar/planq`
- (queue library).

### E5

- Literal query: `"process manager" api in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 28
- Per-page result count: p1 28
- Per-page new candidates: p1 28
- Per-page new screening survivors: p1 1 (`saintedlama/invincible`)
- Stop reason: exhausted after page 1
- Close exclusions (process supervisors with HTTP APIs but no file watching in README): `retrixe/octyne`,
- `gar-id/queued`, `charliek/prox`, `EnviralDesign/simple-rust-process-manager`.

### E6

- Literal query: `"continuous build" in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 182
- Per-page result count: p1 100, p2 82
- Per-page new candidates: p1 100, p2 82
- Per-page new screening survivors: p1 0, p2 0
- Stop reason: exhausted after page 2
- Close exclusions: `fornellas/rrb` (re-runs a build on change, kills stale builds, no interface), `NixOS/hydra` and
- `golang/build` (CI servers), `anujb/godev` and `dcbishop/gowatch` (Go-only), `sessions-io/build-box` (CI deploy
- daemon).

### E7

- Literal query: `"remote execution" build in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 20
- Per-page result count: p1 20
- Per-page new candidates: p1 17 (known: `TraceMachina/nativelink`, `nnunley/objfs`, `chaitanya-archive/obelisk.build`)
- Per-page new screening survivors: p1 0
- Stop reason: exhausted after page 1
- Close exclusions: `buildbarn/bb-remote-execution`, `colinrgodsey/goREgo` (remote execution servers),
- `gameswhit/blaze-build-system` (repository root holds only `README.md`, `index.html`, `vercel.json`).

### E8

- Literal query: `"project graph" in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 300
- Per-page result count: p1 100, p2 100
- Per-page new candidates: p1 100, p2 100
- Per-page new screening survivors: p1 0, p2 0
- Stop reason: saturated: pages 1 and 2 both added no new screening survivor (page 3 not fetched)
- Close exclusions: `nx-dotnet/nx-dotnet`, `Wtiben/moon-dotnet-plugin`, `eBay/graph-analytics-plugin` (plugins),
- `HaasStefan/nxdb`, `Commonjava/atlas`, `TheEskhaton/ark`, `kaanbiryol/tuist-to-bazel`.

### E9

- Literal query: `affected monorepo in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 40
- Per-page result count: p1 40
- Per-page new candidates: p1 37 (known: `lemonade-hq/traf`, `leonardochaia/dotnet-affected-action`,
- `api-evangelist/nx`)
- Per-page new screening survivors: p1 4 (`Rani367/affected`, `Toyz/gw`, `rkishan516/fx`, `zinuo-xu/mono-cli`)
- Stop reason: exhausted after page 1
- Close exclusions (affected detection only): `frontops-dev/domino`, `gophersatwork/colony`, `split/lockfile-affected`,
- `DeveloperC286/is_affected`, `AlekseyLeshko/affected-workspaces`, `withgraphite/graphite-transitive-dependencies`;
- plus `katie0109/Arch-Lens` (architecture rule engine), `yushman/lightning` (Gradle telemetry and cache).

### E10

- Literal query: `"persistent tasks" OR "long-running tasks" in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 1033
- Per-page result count: p1 100, p2 100
- Per-page new candidates: p1 99 (known: `turtlemonvh/blanket`), p2 100
- Per-page new screening survivors: p1 0, p2 0
- Stop reason: saturated: pages 1 and 2 without new screening survivors (1000-result cap not reached)
- Screening: AI agent harnesses and skills, job queues, keep-awake utilities, todo apps; close: `nullbadger/bobr`,
- `kim-company/pmux`.

### E11

- Literal query: `"live update" watch in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 24
- Per-page result count: p1 24
- Per-page new candidates: p1 24
- Per-page new screening survivors: p1 0
- Stop reason: exhausted after page 1
- Close exclusions: `teamplanes/cdk-watch` (AWS CDK only), `alexpw/live-cfg` (config reload library).

### E12

- Literal query: `"sync generator" OR "code generation" monorepo in:description`
- Provider: GitHub; sort `stars`, `desc`, `per_page=100`
- `total_count`: 41820 (GitHub matched descriptions containing only "monorepo", for example `vbenjs/vue-vben-admin`)
- Per-page result count: p1 100, p2 100, p3 100, p4 100
- Per-page new candidates: p1 93, p2 95, p3 96, p4 96
- Per-page new screening survivors: p1 0, p2 1 (`ds300/lazyrepo`), p3 0, p4 0
- Stop reason: saturated: pages 3 and 4 without new screening survivors (1000-result cap not reached)
- Known rows: `nrwl/nx`, `microsoft/rushstack`, `moonrepo/moon`, `folke/ultra-runner`, `guigrpa/oao`, `microsoft/lage`,
- `korfuri/awesome-monorepo`, `lerna-lite/lerna-lite`, `vltpkg/vltpkg`, `symplify/monorepo-builder`,
- `gitmono-dev/mega`, `nrwl/monorepo.tools`, `loadingalias/cargo-rail`, `mbtproject/mbt`, `Akryum/monorepo-run`,
- `azu/monorepo-utils`, `seansfkelley/yerna`, `giltayar/bilt`, `paularmstrong/onerepo`
- Close exclusions: `Thinkmill/manypkg`, `preconstruct/preconstruct`, `deref/uni` (Unirepo bundling tool, alpha 2021),
- `amperity/lein-monolith` (Leiningen plugin), `DavidVujic/poetry-multiproject-plugin`,
- `enspirit/makefile-for-monorepos`, `folke/vscode-monorepo-workspace`.

### E13

- Literal query: `task runner daemon`
- Provider: npm `registry.npmjs.org/-/v1/search`, `size=250`, `popularity=1.0`, `quality=0.0`, `maintenance=0.0`
- `total`: 100728
- Per-page result count: p0 250, p1 250
- Per-page new candidates: p0 242, p1 243
- Per-page new screening survivors: p0 0, p1 0
- Stop reason: saturated: pages 0 and 1 without new screening survivors
- Known rows include `@go-task/cli`, `@visulima/vis` and `@visulima/task-runner`, `@voidzero-dev/vite-task-client`,
- `@theholocron/astromech`, `runner-run`, `ultra-runner`, `concurrent-tasks`, `@rushstack/rush-daemon`, `composer`,
- `taskr`
- Close exclusions: `turbo-daemon` (Turborepo remote-cache server wrapper), `@hasna/loops` (scheduled loops daemon, no
- file watching), `@brutalsystems/forge` (dev-process daemon with HTTP CLI; no file watching; no repository in npm
- metadata), `hash-runner`, `@bazel/ibazel`, `sock-daemon`, `task-graph-runner`.

### E14

- Literal query: `monorepo watch daemon`
- Provider: npm search, same parameters as E13
- `total`: 103023
- Per-page result count: p0 to p8 250 each
- Per-page new candidates: p0 187, p1 184, p2 227, p3 222, p4 230, p5 238, p6 235, p7 239, p8 240
- Per-page new screening survivors: p0 1 (`jazelle`), p1 0, p2 1 (`@livetest/cli`), p3 1 (`monex`), p4 2 (`wireit`,
- `@fluidframework/build-tools`), p5 0, p6 1 (`@factorialco/shadowdog`), p7 0, p8 0
- Stop reason: saturated: pages 7 and 8 without new screening survivors
- Known rows include `fb-watchman`, `bun-workspaces`, `pacwich`, `lerna-watch`, `fynpo`, `oao`, `@vonage/monowatch`,
- `monorepo-run`, `@aklinker1/buildc`, `@lerna-lite/watch`, `run-shared-scripts`, `@enspirit/emb`, `unorepo`,
- `@icebreakers/monorepo`, `@aip-tech/braid`, `workspace-tools`, `vite-plus` (Vite+ `vp run`, known as
- `voidzero-dev/vite-task`)
- Close exclusions: `turbowatch` (watch and task orchestration, no RPC, IPC, or HTTP interface), `standard-monorepo`
- (run and watch commands are unchecked roadmap items), `diffstalkerd` (git status backend), `@toptal/davinci-monorepo`
- (repository returns HTTP 404), `@travetto/repo` (framework module), `@mpis/monorepo` (README does not state
- dependency ordering), `light-server`, `dev-manager-mcp` (no file watching), `relife2` (no repository), `wsrun-ng`
- (fork of known wsrun), `@smoothbricks/cli`, `aitomator`.

### E15

- Literal query: `build orchestrator`
- Provider: npm search, same parameters as E13
- `total`: 608154
- Per-page result count: p0 250, p1 250
- Per-page new candidates: p0 236, p1 241
- Per-page new screening survivors: p0 0, p1 0
- Stop reason: saturated: pages 0 and 1 without new screening survivors
- Known rows: `workgraph`, `jake`, `@rushstack/heft`, `@microsoft/rush`, `@rushstack/rush-bridge-cache-plugin`,
- `@theholocron/cli`
- Close exclusions: `scripts-orchestrator` (command-level dependencies; workspace support only aggregates result JSON),
- `orchestrator` and `tinytick` (libraries), `gulp`, `broccoli`.

### E16

- Literal query: `build orchestrator`
- Provider: crates.io `api/v1/crates`, `sort=downloads`, `per_page=100`, user agent `monochromatic-vet-discovery`
- `total`: 4410
- Per-page result count: p1 100, p2 100
- Per-page new candidates: p1 100, p2 100
- Per-page new screening survivors: p1 0, p2 0
- Stop reason: saturated: pages 1 and 2 without new screening survivors
- Screening: libraries and frameworks (Burn, Vortex, cxx, orchestra, surrealdb), workflow engines (duroxide, obelisk,
- dagrs); close: `cargo-leptos` (Leptos-only).

### E17

- Literal query: `file watcher daemon`
- Provider: crates.io, same parameters as E16
- `total`: 337
- Per-page result count: p1 100, p2 100, p3 100
- Per-page new candidates: p1 100, p2 100, p3 99 (known: `harness-canopy`)
- Per-page new screening survivors: p1 1 (`don`), p2 0, p3 0
- Stop reason: saturated: pages 2 and 3 without new screening survivors (page 4 not fetched)
- Close exclusions: `eis` and `watchers` (auto-commit daemons, no RPC), `gity-daemon` (Git acceleration), `lode-daemon`
- (repository `lode-rs/lode` returns HTTP 404), `ambient-fsd` (repository `kollabor/ambient-fs` returns HTTP 404),
- `agproc` ("no central daemon, no socket"), `task-trigger-mcp` (deprecated predecessor of known `harness-canopy`).

### E18

- Provider: `gh api repos/<owner>/<repo>` exact lookups plus README reads
- Buck2, https://github.com/facebook/buck2: duplicate (known: `discovery-registry.md`, `discovery-web-repo.md`)
- Wireit, https://github.com/google/wireit: duplicate of this schedule (first found on E14 p4 as npm `wireit`)
- Watchman, https://github.com/facebook/watchman: duplicate (known: `screening-registry-primary.json`)
- just, https://github.com/casey/just: duplicate (known: `screening-registry-runners.json`)
- Mage, https://github.com/magefile/mage: new; not a survivor (Make-like Go build tool; no project model, no daemon)
- batect, https://github.com/batect/batect: new; not a survivor (archived; README "Batect is no longer maintained";
- container task runner without project model)
- Dagger, https://github.com/dagger/dagger: new; not a survivor (README describes a containerized pipeline engine with
- a system API and input-keyed caching; it mentions no workspace project model and no file watching)
- watchexec, https://github.com/watchexec/watchexec: new; not a survivor (watch-and-run with no inspection or control
- interface)
- Tilt, https://github.com/tilt-dev/tilt: new; survivor (component)
- process-compose, https://github.com/F1bonacc1/process-compose: new; survivor (component; README lists REST API and
- File Watching)
- pueue, https://github.com/Nukesor/pueue: new; not a survivor (command queue daemon with IPC, no file watching)
- pitchfork, https://github.com/jdx/pitchfork: duplicate (known: `discovery-web-repo.md`, `screening-primary.json`)
- Nadle, https://github.com/nadlejs/nadle: duplicate (known: `screening-runners.json`)
- Earthly, https://github.com/earthly/earthly: duplicate (known: `screening-runners.json`)
- Aspect CLI, https://github.com/aspect-build/aspect-cli: duplicate (known: `discovery-web-repo.md`,
- `screening-primary.json`)
- Totals: 8 duplicates (7 known, 1 schedule duplicate), 7 new, 2 new survivors

### Saturation summary

- Exhausted: E1, E2, E3, E4, E5, E6, E7, E9, E11
- Saturated by two consecutive empty pages: E8 (p1, p2), E10 (p1, p2), E12 (p3, p4), E13 (p0, p1), E14 (p7, p8), E15
- (p0, p1), E16 (p1, p2), E17 (p2, p3)
- Blocked by cap: none
- New screening survivors: 17 (E2 2, E5 1, E9 4, E12 1, E14 6, E17 1, E18 2)

## New candidates

Only screening survivors and the closest exclusions are listed; every other new result is in the saved pages under
`exp/`.

### Screening survivors

- nsfw.d
  - https://github.com/louisgv/nsfw.d
  - discovered by: E2 p1; category guess: component (file watcher daemon broadcasting over WebSocket)
- fwatchd
  - https://github.com/panord/fwatchd
  - discovered by: E2 p1; category guess: component (inotify daemon controlled by `fwatchctl` over a socket)
- Invincible
  - https://github.com/saintedlama/invincible
  - discovered by: E5 p1; category guess: component (dev process manager with file watching and HTTP API)
- affected
  - https://github.com/Rani367/affected
  - discovered by: E9 p1; category guess: monorepo manager (affected selection plus `affected run`)
- gw
  - https://github.com/Toyz/gw
  - discovered by: E9 p1; category guess: monorepo manager (Go multi-module workspaces)
- fx
  - https://github.com/rkishan516/fx
  - discovered by: E9 p1; category guess: monorepo manager (Dart and Flutter)
- mono-cli
  - https://github.com/zinuo-xu/mono-cli
  - discovered by: E9 p1; category guess: monorepo manager (npm workspaces)
- lazyrepo
  - https://github.com/ds300/lazyrepo
  - discovered by: E12 p2; category guess: monorepo manager
- Jazelle
  - https://github.com/uber-web/jazelle (npm `jazelle`)
  - discovered by: E14 p0; category guess: monorepo manager (JS monorepos over Bazel)
- Live Test Runner (`@livetest/cli`)
  - https://github.com/bush1D3v/livetest
  - discovered by: E14 p2; category guess: component (watch daemon with TCP event channel)
- Monex
  - https://github.com/fabiospampinato/monex
  - discovered by: E14 p3; category guess: component (watch-and-restart daemon with RPC commands)
- Wireit
  - https://github.com/google/wireit
  - discovered by: E14 p4, E18; category guess: monorepo manager (borderline: declared cross-package script
  - dependencies)
- fluid-build (`@fluidframework/build-tools`)
  - https://github.com/microsoft/FluidFramework/tree/main/build-tools/packages/build-tools
  - discovered by: E14 p4; category guess: monorepo manager
- Shadowdog (`@factorialco/shadowdog`)
  - https://github.com/factorialco/shadowdog
  - discovered by: E14 p6; category guess: component (artifact-generation watcher with socket and MCP plugins)
- don
  - https://github.com/pjtatlow/don
  - discovered by: E17 p1; category guess: component (dev-stack orchestrator with watch and unix-socket HTTP API)
- Tilt
  - https://github.com/tilt-dev/tilt
  - discovered by: E18; category guess: component (file watching plus Tilt API server)
- process-compose
  - https://github.com/F1bonacc1/process-compose
  - discovered by: E18; category guess: component (process orchestrator with file watching and REST API)

### Closest exclusions (new, not screened)

- `kuznetsss/tasksd` https://github.com/kuznetsss/tasksd (E3, E4): JSON-RPC process daemon without file watching
- `retrixe/octyne` https://github.com/retrixe/octyne (E5): process manager with HTTP API, no file watching
- `charliek/prox` https://github.com/charliek/prox (E5): API-first dev process manager, no file watching in README
- `fornellas/rrb` https://github.com/fornellas/rrb (E6): watch-and-rebuild without interface
- `gophersatwork/colony` https://github.com/gophersatwork/colony (E9): Go affected detection only
- `frontops-dev/domino` https://github.com/frontops-dev/domino (E9): semantic affected detection only
- `Thinkmill/manypkg` https://github.com/Thinkmill/manypkg (E12): workspace checks and per-package run, no
- relationship-driven selection
- `turbowatch` https://www.npmjs.com/package/turbowatch (E14 p3): watch and task orchestration, no RPC, IPC, or HTTP
- interface
- `standard-monorepo` https://www.npmjs.com/package/standard-monorepo (E14 p1): run and watch commands unimplemented
- `dev-manager-mcp` https://github.com/BloopAI/dev-manager-mcp (E14 p8): MCP dev-server daemon without file watching
- `scripts-orchestrator` https://www.npmjs.com/package/scripts-orchestrator (E15 p0): command dependencies, not project
- relationships
- `agproc` https://github.com/jmjoy/agproc (E17 p3): per-service runners without daemon or socket
- `pueue` https://github.com/Nukesor/pueue (E18): IPC command queue without file watching
- `watchexec` https://github.com/watchexec/watchexec (E18): watch-and-run without interface

## Screening

Gates stop at the first failure.
G4 is recorded only for survivors; there are none.

### nsfw.d

- URL: https://github.com/louisgv/nsfw.d
- Archived: no; last push 2023-01-07; 1 star
- G1: pass, component (borderline: event broadcaster, no control API).
  README CLI help: "🚀 File watcher socket daemon." with `-wp  WebSocket port` and `-sp  Static file port`;
  `package.json` depends on `nsfw` (native watcher) and `uWebSockets.js`.
- G2: fail.
  Repository root holds
  `.editorconfig, .gitattributes, .gitignore, commands, core, package-lock.json, package.json, readme.md`: no license
  file.
  GitHub license detection returns none; only `package.json` declares `"license": "MIT"`.
- G3: not evaluated.
- Final: exit at G2 (no license file).

### fwatchd

- URL: https://github.com/panord/fwatchd
- License: MIT (`LICENSE` at root); archived: no; last push 2024-11-28
- G1: pass, component.
  README: "fwatchd can be controlled using fwatchctl, which may instruct fwatchd to track files and perform some action
  based on inotify events on that file"; source tree has `src/socket.rs`, `src/fwatchctl.rs`, `src/fwatchd.rs`.
- G2: pass.
- G3: fail, coverage (a), (b), (c).
  - Official docs are the 717-character README (https://github.com/panord/fwatchd#readme): sections "Create fwatchd
  - user", "Example Usage", "Customizing usage of the event".
  - (a) configuration reference: absent. (b) CLI reference: only three example invocations of `fwatchctl`. (c)
  - extension mechanism: absent.
  - Incidental confusion trigger: the README example
  - `fwatchctl track /tmp/example --alias /usr/bin/echo --script /usr/bin/cat"` ends with an unmatched quote.
- Final: exit at G3 coverage.

### Invincible

- URL: https://github.com/saintedlama/invincible
- License: MIT (`LICENSE`); archived: no; last push 2026-09-12; README: "Proof of concept. ... APIs, config format, and
- behaviour WILL change without notice."
- G1: pass, component.
  README: "optionally watch files for auto-rebuild ... Comes with a terminal UI for humans and an HTTP API for agents";
  section "HTTP API" lists `GET /processes`, `POST /processes/{name}/restart`, `GET /openapi.json`.
- G2: pass.
- G3: fail, coverage (c).
  - Official docs are the README (https://github.com/saintedlama/invincible#readme).
  - (a) present ("Configuration" annotated `.invincible.toml`); (b) present ("CLI commands", "Running"); (c) absent: no
  - heading or text mentions plugin, extension, extend, or hook.
  - (d) watch claimed: "File watching + auto-restart (opt-in)"; (e) and (f) claimed: "HTTP API".
- Final: exit at G3 coverage (no extension mechanism).

### affected

- URL: https://github.com/Rani367/affected
- License: MIT (`LICENSE`); archived: no; last push 2026-06-05
- G1: pass, monorepo manager.
  README: "detects which packages in your monorepo are affected by git changes, then runs tests, lints, builds, or any
  command on only those packages", demo `api (depends on: core)`, `affected run "cargo clippy -p {package}"`.
- G2: pass.
- G3: fail, coverage (c).
  - Docs site https://rani367.github.io/affected (curl: HTTP 200, 3595 visible characters, no block marker) is a
  - landing page whose only doc links go to the GitHub README.
  - README: (a) "Configuration" is one annotated `.affected.toml` example plus `--config` in "Global Flags"; (b)
  - "Usage" per command; (c) absent (no plugin or extension text).
  - (d) watch claimed: "`affected watch`" section.
- Final: exit at G3 coverage (no extension mechanism).

### gw

- URL: https://github.com/Toyz/gw
- License: MIT (`LICENSE`); archived: no; last push 2026-07-18
- G1: pass, monorepo manager (borderline).
  README commands: `gw run -- <cmd>` "Run a command in every module's directory"; `gw affected --since <ref>` "walk the
  DAG to every impacted module ... Feed selective CI"; `gw graph`.
  Relationships select what CI runs; `gw run`/`gw test` themselves run every module.
  This follows the earlier G1 precedent that promoted `omio-labs/myke` and `metaist/ds` (run in all projects, no
  dependency ordering); under a stricter reading gw exits at G1 instead.
- G2: pass.
- G3: fail, no-JS.
  https://toyz.github.io/gw/ (the repository homepage): HTTP 200, 783-byte HTML whose body is `<gw-site></gw-site>`
  plus `<script type="module" ... src="/gw/assets/index-B0eLrRk6.js">`; visible text is only "gw — Go workspaces at
  scale" (27 characters).
- Final: exit at G3 no-JS.

### fx

- URL: https://github.com/rkishan516/fx
- License: MIT (`LICENSE`); archived: no; last push 2026-06-10; 0 stars
- G1: pass, monorepo manager.
  README: "Run targets across projects with dependency-aware ordering" and affected analysis "Uses the dependency graph
  to include downstream dependents."
- G2: pass.
- G3: fail, confusion (broken documentation link).
  The README header's "Documentation" link is https://fx.dev; curl fails with "Could not resolve host: fx.dev", and
  DNS-over-HTTPS (`dns.google/resolve?name=fx.dev&type=A`) returns no answer, only the registrar SOA.
- Final: exit at G3 confusion.

### mono-cli

- URL: https://github.com/zinuo-xu/mono-cli
- License: MIT (`LICENSE`); archived: no; last push 2026-06-03; 0 stars
- G1: pass, monorepo manager.
  README: "Automatically resolves inter-package dependencies and builds in topological order", "Determines which
  packages are impacted by file changes", `mono build --affected`.
- G2: pass.
- G3: fail, confusion (install instructions install a different package), then coverage.
  - README "Installation" (https://github.com/zinuo-xu/mono-cli#installation) says `npm install -g mono-cli` and
  - `npx mono-cli <command>`.
  - `https://registry.npmjs.org/mono-cli` is an unrelated package: latest 0.2.0, maintainer `case`, no `bin`, no
  - repository, last modified 2022-06-20; the repository's `package.json` is `mono-cli` 1.0.0 with `bin.mono`.
  - Coverage: "Configuration" is two sentences; no extension mechanism.
- Final: exit at G3 confusion.

### lazyrepo

- URL: https://github.com/ds300/lazyrepo
- License: MIT (`LICENSE`); archived: no; last push 2026-03-21; README: "Currently in the prototyping stage ... Get
- help or join in development on discord"
- G1: pass, monorepo manager.
  README: "The tests for `core` will only be started if both `utils` and `primitives` finish successfully" and "If you
  change a source file in `utils` ... both `utils` and `core`'s tests will be executed, in that order."
- G2: pass.
- G3: fail, coverage (a) and (c).
  - Official docs are the README (https://github.com/ds300/lazyrepo#configuration).
  - (a) "Configuration" is a single `lazy.config.js` example with three `cache` keys; the full shape is only in
  - `index.d.ts`.
  - (b) present ("Other commands"). (c) absent.
- Final: exit at G3 coverage.

### Jazelle

- URL: https://github.com/uber-web/jazelle (npm `jazelle`, whose registry metadata has no repository field)
- License: MIT (`LICENSE`); archived: no; last push 2026-09-15
- G1: pass, monorepo manager.
  README `jazelle build`: "Builds a project and its dependencies in topological order."
- G2: pass.
- G3: fail, confusion (missing anchors).
  - README table of contents links "[Lockfile delegation](#lockfile-delegation)", but the section is titled "Top-level
  - file delegation"; curl of https://github.com/uber-web/jazelle (HTTP 200, no block marker) shows
  - `href="#lockfile-delegation"` and only `id="user-content-top-level-file-delegation"`.
  - `jazelle build` says "See also [direct Bazel usage](#direct-bazel-usage)", and no such heading exists (the section
  - is "Using Bazel").
- Final: exit at G3 confusion.

### Live Test Runner (`@livetest/cli`)

- URL: https://github.com/bush1D3v/livetest; docs site https://livetest-kappa.vercel.app/
- License: MIT (`LICENSE`); archived: no; last push 2026-09-08; docs in Portuguese
- G1: pass, component.
  CLI README: `livetest start` "observa o projeto, roda os testes afetados a cada save, escreve `run.log` e
  `status.json` e abre o canal de eventos"; https://github.com/bush1D3v/livetest/blob/master/docs/protocol.md: NDJSON
  over "socket TCP" on `127.0.0.1`, discovery file `.livetest/daemon.json`; any language via `adapter: "command"`.
- G2: pass.
- G3: fail, confusion (custom adapters need tests and an undocumented CLI path).
  - Docs site (curl: HTTP 200, 5435 visible characters) links to GitHub docs.
  - https://github.com/bush1D3v/livetest/blob/master/docs/adapters.md (curl: HTTP 200) ends "veja
  - `packages/core/test/graph/` para o padrão usado nos adapters embutidos" (see the tests for the pattern).
  - The same page registers custom adapters only through `createEngine({ graphAdapters, runnerAdapters })` in your own
  - program, and does not say how `livetest start` (the documented daemon) loads them.
- Final: exit at G3 confusion.

### Monex

- URL: https://github.com/fabiospampinato/monex
- License: MIT (`license` file; `package.json` has no license field); archived: no; last push 2026-07-16; npm 2.2.1
- (2023-09)
- G1: pass, component.
  README: "restart them whenever they crash or a watched file changes"; `monex-daemon start|stop|ping|log|stat`; source
  has `src/daemon/server.ts`, `src/daemon/client.ts`, dependency `picorpc`.
- G2: pass.
- G3: fail, confusion (wrong command name), also coverage (c).
  - README "Commands" (https://github.com/fabiospampinato/monex, curl HTTP 200): "Various sub-commands are provided by
  - the `context-daemon` command for managing the daemon", while every example uses `monex-daemon`.
  - Coverage: no extension or plugin mechanism in the README.
- Final: exit at G3 confusion.

### Wireit

- URL: https://github.com/google/wireit
- License: Apache-2.0 (`LICENSE`); archived: no; last push 2026-09-16
- G1: pass, monorepo manager (borderline: dependencies are declared per script, not discovered from workspaces).
  README "Cross-package dependencies": "Dependencies can refer to scripts in other npm packages by using a relative
  path ... work well for npm workspaces, as well as in other kinds of monorepos"; watch mode "re-runs only the affected
  scripts".
- G2: pass.
- G3: fail, confusion (reference table link), also coverage (c).
  - README "Reference > Configuration" row `service`:
  - "[Whether this script is long-running, e.g. a server](#cleaning-output)"; curl of https://github.com/google/wireit
  - shows `<a href="#cleaning-output">Whether this script is long-running, e.g. a server`, pointing to the cleaning
  - section instead of "Services".
  - Coverage: no extension mechanism; the only "extension" is the VS Code editor extension.
- Final: exit at G3 confusion.

### fluid-build (`@fluidframework/build-tools`)

- URL: https://github.com/microsoft/FluidFramework/tree/main/build-tools/packages/build-tools
- License: MIT (`LICENSE` in the package directory and repository root); archived: no
- G1: pass, monorepo manager.
  README: "`fluid-build` is a build task scheduler. It support declarative task and dependencies definition,
  incremental detection for a range of tools and multiple workspace (a.k.a. release group) in a repo."
- G2: pass.
- G3: fail, confusion (source file and repo config needed).
  - README "Incremental and Tasks" (curl HTTP 200 on the tree URL, text present): "See the object definition
  - `executableToLeafTask` in ./src/fluidBuild/tasks/taskFactory.ts for the full list of task."
  - README "Release Group definition": "See [fluidBuild.config.cjs](../../../fluidBuild.config.cjs) for how it looks
  - like."
  - README opening: "The content and example below will focus on the Fluid Framework repo."
- Final: exit at G3 confusion.

### Shadowdog (`@factorialco/shadowdog`)

- URL: https://github.com/factorialco/shadowdog
- License: MIT (`LICENSE`); archived: no; last push 2026-04-21
- G1: pass, component.
  README: watchers run commands when files change; `shadowdog-socket` "Provides an external communication channel";
  `shadowdog-mcp` "HTTP endpoint at `http://localhost:8473/mcp`" with tools `pause-shadowdog`, `resume-shadowdog`,
  `compute-artifact`, `get-shadowdog-status`.
- G2: pass.
- G3: fail, confusion (contradiction), also coverage (c).
  - README "CLI commands": "**Watch mode** (includes MCP server for external tool integration)", and "MCP Integration":
  - "The MCP server starts automatically when you run Shadowdog in watch mode", yet "Quick Setup" step 1 is "Add the
  - MCP plugin to your configuration" (curl of https://github.com/factorialco/shadowdog: HTTP 200, texts present).
  - `shadowdog-socket` names events but no transport, path, or port.
  - Coverage: "Plugin Support: Extend functionality with custom or community-built plugins" is claimed, but only
  - built-in plugin names and enabling are documented; no plugin authoring docs.
- Final: exit at G3 confusion.

### don

- URL: https://github.com/pjtatlow/don
- Archived: no; last push 2026-09-09; 2 stars
- G1: pass, component.
  README: services with `watch = ["src/**/*.rs", "Cargo.toml"]` and "Watch for file changes and rebuild/restart
  automatically"; "Daemon API": "Don exposes a unix socket API at `.don/don.sock`" with `GET /status`, `GET /events`,
  `POST /restart/:name`.
- G2: fail.
  Repository root:
  `.git-blame-ignore-revs, .github, .gitignore, AGENTS.md, CLAUDE.md, Cargo.lock, Cargo.toml, README.md, build.rs, dev,
  dist-workspace.toml, docs, don.toml, examples, skills, src, tests, tools, web`: no license file; GitHub license detect
  ion returns none; only `Cargo.toml` declares `license = "MIT"`.
- G3: not evaluated.
- Final: exit at G2 (no license file).

### Tilt

- URL: https://github.com/tilt-dev/tilt; docs https://docs.tilt.dev/, API https://api.tilt.dev
- License: Apache-2.0 (`LICENSE`); archived: no; last push 2026-09-14
- G1: pass, component.
  README: "Tilt automates all the steps from a code change to a new process: watching files, building container
  images"; https://docs.tilt.dev/cli/tilt_trigger.html: "Trigger an update for the specified resource" with `--port`
  "Port for the Tilt HTTP server ... (default 10350)"; https://api.tilt.dev: "The Tilt Server API allows you to drill
  down into the depths of Tilt".
- G2: pass.
- G3: fail, confusion (consumed watch behavior needs a bug report or source).
  - https://docs.tilt.dev/file_changes.html (curl: HTTP 200, 11219 visible characters, no block marker): "Tilt has a
  - hard-coded list of temp files in common text editors (Emacs, Vim, etc.). ... If you find that temp files in your
  - editor trigger builds, please file a bug and we will add it to the list." The list itself is not documented.
  - Coverage otherwise looked complete before the trigger: Tiltfile API reference (https://docs.tilt.dev/api.html), CLI
  - reference (`tilt up`, `tilt get`, `tilt trigger`), extensions (https://docs.tilt.dev/extensions.html), FileWatch
  - API object (https://api.tilt.dev/core/file-watch-v1alpha1.html). The API guide shows only CLI access, not raw HTTP
  - calls.
- Final: exit at G3 confusion.

### process-compose

- URL: https://github.com/F1bonacc1/process-compose; docs https://f1bonacc1.github.io/process-compose/
- License: Apache-2.0 (`LICENSE`); archived: no; last push 2026-09-07
- G1: pass, component.
  README features: "REST API (OpenAPI a.k.a Swagger) with optional token authentication", "File Watching - restart a
  process when its files change, optionally cascading to its dependents", "MCP Server integration".
- G2: pass.
- G3: fail, confusion (docs defer to an issue), also coverage (c).
  - https://f1bonacc1.github.io/process-compose/launcher/ (curl: HTTP 200, 15828 visible characters): "If parent
  - process (starter) won't close stdout and stderr within specified launch_timeout_seconds ... process compose will
  - stop waiting for its log completion and start waiting for process termination. (more details are here)", linking
  - https://github.com/F1bonacc1/process-compose/issues/258#issuecomment-2439544894.
  - The site navigation (Installation, Logging, Intro, Launcher, CLI pages, Blog, Contributing, Health, Configuration,
  - Interactive processes, Merge, Client, TUI, Graph, Scheduled processes, Watch, MCP server, Sponsors, blog archives)
  - has no plugin or extension page, and mixes blog archives and sponsors with reference pages.
- Final: exit at G3 confusion.

### @sohaha/zzz (carried over)

- URL: https://github.com/sohaha/zzz; homepage https://docs.73zls.com/zls-go/#/c1d6d23a-2921-42f8-a46b-53641297987d
- License: Apache-2.0 (`LICENSE`); archived: no; last push 2026-05-16
- G1: pass, component (borderline: browser live-reload channel, no control API).
  Repository description "Go程序热编译、压力测试等" (hot compile for Go programs, stress testing); the README documents only
  installation and `zzz help`.
  Source `app/watch/http.go`: `zzz watch` starts an HTTP server with a WebSocket (`melody`) that `Broadcast`s
  changed-file messages (`sendChang`) and serves or proxies pages with injected reload JS.
- G2: pass.
- G3: fail, no-JS.
  The homepage docs URL returns HTTP 200 with 729 bytes of HTML whose visible text is "进入中 加载中，请稍候..." ("entering,
  loading, please wait"), 14 characters; the README has no feature docs.
- Final: exit at G3 no-JS.

### rnme / runme (carried over)

- URL: https://github.com/dgrijalva/runme (crate `rnme`, binary `runme`)
- License: MIT (`LICENSE`); archived: no; last push 2026-06-22; README: "This is a hobby project ... the API is still
- changing rapidly"
- G1: pass, component.
  README: "Can also be used as an MCP for direct agent access via `runme --mcp`" and "`ctx.watch()` returns a debounced
  stream of changed paths"; https://github.com/dgrijalva/runme/blob/main/docs/mcp_design.md: the MCP supervisor
  "Watches RUNME.rs files; on change, spawns a new engine", engines serve TCP JSONL.
  README also says "There's currently no in-built dependency graph", so not a monorepo manager.
- G2: pass.
- G3: fail, confusion (MCP mode documented only in a design doc that contradicts the binary name).
  - The README has no MCP section; the only MCP description is the design document
  - https://github.com/dgrijalva/runme/blob/main/docs/mcp_design.md (curl HTTP 200), which says "`--mcp` — MCP server
  - on stdio" and uses `rnme --mcp` / `rnme --engine` four times.
  - README and `Cargo.toml` (`[[bin]] name = "runme"`) say the installed binary is `runme`.
  - `docs/` mixes design documents (`build_system_design.md`, `runtime_engine_design.md`, `open_issues.md`) with the
  - user manual (`docs/manual/rnme/SKILL.md`).
- Final: exit at G3 confusion.

## Survivors

None.

Exits by gate (19 screened: 17 new plus 2 carried over):

- G1: 0 (borderline passes recorded for gw, Wireit, nsfw.d, zzz)
- G2: 2 (nsfw.d, don: license declared only in a manifest, no license file)
- G3: 17
  - no-JS: 2 (gw, zzz)
  - confusion: 11 (fx, mono-cli, Jazelle, Live Test Runner, Monex, Wireit, fluid-build, Shadowdog, Tilt,
  - process-compose, runme)
  - coverage: 4 (fwatchd, Invincible, affected, lazyrepo)

## Later taxonomy terms

Recorded only; no queries appended.

- Build Server Protocol (BSP), BSP server, IDE build integration
- task runner daemon, JSON-RPC task daemon, terminal multiplexer daemon
- file watcher socket daemon, file-watching daemon with control CLI (`fwatchctl`)
- process manager with HTTP API for agents, dev-stack orchestrator, dev process cockpit, readiness probes, port
- allocation, socket passing (`LISTEN_FDS`)
- daemon API over unix socket (`.don/don.sock`), discovery file (`.livetest/daemon.json`), NDJSON event protocol over
- TCP, event snapshot on connect
- generational engine supervision (runme MCP)
- affected detection, blast radius, AST-level semantic change detection, lockfile-affected, selective CI matrix
- continuous build, re-run build on change, stale build cancellation
- remote execution API (REAPI), build cache server
- project graph, task graph, release groups (fluid-build), execution cascade and services (Wireit), incremental
- detection per executable
- artifact generation watchers, artifact lock file (Shadowdog)
- Tiltfile, FileWatch API object, trigger mode, `.tiltignore`
- process-compose merge and override files, scheduled processes, dependency cascade on file change
- persistent tasks, long-running tasks (mostly AI-agent vocabulary in results)
- leads seen inside results, not screened: `neptaco/mcproc` (Rust daemon plus CLI plus MCP for background dev
- processes, E5 README), omni task runner (`@omni-oss/task-bench` benchmarks "omni/turbo/nx", E13 p1), PM2 (only
- add-ons returned, E13 and E14)
