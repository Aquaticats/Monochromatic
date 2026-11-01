# Rescreen chunk 2: G3 after withdrawing requirement (c)

Input: `rescreen-chunk-2.json` (10 candidates).
Screened 2026-09-16.

## Method

- Earlier evidence read first: `doc/audit/tech-monorepo-manager-vet-2026-09-16/screening-registry-chunk-3.md`,
- `screening-promoted-runners.md`, `discovery-expansion.md`, `rescreen-list.md`.
- Repository metadata, file trees, READMEs and repo Markdown docs were read as raw content through `gh api` (script
- `scratchpad/rs2/gh-fetch.ts`, `scratchpad/rs2/gh-files.ts`).
- Every page a verdict relies on was fetched exactly once with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0' <url>` through `scratchpad/rs2/curl-page.ts`, which refuses refetches, strips `<script>`/`<style>`, measures vi
- sible text, and looks for block markers.
  Log: `scratchpad/rs2/pages/fetch-log.txt`; captures beside it.
- Every cited phrase was confirmed present in the curl capture (not only in the `gh api` Markdown).
- No GitHub fetch returned HTTP 429 in this run, so the `gh api` fallback of rule 2 was never needed.
- Release and engine metadata came from `gh api repos/<o>/<r>/releases/latest`, `https://registry.npmjs.org/<pkg>`, and
- `https://crates.io/api/v1/crates/<name>`.
- No candidate tool was installed or executed; no browser was used; nothing under `/var/home/user/Monochromatic` was
- modified.
- G1 and G2 were not re-run; the earlier evidence for each candidate matched current repository metadata (license,
- archived state, latest release).
- Rule 1 was applied as written: the first confusion trigger culls the candidate, so later pages were not needed for a
- verdict.
  Extra triggers seen while confirming the first are listed as corroboration only.

## Candidates

### cargo-flux

- URL: https://github.com/ignition-is-go/cargo-flux
- Earlier reason: G3 exit (coverage (c)): README-only docs; "Plugin Architecture" lists built-in plugins but documents
- no way to write, register, or load one.
- Current metadata: MIT, not archived, latest release v0.7.4 (2026-09-13); `docs/` holds only `superpowers/plans` and
- `superpowers/specs` design notes.
- G3 rule 2 (no-JS, bot wall): https://github.com/ignition-is-go/cargo-flux HTTP 200, 34602 visible characters, no
- block marker; README text present.
- G3 rule 1 (confusion): triggered, contradiction between README sections.
  - "Planning Semantics", "1. Entry points are opt-in": "Only packages that opt into a task are chosen as task
  - entrypoints." and "Flux does not run a task on every package in the workspace."
  - "`autoapply`": "`all`: same as `inherit`, and if no package explicitly opts into the task, Flux seeds the task on
  - every compatible package".
    The section the README calls "the most important rule set in Flux" denies the behavior that `autoapply = "all"`
    documents (and the "Workspace-root tasks" example uses `autoapply = "all"`).
  - Corroboration (not needed for the verdict): the "Task Definitions" reference and "Supported task variants" list
  - omit `root_steps`, `outputs` and `when`, which appear only inside examples under "Workspace-root tasks"; the
  - `report github-output` subcommand is absent from the "Commands" section.
- G3 rule 3 (coverage): not reached.
- Final: exit (G3 rule 1, contradiction: opt-in-only entrypoints versus `autoapply = "all"` seeding every compatible
- package).

### guild (guild-cli)

- URL: https://github.com/sprouted-dev/guild
- Earlier reason: G3 exit (coverage (c)): README-only docs; no extension or plugin mechanism.
- Current metadata: MIT, not archived, latest release v0.1.0 (2026-03-05); repository has no docs directory (only
- `README.md` and `CLAUDE.md`).
- G3 rule 2: https://github.com/sprouted-dev/guild HTTP 200, 8087 visible characters, no block marker; README text
- present.
- G3 rule 1: triggered, feature with no documented location.
  - "`guild.toml` Format", "Project": `tags = ["lib", "rust"]  # Optional tags for filtering`.
  - No command, flag, or section documents how to filter by tag: "CLI Commands" lists `guild`, `dev`, `build`, `test`,
  - `lint`, `run <target> [project]`, `affected <target>`, `list`, `graph`, `cache status`, `cache clean`, `init`, with
  - no options, and the only flags anywhere in the README are `--no-cache` and `init --yes`.
  - Corroboration: Quick Start `guild affected test  # Test only what changed since main` documents no way to choose a
  - base other than `main`.
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, tag filtering is promised but documented nowhere).

### Watchman

- URL: https://github.com/facebook/watchman (docs https://facebook.github.io/watchman/)
- Earlier reason: G3 exit (coverage (c)): sitemap has no extension or plugin page.
- Current metadata: MIT, not archived, latest release v2026.09.14.00 (2026-09-14).
- G3 rule 2: https://facebook.github.io/watchman/docs/install HTTP 200, 7832 visible characters, no block marker; page
- text present.
- G3 rule 1: triggered, obsolete docs on the install page (the entry page of the "Installation" nav section).
  - "Linux binaries are compiled on a GitHub Action VM (ubuntu-20.04 at the time of this writing)".
    The current release workflow's `linux-build` job, which uploads `watchman-<release>-linux.zip`, uses
    `runs-on: ubuntu-24.04` (`.github/workflows/release.yml`, read with `gh api` only to confirm the staleness).
  - "macOS File Descriptor Limits": "Only applicable on macOS 10.6 and earlier".
  - "Note : Our binaries are built from the main branch only. We don't provide binaries for v4.9.0." while releases are
  - date tags (latest v2026.09.14.00).
  - Supported-systems text still names "FreeBSD 9.1, OpenBSD 5.2" and "Windows 7 support is provided by community
  - patches"; footer "Copyright © 2023".
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, obsolete install docs: ubuntu-20.04 build host versus ubuntu-24.04 in the release workflow,
- macOS 10.6 guidance, v4.9.0 label).

### dev-process-manager

- URL: https://dev-process-manager.com (source https://github.com/vivid-planet/dev-process-manager)
- Earlier reason: G3 exit (coverage (c)); borderline note that the CLI-to-daemon channel is not documented as an
- interface.
- Current metadata: BSD-2-Clause, not archived, latest release v4.1.0 (2026-09-16); npm `dev-process-manager` 4.1.0.
- G3 rule 2: https://dev-process-manager.com/docs/getting-started HTTP 200 (redirected to `/docs/getting-started/`),
- 2354 visible characters, no block marker; page text present.
- G3 rule 1: triggered, version requirement disagrees with the release.
  - Getting Started, "Prerequisites": "Node.js ≥ 18".
  - Published package: `https://registry.npmjs.org/dev-process-manager/4.1.0` `engines` is `{"node":">=22"}`; the
  - repository `package.json` says the same.
  - Corroboration (read as Markdown through `gh api`, not curl-fetched): README "Logs" documents `-n [lines]` (optional
  - value) while `docs/docs/commands.md`, the source of https://dev-process-manager.com/docs/commands, documents
  - `-n, --lines <number>` (required value).
- G3 rule 3: not reached.
  (Read as Markdown source before the trigger: `docs/docs/configuration.md` and `docs/docs/commands.md` do contain a
  field table and a command reference; watch mode is not claimed; the daemon is driven only through the documented CLI.)
- Final: exit (G3 rule 1, Getting Started "Node.js ≥ 18" versus published `engines.node` ">=22").

### baton-run (Baton)

- URL: https://kanumuri9593.github.io/Baton/ (source https://github.com/kanumuri9593/Baton)
- Earlier reason: G3 exit (coverage (c)): adapters only in `src/adapters/`; no adapter or plugin mechanism documented.
- Current metadata: MIT, not archived, latest release v0.2.6 (2026-09-09); npm `baton-run` latest 0.2.6.
- G3 rule 2:
  - https://kanumuri9593.github.io/Baton/ HTTP 200, 3785 visible characters, no block marker; shows "v0.2.6 preview".
  - https://github.com/kanumuri9593/Baton HTTP 200, 19707 visible characters, no block marker; README text present.
- G3 rule 1: triggered, version labels disagree within the README.
  - Top of README: "Baton 0.2.6 is a public developer preview."
  - "Status" section: "0.2.3 is working and tested against a large production Flutter app".
  - The latest GitHub release and npm version are 0.2.6, so the "Status" claim describes an older version with no
  - statement about 0.2.6.
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, README "0.2.6" preview versus "Status" "0.2.3 is working and tested").

### clier-ai (clier)

- URL: https://github.com/somersstack/clier
- Earlier reason: G3 exit (coverage (c)); borderline note that the Unix socket has no protocol reference.
- Current metadata: MIT, not archived, no GitHub releases or tags; npm `clier-ai` latest 1.5.0 (2026-02-19) with
- `engines.node` ">=20.0.0"; repository `package.json` still says version 1.0.0.
- G3 rule 2:
  - https://github.com/somersstack/clier HTTP 200, 10294 visible characters, no block marker; README text present.
  - https://github.com/somersstack/clier/blob/main/docs/configuration.md HTTP 200, 23728 visible characters, no block
  - marker; rendered Markdown text present.
- G3 rule 1: triggered, contradictory command names for the same feature.
  - README "CLI Reference": `clier send <name> "input"   # Send stdin to a running process` (also
  - `docs/GETTING-STARTED.md` and `docs/AGENTS.md`: `clier send <process> "data"`).
  - `docs/configuration.md` "Input Configuration": `clier input interactive-app "hello world"` and
  - `clier input interactive-app "data" --no-newline` (also `docs/AGENTS-PIPELINE.md` and `docs/api-reference.md`:
  - `clier input`).
  - Corroboration: README "Requirements: Node.js >= 18.0.0" versus npm 1.5.0 `engines.node` ">=20.0.0";
  - `docs/configuration.md` "Mixed Pipeline Example" triggers on `build-api:exit`, an event absent from its own
  - "Built-in Events" list; `name` field "Used in: PM2 process name" and `docs/examples/README.md` "Check PM2:
  - `pm2 list`" while the README describes its own daemon.
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, `clier send` versus `clier input` for sending stdin).

### fbecart/zinoma

- URL: https://github.com/fbecart/zinoma (schema docs
- https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/struct.Project.html)
- Earlier reason: G3 exit (coverage): (a), (b), (d) present, (c) absent; borderline note that the sample output prints
- "Žinoma 0.19.0" while the latest release is 0.19.6.
- Current metadata: MIT, not archived, latest release 0.19.6 (2023-12-12); crates.io `max_version` 0.19.6.
- G3 rule 2:
  - https://github.com/fbecart/zinoma HTTP 200, 9301 visible characters, no block marker; README text present.
  - https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/struct.Project.html HTTP 200, 7440 visible
  - characters, no block marker.
  - https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/enum.OutputResource.html HTTP 200, 5738 visible
  - characters, no block marker.
- G3 rule 1: triggered, broken anchor plus version label disagreeing with releases.
  - README "Clean flag (`--clean`)" links `output.paths` to
  - `https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/enum.OutputResource.html#variant.Paths.field.paths`.
    The fetched page has no `variant.Paths` anchor; its ids are `variant.Files`, `variant.Files.field.paths`,
    `variant.Files.field.extensions`, `variant.CmdStdout`, `variant.CmdStdout.field.cmd_stdout` (the enum variant is
    `Files`, not `Paths`).
  - README "Building": `./target/release/zinoma --version` prints "Žinoma 0.19.0", while the latest release and the
  - rustdoc sidebar are 0.19.6.
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, missing `#variant.Paths.field.paths` anchor; "Žinoma 0.19.0" versus release 0.19.6).

### Invincible

- URL: https://github.com/saintedlama/invincible
- Earlier reason: G3 fail, coverage (c): README-only docs, (a) and (b) present, no plugin text.
- Current metadata: MIT, not archived, latest release v1.7.0 (2026-08-15); README opens with "Proof of concept ...
- APIs, config format, and behaviour WILL change without notice."
- G3 rule 2: https://github.com/saintedlama/invincible HTTP 200, 16345 visible characters, no block marker; README text
- present.
- G3 rule 1: triggered, contradictory config file name.
  - "Configuration": "Invincible looks for `.invincible.toml` in the current directory by default."; Quick start "Edit
  - .invincible.toml"; "Running" `--config` "(default: .invincible.toml)".
  - "CLI commands", "`invincible init`": "Create a starter `invincible.toml` in the current directory."
    The README does not say whether `init` writes a file the default lookup ignores.
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, `.invincible.toml` lookup versus `init` creating `invincible.toml`).

### affected

- URL: https://github.com/Rani367/affected (docs site https://rani367.github.io/affected)
- Earlier reason: G3 fail, coverage (c); borderline note that (a) is one annotated `.affected.toml` example.
- Current metadata: MIT, not archived, latest release v1.0.0 (2026-04-02).
- G3 rule 2:
  - https://rani367.github.io/affected HTTP 200 (redirected to `/affected/`), 3595 visible characters, no block marker;
  - its "Documentation" link goes to the GitHub README.
  - https://github.com/Rani367/affected HTTP 200, 13717 visible characters, no block marker; README text present.
- G3 rule 1: triggered, docs site examples contradict the README reference for the watch feature.
  - Docs site "Watch mode" example: `$ affected test --watch --base main`.
  - README "Usage", "`affected watch`": `affected watch test --base main` (watch is a subcommand; no `--watch` flag
  - appears in any README usage block or in "Global Flags").
  - Corroboration: site `affected lint --output junit` versus README `--junit results.xml` and no `lint` subcommand;
  - site and README both say "13 ecosystems" while the README feature line lists 14 names and the "Supported
  - Ecosystems" section lists 16 rows.
- G3 rule 3: not reached; had it been, (a) would also fail, since "Configuration" is a single annotated
- `.affected.toml` example.
- Final: exit (G3 rule 1, `affected test --watch` on the docs site versus `affected watch test` in the README).

### devtooie

- URL: https://github.com/rhyek/devtooie
- Earlier reason: coverage (c); one HTTP 429 from GitHub after heavy request volume.
- Current metadata: MIT, not archived, latest release 0.7.1 (2026-08-17); README "Requirements": "Unix only
- (macOS/Linux). Windows is not supported."
- G3 rule 2:
  - https://github.com/rhyek/devtooie HTTP 200, 16448 visible characters, no block marker.
  - https://github.com/rhyek/devtooie/blob/main/docs/configuration.md HTTP 200, 16052 visible characters, no block
  - marker.
  - https://github.com/rhyek/devtooie/blob/main/docs/package-lifecycle.md HTTP 200, 5734 visible characters, no block
  - marker.
  - https://github.com/rhyek/devtooie/blob/main/packages/devtooie/docs/agents.md HTTP 200, 66679 visible characters, no
  - block marker.
  - The earlier HTTP 429 (on `docs/control-api.md`) did not recur; rule 2 does not trigger.
- G3 rule 1: triggered, contradictions about controlling a running session through the control API (the (f) behavior we
- would consume).
  - `docs/configuration.md`, `autostart`: "Set `false` to leave it stopped; start it yourself with the `s` hotkey (or a
  - control-API `restart`)."
  - `docs/package-lifecycle.md`, "The two manual commands": restart (`POST /command/restart`) "re-runs the dev command;
  - available for any running package".
  - The consolidated agent guide contradicts itself on the same endpoint: `packages/devtooie/docs/agents.md` says
  - "`POST /command/restart/<name>` starts a stopped package" in the `autostart` entry and
  - "`POST /command/restart/<name>` works for any running package." later.
  - Corroboration: `docs/cli.md` `--rebuild` "first clears `dist/` for every build target", while the rebuild command
  - in `docs/package-lifecycle.md` depends on `cleans: true` or `clean` + `build` scripts and is otherwise "a no-op".
- G3 rule 3: not reached.
- Final: exit (G3 rule 1, whether control-API `restart` starts a stopped package).

## Exit summary

- G3 rule 1 (confusion): 10
  - contradiction between pages or sections: cargo-flux, clier, affected, devtooie, Invincible
  - feature with no documented location: guild
  - obsolete docs: Watchman
  - version labels or requirements disagreeing with releases: dev-process-manager, Baton, zinoma (plus a missing anchor)
- G3 rule 2 (no-JS or bot wall): 0
- G3 rule 3 (coverage): 0 reached

## Survivors

None.
G4 was not assessed because no candidate passed G3.
