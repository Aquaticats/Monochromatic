# Monorepo manager discovery registry

Discovery run date: 2026-09-16.
Scope: candidate monorepo managers (task/build orchestrators that run arbitrary command tasks across workspace
packages) and component tools that provide a persistent watch or process with an inspection surface that could compose
with one.
Screening used registry metadata only (name, description, keywords, last publish, repository link).
No primary-source verification (README, docs, source) was done for any candidate in this round.

Definitions used in this file:

- **result**: one package or crate object returned by the provider on that page.
- **new names**: results whose package or crate name had not appeared on any earlier page of the frozen schedule
- (schedule order Q1 to Q8).
- **new screening survivors**: distinct tools (package families collapsed, forks and mirrors merged) first recorded as
- survivors on that page.
- **survivor**: the registry description itself claims a category function (runs tasks across packages, build
- orchestration, task runner, or a persistent watch/process with dashboard, daemon, monitor or API).
- **survivor (unverified)**: the description plausibly fits but is vague, very short, or has no repository link; verify
- first in the next round.
- **generic task runner**: a Make/Task/just-class runner with no stated workspace graph.
  It was kept because the brief lists Task as an in-scope example.
- **excluded**: out of category (library, template/scaffold, bundler, package-manager-only, CI service,
- release/versioning tool, linter, agent orchestration, cron/job queue, script picker, delegating command router,
- add-on of another manager, multi-repo tool, insufficient description).
- Archived or abandoned status is noted, never a disqualifier.

## Query ledger

### Q1: npm keywords:monorepo

- Literal URL:
- `https://registry.npmjs.org/-/v1/search?text=keywords:monorepo&size=250&popularity=1.0&quality=0.0&maintenance=0.0&fro
- m=<N>`
- Provider: npm registry search API
- Filters: `keywords:monorepo`
- Sort: popularity weight 1.0, quality 0.0, maintenance 0.0
- Pages fetched: 0 to 17 (from=0 to from=4250)
- Provider total: 4145 on pages 0 and 1, 4146 on pages 2 to 17 (the live index grew during the run)
- Rate limiting: pages 14 to 16 first returned HTTP 429; they were refetched successfully with 8 s spacing a few
- minutes later.
  The refetch introduces a small ordering-drift risk at those offsets.
- Spam: pages 2 to 7 were dominated by lorem-ipsum scoped packages published 2024-04/05 (1121 hidden across the query
- by a scope/name/date filter during review).
  An audit of every hidden description for task, runner, build system, build tool, orchestrat, watch or daemon signals
  found one hit, a copied release-it README (spam).
- Per page (results / new names / new screening survivors):
  - p0: 250 / 250 / 18
  - p1: 250 / 250 / 11
  - p2: 250 / 249 / 11
  - p3: 250 / 250 / 3
  - p4: 250 / 250 / 0
  - p5: 250 / 250 / 1
  - p6: 250 / 250 / 3
  - p7: 250 / 250 / 4
  - p8: 250 / 250 / 15
  - p9: 250 / 250 / 17
  - p10: 250 / 250 / 12
  - p11: 250 / 250 / 20
  - p12: 250 / 250 / 9
  - p13: 250 / 250 / 13
  - p14: 250 / 250 / 12
  - p15: 250 / 250 / 7
  - p16: 146 / 146 / 5
  - p17: 0 / 0 / 0
- Stop reason: exhausted (p16 partial, p17 empty).
  Never saturated: p4 had no survivor, but p5 did, so two consecutive empty pages never occurred.
- Coverage gap: `nx`, `turbo`, `wireit`, `lage` and `@moonrepo/cli` do not appear in any npm result page of Q1, Q2 or
- Q3 (checked by exact name against every saved page), so npm keyword metadata misses several major tools.

### Q2: npm keywords:build-system

- Literal URL:
- `https://registry.npmjs.org/-/v1/search?text=keywords:build-system&size=250&popularity=1.0&quality=0.0&maintenance=0.0
- &from=<N>`
- Provider: npm registry search API
- Filters: `keywords:build-system`
- Sort: popularity weight 1.0, quality 0.0, maintenance 0.0
- Pages fetched: 0 and 1
- Provider total: 48
- Per page (results / new names / new screening survivors):
  - p0: 48 / 36 / 5 (12 names already seen in Q1: `@front-ops/domino`, `@nu-art/build-and-install`, three
  - `@fabr-build/*`, `@fiducial/fiducial`, `monox` and four `@monoxon/*` binaries, `@monoloom/cli`)
  - p1: 0 / 0 / 0
- Stop reason: exhausted (single partial page, then empty).

### Q3: npm keywords:task-runner

- Literal URL:
- `https://registry.npmjs.org/-/v1/search?text=keywords:task-runner&size=250&popularity=1.0&quality=0.0&maintenance=0.0&
- from=<N>`
- Provider: npm registry search API
- Filters: `keywords:task-runner`
- Sort: popularity weight 1.0, quality 0.0, maintenance 0.0
- Pages fetched: 0 to 2
- Provider total: 330
- Per page (results / new names / new screening survivors):
  - p0: 250 / 223 / 44 (27 names already seen in Q1 or Q2)
  - p1: 80 / 76 / 13 (4 names already seen: `ditoh`, `repo-runner`, `@osndot/osn`, `sleetrepo`)
  - p2: 0 / 0 / 0
- Stop reason: exhausted.
- Noise profile: the keyword is shared by AI-agent task loops, job queues, cron schedulers and Hardhat (smart-contract)
- forks; all excluded.

### Q4: crates.io q=build system

- Literal URL: `https://crates.io/api/v1/crates?q=build%20system&sort=downloads&per_page=100&page=<N>`
- Provider: crates.io API, User-Agent `monochromatic-vet-discovery`
- Filters: full-text `build system`
- Sort: downloads
- Pages fetched: 1 and 2
- Provider total: 30451
- Per page (results / new names / new screening survivors):
  - p1: 100 / 100 / 0 (hashbrown, syn, getrandom, libc, tokio and similar top-downloaded libraries)
  - p2: 100 / 100 / 0 (system-deps, tauri family, diesel and similar)
- Stop reason: saturated (two consecutive complete pages with no survivor).
- Relevance caveat: under download sort the query matches the broad tokens, so any real build systems in the 30451
- results are buried far below the pages the rule allowed.
  Treat this source as low-yield rather than as evidence of absence.

### Q5: crates.io q=task runner

- Literal URL: `https://crates.io/api/v1/crates?q=task%20runner&sort=downloads&per_page=100&page=<N>`
- Provider: crates.io API, User-Agent `monochromatic-vet-discovery`
- Filters: full-text `task runner`
- Sort: downloads
- Pages fetched: 1 to 19
- Provider total: 1771 (page 19 reported total 0 with no crates)
- Per page (results / new names / new screening survivors):
  - p1: 100 / 98 / 10
  - p2: 100 / 100 / 19
  - p3: 100 / 100 / 15
  - p4: 100 / 100 / 12 (cargo-rssc also matched but is merged into the cargo-auto entry)
  - p5: 100 / 100 / 5
  - p6: 100 / 100 / 2
  - p7: 100 / 99 / 2
  - p8: 100 / 100 / 6
  - p9: 100 / 100 / 7
  - p10: 100 / 100 / 7
  - p11: 100 / 100 / 5
  - p12: 100 / 100 / 2
  - p13: 100 / 100 / 3
  - p14: 100 / 99 / 2
  - p15: 100 / 100 / 7
  - p16: 100 / 100 / 7
  - p17: 100 / 100 / 5
  - p18: 71 / 70 / 1
  - p19: 0 / 0 / 0
- Stop reason: exhausted (never two consecutive pages without a survivor).
- Noise profile: AI agent runtimes, async executors, embedded crates and rCore tutorial crates.

### Q6: crates.io q=monorepo

- Literal URL: `https://crates.io/api/v1/crates?q=monorepo&sort=downloads&per_page=100&page=<N>`
- Provider: crates.io API, User-Agent `monochromatic-vet-discovery`
- Filters: full-text `monorepo`
- Sort: downloads
- Pages fetched: 1 to 8
- Provider total: 1777
- Per page (results / new names / new screening survivors):
  - p1: 100 / 98 / 3
  - p2: 100 / 96 / 5
  - p3: 100 / 92 / 2
  - p4: 100 / 91 / 1
  - p5: 100 / 88 / 1
  - p6: 100 / 91 / 1
  - p7: 100 / 93 / 0
  - p8: 100 / 90 / 0
- Stop reason: saturated (p7 and p8 complete pages with no new survivor).
- Noise profile: large library families that live in monorepos (nym, zebra, aprender, hjkl, libreoffice-rs `lo_*`,
- fallow, stac, taceo) plus release/versioning tools.

### Q7: monorepo.tools comparison

- Literal URLs: `https://monorepo.tools` (no comparison table; it links to `/compare`) and
- `https://monorepo.tools/compare`
- Provider: monorepo.tools (Nx team site), fetched with WebFetch and cross-checked against raw HTML from curl (HTTP
- 200, 633312 bytes for `/compare`)
- Filters / sort: none (single page)
- Pages fetched: 2 (landing page plus `/compare`)
- Results: 9 compared tools: Bazel, Gradle, Lage, Lerna, moon, Nx, Pants, Rush, Turborepo.
  Also referenced on `/compare`: `bazel-contrib/target-determinator` (Bazel affected detection), BuildBuddy and EngFlow
  (remote execution services), Nx Cloud.
- New screening survivors: 7 (Bazel, Gradle, Lage, moon, Nx, Pants, Turborepo; Lerna and Rush were already recorded
- from Q1)
- Comparison dimensions listed: local computation caching, local task orchestration, distributed computation caching,
- distributed task execution, transparent remote execution, detecting affected projects/packages, task splitting,
- deflaking, workspace analysis, project graph visualization, source code sharing, polyglot support, code generation,
- project constraints and visibility, hermetic builds, MCP server, AI skills, workspace analysis for AI, task execution
- via AI, agentic CI.
- Stop reason: exhausted (single page source).

### Q8: korfuri/awesome-monorepo README

- Literal command: `gh api repos/korfuri/awesome-monorepo/readme --jq .content | base64 --decode`
- Provider: GitHub API (repo not archived; last push 2024-08-16)
- Filters / sort: none (single document, 171 lines)
- Sections screened: Build systems & dependency management tools (24 entries), Repository management tools (10 entries,
- `oao` listed twice), Version control systems & add-ons tools (Git 14 entries, Mercurial 1 entry), Development process
- tools: CI tools (6 entries), Migration tools (5 entries).
  Good reads, code review, code ownership, notable public monorepos and development workflows were read and hold no
  tools in category.
- New screening survivors: 15 (baur, Bit, Bolt, Buck/Buck2, drkns, Garment, MBT, Nix, Please, pnpm, Yarn, Builder,
- Lank, wsrun, Watchman).
  Already recorded: Bazel, Lerna, Nx, OAO, Pants, Rush, Turborepo, monorepo-run, Ultra Runner.
- Stop reason: exhausted (single page source).

### Totals

- Distinct screening survivors: 375 (Q1 161, Q2 5, Q3 57, Q4 0, Q5 117, Q6 13, Q7 7, Q8 15), one candidate entry each
- in the Candidates section.
- Saturated sources: Q4, Q6.
- Exhausted sources: Q1, Q2, Q3, Q5, Q7, Q8.
- Blocked sources: none (the npm 429s on Q1 were transient and resolved by retry; no provider capped results before
- saturation or exhaustion).

## Candidates

Entry format: name, URL, discovered by (query and page), category guess, screening note, taxonomy terms seen.
Unless noted, "last" dates are the last registry publish, not repository activity.

### Monorepo managers and build systems: established (from Q7 and Q8)

- **Bazel**: https://bazel.build (github.com/bazelbuild/bazel, active, Apache-2.0); Q7, Q8; manager/build system;
- survivor; add-ons excluded elsewhere: `@paretools/bazel` MCP (Q2), `cargo-raze` (Q6 p1), `bazeld` (Q6 p4),
- `target-determinator` (Q7), Gazelle (Q8); terms: hermetic builds, remote execution, BUILD files, target determinator.
- **Gradle**: https://github.com/gradle/gradle (active, Apache-2.0); Q7; manager/build system; survivor; add-on
- `gradleup` excluded (Q3 p1); terms: polyglot, build cache.
- **Lage**: https://github.com/microsoft/lage (active, MIT); Q7; manager; survivor; absent from npm keyword results.
- **Lerna**: https://lerna.js.org (github.com/lerna/lerna); Q1 p0, Q7, Q8; manager; survivor; forks/repackages
- excluded: `@depup/lerna`, `@0x-lerna-fork/lerna`, `@spryker-lerna/lerna`, `@erquhart/lerna`, `merna`, `lernify`,
- `pubbo`, `yernak`, `@yoitsro/lerna`, `@mcesystems/lerna`, `lerna-mit`; terms: build system, multi-package, publish.
- **moon**: https://github.com/moonrepo/moon (active, MIT); Q7 (also implied by `callisto-moon` Moon WASM plugin in Q5
- p13 and `warpgate` from moonrepo/proto in Q6 p1); manager; survivor; absent from npm keyword results; terms: WASM
- plugins, polyglot.
- **Nx**: https://nx.dev (github.com/nrwl/nx, active, MIT); Q7, Q8; manager; survivor; add-ons excluded: `nx-mcp` (Q1
- p0), `@actionforge/nx`, `nx-dash` (Q1 p11), `nx-dev-cli` (Q1 p14), `nx-log-viewer`, `@meticoeus/terminx`, `nx-dagger`
- (Q1 p15), `magi-nx-axi` (Q5 p18), many Nx plugins; terms: MCP server, project graph, affected, Nx Cloud, distributed
- task execution.
- **Pants**: https://www.pantsbuild.org (github.com/pantsbuild/pants, active, Apache-2.0); Q7, Q8; manager/build
- system; survivor.
- **Rush**: https://rushjs.io (github.com/microsoft/rushstack); Q1 p0, Q7, Q8; manager; survivor; add-ons excluded:
- `@rushstack/mcp-server` (Q1 p1), `@rushstack/lockfile-explorer` (Q1 p0), `@rush-extras/*`, `@rushkit/select` (Q1
- p12); terms: build orchestrator, incremental, MCP server.
- **Turborepo**: https://turborepo.org (github.com/vercel/turborepo, active, MIT); Q7, Q8; manager; survivor; absent
- from npm keyword results; add-ons excluded: `turborepo-remote-cache`, `@yeger/turbo-graph` (Q1 p0), `hlidskjalf` (Q1
- p2), `turbometro` (Q1 p8), `@supamard/logview` (Q1 p9), `@turbotui/*` (Q1 p9, p15), `@abhijayb/turbo-groups` (Q1
- p10), `whycache` (Q1 p14, Q5 p18), `turbo-plus` (Q1 p15), `dex-runner` (Q1 p16); terms: remote cache, cache miss
- explanation, persistent tasks.
- **Buck / Buck2**: https://buck2.build (github.com/facebook/buck2 active; github.com/facebook/buck archived 2023); Q8
- (Buck); also implied by `buck-reindeer` (Q6 p5); manager/build system; survivor; terms: monorepo-oriented build
- system.
- **Please**: https://please.build (github.com/thought-machine/please, active, Apache-2.0); Q8; build system; survivor;
- terms: cross-language, reproducible.
- **baur**: https://github.com/simplesurance/baur (active, GPL-2.0); Q8; manager; survivor; terms: builds only changed
- applications, build artifact management.
- **Bit**: https://github.com/teambit/bit (active, license NOASSERTION per GitHub); Q8; manager (component-oriented);
- survivor; license needs checking; terms: component dependency graph.
- **Bolt**: https://github.com/boltpkg/bolt (last push 2024-06, not archived); Q8; manager (Yarn-era workspaces);
- survivor; likely abandoned.
- **drkns**: https://github.com/frantzmiccoli/drkns (active, license NOASSERTION); Q8; manager; survivor; terms:
- language agnostic.
- **Garment**: https://github.com/Farfetch/garment (archived 2024-01); Q8; manager; survivor (archived); terms:
- centralized customizable task management.
- **MBT**: https://github.com/mbtproject/mbt (last push 2023-10); Q8; build tool; survivor; terms: differential build.
- **Nix**: https://github.com/NixOS/nix (active, LGPL-2.1); Q8; build system (derivation graph); survivor (unverified
- fit as a workspace task orchestrator); related `now-runner` (Q5 p13); terms: remote caching, reproducible.
- **pnpm**: https://pnpm.io; Q8; package manager with recursive topological script runs; survivor (weak: package
- manager, no task cache); terms: workspace filter, recursive run.
- **Yarn**: https://yarnpkg.com; Q8; package manager with workspaces foreach; survivor (weak: package manager); terms:
- workspaces.
- **Builder (FormidableLabs)**: https://github.com/FormidableLabs/builder (archived 2022); Q8; shared script runner;
- survivor (archived).
- **Lank**: https://github.com/FormidableLabs/lank (archived 2022); Q8; manager (links packages, runs commands across
- all or subsets); survivor (archived).
- **wsrun**: https://github.com/hfour/wsrun (redirected from whoeverest/wsrun; last push 2024-05); Q8; manager (command
- per Yarn workspace package); survivor.
- **OAO**: https://github.com/guigrpa/oao (last push 2023-01; npm last 2021-04); Q1 p0, Q8; manager; survivor
- (abandoned).
- **monorepo-run**: https://github.com/Akryum/monorepo-run (archived 2021; npm last 2019-10); Q1 p0, Q8; manager
- (minimal); survivor (archived); fork `monorepo-run-workspace` excluded (Q1 p13); terms: separated panes.
- **Ultra Runner**: https://github.com/folke/ultra-runner (npm last 2021-02); Q1 p0, Q8; manager (lightweight);
- survivor (npm abandoned); forks `@42technologies/ultra-runner`, `@depup/ultra-runner`, `@jeffrson/ultra-runner`
- excluded; terms: dependency topology, skip unchanged builds.

### Monorepo managers and build systems: npm registry discoveries

- **Vite+ task runner (vp run)**: https://github.com/voidzero-dev/vite-task; Q1 p0 (via
- `@voidzero-dev/vite-task-client`); manager; survivor; license and openness of Vite+ need checking; add-on
- `vite-plugin-makefile` excluded (Q3 p1), `config-vp` excluded (Q1 p10); terms: cache-correctness client, task cache.
- **@visulima/vis** (with `@visulima/task-runner`, `@visulima/task-runner-client`): https://visulima.com/packages/vis;
- Q1 p0, Q3 p0; manager; survivor; terms: task runner, remote caching, git hooks, AI agent integrations,
- cache-correctness fingerprint client.
- **bun-workspaces**: https://bunworkspaces.com; Q1 p0; manager (Bun); survivor; terms: CLI and API.
- **pacwich**: https://pacwich.dev; Q1 p0; manager; survivor; terms: orchestrates package.json scripts, TypeScript
- library API.
- **mono-vir**: https://github.com/electrovir/mono-vir; Q1 p0; manager (minimal); survivor; terms: for each.
- **Holocron astromech (@theholocron/astromech)**: https://github.com/theholocron/holocron; Q1 p0, Q3 p0; manager
- wrapper; survivor (unverified; keyword turbo suggests it drives Turborepo); terms: one task manifest drives run, CI
- and checks.
- **yakumo**: https://github.com/cordiverse/yakumo; Q1 p0; manager (plugin-based workspace scripts); survivor; fork
- `@hieuzest/yakumo` excluded.
- **bun-spaces**: https://github.com/Pckool/bun-spaces; Q1 p0; manager (Bun); survivor; last 2025-01.
- **@williamthorsen/nmr**: https://github.com/williamthorsen/node-monorepo-tools; Q1 p0; manager (lightweight);
- survivor; terms: context-aware script runner.
- **npm-recursive-runner**: https://github.com/avi747av/npm-recursive-runner; Q1 p0; manager (minimal); survivor.
- **fynpo** (with `fyn`, `fynpo-cli`, `@fynpo/base`): https://github.com/jchip/fynjs; Q1 p0; manager; survivor; terms:
- zero setup, dep graph, topo.
- **@forklaunch/bunrun**: https://www.npmjs.com/package/@forklaunch/bunrun; Q1 p0; manager (Bun); survivor; terms:
- topological ordering.
- **@aklinker1/buildc**: https://github.com/aklinker1/buildc; Q1 p1; manager; survivor; terms: caching and
- orchestrating builds.
- **packer-commander**: https://github.com/Slowmoney/packer-commander; Q1 p1, Q3 p0; manager (TUI); survivor; terms:
- live task statuses, pipelines.
- **@enspirit/emb**: https://github.com/enspirit/emb; Q1 p1; manager; survivor; terms: Makefile-for-monorepos, docker
- compose, sentinel.
- **run-shared-scripts**: https://github.com/bubkoo/run-shared-scripts; Q1 p1; manager (minimal); survivor; last
- 2025-10.
- **workspace-utils**: https://torstendittmann.github.io/workspace-utils/; Q1 p1; manager; survivor; terms:
- dependency-aware builds, build orchestration.
- **northbrook**: https://github.com/northbrookjs/northbrook; Q1 p1; manager; survivor; last 2017-01 (abandoned).
- **scriptio**: https://github.com/michaelcocova/scriptio; Q1 p2; generic task runner; survivor.
- **rman**: https://github.com/panates/rman; Q1 p2; manager; survivor (unverified); terms: repository manager,
- lerna-like.
- **bru**: https://github.com/kamilkisiela/bru; Q1 p2; manager; survivor (unverified; may be dependency-only); last
- 2018-12.
- **@nu-art/build-and-install**: https://github.com/nu-art-js/thunderstorm; Q1 p2, Q2 p0; manager; survivor; terms:
- dependency-aware phase execution, units.
- **neex**: https://github.com/Neexjs/neex; Q1 p2; manager; survivor; last 2026-01; `create-neex` template excluded;
- terms: tiered caching, Rust.
- **@layermix/cli**: https://github.com/layermix-labs/cli; Q1 p2, Q3 p0; manager; survivor; terms: DAG-based task
- runner, Ink TUI.
- **@tinyaxis/toolkit**: https://www.npmjs.com/package/@tinyaxis/toolkit; Q1 p2, Q3 p0; generic task runner; survivor
- (unverified; no repo).
- **@halecraft/verify**: https://github.com/halecraft/verify; Q1 p2; generic task runner; survivor; terms: hierarchical
- verification runner.
- **ts-task (@oliveryasuna/ts-task)**: https://github.com/oliveryasuna/ts-task; Q1 p2 (plugin), Q3 p0; generic task
- runner; survivor; terms: TypeScript-configured tasks, plugins.
- **greenly**: https://github.com/yusifaliyevpro/greenly; Q1 p1, Q3 p0; generic task runner; survivor; terms: local CI,
- check runner.
- **rune (@multiterm/rune, also @super-repo/rune)**: https://www.npmjs.com/package/@multiterm/rune; Q1 p1, Q1 p7, Q3
- p0; generic task runner; survivor (unverified; no repo).
- **calviche**: https://github.com/sosafeapp/calviche; Q1 p3; manager (minimal); survivor; last 2020-08; terms:
- hierarchical dependency order.
- **laoban**: https://github.com/phil-rice/laoban; Q1 p3; manager; survivor.
- **qiao-project**: https://github.com/uikoo9/qiao-nodejs; Q1 p3; manager; survivor (unverified; vague description).
- **@meslzy/outdo**: https://meslzy.github.io/outdo/; Q1 p6, Q3 p0; generic task runner (Bun); survivor; terms: typed
- do.ts, Bun Shell.
- **laufen**: https://github.com/zrosenbauer/lauf; Q1 p6, Q3 p0; manager (lightweight); survivor; terms: typed script
- runner.
- **crowd (also @d-fischer/crowd)**: https://github.com/d-fischer/crowd; Q1 p6, Q1 p12; manager; survivor (unverified).
- **mondorepo**: https://github.com/sencha/mondorepo; Q1 p7; manager; survivor (unverified); last 2016-11.
- **Bilt (@bilt/cli and family)**: https://github.com/giltayar/bilt; Q1 p7; manager; survivor; last 2022-05; terms:
- artifact dependency graph, packages to build.
- **mrpm**: https://github.com/ota-meshi/mrpm; Q1 p7; manager; survivor (unverified); last 2022-02.
- **@jakehamilton/titan**: https://github.com/jakehamilton/packages; Q1 p8; manager; survivor (unverified); last
- 2024-08.
- **buildverse**: https://github.com/Typeverse/buildverse-sdk; Q1 p8; manager; survivor (unverified); last 2017-10.
- **vx (@vzn/vx, with @vzn/vx-cloud)**: https://github.com/vznjs/vx; Q1 p8, Q1 p12, Q3 p0; manager; survivor; terms:
- extensible task runner, cache, vx-cloud orchestrator service, distributed, remote cache.
- **lattice (@latticeandcompany/lattice)**: https://latticeandcompany.github.io/lattice; Q1 p8, Q3 p0; manager;
- survivor; terms: local toolchain, task-runner, cache.
- **monopkg**: https://github.com/beerush-id/monopkg; Q1 p8; manager; survivor (unverified).
- **fiducial (@fiducial/fiducial)**: https://github.com/AleksaZCodes/fiducial; Q1 p8, Q2 p0; build system; survivor
- (unverified); terms: derive every artifact from declared facts.
- **Fabr (@fabr-build/core, cli, js)**: https://fabr.build/; Q1 p8, Q1 p10, Q2 p0; build system; survivor.
- **rune (@gio-labs/rune)**: https://github.com/giancarlosisasi/rune; Q1 p8, Q3 p0; manager (lightweight); survivor;
- terms: centralized script runner.
- **unorepo**: https://github.com/0livare/unorepo; Q1 p8; manager; survivor; last 2023-09; terms: build, watch.
- **Roc with roc-plugin-repo** (also `roc-plugin-repo-react`, `roc-plugin-repo-roc`):
- https://www.npmjs.com/package/roc-plugin-repo; Q1 p8, p13, p14; manager (plugin); survivor (unverified); last 2018-02.
- **changeset-releaser**: https://github.com/harnyk/changeset-releaser; Q1 p8; manager; survivor; last 2023-10; terms:
- incremental builds and tests from changesets.
- **monox (with @monoxon/* binaries)**: https://github.com/monoxon/monox; Q1 p8, Q1 p9, Q2 p0; manager; survivor; last
- 2025-08; terms: Rust, concurrent, dependency analysis.
- **@varlabs/monorun**: https://github.com/HamzaKV/monorun; Q1 p8; manager; survivor; last 2025-05; `create-monorun`
- template excluded.
- **@enzsft/mono**: https://github.com/enzsft/mono; Q1 p9; manager; survivor (unverified); last 2019-04.
- **@shazhou/proman (with proman-core)**: https://www.npmjs.com/package/@shazhou/proman; Q1 p9, p10; manager; survivor
- (unverified; no repo).
- **Haetae (@haetae/core, cli, git, javascript, utils; `haetae`)**: https://github.com/haetae-org/haetae; Q1 p9, p10,
- p11; manager; survivor; last 2023-12; terms: incremental test/lint/build, automatic dependency graph resolver.
- **bun-manage-workspace**: https://github.com/kkiwior/bun-manage-workspace; Q1 p9; manager (Bun); survivor; terms:
- graph-aware script runner with cache, graph diagnostics.
- **moci**: https://oss.zero-one-group.com/monorepo; Q1 p9; manager; survivor (unverified).
- **lerna-run**: https://github.com/shokai/lerna-run; Q1 p9; manager (minimal); survivor; last 2016-02.
- **dragon (@bunvader/dragon)**: https://github.com/Flora90001/dragon; Q1 p9; manager; survivor; terms: experimental
- build system, cache.
- **@zssz-soft/zs**: https://github.com/zssz-soft/libraries; Q1 p9; manager (org-specific); survivor; terms: hash-based
- incremental builds.
- **monilla**: https://github.com/ctrlplusb/monilla; Q1 p9; manager; survivor (unverified); last 2022-06.
- **just-build-tools**: https://github.com/mattsibs/just-build-tools; Q1 p9; manager; survivor; last 2024-10; terms:
- dependency graph, tech-agnostic.
- **@speedy-js/mono (fork @nomadland/mono)**: https://github.com/speedy-js/workflow; Q1 p9, p10; manager; survivor
- (unverified); last 2022-02.
- **hammerkit**: https://no0dles.gitbook.io/hammerkit/; Q1 p9, Q3 p0; manager; survivor; terms: containerized build,
- incremental caching, local, Docker and Kubernetes.
- **mono-runner**: https://github.com/JanNitschke/mono-runner; Q1 p9; manager (lightweight); survivor; last 2025-05.
- **pmnps (with @pmnps/* plugins)**: https://github.com/filefoxper/pmnps-workspace; Q1 p9 to p13; manager; survivor
- (unverified); last 2024-12.
- **@webeferen/buildable**: https://github.com/WebEferen/buildable; Q1 p10; manager; survivor (unverified); last
- 2023-03.
- **monist / monist-tools**: https://github.com/lddubeau/monist; Q1 p10, p11; manager; survivor (unverified); last
- 2020-02 and 2021-11.
- **mrdr**: https://github.com/padcom/mrdr; Q1 p10; manager; survivor.
- **yamat**: https://github.com/cancerberoSgx/yamat; Q1 p10; manager; survivor; last 2019-06; terms: lerna and rush
- alternative.
- **workspaces-filter**: https://github.com/tunnckoCore/workspaces-filter; Q1 p10; manager (minimal); survivor; terms:
- run scripts on workspace subsets.
- **mr-yarn**: https://github.com/leecheneler/mr-yarn; Q1 p10; manager; survivor (unverified); last 2018-10.
- **workspace-builder**: https://github.com/suchipi/workspace-builder; Q1 p10; manager (minimal); survivor; last
- 2021-02.
- **kodrdriv tree (@grunnverk/tree-execution, @grunnverk/commands-tree)**: https://github.com/grunnverk/tree-execution;
- Q1 p10; manager; survivor; terms: parallel execution, dependency graph, checkpoint recovery.
- **@jacob-ebey/mono-build**: https://www.npmjs.com/package/@jacob-ebey/mono-build; Q1 p10; manager (minimal);
- survivor; last 2018-09.
- **verify-grid**: https://github.com/kirull1/verify-grid; Q1 p10; generic task runner; survivor; terms: task matrix,
- status reporting.
- **yanice (with @yanice/import-boundaries)**: https://github.com/abuob/yanice; Q1 p10, p12; manager; survivor; terms:
- incremental command executor, change detection.
- **buildsure**: https://github.com/clasen/BuildSure; Q1 p11; manager (lightweight); survivor; terms: build only when
- sources changed, mtime.
- **kitchen-tools**: https://github.com/JacopoPatroclo/kitchen-tools; Q1 p11; manager; survivor (unverified); last
- 2020-10.
- **workgraph**: https://www.npmjs.com/package/workgraph; Q1 p11; manager; survivor (no repo); terms: parallel build
- orchestrator, watch.
- **affected-ci**: https://github.com/Cst2989/affected-ci; Q1 p11; manager (affected runner); survivor; terms:
- import-graph affected detection.
- **@monoloom/cli**: https://git.fallet.net/monoloom/workspace; Q1 p11, Q2 p0; manager; survivor; terms: affected build
- and test.
- **@blaze-js/cli**: https://www.npmjs.com/package/@blaze-js/cli; Q1 p11; manager; survivor (unverified; no repo); last
- 2022-09.
- **nasti-task (Nasti Plus)**: https://github.com/nasti-toolchain/nasti-task; Q1 p11, Q3 p0; manager; survivor; terms:
- dependency-aware, cache.
- **Hark (@hark/plugin-monorepo)**: https://github.com/sparebytes/hark; Q1 p11; generic task runner; survivor
- (unverified); last 2020-04.
- **gmm**: https://github.com/k-koehler/gmm; Q1 p11; manager; survivor (unverified); last 2021-09.
- **rumos**: https://github.com/remirobichet/rumos; Q1 p11; manager; survivor (unverified); last 2023-05.
- **npm-mono-repo (Sodaru)**: https://github.com/sodaru/mono-repo; Q1 p11; manager; survivor (unverified); last 2022-09.
- **@yamato-daiwa/monorepo-helper**: https://github.com/TokugawaTakeshi/Yamato-Daiwa-Monorepo-Helper; Q1 p11; manager;
- survivor (unverified).
- **Rex (@rex-js/rex-cli, @rex-js/rex)**: https://github.com/nikeokoronkwo/rex; Q1 p11, p14; manager; survivor
- (unverified); last 2024-04.
- **fraktos**: https://github.com/holasoymalva/fraktos; Q1 p11; manager; survivor; terms: dependency ordering, log
- multiplexing.
- **robonaut**: https://github.com/f1lt3r/robotnaut; Q1 p11; manager; survivor (unverified); last 2017-01.
- **@emeryld/manager**: https://www.npmjs.com/package/@emeryld/manager; Q1 p11; manager (interactive); survivor
- (unverified; no repo).
- **bdep**: https://github.com/sourcewizard-ai/bdep; Q1 p11; manager (lightweight); survivor; terms: topological
- parallel builds.
- **Plymor (@plymor/cli, core, deploy adapters)**: https://github.com/plymor/plymor; Q1 p11 to p13, Q3 p0; manager;
- survivor; terms: polyglot orchestration, manifest, affected detection, deploy adapters.
- **npm-ws**: https://github.com/bstefanescu/npm-ws; Q1 p12; manager; survivor; last 2021-05; terms: multi-package
- build manager, watch.
- **auto-n-glad**: https://www.npmjs.com/package/auto-n-glad; Q1 p12; manager (minimal); survivor (no repo).
- **monopod**: https://github.com/studio-b12/monopod; Q1 p12; manager; survivor (unverified); last 2016-05.
- **obr (One Big Repo)**: https://github.com/sourcewizard-ai/obr; Q1 p12; manager; survivor.
- **sail (@tylerbu/sail)**: https://github.com/tylerbutler/tools-monorepo; Q1 p12; manager; survivor; terms: build
- orchestration CLI.
- **@anzerr/mono.cli**: https://github.com/anzerr/mono.cli; Q1 p13; manager; survivor (unverified); last 2022-01.
- **@sfomin/for-each-package**: https://github.com/slavafomin/for-each-package; Q1 p13; manager (minimal); survivor;
- last 2020-04.
- **mrex**: https://github.com/ssupinsky/mrex; Q1 p13; manager (minimal); survivor; last 2022-12; terms: order of
- inclusion.
- **hwan**: https://www.npmjs.com/package/hwan; Q1 p13; manager; survivor (unverified); last 2018-04.
- **alle (with alle-publish)**: https://github.com/kogosoftwarellc/alle; Q1 p13, p14; manager; survivor (unverified);
- last 2018-03.
- **orca (@fasunle/orca)**: https://github.com/fasunle/orca; Q1 p13, Q3 p0; manager; survivor; terms: task
- orchestration, build cache, turbo alternative.
- **bun-cache (@fasunle/bun-cache)**: https://github.com/fasunle/bun-cache; Q1 p14; manager; survivor; terms:
- Turbo-inspired task orchestrator, zero-config caching.
- **spinx**: https://github.com/nirikshan/spinx; Q1 p13; manager; survivor; terms: parallel execution.
- **monob**: https://www.npmjs.com/package/monob; Q1 p13; manager; survivor (unverified; no repo); last 2020-12.
- **lolaus**: https://gitlab.com/tom.davidson/lolaus; Q1 p13; manager; survivor; last 2018-05; terms: commands on
- directories with diffs against an ancestor.
- **@cvsky/monorepo-builder**: https://www.npmjs.com/package/@cvsky/monorepo-builder; Q1 p13; manager; survivor
- (unverified; no repo).
- **ditoh**: https://github.com/katerman/ditoh; Q1 p14, Q3 p1; manager (lightweight); survivor; terms: meta scripts.
- **BuildNet (@musclemap.me/buildnet)**: https://musclemap.me/buildnet; Q1 p14; manager; survivor (unverified; possibly
- org-specific); terms: build orchestration, incremental, caching.
- **onom**: https://github.com/pshrmn/onom; Q1 p14; manager; survivor (unverified); last 2018-08.
- **@mono-repo/cli**: https://github.com/mono-repo/mono-repo; Q1 p14; manager; survivor (unverified); last 2020-08.
- **monorunyg**: https://www.npmjs.com/package/monorunyg; Q1 p14; manager; survivor (no repo); terms: lightweight
- Turborepo alternative.
- **@kienleholdings/mrt-run**: https://github.com/kienleholdings/monorepo-tools; Q1 p14; manager (minimal); survivor;
- last 2022-02.
- **mr-monorepo**: https://github.com/SoulEvans07/mr-monorepo; Q1 p14; manager; survivor (unverified); last 2022-06.
- **@mykulyak/linterpol**: https://github.com/mykulyak/linterpol; Q1 p14; manager (minimal); survivor; last 2022-01;
- terms: modified packages.
- **@messman/node-mono-builder**: https://github.com/messman/node-mono-builder; Q1 p14; manager; survivor (unverified);
- last 2022-05.
- **quimera**: https://github.com/Tautorn/quimera; Q1 p14; manager; survivor (unverified; may be scaffold); last
- 2018-10.
- **tukod**: https://github.com/vexCoder/tukod; Q1 p15; manager; survivor (unverified); last 2022-11.
- **deicide**: https://github.com/binbandit/deicide; Q1 p15; manager; survivor (unverified).
- **@xelbera/monobuild**: https://github.com/matafonoff/monobuild; Q1 p15; manager; survivor; terms: incremental
- builds, dependency-aware sorting, build caching.
- **ghidora (@hyperforge/ghidora)**: https://ghidora.hyperforge.in; Q1 p15; manager; survivor; terms: Rust plus WASM,
- build, test, watch.
- **ystage**: https://github.com/amurdock/ystage; Q1 p15; manager; survivor (unverified); last 2019-01.
- **@linbop/modulify**: https://linbop.github.io/modulify; Q1 p15; manager; survivor (unverified); last 2022-02.
- **bun-pkg**: https://github.com/thejasonxie/bun-pkg; Q1 p15; manager (Bun); survivor (unverified); last 2024-04.
- **bouddha-monorepo**: https://github.com/andrade0/bouddha-monorepo; Q1 p16; manager; survivor (unverified); last
- 2024-07.
- **umrepo**: https://github.com/umrepo/um; Q1 p16; manager; survivor (unverified); last 2024-10; terms: zero-footprint.
- **@osndot/osn**: https://github.com/osndot/osn; Q1 p16, Q3 p1; generic task runner; survivor (unverified); terms:
- plugin-driven developer runtime.
- **rootrun**: https://github.com/Barani1912/rootrun; Q1 p16; manager (minimal); survivor.
- **Sleet (sleetrepo)**: https://github.com/dupyjs/sleet; Q1 p16, Q3 p1; manager; survivor; terms: incremental.
- **dk (@dkjs/cli)**: https://diskuv.com/dk/; Q2 p0; build system; survivor; terms: reproducible, deterministic.
- **simplebuild (@simple-software/simplebuild)**: https://github.com/SimpleSoftwareOrg/Simple-Build; Q2 p0; build
- system; survivor (unverified); last 2025-05.
- **Makedown (@makedown/cli, engine and family)**: https://github.com/khoi03/makedown; Q2 p0; build system; survivor
- (unverified; LLM-oriented); terms: incremental, content-addressed, DAG, provenance, sandboxing.
- **oxymake (@noogram/oxymake)**: https://noogram.org/oxymake; Q2 p0; workflow/build orchestration; survivor
- (unverified); terms: Rust, make.
- **nadle (@nadle/cli, kernel, language-server, eslint-plugin-nadle)**: https://nadle.dev; Q3 p0, p1; manager;
- survivor; terms: Gradle-inspired, workspace resolution, language server.
- **linkctl**: https://github.com/saintparish4/linkctl; Q3 p0; manager; survivor; terms: task DAG, content caching,
- runtime detection.
- **reckon (@andrew24601/reckon)**: https://www.npmjs.com/package/@andrew24601/reckon; Q3 p0; manager; survivor; terms:
- incremental build graph runner.
- **frunk**: https://github.com/ludicroushq/frunk; Q3 p0; manager (script orchestration); survivor; terms: wireit,
- npm-run-all.
- **nalth**: https://www.nalthjs.com; Q3 p0; unified toolchain; survivor (unverified); last 2025-12; terms: vite-plus,
- task-runner.
- **machora**: https://github.com/jasonz1987/machora; Q3 p0; distributed job orchestration; survivor (unverified);
- terms: build orchestration across machines.

### Monorepo managers and build systems: crates.io discoveries

- **cargo-make**: https://github.com/sagiegurari/cargo-make; Q5 p1; generic task runner and build tool; survivor; last
- 2025-01.
- **garden (garden-tools, garden-gui)**: https://garden-rs.gitlab.io (gitlab.com/garden-rs/garden); Q5 p1, p4; manager
- (commands over collections of git trees); survivor.
- **Chomp (chompbuild)**: https://github.com/guybedford/chomp; Q5 p1, Q6 p1; manager; survivor; terms: Make-like
- parallel task runner, JS extensions.
- **tinyrick (with macros, models)**: https://github.com/mcandre/tinyrick; Q5 p1, p4, p3; build system; survivor.
- **astro-run (with remote runner, protocol)**: https://github.com/panghu-huang/astro-run; Q5 p2; workflow
- orchestrator; survivor (unverified; may be CI-like); last 2024-07; terms: remote runner.
- **cuenv (cuenv, cuengine and family)**: https://github.com/cuenv/cuenv; Q5 p2 to p14, Q6 p2, p3, p5; manager;
- survivor; terms: CUE, task graph DAG, content-addressed task caching, CAS modelled on Bazel Remote Execution API,
- workspaces across package managers, service supervision, readiness probes, Dagger backend, CI emitters, codegen.
- **whiz**: https://crates.io/crates/whiz; Q5 p2, Q6 p2; manager; survivor (no repo link); last 2023-08; terms:
- DAG/tasks runner for multi-platform monorepos.
- **dors**: https://github.com/aklitzke/dors; Q5 p2; manager (cargo workspace); survivor; last 2020-06.
- **maid**: https://github.com/theMackabu/maid; Q5 p2; generic task runner; survivor; terms: dependencies, cached build
- steps.
- **engage**: https://or.computer.surgery/charles/engage; Q5 p2; generic task runner; survivor; last 2023-09; terms:
- DAG-based parallelism.
- **bake (bake-cli)**: https://github.com/trinio-labs/bake; Q5 p2, Q6 p3; manager; survivor; last 2024-02.
- **cargo-flux**: https://github.com/ignition-is-go/cargo-flux; Q5 p3, Q6 p5; manager; survivor; terms: workspace
- topology, task orchestration, mixed-language.
- **fern (fern-run)**: https://github.com/felipesere/fern; Q5 p3; manager; survivor; last 2020-03; terms: unified task
- interface across a mono-repo.
- **fyrer**: https://github.com/07calc/fyrer; Q5 p4, Q6 p5; manager; survivor; terms: declarative language-agnostic
- monorepo task orchestrator.
- **taskrush**: https://github.com/iPeluwa/rush; Q5 p4; generic task runner; survivor; last 2025-07; terms: parallel,
- intelligent caching.
- **make_ultra**: https://github.com/CoolOppo/make-ultra; Q5 p4; generic task runner; survivor; last 2022-04; terms:
- tracks file changes.
- **compi**: https://github.com/allyedge/compi; Q5 p4; build system; survivor (unverified).
- **heron-rebuild**: https://github.com/heronsounds/heron-rebuild; Q5 p4; workflow runner; survivor (unverified); last
- 2024-10.
- **haz (haz-cli and family)**: https://forge.cloudsling.dev/andreu/haz-cli; Q5 p8 to p15; manager; survivor; terms:
- task DAGs across a workspace of projects, content-addressed cache, task query language, target selection, discovery.
- **trellis-gleam**: https://github.com/tylerbutler/trellis; Q5 p9; manager (Gleam); survivor; terms: task fan-out,
- introspection, release orchestration.
- **monorepo-meta**: https://github.com/wolven-tech/rust-v1; Q5 p9; meta orchestrator; survivor; terms: orchestrates
- Turborepo, Cargo and Bacon in tmux.
- **repoctl-runner (Rust repoctl)**: https://github.com/tyrchen/repoctl; Q5 p10; manager; survivor; terms: affected
- analysis, CI matrix, task execution.
- **zcheck (with zcheck-core)**: https://github.com/zsumz/zcheck; Q5 p10; manager; survivor (unverified); terms:
- deterministic repository qualification task graphs.
- **zetten (ztn)**: https://docs.zetten.in (github.com/amit-devb/zetten); Q5 p10, p13; manager; survivor; terms:
- Python-aware, deterministic caching, DAG scheduling.
- **yatr (with yatr-plugin)**: https://github.com/cargopete/yatr; Q5 p10, p17; manager; survivor; terms:
- content-addressed shared cache, WASM task plugins.
- **kiln (kiln-core, cache, exec, runtime, cli)**: https://github.com/nebucloud/kiln; Q5 p10, p11, p13, p15;
- build/pipeline system; survivor (unverified); terms: hermetic, content-addressed (BLAKE3), parallel.
- **nao (nao, nao-base, pal, recipe, engine, tui)**: https://github.com/manuel-woelker/nao; Q5 p7, p8, p10, p11, p13;
- manager; survivor (unverified); terms: run planning engine.
- **runkernel (with cli)**: https://github.com/Adriftdev/runkernel; Q5 p7, p13, p14; workflow engine; survivor
- (unverified); terms: define, inspect and run task graphs.
- **winch**: https://codeberg.org/esavier/winch; Q5 p14; manager (affected CI); survivor (unverified); terms: granular
- CI helper for large Rust repositories.
- **guild (guild-cli)**: https://github.com/sprouted-dev/guild; Q5 p16; manager; survivor; terms: polyglot monorepo
- orchestrator.
- **buildsmith**: https://github.com/ze/buildsmith; Q5 p17; build system; survivor; terms: content-hashed, DAG-based.
- **breach (breach-cli)**: https://github.com/radical-beard/breach; Q5 p17; manager; survivor; terms: config-driven
- monorepo command runtime.
- **rivox**: https://rivox.dev (github.com/Grevix/Rivox); Q5 p18; manager; survivor; terms: polyglot build coordination
- for Python, Rust and Node.
- **now-runner**: https://now.dev.br (github.com/EpicEric/now); Q5 p13; distributed runner; survivor (unverified);
- terms: Nix-based distributed command runner.
- **tracel-xtask (with cli, utils)**: https://github.com/tracel-ai/xtask; Q6 p1, p2; xtask framework; survivor
- (unverified); terms: reusable repository commands.
- **anda (Andaman, with andax, anda-config)**: https://github.com/FyraLabs/anda; Q6 p1, p2; build toolchain; survivor
- (unverified); terms: scripting runtime.
- **cargo-mono**: https://github.com/delinoio/oss; Q6 p1; manager (Rust); survivor (unverified; may be release-only).
- **monorail**: https://github.com/pnordahl/monorail; Q6 p2; manager; survivor; last 2024-12; terms: polyglot,
- multi-project.
- **orcs**: https://github.com/nmoutschen/orcs; Q6 p2; manager; survivor; last 2020-12; terms: microservices monorepo
- orchestration.
- **Blaze (blaze-cli)**: https://blaze-monorepo.dev (github.com/rnza0u/blaze); Q6 p2; build system; survivor; last
- 2024-09.
- **esteem**: https://github.com/IgnisDa/developrs; Q6 p2; manager (reads Nx workspaces); survivor (unverified); last
- 2022-07.
- **hammer (hammer-cli)**: https://crates.io/crates/hammer-cli; Q6 p2; manager; survivor (no repo link); last 2023-03;
- terms: concurrent tasks, monorepo support.
- **shiv (shivr)**: https://github.com/xtenduke/shiv; Q6 p3; manager; survivor; last 2023-12; terms: monorepo package
- command runner.
- **affected**: https://github.com/DenysVuika/affected; Q6 p3; manager (affected runner); survivor; last 2024-11.
- **superepo**: https://github.com/GrandEngineering/superepo; Q6 p4; manager; survivor; last 2024-12; terms: builds and
- runs.
- **cargo-tangerine**: https://github.com/alxolr/cargo-tangerine; Q6 p5; manager (Cargo); survivor; terms: inspired by
- Lerna.
- **cargo-monorepo**: https://github.com/legion-labs/cargo-monorepo; Q6 p6; manager (Cargo); survivor (unverified);
- last 2021-12.

### Generic task runners (no stated workspace graph)

npm (Q3):

- **Task (go-task)**: https://taskfile.dev (github.com/go-task/task); Q3 p0 (`@go-task/cli`; mirrors `@nmnmcc/task`,
- `@ryusuke410/go-task-cli`, `@ryusuke410/setup-go-task-cli`; repackage `@ssuf1998dev/task*`); survivor; terms:
- Taskfile, Make-inspired.
- **just**: https://github.com/casey/just; Q3 p0 (TS port `@indiekitai/just`), Q5 p1 (`just`, `pub-just`); survivor;
- add-ons excluded: `just-mcp`, `just-mcp-lib`, `just-fancy`, `just-claude`; terms: justfile, command runner.
- **bunosh**: https://www.npmjs.com/package/bunosh; Q3 p0; survivor; terms: JS functions as CLI commands.
- **@jonacem/do-file**: https://www.npmjs.com/package/@jonacem/do-file; Q3 p0; survivor; terms: YAML tasks with
- dependencies.
- **mxflow**: https://www.npmjs.com/package/mxflow; Q3 p0; survivor; last 2023-06.
- **task-pipeliner**: https://www.npmjs.com/package/task-pipeliner; Q3 p0; survivor (unverified).
- **@madinco/smith**: https://www.npmjs.com/package/@madinco/smith; Q3 p0; survivor (unverified).
- **zet-x**: https://www.npmjs.com/package/zet-x; Q3 p0; survivor.
- **@glyphtek/scriptit**: https://www.npmjs.com/package/@glyphtek/scriptit; Q3 p0; survivor (unverified).
- **bask.sh**: https://www.npmjs.com/package/bask.sh; Q3 p0; survivor (minimal); last 2020-09.
- **swig-cli**: https://www.npmjs.com/package/swig-cli; Q3 p0; survivor.
- **@xenoverseup/trane**: https://www.npmjs.com/package/@xenoverseup/trane; Q3 p0; survivor.
- **@ras0q/cute**: https://www.npmjs.com/package/@ras0q/cute; Q3 p0; survivor; terms: commands from Markdown.
- **invokej**: https://www.npmjs.com/package/invokej; Q3 p0; survivor; terms: Python Invoke-inspired.
- **invoket**: https://www.npmjs.com/package/invoket; Q3 p0; survivor.
- **oh**: https://www.npmjs.com/package/oh; Q3 p0; survivor (unverified); last 2020-10.
- **run-task**: https://www.npmjs.com/package/run-task; Q3 p0; survivor; last 2017-07.
- **helm-ctrl**: https://www.npmjs.com/package/helm-ctrl; Q3 p0; survivor (unverified).
- **boot-stacker**: https://www.npmjs.com/package/boot-stacker; Q3 p0; survivor (unverified); last 2017-02.
- **gue (formerly gluey)**: https://www.npmjs.com/package/gue; Q3 p0, p1; survivor; last 2017-08.
- **ok-runner**: https://www.npmjs.com/package/ok-runner; Q3 p0; survivor; last 2017-03.
- **@zemerik/task-runner**: https://www.npmjs.com/package/@zemerik/task-runner; Q3 p0; survivor (unverified).
- **master-roshi**: https://www.npmjs.com/package/master-roshi; Q3 p0; survivor (minimal); last 2018-10.
- **noxedo**: https://www.npmjs.com/package/noxedo; Q3 p0; survivor (unverified); terms: nox-style sessions, test
- matrix.
- **@samvv/bake (BakeJS)**: https://github.com/samvv/BakeJS; Q3 p0; survivor (unverified); last 2022-05.
- **jido (with jido-kit)**: https://www.npmjs.com/package/jido; Q3 p0; survivor; terms: named flows.
- **clii**: https://www.npmjs.com/package/clii; Q3 p0; survivor; last 2022-02.
- **@devx-cli/devx**: https://www.npmjs.com/package/@devx-cli/devx; Q3 p0; survivor (unverified).
- **@yampp/yampp**: https://www.npmjs.com/package/@yampp/yampp; Q3 p0; survivor; terms: concurrent, DSL.
- **jtg**: https://www.npmjs.com/package/jtg; Q3 p0; survivor; last 2021-06.
- **metcalf**: https://www.npmjs.com/package/metcalf; Q3 p0; survivor; last 2021-07.
- **shellforge**: https://github.com/BaerBonesTechnology/shellforge; Q3 p0; survivor (unverified).
- **Ease task manager (via ease-task-sass)**: https://github.com/chisel/ease-task-sass; Q3 p0; survivor (unverified);
- last 2021-02.
- **@guiho/runx**: https://github.com/CGuiho/runx; Q3 p0; survivor (unverified); terms: Go/Cobra command catalog.
- **npm-tasks**: https://github.com/frontainer/npm-tasks; Q3 p1; survivor (unverified); last 2016-10.
- **run-project-commands**: https://github.com/AdarshHatkar/run-project-commands; Q3 p1; survivor (unverified).
- **Flow (@nikivdev/flow)**: https://myflow.sh; Q3 p1; survivor.
- **buildfile (@dev-kas/buildfile)**: https://github.com/dev-kas/buildfile; Q3 p1; survivor; terms: DSL, Makefile
- alternative.
- **mdrun (@leyohli/mdrun)**: https://github.com/liyiheng/mdrun; Q3 p1; survivor; terms: Markdown-based.
- **pylp**: https://github.com/ppetkov89/pylp; Q3 p1; survivor (unverified); last 2023-10.
- **buildtool.js**: https://github.com/leso-kn/buildtool.js; Q3 p1; survivor; last 2022-07.
- **ssal**: https://www.npmjs.com/package/ssal; Q3 p1; survivor (unverified); terms: automation language.
- **sarata-task-runner**: https://github.com/gorlIoll/sarata-task-runner; Q3 p1; survivor (unverified); last 2024-03.
- **violet.ts**: https://www.npmjs.com/package/violet.ts; Q3 p1; survivor.
- **i-do**: https://github.com/christianheyn/i-do; Q3 p1; survivor (unverified); last 2016-07.
- **zolo**: https://www.npmjs.com/package/zolo; Q3 p1; survivor.
- **@sohaha/zzz**: https://github.com/sohaha/zzz; Q3 p0; survivor (unverified); terms: daily development aids (Go).
- **vibe-queue**: https://www.npmjs.com/package/vibe-queue; Q3 p0; survivor (unverified; no repo); terms: execution
- engine, dashboard.
- **bs (@jaandrle/bs)**: https://github.com/jaandrle/bs; Q2 p0; survivor (minimal); terms: executable scripts as build
- system.

crates.io (Q5 unless noted):

- **cargo-auto (with cargo-rssc predecessor)**: https://codeberg.org/automation-tasks-rs/cargo-auto; Q5 p1, p4;
- survivor.
- **argc**: https://github.com/sigoden/argc; Q5 p1; survivor; terms: bash-based command runner.
- **nur**: https://github.com/nur-taskrunner/nur; Q5 p1; survivor; terms: nu shell.
- **neomake**: https://github.com/cchexcode/neomake; Q5 p1; survivor; last 2025-02.
- **rmake**: https://github.com/mass10/rmake; Q5 p1; survivor; last 2022-06.
- **mask**: https://github.com/jacobdeichert/mask; Q5 p1; survivor; terms: Markdown-defined tasks.
- **cargo-task**: https://github.com/neonphog/cargo-task; Q5 p2; survivor; last 2021-10.
- **mk**: https://github.com/ffimnsr/mk-rs; Q5 p2; survivor.
- **checkexec**: https://github.com/kurtbuilds/checkexec; Q5 p2; survivor (minimal); last 2021-12; terms: conditional
- run like Make.
- **devrc**: https://github.com/devrc-hub/devrc; Q5 p2; survivor; last 2023-06.
- **yamis**: https://github.com/adrianmrit/yamis; Q5 p2; survivor; last 2023-01.
- **mom (mom-task)**: https://github.com/adrianmrit/mom; Q5 p3; survivor; last 2023-06.
- **yake**: https://gitlab.com/elbartus/yake; Q5 p2; survivor; last 2020-07.
- **party (party-run)**: https://github.com/iamroot99/party; Q5 p2, p5; survivor (unverified).
- **haku**: https://github.com/VladimirMarkelov/haku; Q5 p2; survivor; last 2020-08.
- **rhiz**: https://bitbucket.org/nathanielknight/rhiz/; Q5 p2; survivor; last 2021-09.
- **ruke**: https://github.com/kauefraga/ruke; Q5 p2; survivor; last 2024-05.
- **rsbuild**: https://github.com/JeanMaximilienCadic/rsbuild; Q5 p2; survivor (unverified).
- **jog**: https://github.com/callum-oakley/jog; Q5 p3; survivor.
- **runme (sigoden)**: https://github.com/sigoden/runme; Q5 p3; survivor; last 2023-03.
- **sate**: https://github.com/nicholasbishop/sate; Q5 p3; survivor; last 2024-03.
- **xf**: https://github.com/sigoden/xf; Q5 p3; survivor (minimal); last 2022-03; terms: file-aware dynamic command
- runner.
- **nauman**: https://github.com/EgorDm/nauman; Q5 p3; survivor; last 2021-12; terms: CI-inspired local job automation.
- **faster (faster-build)**: https://github.com/happenslol/faster; Q5 p3; survivor; last 2022-05.
- **cb2**: https://crates.io/crates/cb2; Q5 p3; survivor; last 2019-01.
- **cargo-metask**: https://github.com/kanarus/cargo-metask; Q5 p3; survivor.
- **jarvis**: https://github.com/moseschmiedel/jarvis; Q5 p3; survivor; last 2020-07.
- **hoi**: https://github.com/kevinquillen/hoi; Q5 p3; survivor.
- **tasksitter**: https://github.com/lionkor/tasksitter; Q5 p3; survivor (unverified).
- **cargo-task-wasm**: https://github.com/yoshuawuyts/cargo-task-wasm; Q5 p3; survivor; last 2024-09.
- **rushon**: https://crates.io/crates/rushon; Q5 p4; survivor (unverified; no repo); last 2024-06.
- **voluntary (rtask, Lua)**: https://github.com/icecafecup/rtask; Q5 p4; survivor; last 2022-08.
- **umm**: https://crates.io/crates/umm; Q5 p4; survivor (minimal); last 2021-08.
- **rumake**: https://github.com/letaron/rumake; Q5 p4; survivor; last 2019-07.
- **hulk family (breakingbad, brown_script, jest, spiderman, wiz, prince, impact)**: https://github.com/skillzaa/hulk;
- Q5 p4, p5; survivor (unverified; looks like a name-squat set); last 2021.
- **cmd-runner**: https://gitee.com/mathegg/siphan; Q5 p4; survivor (minimal); last 2021-01.
- **rxe**: https://github.com/loxygenK/rxe; Q5 p4; survivor; last 2022-05.
- **instruct**: https://github.com/manuel2258/instruct; Q5 p5; survivor; last 2022-05.
- **digtask (Dig)**: https://crates.io/crates/digtask; Q5 p5; survivor (unverified); last 2024-03; terms: YAML-defined
- OS-level task orchestrator.
- **jake**: https://github.com/AstraBert/jake; Q5 p5, Q6 p6; survivor.
- **bake-tool**: https://github.com/myferr/bake; Q5 p5; survivor; last 2025-07.
- **rune-rs**: https://github.com/safinsingh/rune; Q5 p5; survivor (unverified); last 2020-09.
- **fledge**: https://github.com/CorvidLabs/fledge; Q5 p6; survivor (unverified); terms: dev lifecycle CLI.
- **runtask**: https://crates.io/crates/runtask; Q5 p6; survivor (no repo).
- **trix**: https://crates.io/crates/trix; Q5 p8; survivor (unverified; no repo).
- **plzplz**: https://github.com/k88hudson/plzplz; Q5 p8; survivor.
- **tazk**: https://github.com/nehu3n/tazk; Q5 p8; survivor.
- **rnme (runme.rs)**: https://github.com/dgrijalva/runme; Q5 p8; survivor; terms: tasks are plain Rust.
- **xeq**: https://github.com/opmr0/xeq; Q5 p9; survivor; terms: TOML command sequences.
- **madoru**: https://codeberg.org/Fatcat560/madoru; Q5 p9; survivor; terms: Markdown task runner.
- **only**: https://github.com/KercyDing/only; Q5 p9; survivor; terms: deterministic, staged language pipeline.
- **mq-task**: https://github.com/harehare/mq; Q5 p9; survivor; terms: Markdown task runner.
- **oxdock**: https://github.com/jzombie/rust-oxdock; Q5 p9; survivor (unverified); terms: Dockerfile-inspired build
- DSL.
- **mmz**: https://github.com/mlavrinenko/mmz; Q5 p10; survivor (minimal); terms: memoized command runner.
- **machfile (with machfile-cli)**: https://github.com/machfile/machfile; Q5 p10, p12; survivor.
- **rhask**: https://github.com/nakkiy/rhask; Q5 p11; survivor; terms: Rhai.
- **yarli**: https://github.com/rahulrajaram/yarli; Q5 p11; survivor (unverified); terms: scheduler, store, API, TUI.
- **mdtask (with mdtask-core)**: https://github.com/jhheider/mdtask; Q5 p11, p12; survivor; terms: Markdown tasks,
- embeddable.
- **jao**: https://github.com/RoyPrinsGH/jao; Q5 p12; survivor (unverified); terms: workspace scripts.
- **grim (grimoire)**: https://github.com/Vaishnav-Sabari-Girish/grimoire; Q5 p13; survivor (unverified).
- **meriadoc**: https://github.com/segunmo/meriadoc; Q5 p13; survivor (unverified).
- **rusk-task**: https://github.com/gw31415/rusk-task; Q5 p14; survivor.
- **devrunner**: https://github.com/NyxTools/DevRunner; Q5 p15; survivor (unverified).
- **besaz**: https://github.com/ManiProjs/besaz; Q5 p15; survivor.
- **jobfile**: https://github.com/darwincereska/jobfile; Q5 p15; survivor (unverified).
- **otto (otto-cli)**: https://github.com/mcmanussliam/otto; Q5 p15; survivor; terms: retries, timeouts, history,
- notifications.
- **cargo-x-do**: https://github.com/j-Cis/cargo-x-do; Q5 p15; survivor (unverified).
- **zua**: https://github.com/abab-bk/zua; Q5 p15; survivor; terms: Rhai.
- **alba**: https://github.com/albahq/alba; Q5 p16; survivor; terms: Beamfile, parallel beams, caching, watch mode,
- interactive interface.
- **flux-core**: https://github.com/yunusgungor/flux; Q5 p16; survivor; terms: dependency management, parallel
- execution, watch mode.
- **shifu**: https://gitlab.com/implabinash/shifu; Q5 p16; survivor (unverified).
- **sentinel-rs**: https://github.com/ayushbindlish/sentinel-rs; Q5 p16; survivor (unverified); terms: out-of-band
- notifications.
- **rtask (wensheng/rusk)**: https://github.com/wensheng/rusk; Q5 p16; survivor; terms: YAML.
- **patmat (drevo-patmat)**: https://github.com/ElectricPulse/patmat; Q5 p17; survivor (unverified); terms:
- Makefile-like GUI build tool.
- **forger-runner**: https://gitlab.com/mlejeune/forge; Q5 p17; survivor.
- **phenotype-forge**: https://github.com/KooshaPari/forge; Q5 p17; survivor (unverified).
- **steward**: https://github.com/alexfedoseev/steward; Q5 p2; survivor; terms: task runner and process manager.
- **emergent-engine**: https://github.com/govcraft/emergent; Q5 p12; survivor (unverified); terms: event-driven,
- pub-sub CLI pipelines.

### Components: persistent watch, process or inspection surface

- **Watchman**: https://github.com/facebook/watchman (active, MIT); Q8; component; survivor; terms: file watching
- daemon, triggers, query API, fsmonitor.
- **dev-process-manager**: https://dev-process-manager.com; Q1 p0, Q3 p0; component; survivor (unverified API); terms:
- process manager, orchestration.
- **braid (@aip-tech/braid)**: https://github.com/aip-tech/braid; Q1 p1; component; survivor; terms: background daemon,
- PID tracking, watch-triggered restarts, persistent rotated logs, process supervisor.
- **baton-run (Baton)**: https://kanumuri9593.github.io/Baton/; Q1 p1; component; survivor; terms: local run control
- plane, launch, hot-reload, inspect, capture evidence, MCP, HUD.
- **lerna-watch**: https://github.com/mattstyles/lerna-watch; Q1 p1; component/manager; survivor; last 2020-06; terms:
- watch dependency tree.
- **@fervon/launchpad**: https://fervon.dev/launchpad/; Q1 p2; component; survivor; terms: local dashboard, launch and
- monitor, live logs, binds 127.0.0.1.
- **devtooie**: https://github.com/rhyek/devtooie; Q1 p2; component/manager; survivor; terms: dependency-aware dev
- processes, TUI.
- **@vonage/monowatch**: https://github.com/Vonage/monowatch; Q1 p5; component; survivor; last 2021-05; terms: watcher,
- monitor, script runner.
- **@18ways/monorunner**: https://github.com/ways-labs/18ways; Q1 p7; component; survivor; terms: run and monitor local
- services.
- **runny (@polymech/runny; repackage @icydotdev/runny excluded)**: https://github.com/polymech-info/runny; Q1 p8;
- component/manager; survivor; terms: local GUI, dashboard, soft-ci.
- **@codepadding/coder**: https://www.npmjs.com/package/@codepadding/coder; Q1 p8; component; survivor (unverified; no
- repo); terms: browser dashboard, tmux.
- **@funeste38/rome**: https://github.com/jEFFLEZ/rome; Q1 p9; component/manager; survivor (unverified); terms:
- detached process management.
- **devmux (@chriscode/devmux)**: https://github.com/hassoncs/devmux; Q1 p9; component; survivor; terms: tmux service
- management, human-agent shared awareness.
- **port-daddy**: https://github.com/curiositech/port-daddy; Q1 p9; component; survivor; terms: daemon, pub/sub
- messaging, distributed locks, agent registry, service orchestration.
- **veloctl**: https://github.com/MahmoudGhoraba/veloctl; Q1 p10; component/manager; survivor (unverified); terms:
- dev-environment orchestrator, process manager.
- **@khijo/devportal**: https://github.com/Khijo/DevPortal; Q1 p11; component; survivor (unverified); terms: dashboard,
- process manager.
- **@gbdx/devis**: https://www.npmjs.com/package/@gbdx/devis; Q1 p11; component; survivor (no repo); terms: local
- development dashboard, workspace scanning, process management (pm2), HTTPS proxy.
- **grove (@theagileengineer/grove)**: https://www.npmjs.com/package/@theagileengineer/grove; Q1 p11; component;
- survivor (no repo); terms: dashboard to start, stop and monitor dev servers.
- **bun-wtui**: https://github.com/Zerodayu/bun-wtui; Q1 p12; component; survivor (unverified); terms: TUI dashboard.
- **helix (@stellix-agency/helix)**: https://github.com/Stellix-Agency/Helix; Q1 p12; component; survivor (unverified);
- terms: process monitoring TUI, restart on the fly.
- **monoboard**: https://www.npmjs.com/package/monoboard; Q1 p12; component; survivor (no repo); terms: local dashboard
- for running and observing packages.
- **monotui (@factorim/monotui)**: https://github.com/factorim/monotui; Q1 p13; component; survivor (unverified);
- terms: workspace TUI dashboard.
- **stackwake**: https://github.com/tofu-dev0123/Stackwake; Q1 p13; component; survivor (unverified); terms: start,
- sync and inspect repositories, Procfile, process manager.
- **Turbo Stoplight (@turbo-stoplight/server, vite-plugin)**: https://github.com/alexandreh92/turbo-stoplight; Q1 p13,
- p15; component (Turborepo companion server); survivor; last 2025-03; terms: persistent Turborepo pipelines,
- notifications from builds.
- **pnpm-dash**: https://github.com/artygus/pnpm-dash; Q1 p14; component/manager; survivor; terms: TUI dashboard, run
- scripts across packages.
- **clier-ai (clier)**: https://github.com/somersstack/clier; Q3 p0; component/manager; survivor (unverified); terms:
- process orchestration, event-driven pipeline management.
- **devsurface**: https://github.com/mrfandu1/devsurface; Q3 p0; component; survivor (unverified); terms: developer
- dashboard, scripts, ports, logs, repo health.
- **asciyml (@irautox/asciyml)**: https://github.com/IrAutoX/asciyml; Q3 p1; component; survivor (unverified); terms:
- daemon, watcher, scheduler.
- **BindPort (bindport, bindport-runner, bindport-dashboard, core, registry, adapters)**:
- https://github.com/bindport/bindport; Q5 p8, p9, Q6 p8; component; survivor (unverified); terms: local port registry,
- runner, local dashboard server.
- **bizi (bizi-server)**: https://getbizi.dev (github.com/ieedan/bizi); Q5 p10; component (task runner server);
- survivor; terms: task runner server.
- **oxproc**: https://github.com/fcoury/oxproc; Q5 p11; component; survivor (unverified); last 2025-10; terms: Procfile
- process manager, daemon mode, log following.
- **harness-canopy**: https://github.com/UniverLab/harness-canopy; Q5 p15; component; survivor (unverified;
- agent-oriented); terms: MCP server, task scheduling, file watching.
- **park (park-cli)**: https://github.com/Natoandro/park; Q5 p16; component; survivor (unverified); terms:
- project-scoped background process manager.

### Excluded near-misses (grouped by reason)

Add-ons of another manager (recorded above under the host tool), plus:

- Bazel: `@paretools/bazel` (Q2 p0), `cargo-raze` (Q6 p1), `bazeld` (Q6 p4), `target-determinator` (Q7), Gazelle and
- `bazel-travis` (Q8).
- Buck2: `buck-reindeer` (Q6 p5).
- Gradle: `gradleup` (Q3 p1).
- Make/Just: `@paretools/make` (Q3 p0).
- Vite+: `vite-plugin-makefile` family (Q3 p1), `config-vp` (Q1 p10), `@lcabrera/vite-config`,
- `@stealthscale/vite-config`.
- moon: `callisto-moon` (Q5 p13), `warpgate` (Q6 p1).
- Lerna: `lerna-ci` (Q1 p8), `@tamara027/lerna-terminal` (Q1 p13), `lerda` (Q1 p16), `cross-lerna` (Q1 p14).
- pueue: `bzb`, `bzb-core` (Q5 p14, p16); taxonomy pueue.
- Neovim overseer: `overseer-nvim-mcp` (Q3 p0).

Delegating command routers and pickers (no own task graph):

- `runner-run` and `@runner-run/*` binaries (Q1 p1, Q3 p0, Q5 p7, Q6 p8), `@simon_he/pi` (Q1 p2),
- `task-runner-detector` (Q1 p12, Q3 p0, Q5 p14), `dev-cli-universal` (Q1 p12), `@di-rs/rollercoaster`, `@aleyan/dela`
- / `dela` (Q3 p0, Q5 p3), `@happytoolin/alur` / `alur`, `@aymericbeaumet/run` / `run-cli` (Q3 p1, Q5 p2), `rt-cli` (Q5
- p11), `ck-cracker` (Q5 p4), `task-keeper` (Q5 p1), `karo-task-runner` (Q5 p18), `repo-runner` (Q1 p15), `@rodbe/nsl`,
- `srn`, `@lionad/dv`, `monorepo-ez-script`, `rundo`, `mono-exec`, `turbo-run`, `@tigerbook/run-scripts`, `monorun`
- (homoky), `@x-9lab/launch`, `lazy-typer`, `@glincker/palrun`, `spaceman`, `@jcamp/rig`, `devrun`, `runtui`,
- `smart-run`, `fzf-make`, `maki-cli`, `@wingring/monorepo`.

Launchers, terminal multiplexers and supervisors without a stated inspection surface:

- `@pinkynrg/crew`, `@gachlab/devup`, `herdy` (Q1 p1), `@cbhasib/devpilot`, `@busyburger/p3k` (Q1 p7),
- `@voudo/devpanes` / `devpanes` (Q1 p8, p16), `@remcostoeten/dev-menu` (Q1 p9), `rift-dev`, `@mrdiggles2/mux` (Q1
- p10), `dawnfall` (Q1 p11), `tcomposer` (Q1 p14), `muxa` (mprocs wrapper), `proctide` (Procfile; kw foreman,
- overmind), `devmux-cli` (Q1 p15), `orchd`, `node-project-scanner`, `diavola`, `npm-script-runner`,
- `@kwiruu/taki-cli`, `scriptlane`, `tuirunner`, `supi-cli`, `shell-compose`, `pend`, `jocker` (Q6 p4), `rustywatch`,
- `funzzy`, `vibewatch` (file watchers without API), `penguin` (dev server), `gity-cli` (git fsmonitor daemon), `mtsc`,
- `@lhechenberger/tsup-watch-monorepo`, `@mareklesko/watch-ng-libraries` (tool-specific watch).

Workflow schedulers, cron, job queues, CI and deploy:

- `@dagucloud/dagu` (workflow orchestration engine with UI), `runwisp` (cron plus process supervisor web UI),
- `taskherder`, `pitufo`, `@rongyan/cron-task`, `@krissself/quick-quick-run`, `kwon`, `cron-core`, `@trap_stevo/cynq`,
- `@ticatec/omniflow`, `monofo`, `buildpipe`, `rivendell`, `rustyochestrator`, `forge-runner`, `sb-forge`, `roxid`,
- GitLab CI, Codefresh, Semaphore CI, BuildBuddy, EngFlow, Nx Cloud (services), job-queue libraries
- (`@furystack/task-runner`, `headgate`, `rustvello-cli`, `stoker-engine`, `yarn` crate, `job`, `cloacina`,
- `autumn-harvest`).

Libraries (in-process orchestration, DAG, affected detection):

- `composer` (doowb), `flowed`, `alvamind-workflow`, `@calmo/task-runner`, `taskx`, `job-planner`, `orqis`,
- `dag-workflow-engine`, `@ruvector/rudag`, `@billdaddy/dagkit`, `slot-graph`, `dag-runner`, `contrepoint`,
- `determinator` (guppy), `is_affected`, `@front-ops/domino`, `@traf/*`, `ws-changed`, `lockfile-affected`,
- `@typescript-tools/packages-to-rebuild-on-changes`, `@grunnverk/tree-core`, `monorepo-cache-manager`, `mbx` and
- `mbx-cache-*` (jdx build cache primitives), `scopetest-cli`, `test-impact`, `testpick`, `Tainted`, `go-diff`.

Toolchain-, language- or org-specific:

- `hound-tauri-build`, `esburu`, `tsc-mono`, `tsrf`, `helm-mono`, `moqo`, `jbuild`, `ym`, `ctplt`, `Spago`,
- `Symplify/MonorepoBuilder`, `vix-tasks`, `@zssz-soft/zs` kept as survivor with org-specific note, `storm-workspace`,
- `pleme`, `@ebowwa/tooling`, `@bro-code/cli`, `@cosyte/process`, `@wp-operations/wp-ops`.

Insufficient description (flag for a later look if a term expansion surfaces them again):

- `@technance/windrun`, `@tscmono/cli`, `nested-workspace-helper`, `monorepotime`, `monowalk`, `gitmonorepo`,
- `@dunes/mono`, `monodic`, `bunn`, `@monocli/cli`, `@zaeper/pkgm`, `monop`, `tsmono`, `@digicroz/dev-kit`,
- `codiaum-task-runner`, `xtomate`, `don`, `wsb`, `metactl`, `typescript_tools`, `monorepo` crate,
- `js_workspace_tools`, `morp`.

Multi-repo and VCS tooling (Q8 Git tools and similar):

- meta, `metarepo`, `repolith`, `@dashkite/tempo`, `binder-cli`, `gitgrip`, `garden` kept as survivor, GVFS, SlothFS,
- josh, git subtree/subsplit, splitsh-lite, mgt, `scorpiofs` (FUSE overlay for monorepo builds), FBShipIt,
- adeira/shipit, tomono, shopsys/monorepo-tools, fastlane monorepo tools.

Everything else in the result pages was clearly off-category and is not enumerated individually: templates and
scaffolders (`create-*`), shared configs, linters and boundary checkers, release, changelog and versioning tools
(changesets family, `monochange`, `semifold`, `cocogitto`, `versio`), env and secret managers, AI agent harnesses and
MCP code-intelligence servers, component libraries, Hardhat forks, rCore tutorial crates and the 2024 spam scopes.

## New taxonomy terms

- affected detection / true affected / semantic change detection
- build graph / task graph / DAG-based task runner
- incremental build / differential build / hash-based incremental build
- local computation caching / tiered caching / memoized command runner
- remote cache / remote caching / distributed computation caching
- content-addressed store / action cache / Bazel Remote Execution API
- remote execution / transparent remote execution / distributed task execution
- distributed command runner / jobs across machines
- hermetic execution / hermetic builds / sandboxed builds
- task splitting / deflaking
- workspace analysis / project graph visualization / workspace topology
- project constraints and visibility / module boundaries
- polyglot monorepo orchestration / cross-language build coordination
- phase execution / pipelines / named flows
- cache-correctness client / fingerprint client
- cache miss explanation
- CI matrix / dynamic pipeline generation
- build orchestrator / task orchestrator / turbo alternative / nx alternative
- Gradle-inspired task runner
- Taskfile / justfile / Makefile alternative / command runner
- Markdown task runner / markdown-defined tasks
- script runner with topological ordering / recursive run / workspaces foreach
- persistent tasks / persistent pipelines
- watch mode / watch-triggered restarts / file watcher daemon / fsmonitor
- process supervisor / process manager / Procfile / foreman / overmind
- background daemon / daemon mode / singleton daemon
- local run control plane / service orchestration / readiness probes / service supervision
- local development dashboard / TUI dashboard / HUD / live logs / log multiplexing
- terminal multiplexer / tmux orchestration / mprocs
- task runner server / JSON-RPC task API
- pub/sub messaging / agent registry / distributed locks
- MCP server for task execution / task execution via AI / agentic CI
- WASM task plugins / extensible task runner plugins
- task query language / language server for task config
- virtual filesystem for builds (FUSE overlay, GVFS)
- sparse checkout / target determinator
- pueue (command queue daemon)
- Dagger backend / Nix-based runner
- local CI / soft CI / CI-inspired local job automation
