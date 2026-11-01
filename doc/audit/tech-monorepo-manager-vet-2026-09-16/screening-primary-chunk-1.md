# Monorepo manager screening, chunk 1

Screened 2026-09-16.
Repository metadata comes from `gh api repos/<owner>/<repo>` and `gh api repos/<owner>/<repo>/releases?per_page=100`
(newest by `published_at`; "stable" excludes prerelease, canary, rc, milestone tags).
Tag-only projects use registry publish dates (npm registry JSON, pub.dev API) where noted.
Docs were read through `llms.txt` indexes, raw `.md` page variants, rendered HTML sidebars, and `gh api` repo contents.
No candidate tool was installed or executed.

Interpretation notes applied to every candidate:

- G1 "component" requires both a persistent watch process (watches for changes, or supervises long-running watch-style
- dev processes) and an RPC, IPC, or HTTP interface.
- G3 "official docs" excludes blog posts, changelogs, issues, source files, and chat.
  A one-line tool list entry without behavior (what is observed, how it is discovered, limits) does not count as
  behavior-level docs.
- Once a gate fails, later gates are not evaluated; G4 lists only evidence already gathered, marked "incidental".
- G3 confusion rule (user rule change received mid-screening, applied retroactively): the first confusion or
- frustration while reading a candidate's docs (unclear feature location, broken links, truncated or missing summaries,
- contradictory text, needing issues/source/chat to understand consumed behavior, pages a human cannot load) is an
- immediate G3 exit, recorded as "G3 confusion" with the trigger URL.
  Candidates that had already exited at G3 coverage keep that exit and list their confusion triggers too.
- G3 no-JS rule (second user rule change, replaces the earlier "recheck once in a real browser" exception, applied
- retroactively): every docs page relied on for a pass is fetched once with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0' <url>`.
  A bot challenge, captcha page, 403/429/202 block, or HTML without the documentation text (JS app shell) is an
  immediate G3 exit, recorded as "G3 no-JS" with URL, HTTP status, and excerpt.
  Block markers are detected in the HTML outside `<script>` elements; visible text is the HTML with scripts, styles,
  and tags removed.

### sgaunet/runq

- URL: https://github.com/sgaunet/runq
- License: MIT (`LICENSE` at repo root)
- Latest release: v0.3.1, 2026-09-03
- Archived: no
- G1: fail, category mismatch.
  - README: "a small, single-binary Go CLI for running many shell or argv commands in parallel ... and an implicit
  - local Unix socket that lets a second `runq` invocation forward extra commands into the running instance"
  - (https://github.com/sgaunet/runq#readme).
  - `runq serve` is a persistent listener with Unix-socket IPC for submitting commands and `runq stop` for shutdown,
  - but it watches nothing: no watch feature in README, no watcher dependency in `go.mod` (only `bullets`, `cobra`,
  - `pflag`, `x/sys`, `x/term`), and no watch-related file in the repo tree.
  - It also has no project or workspace model, so it is not a monorepo manager either.
  - Borderline note: if a persistent command queue with IPC submission (no watching, no status query) is acceptable as
  - a component, re-screen from G2.
- G2: not evaluated.
- G3: not evaluated.
  Incidental: the README "Quickstart" links to `specs/001-parallel-cmd-runner/quickstart.md`, which does not exist in
  the repo tree (broken link).
- G4 (incidental):
  - d: no watch mode.
  - e: no documented status or inspection over the socket; only command forwarding (README "Serve (persistent
  - listener)").
  - f: submit commands (`runq '<cmd>'` forwards into `runq serve`) and graceful stop (`runq stop`) are documented in
  - the README; no rerun or per-command cancel.
  - g: none.
- Final: exit at G1 (not a watch process and not a monorepo manager).

### standardbeagle/brummer

- URL: https://github.com/standardbeagle/brummer
- License: no license file in the repo (GitHub license detection returns none; root listing has no
- `LICENSE`/`COPYING`); README ends with "License: MIT" and `package.json` declares `"license": "MIT"`.
- Latest release: v0.1.0, 2025-06-13 (last commit 2026-08-01, docs URL change only)
- Archived: yes
- G1: pass as component.
  - README: "A TUI (Terminal User Interface) for managing npm/yarn/pnpm/bun scripts with integrated MCP server for
  - external tool access" and "The MCP server runs on port 7777 by default and implements the official MCP Streamable
  - HTTP transport protocol" with tools `scripts_list`, `scripts_run`, `scripts_stop`, `scripts_status`, `logs_stream`
  - (https://github.com/standardbeagle/brummer#readme).
  - It is a persistent supervisor of long-running dev scripts with an HTTP JSON-RPC interface; `go.mod` includes
  - `fsnotify`.
- G2: fail.
  - The requirement is a license file in the repo; only README prose and the npm manifest field state MIT.
- G3: not evaluated.
  Incidental: the README's docs site https://dev.standardbeagle.com/brummer/ returned HTTP 404 on 2026-09-16; docs
  sources exist under `docs-site/docs/` and a separate, overlapping `docs/` tree of implementation plans and test notes.
- G4 (incidental):
  - d: supervises script processes; no documented re-run-on-change found in README.
  - e: MCP HTTP tools `scripts_status`, `logs_stream`, `logs_search` (README "MCP Server Integration").
  - f: MCP HTTP tools `scripts_run`, `scripts_stop`; restart is a TUI key only (README "Process Management").
  - g: none found.
- Final: exit at G2 (no license file; project also archived).

### lerna/lerna

- URL: https://github.com/lerna/lerna
- License: MIT
- Latest release: v10.0.1, 2026-08-19
- Archived: no
- G1: pass, monorepo manager.
  - https://lerna.js.org/docs/lerna-and-nx: "Lerna uses Nx to detect packages in the workspace and dependencies between
  - them. Lerna defers to Nx's powerful task runner to run scripts ... all while ensuring that dependencies between
  - packages are respected."
- G2: pass. MIT `LICENSE` at repo root. Nx Cloud (remote cache, distribution) is a hosted service outside the local
- tool.
- G3: fail, coverage (c).
  - a, configuration reference: present, https://lerna.js.org/docs/api-reference/configuration (`lerna.json` plus the
  - `nx.json` subset Lerna uses).
  - b, CLI reference: present, https://lerna.js.org/docs/api-reference/commands (per-command option details link out to
  - GitHub READMEs such as https://github.com/lerna/lerna/tree/main/packages/lerna/src/commands/watch#lerna-watch).
  - c, extension or plugin mechanism: absent.
    The sidebar (Introduction, Getting Started, Lerna and Nx, Lerna and AI, Features, Concepts, Recipes, API Reference,
    version matrix, Legacy Package Management, FAQ, Lerna 6 obsolete options, Troubleshooting) has no plugin or
    extension page; text search of https://lerna.js.org/docs/lerna-and-nx for "plugin" or "extend" returns no hits, the
    configuration reference has none, and a `site:lerna.js.org plugin` web search surfaces only the Nx Console
    editor-integration page.
  - d, watch mode (claimed): documented at https://lerna.js.org/docs/features/workspace-watching, with options deferred
  - to the GitHub README.
  - e, f: not claimed.
  - Structure signals:
    - Command behavior is split between the docs site command index and per-command READMEs in the repo, with the site
    - page acting as a link hub (https://lerna.js.org/docs/api-reference/commands).
    - Top-level sidebar mixes guides, a compatibility matrix, legacy management, and obsolete-option lists as flat
    - siblings (https://lerna.js.org/docs/introduction).
- G4:
  - d: `lerna watch -- <command>` runs a command on package file changes with
  - `$LERNA_PACKAGE_NAME`/`$LERNA_FILE_CHANGES` (https://lerna.js.org/docs/features/workspace-watching).
  - e: none documented.
  - f: none documented.
  - g: `lerna create` is listed in the command index (behavior only in its GitHub README); no sync or enforce mechanism
  - documented.
- Final: exit at G3 coverage (no extension or plugin mechanism documented).
  Also a G3 confusion trigger under the retroactive rule: https://lerna.js.org/docs/api-reference/commands sends each
  command's options out to per-command GitHub READMEs, so the site alone does not show where command behavior lives.

### vercel/turborepo

- URL: https://github.com/vercel/turborepo
- License: MIT
- Latest release: v2.10.14-canary.4, 2026-09-16; newest stable v2.10.13, 2026-09-14
- Archived: no
- G1: pass, monorepo manager.
  - https://turborepo.dev/llms.txt: "Run tasks across a monorepo with caching and parallelism";
  - https://turborepo.dev/docs/core-concepts/package-and-task-graph: "Turborepo builds a Task Graph based on your
  - configuration and repository structure."
- G2: pass. MIT `LICENSE` at repo root. Vercel Remote Cache is hosted; a self-hosted cache API is specified
- (https://turborepo.dev/docs/openapi).
- G3: fail, coverage (c).
  - a: present, https://turborepo.dev/docs/reference/configuration and
  - https://turborepo.dev/docs/reference/package-configurations.
  - b: present, https://turborepo.dev/docs/reference/run and https://turborepo.dev/docs/reference/options-overview.
  - c: absent.
    The full docs index https://turborepo.dev/llms.txt lists no plugin or extension page, and a text search of the full
    corpus https://turborepo.dev/llms-full.txt for "plugin" returns only ESLint/Buildkite/Nx-migration mentions and the
    Nx comparison ("Nx uses layers of plugins ...").
    The only third-party surfaces are generators (`turbo gen`, code scaffolding) and the Remote Cache OpenAPI spec (a
    cache backend protocol); neither lets third parties add task-running behavior.
  - d, watch mode (claimed): documented, https://turborepo.dev/docs/reference/watch.
  - e, f: not claimed.
    https://turborepo.dev/docs/reference/configuration (`daemon`): "The daemon is still used by `turbo watch` and the
    Turborepo LSP", with no documented client interface.
  - Structure signals:
    - The configuration reference `daemon` entry says it is deprecated for `turbo run` but still used by `turbo watch`,
    - while the watch reference page never mentions the daemon (https://turborepo.dev/docs/reference/configuration,
    - https://turborepo.dev/docs/reference/watch).
    - The API reference sidebar lists configuration pages, CLI commands, and separate npm packages
    - (`eslint-config-turbo`, `@turbo/codemod`, `@turbo/gen`) as flat siblings (https://turborepo.dev/docs/reference).
- G4:
  - d: yes, `turbo watch` is "dependency-aware" and re-runs tasks on change; `interruptible: true` restarts persistent
  - tasks (https://turborepo.dev/docs/reference/watch).
  - e: none documented. `turbo devtools` serves a package-graph visualization
  - (https://turborepo.dev/docs/reference/devtools) and `turbo query` runs GraphQL against the repo
  - (https://turborepo.dev/docs/reference/query); neither inspects a running watch.
  - f: none documented.
  - g: generators, https://turborepo.dev/docs/guides/generating-code and https://turborepo.dev/docs/reference/generate;
  - experimental `turbo boundaries` "checks for workspace dependency violations" and sends feedback to a GitHub RFC
  - discussion (https://turborepo.dev/docs/reference/boundaries).
- Final: exit at G3 coverage (no extension or plugin mechanism documented).
  Also a G3 confusion trigger under the retroactive rule: https://turborepo.dev/docs/reference/configuration (`daemon`)
  says the daemon still backs `turbo watch`, while https://turborepo.dev/docs/reference/watch never mentions it.

### nrwl/nx

- URL: https://github.com/nrwl/nx
- License: MIT
- Latest release: 22.7.12, 2026-09-10 (docs version picker shows v23)
- Archived: no
- G1: pass, monorepo manager.
  - https://nx.dev/llms.txt: "Use Nx when a repository holds more than one buildable or testable project and you need
  - to know what depends on what, run only the affected tasks, or reuse cached results."
- G2: pass for the local tool. MIT `LICENSE` at repo root.
  Proprietary or hosted parts: Nx Cloud (remote cache, Nx Agents, self-healing CI) and Enterprise features
  (conformance, owners) need an Nx Cloud plan or license (https://nx.dev/docs/enterprise/activate-license).
- G3: fail, coverage (e). Borderline: only gap found.
  - a: present, https://nx.dev/docs/reference/nx-json and https://nx.dev/docs/reference/project-configuration.
  - b: present, https://nx.dev/docs/reference/nx-commands.
  - c: present, https://nx.dev/docs/concepts/nx-plugins, https://nx.dev/docs/extending-nx,
  - https://nx.dev/docs/reference/devkit.
  - d, watch mode (claimed): present, https://nx.dev/docs/kb/workspace-watching (callback env vars, batching behavior).
  - e, inspecting running processes (claimed): lacks behavior-level docs.
    https://nx.dev/docs/features/enhance-ai says the MCP server "Provides connectivity to ... running processes";
    https://nx.dev/docs/reference/nx-mcp lists `nx_current_running_tasks_details` ("Lists currently running Nx TUI
    processes and their task statuses") and `nx_current_running_task_output` as one-line table rows.
    No official page explains how running processes are discovered, which runs are visible (TUI disabled, `nx watch`,
    daemon), or what the returned data contains.
    The nx-console repo README (`apps/nx-mcp/README.md`) repeats the same one-liners, and
    `libs/shared/running-tasks/README.md` is generator boilerplate.
    The only longer treatment is the blog post "Real-time Terminal Integration" linked from the docs, which does not
    count.
  - f: not claimed (TUI control is keyboard-only, https://nx.dev/docs/kb/terminal-ui).
  - Structure signals:
    - Watch mode is reachable only through Knowledge Base, "Tasks & caching" topic
    - (https://nx.dev/docs/kb/workspace-watching), not from the main sidebar Features list; the daemon page, which
    - describes workspace file watching, is in Reference (https://nx.dev/docs/reference/nx-daemon) and does not mention
    - `nx watch`, and no page ties them together.
    - Knowledge Base index lists unrelated kinds of pages as flat siblings: CI pricing, TypeScript 7 migration,
    - framework hubs, Nx Cloud tokens, and "Nx vs Blacksmith" (https://nx.dev/docs/kb).
    - "Enforce module boundaries" exists at https://nx.dev/docs/features/enforce-module-boundaries,
    - https://nx.dev/docs/platform-features/code-organization/enforce-module-boundaries,
    - https://nx.dev/docs/guides/enforce-module-boundaries, and https://nx.dev/docs/kb/enforce-module-boundaries.
    - "Extending Nx" exists at both https://nx.dev/docs/extending-nx and https://nx.dev/docs/kb/extending-nx, with
    - plugin authoring pages spread across KB (https://nx.dev/docs/kb/organization-specific-plugin,
    - https://nx.dev/docs/kb/publish-plugin, https://nx.dev/docs/kb/create-sync-generator) and Reference
    - (https://nx.dev/docs/reference/devkit).
    - The agent index https://nx.dev/llms.txt gives count-only summaries for Technologies, Reference, and Knowledge
    - Base ("227 pages, indexed separately").
- G4:
  - d: yes, `nx watch` (https://nx.dev/docs/kb/workspace-watching); continuous tasks
  - (https://nx.dev/docs/reference/project-configuration).
  - e: MCP tools list running TUI task status and output (https://nx.dev/docs/reference/nx-mcp), behavior undocumented;
  - daemon logs via `nx daemon` (https://nx.dev/docs/reference/nx-daemon).
    The daemon's Unix socket is documented as internal: "The socket is a remote for code execution".
  - f: none documented.
  - g: yes, generators and sync generators that update files before tasks run
  - (https://nx.dev/docs/features/generate-code, https://nx.dev/docs/concepts/sync-generators).
- Final: exit at G3, coverage and confusion.
  Coverage: natively claimed running-process inspection has only one-line tool listings.
  Confusion (retroactive rule): watch mode is not reachable from the main sidebar and sits only in the Knowledge Base
  (https://nx.dev/docs/kb/workspace-watching), and "Enforce module boundaries" has four locations with no single
  canonical page.
  With the confusion rule applied, this exit no longer depends on the borderline coverage reading.

### bazelbuild/bazel

- URL: https://github.com/bazelbuild/bazel
- License: Apache-2.0
- Latest release: 9.3.0rc1, 2026-09-15; newest stable by publish date 8.8.0, 2026-08-31
- Archived: no
- G1: pass, monorepo manager.
  - https://bazel.build/concepts/build-ref ("Repositories, workspaces, packages, and targets") and
  - https://bazel.build/concepts/dependencies define a workspace of packages and targets whose build, test, and run
  - commands follow target dependencies.
- G2: pass. Apache-2.0 `LICENSE` at repo root.
- G3: fail, confusion (coverage alone would pass).
  - Trigger, confirmed in the no-JS curl response (HTTP 200, 311566 bytes, doc text present): the User guide sidebar on
  - https://bazel.build/remote/bep lists "Configurable Build Attributes" twice (excerpts "Calling Bazel from scripts
  - Advanced Configurable Build Attributes Integrating with C++ Rules" and "C++ Toolchain Configuration Configurable
  - Build Attributes bazel mobile-install") (`/configure/attributes` and `/docs/configurable-attributes`), and files
  - the Build Event Protocol, a local build-event stream, under "Remote Execution" next to remote caching pages, so it
  - is unclear where build observation is documented.
  - Second trigger: https://bazel.build/remote/bep defines the event format by linking to `build_event_stream.proto` in
  - GitHub source rather than a docs page.
  - Coverage evidence gathered before the rule change:
  - a: https://bazel.build/run/bazelrc (bazelrc), https://bazel.build/reference/be/overview (BUILD encyclopedia),
  - https://bazel.build/rules/lib/globals/module (`MODULE.bazel`).
  - b: https://bazel.build/reference/command-line-reference and https://bazel.build/run/build.
  - c: https://bazel.build/extending/concepts, https://bazel.build/rules/rules-tutorial, Starlark API under
  - https://bazel.build/rules/lib/overview.
  - d: not claimed natively (the iterative watcher is the separate `bazel-watcher`/`ibazel` project).
  - e (claimed: the Build Event Protocol streams a running build's events to a file or a gRPC Build Event Service):
  - behavior documented at https://bazel.build/remote/bep, https://bazel.build/remote/bep-glossary,
  - https://bazel.build/remote/bep-examples.
    The event schema itself links to the `.proto` source.
  - f: not claimed (the client/server page documents a long-lived server and `shutdown`, no external control API,
  - https://bazel.build/run/client-server).
  - Structure signals:
    - The Reference nav has both a "Build encyclopedia" and a "Build Encyclopedia" group (agent docs index
    - https://bazel.build/_llms/head.md; not rechecked against the rendered sidebar).
    - The "Remote Execution" section holds local-only topics as flat siblings: Persistent Workers and Build Event
    - Protocol pages (https://bazel.build/remote/persistent, https://bazel.build/remote/bep).
    - "Recommended Rules" appears under both User guide Basics and Community Programs, and "Policy" under both
    - Contributing and Getting help (https://bazel.build/_llms/9-1.md).
    - Docs are versioned (HEAD plus `versions/9.1.0` and older); the versioned llms indexes list 119 pages vs 317 at
    - HEAD, though spot-checked pages such as https://bazel.build/versions/9.1.0/run/bazelrc still resolve.
- G4:
  - d: no native watch mode.
  - e: BEP / Build Event Service streams live build progress to another process (https://bazel.build/remote/bep).
  - f: none documented.
  - g: no native documented way to write generated files back into the source tree; genrule and rule outputs land in
  - the output tree (https://bazel.build/reference/be/general).
- Final: exit at G3 confusion (duplicate sidebar entries and BEP filed under Remote Execution,
- https://bazel.build/remote/bep).

### gradle/gradle

- URL: https://github.com/gradle/gradle
- License: Apache-2.0
- Latest release: v9.8.0-RC1, 2026-09-08; newest stable v9.7.1, 2026-08-19
- Archived: no
- G1: pass, monorepo manager.
  - https://docs.gradle.org/current/userguide/multi_project_builds.html and
  - https://docs.gradle.org/current/userguide/multi_project_builds_intermediate.html ("Structuring Multi-Project
  - Builds") model a build of many projects whose tasks run across projects through project dependencies.
- G2: pass. Apache-2.0 `LICENSE` at repo root. Build Scans (https://docs.gradle.org/current/userguide/inspect.html)
- publish to Develocity, a hosted or commercial service outside the local tool.
- G3: fail, confusion.
  - Trigger (no-JS curl of https://docs.gradle.org/current/userguide/continuous_builds.html, HTTP 200, doc text
  - present): the same sidebar carries two parallel plugin-authoring page sets, "Creating Plugins 1. Plugin
  - Introduction 2. Pre-Compiled Script Plugins 3. Binary Plugins 4. Developing Binary Plugins 5. Testing Binary
  - Plugins 6. Publishing Binary Plugins" (for example
  - https://docs.gradle.org/current/userguide/binary_plugin_advanced.html) and "Plugins Introduction to Plugins
  - Precompiled Script Plugins Convention Plugins Binary Plugins Testing Plugins Preparing to Publish Publishing
  - Plugins" (for example https://docs.gradle.org/current/userguide/implementing_gradle_plugins_binary.html), plus a
  - third plugin tutorial series (https://docs.gradle.org/current/userguide/part1_gradle_init_plugin.html).
    It is unclear which "Binary Plugins" page is the canonical extension documentation.
  - Coverage evidence gathered before the trigger (all static HTML, readable without JS): a
  - https://docs.gradle.org/current/userguide/build_environment.html and
  - https://docs.gradle.org/current/dsl/index.html; b
  - https://docs.gradle.org/current/userguide/command_line_interface.html; c the plugin pages named in the trigger; d
  - https://docs.gradle.org/current/userguide/continuous_builds.html; e and f
  - https://docs.gradle.org/current/userguide/tooling_api.html ("Listen to progress and testing events as a build
  - executes ... Cancel a build that is running"; "the Javadoc is the main documentation").
  - Additional structure signal: dependency declaration has three entry points in one sidebar,
  - https://docs.gradle.org/current/userguide/dependencies_intermediate.html,
  - https://docs.gradle.org/current/userguide/declaring_dependencies.html,
  - https://docs.gradle.org/current/userguide/declaring_dependencies_basics.html.
- G4 (incidental):
  - d: yes, continuous build `-t`/`--continuous` re-executes requested tasks when inputs change
  - (https://docs.gradle.org/current/userguide/continuous_builds.html).
  - e: Tooling API progress events for builds the client launches, through the daemon
  - (https://docs.gradle.org/current/userguide/tooling_api.html); no documented attach to a build started elsewhere.
  - f: Tooling API cancellation; continuous builds run through the Tooling API "can be cancelled using the Tooling
  - API's cancellation mechanism" (https://docs.gradle.org/current/userguide/continuous_builds.html).
  - g: Build Init plugin generates project skeletons
  - (https://docs.gradle.org/current/userguide/build_init_plugin.html); no dedicated sync or enforce mechanism.
- Final: exit at G3 confusion (duplicate plugin-authoring page sets in one sidebar).

### teambit/bit

- URL: https://github.com/teambit/bit
- License: Apache-2.0 (`LICENSE` at repo root is the Apache 2.0 notice with a "Bit - A development toolchain" header,
- which is why GitHub reports NOASSERTION)
- Latest release: v2.0.26, 2026-07-21
- Archived: no
- G1: pass, monorepo manager.
  - README (https://github.com/teambit/bit#readme): "Bit is the build system to connect components and apps from
  - development to CI"; repo CLI reference `scopes/harmony/cli-reference/cli-reference.mdx`: `bit build` "executes the
  - complete build pipeline including compilation, testing, linting, and other tasks defined by component
  - environments", across workspace components.
- G2: pass for the local tool (Apache-2.0).
  Hosted parts: bit.cloud scopes, Ripple CI, and the hosted Cloud MCP; the repo CLI reference calls the local
  `mcp-server` "the legacy local stdio MCP server" and recommends "the hosted Cloud MCP at https://mcp.bit.cloud/mcp".
  Self-hosted scope servers are documented (https://bit.dev/reference/reference/scope/running-a-scope-server).
- G3: fail, no-JS.
  - https://bit.dev/reference/cli-reference/: HTTP 200, 101091 HTML bytes, 2755 visible text characters, and the
  - documentation text is absent; excerpt "CLI Reference | Bit Docs Solutions Reference Teams Enterprise Blog Login Get
  - Started Contact Search ⌘ Ctrl K Docs Solutions By codebase Monorepo AI-native monorepo ..." (header and navigation
  - only; command entries such as "Create or reinitialize an empty workspace" are missing).
  - https://bit.dev/reference/workspace/workspace-json (configuration reference): HTTP 200, 2743 visible text
  - characters, only header and navigation; `workspace.jsonc` text absent.
  - https://bit.dev/reference/extending-bit/aspect-overview (extension docs): HTTP 200, 2758 visible text characters,
  - same app shell.
  - Earlier confusion evidence (gathered before the no-JS rule, kept for reference): the site CLI list, once rendered
  - by JS, does not match the repo's CLI reference `scopes/harmony/cli-reference/cli-reference.mdx` (the site lists
  - `server`, `use`, `mini-status`, `completion`; the repo file lists `mcp-server`, `ci`, `ripple`, `stash` instead),
  - and the repo file gives both `run` and `start` the alias `c`.
- G4 (incidental, from the repo CLI reference):
  - d: `bit watch` "watch and compile components on file changes"; `bit start` dev server "includes hot module
  - reloading".
  - e: `bit server`, listed on the rendered site as "EXPERIMENTAL. communicate with bit cli program via http requests",
  - is absent from the repo reference; no behavior docs.
  - f: none documented.
  - g: component generators and workspace starters (https://bit.dev/reference/generator/create-component-generator),
  - and `bit ws-config write` writes IDE config files.
- Final: exit at G3 no-JS (reference pages are JS app shells without documentation text).

### facebook/buck

- URL: https://github.com/facebook/buck
- License: Apache-2.0
- Latest release: v2022.05.05.01, published 2022-08-08
- Archived: yes
- G1: pass, monorepo manager.
  - https://buck.build/command/build.html (HTTP 200 without JS) documents building targets and their "transitive
  - dependencies" across a repo configured by `.buckconfig` (https://buck.build/files-and-dirs/buckconfig.html: "The
  - root of your project must contain a configuration file").
- G2: pass. Apache-2.0 `LICENSE` at repo root.
- G3: fail, confusion.
  - Trigger: https://buck.build (HTTP 200, doc text present) says "Use Buck2 This project is no longer actively
  - maintained. Please see https://buck2.build for the build system that replaces it. Old content continues below for
  - historical" purposes, and the README opens "This repo is dead."
    Every docs page is a frozen historical set for an abandoned tool.
- G4: not evaluated.
- Final: exit at G3 confusion (frozen docs for a dead project).

### microsoft/rushstack

- URL: https://github.com/microsoft/rushstack
- License: MIT (root `LICENSE`: "The projects in this monorepo are licensed under the MIT license"; `apps/rush/LICENSE`
- for `@microsoft/rush`)
- Latest release: no GitHub releases; npm `@microsoft/rush` 5.179.0 published 2026-09-05
- Archived: no
- G1: pass, monorepo manager.
  - https://rushjs.io/pages/advanced/watch_mode/: "The arrow from D to C indicates that D depends on C ; this means
  - that C must be built before D can be built", with project selection such as `rush build --to-except D`.
- G2: pass (MIT).
- G3: fail, confusion.
  - Trigger (no-JS curl, HTTP 200, doc text present): https://rushjs.io/pages/advanced/watch_mode/ defers the feature's
  - state to GitHub: "( PR #2298 aims to simplify this step for projects whose "build:watch" would be the same as
  - "build" ...)" and "The "watchForChanges" feature is still in its early stages. Feedback is welcome! GitHub issue
  - #1202 tracks additional work items".
    The section headed "The "watchForChanges" setting (experimental)" also documents the separate phased-command
    `watchOptions` mechanism, so it is unclear which watch mechanism the heading describes.
  - Coverage evidence seen before the trigger: a https://rushjs.io/pages/configs/rush_json/ and
  - https://rushjs.io/pages/configs/command-line_json/; b https://rushjs.io/pages/commands/rush_build/; c
  - https://rushjs.io/pages/extensibility/creating_plugins/ ("Creating Rush plugins (experimental)").
- G4 (incidental):
  - d: yes, `--watch` for phased commands with `watchOptions`, and bulk `watchForChanges`
  - (https://rushjs.io/pages/advanced/watch_mode/).
  - e, f: no watch inspection or control channel in the docs navigation; a Rush MCP server page exists
  - (https://rushjs.io/pages/ai/rush_mcp/) but was not evaluated.
  - g: not evaluated.
- Final: exit at G3 confusion (watch docs defer to GitHub PR and issue).

### sbt/sbt

- URL: https://github.com/sbt/sbt
- License: Apache-2.0
- Latest release: v2.0.9, 2026-09-14
- Archived: no
- G1: pass, monorepo manager.
  - https://www.scala-sbt.org/2.x/docs/en/guide/multi-project-basics.html:
  - `lazy val util = (project in file("util")) .dependsOn(core)` under "Subproject dependency".
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, confusion.
  - Trigger (no-JS curl, HTTP 200, doc text present):
  - https://www.scala-sbt.org/2.x/docs/en/concepts/command-basics.html says "sbt server is a service that accepts
  - commands from either the command line or a network API called Build Server Protocol", and
  - https://www.scala-sbt.org/2.x/docs/en/reference/sbt.html says "sbt server accepts commands from sbtn, network API,
  - or via its own sbt shell".
    The 2.x table of contents (https://www.scala-sbt.org/2.x/docs/en/toc.html) has no page for that network API; the
    only server protocol page is in the 1.x docs set, https://www.scala-sbt.org/1.x/docs/sbt-server.html, which says
    "The wire protocol we use is Language Server Protocol 3.0 (LSP), which in turn is based on JSON-RPC".
    The current docs name a different protocol than the only protocol page, which lives in the older version's docs.
  - Also noted: the 2.x docs sidebar is injected by JS; without JS, mdBook falls back to a `<noscript>` iframe of
  - `toc.html`, which still lists the pages.
- G4 (incidental):
  - d: yes, watch command (https://www.scala-sbt.org/2.x/docs/en/reference/watch.html).
  - e, f: claimed via sbt server and thin client; behavior only in 1.x docs, as recorded in the trigger.
  - g: `sbt new` templates (https://www.scala-sbt.org/2.x/docs/en/guide/sbt-new.html).
- Final: exit at G3 confusion (server protocol contradicts across doc versions; no current-version page).

### moonrepo/moon

- URL: https://github.com/moonrepo/moon
- License: MIT
- Latest release: v2.5.5, 2026-09-15
- Archived: no
- G1 to G4: not evaluated.
- Final: user exclusion (documentation), skipped as instructed.

### pantsbuild/pants

- URL: https://github.com/pantsbuild/pants
- License: Apache-2.0
- Latest release: release_2.33.1, 2026-08-27
- Archived: no
- G1: pass, monorepo manager.
  - https://www.pantsbuild.org/stable/docs/introduction/welcome-to-pants: "Pants is a fast, scalable, user-friendly
  - build and developer workflow system for codebases of all sizes";
  - https://www.pantsbuild.org/stable/docs/using-pants/advanced-target-selection: "Use --changed-dependents=direct or
  - --changed-dependents=transitive" to run over "any targets that depend on the changed files".
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, confusion.
  - Trigger (no-JS curl, HTTP 200, doc text present):
  - https://www.pantsbuild.org/stable/docs/writing-plugins/the-rules-api/tips-and-debugging says "Rule graph issues can
  - be particularly hard to figure out - the error messages are noisy and do not make clear how to fix the issue. We
  - plan to improve this. We encourage you to reach out in #plugins on Slack for help."
    Understanding a plugin failure mode is sent to chat instead of docs.
  - Structure signal (retroactive rule also makes it an exit): the "Common plugin tasks" section lists "Plugin upgrade
  - guide" as a flat sibling of "Add a linter", "Add codegen", "Run Tests"
  - (https://www.pantsbuild.org/stable/docs/writing-plugins/common-plugin-tasks/plugin-upgrade-guide).
  - Coverage evidence seen before the trigger: a https://www.pantsbuild.org/stable/reference/global-options; b
  - https://www.pantsbuild.org/stable/reference/goals; c
  - https://www.pantsbuild.org/stable/docs/writing-plugins/overview; d
  - https://www.pantsbuild.org/stable/docs/using-pants/key-concepts/goals.
- G4 (incidental):
  - d: yes, `--loop`: "Pants will wait until a relevant file has changed to try running them again"
  - (https://www.pantsbuild.org/stable/docs/using-pants/key-concepts/goals; option `loop` "Run goals continuously as
  - file changes are detected", https://www.pantsbuild.org/stable/reference/global-options).
  - e: in-process streaming workunit handler plugins only
  - (https://www.pantsbuild.org/stable/docs/writing-plugins/common-plugin-tasks/streaming-workunit-handlers); no
  - external channel found.
  - f: none found.
  - g: `tailor` goal generates BUILD metadata (https://www.pantsbuild.org/stable/reference/goals/tailor).
- Final: exit at G3 confusion (plugin debugging deferred to Slack).

### sagiegurari/cargo-make

- URL: https://github.com/sagiegurari/cargo-make
- License: Apache-2.0
- Latest release: 0.37.24, 2025-01-18 (last push 2026-02-05)
- Archived: no
- G1: fail, category mismatch.
  - Docs site https://sagiegurari.github.io/cargo-make/ (README rendering), "Workspace Support": "it will go to each
  - workspace member directory and execute: cargo make mytask at that directory ... The order of the members is defined
  - by the member attribute in the workspace `Cargo.toml`."
  - Tasks fan out over members in listed order; relationships between member crates are not used, so it is a task
  - runner rather than a monorepo manager. It has no persistent process with an RPC or IPC interface, so it is not a
  - component either.
- G2, G3: not evaluated.
- G4 (incidental): d watch attribute exists (README section "Watch"); plugins exist (README "Plugins").
- Final: exit at G1 (no use of project relationships).

### com-lihaoyi/mill

- URL: https://github.com/com-lihaoyi/mill
- License: MIT
- Latest release: 1.1.9, 2026-09-07
- Archived: no
- G1: pass, monorepo manager.
  - https://mill-build.org/mill/fundamentals/modules.html: modules declare `moduleDeps` and tasks traverse them
  - (`Task.traverse(moduleDeps)(_.classPath)`); the sidebar has a "Large Builds and Monorepos" section.
- G2: pass. MIT `LICENSE`.
- G3: fail, confusion.
  - Trigger (no-JS curl of https://mill-build.org/mill/cli/flags.html, HTTP 200, doc text present): the sidebar
  - "Reference" section contains only "Mill API Reference" (Scaladoc), "Changelog" (a GitHub link,
  - https://github.com/com-lihaoyi/mill/blob/main/changelog.adoc), and "Talks & Blog Posts", while the CLI flag and
  - configuration references sit under "The Mill CLI"; it is unclear where reference material lives.
  - Second trigger: "Extending Mill" lists a catalog of contrib plugins (Artifactory to Version file), "Third-Party
  - Plugins", "Writing Mill Plugins", "The Mill Meta-Build", and language-support examples as flat siblings.
  - Also noted: https://mill-build.org/mill/cli/flags.html embeds `mill --help-advanced` output labelled "Mill Build
  - Tool, version 1.1.0-28-6e5003" while the latest release is 1.1.9.
- G4 (incidental):
  - d: yes, `-w --watch` "Watch and re-run the given tasks when their inputs change"
  - (https://mill-build.org/mill/cli/flags.html).
  - e, f, g: not evaluated.
- Final: exit at G3 confusion (reference and extension navigation mix unrelated page kinds).

### thought-machine/please

- URL: https://github.com/thought-machine/please
- License: Apache-2.0
- Latest release: v17.33.0, 2026-09-09
- Archived: no
- G1: pass, monorepo manager.
  - https://please.build/basics.html ("If you're familiar with Blaze / Bazel, Buck or Pants you will probably find
  - Please ...") and https://please.build/commands.html (`plz watch` rebuilds when "their source files (or that of any
  - dependency)" change) describe targets with dependencies across a repo.
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, confusion.
  - Trigger (no-JS curl, HTTP 200, doc text present): https://please.build/plugins.html says "Plugins are a way to
  - extend Please with build rules for additional languages or technologies" and "The full list of available plugins
  - can be found here", then inlines full rule references for first-party plugins; it has no text on writing a plugin.
    The only plugin-authoring material is the `[PluginDefinition]` section of https://please.build/config.html, flagged
    "This feature is still experimental; some aspects may not work fully or at all just yet."
    Custom rules are separately in "Advanced Topics & Custom Rules" (https://please.build/build_rules.html), so it is
    unclear which extension path is documented and supported.
  - Also noted: the sidebar ends with "Browse the source".
  - Coverage evidence seen before the trigger: a https://please.build/config.html; b https://please.build/commands.html.
- G4 (incidental):
  - d: yes, `plz watch` (https://please.build/commands.html).
  - e, f, g: not evaluated.
- Final: exit at G3 confusion (plugin authoring location unclear and flagged experimental).

### blade-build/blade-build

- URL: https://github.com/blade-build/blade-build
- License: BSD-3-Clause ("NewBSD" `COPYING`; GitHub reports NOASSERTION)
- Latest release: v3.1.0, 2026-07-26
- Archived: no
- G1: pass, monorepo manager.
  - README: "A modern, high-performance build system optimized for trunk-based development in large-scale monorepo
  - environments" and "Blade resolves the dependencies between targets and unifies compilation, linking, testing".
- G2: pass (BSD-3-Clause).
- G3: fail, confusion.
  - Trigger (no-JS curl, HTTP 200, doc text present): the User Guide sidebar entry "Accessibility"
  - (https://blade-build.github.io/docs/en/misc/) opens a page whose content is "Auxiliary Commands: install, lsrc,
  - genlibbuild, alt" and "vim integration", ending "The source code for this command is here".
    The page title does not describe its content (apparent mistranslation of "auxiliary features"), so auxiliary
    commands cannot be found by name.
  - Coverage evidence seen before the trigger: a https://blade-build.github.io/docs/en/config/; b
  - https://blade-build.github.io/docs/en/command_line/; c https://blade-build.github.io/docs/en/build_rules/extension/.
- G4: not evaluated.
- Final: exit at G3 confusion (misleading page title).

### invertase/melos

- URL: https://github.com/invertase/melos
- License: Apache-2.0
- Latest release: no GitHub releases; tag `melos-v8.7.0`; pub.dev 8.7.0 published 2026-09-09
- Archived: no
- G1: pass, monorepo manager.
  - https://melos.invertase.dev/~melos-latest: "Melos is a CLI tool used to help manage Dart projects with multiple
  - packages"; https://melos.invertase.dev/~melos-latest/filters: `--include-dependencies` and `--include-dependents`
  - select packages through "transitive dependencies" and "transitive dependents".
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, coverage (c).
  - a: https://melos.invertase.dev/~melos-latest/configuration/overview. b: command pages such as
  - https://melos.invertase.dev/~melos-latest/commands/exec.
  - c: absent. The docs sidebar source `docs.json` lists Overview, Getting Started, Filters, Configuration (Overview,
  - Scripts), Commands, Environment Variables, IDE Support, Guides; no plugin or extension page.
  - Confusion triggers too: the "Guides" section links an external Medium post
  - (https://sagarsuri56.medium.com/managing-multi-package-flutter-projects-with-melos-c8ce96fa7c82) as a guide, and
  - the no-JS request returns `content-type: text/markdown` raw MDX with front matter and `<Info>` components instead
  - of rendered HTML (HTTP 200, excerpt "--- description: A tool for managing Dart projects with multiple packages. ---
  - ## About").
- G4: not evaluated.
- Final: exit at G3 coverage (no extension mechanism).

### folke/ultra-runner

- URL: https://github.com/folke/ultra-runner
- License: Apache-2.0
- Latest release: v3.10.5, 2021-02-28 (last push 2026-09-16)
- Archived: no
- G1: pass, monorepo manager.
  - README: "workspace dependencies are automatically resolved and used for parallel builds" and "Packages are build
  - concurrently as soon as their dependencies are build".
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, coverage (c).
  - Official docs are the README only (https://github.com/folke/ultra-runner#readme): a (partial) "Optional
  - Configuration", b "Usage"; c absent, with no plugin or extension section.
- G4: not evaluated.
- Final: exit at G3 coverage (no extension mechanism; README-only docs).

### guigrpa/oao

- URL: https://github.com/guigrpa/oao
- License: MIT
- Latest release: no GitHub releases; npm `oao` 2.0.2 published 2021-04-18 (last push 2023-01-03)
- Archived: no
- G1: pass, monorepo manager.
  - README: "Runs a command or `package.json` script on all sub-packages, serially or in parallel, optionally following
  - the inverse dependency tree" (`--tree`).
- G2: pass. MIT `LICENSE`.
- G3: fail, coverage (a, c).
  - README-only docs (https://github.com/guigrpa/oao#readme; `docs/` holds only images): b command list present; a no
  - configuration reference; c no extension mechanism.
- G4: not evaluated.
- Final: exit at G3 coverage.

### lerna-lite/lerna-lite

- URL: https://github.com/lerna-lite/lerna-lite
- License: MIT
- Latest release: v5.6.1, 2026-08-21
- Archived: no
- G1: pass, monorepo manager.
  - `packages/run/README.md`: runs scripts "in parallel and in topological order"; `--parallel` "completely disregards
  - concurrency and topological sorting".
- G2: pass. MIT `LICENSE`.
- G3: fail, coverage (c).
  - a: `lerna.json` reference on the GitHub wiki (https://github.com/lerna-lite/lerna-lite/wiki/lerna.json) plus JSON
  - schema; b: per-command package READMEs; c: absent (README lists optional command packages only; no third-party
  - extension doc).
  - Confusion triggers too: configuration docs live in the wiki while command docs live in package READMEs, and the
  - README "Great Lerna Tutorials / References" section points to a YouTube video.
- G4 (incidental): d `@lerna-lite/watch` "watch for changes & execute commands when fired" (README).
- Final: exit at G3 coverage (no extension mechanism).

### benchkram/bob

- URL: https://github.com/benchkram/bob
- License: Apache-2.0
- Latest release: 0.8.2-nix-shell-3, 2024-01-26 (last push 2024-05-01)
- Archived: no
- G1: provisional pass, monorepo manager.
  - README: "Use it to build codebases organized in multiple repositories or in a monorepo" and "Bob generates its
  - internal build graph from tasks described in a `bob.yaml` file"; cross-project details were on the unreachable docs
  - site.
- G2: pass. Apache-2.0 `LICENSE`.
- G3: fail, docs unreachable (confusion and no-JS rules).
  - The README's docs links https://bob.build/docs/ and https://bob.build/docs/getting-started/installation/ fail:
  - `curl` error "Could not resolve host: bob.build", and `dig bob.build @8.8.8.8` returns `status: NXDOMAIN`.
    Repo `doc/` holds only `projectname.md`, `target.md`, and an image.
- G4: not evaluated.
- Final: exit at G3 (docs domain does not resolve).

### simplesurance/baur

- URL: https://github.com/simplesurance/baur
- License: GPL-2.0
- Latest release: v5.4.6, 2026-07-13
- Archived: no
- G1: pass, monorepo manager.
  - README: "baur is an incremental task runner for monolithic Git repositories"; task inputs include "Results from
  - other task runs" and definitions are shared across applications through include files.
- G2: pass. GPL-2.0 `LICENSE`.
- G3: fail, coverage (c).
  - Official docs are the wiki (https://github.com/simplesurance/baur/wiki, HTTP 200 without JS);
  - https://github.com/simplesurance/baur/wiki/v5-Documentation has no plugin or extension mechanism.
  - Confusion triggers too: the wiki home lists "Version 5.x.x" and "Version 3.x.x" documentation (no 4.x) next to
  - "baur 0.x Concept (outdated)", "Tasks Design Draft (outdated)", and "Logos" as flat siblings.
- G4: not evaluated.
- Final: exit at G3 coverage (no extension mechanism).

### ojkelly/yarn.build

- URL: https://github.com/ojkelly/yarn.build
- License: MIT
- Latest release: v4.1.4, 2026-05-26
- Archived: no
- G1: pass, monorepo manager.
  - https://yarn.build/: "yarn .BUILD uses the dependency graph you have already defined between your local packages."
- G2: pass. MIT `LICENSE`.
- G3: fail, confusion (and coverage c).
  - Trigger (no-JS curl, HTTP 200, doc text present): https://yarn.build/ shows install tabs "Yarn 4" and "Yarn 2" but
  - states "yarn .BUILD is a plugin for Yarn v2 and v3 (berry)", while the repo README says "yarn.BUILD is a plugin for
  - Yarn 4 (berry)".
  - Coverage: no extension mechanism documented (README sections: Commands, Exclude, Git / CI integration, Config,
  - Troubleshooting).
- G4 (incidental): README "For Dev" example mentions using it at the start of a dev command "where you are watching for
- changes"; no native watch.
- Final: exit at G3 confusion (supported Yarn versions contradict).

### mbtproject/mbt

- URL: https://github.com/mbtproject/mbt
- License: Apache-2.0 (`LICENCE`)
- Latest release: v0.24, 2019-11-25 (last push 2023-10-13)
- Archived: no
- G1: pass, monorepo manager.
  - https://mbtproject.github.io/mbt/: `.mbt.yml` "dependencies: An array of modules that this module's build depend
  - on"; docs "Module Dependencies" section builds impacted modules.
- G2: pass (Apache-2.0).
- G3: fail, coverage (c).
  - a and b present (https://mbtproject.github.io/mbt/, generated `docs/mbt_*.md` command pages); c absent.
  - Confusion trigger too: README "Status" says "Visit Github issues for support".
- G4: not evaluated.
- Final: exit at G3 coverage (no extension mechanism).

### toss/yarn-plugin-workspace-since

- URL: https://github.com/toss/yarn-plugin-workspace-since
- License: none (no license file at repo root; `package.json` has no `license` field)
- Latest release: none (no releases or tags; last commit 2024-12-02)
- Archived: no
- G1: pass, monorepo manager.
  - README (Korean): `yarn workspaces since run <command> <from> [to]` runs a command on workspaces changed between
  - revisions, and changes propagate: when workspace "B" depends on "A", `run` executes for both.
- G2: fail, no license.
- G3, G4: not evaluated.
- Final: exit at G2 (no license).

### Akryum/monorepo-run

- URL: https://github.com/Akryum/monorepo-run
- License: MIT
- Latest release: v0.4.2, 2019-10-27 (marked prerelease)
- Archived: yes
- G1: fail, category mismatch.
  - README: "Run scripts in monorepo with colors, streaming and separated panes"; "Execute `mono-run <script>` to run a
  - NPM script in your monorepo packages."
    No ordering by package relationships is documented, and the dependency list has no graph or topology library; not a
    watch process with RPC either.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (runs scripts in every package without project relationships).

### seansfkelley/yerna

- URL: https://github.com/seansfkelley/yerna
- License: Apache-2.0 (`LICENSE` text; GitHub reports NOASSERTION)
- Latest release: v0.6.0, 2018-04-27 (last push 2018-05-04)
- Archived: no
- G1: pass, monorepo manager.
  - README: tasks "respect the dependency ordering of packages when running tasks", with `--dependencies` and
  - `--dependents` flags.
- G2: pass (Apache-2.0).
- G3: fail, coverage (c).
  - README-only docs (https://github.com/seansfkelley/yerna#readme); `yerna.json` supports only `packages` ("The
  - configuration file does not support any other properties"); no extension mechanism.
  - Confusion trigger too: the README calls itself "a hack-based stopgap/overgrown experiment" and "a glorified proof
  - of concept".
- G4: not evaluated.
- Final: exit at G3 coverage (no extension mechanism).

### giltayar/bilt

- URL: https://github.com/giltayar/bilt
- License: MPL-2.0 (root `LICENSE`; npm `@bilt/cli` declares MIT)
- Latest release: no GitHub releases or tags; npm `@bilt/cli` 6.0.4 published 2022-05-05 (last push 2022-12-27)
- Archived: no
- G1: pass, monorepo manager.
  - README: "builds, tests, and publishes NPM packages in your monorepos, and does that in the proper order, according
  - to the packages dependency graph".
- G2: pass (MPL-2.0).
- G3: fail, confusion.
  - Trigger (no-JS curl of https://github.com/giltayar/bilt/blob/main/docs/build-configurations.md, HTTP 200, doc text
  - present): "Build configuration reference ... You can see the definition in Typescript (as embedded in JSDoc) here",
  - linking `./packages/build-with-configuration/src/types.js`.
    Relative to `docs/`, that target does not exist (the repo tree has only
    `packages/build-with-configuration/src/types.js`), so the link is broken, and it points at source for the type
    definition.
  - Coverage evidence seen before the trigger: a `.biltrc.json` in
  - https://github.com/giltayar/bilt/blob/main/docs/reference.md; b `bilt` CLI in the same page; c build configurations
  - with `extends` (https://github.com/giltayar/bilt/blob/main/docs/build-configurations.md).
- G4: not evaluated.
- Final: exit at G3 confusion (broken link to source for the configuration type definition).

## Survivors

None.
All 29 candidates in this chunk exited or were skipped: 3 at G1 (runq, cargo-make, monorepo-run), 2 at G2 (brummer,
yarn-plugin-workspace-since), 23 at G3, and moon skipped by user exclusion.

G3 exits by primary reason:

- Coverage (10): lerna, turborepo, nx (also confusion), melos, ultra-runner, oao, lerna-lite, baur, mbt, yerna.
- Confusion (11): bazel, gradle, buck, rushstack, sbt, pants, mill, please, blade-build, yarn.build, bilt.
- No-JS (1): bit.
- Docs unreachable (1): bob (domain NXDOMAIN).

Closest to passing before the confusion rule: bazel and gradle had full coverage evidence, and nx failed coverage only
on the one-line MCP running-task listings.
