# Registry screening, chunk 3

Input: `screening-registry-chunk-3.json` (68 candidates: 35 monorepo, 33 component).
Screened 2026-09-16.

## Method and caveats

- Repository metadata (license, archived, releases, root listing, README) came from `gh api`.
  Crate metadata came from `https://crates.io/api/v1/crates/<name>` with the `monochromatic-vet-discovery` user agent.
  Package metadata came from `https://registry.npmjs.org/<pkg>`.
- Docs sites were fetched once each with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0'`.
- Repo-hosted Markdown docs were read as raw content through `gh api repos/<owner>/<repo>/contents/<path>`.
  A single probe of a rendered GitHub blob page with the required curl command
  (`https://github.com/rhyek/devtooie/blob/main/docs/control-api.md`) returned `HTTP 429` with the page "Whoa there!
  You have triggered an abuse detection mechanism."
  A later probe of a repository root page (`https://github.com/Natoandro/park`) returned `200`.
  Under G3 rule 2 this block would exit any candidate whose only docs are rendered GitHub blob pages; no candidate in
  this chunk reached that point without an earlier exit, so it changes no verdict here.
- No candidate tool was installed or executed. One published crate tarball (`haz-cli-0.2.0.crate`) was listed with
- `tar --list` to confirm a license file.

## Candidates

### garden (garden-tools, garden-gui)

- URL: https://garden-rs.gitlab.io (source https://gitlab.com/garden-rs/garden)
- License: MIT (crates.io metadata)
- Latest release: `garden-tools` 2.7.0, 2026-09-01 (crates.io)
- Archived: not reported by the public GitLab API; last activity 2026-09-01
- G1: exit (category mismatch, multi-repo manager). https://garden-rs.gitlab.io (HTTP 200): "Garden streamlines
- development workflows that involve a loosely-coupled set of multiple, independent Git trees." and "Garden is all
- about making it easy to remix and reuse libraries maintained in separate Git repositories."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, multi-repo Git tree manager)

### Chomp (chompbuild)

- URL: https://github.com/guybedford/chomp
- License: Apache-2.0 (`LICENSE`)
- Latest release: 0.3.1, 2026-07-26
- Archived: false
- G1: exit (recipe/task runner without a workspace model). README: "It provides features similar to turbo and nx but
- focuses on ease of use, *not monorepos." and "There is no first-class monorepo support in chomp" and "Cross-project
- dependencies are not currently supported."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, single-chompfile task runner, no project relationships)

### tinyrick (with macros, models)

- URL: https://github.com/mcandre/tinyrick
- License: GitHub reports NOASSERTION; crates.io metadata 0BSD (`LICENSE.md`)
- Latest release: v0.0.30, 2026-08-20
- Archived: false
- G1: exit (task framework, no workspace model). README: "tinyrick is a framework for writing development tasks as Rust
- code."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, Rust task-code framework)

### astro-run (with remote runner, protocol)

- URL: https://github.com/panghu-huang/astro-run
- License: MIT
- Latest release: no GitHub releases; crate 1.0.0, 2024-07-20
- Archived: false (last push 2024-07-20)
- G1: exit (library; CI-style workflow runner). README: "Astro Run is a highly extensible runner that can execute any
- workflow." and "Astro Run only defines the interface for Runners."; usage is a Rust library embedding a
- `jobs`/`steps` workflow string.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, workflow runner library)

### cuenv (cuenv, cuengine and family)

- URL: https://github.com/cuenv/cuenv (docs https://cuenv.dev)
- License: AGPL-3.0 (`license.md`)
- Latest release: GitHub 0.55.0, 2026-07-26 (crate `cuenv` 0.40.6, 2026-04-21)
- Archived: false
- G1: pass (monorepo manager). https://cuenv.dev/tutorials/monorepo/ shows a CUE module with `services/api` and
- `services/web` projects; https://cuenv.dev/decisions/adrs/adr-0007-cross-project-task-dependencies/: "allow tasks in
- one project of a Git monorepo to depend on and consume outputs from tasks defined in other projects".
- G2: pass. `license.md`, AGPL-3.0 (OSI).
- G3: exit (confusion, can't tell where a consumed feature is documented; internal notes in user docs).
  - https://cuenv.dev/tutorials/monorepo/ (HTTP 200) only runs `cuenv task dev -p services/api` and
  - `cuenv task dev -p services/web` separately; https://cuenv.dev/how-to/run-tasks/ (HTTP 200) has no cross-project
  - section.
    The only description of cross-project dependencies (`externalInputs`) found is ADR-0007, which also says "Generated
    JSON schema has been regenerated from Rust types; future iteration will align serde renames."
  - https://cuenv.dev/explanation/cuenv-workspaces/ (HTTP 200) mixes test-implementation notes into user docs, for
  - example "Cargo lockfile parser integration tests return Result and use named entry lookup/error helpers so parser
  - assertions stay explicit without file-level unwrap_used or expect_used allowances."
    The "Explanation" navigation mixes crate internals (`cuenv-codeowners`, `cuenv-events`), "Cuetty gap analysis", and
    "Roadmap".
- G4: none (not a survivor)
- Final: exit (G3 confusion)

### engage

- URL: https://or.computer.surgery/charles/engage (redirects to https://gitlab.computer.surgery/charles/engage)
- License: crates.io metadata MIT OR Apache-2.0; GitLab API `license: null`
- Latest release: v0.2.0, 2023-09-20 (GitLab releases API); crate `engage` 0.0.0 is yanked
- Archived: not reported by the GitLab API; last activity 2026-03-30
- G1: exit (process composer without a workspace model). Raw README: "Engage is a process composer with ordering and
- parallelism based on directed acyclic graphs." and "given a collection of process definitions which can include
- ordering dependencies, Engage can run those processes in the order defined".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, single-collection process composer)

### cargo-flux

- URL: https://github.com/ignition-is-go/cargo-flux
- License: MIT (`LICENSE`)
- Latest release: v0.7.4, 2026-09-13
- Archived: false
- G1: pass (monorepo manager). README: "`cargo-flux` is a workspace topology and task-planning tool for mixed-language
- repositories." It "discovers workspace packages", "resolves native in-workspace dependencies", "follows
- cross-ecosystem bridge dependencies", and "executes tasks in dependency order with `run`".
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c), no extension mechanism documentation). Official docs are the README only (`docs/` holds
- `superpowers/plans` and `superpowers/specs`; crates.io `documentation` is null).
  README "Plugin Architecture" only lists "Current built-in plugins: Cargo plugin, JS plugin, uv plugin" and what each
  owns; it documents no way to write, register, or load a plugin.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### fern (fern-run)

- URL: https://github.com/felipesere/fern
- License: MIT
- Latest release: no GitHub releases; crate 0.0.2, 2020-03-23
- Archived: false (last push 2021-11-29)
- G1: exit (command runner without project relationships). README: "There is no way to describe interdependencies or
- any intricate ordering between files." and Anti-feature "Dependency ordering and tracking: `fern` us just meant to
- run commands."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, per-directory command runner with no relationships)

### fyrer

- URL: https://github.com/07calc/fyrer (docs https://fyrer.pages.dev)
- License: MIT (`LICENSE`)
- Latest release: v0.5.0, 2026-08-23
- Archived: false
- G1: pass (monorepo manager). README: "`fyrer` reads a `fyrer.yml` file describing the packages in a monorepo and
- their tasks, resolves the dependency graph between tasks"; `depends_on` accepts `package:task`.
- G2: pass. `LICENSE`, MIT.
- G3: exit (confusion, then coverage (c)).
  - Confusion: README "Notes and limitations" says tasks start "up to the `concurrency` limit", but neither the README
  - Configuration block nor the "Task options" table defines any `concurrency` key or flag. The same Configuration
  - block annotates `cache: false` with "allow skipping when inputs haven't changed".
  - Coverage (c): https://fyrer.pages.dev (HTTP 200, text present) states "No plugins • no adapters • just shell
  - commands".
- G4: none (not a survivor)
- Final: exit (G3 confusion; also coverage (c))

### compi

- URL: https://github.com/allyedge/compi
- License: MIT
- Latest release: v0.5.4, 2026-07-08
- Archived: false
- G1: exit (single-file build/task runner, no workspace model). README: "A build system written in Rust." and "Create a
- `compi.toml` in your project root."; tasks and `dependencies` are task IDs in that one file.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### heron-rebuild

- URL: https://github.com/heronsounds/heron-rebuild
- License: MPL-2.0 (`LICENSE.txt`)
- Latest release: tag 0.2.0; crate 0.2.0, 2024-10-04
- Archived: false (last push 2024-10-04)
- G1: exit (workflow runner, no workspace model). README: "It's a bunch of bash snippets with dependencies that can be
- run as a single workflow." and "It's not *really* a build system".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### haz (haz-cli and family)

- URL: https://forge.cloudsling.dev/andreu/haz-cli
- License: AGPL-3.0-or-later (crates.io metadata; `LICENSE.md` inside the published crate)
- Latest release: `haz-cli` 0.2.0, 2026-06-05 (crates.io)
- Archived: unknown (forge unreadable)
- G1: pass (monorepo manager). crates.io README: "Tracks task DAGs across a workspace of projects, executes them with
- caching, and supports queries over the graph."
- G2: pass. `haz-cli-0.2.0/LICENSE.md` present in the published crate; crate metadata AGPL-3.0-or-later.
- G3: exit (docs unreadable without JavaScript, bot challenge). The crate README says "The normative specification
- lives at /docs/spec" in the forge repository.
  `curl` of https://forge.cloudsling.dev/andreu/haz-cli returned HTTP 200 with an Anubis challenge: "Making sure you're
  not a bot! ... Sadly, you must enable JavaScript to get past this challenge."
- G4: none (not a survivor)
- Final: exit (G3 bot challenge on first request)

### monorepo-meta

- URL: https://github.com/wolven-tech/rust-v1 (tool docs in `tooling/meta/README.md`)
- License: MIT (`LICENSE.md`)
- Latest release: crate 0.7.2, 2026-04-17 (GitHub release v0.7.1, 2026-03-16)
- Archived: false
- G1: exit (wrapper that routes to other tools; no project relationships). `tooling/meta/README.md`: "**Meta**
- orchestrates Turborepo, Cargo, and Bacon in tmux for polyglot monorepos." and "Smart Routing - Turborepo from root,
- Bacon/Cargo from project directory"; project tasks are
- `{ tool = "turborepo", command = "run dev --filter=@org/web" }` with no dependency fields.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, tmux wrapper over Turborepo/Cargo/Bacon)

### repoctl-runner (Rust repoctl)

- URL: https://github.com/tyrchen/repoctl
- License: MIT (`LICENSE.md`)
- Latest release: GitHub v0.5.1, 2026-06-07 (crates 0.6.2, 2026-06-07)
- Archived: false
- G1: pass (monorepo manager). README: "graph-aware control plane for functional monorepos"; `project.yaml` per
- project; `repoctl affected` and `repoctl run check --affected`.
- G2: pass. `LICENSE.md`, MIT.
- G3: exit (coverage (a) and (c)). Docs index `docs/index.md` lists only the user guide, developer guide, Chinese
- translations, and research notes.
  `docs/guides/user-guide.md` headings are Mental model, Initialize, Create projects, Validate, Boundaries, Affected,
  Run tasks, CI data, Adopt, Hygiene, Templates, Proto/IaC, Ops, AI context, Output formats, Practical workflow.
  The only mention of the manifests is "The graph is built from `repo.yaml` and every discovered `project.yaml`"; no
  field reference exists, and no plugin or extension mechanism is documented.
- G4: none (not a survivor)
- Final: exit (G3 coverage (a), (c))

### zcheck (with zcheck-core)

- URL: https://github.com/zsumz/zcheck
- License: Apache-2.0
- Latest release: v0.0.2, 2026-08-24
- Archived: false
- G1: exit (single-manifest check runner, no workspace of projects). README: "Add `zcheck.toml` at the repository root"
- and "zcheck qualifies finite repository checks. It does not replace a build system, analyze architecture, ... run dev
- servers".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### zetten (ztn)

- URL: https://docs.zetten.in (source https://github.com/amit-devb/zetten)
- License: MIT
- Latest release: v1.3.4, 2026-02-10
- Archived: false
- G1: exit (single-project task runner). README: "The High-Performance Task Runner for Python Backends." with
- configuration in "`pyproject.toml` (preferred)" or "`zetten.toml`"; tasks depend on tasks in that file only.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### kiln (kiln-core, cache, exec, runtime, cli)

- URL: https://github.com/nebucloud/kiln
- License: Apache-2.0 OR MIT (`LICENSE-APACHE`, `LICENSE-MIT`)
- Latest release: v0.1.1, 2026-05-11
- Archived: false
- G1: exit (sandboxed pipeline executor, no workspace model). README: "kiln runs declarative task pipelines inside
- Linux namespace sandboxes, with content-addressed BLAKE3 caching"; no project or workspace concept is described.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### nao (nao, nao-base, pal, recipe, engine, tui)

- URL: https://github.com/manuel-woelker/nao
- License: GitHub detects none; crates.io metadata MIT
- Latest release: v0.1.7, 2026-08-06
- Archived: false
- G1: exit (single-recipe task runner). README: "`nao` is a task runner for local development and CI." and "Run it from
- a repository root that contains `.nao/nao.kdl`".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### runkernel (with cli)

- URL: https://github.com/Adriftdev/runkernel
- License: MIT
- Latest release: no GitHub releases; crate 0.1.3, 2026-09-11
- Archived: false
- G1: exit (library-first task graph engine). README: "It is currently a local, library-first engine" and "runkernel is
- a local library, not a distributed workflow system."; tasks are Rust `Pipeline` code with no workspace model.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, library)

### winch

- URL: https://codeberg.org/esavier/winch
- License: Apache-2.0 (`LICENSE`)
- Latest release: crate 0.1.6, 2026-06-25
- Archived: false (Codeberg API)
- G1: exit (Rust CI command planner, not project-relationship task running). README (Codeberg page HTTP 200): "winch
- runs typed Rust CI tasks from a workspace-owned winch.toml" and "winch is a small planner and runner for Rust CI
- commands."; tasks are profile-ordered cargo steps with package selectors.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### guild (guild-cli)

- URL: https://github.com/sprouted-dev/guild
- License: MIT (`LICENSE`)
- Latest release: v0.1.0, 2026-03-05
- Archived: false
- G1: pass (monorepo manager). README: workspace `guild.toml` with `projects = ["apps/*", ...]`, project `depends_on`,
- and `"^build"` meaning "depends on the `build` target in all upstream dependency projects".
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c)). Official docs are the README only (root listing has no docs directory). README sections:
- Installation, Quick Start, bootstrap, `guild.toml` Format, Caching, Dependency Syntax, CLI Commands, Development. No
- extension or plugin mechanism is documented.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### buildsmith

- URL: https://github.com/ze/buildsmith (API 404; crate README clone URL is github.com/zemadeiran/buildsmith)
- License: MIT (crates.io metadata)
- Latest release: crate 0.1.0, 2026-08-05
- Archived: repository not found
- G1: exit (single-project build system). crates.io README: "A content-hashed, DAG-based build system written in Rust."
- with one `buildsmith.toml` containing `project = "myapp"` and task `deps` by task name.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### breach (breach-cli)

- URL: https://github.com/radical-beard/breach
- License: MIT OR Apache-2.0 (`LICENSE-APACHE`, `LICENSE-MIT`)
- Latest release: v0.1.0, 2026-06-07
- Archived: false
- G1: pass (monorepo manager, marginal). README: "`breach.toml` files define commands, aliases, dependencies, and
- service identity"; `steps` accept `service:command` references across service configs.
- G2: pass. Dual MIT and Apache-2.0 license files.
- G3: exit (confusion, undefined config fields). `docs/config-schema.md` (the only docs file) shows
- `[service] dependencies = ["intermediary", "opencode"]` and `mode = "runnable"` in the service example, but "Command
- fields" and "Resolution behavior" never define `dependencies` or `mode`, so the effect of declared dependencies on
- execution cannot be determined.
- G4: none (not a survivor)
- Final: exit (G3 confusion)

### now-runner

- URL: https://now.dev.br (source https://github.com/EpicEric/now)
- License: AGPL-3.0
- Latest release: tag v0.3.0; crate 0.3.0, 2026-08-06
- Archived: false
- G1: exit (CI-style distributed job runner, no workspace model). README: "Nix-based distributed command runner." with
- `now.nix` `jobs` of `steps` ("Jobs are a sequence of steps run on a single machine").
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### tracel-xtask (with cli, utils)

- URL: https://github.com/tracel-ai/xtask
- License: MIT OR Apache-2.0
- Latest release: v5.3.0, 2026-09-04
- Archived: false
- G1: exit (command library for an xtask binary). README: "A collection of easy-to-use and extensible commands to be
- used in your xtask CLI based on clap." Subrepos are separate Cargo workspaces; no relationship-driven task running is
- described.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, library)

### anda (Andaman, with andax, anda-config)

- URL: https://github.com/FyraLabs/anda
- License: MIT (`LICENSE.md`)
- Latest release: 0.8.10, 2026-09-08
- Archived: false
- G1: exit (package build frontend). README: "Andaman is simply a meta-build system that calls upon other build systems
- to build and distribute packages." Features: "Building RPMs", "Building Docker/Podman/OCI images", "Building Flatpak
- packages".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### cargo-mono

- URL: https://github.com/delinoio/oss (`crates/cargo-mono`)
- License: MIT (crates.io metadata); repository license NOASSERTION
- Latest release: crate 0.6.9, 2026-05-25
- Archived: false
- G1: exit (release tool only). `crates/cargo-mono/README.md` commands are `list`, `changed`, `bump`, `publish`;
- "`cargo mono publish` always delegates to `cargo publish --no-verify`". No task running.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### orcs

- URL: https://github.com/nmoutschen/orcs
- License: crates.io metadata MIT; no license file in repository
- Latest release: crate 0.0.8, 2020-12-31
- Archived: false (last push 2021-01-12)
- G1: provisional pass on registry text only. No README exists (GitHub readme API 404); crate description
- "Microservices monorepo orchestration tool".
- G2: exit. Repository root contains only `.gitignore`, `Cargo.toml`, `src/`; no license file.
- G3: not evaluated
- G4: none
- Final: exit (G2, no license file)

### esteem

- URL: https://github.com/IgnisDa/developrs (`apps/esteem`)
- License: MIT
- Latest release: esteem-v1.1.8, 2022-08-01
- Archived: true
- G1: exit (add-on of another manager). `apps/esteem/README.md`: "Make your NX workspaces go easier on your disk." and
- "It just keeps track of individual project's dependencies." Repository README marks it "[unmaintained]".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, Nx add-on)

### hammer (hammer-cli)

- URL: https://crates.io/crates/hammer-cli
- License: MIT OR Apache-2.0 (crates.io metadata)
- Latest release: 0.3.1, 2023-03-05
- Archived: no repository link
- G1: exit (script fan-out without project relationships). crates.io README: "hammer dev: Runs all the workspaces
- projects "hammer:dev" scripts." with `--filter` by name; no dependency ordering or project relationships are
- described. (G2 would also fail: crate has no repository link.)
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### shiv (shivr)

- URL: https://github.com/xtenduke/shiv (redirects to xtenduke/shivr)
- License: MIT (GitHub); crates.io "non-standard"
- Latest release: v0.1.5, 2023-12-14
- Archived: true
- G1: exit (command fan-out without project relationships). README: "Lightweight command runner for monorepos. Can run
- commands on only packages changed against defined root branch"; per-package `shiv.json` scripts, no dependency
- ordering.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### affected

- URL: https://github.com/DenysVuika/affected (API 404)
- License: Apache-2.0 (crates.io metadata)
- Latest release: crate 0.1.8, 2024-11-24
- Archived: repository not found
- G1: exit (affected-file command runner). crates.io README: "A tool to find affected files or projects in a git
- repository and run commands on them."; tasks are file `patterns` with a `{files}` placeholder, not tasks across
- projects via relationships.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### superepo

- URL: https://github.com/GrandEngineering/superepo (redirects to voltaero/superepo)
- License: crates.io metadata GPL-2.0-only; no license file in repository
- Latest release: v0.2.0a, 2024-12-16
- Archived: false
- G1: exit (command aliases, no relationships). README config lists `[[monorepo.libs]]`, `[[monorepo.bins]]`,
- `[[monorepo.macros]]` each with `build`/`run` commands and no dependency fields;
- `superepo build <binary_name/lib_name>` runs that entry's command.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### cargo-tangerine

- URL: https://github.com/alxolr/cargo-tangerine
- License: crates.io metadata MIT; no license file in repository
- Latest release: crate 0.1.5, 2026-08-28
- Archived: false
- G1: exit (publish tool only). README: "A cargo subcomand to handle workspaces and publish only the changed crates in
- the right order." Usage is `cargo tangerine publish`.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### cargo-monorepo

- URL: https://github.com/legion-labs/cargo-monorepo
- License: crates.io metadata MIT OR Apache-2.0; no license file in repository
- Latest release: v0.1.0, 2021-11-30 (crate 0.2.0, 2021-12-01)
- Archived: true
- G1: exit (artifact packaging tool). README: "Builds distributable artifacts from cargo crates in various forms."
- supporting "AWS Lambda packages as well as a subset of Docker images".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### Watchman

- URL: https://github.com/facebook/watchman (docs https://facebook.github.io/watchman/)
- License: MIT (`LICENSE`)
- Latest release: v2026.09.14.00, 2026-09-14
- Archived: false
- G1: pass (component). https://facebook.github.io/watchman/docs/cmd/trigger (HTTP 200): "Triggered processes are
- spawned by the Watchman server process that runs in the background"; docs include `socket-interface` and `subscribe`.
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c)). Docs pages are readable without JavaScript (install and trigger pages returned text). The
- full sitemap (https://facebook.github.io/watchman/sitemap.xml) lists install, release notes, CLI options,
- capabilities, commands, expression terms, queries, BSER, socket interface, config, clients, `watchman-make`,
- `watchman-wait`, troubleshooting, and contributing; no extension or plugin mechanism page exists.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### dev-process-manager

- URL: https://dev-process-manager.com (source https://github.com/vivid-planet/dev-process-manager)
- License: BSD-2-Clause (`LICENSE`)
- Latest release: npm 4.1.0, 2026-09-16
- Archived: false
- G1: pass (component). README: "Start Daemon: Starts the dev-pm daemon, usually done automatically by other
- commands."; `start`/`stop`/`restart`/`status`/`logs` act on processes owned by that daemon.
- G2: pass. `LICENSE`, BSD-2-Clause.
- G3: exit (coverage (c)). https://dev-process-manager.com/ (HTTP 200) and
- https://dev-process-manager.com/docs/commands (HTTP 200) are readable. Docs navigation: Introduction, Use Cases,
- Comparison, Guides, Getting Started, Configuration, Commands, Examples. No extension or plugin mechanism is
- documented, and the CLI-to-daemon channel is not documented as an interface.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### braid (@aip-tech/braid)

- URL: https://github.com/aip-tech/braid (docs https://aip-tech.github.io/braid/)
- License: MIT (`LICENSE`)
- Latest release: npm 0.8.0, 2026-09-15 (tags only on GitHub, latest v0.2.5)
- Archived: false
- G1: pass (component). `packages/braid/README.md`: "Runs multiple long-lived processes as one unit, as a background
- daemon by default" with "watch-triggered restarts" and "`start` runs a loopback-only, token-guarded control server
- that plugins register routes on."
- G2: pass. `LICENSE`, MIT.
- G3: exit (confusion, contradictory text and reliance on source).
  - `packages/braid/README.md` "Plugins": "No external plugins ship yet." versus https://aip-tech.github.io/braid/
  - (HTTP 200): "`@aip-tech/braid-plugin-ui` adds a web UI as an opt-in plugin."
  - The README defers the configuration reference to source: "Full field list and defaults: [`src/types.ts`]" and the
  - plugin API to "`PluginContext` in `src/types.ts`".
- G4: none (not a survivor)
- Final: exit (G3 confusion)

### baton-run (Baton)

- URL: https://kanumuri9593.github.io/Baton/ (source https://github.com/kanumuri9593/Baton)
- License: MIT (`LICENSE`)
- Latest release: v0.2.6, 2026-09-09
- Archived: false
- G1: pass (component). Site (HTTP 200): "One daemon on loopback. The app, the CLI, and baton-mcp all authenticate with
- a token in ~/.baton/daemon.json"; MCP tools include `run_target`, `hot_reload`, `stop_session`, `read_logs`.
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c)). Docs are the marketing page plus repo Markdown (`docs/launch-guide.md`,
- `docs/app-settings.md`, `docs/architecture.md`, `AGENTS.md`). `docs/architecture.md` lists "Adapters |
- `src/adapters/` | Framework-specific start/reload/restart behavior", README says "contributions extending them are
- very welcome, see CONTRIBUTING.md", and the site lists missing adapters under "Feature requests". No adapter or
- plugin mechanism is documented for users.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### lerna-watch

- URL: https://github.com/mattstyles/lerna-watch
- License: MIT (`license`)
- Latest release: npm 1.0.0, 2020-06-30
- Archived: false (last push 2023-02-11)
- G1: exit (add-on of another manager). README: "`lerna-watch` is a complementary package that builds on lerna tooling
- to help set up a development environment for a package within your monorepo."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, Lerna add-on)

### @fervon/launchpad

- URL: https://fervon.dev/launchpad/ (source https://github.com/JoniMartin27/launchpad)
- License: MIT
- Latest release: npm 1.3.0, 2026-08-17
- Archived: false
- G1: pass (component, marginal). npm README: "Binds `127.0.0.1` (HTTP + WebSocket)", "`npm start` # serve UI + API +
- WS", "Live re-scan ... Watching is filtered to the manifests that matter", and
- `curl -X POST localhost:7777/api/batch/start`.
- G2: pass. MIT (GitHub license detection).
- G3: exit (coverage (c), (e), (f); naming confusion). https://fervon.dev/launchpad/ (HTTP 200) is a Spanish marketing
- page linking `SPEC.md`. The README (titled "Mission Control", with `MISSION_CONTROL_*` variables, for a package named
- launchpad) has sections Why, Features, Quick start, How it works, Configuration, Requirements, Security posture,
- Contributing, License. The HTTP API is claimed through a single curl example with no API reference, and no extension
- mechanism is documented.
- G4: none (not a survivor)
- Final: exit (G3 coverage)

### devtooie

- URL: https://github.com/rhyek/devtooie
- License: MIT (`LICENSE`)
- Latest release: 0.7.1, 2026-08-17
- Archived: false
- G1: pass (monorepo manager and component). README: "devtooie resolves build-time, dev-time, and runtime dependencies
- between them, builds whatever needs building (in the right order)" and "A localhost HTTP API drives a running session
- headlessly".
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c); docs page blocked). Repo docs (`docs/configuration.md`, `docs/cli.md`, `docs/control-api.md`,
- `docs/logging.md`, `docs/package-lifecycle.md`, read through `gh api`) contain no plugin or extension mechanism
- (search for plugin, extension, hook found only unrelated uses of "extend"). The required curl of
- https://github.com/rhyek/devtooie/blob/main/docs/control-api.md returned HTTP 429 "Whoa there! You have triggered an
- abuse detection mechanism."
- G4: none (not a survivor)
- Final: exit (G3 coverage (c); also rule 2 block)

### @vonage/monowatch

- URL: https://github.com/Vonage/monowatch (API 404)
- License: MIT (npm metadata)
- Latest release: npm 0.0.7, 2021-05-18
- Archived: repository not found
- G1: provisional pass (manager-like watcher). npm README: "Monowatch is a basic file watcher/npm runner made
- especially for use with monorepos"; it detects "Yarn workspaces and Lerna packages" and "Runs dependant packages
- builds concurrently."
- G2: exit (no public source repository). npm README: "This package is only available through our private Github
- repository atm."; `gh api repos/Vonage/monowatch` returned 404.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### @18ways/monorunner

- URL: https://github.com/ways-labs/18ways (API 404)
- License: MIT (npm metadata)
- Latest release: npm 0.1.0, 2026-09-04
- Archived: repository not found
- G1: exit (interactive terminal runner without an RPC/IPC/HTTP interface or project relationships). npm README:
- "`monorunner` is an interactive local development runner for repositories with several services"; `dev.json` `modes`
- are plain app lists; no HTTP, socket, IPC, or daemon interface is documented.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### runny (@polymech/runny)

- URL: https://github.com/polymech-info/runny (fork of icydotdev/runny)
- License: MIT (`LICENSE`)
- Latest release: npm 0.2.0, 2026-08-13 (tags only, latest v0.1.5)
- Archived: false
- G1: exit (dashboard wrapping package-manager scripts). README: "A visual dashboard for all your npm scripts. Zero
- config." and "Local-only: Express + WebSocket on `127.0.0.1`"; the API serves only the bundled UI and is not
- documented for other clients.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### @codepadding/coder

- URL: https://www.npmjs.com/package/@codepadding/coder
- License: MIT (npm metadata)
- Latest release: npm 1.1.0, 2026-06-10
- Archived: no repository link
- G1: exit (dashboard wrapping tmux). npm README: "`coder` starts a lightweight Node.js server that serves the
- dashboard UI and manages your dev services. Each service runs in its own named tmux session"; no API is documented.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### @funeste38/rome

- URL: https://github.com/jEFFLEZ/rome (API 404)
- License: MIT (npm metadata)
- Latest release: npm 1.5.5, 2026-03-29
- Archived: repository not found
- G1: exit (process launcher without project relationships or an interface). npm README: "runs one, two or three
- services with prefixed logs", `rome duo`/`rome trio`; "native status dashboard for detached processes" is listed
- under "Good next improvements".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### devmux (@chriscode/devmux)

- URL: https://github.com/hassoncs/devmux (docs https://devmux.pages.dev)
- License: MIT (`LICENSE`)
- Latest release: v1.10.2, 2026-06-16
- Archived: false
- G1: pass (component, marginal). README: "`devmux` uses tmux sessions with predictable naming conventions as a shared
- registry of running services"; also `devmux dashboard`, a telemetry server on `ws://127.0.0.1:9876`, and a
- "Programmatic API".
- G2: pass. `LICENSE`, MIT.
- G3: exit (confusion, contradictory text). https://devmux.pages.dev (HTTP 200): "Uses the omo-{project}-{service}
- naming convention for discoverability." versus README: "Starts API in tmux session "devmux-myapp-api"" and "Session
- naming convention" section.
- G4: none (not a survivor)
- Final: exit (G3 confusion)

### port-daddy

- URL: https://github.com/curiositech/port-daddy
- License: FSL-1.1-MIT (`LICENSE`; GitHub reports NOASSERTION)
- Latest release: GitHub v3.30.6, 2026-08-31 (npm 3.15.0, 2026-05-22)
- Archived: false
- G1: pass (component). README: "**Port Daddy** is a daemon that gives every AI agent its own port"; npm description
- includes "port-daddy up/down".
- G2: exit (source-available license). `LICENSE` first line: "Functional Source License, Version 1.1, MIT Future
- License"; "Permitted purposes means any purpose other than providing a product or service that competes with the
- Software".
- G3: not evaluated
- G4: none
- Final: exit (G2, FSL)

### veloctl

- URL: https://github.com/MahmoudGhoraba/veloctl (API 404)
- License: GPL-3.0-only (npm metadata)
- Latest release: npm 0.1.0, 2026-08-10
- Archived: repository not found
- G1: provisional pass. npm README: "veloctl is a monorepo and development-environment orchestrator. It starts every
- component of a multi-component stack ... in dependency order".
- G2: exit (no source repository). `gh api repos/MahmoudGhoraba/veloctl` returned 404.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### @khijo/devportal

- URL: https://github.com/Khijo/DevPortal (API 404)
- License: MIT (npm metadata)
- Latest release: npm 1.2.3, 2026-01-07
- Archived: repository not found
- G1: provisional pass (component). npm README: "DevPortal (browser) ←→ Backend Server (port 3100) → Your Apps (ports
- 3000+)"; "UI updates in real-time via WebSocket".
- G2: exit (no source repository). `gh api repos/Khijo/DevPortal` returned 404.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### @gbdx/devis

- URL: https://www.npmjs.com/package/@gbdx/devis
- License: AGPL-3.0-or-later (npm metadata)
- Latest release: npm 1.0.3, 2026-09-15
- Archived: no repository link
- G1: provisional pass on registry description only (npm README is empty). Description: "Local development dashboard
- for pnpm monorepos, workspace scanning, process management, HTTPS proxy, Docker, and more."
- G2: exit (no source repository). npm metadata has no `repository` field and the registry README is empty.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### grove (@theagileengineer/grove)

- URL: https://www.npmjs.com/package/@theagileengineer/grove
- License: MIT (npm metadata)
- Latest release: npm 1.0.0-beta.0, 2026-07-28
- Archived: no repository link
- G1: provisional pass (component). npm README: "A local dashboard for starting, stopping, and monitoring dev servers
- across multiple projects"; architecture table: "`server.js` | HTTP server, SSE broadcaster, process manager".
- G2: exit (no source repository). No `repository` field in npm metadata and no repository link in the README.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### bun-wtui

- URL: https://github.com/Zerodayu/bun-wtui
- License: MIT
- Latest release: npm 1.1.0, 2026-02-23
- Archived: false
- G1: exit (TUI dashboard). README: "Bun-Wtui is not a task graph engine. It does not cache builds. It does not
- optimize pipelines. It does not manage dependencies." It runs `bun run dev` in the selected workspace.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### helix (@stellix-agency/helix)

- URL: https://github.com/Stellix-Agency/Helix
- License: MIT
- Latest release: npm 1.0.5, 2026-07-13 (GitHub release v1.0.1, 2026-06-27)
- Archived: false
- G1: exit (TUI wrapping a package manager). README: "A terminal UI for monitoring processes." and "Each process runs
- as `runner run --filter filter script`."
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### monoboard

- URL: https://www.npmjs.com/package/monoboard
- License: ISC (npm metadata)
- Latest release: npm 0.1.0, 2026-06-23
- Archived: no repository link
- G1: pass (component). npm README: "The backend serves the API, WebSocket, and the built UI all on a single port."
- with an "API" section.
- G2: exit (no source repository). No `repository` field in npm metadata and no repository link in the README.
- G3: not evaluated
- G4: none
- Final: exit (G2)

### monotui (@factorim/monotui)

- URL: https://github.com/factorim/monotui
- License: MIT (GitHub); npm metadata ISC
- Latest release: v0.6.1, 2026-03-10
- Archived: false
- G1: exit (TUI dashboard). README: "MonoTUI is a **Text User Interface (TUI)** dashboard for managing
- **Node monorepos**." Features: start/stop services and real-time status; no interface or relationships.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### stackwake

- URL: https://github.com/tofu-dev0123/Stackwake
- License: MIT
- Latest release: npm 0.1.1, 2026-07-19 (tag v0.1.1)
- Archived: false
- G1: exit (multi-repo manager). README: "a polyrepo-aware, language-agnostic CLI that starts, syncs, and inspects
- multiple repositories at once" and "Think "Procfile, but for polyrepos.""
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### Turbo Stoplight (@turbo-stoplight/server, vite-plugin)

- URL: https://github.com/alexandreh92/turbo-stoplight
- License: MIT (`packages/server/LICENSE`; no root license)
- Latest release: v0.2.1, 2025-03-17
- Archived: false
- G1: exit (add-on of another manager). README: "A lightweight toolkit for managing persistent Turborepo pipelines in
- monorepos." and "Built specifically for Turborepo workflows".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1, Turborepo add-on)

### pnpm-dash

- URL: https://github.com/artygus/pnpm-dash
- License: Unlicense
- Latest release: v0.1.4, 2026-02-08
- Archived: false
- G1: exit (TUI wrapping pnpm). README: "pnpm-dash is a terminal-based user interface (TUI) for pnpm run command".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### clier-ai (clier)

- URL: https://github.com/somersstack/clier
- License: MIT (`LICENSE`)
- Latest release: npm 1.5.0, 2026-02-19 (no GitHub releases or tags)
- Archived: false
- G1: pass (component). README: "Clier runs a background daemon that manages your processes over a Unix socket. Any
- terminal, script, or AI agent session can connect to it."
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (c)). Repo docs (`docs/configuration.md`, `docs/api-reference.md`, `docs/GETTING-STARTED.md`, read
- through `gh api`) cover config, a TypeScript API, and CLI commands, but contain no plugin or extension mechanism. The
- Unix socket is described only as "Daemon creates Unix socket at `.clier/daemon.sock`" with no protocol reference.
- G4: none (not a survivor)
- Final: exit (G3 coverage (c))

### devsurface

- URL: https://github.com/mrfandu1/devsurface
- License: MIT
- Latest release: npm 1.2.1, 2026-07-19 (tag v1.0.0)
- Archived: false
- G1: exit (dashboard wrapping other tools' commands). README: "DevSurface is a zero-config CLI and local browser
- dashboard for understanding, configuring, and running unfamiliar repositories. It detects Node.js package scripts,
- ... Makefile / Justfile / Taskfile / Deno tasks". The only documented endpoint is a read-only passport at
- `/api/workspaces/:id/passport`.
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### asciyml (@irautox/asciyml)

- URL: https://github.com/IrAutoX/asciyml
- License: MIT
- Latest release: npm 1.0.5, 2026-07-29 (GitHub release "asciyml", 2026-07-30)
- Archived: false
- G1: exit (task runner and scheduler without an RPC/IPC/HTTP interface or workspace model). README: "a lightweight,
- zero-dependency YAML task runner and process manager"; `daemon` is "Run interval tasks".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### BindPort (bindport, bindport-runner, bindport-dashboard, core, registry, adapters)

- URL: https://github.com/bindport/bindport
- License: MIT (`LICENSE`)
- Latest release: v0.8.3, 2026-08-17
- Archived: false
- G1: pass (component, marginal). README: "Local dashboard API and embedded UI for active, stopped, and stale registry
- entries, including ... service-style `start` / `status` / `stop` controls."
- G2: pass. `LICENSE`, MIT.
- G3: exit (confusion in the extension mechanism). `docs/integrations/hooks.md` (hooks are the documented extension
- point): "BindPort passes event metadata through a minimal environment" listing `BINDPORT_HOOK_EVENTS`,
- `BINDPORT_HOOK_SOURCES`, `BINDPORT_HOOK_CONTEXT`, but no page defines their value formats (search of `docs/` and
- `docs/llms-full.txt` finds only the same name lists in `hooks.md` and `daily-use/configuration.md`). The docs site is
- unpublished: `docs/project/docs-site.md` says "The docs site is a work in progress" with local mdBook preview only.
- G4: none (not a survivor)
- Final: exit (G3 confusion)

### bizi (bizi-server)

- URL: https://getbizi.dev (source https://github.com/ieedan/bizi)
- License: MIT (`LICENSE`)
- Latest release: `bizi-server` crate 0.5.1, 2026-08-06; npm `bizi` 0.7.0, 2026-04-30
- Archived: false
- G1: pass (component). README: "This will install the server and start it as a background service." with CLI clients
- `bizi run`, `bizi cancel`, `bizi stat`.
- G2: pass. `LICENSE`, MIT.
- G3: exit (coverage (a), (c), (e), (f); example text absent). https://getbizi.dev/ (HTTP 200, 523 characters of text)
- is a single install page. Step "3. Setup your task.config.json" has no config text in the returned HTML, and no
- configuration reference, server API reference, or extension mechanism exists beyond a `$schema` URL in the README.
- G4: none (not a survivor)
- Final: exit (G3 coverage)

### oxproc

- URL: https://github.com/fcoury/oxproc
- License: MIT (`LICENSE`)
- Latest release: tag v0.1.0; crate 0.1.0, 2025-10-17
- Archived: false
- G1: exit (Procfile manager controlled through PID files and signals, no RPC/IPC/HTTP interface or workspace model).
- README: daemon "writes state under `$XDG_STATE_HOME/oxproc/<project-id>/`" with "PID file: .../manager.pid"; `stop`
- "sends SIGTERM, then SIGKILL after a grace period".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### harness-canopy

- URL: https://github.com/UniverLab/harness-canopy
- License: MIT
- Latest release: v2.3.1, 2026-08-25
- Archived: false
- G1: exit (category mismatch, AI agent session orchestrator). README: "a modern, self-contained MCP (Model Context
- Protocol) server and TUI for orchestrating AI agent sessions" and "canopy orchestrates AI agent tools, it doesn't
- ship one. Before installing, you need at least one supported platform installed and authenticated".
- G2: not evaluated
- G3: not evaluated
- G4: none
- Final: exit (G1)

### park (park-cli)

- URL: https://github.com/Natoandro/park (site http://park.abela.mg/)
- License: Apache-2.0 (`LICENSE`)
- Latest release: v0.2.1, 2026-09-01
- Archived: false
- G1: pass (component). README: "On-demand per-user daemon management independent of the launching terminal" and "The
- daemon socket, lock, and PID marker are ephemeral files under `$XDG_RUNTIME_DIR/park`"; `park status`,
- `park restart`, `park wait` act from other processes.
- G2: pass. `LICENSE`, Apache-2.0.
- G3: exit (confusion, then coverage (c)). The README command reference is headed "Intended Interface", leaving shipped
- versus planned commands unclear. README "Configuration" says "The configuration format defines daemon re-exec and
- managed-process restart policies, but those config-driven CLI behaviors are still under development", while "Not Yet
- Implemented" lists "Automatic restart policies". No extension mechanism is documented.
- G4: none (not a survivor)
- Final: exit (G3 confusion; also coverage (c))

## Exits by gate

- G1 (42): garden, Chomp, tinyrick, astro-run, engage, fern, compi, heron-rebuild, monorepo-meta, zcheck, zetten, kiln,
- nao, runkernel, winch, buildsmith, now-runner, tracel-xtask, anda, cargo-mono, esteem, hammer, shiv, affected,
- superepo, cargo-tangerine, cargo-monorepo, lerna-watch, @18ways/monorunner, runny, @codepadding/coder,
- @funeste38/rome, bun-wtui, helix, monotui, stackwake, Turbo Stoplight, pnpm-dash, devsurface, asciyml, oxproc,
- harness-canopy
- G2 (8): orcs, @vonage/monowatch, veloctl, @khijo/devportal, @gbdx/devis, grove, monoboard, port-daddy
- G3 (18): cuenv, cargo-flux, fyrer, haz, repoctl, guild, breach, Watchman, dev-process-manager, braid, Baton,
- @fervon/launchpad, devtooie, devmux, clier, BindPort, bizi, park

## Survivors

- None.
