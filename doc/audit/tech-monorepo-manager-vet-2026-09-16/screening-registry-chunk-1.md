# Monorepo manager screening, registry chunk 1

Input: `screening-registry-chunk-1.json` (68 candidates).
Screened 2026-09-16.
Nothing was installed or executed.
Evidence came from `gh api` (repository metadata, READMEs, docs trees), `https://registry.npmjs.org/<pkg>` (metadata
and tarball file lists, extracted without running), PyPI JSON, and docs pages.

Rules applied, including the two user rule changes received mid-task:

- Gates stop at the first failure.
- G3 confusion rule: any confusion reading official docs (broken or 404 links, truncated or contradictory summaries,
- needing source or outside material to understand documented behavior, pages that do not load) is an immediate G3
- exit, applied retroactively to recorded structure signals.
- G3 no-JavaScript rule: each docs page relied on was fetched with plain
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0'`; a block page, empty JS shell, or missing docs text is an exit.
  Some early pnpm, Yarn, and Vite+ reads used WebFetch; every page cited for those candidates was re-fetched with that
  curl command and the cited text was found in the returned HTML.
- "Latest release" is the newest registry publish (npm unless stated), with GitHub release or tag dates where they
- differ.
- "License declared in package.json only" means the repository root has no LICENSE file and GitHub detects no license,
- but the published manifest names one.

### Bolt

- URL: https://github.com/boltpkg/bolt
- License: MIT
- Latest release: npm `bolt` 0.24.10, 2021-09-28 (no GitHub releases; last push 2024-06-01)
- Archived: no; npm deprecated flag not set
- G1: pass.
  README (https://github.com/boltpkg/bolt#readme) models a project of workspaces and runs `bolt ws run [script]` "in
  every package".
- G2: pass, MIT.
- G3: exit (coverage).
  The README is the only doc.
  It has no extension or plugin documentation (c), and the configuration section documents only `bolt.workspaces`.
  The command table marks several commands as not yet implemented.
- G4: no watch, inspection, or control features documented.
- Final: exit at G3 (no extension mechanism documentation; README-only docs).

### drkns

- URL: https://github.com/frantzmiccoli/drkns
- License: MIT (`LICENSE.md`; GitHub API reports NOASSERTION)
- Latest release: PyPI `drkns` 4.0.0, 2026-08-25 (npm 404; tag 4.0.0)
- Archived: no
- G1: pass.
  README: each `drkns.yml` "unit" declares `dependencies`, and `drkns run` runs steps with dependency-aware caching
  across units.
- G2: pass, MIT.
- G3: exit (coverage).
  README (https://github.com/frantzmiccoli/drkns#readme) has `drkns.yml` fields (a) and a CLI command list (b), but no
  extension or plugin mechanism (c).
- G4: (g) `drkns generate` renders a CI config from a `.drknsgeneration/*.template.*` file (README "Parallel CI
- generation").
  No watch or daemon.
- Final: exit at G3 (no extension mechanism documentation).

### Nix

- URL: https://github.com/NixOS/nix
- License: LGPL-2.1
- Latest release: tag 2.35.2, 2026-08-12
- Archived: no
- G1: exit (category mismatch).
  https://nix.dev/manual/nix/latest/introduction: "Nix is a purely functional package manager."
  The manual introduction describes no workspace or project model and no running of tasks across repository projects;
  builds are derivations.
- G2, G3: not evaluated.
- Final: exit at G1 (package manager and build tool, no workspace task model).

### pnpm

- URL: https://pnpm.io
- License: MIT
- Latest release: v12.4.2, 2026-09-15
- Archived: no
- G1: pass.
  https://pnpm.io/cli/recursive: with `--sort` (default) "commands follow the workspace dependency graph. Recursive
  `run` also applies relationships from the `tasks` setting."
- G2: pass, MIT.
- G3: exit (confusion and structure, per the retroactive rule).
  Coverage exists: settings reference https://pnpm.io/settings, CLI https://pnpm.io/cli/run and
  https://pnpm.io/cli/recursive, hooks https://pnpm.io/pnpmfile.
  Triggers:
  - Task running is split across unrelated nav sections with no linking page: the `tasks` graph is under Features
  - (https://pnpm.io/workspace-task-orchestration), `pnpm pipeline` is under CLI commands > Run scripts
  - (https://pnpm.io/cli/pipeline), and `pnpm -r` is under CLI commands > Misc. (next to `self-update`, `store`,
  - `doctor`).
  - https://pnpm.io/workspace-task-orchestration "Caching a task" says the `outputs`, `inputs`, `env`, `cache`, and
  - `cargoTargetDir` keys "are read by pnpm pipeline only", but the article body contains no links, so a reader cannot
  - reach the pipeline page from there.
  - The settings reference defers `tasks`, `pipelines`, and `pipelineBase` to a "Settings documented elsewhere" list
  - rather than documenting them.
- G4: (d) `pnpm pipeline --repo <url> --watch` polls a git repository for new revisions; the docs call it "a proof of
- concept, not a hosted CI service" (https://pnpm.io/cli/pipeline).
  It is not file-change watching.
  (e) No inspection of a running process; pnpr stores reports of settled runs over HTTP
  (https://pnpm.io/pnpr/pipeline-runs).
  (f) None.
- Final: exit at G3 (task-running docs scattered across Features, Run scripts, and Misc. with no links between them).

### Yarn

- URL: https://yarnpkg.com
- License: BSD-2-Clause
- Latest release: `@yarnpkg/cli` 4.18.0, 2026-07-29
- Archived: no
- G1: pass.
  https://yarnpkg.com/cli/workspaces/foreach: "If `-t,--topological` is set, Yarn will only run the command after all
  workspaces that it depends on through the `dependencies` field have successfully finished executing."
- G2: pass, BSD-2-Clause.
- G3: exit (confusion).
  https://yarnpkg.com/advanced/plugin-tutorial, section "Official hooks": "Our new website doesn't support generating
  the hook list yet; sorry :("
  The plugin hook reference is missing from the plugin guide, and the page links nowhere else for it.
  Config reference https://yarnpkg.com/configuration/yarnrc and CLI reference exist.
- G4: (g) constraints enforce and fix manifest rules (https://yarnpkg.com/features/constraints).
  No watch, daemon, or remote control claimed.
- Final: exit at G3 (plugin hook list missing from official docs).

### Builder (FormidableLabs)

- URL: https://github.com/FormidableLabs/builder
- License: MIT
- Latest release: npm `builder` 5.0.0, 2019-11-07
- Archived: yes (GitHub archived)
- G1: exit (category mismatch).
  README: "Builder takes your `npm` tasks and makes them composable", using "archetypes" to share `package.json`
  scripts across many similar repositories.
  It runs tasks within one project and has no workspace or relationship model.
- Final: exit at G1 (shared npm-script runner).

### Lank

- URL: https://github.com/FormidableLabs/lank
- License: MIT
- Latest release: npm `lank` 0.2.1, 2018-08-21
- Archived: yes
- G1: exit (category mismatch).
  README: relationships are used only by `lank link` (to delete cross-project `node_modules` copies).
  `lank exec` "runs the same command in all linked projects", filtered by tags or module names, with no dependency
  ordering.
- Final: exit at G1 (multi-repo linker plus flat exec).

### wsrun

- URL: https://github.com/hfour/wsrun
- License: MIT
- Latest release: npm 5.2.4, 2020-10-08 (last push 2024-05-04)
- Archived: no
- G1: pass.
  README: `--stages` runs "starting from those that don't depend on other packages"; `-r` includes dependencies.
- G2: pass, MIT.
- G3: exit (coverage).
  The README (https://github.com/hfour/wsrun#readme) is the only doc: it has CLI flags (b) but no configuration file
  reference (a) and no extension mechanism (c).
- G4: no native watch; runs package `watch` scripts with `--done-criteria`.
- Final: exit at G3 (no config or extension documentation).

### Vite+ task runner (vp run)

- URL: https://github.com/voidzero-dev/vite-task
- License: MIT (vite-task and vite-plus repositories)
- Latest release: `vite-plus` 0.3.2, 2026-09-14; `@voidzero-dev/vite-task-client` 0.2.0, 2026-06-18; vite-task
- repository has no releases or tags
- Archived: no
- G1: pass.
  https://viteplus.dev/guide/run documents `vp run -r` in dependency order and `-t` transitive runs; the tasks config
  reference is https://viteplus.dev/config/run.
- G2: pass, MIT.
- G3: exit (coverage and confusion).
  The official docs have no extension or plugin mechanism for the task runner (c).
  The only integration point, `@voidzero-dev/vite-task-client`, gets one sentence on
  https://viteplus.dev/guide/automatic-data-tracking ("Third-party tools can report cache metadata with
  @voidzero-dev/vite-task-client"), which links to an npmx.dev package page instead of an API reference.
- G4: no watch mode or remote control on the run guide.
- Final: exit at G3 (task-runner extension point documented only by an off-site package link).

### @visulima/vis

- URL: https://visulima.com/packages/vis
- License: MIT
- Latest release: `@visulima/vis` 4.0.2, 2026-09-07 (`@visulima/task-runner` 1.0.6, 2026-09-03)
- Archived: no
- G1: pass.
  https://visulima.com/docs/packages/vis: "A CLI task runner for monorepo workspaces with task caching,
  dependency-aware scheduling".
- G2: pass, MIT.
- G3: exit (confusion: truncated and contradictory summary).
  https://visulima.com/docs/packages/vis/configuration, "Plugins", presents "Available hooks" as a table of 7 hooks
  with no link to a complete list.
  https://visulima.com/docs/packages/vis/guides/plugins documents 13, adding `task:stdout`, `task:stderr`,
  `task:retry`, `task:fingerprint`, `service:start`, `service:stop`, and `service:attach`.
  The two pages also name the `task:cacheMiss` argument differently (`reasons` versus `reason`).
  Also seen: `packages/tooling/vis/docs/guides/caching.mdx` exists in the repository, but
  https://visulima.com/docs/packages/vis/guides/caching returns HTTP 404 "Page not found", and without JavaScript the
  sidebar exists only as hydration data (8 anchor links in the served HTML).
- G4: (d) `vis run --watch` reruns on file change and documents its watch scope and backends
- (https://visulima.com/docs/packages/vis/commands/run, "Watch Mode").
  (e) `vis-mcp` "speaks JSON-RPC over stdio" with read-only workspace and run-log tools
  (https://visulima.com/docs/packages/vis/guides/ai-integration), but it is not attached to a running watch process.
  (f) The MCP server "ships no side-effecting tools"; `vis service` manages long-lived services.
  (g) The configuration page has Constraints and CODEOWNERS sections.
- Final: exit at G3 (hook reference summary contradicts the plugin guide).

### bun-workspaces

- URL: https://bunworkspaces.com
- License: MIT
- Latest release: npm `bun-workspaces` 1.12.0, 2026-06-14; its successor `pacwich` v0.7.2, 2026-09-01 (GitHub release,
- repository `smorsic/pacwich`)
- Archived: no; https://bunworkspaces.com shows a banner saying bun-workspaces "has been deprecated and is now
- developed as pacwich" (npm deprecated field not set)
- G1: pass (evaluated on the pacwich continuation).
  https://pacwich.dev/concepts/affected runs scripts on affected workspaces, where "if a workspace dependency is
  affected for any reason, its dependents are also considered affected".
  Script order itself is alphanumeric or manual `order` values (https://pacwich.dev/concepts/script-execution-order/).
- G2: pass, MIT.
- G3: exit (confusion).
  - https://pacwich.dev/config/env-vars/ is labeled "Environment Variables" in the sidebar, but its `<title>` is
  - "Workspace Configuration | pacwich Documentation", the same title as the real workspace configuration page.
    https://pacwich.dev/llms.txt lists it the same way.
  - https://pacwich.dev/concepts/workspace-dependencies/ shows leftover authoring text in the published page: "Alt
  - workspace dependency examples for .md page in place of <WorkspaceDependencyExample /> above:".
  - No plugin or extension mechanism is documented; the docs index has only a CLI, a TypeScript API, and config.
- G4: not assessed beyond the above; an MCP page exists at https://pacwich.dev/ai/mcp/.
- Final: exit at G3 (mislabeled page title and leaked placeholder text; no extension mechanism).

### mono-vir

- URL: https://github.com/electrovir/mono-vir
- License: MIT OR CC0-1.0 (npm); repository reports CC0-1.0
- Latest release: v2.5.1, 2026-07-29
- Archived: no
- G1: pass.
  README: "runs commands for each package in correct order based on each package's dependency graph".
- G2: pass.
- G3: exit (coverage).
  README only: no configuration reference (a) and no extension mechanism (c).
- G4: (g) `print` writes the dependency graph to an SVG; no watch.
- Final: exit at G3.

### Holocron astromech (@theholocron/astromech)

- URL: https://github.com/theholocron/holocron
- License: MIT
- Latest release: npm `@theholocron/astromech` 4.19.0, 2026-09-11 (repository prerelease v5.0.0-alpha.13, 2026-09-16)
- Archived: no
- G1: exit (category mismatch: library that wraps other runners).
  `packages/astromech/README.md` calls it "A plain library" that `@theholocron/cli` instantiates.
  `holocron run <task>` resolution: "turbo.json defines the task → turbo run <task>", else "<detected pm> run <task>".
  It generates workflows and scripts; it does not model projects or relationships itself.
- Final: exit at G1 (wrapper library over Turborepo or the package manager).

### yakumo

- URL: https://github.com/cordiverse/yakumo
- License: MIT
- Latest release: 3.2.1, 2026-05-05
- Archived: no
- G1: not established from docs.
  The README says only "Manage complex workspaces with ease" and lists esbuild, mocha, and tsc extensions; it documents
  no cross-package task running.
- G2: pass, MIT.
- G3: exit (coverage).
  The repository docs are the root README (1.6 KB) plus per-extension readmes (`packages/*/readme.md`), with no
  configuration reference, no CLI reference, and no extension authoring guide, although the README says it "Allows to
  customize your own scripts".
- Final: exit at G3 (docs are a feature blurb).

### bun-spaces

- URL: https://github.com/Pckool/bun-spaces
- License: MIT (package.json and README only)
- Latest release: 0.2.7, 2025-01-17
- Archived: no
- G1: pass.
  README: "Dependency-aware script execution"; `bun-spaces run --deps`.
- G2: pass.
- G3: exit (coverage).
  README only: CLI examples, but no configuration reference (a) and no extension mechanism (c).
- Final: exit at G3.

### @williamthorsen/nmr

- URL: https://github.com/williamthorsen/node-monorepo-tools
- License: ISC
- Latest release: 0.37.0, 2026-09-15
- Archived: no
- G1: exit (add-on of another manager).
  README: "Context-aware script runner for pnpm monorepos".
  From the root, `nmr test` runs "`root:test`, then `-R test`"; `packages/nmr/docs/reporting.md` describes "packages
  running concurrently under `pnpm --recursive`".
  Cross-package ordering is pnpm's, not nmr's own.
- Final: exit at G1 (pnpm wrapper).

### npm-recursive-runner

- URL: https://github.com/avi747av/npm-recursive-runner
- License: MIT (package.json only)
- Latest release: 1.0.2, 2025-05-15
- Archived: no
- G1: exit (category mismatch).
  README: "Run any npm command recursively across all package.json directories" with parallelism and directory skips;
  it has no workspace model and no relationships.
- Final: exit at G1.

### @forklaunch/bunrun

- URL: https://www.npmjs.com/package/@forklaunch/bunrun (source: `framework/bunrun` in
- https://github.com/forklaunch/forklaunch)
- License: MIT (package.json and package README; the root LICENSE says projects carry individual licenses)
- Latest release: 1.2.24, 2026-09-04
- Archived: no
- G1: pass.
  `framework/bunrun/README.md`: "Topological ordering: Runs scripts in dependency order".
- G2: pass.
- G3: exit (coverage).
  The package README is the only doc: CLI options (b), but no configuration reference (a) and no extension mechanism
  (c).
- Final: exit at G3.

### @aklinker1/buildc

- URL: https://github.com/aklinker1/buildc
- License: MIT (package.json only)
- Latest release: npm 1.1.7, 2026-02-11 (GitHub prerelease v2.0.0-alpha9, 2025-03-04)
- Archived: no
- G1: pass (repository description "caching and orchestrating builds in monorepos"; TODO list has `buildc graph` and
- `buildc deps`).
- G2: pass.
- G3: exit (coverage).
  The README is a TODO checklist plus contributor notes: no configuration reference, no CLI reference, no extension
  mechanism.
- Final: exit at G3.

### packer-commander

- URL: https://github.com/Slowmoney/packer-commander
- License: MIT
- Latest release: 0.4.1, 2026-08-13
- Archived: no
- G1: exit (category mismatch).
  README (Russian): "TUI-раннер npm-скриптов монорепозитория" (TUI runner for a monorepo's npm scripts).
  It discovers workspace folders and launches scripts interactively, plus Docker Compose and GitLab pipeline panels.
  No documented use of inter-package relationships.
- Final: exit at G1 (interactive script launcher).

### @enspirit/emb

- URL: https://github.com/enspirit/emb
- License: ISC (package.json only)
- Latest release: 0.31.1, 2026-08-17
- Archived: no
- G1: pass.
  Docker-component monorepo builder with component dependencies and build ordering (docs site
  https://enspirit.github.io/emb/, tutorial "Build Ordering").
- G2: pass.
- G3: exit (coverage).
  The configuration reference (`website/src/content/docs/reference/configuration.md`) documents only built-in plugins
  (`autodocker`, `dotenv`, `embfiles`, `op`, `vault`) and how to configure them.
  No page documents writing or loading a third-party plugin (c).
- G4: (d) no watch mode; `watch-paths` is a rebuild-trigger strategy checked at build time.
- Final: exit at G3 (plugin system limited to built-ins, no authoring docs).

### run-shared-scripts

- URL: https://github.com/bubkoo/run-shared-scripts
- License: MIT
- Latest release: 1.1.6, 2025-10-15
- Archived: no
- G1: exit (category mismatch).
  README: define shared script bodies under `rss` in the root `package.json` and call `rss` from each package's own
  script; it runs one package's task and does not run across packages.
- Final: exit at G1.

### workspace-utils

- URL: https://torstendittmann.github.io/workspace-utils/
- License: MIT
- Latest release: 2.1.2, 2026-08-21
- Archived: no
- G1: pass.
  README: "Dependency-aware builds with topological sorting"; `wsu run --topological`.
- G2: pass, MIT.
- G3: exit (coverage).
  Docs (`docs/src`: commands, configuration, troubleshooting) contain no plugin, extension, or hook mechanism; a grep
  for plugin, extens, or hook found none (c).
- G4: no native watch; `wsu dev` runs dev scripts concurrently.
- Final: exit at G3.

### northbrook

- URL: https://github.com/northbrookjs/northbrook
- License: MIT
- Latest release: 4.6.3, 2017-01-30
- Archived: no (last push 2017-05)
- G1: exit (category mismatch).
  `PLUGINS.md`: the `exec` plugin runs commands "one by one, enter each package directory"; relationships are used only
  by `link`.
  Also seen: the Plugin API sections "Each Handler" and "Changed Packages" say "Wait just a while longer for this to be
  documented" and point to source.
- Final: exit at G1.

### scriptio

- URL: https://github.com/michaelcocova/scriptio
- License: MIT
- Latest release: 0.0.1-beta.3, 2026-08-30
- Archived: no
- G1: exit (category mismatch).
  README: "项目级通用任务 CLI" (project-level general task CLI) that loads `scriptio.config.ts` and routes interactive steps
  to commands; it has no workspace or relationship model.
- Final: exit at G1.

### rman

- URL: https://github.com/panates/rman
- License: MIT
- Latest release: 1.0.12, 2026-09-16
- Archived: no
- G1: pass.
  README: `rman build` runs "in every package, dependencies first".
- G2: pass, MIT.
- G3: exit (coverage).
  Repository docs `docs/cli.md`, `docs/cli/*.md`, and `docs/api.md` cover the CLI and a programmatic API, with
  configuration keys inside `docs/api.md`.
  No plugin or extension mechanism is documented, only pre/post script hooks (c).
- Final: exit at G3.

### bru

- URL: https://github.com/kamilkisiela/bru
- License: MIT
- Latest release: 1.3.0, 2018-12-31
- Archived: no
- G1: exit (category mismatch).
  README commands are `add`, `bump`, `set`, `get`, `remove`, and `check` (dependency version management); it has no
  task running.
- Final: exit at G1.

### @nu-art/build-and-install

- URL: https://github.com/nu-art-js/thunderstorm
- License: Apache-2.0
- Latest release: 0.500.6, 2026-05-07
- Archived: no
- G1: pass.
  `build-and-install/impl/README.md`: "builds a dependency graph between them, and executes lifecycle phases in a
  deterministic, dependency-aware order."
- G2: pass, Apache-2.0.
- G3: exit (coverage and confusion).
  The npm page README is "ERROR: No README data found!".
  The package README's "Extensibility" section only says BAI "is designed to be extended via: custom unit types, custom
  phases, custom unit discovery rules", with no mechanism documented.
  "CLI-driven execution" gives only "Common capabilities include" without a complete reference, and there is no
  configuration reference.
- Final: exit at G3.

### neex

- URL: https://github.com/Neexjs/neex
- License: MIT
- Latest release: npm `neex` 0.8.20, 2026-01-01
- Archived: no
- G1: pass.
  README: "Run on all packages", dependency graph (`--graph`).
- G2: pass, MIT.
- G3: exit (coverage and confusion).
  The README is the only doc and has no configuration reference beyond cloud cache setup and no extension mechanism.
  It also names the product and install command `neexp` (`npm install -g neexp`) while this package is `neex`.
- G4: the README architecture lists `neexp-daemon/ # Background: Watcher, P2P, State` without documented behavior.
- Final: exit at G3.

### @layermix/cli

- URL: https://github.com/layermix-labs/cli
- License: MIT
- Latest release: 2.4.0, 2026-04-20
- Archived: no
- G1: exit (category mismatch).
  README: tasks and `dependsOn` in one `task-runner.json`.
  `docs/config.md` monorepo support is only config inheritance ("a package-local `task-runner.json` can override one
  task from the root config"); there is no model of projects or relationships between them.
- Final: exit at G1 (single-graph task runner).

### @tinyaxis/toolkit

- URL: https://www.npmjs.com/package/@tinyaxis/toolkit
- License: Apache-2.0 (LICENSE in tarball)
- Latest release: 0.4.1, 2026-08-25
- Archived: no repository
- G1: exit (category mismatch).
  Tarball README: "执行 project.tiny 快捷指令的小型命令行工具" (small CLI running `project.tiny` shortcuts) plus git hooks and
  template creation; it has no workspace model.
- Final: exit at G1.

### @halecraft/verify

- URL: https://github.com/halecraft/verify
- License: MIT
- Latest release: 1.6.0, 2026-09-02
- Archived: no
- G1: exit (category mismatch).
  README: "hierarchical verification runner" over tasks in one `verify.config.ts`; its only monorepo mention is `.bin`
  lookup.
- Final: exit at G1.

### greenly

- URL: https://github.com/yusifaliyevpro/greenly
- License: LGPL-2.1-or-later
- Latest release: 1.1.6, 2026-08-23
- Archived: no
- G1: exit (category mismatch).
  README: "Config-driven project check runner" for lint, format, typecheck, and test steps in one `greenly.config.ts`;
  it has no workspace model.
- Final: exit at G1.

### rune (@multiterm/rune, also @super-repo/rune)

- URL: https://www.npmjs.com/package/@multiterm/rune
- License: MIT
- Latest release: `@multiterm/rune` 0.4.1-b.14, 2026-08-12; `@super-repo/rune` 0.4.0, 2026-05-10
- Archived: no repository listed
- G1: exit (category mismatch).
  Tarball README: "Runtime script orchestrator ... a comment-friendly replacement for `package.json#scripts`"; it has
  no project graph.
- Final: exit at G1.

### calviche

- URL: https://github.com/sosafeapp/calviche
- License: MIT
- Latest release: npm 1.0.1, 2020-08-01
- Archived: no
- G1: pass.
  README: "Executes command for all local dependencies respecting the hierarchical dependency order."
- G2: pass, MIT.
- G3: exit (coverage).
  The README only shows one usage example: no configuration reference, no CLI reference, no extension mechanism.
- Final: exit at G3.

### laoban

- URL: https://github.com/phil-rice/laoban (now https://github.com/laoban-dev/laoban)
- License: MIT
- Latest release: 1.4.59, 2026-04-17
- Archived: no
- G1: pass.
  README: "Scripts execute in the 'right order' if there are dependencies between packages."
- G2: pass, MIT.
- G3: exit (confusion).
  https://laoban.dev/laoban/LAOBAN.JSON.html (the configuration page) lists only some keys and says "most of these have
  defaults if you include the core in parents".
  The defaults are only in raw GitHub JSON files under the old `phil-rice/laoban` repository.
  The page also shows a stray `</div` fragment as visible text.
  No plugin mechanism is documented; templates and scripts are the only customization.
- Final: exit at G3.

### qiao-project

- URL: https://github.com/uikoo9/qiao-nodejs
- License: MIT
- Latest release: 5.0.6, 2025-12-23
- Archived: no
- G1: exit (category mismatch).
  npm README: "集成了一些 monorepo 的操作" (integrates some monorepo operations) plus bundled commitizen, eslint, prettier, and
  rollup presets; it documents no cross-project task running.
  The homepage https://qiao-project.vincentqiao.com/#/ returned HTTP 200 with a 1,936-byte JavaScript shell containing
  30 characters of text.
- Final: exit at G1.

### laufen

- URL: https://github.com/zrosenbauer/lauf
- License: MIT
- Latest release: 1.3.1, 2026-03-30
- Archived: no
- G1: exit (category mismatch).
  README: "Discover, validate, and execute TypeScript scripts with Zod-powered arguments"; `lauf run` runs one named
  script, with no relationship-aware cross-package runs.
- Final: exit at G1.

### crowd (also @d-fischer/crowd)

- URL: https://github.com/d-fischer/crowd
- License: MIT
- Latest release: `crowd` 0.3.3, 2026-09-13; `@d-fischer/crowd` 0.1.2, 2022-11-08
- Archived: no
- G1: claimed but not documented.
  README: "monorepo manager for TypeScript", with no behavior described.
- G2: pass, MIT.
- G3: exit (coverage).
  The README says "Documentation will follow." and has nothing else.
- Final: exit at G3.

### mondorepo

- URL: https://github.com/sencha/mondorepo
- License: MIT
- Latest release: 0.1.705, 2016-11-08
- Archived: no (last push 2017-08)
- G1: exit (category mismatch).
  README: `mondo install` "will connect all used repositories"; neither the README nor `docs/gettingstarted.md`
  documents running tasks.
- Final: exit at G1 (multi-repo linker).

### mrpm

- URL: https://github.com/ota-meshi/mrpm
- License: MIT
- Latest release: 4.1.0, 2022-02-02
- Archived: no
- G1: pass.
  README: "created to execute a simple npm command in order of dependencies."
- G2: pass, MIT.
- G3: exit (coverage).
  The README has one option (`--mrpm-max-workers`), no configuration reference, and no extension mechanism.
- Final: exit at G3.

### @jakehamilton/titan

- URL: https://github.com/jakehamilton/packages
- License: Apache-2.0
- Latest release: 5.11.5, 2024-08-09
- Archived: no
- G1: pass.
  npm README: `titan run build --ordered` "Build all packages in order of dependencies."
- G2: pass, Apache-2.0.
- G3: exit (coverage).
  The README is CLI help output only: no configuration reference and no extension mechanism (starter templates via
  `@starters/core` are scaffolding, not titan extensions).
- Final: exit at G3.

### buildverse

- URL: https://github.com/Typeverse/buildverse-sdk
- License: GPL-3.0
- Latest release: 3.4.1, 2017-10-12
- Archived: repository returns 404
- G1: exit (category mismatch).
  Tarball README: "Notice that the run order was the same order that the respective projects were added";
  `for-each-project` runs in list order, not by relationships.
- Final: exit at G1 (also no reachable source repository).

### vx (@vzn/vx, with @vzn/vx-cloud)

- URL: https://github.com/vznjs/vx
- License: MIT
- Latest release: `@vzn/vx` 0.0.21, 2026-09-13; `@vzn/vx-cloud` 0.0.16, 2026-07-12
- Archived: no
- G1: pass.
  README: task graph with `dependsOn: ['^build']`, `vx run build` covers the "cwd project + its workspace deps".
- G2: pass, MIT.
- G3: exit (confusion: broken link on the docs home).
  The first docs page, https://vznjs.github.io/vx/ (HTTP 200 via curl), has a "Benchmarks: tables, methodology,
  reproduce it →" link whose served `href` is the unrendered template string `{href('benchmarks/')}`.
  It resolves to https://vznjs.github.io/vx/%7Bhref('benchmarks/')%7D, which returns HTTP 404 "Page not found".
  Reading stopped there, per the rule.
- G4: per the README only: (d) `vx watch lint` "re-run on file changes".
- Final: exit at G3.

### lattice (@latticeandcompany/lattice)

- URL: https://latticeandcompany.github.io/lattice
- License: ISC
- Latest release: 1.1.1, 2026-09-10
- Archived: no
- G1: pass.
  README: "Lattice runs the tasks in your repo in dependency order and in parallel."
- G2: pass, ISC.
- G3: exit (coverage).
  CLI (https://latticeandcompany.github.io/lattice/docs/cli) and configuration
  (https://latticeandcompany.github.io/lattice/docs/configuration) references exist.
  There is no plugin or extension mechanism: the only "Extension seams"
  (https://latticeandcompany.github.io/lattice/docs/architecture) are contributor instructions to edit source, for
  example "Add a DriverSpec to the DRIVERS array in lattice-workspace" (c).
- G4: (e)(f) the desktop app links the engine in-process ("The engine is linked into the app";
- https://latticeandcompany.github.io/lattice/docs/desktop-app), so there is no IPC.
  (d) Persistent tasks are supported, but no watch mode is documented.
- Final: exit at G3 (extension only by modifying source).

### monopkg

- URL: https://github.com/beerush-id/monopkg
- License: MIT (package.json only)
- Latest release: 0.5.0, 2025-12-22
- Archived: no
- G1: pass.
  Repository `docs/guides/run.md`: `--strict` "Wait for the dependencies to be resolved before running the scripts."
- G2: pass.
- G3: exit (confusion: official docs site unreachable).
  The README links "Documentation" to https://monopkg.beerush.io, which is also the repository homepage; curl returned
  "Could not resolve host: monopkg.beerush.io".
- Final: exit at G3.

### fiducial (@fiducial/fiducial)

- URL: https://github.com/AleksaZCodes/fiducial
- License: MIT
- Latest release: `@fiducial/fiducial` 0.1.0, 2026-09-06
- Archived: no
- G1: exit (category mismatch).
  README: "a cross-domain build system for products that span web, firmware, electronics, mechanical, simulation and
  content"; `fid derive` generates artifacts from declared facts.
  It is not a workspace-of-projects task orchestrator.
- Final: exit at G1.

### Fabr (@fabr-build/core, cli, js)

- URL: https://fabr.build/
- License: GPL-3.0-or-later
- Latest release: npm 0.2.1, 2026-08-11 (GitHub release v0.2.1, 2026-08-12)
- Archived: no
- G1: pass.
  README: "every build step lands in one dependency graph, cached by its inputs"; targets declare dependencies, with
  plugin rules (`plugin @fabr-build/js`).
- G2: pass, GPL-3.0-or-later.
- G3: exit (no-JavaScript and curl rule: docs not retrievable).
  The first curl request to https://fabr.build/ failed with "Connection timed out after 40000 milliseconds" (HTTP
  status 000; resolves to 67.205.27.5; an IPv4 retry also timed out).
  WebFetch from another network did load the page, so this looks like network-level blocking of this client or a
  routing problem, not an outage.
  This exit is weaker than the others; see the report notes.
- Final: exit at G3.

### rune (@gio-labs/rune)

- URL: https://github.com/giancarlosisasi/rune
- License: MIT
- Latest release: 0.1.4, 2026-08-10
- Archived: no
- G1: exit (category mismatch).
  README: "Rune is a script registry and a runner. It is not a task graph: no caching, no topological ordering, no
  remote execution."
- Final: exit at G1.

### unorepo

- URL: https://github.com/0livare/unorepo
- License: ISC (package.json only)
- Latest release: 0.2.2, 2023-09-12
- Archived: no
- G1: exit (add-on or wrapper of other managers).
  README: "A tool for managing a monorepo via Lerna and Yarn workspaces", and "Some of the commands in this utility
  will merely be aliases to other commands".
- G4: (d) `uno watch` runs a script in modified packages.
- Final: exit at G1.

### Roc with roc-plugin-repo

- URL: https://www.npmjs.com/package/roc-plugin-repo
- License: npm license field unset; Roc repository (https://github.com/rocjs/roc) is MIT
- Latest release: `roc-plugin-repo` 0.1.7, 2018-02-05
- Archived: no
- G1: exit (plugin of another tool).
  npm README: "Roc plugin making it easy to manage JavaScript repositories".
- Final: exit at G1.

### changeset-releaser

- URL: https://github.com/harnyk/changeset-releaser
- License: none (no LICENSE file, no package.json license, GitHub detects none)
- Latest release: 0.1.2, 2023-10-23
- Archived: no
- G1: pass.
  npm README: builds "only for the packages that have changed and their dependencies and/or dependents" using the
  monorepo dependency graph.
- G2: exit (no license).
- Final: exit at G2.

### monox (with @monoxon/* binaries)

- URL: https://github.com/monoxon/monox
- License: MIT
- Latest release: npm 0.4.12, 2025-08-30
- Archived: no
- G1: pass.
  README: "monox run --all --command build" builds "in dependency order".
- G2: pass, MIT.
- G3: exit (coverage).
  README (plus Chinese README, `DESIGN.md`, `docs/release.md`) mentions `monox.toml` without a field reference, and
  there is no extension mechanism; a grep for plugin, extens, or hook found none.
- Final: exit at G3.

### @varlabs/monorun

- URL: https://github.com/HamzaKV/monorun
- License: MIT
- Latest release: npm 0.1.5, 2025-05-17 (repository pushed 2026-08-13)
- Archived: no
- G1: pass.
  README: "Tasks run in topological order based on workspace relationships."
- G2: pass, MIT.
- G3: exit (confusion).
  README "Configuration" gives only an example: the `hooks.cache.read` comment says "return a CacheRow", and
  `write(hash, ctx)` takes a `ctx`, but neither `CacheRow` nor `ctx` is documented, so their shape must be read from
  source.
  There is no complete configuration field reference.
- G4: README "Future Plans: Support for watch mode", so no watch today.
- Final: exit at G3.

### @shazhou/proman (with proman-core)

- URL: https://www.npmjs.com/package/@shazhou/proman
- License: MIT per tarball LICENSE (package.json license field unset)
- Latest release: `@shazhou/proman` 0.12.0, 2026-07-22
- Archived: no repository listed
- G1: exit (category mismatch).
  Tarball `prompts/usage.md`: "the standard development toolchain for Shazhou team pnpm monorepos" (build, test, bump,
  publish, deploy by package type); it documents no relationship-aware task running.
- Final: exit at G1 (also no source repository).

### bun-manage-workspace

- URL: https://github.com/kkiwior/bun-manage-workspace
- License: MIT (package.json only)
- Latest release: 0.4.2, 2026-07-25
- Archived: no
- G1: pass.
  README: `bmw run` "Runs a package.json script across the workspace dependency graph."
- G2: pass.
- G3: exit (coverage).
  README only: flags documented (b), but no configuration file reference (only `.bmwignore` and `BMW_CACHE_DIR`) and no
  extension mechanism (c).
- G4: (d) `bmw run build --watch` "re-runs the plan" on change, with changes coalesced (README "Watch mode").
- Final: exit at G3.

### moci

- URL: https://oss.zero-one-group.com/monorepo
- License: MIT
- Latest release: 0.2.0, 2025-10-16
- Archived: no (repository description "[WIP]")
- G1: exit (category mismatch).
  The README documents no commands.
  The docs site overview (https://oss.zero-one-group.com/monorepo/overview/) describes a monorepo template whose task
  running is moon ("moon :test Run tests in all projects"), so moci is scaffolding and migration tooling.
- Final: exit at G1.

### lerna-run

- URL: https://github.com/shokai/lerna-run
- License: MIT (package.json only)
- Latest release: 0.0.2, 2016-02-08
- Archived: no
- G1: exit (category mismatch).
  README: "move into each directories, and execute command", serial or parallel, with no relationships.
- Final: exit at G1.

### dragon (@bunvader/dragon)

- URL: https://github.com/Flora90001/dragon
- License: MIT (tarball LICENSE)
- Latest release: 0.0.22, 2026-04-23
- Archived: the GitHub repository is empty ("This repository is empty", HTTP 404 on contents)
- G1: claimed only by the npm description "This is an experimental Monorepo Build-System."; there is no README (npm
- shows "ERROR: No README data found!").
- G2: exit (no inspectable source in the linked repository).
- Final: exit at G2.

### @zssz-soft/zs

- URL: https://github.com/zssz-soft/libraries
- License: MIT (package.json)
- Latest release: 0.6.5, 2026-03-09
- Archived: repository returns HTTP 404
- G1: claimed only by the npm description ("Smart monorepo build CLI ... for the zs-platform"); the tarball has no
- README.
- G2: exit (source repository not reachable).
- Final: exit at G2.

### monilla

- URL: https://github.com/ctrlplusb/monilla
- License: MIT (package.json only)
- Latest release: 0.3.0, 2022-06-29
- Archived: no
- G1: exit (category mismatch).
  README "CLI Reference" lists `clean`, `install`, `link`, `refresh`, `watch`, and `upgrade`; `watch` updates linked
  packages, and nothing runs tasks across projects.
- Final: exit at G1.

### just-build-tools

- URL: https://github.com/mattsibs/just-build-tools
- License: ISC (package.json only)
- Latest release: 0.0.12, 2024-10-21
- Archived: no
- G1: pass.
  README: finds each `justfile` with a `build` recipe and runs "them in dependant order" from `depends-on.yml`.
- G2: pass.
- G3: exit (coverage).
  README only: no configuration reference beyond one `depends-on.yml` example, and no extension mechanism.
- Final: exit at G3.

### mono-runner

- URL: https://github.com/JanNitschke/mono-runner
- License: MIT (package.json only)
- Latest release: 0.6.0, 2025-05-09
- Archived: no
- G1: pass.
  README: `--all` "will run the script on all packages but still respect the dependency order."
- G2: pass.
- G3: exit (confusion).
  README "run a script": "Mono also provides the ```--no-wait``` flag to run scripts ." (the sentence stops without
  saying what the flag does).
  The "exit code" and "package managers" sections repeat an unrelated `mono <package> <script> -- <args>` example.
  A resolver extension (`mono.config.js`) is documented.
- Final: exit at G3.

### pmnps (with @pmnps/* plugins)

- URL: https://github.com/filefoxper/pmnps-workspace
- License: MIT (repository `LICENSE.md`; npm license field unset)
- Latest release: 4.5.3, 2024-12-09
- Archived: no
- G1: pass (weak).
  README: `start` can be limited to "packages which are in the dependencies tree of starting platforms" via the
  `build by dependencies` option.
- G2: pass, MIT.
- G3: exit (coverage and confusion).
  README "plugins": "you can write plugins", but no plugin API is documented.
  "The `run` command is not listed in pmnps command options."
- Final: exit at G3.

### @webeferen/buildable

- URL: https://github.com/WebEferen/buildable
- License: MIT (package.json only)
- Latest release: 1.4.2, 2023-03-09
- Archived: no
- G1: pass.
  README: `execution-order` "generates execution order in which projects should be run", and `run` runs per project
  with dependencies.
- G2: pass.
- G3: exit (coverage).
  The configuration is only an example file (`EXAMPLE.md`), and there is no extension mechanism.
- Final: exit at G3.

### monist / monist-tools

- URL: https://github.com/lddubeau/monist
- License: MIT (package.json only)
- Latest release: `monist` 1.7.0, 2020-02-10; `monist-tools` 2.0.0, 2021-11-06
- Archived: no; the README says "Monist is being phased out in favor of npm workspaces"
- G1: pass.
  README: `monist run` "orders execution by taking into account inter-package dependencies."
- G2: pass.
- G3: exit (coverage).
  The README documents `monistrc.json` and commands, but no extension mechanism (c).
- Final: exit at G3.

### mrdr

- URL: https://github.com/padcom/mrdr
- License: Apache-2.0 (package.json only)
- Latest release: 0.4.0, 2026-03-31
- Archived: no
- G1: pass.
  README: runs the `dev` script "in order from least-dependent to most-dependent."
- G2: pass.
- G3: exit (coverage).
  README only: CLI help (b), but no configuration reference and no extension mechanism.
- Final: exit at G3.

### yamat

- URL: https://github.com/cancerberoSgx/yamat
- License: MIT
- Latest release: 0.1.3, 2019-06-01
- Archived: no
- G1: exit (category mismatch).
  README: "dependency order in configuration file is user's responsibility"; `yamat run` "will execute `npm test` on
  each package, serially" in configured order.
- Final: exit at G1.

## Survivors

None.
All 68 candidates in this chunk exit:

- G1 (category): 29 exits: Nix, Builder, Lank, Holocron astromech, nmr, npm-recursive-runner, packer-commander,
- run-shared-scripts, northbrook, scriptio, bru, @layermix/cli, @tinyaxis/toolkit, @halecraft/verify, greenly,
- @multiterm/rune, qiao-project, laufen, mondorepo, buildverse, fiducial, @gio-labs/rune, unorepo, roc-plugin-repo,
- @shazhou/proman, moci, lerna-run, monilla, yamat.
- G2 (license or source): 3 exits: changeset-releaser (no license), dragon (empty repository), @zssz-soft/zs
- (repository 404).
- G3 (documentation, including the confusion and no-JavaScript rules): 36 exits: Bolt, drkns, pnpm, Yarn, wsrun, Vite+
- task runner, @visulima/vis, bun-workspaces/pacwich, mono-vir, yakumo, bun-spaces, @forklaunch/bunrun,
- @aklinker1/buildc, @enspirit/emb, workspace-utils, rman, @nu-art/build-and-install, neex, calviche, laoban, crowd,
- mrpm, @jakehamilton/titan, vx, lattice, monopkg, Fabr, monox, @varlabs/monorun, bun-manage-workspace,
- just-build-tools, mono-runner, pmnps, @webeferen/buildable, monist, mrdr.

Exit to re-check first: Fabr.
Its G3 exit rests on a curl connection timeout from this host (HTTP 000), while another fetcher loaded
https://fabr.build/.
The user rules name bot challenges, block pages, and JavaScript shells, not TCP timeouts, so this exit is less certain
than the rest.
