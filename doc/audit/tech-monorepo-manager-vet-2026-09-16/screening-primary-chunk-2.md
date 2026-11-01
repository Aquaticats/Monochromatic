# Monorepo manager screening, chunk 2

Screened 2026-09-16.
Input: `screening-chunk-2.json` (29 candidates).
Rules applied: G1 category fit, G2 inspectable open source, G3 documentation gate, G4 capability notes.
Mid-task rule changes applied to every candidate, including retroactively:

- G3 confusion rule: any confusion or frustration while reading official docs (broken links, contradictory text,
- features only explained by source or fixtures, pages that do not load) is an immediate G3 exit, and a recorded
- structure signal counts as an exit.
- G3 no-JS rule: every docs page relied on was fetched with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0'`, checking HTTP status, bot-challenge markers, and whether the doc text is present outside `<script>` tags.
  All pages checked returned 200 with doc text present in server-rendered HTML, except `https://docs.warp.build/` (DNS
  failure).
  On GitHub pages the literal `captcha` appears only inside feature-flag names (`octocaptcha_origin_optimization`), not
  a challenge.
  No candidate survived because a browser rendered its docs.

Evidence method: `gh api repos/<owner>/<repo>`, releases and tags (newest tag date = tagged commit date), shallow
clones of each repo for docs sources, and fetches of live docs pages.
No candidate tool was installed or executed.

### colcon/colcon-core

- URL: https://github.com/colcon/colcon-core
- License: Apache-2.0 (`LICENSE` at repo root)
- Latest release: no GitHub releases; newest tag `0.21.2` (tagged commit 2026-09-11)
- Archived: false
- G1 pass (monorepo manager).
  README: "a command line tool to improve the workflow of building, testing and using multiple software packages. It
  automates the process, handles the ordering".
  Workspace concept and dependency-aware selection (`--packages-up-to`, `--packages-above`) at
  https://colcon.readthedocs.io/en/released/user/how-to.html
- G2 pass: Apache-2.0 license file covers the tool; no hosted-only parts.
- G3 exit (coverage (c)).
  - (a) Configuration reference present: https://colcon.readthedocs.io/en/released/user/configuration.html
  - (`colcon.pkg`, `.meta`, `defaults.yaml`, `COLCON_HOME`).
  - (b) CLI reference present: https://colcon.readthedocs.io/en/released/reference/verb/build.html and sibling verb
  - pages, plus global/executor/selection argument pages.
  - (c) Missing how third parties add behavior.
  - https://colcon.readthedocs.io/en/released/developer/extension-point.html only names extension points and when they
  - run ("The following describes some of the extension points and how they are used"); it never says how an extension
  - is registered, discovered, or what interface it implements.
    The design page (https://colcon.readthedocs.io/en/released/developer/design.html) states extensibility as a goal
    only.
    Writing an extension requires reading source, which counts as undocumented.
  - (d) to (f) not claimed natively.
  - No-JS check: extension-point and configuration pages 200, text present.
- G4
  - (d) watch mode: none documented natively.
  - (e) inspect running process: none.
  - (f) control running process: none.
  - (g) generate/enforce files: none documented.
- Final: exit at G3 (coverage (c): extension registration undocumented).

### paularmstrong/onerepo

- URL: https://github.com/paularmstrong/onerepo
- License: MIT (`LICENSE.md`)
- Latest release: no GitHub releases (single tag `v0`, 2024-01-05); npm `onerepo` latest `2.0.0` published 2025-06-28
- Archived: false
- G1 pass (monorepo manager).
  `one tasks` "Run tasks against repo-defined lifecycles. This command will limit the tasks across the affected
  Workspace set based on the current state of the repository." (https://onerepo.tools/core/tasks/)
- G2 pass: MIT license file.
- G3 exit (confusion).
  - Coverage was otherwise met: config reference https://onerepo.tools/docs/config/, tasks CLI reference
  - https://onerepo.tools/core/tasks/, plugin mechanism https://onerepo.tools/plugins/#writing-plugins plus
  - https://onerepo.tools/api/.
  - Confusion trigger (contradictory text): on https://onerepo.tools/docs/config/ the `taskConfig?` section documents
  - `lifecycles?` and `stashUnstaged?` as keys under `taskConfig`, but both examples set them under `tasks`
  - (`tasks: { lifecycles: ['deploy-staging'] }` and `tasks: { stashUnstaged: [...] }`).
    https://onerepo.tools/core/tasks/#adding-more-lifecycles uses `taskConfig: { lifecycles: [...] }`.
    A reader cannot tell which key path is correct without reading source.
    Confirmed in served HTML by plain curl (200; excerpt: "tasks: { lifecycles: [ 'deploy-staging' ], }, };
    stashUnstaged?").
  - Further signal: the "API Reference" nav group lists only `builders`, `file`, `git` namespaces while most API
  - (plugins, graph, config types) lives on the single `/api/` page.
- G4
  - (d) no native task watch mode; the Jest and Vitest plugins pass `--watch` to those tools
  - (https://onerepo.tools/plugins/jest/, https://onerepo.tools/plugins/vitest/).
  - (e) none. (f) none.
  - (g) yes: `one generate` from templates (https://onerepo.tools/core/generate/), CODEOWNERS sync/verify
  - (https://onerepo.tools/core/codeowners/), TypeScript plugin syncs project references
  - (https://onerepo.tools/plugins/typescript/).
- Final: exit at G3 (confusion: config reference examples contradict documented key paths).

### sportradar/elixir-workspace

- URL: https://github.com/sportradar/elixir-workspace
- License: no root license (GitHub reports none); the tool package has MIT at `workspace/LICENSE`, and README says each
- package carries its own MIT `LICENSE`
- Latest release: no GitHub releases; newest tag `workspace/v0.3.2` (2026-08-05)
- Archived: false
- G1 pass (monorepo manager).
  "`mix workspace.run` allows you to: Run a task against all or a subset of projects. Run tasks only on projects based
  on their status and workspace graph topology." (https://hexdocs.pm/workspace/readme.html)
- G2 pass: MIT license file in `workspace/` covers the tool.
- G3 exit (confusion).
  - Coverage was otherwise met: config reference https://hexdocs.pm/workspace/Workspace.Config.html, CLI reference
  - https://hexdocs.pm/workspace/Mix.Tasks.Workspace.Run.html, custom checks behaviour
  - https://hexdocs.pm/workspace/Workspace.Check.html.
  - Confusion trigger (feature referenced but not documented anywhere):
  - https://hexdocs.pm/workspace/Workspace.Config.html says the config file is the place to "Define any other
  - configuration option may be needed by a third party plugin", and the module docs refer to "various plugins or mix
  - tasks" defining options.
    No page explains what a workspace plugin is, how one is loaded, or how it reads these options; the only extension
    doc is custom checks.
  - Structure signal: navigation is the ExDoc module list (Workspace, Check APIs, Checks, Utilities, Test Coverage,
  - Testing, plus task pages) with a single Overview guide; no guide ties features together.
  - No-JS check: Config and workspace.run pages 200, text present.
- G4
  - (d) none. (e) none. (f) none.
  - (g) partial: `mix workspace.check` enforces project config rules (dependency versions, boundaries) rather than
  - files; `workspace_new` installer scaffolds a new workspace (README).
- Final: exit at G3 (confusion: "third party plugin" referenced with no plugin documentation).

### chrismatix/grog

- URL: https://github.com/chrismatix/grog
- License: MIT (`LICENSE`)
- Latest release: `v0.46.1` published 2026-09-04
- Archived: false
- G1 pass (monorepo manager). Repo description: "a mono-repo build tool that is agnostic on how you run your build
- commands, but instead focuses on caching and parallel execution"; targets with cross-package dependencies
- (https://grog.build/build-configuration/).
- G2 pass: MIT license file.
- G3 exit (coverage (c)).
  - (a) present: https://grog.build/reference/configuration/ ("an overview of all possible grog configuration options").
  - (b) present: https://grog.build/reference/commands/ (generated cobra reference).
  - (c) missing: no plugin or extension mechanism anywhere in the docs sources (`docs/src/content/docs`: Start Here,
  - Topics, Tracing, Reference); searched for plugin/extension terms with no mechanism found.
  - (d) to (f) not claimed.
  - No-JS check: configuration and commands pages 200, text present.
- G4
  - (d) none documented.
  - (e) none. (f) none.
  - (g) none documented.
- Final: exit at G3 (coverage (c): no extension mechanism doc).

### smorsic/pacwich

- URL: https://github.com/smorsic/pacwich
- License: MIT (`LICENSE.md`)
- Latest release: `pacwich-v0.7.2` published 2026-09-01
- Archived: false
- G1 pass (monorepo manager). README: "Monorepo tooling that works on top of Bun, npm, and pnpm workspaces ... Has an
- affected graph ... running scripts across workspaces"; script execution order concept at
- https://pacwich.dev/concepts/script-execution-order
- G2 pass: MIT license file.
- G3 exit (coverage (c)).
  - (a) present: https://pacwich.dev/config and subpages (project, workspace, env vars).
  - (b) present: https://pacwich.dev/cli/commands and https://pacwich.dev/api/reference.
  - (c) missing: no plugin, hook, or extension mechanism documented; the TypeScript API is a consumer library, not a
  - way to add behavior to pacwich. Searches of the docs source (`workspaces/web/documentation-website/src`) for
  - plugin/extension/hook found none.
  - No-JS check: CLI commands, config, API reference pages 200, text present.
- G4
  - (d) no native watch; `--watch` appears only as an argument passed through to a user script in a `run-interactive`
  - example.
  - (e) `pacwich mcp-server` serves documentation resources over stdio (https://pacwich.dev/ai/mcp), not a
  - running-watch inspection channel.
  - (f) none.
  - (g) none; `verify` checks declared workspace dependencies (https://pacwich.dev/concepts/verify) rather than
  - generating files.
- Final: exit at G3 (coverage (c): no extension mechanism doc).

### nickolaj-jepsen/fnug

- URL: https://github.com/nickolaj-jepsen/fnug
- License: GPL-3.0 (`LICENSE`)
- Latest release: `v0.1.0-alpha.13` published 2026-03-14
- Archived: false
- G1 exit (category mismatch).
  README: "Fnug is a TUI command runner that automatically selects and executes lint and test commands based on git
  changes or file watching. Think of it as a terminal multiplexer".
  Its "Workspace support" only "discover[s] and merge[s] `.fnug.yaml` files from subdirectories ... as child groups";
  `depends_on` links commands, not projects, so there is no project model or project relationships.
  Not a component either: the watch loop lives inside the TUI with no documented RPC/IPC/HTTP channel.
  An `fnug mcp` stdio server exists in source (`src/mcp.rs`, tools `list_lints`, `run_lint`, `run_all`) and is
  mentioned only in the contributor file `CLAUDE.md`; it runs commands itself instead of attaching to a running TUI.
- G2 to G3: not reached.
- G4 notes: (d) watch auto-selects commands on file change (README "File watching"); (e)/(f) nothing documented; (g)
- `fnug init-hooks` installs a git pre-commit hook.
- Final: exit at G1 (TUI command runner, no project model).

### run-z/run-z

- URL: https://github.com/run-z/run-z
- License: MIT (`LICENSE`)
- Latest release: `v2.1.0` published 2024-10-08
- Archived: true
- G1 pass (monorepo manager). README: "Tasks can be batched ... The dependencies between packages respected when
- batching - the tasks executed in dependencies-first order."
- G2 pass: MIT license file.
- G3 exit (coverage (a), (c) and confusion).
  - Official docs are the README, a small wiki (pages "Partial builds", "Reusing package selectors", "Task annexes"),
  - and an "API Documentation" link.
  - (a)/(b): no consolidated configuration or options reference; options such as `--batch-parallel`, `--all`,
  - `--with-deps` appear only inside narrative examples in https://github.com/run-z/run-z#readme.
  - (c): no extension mechanism documented.
  - Confusion trigger: the "API Documentation" link https://run-z.github.io/run-z/ renders the README again inside
  - TypeDoc chrome with no API members.
  - No-JS check: repo page and wiki page 200, text present.
- G4: (d) none; (e) none; (f) none; (g) none.
- Final: exit at G3 (no config/options reference, no extension doc, API link is a README copy).

### electrode-io/fynpo

- URL: https://github.com/electrode-io/fynpo
- License: Apache-2.0 (`LICENSE`)
- Latest release: no GitHub releases; newest tag `fynpo-rel-20220416-e5808e8` (2022-04-16), last commit 2022-04-16
- Archived: false
- G1 pass (monorepo manager). README: "fynpo is a zero setup monorepo manager for node.js"; `fynpo bootstrap` installs
- and links local packages with dependency levels, `fynpo run` runs a script in all packages
- (https://jchip.github.io/fynpo/docs/commands/run).
- G2 pass: Apache-2.0 license file.
- G3 exit (coverage (c)).
  - (a) present but thin: https://jchip.github.io/fynpo/docs/getting-started/configuration
  - (b) present: https://jchip.github.io/fynpo/docs/commands/run and sibling command pages.
  - (c) missing: no plugin or extension mechanism in any page (sidebar: Fynpo, Getting Started, Commands).
  - Confusion signals: the configuration page never names the config file; only the bootstrap page mentions
  - `fynpo.config.js` or `fynpo.json`. The reference sits under "Getting Started". README docs link targets
  - `jchip.github.io/fynpo` while the repo homepage field is `electrode.io/fynpo`.
  - No-JS check: configuration and run pages 200, text present.
- G4: (d) none; (e) none; (f) none; (g) none (changelog and publish commands only).
- Final: exit at G3 (coverage (c): no extension mechanism doc).

### Farfetch/garment

- URL: https://github.com/Farfetch/garment
- License: MIT (`LICENSE.md`)
- Latest release: `v0.18.1` published 2022-09-21
- Archived: true (README: "FARFETCH has decided to archive this project, as widely adopted alternatives, like Nx, have
- become available.")
- G1 pass (monorepo manager). README: "Garment employs various technics such as dependency graph analysis, parallel
- execution, incremental builds"; `garment <taskName> [project]` runs across projects.
- G2 pass: MIT license file.
- G3 exit (confusion).
  - Coverage was otherwise met: config reference https://github.com/Farfetch/garment/blob/master/docs/Configuration.md,
  - CLI https://github.com/Farfetch/garment/blob/master/docs/CLI.md, runner and schematics authoring
  - https://github.com/Farfetch/garment/blob/master/docs/creating_runner.md and `docs/creating_schematics.md`, watch
  - behavior spread across CLI.md (`watch`), Configuration.md (`skipWatch`, `buildDependencies.watch`),
  - creating_runner.md (`ctx.input`, `watchDependencies`).
  - Confusion triggers in https://github.com/Farfetch/garment/blob/master/docs/CLI.md: the `action-graph` section shows
  - usage `garment dep-graph <taskName> <projectName>`; the `tasks <project-name>` section appears twice with different
  - usage lines (`garment tasks my-app` and `garment tasks`).
  - Structure signal: watch behavior is split across three files with no tying page.
  - No-JS check: repo page and `docs/CLI.md` blob page 200, text present.
- G4
  - (d) yes: `--watch` re-executes tasks on input change; `skipWatch` per task (Configuration.md).
  - (e) none. (f) none.
  - (g) yes: schematics via `garment generate` (README "Schematics", `docs/creating_schematics.md`).
- Final: exit at G3 (confusion: CLI reference contradicts itself).

### AmbitionEng/qik

- URL: https://github.com/AmbitionEng/qik
- License: BSD-3-Clause (`LICENSE`)
- Latest release: `0.2.9` published 2026-02-09
- Archived: false
- G1 pass (monorepo manager). README: "an extensible command runner for monorepos"; commands parametrized across
- modules, with command, module-import, and distribution dependencies (https://qik.build/en/stable/commands/).
- G2 pass: BSD-3-Clause license file.
- G3 exit (coverage (a) and confusion).
  - (a) missing: no configuration reference; `qik.toml` keys are spread across guide pages (commands, spaces, context,
  - caching, plugin pages). Site nav has Guide, Cookbook, Blog, Roadmap, Release Notes, Contributing, and no Reference
  - section.
  - (b) no full CLI reference: https://qik.build/en/stable/guide/ lists core flags and defers to the runner section;
  - `-v`, `-l`, `--isolated`, `--cache-status` appear only in narrative text.
  - (c) present: https://qik.build/en/stable/plugin_intro/
  - (d) watch claimed; behavior doc is one sentence: "Use `--watch` to reactively re-run commands based on file
  - changes" (https://qik.build/en/stable/commands/#watching-for-changes).
  - Confusion trigger: the guide's "See the command runner section" links to `../commands/#runner`, but the section
  - anchor is `#the-command-runner`, so the link lands at the page top.
  - No-JS check: commands and guide pages 200, text present.
- G4
  - (d) yes, `--watch` (https://qik.build/en/stable/commands/#watching-for-changes).
  - (e) none. (f) none.
  - (g) none (pygraph plugin lints import boundaries, no file generation).
- Final: exit at G3 (no configuration reference; broken runner anchor).

### bazurbat/jagen

- URL: https://github.com/bazurbat/jagen
- License: MIT (`LICENSE`)
- Latest release: no GitHub releases; newest tag `v6.0` (2020-11-24), last commit 2022-01-13
- Archived: false
- G1 pass (loose fit). README: "Build or rebuild any stage of any package respecting dependencies"; a workspace of
- packages built as one system (Repo/Yocto-style).
- G2 pass: MIT license file.
- G3 exit (confusion and coverage).
  - Confusion trigger: README "Warning" section on https://github.com/bazurbat/jagen says "The documentation about
  - rules is now mostly obsolete. The latest stable version before the rewrite was moved to the `legacy` branch." The
  - rules doc (`doc/Rules.md`) is the configuration reference, so the reference is self-declared stale for current
  - `master`.
  - (c) missing: no extension or plugin mechanism doc beyond rule inheritance (`extends`) inside the obsolete rules doc.
  - No-JS check: repo page 200, warning text present.
- G4: (d) none; (e) none; (f) none; (g) none documented.
- Final: exit at G3 (configuration docs self-declared obsolete).

### rawnly/hawk

- URL: https://github.com/rawnly/hawk
- License: no license file in repo (GitHub reports none); `Cargo.toml` declares `license = "MIT"`
- Latest release: `0.1.4` published 2023-03-12
- Archived: false
- G1 exit (category mismatch). README: "Dead simple rust CLI to ease workflows management inside monorepos";
- subcommands are `clean`, `copy` ("Copy files to the `target` directory"), `init`, `list`. It copies GitHub Actions
- workflow files from package folders into `.github/workflows`; it does not run tasks across projects. `hawk --watch`
- watches files to re-copy, with no RPC/IPC/HTTP interface.
- G2 to G3: not reached (G2 would also fail: no license file).
- G4 notes: (d) file watch re-copies workflows, not tasks; (g) generates prefixed workflow files.
- Final: exit at G1 (workflow file copier, not a task runner).

### simplebuild/please.make

- URL: https://github.com/simplebuild/please.make
- License: MIT (`LICENSE`)
- Latest release: no releases or tags; last commit 2020-09-23
- Archived: false
- G1 exit (category mismatch). README: "please.make is a minimalistic set of rules for the please build system" and
- "This repository is a boilerplate for your monorepo". It is a rule set plus template repo for Please (`.build_defs`,
- `.plzconfig`, `pleasew`); the monorepo manager is Please itself.
- G2 to G4: not reached.
- Final: exit at G1 (rule set and boilerplate for another build system).

### charypar/monobuild

- URL: https://github.com/charypar/monobuild
- License: MIT (`LICENSE`)
- Latest release: `iv2018.11.20` published 2019-12-15
- Archived: false
- G1 exit (category mismatch). README: Monobuild "understands a graph of dependencies in a monorepo codebase ... and
- based on it, it can decide what should be built". Commands `print` and `diff` output graphs and build schedules;
- running builds via a generated Makefile is marked "**not implemented**". It does not run tasks. README also says
- "this is Readme driven development. Not everything described in this readme is fully implemented."
- G2 to G4: not reached.
- Final: exit at G1 (change detection and schedule printer, does not run tasks).

### MagnusOpera/terrabuild

- URL: https://github.com/MagnusOpera/terrabuild
- License: FSL-1.1-Apache-2.0 (`LICENSE.md`; GitHub reports NOASSERTION)
- Latest release: `0.200.2` published 2026-09-15
- Archived: false
- G1 pass (monorepo manager). README: "Describe shared rules in `WORKSPACE` and project commands in `PROJECT` files ...
- Terrabuild determines their prerequisites, runs independent work concurrently".
- G2 exit (not open source). `LICENSE.md` is the Functional Source License 1.1: use is granted for any "Permitted
- Purpose", defined as "any purpose other than a Competing Use", which is a field-of-use restriction incompatible with
- OSI definitions.
  Each version gains an Apache-2.0 grant only "effective on the second anniversary of the date we make the Software
  available".
  README adds that the Terrabuild product "is distributed under our commercial terms" and the hosted Insights service
  is separate.
- G3 to G4: not reached.
- Final: exit at G2 (source-available FSL, not OSI open source for current versions).

### leostera/warp

- URL: https://github.com/leostera/warp (the `warp-build/warp` URL redirects here)
- License: MPL-2.0 (`LICENSE`)
- Latest release: `0.0.77` prerelease published 2023-04-06; newest tag `0.0.78` (2023-04-13)
- Archived: false
- G1 pass (monorepo manager, build system class). README: "a friendly, fast, correct, and extensible build system built
- for polyglot monorepos".
- G2 pass: MPL-2.0 license file.
- G3 exit (coverage and confusion).
  - Docs site does not load: `https://docs.warp.build/` fails DNS
  - (`curl: (6) Could not resolve host: docs.warp.build`); `https://warp.build/` resolves but HTTPS timed out after 20
  - s.
  - Repo docs (`docs/docs`) are title-only stubs:
  - https://github.com/leostera/warp/blob/main/docs/docs/advanced/custom-tricorders.md contains only "# Custom
  - Tricorders"; `advanced/writing-rules.md` only "# Custom Rules"; `basics/building.md` only "# Building with Warp";
  - `basics/concepts.md` only headings.
  - (a) no configuration reference, (b) no CLI reference, (c) extension pages are empty.
  - No-JS check: blob page 200, stub text present.
- G4: (d) to (g) none documented.
- Final: exit at G3 (docs site unreachable; extension and usage pages are empty stubs).

### zifeo/whiz

- URL: https://github.com/zifeo/whiz
- License: MPL-2.0 (`LICENSE`)
- Latest release: `v0.5.0` published 2023-08-10 (last push 2026-02-01)
- Archived: false
- G1 exit (category mismatch). README: "Whiz is a modern DAG/tasks runner for multi-platform monorepos. It provides
- convenient live reloading, env management, pipes, and more in a tabbed view." Configuration is a flat list of tasks
- with `workdir`, `watch`, `depends_on`; there is no project or package model. Not a component: the watch process is a
- TUI with key bindings (`r` reruns a tab) and no documented RPC/IPC/HTTP interface.
- G2 to G3: not reached.
- G4 notes: (d) yes, tasks reload on watched file change and dependents reload; (e)/(f) none documented outside the
- TUI; (g) none.
- Final: exit at G1 (task runner without a project model).

### rnza0u/blaze

- URL: https://github.com/rnza0u/blaze
- License: MIT (`LICENCE`)
- Latest release: no GitHub releases; newest tag `0.2.16` (2024-09-21), last commit 2025-05-09
- Archived: false
- G1 pass (monorepo manager). README: "Blaze is a task runner that is designed for monorepos"; projects and targets
- with dependencies (https://blaze-monorepo.dev/docs/guides/dependencies).
- G2 pass: MIT license file.
- G3 exit (confusion).
  - Coverage was otherwise met: configuration reference https://blaze-monorepo.dev/docs/configuration/workspace/schema,
  - CLI reference https://blaze-monorepo.dev/docs/cli/blaze and https://blaze-monorepo.dev/docs/cli/run, custom
  - executors https://blaze-monorepo.dev/docs/executors/ and https://blaze-monorepo.dev/docs/executors/languages/rust.
  - Confusion triggers on https://blaze-monorepo.dev/docs/executors/: "There are two different kinds of executors :"
  - followed by three bullets (Standard executors, Custom executors, Executor resolvers); the `std:exec` executor is
  - linked at `/docs/executors/std/script`.
    In the sidebar, the "CLI reference" category and its child "Global options" both link to `/docs/cli/blaze`.
    Plain curl confirmed (200; excerpt: "There are two different kinds of executors : Standard executors Custom
    executors Executor resolvers").
  - Every page carries the banner "Blaze is still very much under development and will release in alpha soon."
- G4: (d) none; (e) none; (f) none; (g) none documented (`init` command exists in CLI reference).
- Final: exit at G3 (confusion: executors page and CLI nav contradictions).

### Financial-Times/athloi

- URL: https://github.com/Financial-Times/athloi
- License: no license file in repo (GitHub reports none); `package.json` declares `"license": "ISC"`
- Latest release: `v3.0.0-beta.3` published 2023-04-20
- Archived: true
- G1 pass (monorepo manager). README: "Capable of running tasks serially or in parallel whilst preserving topological
- sort order between cross-dependent packages."
- G2 exit: repo root holds `Makefile`, `package.json`, `readme.md`, `renovate.json`, `secret-squirrel.js`, `src`,
- `test` and no license file; only the `package.json` field declares ISC.
- G3 to G4: not reached.
- Final: exit at G2 (no license file in repo).

### electricitymaps/brick

- URL: https://github.com/electricitymaps/brick
- License: none (no license file; `setup.py` and README declare none)
- Latest release: no releases or tags; last commit 2023-02-01
- Archived: true
- G1 pass (monorepo manager). README: "`brick` automatically detects dependencies by searching for inputs intersecting
- outputs of another build, and triggers the apprioriate build dependencies as needed."
- G2 exit: no license anywhere in the repo.
- G3 to G4: not reached.
- Final: exit at G2 (no license).

### abuob/yanice

- URL: https://github.com/abuob/yanice
- License: MIT (`LICENSE`)
- Latest release: `v3.8.0` published 2026-01-11
- Archived: false
- G1 pass (monorepo manager). README: "lets you define various dependency graphs for different 'scopes' ... to model
- the dependencies between your projects, detects changes ... and lets you execute commands depending on those changes".
- G2 pass: MIT license file.
- G3 exit (confusion and coverage (c)).
  - Official docs are the README only.
  - Confusion trigger: README "Configuration" says "The complete version of the `yanice.json` used for this example can
  - be found here: example-yanice.json" but the link targets an image,
  - https://github.com/abuob/yanice/blob/main/resources/yanice-visualize-example.png.
  - (c) plugin API documented only by pointing at test fixtures: "Custom plugins are javascript-files which yanice can
  - require. See here for configuration, here for a custom (untranspiled) plugin example", linking
  - `integration-tests/test-project/yanice.json` and `integration-tests/test-project/custom-scripts/dummy-plugin.ts`.
  - What data a plugin receives and its function signature are only in source.
  - Options table says `port` is "Only relevant in combination with the `--visualize`-parameter", but `visualize` is a
  - first positional parameter, not a flag.
  - No-JS check: repo page 200, text present.
- G4: (d) none; (e) `visualize` starts a local server serving the graph, not a watch process; (f) none; (g) the
- `@yanice/import-boundaries` plugin can generate dependency graphs for `yanice.json` (README).
- Final: exit at G3 (confusion: broken config example link; plugin API only in fixtures).

### tylerbutler/trellis

- URL: https://github.com/tylerbutler/trellis
- License: MIT (`LICENSE`)
- Latest release: `v0.13.2` published 2026-09-14
- Archived: false
- G1 pass (monorepo manager). README: "A workspace CLI for Gleam monorepos ... The dependency graph (topological order,
- publish order, change impact, path-dep rewrite maps) is computed".
- G2 pass: MIT license file.
- G3 exit (coverage (c)).
  - (a) present: https://trellis.tylerbutler.com/docs/configuration/
  - (b) present: https://trellis.tylerbutler.com/docs/reference/ and https://trellis.tylerbutler.com/docs/task-running/
  - (c) missing: no extension or plugin mechanism; custom tasks are config entries. Searches of
  - `website/src/content/docs`, README, and `docs/DESIGN.md` for plugin/extension/hook found none.
  - Structure signal: the site sidebar ends with "Full README" linking out to GitHub, and design decisions such as
  - deferred watch mode live only in `docs/DESIGN.md` outside the site.
  - No-JS check: configuration and reference pages 200, text present.
- G4
  - (d) none; `docs/DESIGN.md` lists watch mode under "Deferred" and suggests `watchexec -- trellis run test`.
  - (e) none. (f) none.
  - (g) partial: `trellis init` writes the `[tools.trellis]` table; changelog rendering from fragments via minijinja
  - templates (https://trellis.tylerbutler.com/docs/changelog/); `doctor` validation.
- Final: exit at G3 (coverage (c): no extension mechanism doc).

### jotform/zenith

- URL: https://github.com/jotform/zenith
- License: no license file in repo (GitHub reports none); `package.json` declares `"license": "ISC"`
- Latest release: `v3.7.0` published 2026-08-21
- Archived: false
- G1 pass (monorepo manager). README: `zenith.json` lists `projects` and per-target `constantDependencies`;
- "Lightning-fast javascript monorepo build tool".
- G2 exit: no license file at repo root (root files include `README.md`, `CHANGELOG.md`, `package.json`, no LICENSE);
- only the `package.json` field declares ISC.
- G3 to G4: not reached.
- Final: exit at G2 (no license file in repo).

### alpha-build/alpha-build

- URL: https://github.com/alpha-build/alpha-build
- License: BSD-3-Clause (`LICENSE.txt`)
- Latest release: no releases or tags; last commit 2026-03-25
- Archived: false
- G1 exit (category mismatch). README: "`make <goal> <optional-targets>` where goal = what tools we run, targets = over
- which files we run these tools". Targets are files, directories, globs, or git diffs; there is no project model or
- project relationships.
- G2 to G4: not reached.
- Final: exit at G1 (Make-based tool runner over file targets).

### chgibb/mono-surveyor

- URL: https://github.com/chgibb/mono-surveyor
- License: MIT (`LICENSE`)
- Latest release: no releases or tags; last commit 2021-11-02
- Archived: false
- G1 pass (monorepo manager). README: dependencies detected from `pubspec.yaml` path dependencies; `run_survey` "will
- determine the minimal set of packages to survey".
- G2 pass: MIT license file.
- G3 exit (coverage (a), (c)).
  - Official docs are the README only.
  - (a) `surveys.json` described in prose with one example, no reference.
  - (b) partial CLI coverage in prose (`run_survey --survey`, `--no-just-affected`, `focus_on`).
  - (c) no extension mechanism.
  - README "Further Work" says surveys "are run serially and without regard to dependency order".
  - No-JS check: repo page 200, text present.
- G4: (d) none; (e) none; (f) none; (g) none.
- Final: exit at G3 (no configuration reference, no extension doc).

### Comcast/tsb

- URL: https://github.com/Comcast/tsb
- License: Apache-2.0 (`LICENSE`)
- Latest release: no releases or tags; last commit 2026-05-04
- Archived: false
- G1 exit (category mismatch). Repo description: "A Transitive Source Builder for managing builds across multiple
- repositories". Commands are `fetch`, `prebuild`, `build`, `update`, `cherry`, `subscribe` over a config repo of
- `repos.yml`, `patches.yml`, `docker-compose.yml`. It manages downstream patch sets across repositories rather than
- running tasks across projects in a workspace.
- G2 to G4: not reached.
- Final: exit at G1 (multi-repo patch and build tool).

### 8bitAlex/raid

- URL: https://github.com/8bitAlex/raid
- License: GPL-3.0 (`LICENSE`)
- Latest release: stable `v0.17.2` published 2026-05-15; newest prerelease `v0.18.0` published 2026-07-03
- Archived: false
- G1 exit (category mismatch). README: "Raid is built for the problem one layer up: operating a multi-repo development
- environment". Commands run per profile or per repo (`raid <command>`, `raid <repo> <command>`) with no relationships
- between projects. FAQ "Can I use Raid with a monorepo?": "Raid is optimized for multi-repo setups, but it works for a
- single repo too". Not a component: no watch process; `raid context serve` is an MCP server exposing repo state and
- commands.
- G2 to G4: not reached.
- Final: exit at G1 (multi-repo command orchestrator without project relationships).

### pnordahl/monorail

- URL: https://github.com/pnordahl/monorail
- License: MIT (`LICENSE`)
- Latest release: no GitHub releases or tags; crates.io `monorail` `3.6.0` published 2024-12-01
- Archived: false
- G1 pass (monorepo manager). README: "A graph representation of your repository, built from a list of `target` entries
- and each target's `uses` list ... A scheduler and parallel execution engine".
- G2 pass: MIT license file.
- G3 exit (confusion and coverage (c)).
  - Confusion trigger: README line 9 "See the [tutorial](#tutorial) below for a practical walkthrough" links to a
  - `#tutorial` anchor that does not exist in https://github.com/pnordahl/monorail#readme; the tutorial is the separate
  - `TUTORIAL.md`, linked later.
  - (c) missing by design: README says it composes with tools "without the need for bespoke rules or plugins", and no
  - extension mechanism is documented.
  - (a) and (b) present in README ("Config", per-feature "APIs" sections) and `Monorail.reference.js`.
  - No-JS check: repo page 200, text present.
- G4: (d) none; (e) none; (f) none; (g) `config generate` produces config (README "`config generate`").
- Final: exit at G3 (confusion: broken tutorial anchor; no extension mechanism).

### aklitzke/dors

- URL: https://github.com/aklitzke/dors
- License: Apache-2.0 and MIT (`LICENSE-APACHE`, `LICENSE-MIT`)
- Latest release: no GitHub releases; newest tag `0.0.6` (2020-04-11), last commit 2020-06-05
- Archived: false
- G1 exit (category mismatch). README: "A task runner for the rust and cargo ecosystem". It fans tasks out to Cargo
- workspace members (`run-from = "members"`, `skip-members`, `only-members`) and orders tasks via `before`/`after`, but
- never uses relationships between members.
- G2 to G4: not reached.
- Final: exit at G1 (task runner with member fan-out, no project relationships).

## Survivors

None.
