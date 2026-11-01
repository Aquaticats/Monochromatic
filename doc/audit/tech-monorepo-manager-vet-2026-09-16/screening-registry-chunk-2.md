# Monorepo manager screening, registry chunk 2

Screened 2026-09-16.
Input: `screening-registry-chunk-2.json` (68 candidates).
No candidate tool was installed or executed.

Evidence sources:

- npm registry JSON (`https://registry.npmjs.org/<pkg>`): latest version and publish time, `license`, `repository`,
- `deprecated`, README.
  No candidate's latest npm version carries a `deprecated` flag.
- `gh api repos/<owner>/<repo>` (license detection, `archived`, `pushed_at`), `/releases`, `/tags`, recursive git trees
- (license-file presence), and raw file contents.
- Forgejo API for `git.fallet.net`, GitLab API for `gitlab.com`.
- Docs pages were fetched once with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0' <url>`; visible text is the HTML with scripts, styles, and tags removed.
  GitHub repo and blob pages return server-rendered README text (HTTP 200) under this command.
  npm package pages do not: https://www.npmjs.com/package/workgraph returned HTTP 403 with "Just a moment... Enable
  JavaScript and cookies to continue".
  So an npm-only package whose README exists only on npmjs.com has no human-readable docs under the no-JS rule.

Interpretation notes applied to every candidate:

- G1 monorepo manager requires the docs to show relationship-aware execution across projects (dependency order,
- dependency cascade, or `^task` dependencies).
  Tools that fan the same command out to every folder or package without using relationships are category mismatches.
- G1 "provisional pass": when a package's docs are empty or too thin to show its category, but its self-description
- claims monorepo management, G1 is recorded as provisional and the later gates decide.
  G1 fails outright only when the docs positively describe a different category.
- G2 follows chunk 1: a license file must exist in an accessible source repository.
  A license declared only in the npm manifest or README counts as missing; each such exit lists its G3 outcome as
  incidental, so no final outcome depends on this reading.
- G3 official docs exclude source files, issues, chat, and blogs.
  Confusion rule: the first confusion trigger ends the read, recorded with URL.
  No-JS rule (user rule change, applied to all candidates): a bot block, 403/429/202, or JS shell without docs text is
  an immediate G3 exit.
- Once a gate fails, later gates are not evaluated; G4 lists only evidence already gathered, marked "incidental".

### workspaces-filter

- URL: https://github.com/tunnckoCore/workspaces-filter
- License: MIT (`LICENSE` at repo root)
- Latest release: v0.8.7, 2025-12-01
- Archived: no
- G1: fail, category mismatch.
  - README describes "A companion for filtering monorepo workspaces, by package name or package dir ... Useful for
  - running scripts on a subset of workspaces" (https://github.com/tunnckoCore/workspaces-filter#readme).
  - Selection is by name or directory glob only; neither the CLI help nor the `filter`/`runCommandOn` API docs mention
  - dependency order or any relationship between packages.
- G2: not evaluated.
- G3: not evaluated.
  Incidental: even under a looser G1 it would exit on coverage, with no configuration reference and no extension
  mechanism in the README (its only doc).
- G4: not evaluated.
- Final: exit at G1 (workspace filter plus script fan-out, no relationship-aware execution).

### mr-yarn

- URL: https://github.com/leecheneler/mr-yarn
- License: MIT (`LICENSE`)
- Latest release: 1.0.0-alpha.11 (prerelease), 2018-10-31
- Archived: yes
- G1: fail, category mismatch.
  - README `run` section: "Run NPM scripts in workspaces. **Currently all scripts are run in parallel all the time.**"
  - (https://github.com/leecheneler/mr-yarn#run).
  - Only `add` looks at local workspace dependencies; task running ignores relationships.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (parallel script fan-out over Yarn workspaces; archived).

### workspace-builder

- URL: https://github.com/suchipi/workspace-builder
- License: MIT (`packages/workspace-builder/LICENSE`; no root license file)
- Latest release: npm 1.0.2, 2021-02-16 (no GitHub releases)
- Archived: no
- G1: pass (minimal), monorepo manager.
  - https://github.com/suchipi/workspace-builder/tree/master/packages/workspace-builder: runs the builder modules named
  - in each Yarn workspace's `package.json`, passing `dependsOn: Array<Workspace>` ("workspace packages that this
  - workspace has in its package.json dependencies").
- G2: pass. MIT license file in the tool's package directory.
- G3: fail, confusion.
  - Trigger: the same page says "It also has a watch mode where it does the same thing, but watches each workspace
  - directory's `src` dir", yet no command, flag, or option to enable it appears anywhere in the docs.
    No-JS check: HTTP 200, text present.
  - The repo root README points to the package README as plain text ("See packages/workspace-builder/README.md for more
  - info.") rather than a link.
  - Coverage would also fail on (b): no CLI reference.
- G4 (incidental):
  - d: claimed watch mode, undocumented invocation.
  - e, f: none.
  - g: none.
- Final: exit at G3 confusion (watch mode claimed with no documented way to enable it).

### kodrdriv tree (@grunnverk/tree-execution, @grunnverk/commands-tree)

- URL: https://github.com/grunnverk/tree-execution
- License: `@grunnverk/tree-execution` MIT (`LICENSE`); `@grunnverk/commands-tree` Apache-2.0
- (https://github.com/grunnverk/commands-tree)
- Latest release: tree-execution v1.5.8, 2026-03-27; commands-tree npm 1.5.14, 2026-03-27
- Archived: no
- G1: fail, category mismatch (library plus add-on of another tool).
  - tree-execution README: "A sophisticated parallel execution framework ... Originally developed as part of the
  - kodrdriv toolkit, this library has been extracted for standalone use"; usage is only programmatic
  - (`createTreeExecutor`, `DynamicTaskPool`).
  - commands-tree npm description: "Tree and dependency management commands for kodrdriv", a command module for the
  - host CLI.
  - The host CLI kodrdriv (https://github.com/grunnverk/kodrdriv) describes itself as "an AI-powered Git workflow
  - automation tool that generates intelligent commit messages and release notes"; `kodrdriv tree` is one subcommand
  - family.
- G2, G3: not evaluated.
  Incidental: kodrdriv's designated docs site https://grunnverk.github.io/kodrdriv/ returned HTTP 200 with 619 bytes
  whose only visible text is "🚀 KodrDriv - Intelligent Git Release Notes" (JS shell), so screening kodrdriv itself
  would exit at G3 no-JS.
- G4: not evaluated.
- Final: exit at G1 (execution library and kodrdriv command module).

### @jacob-ebey/mono-build

- URL: https://www.npmjs.com/package/@jacob-ebey/mono-build
- License: WTFPL (npm manifest only)
- Latest release: npm 0.0.8, 2018-09-10
- Archived: n/a (no repository)
- G1: pass (borderline), monorepo manager.
  - npm README: `mono.json` lists packages, "the build order is defined by the order of the packages in mono.json";
  - commands install and link packages to each other and run a script "in each project in order".
  - The order is a hand-written list, not derived relationships; linking does use inter-package relationships.
- G2: fail. No `repository` field in the npm manifest; `gh api repos/jacob-ebey/mono-build` returns 404 and a
- repository search under `user:jacob-ebey` returns nothing.
- G3: not evaluated.
  Incidental: the README exists only on npmjs.com, which blocks plain curl (HTTP 403).
- G4: not evaluated.
- Final: exit at G2 (no source repository).

### verify-grid

- URL: https://github.com/kirull1/verify-grid
- License: MIT in npm manifest and README badge; no license file in repo (GitHub license detection none, tree has no
- license file)
- Latest release: npm 0.4.0, 2026-01-25 (no GitHub releases)
- Archived: no
- G1: fail, category mismatch.
  - README: "Run matrix of tasks across services with parallel execution and live status reporting"; `services` are
  - `{ name, cwd }` entries and ordering comes from per-task `priority`
  - (https://github.com/kirull1/verify-grid#-configuration).
  - No relationship between services is modeled; it is a task-matrix runner.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (task by service matrix runner without project relationships).

### buildsure

- URL: https://github.com/clasen/BuildSure
- License: MIT in npm manifest and README; no license file in repo
- Latest release: npm 0.6.0, 2026-09-07 (no GitHub releases)
- Archived: no
- G1: fail, category mismatch.
  - README: "If `[path]` contains a `package.json`, it builds that project. Otherwise it iterates immediate
  - subdirectories that have `package.json` + a `build` script." (https://github.com/clasen/BuildSure#cli).
  - It is an mtime-gated install-and-build wrapper per folder with no relationships between projects.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (per-folder conditional build wrapper).

### kitchen-tools

- URL: https://github.com/JacopoPatroclo/kitchen-tools
- License: MIT in npm manifest; no license file in repo
- Latest release: v0.7.0, 2020-10-23
- Archived: no (last push 2022-12-06)
- G1: fail, category mismatch.
  - README: "Set of tools to generate and run docker-composes service oriented projects"; `bake` scaffolds services and
  - `oven` "under the hood it uses docker-compose" (https://github.com/JacopoPatroclo/kitchen-tools#documentation).
  - Scaffolder plus docker-compose wrapper; no task execution across projects.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (scaffolder and docker-compose wrapper).

### workgraph

- URL: https://www.npmjs.com/package/workgraph
- License: MIT (npm manifest only)
- Latest release: npm 0.0.9, 2026-01-20
- Archived: n/a (no repository)
- G1: pass, monorepo manager.
  - npm README: "Scans workspace projects and builds a directed dependency graph", "Determines which projects are
  - affected by changes (transitive dependents)", "Uses Kahn's algorithm to plan build waves".
- G2: fail. No `repository` or `homepage` field in the npm manifest, and a GitHub repository search for
- `workgraph in:name` returns only unrelated projects (aiidateam/aiida-workgraph, GPUOpen WorkGraph samples).
- G3: not evaluated.
  Incidental: the README exists only on npmjs.com, which returned HTTP 403 "Just a moment... Enable JavaScript and
  cookies to continue" to plain curl (no-JS exit); it also links `./RELEASING.md`, which has no target on npm.
- G4 (incidental, npm README):
  - d: `workgraph watch` rebuilds affected projects on change and can start dev servers.
  - e, f: none documented (terminal UI only).
  - g: `workgraph.sources` runs configured code generators before dependent builds.
- Final: exit at G2 (no source repository).

### affected-ci

- URL: https://github.com/Cst2989/affected-ci
- License: MIT (`LICENSE`)
- Latest release: v1.0.2, 2026-07-28
- Archived: no
- G1: fail, category mismatch.
  - README: "Run lint, typecheck, tests and Playwright specs on only what your diff can reach — derived from your
  - import graph, with no workspaces required" and "it works in a single app in a single repo with no package
  - boundaries at all" (https://github.com/Cst2989/affected-ci#readme).
  - It selects files from a module import graph and passes them to fixed tool tiers; there is no workspace-of-projects
  - model.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (file-level affected selector, no project model).

### @monoloom/cli

- URL: https://git.fallet.net/monoloom/workspace
- License: GPL-3.0 (`LICENSE` at repo root, https://git.fallet.net/monoloom/workspace/raw/branch/main/LICENSE)
- Latest release: v1.0.1, 2026-06-07 (Forgejo releases API)
- Archived: no
- G1: pass, monorepo manager.
  - README: `npx monoloom affected:<npm-script-name>` "will calculate your dependency graph based on what you have
  - declared in the `dependencies` key of your `package.json` files, then execute the provided npm script in a fail
  - fast way".
- G2: pass. GPL-3.0 license file in the source repository.
- G3: fail, confusion.
  - Trigger: https://git.fallet.net/monoloom/workspace (README "Github Actions/Forgejo Actions usage") runs
  - `npx workspace-tools affected:lint`, while "Usage" documents the command as
  - `npx monoloom affected:<npm-script-name>`; the docs never mention a `workspace-tools` binary.
    No-JS check: HTTP 200, README text present after the Forgejo "This website requires JavaScript." banner.
  - Also contradictory: the example says "The `test-billing` workspace declares a dependency to the `application`
  - workspace" while its JSON declares `"billing": "*"`.
  - Coverage would also fail on (a) and (c): the tool is zero-config with no configuration reference and has no
  - extension mechanism.
- G4 (incidental):
  - d, e, f: none.
  - g: writes `.workspaces-state` at the repo root after each successful run.
- Final: exit at G3 confusion (CI example uses an undocumented `workspace-tools` command name).

### @blaze-js/cli

- URL: https://www.npmjs.com/package/@blaze-js/cli
- License: ISC (npm manifest only)
- Latest release: npm 0.0.5, 2022-09-11
- Archived: n/a (no repository)
- G1: provisional pass. The npm packument has no README ("ERROR: No README data found!"); the only category signal is
- the description "Monorepo manager".
- G2: fail. No `repository` field; a GitHub repository search for `blaze-js in:name` returns only unrelated projects.
- G3: not evaluated. Incidental: no docs exist.
- G4: not evaluated.
- Final: exit at G2 (no source repository; also no docs).

### nasti-task (Nasti Plus)

- URL: https://github.com/nasti-toolchain/nasti-task
- License: MIT (`LICENSE`)
- Latest release: npm `@nasti-toolchain/nasti-task` 0.1.0, 2026-07-28 (no GitHub releases)
- Archived: no
- G1: pass, monorepo manager.
  - README: "the dependency-aware monorepo task runner behind `np run` ... provides package graph scheduling"; config
  - example uses `"dependsOn": ["^build"]`.
- G2: pass. MIT license file.
- G3: fail, confusion and coverage.
  - Trigger: https://github.com/nasti-toolchain/nasti-task quick start runs `nt run -r build` and
  - `nt run @scope/app#build`, but neither `-r` nor the `#` target syntax is explained anywhere; the README's behavior
  - summary defers to "inspired by Vite Task" and "behind `np run`" with no link.
    No-JS check: HTTP 200, text present.
  - Coverage: no CLI reference (b), no configuration reference beyond one example (a), no extension mechanism (c). The
  - README is the only doc (repo tree has `README.md` and `LICENSE` only).
- G4: not evaluated.
- Final: exit at G3 confusion (undocumented CLI flag and target syntax; README is the only doc).

### Hark (@hark/plugin-monorepo)

- URL: https://github.com/sparebytes/hark
- License: MIT (`LICENSE`)
- Latest release: tag v0.11.5 (no GitHub releases); npm `@hark/plugin-monorepo` 0.11.4, 2020-04-26
- Archived: yes
- G1: fail, category mismatch.
  - README: "Hark - The Reactive Task Runner ... Powered by RxJS ... Hard-forked from nextools @start ... See
  - ./harkfile.ts for an example." (https://github.com/sparebytes/hark#readme).
  - Hark is a generic task runner; monorepo support is a plugin (`@hark/plugin-monorepo`) whose npm packument has no
  - README and whose description reads "🐣 Spawn new child process".
- G2, G3, G4: not evaluated.
- Final: exit at G1 (generic task runner; the monorepo part is an undocumented plugin; archived).

### gmm

- URL: https://github.com/k-koehler/gmm
- License: MIT (npm manifest only)
- Latest release: npm 0.0.3, 2021-09-19
- Archived: n/a (repository missing)
- G1: provisional pass. No README in the npm packument; description "GMM Monoepo Manager".
- G2: fail. `gh api repos/k-koehler/gmm` returns 404 and the user's public repo list has no `gmm`.
- G3: not evaluated. Incidental: no docs exist.
- G4: not evaluated.
- Final: exit at G2 (source repository missing; also no docs).

### rumos

- URL: https://github.com/remirobichet/rumos
- License: MIT in npm manifest; no license file in repo (tree: 6 entries, none a license)
- Latest release: npm 0.2.1, 2023-05-16
- Archived: no
- G1: provisional pass. README: "CLI tool to run faster monorepo scripts"; usage is only `pnpm rumos`.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental: the README (https://github.com/remirobichet/rumos#readme) documents nothing beyond install and
  `pnpm rumos`, so it would also exit at G3.
- G4: not evaluated.
- Final: exit at G2 (no license file; docs are one command).

### npm-mono-repo (Sodaru)

- URL: https://github.com/sodaru/mono-repo
- License: MIT (`LICENSE`)
- Latest release: tag v1.2.0; npm 1.2.0, 2022-09-27
- Archived: no
- G1: pass, monorepo manager.
  - README `run`: "Runs the npm script in every (or selected) package (**In the 'dependency first' order**)"; `publish`
  - also runs in dependency-first order.
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/sodaru/mono-repo `create` "creates a new package at `packagesDir/packageDirName`", but
  - `packagesDir` is not defined or configurable anywhere in the docs.
    No-JS check: HTTP 200, text present.
  - Coverage would also fail on (a) and (c): no configuration reference, no extension mechanism.
- G4: not evaluated.
- Final: exit at G3 confusion (undefined `packagesDir` setting).

### @yamato-daiwa/monorepo-helper

- URL: https://github.com/TokugawaTakeshi/Yamato-Daiwa-Monorepo-Helper
- License: MIT (`LICENSE`)
- Latest release: npm 0.3.0, 2026-09-12
- Archived: no
- G1: fail, category mismatch.
  - README: "The alternative approach to npm workspaces, pnpm, yarn workspaces and Lerna ... Using symlinks during
  - local development"; the only commands are `ydmh version` (set versions, rewrite internal deps to relative paths,
  - `npm install`, `npm audit fix`) and `ydmh publish` (build all, then publish sequentially).
  - It is a linking, versioning, and publishing helper; it runs no user-selected tasks, and builds run in `packages`
  - list order.
- G2, G3: not evaluated.
  Incidental confusion triggers: the install command names `@yamato-daiwa/yamato-daiwa-monorepo-helper` while the
  package is `@yamato-daiwa/monorepo-helper`, and the publish heading reads `ydmn publish`.
- G4: not evaluated.
- Final: exit at G1 (link, version, and publish helper).

### Rex (@rex-js/rex-cli, @rex-js/rex)

- URL: https://github.com/nikeokoronkwo/rex
- License: MIT (`LICENSE`)
- Latest release: GitHub prerelease 0.0.1+alpha, 2024-03-30; npm `@rex-js/rex-cli` 0.0.2, 2024-04-19
- Archived: no
- G1: provisional pass. README: "a powerful and feature-rich tool used for handling, managing and scaling
- monorepositories"; no task-running behavior is described.
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: the README sends readers to "the [wiki] and the [docs](./docs/README.md)"; `[wiki]` has no link target,
  - and https://github.com/nikeokoronkwo/rex/blob/main/docs/README.md is 11 bytes containing only "# Rex Docs".
    No-JS check: HTTP 200, text present.
  - https://github.com/nikeokoronkwo/rex/blob/main/docs/hooks.md ends its list sentence "The Git Hooks supported by Rex
  - are:" with no list.
- G4: not evaluated.
- Final: exit at G3 confusion (empty docs index, truncated hooks page, dead wiki link).

### fraktos

- URL: https://github.com/holasoymalva/fraktos
- License: MIT in npm manifest and README; no license file in repo
- Latest release: npm 0.1.0, 2026-06-08
- Archived: no
- G1: pass (minimal), monorepo manager.
  - README: `fraktos.config.json` lists projects with `path`, `command`, and `depends_on`; it "launches them in a
  - coordinated, dependency-aware DAG".
  - One command per project and no RPC/IPC/HTTP interface (only an in-process API), so it is not a component.
- G2: fail. No license file; the README license badge links to `https://github.com/fraktos/fraktos/blob/main/LICENSE`,
- a different repository.
- G3: not evaluated.
  Incidental: no extension mechanism documented (c) and no CLI reference beyond `--config`.
- G4: not evaluated.
- Final: exit at G2 (no license file).

### robonaut

- URL: https://github.com/f1lt3r/robotnaut
- License: MIT (npm manifest; https://github.com/f1lt3r/robonaut redirects to markserv/robonaut, whose license GitHub
- detects as MIT)
- Latest release: npm 2.2.1-alpha.0, 2017-01-13 (markserv/robonaut last push 2017-02-10)
- Archived: listed URL returns 404; markserv/robonaut not archived
- G1: fail, category mismatch.
  - npm README: commands `embed`, `prime`, `assemble` (git clone and npm install dependencies), `fuse` (`npm link`),
  - `scan` (git diff), `numerate` (version bumps), `transmit` (git push and npm publish).
  - It is a multi-repo clone, link, and release helper; it runs no tasks across projects.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (multi-repo release helper).

### @emeryld/manager

- URL: https://www.npmjs.com/package/@emeryld/manager
- License: MIT (npm manifest only)
- Latest release: npm 1.6.1, 2026-05-02
- Archived: n/a (no repository)
- G1: fail, category mismatch.
  - npm README: "Dev dependency built for Codex agents: scan a pnpm workspace, scaffold RRRoutes packages, and ship
  - releases"; its actions call pnpm (`pnpm -r update`, `pnpm test`, `pnpm build` "filtered to the package when
  - possible").
  - An interactive wrapper over pnpm plus a scaffolder and a format checker; relationship handling is pnpm's.
- G2, G3, G4: not evaluated. Incidental: no `repository` field (would fail G2).
- Final: exit at G1 (interactive wrapper over pnpm).

### bdep

- URL: https://github.com/sourcewizard-ai/bdep
- License: Apache-2.0 (`LICENSE` text is Apache License 2.0; GitHub detection reports NOASSERTION)
- Latest release: npm 0.1.2, 2026-03-09 (no GitHub releases)
- Archived: no
- G1: pass (minimal), monorepo manager.
  - README: "scans your `package.json` for `workspace:` protocol dependencies, constructs a dependency graph, and
  - builds packages in parallel layers, respecting the topological order".
- G2: pass. Apache-2.0 license file.
- G3: fail, coverage and confusion.
  - Coverage (c): no extension or plugin mechanism; (a): "Zero configuration", no configuration reference.
  - Confusion trigger: https://github.com/sourcewizard-ai/bdep "Architecture" step 8 says "Run `bun run build` for each
  - package", while installation covers npm, pnpm, and yarn and the file list names
  - `pm.ts # Package manager detection`; the docs do not say which runner builds a non-Bun workspace.
    No-JS check: HTTP 200, text present.
- G4 (incidental):
  - d, e, f: none.
  - g: none.
- Final: exit at G3 coverage (no extension mechanism) with a confusion trigger (build runner unclear).

### Plymor (@plymor/cli, core, deploy adapters)

- URL: https://github.com/plymor/plymor
- License: Apache-2.0 (npm manifest only)
- Latest release: npm `@plymor/cli` 0.2.2, 2026-05-31
- Archived: n/a (repository missing)
- G1: provisional pass. No README in the npm packuments; descriptions "Polyglot monorepo orchestration" and "Core
- runtime for Plymor (manifest loading, affected detection, task runner)".
- G2: fail. `gh api repos/plymor/plymor` returns 404 and `gh api users/plymor/repos` lists no public repositories.
- G3: not evaluated. Incidental: no docs exist.
- G4: not evaluated.
- Final: exit at G2 (no accessible source repository; also no docs).

### npm-ws

- URL: https://github.com/bstefanescu/npm-ws
- License: MIT (`LICENSE`)
- Latest release: npm 0.9.7, 2021-05-10 (last push 2022-12-30)
- Archived: no
- G1: pass, monorepo manager.
  - README: "project tasks - which are run on each project respecting the dependency order between projects (i.e.
  - projects required by another project will be processed first)".
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/bstefanescu/npm-ws "Tasks": "Not all the build configuration and tasks are documented.
  - To see an example of how to configure a workspace look into the root package.json of Qute ... More documentation is
  - comming soon."
    Understanding configuration requires reading another project's source.
    No-JS check: HTTP 200, text present.
- G4 (incidental):
  - d: `ws run "build dev" watch start` ("Watch for file changes and rebuild"), with a dev server that has live reload.
  - e, f: none.
  - g: none.
- Final: exit at G3 confusion (docs defer configuration to another project's package.json).

### auto-n-glad

- URL: https://www.npmjs.com/package/auto-n-glad
- License: MIT (npm manifest only)
- Latest release: npm 1.0.3, 2026-03-13
- Archived: n/a (no repository)
- G1: fail, category mismatch.
  - npm README: "automatically finds all Node.js projects inside a folder and runs commands like `install`, `dev`, or
  - `build` in each project"; no relationships.
- G2, G3, G4: not evaluated. Incidental: no `repository` field.
- Final: exit at G1 (folder-wide script fan-out).

### monopod

- URL: https://github.com/studio-b12/monopod
- License: MIT per npm manifest and `License.md` (GitHub detection NOASSERTION)
- Latest release: npm 1.0.1, 2016-05-31
- Archived: yes
- G1: fail, category mismatch.
  - README: "Available commands: `bootstrap`, `debootstrap`" (https://github.com/studio-b12/monopod#options); it
  - synchronizes dependencies and links packages but runs no tasks.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (bootstrap-only linker; archived).

### obr (One Big Repo)

- URL: https://github.com/sourcewizard-ai/obr
- License: Apache-2.0 (npm manifest only)
- Latest release: npm 0.1.0, 2025-08-04
- Archived: n/a (repository missing)
- G1: provisional pass. npm README: "Work in Progress ... A TypeScript CLI tool for managing monorepos" with
- `obr build` "Build packages in the monorepo".
- G2: fail. `gh api repos/sourcewizard-ai/obr` returns 404; the org's public repos list has no `obr`.
- G3: not evaluated. Incidental: the README's install steps (`npm install`, `npm run build`, `./bin/obr --help`) assume
- a source checkout that is not public.
- G4: not evaluated.
- Final: exit at G2 (source repository missing).

### sail (@tylerbu/sail)

- URL: https://github.com/tylerbutler/tools-monorepo
- License: MIT (`packages/sail/LICENSE`)
- Latest release: npm 0.2.3, 2026-01-19 (no GitHub releases)
- Archived: no
- G1: pass, monorepo manager.
  - https://github.com/tylerbutler/tools-monorepo/blob/main/packages/sail/docs/build.md: `sail build` filters packages,
  - release groups, and workspaces, runs `--task` values, and `--force` "Force the tasks to run, ignoring dependencies".
- G2: pass. MIT license file in the package directory.
- G3: fail, confusion.
  - Trigger: https://github.com/tylerbutler/tools-monorepo/blob/main/packages/sail/docs/build.md lists both
  - `-g, --releaseGroup=<value>  The name of a release group.` and
  - `-w, --workspace=<value>  The name of a release group.`, so the workspace flag's meaning is unclear.
    No-JS check: HTTP 200, text present.
  - The package README (https://github.com/tylerbutler/tools-monorepo/tree/main/packages/sail) says "Installation:
  - Coming soon."
  - No `sail.config.ts` configuration reference page exists among `packages/sail/docs/*.md`; `custom-task-handlers.md`
  - shows `version`, `plugins`, and `customHandlers` only in examples.
- G4 (incidental):
  - d, e, f: none documented.
  - g: none.
  - Extension: task handler plugins, `packages/sail/docs/custom-task-handlers.md`.
- Final: exit at G3 confusion (workspace flag described as a release group; install section "Coming soon").

### @anzerr/mono.cli

- URL: https://github.com/anzerr/mono.cli
- License: MIT (`LICENSE`)
- Latest release: tag v1.0.12; npm 1.0.12, 2022-01-21
- Archived: no
- G1: provisional pass. README: "Cli to help working on a monorepo using a local npm registry like verdaccio"; only
- four example command lines.
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/anzerr/mono.cli shows `mono restore` and `mono exec --count 1 "pwd"` with no
  - explanation of what `restore` does or what `--count` counts; the README has no other sections.
    No-JS check: HTTP 200, text present.
- G4: not evaluated.
- Final: exit at G3 confusion (undocumented commands and flags in example-only README).

### @sfomin/for-each-package

- URL: https://github.com/slavafomin/for-each-package
- License: MIT (`LICENSE`)
- Latest release: GitHub v0.0.1, 2020-04-15; npm 0.0.2, 2020-04-28
- Archived: no
- G1: fail, category mismatch.
  - README: "Runs command for each npm package under current working directory", filtered by `--name` glob or RegExp;
  - no relationships.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (per-package command fan-out).

### mrex

- URL: https://github.com/ssupinsky/mrex
- License: ISC in npm manifest; no license file in repo (tree: 4 entries)
- Latest release: npm 0.0.10, 2022-12-22
- Archived: no
- G1: pass, monorepo manager.
  - README: "Run package scripts for every package in a monorepo that uses workspaces, with respect to the order of
  - inclusion", with a dependency-order example.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental: README-only docs with no configuration reference or extension mechanism, so it would also exit at G3
  coverage.
- G4: not evaluated.
- Final: exit at G2 (no license file).

### hwan

- URL: https://www.npmjs.com/package/hwan
- License: MIT (`LICENSE` in https://github.com/theJian/hwan, found by repository search; the npm manifest has no
- `repository` field)
- Latest release: npm 0.1.0-alpha.4, 2018-04-04
- Archived: yes (theJian/hwan)
- G1: provisional pass. README: "minimalistic monorepo management tool".
- G2: pass. MIT license file in theJian/hwan.
- G3: fail, confusion.
  - Trigger: https://github.com/theJian/hwan README body is only "// TODO". No-JS check: HTTP 200, text present.
- G4: not evaluated.
- Final: exit at G3 confusion (README is a TODO placeholder; archived).

### alle (with alle-publish)

- URL: https://github.com/kogosoftwarellc/alle
- License: MIT (`LICENSE`)
- Latest release: npm `alle` 0.0.0, 2018-03-16; `alle-publish` 1.0.1, 2016-11-14 (separate repo tlvince/alle-publish)
- Archived: no (last push 2018-05-27)
- G1: fail, category mismatch.
  - The main README defers to `packages/alle.cli`, whose README documents one command: "analyze: Outputs dependency
  - information for the packages in the current working directory's monorepo packages".
  - `alle-publish` is "An approximation of Lerna publish in alle", a publish add-on. No task running across projects.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (dependency analyzer plus publish add-on).

### orca (@fasunle/orca)

- URL: https://github.com/fasunle/orca
- License: MIT in npm manifest and README badge; no license file in repo
- Latest release: GitHub 0.1.0, 2026-06-15; npm 0.1.1, 2026-06-15
- Archived: no
- G1: pass, monorepo manager.
  - README: "Discovers all workspaces ↓ Builds task dependency graph ↓ Performs topological sort"; tasks support
  - `"dependsOn": ["^build"]`.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental confusion triggers (https://github.com/fasunle/orca#readme): the tagline reads "A Orca-inspired task
  orchestrator" and the FAQ asks "How is this different from Orca?", answering "Orca is a general monorepo tool. orca
  is specifically optimized for Bun"; examples use `persistent: true`, which the "Configuration Reference" list omits.
- G4: not evaluated.
- Final: exit at G2 (no license file).

### bun-cache (@fasunle/bun-cache)

- URL: https://github.com/fasunle/bun-cache
- License: MIT in npm manifest; no license file (the URL redirects to https://github.com/Fasunle/orca, the same
- repository as orca)
- Latest release: npm `@fasunle/bun-cache` 0.1.1, 2026-06-15
- Archived: no
- G1: pass, monorepo manager (same README as orca, reading "A Turbo-inspired task orchestrator ... Works with standard
- `turbo.json`").
- G2: fail. The repository was renamed to orca and has no license file.
- G3: not evaluated. Incidental: the package and its repository were renamed to orca, and the published README still
- shows `bun-cache` commands.
- G4: not evaluated.
- Final: exit at G2 (no license file; superseded by orca, same repo).

### spinx

- URL: https://github.com/nirikshan/spinx
- License: MIT in npm manifest and README; no license file in repo
- Latest release: npm 0.1.3, 2025-11-04
- Archived: no
- G1: pass, monorepo manager.
  - README: "Build all workspaces in dependency order", `spinx build --since=origin/main`, `start --with-deps`;
  - workspaces declare `dependsOn`.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental confusion triggers (https://github.com/nirikshan/spinx#readme): the config interface documents a `watch`
  block, while "Contributing" lists "Watch mode with hot reload" as a future enhancement; `spinx run` refers to
  "command key from spinup.config.js" instead of `spinx.config.js`; `spinx init` is used but not in the command list.
- G4: not evaluated.
- Final: exit at G2 (no license file).

### monob

- URL: https://www.npmjs.com/package/monob
- License: none declared (npm manifest has no `license`)
- Latest release: npm 0.0.3, 2020-12-27
- Archived: n/a (no repository)
- G1: provisional pass. npm README: "build local projects in an optimized manner without needing to understand the
- dependency structure of the monorepo".
- G2: fail. No `repository` field, no license, and a GitHub repository search for `monob in:name` returns only
- unrelated projects.
- G3, G4: not evaluated.
- Final: exit at G2 (no source repository, no license).

### lolaus

- URL: https://gitlab.com/tom.davidson/lolaus
- License: MIT (npm manifest; GitLab API `license_url` null)
- Latest release: npm 0.3.1, 2018-05-21 (GitLab last activity 2018-05-23)
- Archived: no (GitLab `archived` not set)
- G1: fail, category mismatch.
  - npm README: "a convenience shell wrapper of host's git client ... the command is invoked from a sorted and unique
  - list of directories sequentially via eval in a loop"; directories are chosen by glob and git diff.
  - No workspace or project relationship model.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (git-diff directory command loop).

### @cvsky/monorepo-builder

- URL: https://www.npmjs.com/package/@cvsky/monorepo-builder
- License: MIT (npm manifest only)
- Latest release: npm 1.0.0, 2025-11-22
- Archived: n/a (no repository)
- G1: provisional pass. Description "Monorepo 交互式构建工具" (interactive build tool); the README is a local debugging note
- about `package.json` fields.
- G2: fail. No `repository` field.
- G3, G4: not evaluated.
- Final: exit at G2 (no source repository).

### ditoh

- URL: https://github.com/katerman/ditoh
- License: MIT (`LICENSE.md`)
- Latest release: ditoh@1.1.0, 2026-05-20
- Archived: no
- G1: fail, category mismatch.
  - README: "run centralized meta-scripts from within subpackages ... recursively searches up the directory tree for a
  - `ditoh.config.json` file and executes the defined commands".
  - A script-lookup runner; no cross-project execution or relationships.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (root script launcher).

### BuildNet (@musclemap.me/buildnet)

- URL: https://musclemap.me/buildnet
- License: MIT (npm manifest only)
- Latest release: npm 1.0.0, 2026-01-26
- Archived: n/a (repository missing)
- G1: pass, monorepo manager (with a daemon).
  - npm README: `.buildnet/config.json` lists packages with `dependencies`, `build_cmd`, `sources`, `output_dir`; the
  - `buildnetd` daemon serves HTTP on port 9876 with `buildAll`, `buildPackage`, `status`, and Server-Sent Events.
- G2: fail.
  - The npm package is only "TypeScript client for BuildNet"; the daemon is built from `packages/buildnet-native` in
  - https://github.com/jeanpaulniko/musclemap, which returns 404 (the user's public repos do not include it).
  - The homepage https://musclemap.me/buildnet failed with a TLS error under plain curl.
- G3: not evaluated.
- G4 (incidental, npm README):
  - d: none documented.
  - e: HTTP `health`, `status`, `stats`, `listBuilds`, SSE event stream.
  - f: HTTP `buildAll`, `buildPackage`, `cacheClear`; no cancel.
  - g: none.
- Final: exit at G2 (daemon source not available).

### onom

- URL: https://github.com/pshrmn/onom
- License: MIT (npm manifest only)
- Latest release: npm 1.0.0-alpha.0, 2018-08-21
- Archived: n/a (repository missing)
- G1: fail, category mismatch (library).
  - npm README: `const manager = await onom(config); manager.link(); manager.list();` creates symlinks and lists
  - dependencies; no task running.
- G2, G3, G4: not evaluated. Incidental: `pshrmn/onom` returns 404.
- Final: exit at G1 (linking library).

### @mono-repo/cli

- URL: https://github.com/mono-repo/mono-repo
- License: MIT (npm manifest only)
- Latest release: npm 1.4.4, 2020-08-15
- Archived: n/a (repository missing)
- G1: pass, monorepo manager.
  - npm README `run`: "By default packages are run in an efficient optimal order parallelizing scripts where it is
  - safe", `--sync` "obeys the dependency tree".
- G2: fail. `mono-repo/mono-repo` returns 404, and the README badge repository `mono-repo-dev/mono-repo` also returns
- 404.
- G3, G4: not evaluated.
- Final: exit at G2 (source repository missing).

### monorunyg

- URL: https://www.npmjs.com/package/monorunyg
- License: none in repo; npm manifest MIT
- Latest release: npm 1.0.1, 2025-07-28
- Archived: no (source https://github.com/yagyagoel1/MonoRepoYg-TurboRepo-alternative, found via the author's repo
- list; the npm manifest has no `repository` field; last push 2025-07-31)
- G1: pass, monorepo manager.
  - npm README: "Dependency-aware build ordering", "Builds a dependency graph of workspace packages", "Topological
  - Sort".
- G2: fail. The source repository has no license file at any path.
- G3: not evaluated.
  Incidental: the npm README installs `monorun` (`npm install --save-dev monorun`), a different package name from
  `monorunyg`.
- G4: not evaluated.
- Final: exit at G2 (no license file).

### @kienleholdings/mrt-run

- URL: https://github.com/kienleholdings/monorepo-tools
- License: MIT (npm manifest only)
- Latest release: npm 0.1.2, 2022-02-18
- Archived: n/a (repository missing)
- G1: fail, category mismatch.
  - npm README: "Run parallel commands in monorepos with no fuss"; options are `--npmCommand`, `--packagesDir`,
  - `--parallel`; no relationships.
- G2, G3, G4: not evaluated. Incidental: repository returns 404.
- Final: exit at G1 (script fan-out over a packages directory).

### mr-monorepo

- URL: https://github.com/SoulEvans07/mr-monorepo
- License: Beerware (npm manifest only)
- Latest release: npm 0.1.0, 2022-06-16
- Archived: n/a (repository missing)
- G1: provisional pass. No README; description "npm workspace based monorepo toolkit"; the tarball has a single file.
- G2: fail. `SoulEvans07/mr-monorepo` returns 404 and is absent from the user's public repos.
- G3, G4: not evaluated.
- Final: exit at G2 (source repository missing; placeholder package).

### @mykulyak/linterpol

- URL: https://github.com/mykulyak/linterpol
- License: MIT (`LICENSE`)
- Latest release: npm 0.2.0, 2022-01-20
- Archived: yes
- G1: fail, category mismatch.
  - README: "run multiple NPM scripts on those packages those files have been modified with respect to their remote
  - versions ... Commands will be executed sequentially".
  - Change-scoped script runner with no use of relationships between packages.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (modified-package script runner; archived).

### @messman/node-mono-builder

- URL: https://github.com/messman/node-mono-builder
- License: MIT (`LICENSE`)
- Latest release: npm 1.1.0, 2022-05-26
- Archived: no
- G1: pass, monorepo manager.
  - README: "Run commands from the `package.json scripts` of local projects in dependency order".
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/messman/node-mono-builder "Sample Use Case" gives the only configuration example as
  - "(pseudocode)" and runs `node-mono-builder build proj-c proj-b`, while "API" says the package "exposes a library"
  - with a `parse` function whose commands are `help`, `list`, `pushpull`, and `run [script] [projects]` (no `build`).
    No-JS check: HTTP 200, text present.
  - The API example `run build bridge-client bridge-iso --pushpull` is captioned "Build/push projA and then projB".
- G4: not evaluated.
- Final: exit at G3 confusion (pseudocode config; CLI example contradicts the command list).

### quimera

- URL: https://github.com/Tautorn/quimera
- License: MIT (`LICENSE`)
- Latest release: npm 0.2.0, 2018-10-25
- Archived: no
- G1: provisional pass. README: "A tool for managing multi-projects javascript"; npm description "Tool for create and
- manager monorepo".
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/Tautorn/quimera README ends at the heading "getting start" with no content. No-JS
  - check: HTTP 200, text present.
- G4: not evaluated.
- Final: exit at G3 confusion (README truncated after its first heading).

### tukod

- URL: https://github.com/vexCoder/tukod
- License: MIT (`LICENSE`)
- Latest release: npm 0.1.1, 2022-11-20
- Archived: no
- G1: fail, category mismatch.
  - README commands: `generate` (new app from template), `delete` (remove an app), `init` (base files); no task
  - execution.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (app scaffolder).

### deicide

- URL: https://github.com/binbandit/deicide
- License: MIT (`LICENSE`)
- Latest release: npm 0.0.2, 2025-10-29
- Archived: no
- G1: fail, category mismatch.
  - README: `deicide init` generates tsconfigs and `deicide ts-doctor` checks LSP issues; "Dev and CI are placeholders
  - for now".
- G2, G3, G4: not evaluated.
- Final: exit at G1 (tsconfig generator and diagnostics).

### ghidora (@hyperforge/ghidora)

- URL: https://ghidora.hyperforge.in
- License: Apache-2.0 (`LICENSE` in https://github.com/hyper-forge/ghidora; GitHub detection NOASSERTION)
- Latest release: npm 0.0.10, 2026-05-01 (no GitHub releases; last push 2026-07-11)
- Archived: no
- G1: pass, monorepo manager.
  - https://ghidora.hyperforge.in: "Runs a task across all matched packages. If dependencies are configured for that
  - task, Ghidora runs them first in the correct order"; `dependsOn: ["build", "^build"]`.
- G2: pass. Apache-2.0 license file.
- G3: fail, coverage (c) and confusion.
  - No-JS check: https://ghidora.hyperforge.in returned HTTP 200 with 14859 characters of visible docs text (single
  - page: Intro, Install, QuickStart, AdvancedConfig, CloudCaching).
  - a: present (config example with comments; "Core Configuration Controls").
  - b: present (QuickStart command list and execution flags).
  - c: absent. The page states "It does not rely on external JavaScript ecosystems, plugin chains, or hidden runtime
  - layers" and documents no extension mechanism.
  - Confusion trigger (same page, `ghidora.config.mjs`): the `build` task comment says "Run test on dependency packages
  - first // dependsOn: ["test"]" while the `test` task says "Tests require build to have run dependsOn: ["build"]".
    The same section says "all want is just define the run/build/test command on respective package's package.json
    file" while package discovery lists "JS, Rust, Go, Python".
  - d: persistent tasks are "Long-running (watch / dev servers)"; no watch mode claimed.
- G4 (incidental):
  - d: none (only `persistent: true` long-running tasks).
  - e: `ghidora graph` "Starts a local HTML view of the workspace graph"; nothing inspects a running process.
  - f: none.
  - g: `ghidora init`, `init --app`, `init --lib` scaffold workspace files.
- Final: exit at G3 coverage (no extension mechanism), also a confusion trigger (contradictory `dependsOn` comments).

### ystage

- URL: https://github.com/amurdock/ystage
- License: MIT in npm manifest; no license file in repo
- Latest release: v1.3.0, 2019-01-21
- Archived: no
- G1: provisional pass. npm description "A tool for managing ci/cd aspects of yarn workspaces";
- `ystage run --from --to <npm script>`.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental confusion trigger: https://github.com/amurdock/ystage README reads "determine what needs to be ??? based
  on from/to COMMIT SHA".
- G4: not evaluated.
- Final: exit at G2 (no license file).

### @linbop/modulify

- URL: https://linbop.github.io/modulify
- License: MIT in npm manifest; no license file in https://github.com/Linbop/modulify
- Latest release: npm 1.0.6, 2022-02-11
- Archived: yes
- G1: provisional pass. `docs/README.md`: "There are a lot of tools that can be used to build and maintain JS/TS
- monorepos ... Modulify is one of them"; esbuild runs, watch, hooks, concurrent scripts.
- G2: fail. No license file in the repository.
- G3: not evaluated.
  Incidental no-JS failure: https://linbop.github.io/modulify/ returned HTTP 200 with a 741-byte docsify shell
  (`<div id="app"></div>`), visible text only "Document".
- G4: not evaluated.
- Final: exit at G2 (no license file; docs site is also a JS shell; archived).

### bun-pkg

- URL: https://github.com/thejasonxie/bun-pkg
- License: MIT (`LICENSE`)
- Latest release: npm 1.0.14, 2024-04-18
- Archived: no
- G1: fail, category mismatch.
  - README: "an attempt to replicate pnpm's recursive command in bun workspaces"; `bun pkg -a run build` runs the
  - script in all packages and `-n` scaffolds a package; no dependency order is documented.
- G2, G3, G4: not evaluated.
  Incidental: the README license badge links to `brunobasto/bun-pkg`, a different repository.
- Final: exit at G1 (recursive fan-out without documented relationships).

### bouddha-monorepo

- URL: https://github.com/andrade0/bouddha-monorepo
- License: MIT in npm manifest and README; no license file in repo
- Latest release: npm 1.0.8, 2024-07-31
- Archived: no
- G1: fail, category mismatch.
  - README: "automates the process of copying and processing library files" into frontend and backend projects and
  - "watches the `libs` directory for changes"; no tasks and no RPC/IPC/HTTP interface.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (shared-library copier).

### umrepo

- URL: https://github.com/umrepo/um
- License: MIT (npm manifest only)
- Latest release: npm 0.0.0-alpha.0, 2024-10-25
- Archived: n/a (repository missing)
- G1: provisional pass. No README; description "A zero-footprint monorepo tool for native NPM/Bun workspace"; tarball
- has a single file.
- G2: fail. `umrepo/um` returns 404 and `users/umrepo` returns 404.
- G3, G4: not evaluated.
- Final: exit at G2 (source repository missing; placeholder package).

### @osndot/osn

- URL: https://github.com/osndot/osn
- License: MIT (`LICENSE`)
- Latest release: v0.2.0, 2026-03-17
- Archived: no
- G1: fail, category mismatch.
  - README: `osn init` "creates a `.osn/project.json` configuration file" with tasks and `dependsOn` between tasks of
  - that one project; there is no multi-project workspace model.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (single-project task runner with plugins).

### rootrun

- URL: https://github.com/Barani1912/rootrun
- License: MIT (`LICENSE`)
- Latest release: npm 1.0.1, 2026-03-15
- Archived: no
- G1: fail, category mismatch.
  - README: "Runs the `build` script concurrently in all packages"; scanning subdirectories with prefixed logs, no
  - relationships.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (concurrent script fan-out).

### Sleet (sleetrepo)

- URL: https://github.com/dupyjs/sleet
- License: proprietary (`LICENSE`: "PROPRIETARY LICENSE ... All rights reserved")
- Latest release: npm `sleetrepo` 1.0.0, 2025-11-02
- Archived: no
- G1: provisional pass. README: "A minimal monorepo orchestrator built for pure speed".
- G2: fail. Proprietary license: "Unauthorized copying, transferring, or reproduction ... is strictly prohibited".
- G3, G4: not evaluated.
- Final: exit at G2 (proprietary).

### simplebuild (@simple-software/simplebuild)

- URL: https://github.com/SimpleSoftwareOrg/Simple-Build
- License: MIT (npm manifest only)
- Latest release: npm 0.1.4, 2025-05-29
- Archived: n/a (repository missing)
- G1: fail, category mismatch.
  - npm README: "Build current directory"; phases mirror sources, analyze imports, "create Bazel build files", run
  - Bazel. A single-project Bazel generator with no workspace-of-projects model.
- G2, G3, G4: not evaluated. Incidental: repository returns 404 (the org's public repos are Bazel rule forks).
- Final: exit at G1 (single-project Bazel wrapper).

### Makedown (@makedown/cli, engine and family)

- URL: https://github.com/khoi03/makedown
- License: Apache-2.0 (`LICENSE`; server under AGPL-3.0 per README)
- Latest release: v0.1.1, 2026-06-30
- Archived: no
- G1: fail, category mismatch.
  - README: "Make for LLM workflows ... treats a directory of Markdown sources plus a `build.md` spec as a dependency
  - graph"; targets are LLM `chat`, `eval`, `map`, `transform`, `agent` steps.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (LLM workflow build system).

### oxymake (@noogram/oxymake)

- URL: https://noogram.org/oxymake
- License: MIT or Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE` in https://github.com/noogram/oxymake)
- Latest release: v0.4.0, 2026-09-14
- Archived: no
- G1: fail, category mismatch.
  - README: "a workflow engine ... your pipeline is a plain, declarative TOML file", positioned against Snakemake
  - (`ox translate Snakefile`), and "Daemon-free: every `ox run` is a self-contained process".
  - A recipe or workflow runner with no workspace-of-projects model, and not a persistent process.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (data workflow engine).

### linkctl

- URL: https://github.com/saintparish4/linkctl
- License: MIT (`LICENSE`)
- Latest release: tag v2.0.0 (no GitHub releases); npm 2.0.0, 2026-08-30
- Archived: no
- G1: pass, monorepo manager.
  - https://github.com/saintparish4/linkctl/blob/master/docs/monorepo.md: discovers pnpm/npm/Yarn workspaces, generates
  - `<package-name>:build` tasks, and `--affected` "cascades to include any package that depends on a changed package
  - (transitively)".
- G2: pass. MIT license file.
- G3: fail, confusion.
  - Trigger: https://github.com/saintparish4/linkctl/blob/master/docs/getting-started.md step 4 runs
  - `npx linkctl build` with a config where only `test` declares `dependsOn: ["build"]`, then says it "runs both
  - tasks", and the sample output shows `test` running.
    That contradicts the page's own advice to "Add `dependsOn` between tasks to express ordering".
    No-JS check: HTTP 200, text present.
  - Coverage would also fail: the `docs/` set (config-reference, getting-started, impact-and-workspace, monorepo,
  - nextjs, pr-commands, remote-cache, troubleshooting, vite) has no CLI reference page (b) and no plugin page,
  - although the README architecture lists `core/plugins` (c).
  - The repo homepage field points to https://www.npmjs.com/package/antiscaler, a different package name.
- G4: not evaluated.
- Final: exit at G3 confusion (getting-started run output contradicts `dependsOn` semantics).

### reckon (@andrew24601/reckon)

- URL: https://www.npmjs.com/package/@andrew24601/reckon
- License: MIT (npm manifest; source https://github.com/andrew24601/reckon found by repository search)
- Latest release: npm 0.1.2, 2026-05-14
- Archived: no
- G1: fail, category mismatch (library).
  - npm README: "Reckon is a library-first incremental build system ... It currently does not include a standalone CLI,
  - watch mode"; builds are JS task graphs with C and macOS app helpers, with no workspace model.
- G2, G3, G4: not evaluated.
  Incidental: the README installs `reckon` (`npm install --save-dev reckon`), not the published name
  `@andrew24601/reckon`.
- Final: exit at G1 (build-graph library).

### nalth

- URL: https://www.nalthjs.com
- License: MIT (`LICENSE` in https://github.com/nalikiru-dev/Nalth.js)
- Latest release: npm 0.9.0, 2025-12-19 (GitHub tag v2.2.0)
- Archived: no
- G1: fail, category mismatch.
  - npm README: "the world's first security-first web development framework ... powered by a security-enhanced Vite.js
  - foundation"; `nalth run build` is one command of a single-app toolchain, with no workspace-of-projects model.
- G2, G3, G4: not evaluated.
  Incidental: https://www.nalthjs.com/ returned HTTP 200 with visible text only "nalth — Real Estate Commission
  Reconciliation", which is not the project's docs.
- Final: exit at G1 (single-app web toolchain).

### machora

- URL: https://github.com/jasonz1987/machora
- License: Apache-2.0 (`LICENSE`)
- Latest release: npm 0.8.1, 2026-09-02 (no GitHub releases)
- Archived: no
- G1: fail, category mismatch.
  - README: "A lightweight, self-hosted development workload orchestrator that turns spare Macs, Windows PCs, and Linux
  - servers into Git-native task machines"; "A project is a local Git repository associated with one task machine".
  - Projects are independent repositories with no inter-project relationships.
    The controller has a local HTTP API (`/api/*`, `http://127.0.0.1:4178`) but watches nothing: jobs start from the
    CLI, dashboard, or Git push hooks. So it is neither a monorepo manager nor a watch component.
- G2, G3, G4: not evaluated.
- Final: exit at G1 (remote job orchestrator).

## Exit summary

- G1 (32): workspaces-filter, mr-yarn, kodrdriv tree, verify-grid, buildsure, kitchen-tools, affected-ci, Hark,
- @yamato-daiwa/monorepo-helper, robonaut, @emeryld/manager, auto-n-glad, monopod, @sfomin/for-each-package, alle,
- lolaus, ditoh, onom, @kienleholdings/mrt-run, @mykulyak/linterpol, tukod, deicide, bun-pkg, bouddha-monorepo,
- @osndot/osn, rootrun, simplebuild, Makedown, oxymake, reckon, nalth, machora.
- G2 (22): @jacob-ebey/mono-build, workgraph, @blaze-js/cli, gmm, rumos, Plymor, obr, fraktos, mrex, orca, bun-cache,
- spinx, monob, @cvsky/monorepo-builder, BuildNet, @mono-repo/cli, monorunyg, mr-monorepo, ystage, @linbop/modulify,
- umrepo, Sleet.
- G3 (14): workspace-builder (confusion), @monoloom/cli (confusion), nasti-task (confusion), npm-mono-repo (confusion),
- Rex (confusion), bdep (coverage c, plus confusion), npm-ws (confusion), sail (confusion), @anzerr/mono.cli
- (confusion), hwan (confusion), @messman/node-mono-builder (confusion), quimera (confusion), ghidora (coverage c, plus
- confusion), linkctl (confusion).
- Tally: 32 at G1, 22 at G2, 14 at G3, 0 survivors (68 total).

## Survivors

None.
