# Monorepo manager screening, chunk 3

Screened 29 candidates from `screening-chunk-3.json`, in input order.
The screening was read-only: no candidate tool was installed or run.
Metadata comes from `gh api repos/<owner>/<repo>`, its releases and tags, and the npm or crates.io registries where
GitHub has no releases.

## Rules applied

- G1 fit means the tool itself knows the workspace's set of projects or packages and uses the relationships between
- them (dependency edges, cross-project references) to drive what it runs.
  A task runner that has only a task graph inside one file, or that only fans a command out over directories, is a
  category mismatch.
- The coordinator relayed two G3 rule changes from the user mid-run.
  Both are applied to every candidate here, including those screened before the changes arrived.
  - G3 confusion: any confusion or frustration while reading the official docs causes an exit, and any recorded
  - structure signal counts as that.
  - G3 no-JS and no-bot-block: every docs page relied on was fetched with
  - `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefo
  - x/130.0' <url>`.
    A bot challenge, a block page, or an empty JS shell is an exit.
    No browser was used for any candidate.
- Every confusion trigger cited under a G3 exit was re-checked against the deployed page with that curl command, and
- each returned HTTP 200 with the quoted text present (except the systemd man pages, which are the bot-block trigger).

### nosebit/act

- URL https://github.com/nosebit/act; license GPL-3.0 (`LICENSE`); latest release v1.5.3 on 2022-05-01; archived: no
- G1: category mismatch.
  The README calls it "a task runner and supervisor tool" with "process supervision in a project level", subacts, regex
  act names and includes.
  It has no model of projects or the relationships between them.
  Its daemon mode (`act run -d`, `act list`, `act stop`) is CLI-only, with no watch and no RPC, IPC or HTTP interface,
  so it is not a component either.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### IKatsuba/runx

- URL https://github.com/IKatsuba/runx; license MIT (`LICENSE`); latest release v0.5.0 on 2025-03-11; archived: no
- G1: monorepo manager.
  The README says it "Builds and validates dependency relationships between packages", "Executes tasks in correct
  topological order", and has `--affected`.
- G2: pass (MIT `LICENSE`).
- G3: exit on coverage and confusion.
  - The only docs are the README (https://github.com/IKatsuba/runx#readme), repeated in `packages/cli/README.md`.
  - (a) No configuration reference: the tool is "Zero Configuration" and nothing documents what it reads.
  - (c) No extension or plugin mechanism is documented.
  - Confusion: the install command pins `jsr:@runx/cli@0.2.0`, while the latest release is v0.5.0.
- G4: none of (d), (e), (f) or (g) is documented.
- Final: exit at G3 (coverage: no config reference and no extension doc; confusion: stale install version).

### kristofferlind/knega

- URL https://github.com/kristofferlind/knega; license MIT (`LICENSE`); latest release v0.4.0 on 2022-11-27; archived:
- no
- G1: monorepo manager.
  It runs actions per component, and components declare dependencies on other components
  (`examples/components/with-dependency-on-example-1/.component.yml` has inputs `type: "Component"`,
  `name: "example-1"`).
- G2: pass (MIT `LICENSE`).
- G3: exit on coverage and confusion.
  - The only docs are a 1.3 KB README plus the `examples/` directory.
  - (a) No configuration reference: the README only says "Example configurations can be found in examples/".
  - (c) No extension doc.
  - Confusion: the install snippet sets `KNEGA_VERSION=0.1.0`, while the latest release is v0.4.0.
- G4: none of (d), (e), (f) or (g) is documented.
- Final: exit at G3 (coverage and confusion).

### chaliy/mrt

- URL https://github.com/chaliy/mrt; license MIT (`LICENSE`); no GitHub releases or tags (the README nonetheless points
- to prebuilt binaries on the Releases page); archived: no
- G1: category mismatch.
  The README says it "discovers packages in a polyglot monorepo and runs the same script across them".
  Its Roadmap still lists "Model package dependencies and execution order", so it does not use relationships between
  projects.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch: fan-out runner with no project relationships).

### egladman/magus

- URL https://github.com/egladman/magus; license GPL-3.0 (`LICENSE`); latest release v0.4.3 on 2026-09-06; archived: no
- G1: monorepo manager.
  The README says "magus keeps a dependency graph of your projects" and "`magus affected <target>` runs only the
  projects that change can reach, in dependency order".
- G2: pass (GPL-3.0 `LICENSE`).
- G3: exit on confusion. Coverage alone would have passed.
  - Coverage evidence:
    - (a) https://eli.gladman.cc/magus/reference/config/
    - (b) https://eli.gladman.cc/magus/reference/manpage/magus-run/
    - (c) https://eli.gladman.cc/magus/guides/authoring-spells/
    - (d) https://eli.gladman.cc/magus/reference/manpage/magus-watch/
    - (e) https://eli.gladman.cc/magus/reference/api/ and https://eli.gladman.cc/magus/guides/integrations/daemon/
    - (f) https://eli.gladman.cc/magus/guides/integrations/mcp/
  - Triggers, in the order they were read:
    - https://eli.gladman.cc/magus/reference/api/ lists each service with only a "Methods" count and no description
    - (count-only index).
    - Contradiction on https://eli.gladman.cc/magus/guides/integrations/daemon/: it says "The daemon arbitrates that
    - budget; it does not run your work".
      The same page calls the socket "the proc RPC that dispatches jobs" and "the socket the daemon dispatches build
      jobs on".
    - https://eli.gladman.cc/magus/reference/manpage/magus-status/ disagrees with the daemon guide in two places:
      - `--socket` defaults to "auto-detect from MAGUS_DAEMON_SOCKET or scan sock dir", but the guide names
      - `MAGUS_DAEMON_ADDRESS` and a fixed socket name that needs "no discovery".
      - `--probe` is described as "liveness or readiness", but the guide also documents `mcp` and comma-combined kinds.
    - https://eli.gladman.cc/magus/documentation/ is headed "Generated man pages for every command:" but lists 7 of the
    - 33 man pages (truncated summary).
    - The config reference (https://eli.gladman.cc/magus/reference/config/) gives only key, environment variable, flag
    - and type, with no meanings or defaults.
      For example, the `daemon.idle_ttl` ten-minute behavior is described only in the daemon guide.
- G4:
  - (d) `magus watch | magus affected --stdin build` (magus-watch man page).
  - (e) Daemon Connect/gRPC API with StatusService and StreamStatus (https://eli.gladman.cc/magus/reference/api/), plus
  - `magus status` over the unix socket.
  - (f) MCP tools `magus_run_target` and `magus_run_affected` (https://eli.gladman.cc/magus/guides/integrations/mcp/).
    JobService RunJob triggers only maintenance jobs, and no documented cancel was found.
  - (g) Declared outputs, the `format:rw` charm and generated-file classification
  - (https://eli.gladman.cc/magus/concepts/generated-files/); `magus init` scaffolds `magus.yaml` and `magusfile.buzz`.
- Final: exit at G3 (confusion: count-only API index, contradictory daemon and status pages).

### Grevix/Rivox

- URL https://github.com/Grevix/Rivox; license dual MIT or Apache-2.0 (the single `LICENSE` holds the MIT text and
- refers to `LICENSE-APACHE` and `LICENSE-MIT` files that are not in the repo root); no GitHub releases; crates.io
- 1.0.0 on 2026-08-10; archived: no
- G1: monorepo manager, build-only.
  The README says it constructs "a unified multigraph" from the ecosystems declared in `rivox.toml` plus
  `[[cross_refs]]` consumer and dependency edges.
- G2: pass (an OSI license text is in the repo).
- G3: exit on coverage and confusion.
  - (a) The config reference `docs/configuration.md` (https://github.com/Grevix/Rivox/blob/main/docs/configuration.md)
  - is a single example with no key semantics.
    A second, differing copy exists as `CONFIGURATION.md`.
  - (c) No extension doc exists.
    The `EcosystemAdapter` trait is mentioned only as a source path in `docs/internals.md`.
  - Contradiction: `docs/cli.md` says "Rivox V1 provides four explicit CLI commands", but the README "CLI Reference"
  - lists nine (`graph`, `oci`, `policy`, `remote` and `benchmark` among them).
- G4: none of (d), (e), (f) or (g); the README FAQ says "Rivox does not require a daemon".
- Final: exit at G3 (coverage and confusion).

### hjosugi/frost-build

- URL https://github.com/hjosugi/frost-build; license 0BSD (`LICENSE`); latest release v0.13.1 on 2026-08-27; archived:
- no
- G1: monorepo manager.
  It uses multi-package `//package:target` labels with `deps`, `query rdeps` and `test --affected` (README).
- G2: pass (0BSD).
- G3: exit on coverage and confusion.
  - (b) No CLI reference in the official docs.
    Man pages ship only inside release archives, and the command surface exists only as the test fixture
    `crates/frostbuild-cli/tests/cli-surface.txt`.
  - (c) No extension or plugin doc.
    `docs/10_language_adapters.md` is the manifest-level `kind = "command"` target and is filed under "Research and
    design studies".
  - Confusion:
    - https://hjosugi.github.io/frost-build/docs/ gives a quick start pinned to `--tag v0.8.0`, while the latest
    - release is v0.13.1.
    - `docs/README.md` says numeric prefixes "are used twice (06, 09, 17)".
    - The daemon wire protocol is declared non-contract in `docs/28_compatibility_contract.md` and is not documented
    - anywhere.
- G4:
  - (d) `frost watch` and `frost dev` rebuild and restart on change (`docs/22_developer_loop.md`).
  - (e) `frost daemon status --json` only; the protocol is undocumented.
  - (f) Not documented.
  - (g) `frost init` writes the manifest and `frost ide` generates VS Code config (README).
- Final: exit at G3 (coverage: no CLI reference and no extension doc; confusion: stale versioned quick start).

### pomagrenate/gox

- URL https://github.com/pomagrenate/gox; license MIT (`LICENSE`); latest release v0.1.0 on 2026-08-12; archived: no
- G1: category mismatch.
  It is a "workflow manager for Go monorepos": scaffolding plus `gox build` and `gox test`, which run `go test ./...`
  "recursively across all workspace apps and libs".
  `gox.yaml` lists apps and libs, but no relationships between projects drive execution.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### nshkrdotcom/blitz

- URL https://github.com/nshkrdotcom/blitz; license MIT (`LICENSE`); no GitHub releases, latest tag v0.4.1 on
- 2026-07-31 (Hex package); archived: no
- G1: monorepo manager (Mix only).
  `Blitz.MixWorkspace.Impact` fingerprints "local dependency edges" and selects tasks for "reverse local dependents"
  (README and https://blitz.hexdocs.pm/downstream_integration.html).
- G2: pass (MIT).
- G3: exit on coverage and confusion (had to read source).
  - (a) README "Workspace config keys" (https://blitz.hexdocs.pm/).
  - (b) Mix task moduledocs are only usage stubs.
  - (c) No extension doc.
    https://blitz.hexdocs.pm/downstream_integration.html tells readers to "Use `command_mapper`" and "`only_projects`",
    but neither option is documented; they exist only in `lib/blitz/mix_workspace/impact.ex`.
    `run_many!/3` has no `@doc`, and the `--explain` flag of `mix blitz.workspace.impact` appears only in the source
    `OptionParser`.
- G4: none of (d), (e), (f) or (g).
- Final: exit at G3 (coverage and confusion: options defined only in source).

### Ashutosh0x/hyperblaze

- URL https://github.com/Ashutosh0x/hyperblaze; license MIT (`LICENSE`); latest release v0.1.0 on 2026-05-24; archived:
- no
- G1: monorepo manager (Bazel-like).
  `BUILD.hb` targets with explicit deps and `//:hello` labels.
- G2: pass (MIT).
- G3: exit on coverage and confusion.
  - The docs are the README plus `docs/ARCHITECTURE.md`, `DESIGN.md`, `GETTING_STARTED.md` and `RELEASE.md`.
  - (a) The config reference is one `HYPERBLAZE.toml` snippet.
  - (b) The CLI table marks `test`, `run`, `query`, `fmt` and `graph` as placeholders.
  - (c) No rule-authoring or plugin doc.
  - Contradiction: the README architecture diagram shows a `Watcher (notify)` feeding the graph, but "Current
  - Limitations" says "No file watcher daemon".
- G4: (d) planned only; (e), (f) and (g) none.
- Final: exit at G3 (coverage and confusion).

### pojntfx/dibs

- URL https://github.com/pojntfx/dibs; license AGPL-3.0 (`LICENSE`); latest release 0.0.25 on 2020-03-18; archived: yes
- G1: category mismatch.
  `dibs.yaml` defines targets and platforms with build and test commands for one project (see the root `dibs.yaml` and
  `test-app/dibs.yaml`).
  There is no workspace of projects and no relationships between them.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### chaitanya-archive/obelisk.build

- URL https://github.com/chaitanya-archive/obelisk.build; license GPL-3.0 per the `LICENSE` file (the README says
- "Apache-2.0 OR MIT"); latest release v0.1.0 on 2026-01-03; archived: yes
- G1: monorepo manager (Bazel-like).
  `cc_library` and `cc_binary` targets with `deps = [":mylib"]`, plus `obelisk query deps`.
- G2: pass (the GPL-3.0 file is present; the README license claim contradicts it).
- G3: exit on coverage and confusion.
  - The docs (`docs/ACTION_MODEL.md`, `ARCHITECTURE.md`, `CACHE_KEYS.md` and others) have no configuration reference
  - beyond a README snippet, no CLI reference beyond a README command list, and no rule or extension doc.
  - Confusion: the README badges and "Documentation" links point to `staticpayload/obelisk.build`, and its license
  - statement disagrees with `LICENSE`.
- G4: `obelisk daemon start` is listed but undocumented; (d) and (g) none.
- Final: exit at G3 (coverage and confusion).

### gmullerb/jko

- URL https://github.com/gmullerb/jko; license: `LICENSE` is a proprietary "Executable Software License (ESL)" ("The
- corresponding source code is not provided"); no GitHub releases, npm `jko` 1.0.0 on 2025-07-10; archived: no
- G1: category mismatch.
  https://jko-doc.github.io/ describes "a script execution tool that extends scripts and integrates dependencies from
  external files".
  It has no workspace or project-relationship model.
- G2: not assessed. It would fail: the local tool is binary-only and not open source.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### alpercitak/fasttrack

- URL https://github.com/alpercitak/fasttrack; license MIT (`LICENSE`); latest release v0.2.2 on 2026-05-08; archived:
- no
- G1: category mismatch.
  The README "Command Mapping" shows it only translates `--lint`, `--test` and similar flags into `nx run-many`,
  `nx affected`, `turbo run` or `pnpm run` commands.
  Nx or Turbo own the workspace model and the relationships between projects.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch: command front end over nx or turbo).

### meslzy/outdo

- URL https://github.com/meslzy/outdo; license MIT (`LICENSE`); latest release v0.0.1 on 2026-08-26; archived: no
- G1: monorepo manager.
  It merges member `do.ts` files at a Bun workspace root, and `-F ...api` and `--affected` follow "workspace dependents
  of changed packages" (README "Workspaces" and "Git-aware runs").
- G2: pass (MIT).
- G3: exit on coverage.
  - (a) and (b): https://meslzy.github.io/outdo/reference/task-api and https://meslzy.github.io/outdo/reference/cli.
  - (d) https://meslzy.github.io/outdo/guide/watch-and-services.
  - (c) No extension or plugin mechanism: https://meslzy.github.io/outdo/guide/why-outdo says "no plugin ecosystem",
  - and a grep of every docs page found no extension doc.
- G4:
  - (d) `outdo --watch` re-runs the affected subgraph in topological order.
  - (e) and (f) none; there is only a JSON run report on stdout.
  - (g) `outdo init` scaffolds `do.ts`.
- Final: exit at G3 (coverage: no extension mechanism).

### lukahartwig/mono

- URL https://github.com/lukahartwig/mono; license Apache-2.0 (`LICENSE`); no releases or tags; last push 2023-02-25;
- archived: no
- G1: category mismatch.
  The README has badges only. `module/module.go` models a module as `Name`, `Path` and `Tasks`, with no dependencies.
  The repo description is "A language agnostic task-runner designed for mono repositories".
- G2: not assessed.
- G3: not assessed (the README has no content).
- G4: not assessed.
- Final: exit at G1 (category mismatch: no relationships between projects).

### serrnovik/jax

- URL https://github.com/serrnovik/jax; license Apache-2.0 (`LICENSE`); no GitHub releases, latest tag v0.1.17 on
- 2026-08-21 (PowerShell Gallery); archived: no
- G1: category mismatch.
  The README calls it "a PowerShell 7 task runner for repository-owned environments, flows, scripts, and psake tasks".
  A grep of `docs/README.md` and `docs/CONSUMER-ONBOARDING.md` found no project or dependency model.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### albahq/alba

- URL https://github.com/albahq/alba; license MIT (`LICENSE`); latest release v0.2.0 on 2026-09-10; archived: no
- G1: category mismatch.
  The README says "Alba is a task runner. Build files are described by a Beamfile, and a beam is the unit of work… with
  optional dependencies on other beams".
  Its graph is beams in one Beamfile project, with no workspace of packages.
  Watch mode exists, but its only output is a JSON event stream on stdout, with no RPC, IPC or HTTP, so it is not a
  component.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed. Known from the README: `alba run --watch`, and executor plugins over a line-JSON protocol
- (`docs/plugin-protocol.md`).
- Final: exit at G1 (category mismatch).

### jdarais/cobble

- URL https://github.com/jdarais/cobble; license GPL-3.0 (`COPYING`); latest release v0.8.0 on 2025-07-14; archived: no
- G1: monorepo manager.
  The README says "Tasks can depend on artifacts and tasks from other projects in the workspace, allowing for the
  creation of a full-workspace task dependency graph".
- G2: pass (GPL-3.0 `COPYING`).
- G3: exit on coverage.
  - (a) https://jdarais.github.io/cobble/workspace-def/ and https://jdarais.github.io/cobble/project-def/.
  - (c) User-defined `tool` and `env` definitions (https://jdarais.github.io/cobble/concepts/).
  - (b) Missing: https://jdarais.github.io/cobble/cobl-cli/ says "More detailed documentation of the `cobl` CLI coming
  - soon" and shows only top-level `cobl --help`.
    How `cobl run` selects tasks or takes arguments appears nowhere in the docs.
- G4: none of (d), (e), (f) or (g) is documented.
- Final: exit at G3 (coverage: placeholder CLI reference).

### moon (moonrepo)

- URL https://moonrepo.dev (repo https://github.com/moonrepo/moon); license MIT (`LICENSE`); latest release v2.5.5 on
- 2026-09-15; archived: no
- G1: user exclusion (documentation), skipped.
- G2, G3 and G4: not assessed.
- Final: exit (user exclusion).

### Buck2

- URL https://buck2.build (repo https://github.com/facebook/buck2); license Apache-2.0 or MIT (`LICENSE-APACHE` and
- `LICENSE-MIT`); latest dated release 2026-09-15 (every release is marked prerelease, plus a rolling `latest`);
- archived: no
- G1: monorepo manager.
  Packages and targets form a dependency graph across cells, and target patterns such as `//...` span the workspace.
- G2: pass.
- G3: exit on coverage and confusion.
  - (b) https://buck2.build/docs/users/commands/status/ and the command pages.
  - (c) https://buck2.build/docs/rule_authors/writing_rules/.
  - (a) Incomplete: https://buck2.build/docs/concepts/buckconfig/ states "Below is an incomplete list of supported
  - buckconfigs" and documents only `[alias]` and `[cells]`.
  - Confusion: https://buck2.build/docs/concepts/daemon/ relies on the `[project].ignore` setting, which that reference
  - does not contain.
    Other config keys are scattered across concept and rule-author pages with no page tying them together.
- G4:
  - (d) No native watch-and-rerun. The daemon only "monitors the project's file system for changes" to invalidate state.
  - (e) `buck2 status` (CLI); no public RPC is documented.
  - (f) `buck2 kill` and `buck2 clean` only.
  - (g) None found.
- Final: exit at G3 (coverage: config reference self-declared incomplete; confusion: settings used but not in the
- reference).

### Rush

- URL https://rushjs.io (repo https://github.com/microsoft/rushstack); license MIT (the `LICENSE` file says each
- package carries MIT); latest release `@microsoft/rush` 5.179.0 on npm, 2026-09-05; archived: no
- G1: monorepo manager, using the project dependency graph and selection flags such as `--to-except` and
- `--impacted-by`.
- G2: pass.
- G3: exit on confusion. Coverage alone would have passed.
  - Coverage evidence:
    - (a) https://rushjs.io/pages/configs/command-line_json/
    - (b) https://rushjs.io/pages/commands/rush_build/
    - (c) https://rushjs.io/pages/extensibility/creating_plugins/
    - (d) https://rushjs.io/pages/advanced/watch_mode/
  - Triggers:
    - https://rushjs.io/pages/advanced/watch_mode/ nests the phased-command `watchOptions` instructions under the
    - heading "The "watchForChanges" setting (experimental)", and the closing note says only that "The
    - "watchForChanges" feature is still in its early stages".
      A reader cannot tell whether phased-command watch mode is experimental.
    - The watch-mode page never mentions how to observe or control a running watch.
      https://rushjs.io/pages/maintainer/using_rush_plugins/ describes `@rushstack/rush-serve-plugin` only as a plugin
      that "runs an express server to serve project outputs".
      The plugin's WebSocket build-status and build-control protocol is documented only in its package README on
      GitHub, with message shapes given as TypeScript type imports.
- G4:
  - (d) `watchOptions` plus `--watch` for phased commands.
  - (e) and (f) are not in core. The first-party rush-serve-plugin WebSocket can "execute or abort a build, pause the
  - watcher, change parallelism, invalidate operations"
  - (https://github.com/microsoft/rushstack/tree/main/rush-plugins/rush-serve-plugin).
  - (g) None found in the pages read.
- Final: exit at G3 (confusion).

### Earthly

- URL https://earthly.dev (repo https://github.com/earthly/earthly); license MPL-2.0 (`LICENSE`); latest release
- v0.8.16 on 2025-07-16; archived: no, but the README says "Earthly is no longer actively maintained"
- (https://earthly.dev/blog/shutting-down-earthfiles-cloud)
- G1: category mismatch.
  The README calls itself "a general-purpose CI/CD framework" and contrasts it with Bazel as a "build system".
  Its monorepo support is "split your build logic across multiple Earthfiles… Referencing targets from other
  Earthfiles": explicit target calls, with no model of the workspace's projects.
  Its BuildKit daemon offers no documented task RPC, so it is not a component.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### Task (go-task)

- URL https://taskfile.dev (repo https://github.com/go-task/task); license MIT (`LICENSE`); latest stable release
- v3.53.1 on 2026-08-18 (a nightly prerelease came out 2026-09-16); archived: no
- G1: category mismatch.
  It is a Make-style task runner with Taskfile includes.
  https://taskfile.dev/docs/guide mentions monorepos only as "you can `cd` into a microservice directory and run a task
  command"; there is no project model.
  `--watch` exists (https://taskfile.dev/docs/reference/cli), but there is no RPC, IPC or HTTP interface, so it is not
  a component.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch).

### pnpm workspaces and pnpm pipeline

- URL https://pnpm.io (repo https://github.com/pnpm/pnpm, docs source https://github.com/pnpm/pnpm.io); license MIT
- (`LICENSE`); latest release v12.4.2 on 2026-09-15; archived: no
- G1: monorepo manager.
  https://pnpm.io/workspace-task-orchestration: "`pnpm -r run <script>` schedules a graph of workspace tasks", with
  `^build` dependencies on workspace dependencies, and `pnpm pipeline` runs affected projects plus their dependents.
- G2: pass.
- G3: exit on confusion. Coverage alone would have passed.
  - Coverage evidence:
    - (a) https://pnpm.io/settings
    - (b) https://pnpm.io/cli/run, https://pnpm.io/cli/recursive and https://pnpm.io/cli/pipeline
    - (c) https://pnpm.io/pnpmfile and https://pnpm.io/config-dependencies (`pnpm-plugin-*`)
  - Triggers:
    - Contradiction on `--no-bail`: https://pnpm.io/cli/recursive says "Even if `--no-bail` is used, all tasks will
    - finish".
      https://pnpm.io/workspace-task-orchestration says "After a task fails, tasks that depend on it are skipped. With
      `--no-bail`, independent ready tasks continue to run".
    - Recursive-run options are split with no single place to look.
      `--resume-from` is documented only on the orchestration page, while the `pnpm -r` options page sits under the
      "Misc." CLI category.
- G4:
  - (d) No native file-watch re-run. `pnpm pipeline --watch` polls a git repository for new revisions and is described
  - as "a proof of concept" (https://pnpm.io/cli/pipeline).
  - (e) and (f) none.
  - (g) None found in the pages read.
- Final: exit at G3 (confusion: contradictory `--no-bail` semantics).

### pitchfork

- URL https://pitchfork.jdx.dev (repo https://github.com/jdx/pitchfork); license MIT (`LICENSE`); latest release
- v2.25.0 on 2026-09-11; archived: no
- G1: component.
  The README describes "A background supervisor" that tracks daemons, watches files to restart them
  (https://pitchfork.jdx.dev/guides/file-watching), and exposes an HTTP API
  (https://pitchfork.jdx.dev/reference/http-api).
- G2: pass (MIT).
- G3: exit on confusion. Coverage alone would have passed.
  - Coverage evidence:
    - (a) https://pitchfork.jdx.dev/reference/configuration
    - (b) https://pitchfork.jdx.dev/cli/
    - (c) https://pitchfork.jdx.dev/guides/lifecycle-hooks
    - (d) https://pitchfork.jdx.dev/guides/file-watching
    - (e) and (f) https://pitchfork.jdx.dev/reference/http-api
  - Triggers:
    - Log-streaming protocol contradiction: https://pitchfork.jdx.dev/guides/web-ui says "Real-time log streaming for
    - each daemon via Server-Sent Events (SSE)".
      https://pitchfork.jdx.dev/reference/http-api says `GET /api/logs/{id}/tail` streams "newline-delimited JSON
      (`Content-Type: application/x-ndjson`)".
    - The same web-ui page uses `PITCHFORK_WEB_PORT` for one-time use and `PITCHFORK_WEB_BIND_PORT` for persistent use,
    - for the same port.
      It also calls the port-attempts setting `web.port_attempts`, while its keys live under `[settings.web]`.
    - The web UI's "Config Editing" feature has no endpoint in the HTTP API reference.
- G4:
  - (d) Restarts a daemon on glob matches. It does not re-run tasks.
  - (e) `GET /api/daemons` and `/api/daemons/{id}`.
  - (f) `POST /api/daemons/{id}/start`, `stop` and `restart`.
  - (g) None.
- Final: exit at G3 (confusion).

### systemd user units

- URL https://systemd.io (repo https://github.com/systemd/systemd); license LGPL-2.1 and GPL-2.0 (`LICENSE.LGPL2.1`,
- `LICENSE.GPL2`, `LICENSES/`); latest stable-marked GitHub release v259.9 on 2026-09-11 (v262-rc3 prerelease on
- 2026-09-15); archived: no
- G1: component.
  A persistent service manager that "uses socket and D-Bus activation for starting services" (https://systemd.io/),
  with path units that watch files and a D-Bus control API.
- G2: pass.
- G3: exit on confusion and bot block.
  - Confusion: the navigation at https://systemd.io/ has no entry for user units, `.path` file watching, or the D-Bus
  - control API.
    Its "Manual Pages" section offers only the external freedesktop.org "Directives" and "Index" lists.
    Its groups mix unrelated kinds of page; for example, "Concepts" lists "Porting systemd To New Distributions" and
    "Safely Building Images".
  - Bot block: `curl … https://www.freedesktop.org/software/systemd/man/latest/` returned HTTP 418,
  - `<title>Checking you are not a bot</title>`, with a meta-refresh `challenge/meta-refresh/verify-challenge` redirect.
    `systemd.path.html`, `org.freedesktop.systemd1.html`, `systemctl.html` and `user@.service.html` returned the same
    418 challenge.
- G4: not assessed, because the manual pages could not be read.
- Final: exit at G3 (confusion; manual pages bot-blocked).

### Aspect CLI

- URL https://aspect.build (repo https://github.com/aspect-build/aspect-cli); license Apache-2.0 (`LICENSE`); latest
- release v2026.38.10 on 2026-09-16; archived: no.
  Status checks and PR comments need an `ASPECT_API_TOKEN` linked to the hosted Aspect Workflows apps; the CLI itself
  is open.
- G1: monorepo manager, layered on Bazel.
  The README calls it "a programmable task runner built on top of Bazel" that can "query the build graph" and do
  "selective delivery".
  The project model is Bazel's. It has no watch process or RPC, so it is not a component.
- G2: pass (Apache-2.0 for the local CLI; Aspect Workflows is a separate hosted product).
- G3: exit on confusion. Plain curl returns server-rendered text (HTTP 200), so the no-JS rule passes.
  - Triggers:
    - https://aspect.build/docs/axl/types, titled "Types", lists types and then the builtin function reference (`any`,
    - `chr`, `filter`, `sorted`, …).
      https://aspect.build/docs/axl/builtins, titled "Builtins", shows only "module base64 module hash module time"
      with no summaries, so a reader cannot tell where builtins are documented.
    - The docs index that every page points readers to (https://aspect.build/llms.txt, "Use this file to discover all
    - available pages") lists AXL reference pages with duplicate titles and no descriptions.
      Examples: "Args" twice, "Auth" twice, "Bazel" twice, "Build" twice, "Build event" twice, "Execution log" twice.
- G4:
  - (d) None found in the docs index.
  - (e) and (f): the index mentions an "aspect mcp build results server" on https://aspect.build/docs/cli/guides/agents
  - (not read further).
  - (g) `aspect gazelle` generates and syncs BUILD files (https://aspect.build/docs/cli/tasks/gazelle), and
  - `aspect init` scaffolds a workspace.
- Final: exit at G3 (confusion).

### microsoft/lage

- URL https://github.com/microsoft/lage (docs https://microsoft.github.io/lage/); license MIT (`LICENSE`); no GitHub
- releases, npm `lage` 2.17.0 on 2026-08-18; archived: no
- G1: monorepo manager.
  https://microsoft.github.io/lage/docs/reference/cli/: "Runs a set of commands in a target graph. The targets are
  defined by packages and their scripts", with dependency and dependent scoping.
- G2: pass (MIT).
- G3: exit on coverage and confusion.
  - (a) https://microsoft.github.io/lage/docs/reference/config/
  - (b) https://microsoft.github.io/lage/docs/reference/cli/
  - (c) "Custom reporters" on the CLI page and "Worker tasks" on https://microsoft.github.io/lage/docs/guides/pipeline/
  - (d) Missing: the tool claims `--watch  runs in watch mode` on the CLI page, and a grep of every docs page found no
  - other watch documentation.
  - Confusion: the same page lists `--no-deps|--no-dependents  disables running any dependents`, where the alias name
  - says dependencies but the text says dependents.
- G4:
  - (d) `--watch` exists but its behavior is undocumented.
  - (e) and (f) none found.
  - (g) None.
- Final: exit at G3 (coverage: watch mode undocumented; confusion: flag alias vs description).

## Exit counts

- User exclusion: 1 (moon).
- G1: 11 (nosebit/act, chaliy/mrt, pomagrenate/gox, pojntfx/dibs, gmullerb/jko, alpercitak/fasttrack, lukahartwig/mono,
- serrnovik/jax, albahq/alba, Earthly, Task).
- G2: 0.
- G3: 17 (IKatsuba/runx, kristofferlind/knega, egladman/magus, Grevix/Rivox, hjosugi/frost-build, nshkrdotcom/blitz,
- Ashutosh0x/hyperblaze, chaitanya-archive/obelisk.build, meslzy/outdo, jdarais/cobble, Buck2, Rush, pnpm workspaces
- and pnpm pipeline, pitchfork, systemd user units, Aspect CLI, microsoft/lage).

## Survivors

- None.
