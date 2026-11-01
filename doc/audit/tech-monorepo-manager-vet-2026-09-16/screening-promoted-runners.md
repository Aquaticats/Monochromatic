# Monorepo manager screening: promoted general runners

Input: the `## Promoted` section of `screening-runners-result.md`, 15 GitHub repositories that passed the earlier G1
category pass.
Entries keep input order.
Each candidate stops at its first failing gate.

## Method

- Metadata came from `gh api repos/<owner>/<repo>`, `/releases`, `/tags` and `/contents/`.
  PyPI and npm registry JSON filled in release dates where GitHub has no releases.
  Scripts and saved pages are in `scratchpad/promoted/`.
- G1 and G2 evidence came from repository files read through `gh api`.
- Every docs page cited under G3 was fetched once with
  `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
  130.0' <url>`,
  with only `--output` and `--write-out '%{http_code} %{url_effective} %{size_download}'` added to save the body and
  record the status.
  The quoted text was then checked in the visible HTML text, with `<script>` and `<style>` removed and tags stripped.
- GitHub `blob/` and repository pages returned server-rendered Markdown, and the quoted text was present in the visible
- HTML.
- A GitHub rate-limit note, which is not a candidate block:
  the first curl of https://github.com/OctaHive/octa/blob/main/docs/plugins.md returned HTTP 200 (447158 bytes, title
  "octa/docs/plugins.md at main").
  I then fetched the same URL again in quick succession, and those repeats returned HTTP 429 "Rate limit · GitHub".
  A later retry returned HTTP 200 with the quoted text present.
  The 429s came from my own repeated requests, not from the first request, so they did not count as a bot block.
- Nothing was installed or run, and nothing under the Monochromatic worktree was modified.
- G4 is recorded only for survivors, and there are none.

## Candidates

### nadlejs/nadle

- URL: https://github.com/nadlejs/nadle (docs https://nadle.dev)
- License: MIT (`LICENSE`)
- Latest release: `nadle/v0.5.3` on 2026-06-12 (npm `nadle` 0.5.3, same day)
- Archived: false
- G1: pass (monorepo manager).
  "Monorepo-native — first-class support for multi-package workspaces" (README.md).
  "Sub-workspace tasks also respect implicit dependency ordering based on `package.json` workspace dependencies (e.g.,
  if `app` depends on `lib`, then `lib:build` runs before" (`packages/docs/docs/concepts/workspace.md`).
- G2: pass (MIT `LICENSE`).
- G3: exit (confusion: an example disagrees with the guides).
  - Trigger: https://nadle.dev/docs/getting-started/features/ (linked from the README as "Features"; HTTP 200, text
  - present) shows `CopyTask` as
    `options: { from: "src/assets", to: "dist/assets", include: ["**/*.png", "**/*.svg"] }`.
  - https://nadle.dev/docs/guides/registering-task/ (HTTP 200) shows
  - `run: CopyTask, options: { from: "assets", into: "dist" }`.
    https://nadle.dev/docs/guides/file-operation-tasks/ (HTTP 200) also uses `into: "dist"`.
    The two pages disagree on the option name (`to` or `into`).
  - The same Features page ends its "Next Steps" with "Explore the API Documentation" as plain text with no link.
  - Coverage was not needed for the exit.
    The sidebar source (`packages/docs/sidebars.ts`) lists `config-reference`, `cli-reference` and
    `guides/authoring-plugin`.
- G4: not assessed.
- Final: exit at G3 (confusion: `CopyTask` option names differ between the Features page and the guides).

### GriffinCanCode/bldr

- URL: https://github.com/GriffinCanCode/bldr
- License: custom "GRIFFIN LICENSE Version 1.0" (`LICENSE`); GitHub license detection reports `NOASSERTION` ("Other")
- Latest release: v2.0.3 on 2025-12-01
- Archived: false
- G1: pass (monorepo manager).
  "High-performance build system for polyglot monorepos." and "bldr query 'deps(//src:app)'" (README.md).
- G2: exit (not an OSI-style license).
  - `LICENSE` is a custom license.
    The SPDX license list (3.29.0, 740 entries) has no Griffin entry.
  - Its attribution duty reaches beyond redistribution:
    "TANGENTIAL INCLUSION: Even when the Software is used indirectly, as a dependency, or as a minor component within a
    larger system, the name "Griffin" must be included in the project's attribution documentation."
  - Derivative works must put the name in "Project naming, descriptions, or documentation".
  - Its patent clause restricts works that do not contain the software:
    "Any concepts or innovations that are derived from, inspired by, or substantially similar to those contained in the
    Software may not be patented."
- G3: not assessed.
- G4: not assessed.
- Final: exit at G2 (custom non-OSI license with attribution and patent restrictions beyond the software).

### OctaHive/octa

- URL: https://github.com/OctaHive/octa
- License: MIT (`LICENSE`)
- Latest release: v0.4.0 on 2026-09-15
- Archived: false
- G1: pass (monorepo manager).
  "A root Octafile can automatically discover Octafiles in monorepo projects" and "`packages/api/Octafile.yml` exposes
  its `build` task as `packages:api:build`" (README.md).
  Task dependencies can name tasks in other namespaces.
- G2: pass (MIT `LICENSE`).
- G3: exit (confusion and coverage).
  - The only docs are the README plus `docs/*.md` in the repository; there is no docs site.
    https://github.com/OctaHive/octa returned HTTP 200 with the README text present.
  - First trigger: the README's third line is `![License: MIT](https://img.shields.io/github/license/adrianmrit/mom)`.
    That license badge shows a different repository, `adrianmrit/mom`.
  - Coverage (a) and (b): I could not tell where the Octafile keys or the CLI flags are documented as a reference.
    The README has no reference section.
    Flags such as `--dir`, `--config`, `--parallel`, `--output`, `--watch`, `--interval` and `--cache-profile` appear
    only inside narrative sections.
    `docs/` holds only `cache-formats-v1.md`, `cache-http-v1.md`, `cache-profile.md`, `events.md`,
    `plugin-distribution.md`, `plugins.md`, `runner-protocol.md` and `secrets.md`.
  - Plugin docs point to source code.
    https://github.com/OctaHive/octa/blob/main/docs/plugins.md (HTTP 200 on the first and final fetch; see the method
    note) says:
    "The Rust definitions in crates/octa-plugin/src/protocol.rs are the source of truth. Plugin authors should normally
    use the octa-plugin SDK and serve_plugin".
- G4: not assessed.
- Final: exit at G3 (confusion: wrong-repository license badge, and plugin docs that defer to source; coverage: no
- Octafile configuration reference and no CLI reference).

### no0dles/hammerkit

- URL: https://github.com/no0dles/hammerkit (docs https://no0dles.gitbook.io/hammerkit/)
- License: MIT (`LICENSE`)
- Latest release: v1.6.0 on 2026-05-31
- Archived: false
- G1: pass (monorepo manager).
  "In a monorepo every package needs the same `install`/`build` steps, and packages depend on each other in a specific
  order. Hammerkit handles both with includes ... and references (wire up cross-package order)."
  (https://no0dles.gitbook.io/hammerkit/local-vs-container/monorepo, HTTP 200).
- G2: pass (MIT `LICENSE`).
- G3: exit (confusion: navigation).
  - The GitBook home (HTTP 200) and https://no0dles.gitbook.io/hammerkit/llms.txt render without JavaScript.
  - Trigger: in the sidebar, "Guides" links to `/hammerkit/local-vs-container`, and its child "Local vs container
  - tasks" links to the same URL.
    That page (https://no0dles.gitbook.io/hammerkit/local-vs-container, HTTP 200, title "Guides | hammerkit") has the
    heading "Guides".
    Its body is the local-vs-container guide: "When a task should run in a container and when it should run on the
    host."
    The other guides sit under that page's path, for example the monorepo guide at
    `/hammerkit/local-vs-container/monorepo`.
    So "Guides" is a section label and a specific guide at the same time.
  - The cause is visible in `SUMMARY.md`, which lists `[Guides](docs/guides/local-vs-container.md)` and
  - `[Local vs container tasks](docs/guides/local-vs-container.md)`.
  - Coverage items seen before the trigger (from `llms.txt`):
    a https://no0dles.gitbook.io/hammerkit/build-file/reference.md ("Every key the build-file schema accepts.");
    b https://no0dles.gitbook.io/hammerkit/cli/execute.md;
    d https://no0dles.gitbook.io/hammerkit/task/watching.md.
    The only "Extending" page, https://no0dles.gitbook.io/hammerkit/task/extending (HTTP 200), covers task templates
    ("A task can be used as a base template and extended"), not plugins.
- G4: not assessed.
- Final: exit at G3 (confusion: "Guides" is both a nav section and the local-vs-container page).

### tuist/once

- URL: https://github.com/tuist/once (docs https://buildonce.dev/docs)
- License: MIT (`LICENSE`)
- Latest release: 0.56.0 on 2026-09-14
- Archived: false
- G1: pass (monorepo manager).
  "The Once graph describes the named parts of a workspace and what Once can do with them."
  (https://buildonce.dev/docs/guide/graph, HTTP 200).
- G2: pass (MIT `LICENSE`).
- G3: exit (confusion: the reference sidebar is incomplete).
  - https://buildonce.dev/docs, `/docs/reference`, `/docs/reference/manifest`, `/docs/reference/cli` and
  - `/docs/guide/getting-started` all returned HTTP 200 with their text present.
  - https://buildonce.dev/docs/reference (HTTP 200) says "The Target-kind Reference enumerates every built-in target
  - kind".
  - Trigger: the sidebar's Target Kinds group (the `data-part="nav-link"` anchors, checked on both the Reference page
  - and the Target Kinds page) lists `cargo_dependencies` but not `cargo_workspace`.
    It also has no `mix_workspace`, `mix_project` or `mix_release` entry, and no React Native group.
  - The index page https://buildonce.dev/docs/reference/prelude (HTTP 200) does link all of them.
    It has a "React Native target kinds" section (`react_native_dependencies`, `react_native_module`,
    `react_native_autolinking`, `react_native_codegen`, `react_native_bundle`, `react_native_apple_application`,
    `react_native_android_application`, `react_native_metro`),
    plus `cargo_workspace` ("native integration seed that derives first-party and locked ...") and `mix_workspace`.
  - Those pages exist, for example https://buildonce.dev/docs/reference/prelude/react_native_bundle (HTTP 200).
    The Guide sidebar also has a "React Native" page.
    Someone browsing the reference sidebar cannot tell these target kinds are documented.
- G4: not assessed.
- Final: exit at G3 (confusion: the Target Kinds sidebar leaves out kinds the index lists, including the Rust and
- Elixir workspace seeds and all React Native kinds).

### kraken-build/kraken

- URL: https://github.com/kraken-build/kraken (docs https://kraken-build.github.io/kraken/)
- License: MIT in `kraken-build/LICENSE`.
  There is no root license file, so GitHub reports none.
  `kraken-wrapper/pyproject.toml` declares `license = "MIT"` but has no license file of its own.
- Latest release: 0.52.1 on 2026-06-12 (PyPI `kraken-build` 0.52.1, same day)
- Archived: false
- G1: pass (monorepo manager).
  Gradle-style project and subproject tree: "The `kraken` CLI supports running in a sub project"
  (`docs/docs/krakenw.md`), and the repository's `.kraken.py` calls `project.subproject("kraken-build")`.
- G2: pass (MIT license file for the `kraken-build` package that provides the `kraken` CLI).
- G3: exit (confusion).
  - Trigger: https://kraken-build.github.io/kraken/ (HTTP 200, text present) says, under "Getting started":
    "Currently, Kraken's OSS components are not very well documented and do not provide a convenient way to get
    started."
  - Also in that page's Command-line navigation, the link to `cli/kraken-query-tree/` is labeled "kraken query
  - visualize", the same label as the link to `cli/kraken-query-visualize/`.
- G4: not assessed.
- Final: exit at G3 (confusion: the docs say they are poorly documented, and a nav label is duplicated).

### gregnazario/must

- URL: https://github.com/gregnazario/must (docs site https://gregnazario.github.io/must/, built from `site/mkdocs.yml`)
- License: Apache-2.0 (`LICENSE`)
- Latest release: no GitHub releases; only tag `v1.0.0`, whose commit is dated 2026-05-01
- Archived: false
- G1: pass (monorepo manager).
  "Demonstrates: Multi-package monorepo with cross-package dependencies, per-package workdir, and umbrella recipes."
  (`examples/monorepo-workspace/Mustfile.toml`).
- G2: pass (Apache-2.0 `LICENSE`).
- G3: exit (confusion).
  - First trigger: the README (https://github.com/gregnazario/must, HTTP 200) never visibly links the docs site.
    Its "Documentation" section lists only "Architecture — execution model and internals" and "Migration — migrating
    from Make, Just, and Taskfile".
    The GitHub repository has no homepage set either.
    I found the site only through `site_url` in `site/mkdocs.yml`.
  - Contradictory text: the README says "Apache-2.0 — see LICENSE".
    Every docs site page fetched has the footer "MIT License": https://gregnazario.github.io/must/ and
    https://gregnazario.github.io/must/guide/config-reference/ (both HTTP 200).
    That comes from `copyright: MIT License` in `site/mkdocs.yml`.
- G4: not assessed.
- Final: exit at G3 (confusion: the docs site is not linked from the README, and its footer states a different license
- from the repository).

### earthly/earthly

- URL: https://github.com/earthly/earthly
- License: MPL-2.0 (`LICENSE`)
- Latest release: v0.8.16 on 2025-07-16
- Archived: false
- Already screened in another pass (`screening-result-3.md`, "Earthly"), where it exited at G1 as a category mismatch.
  That pass also recorded the README statement "Earthly is no longer actively maintained".
- G1 to G4: not re-evaluated.
- Final: already screened (exit at G1 in that pass); skipped here.

### meta-company/makex

- URL: https://github.com/meta-company/makex (default branch `hg`)
- License: `LICENSE.md` (GitHub license detection reports `NOASSERTION`); not examined because G1 exits
- Latest release: no GitHub releases or tags; PyPI `makex` 20250502 uploaded 2025-05-27
- Archived: false
- G1: exit (category mismatch).
  Makex has no concept of the workspace's projects.
  A Makex workspace is a single boundary, and tasks point to other tasks by folder path.
  - "Workspaces define the roots or boundaries of projects or a repository." (`documents/source/workspaces.md`)
  - "The `path` is a folder containing the Makex file defining the Task." (`documents/source/task-locators.md`)
  - "Makex favors larger compilation units." (`documents/source/comparison.md`)
  - This is the same shape as Earthly's G1 mismatch in the other pass: explicit cross-directory target references, with
  - no project units.
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch: path-addressed tasks, no project model).

### metaist/ds

- URL: https://github.com/metaist/ds (docs https://docs.metaist.com/ds)
- License: MIT (`LICENSE.md`)
- Latest release: 1.3.0 on 2024-08-29 (PyPI `ds-run` 1.3.0, same day)
- Archived: false
- G1: exit (category mismatch: fan-out only).
  ds reads workspace members and runs a task in each matching member.
  Nothing in the docs uses relationships between members.
  - "To run a task across multiple workspaces, use the `--workspace` or `-w` options one or more times with a pattern
  - that indicates where the tasks should run." (`docs/workspaces.md`)
  - "The following are all equivalent and run `test` in both `member/a` and `member/b`" (`docs/workspaces.md`)
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch: runs a task in each member, with no relationships between members).

### omio-labs/myke

- URL: https://github.com/omio-labs/myke (docs https://omio-labs.github.io/myke)
- License: MIT (`LICENSE`)
- Latest release: v1.1.0 on 2026-09-06
- Archived: false
- G1: exit (category mismatch: aggregation and tags only).
  myke discovers projects and runs a task across all or tagged projects.
  Its only task relationships are before and after hooks, which are shell commands.
  - "`myke build` runs build in all projects" and "`myke <tag>/build` runs build in all projects tagged `<tag>`"
  - (README.md)
  - "`myke` is never a build or deployment tool, its just a task aggregator" and "myke completely bypasses file
  - tracking and only focuses on task aggregation and discoverability" (README.md)
  - `examples/hooks/myke.yml`: "tasks can run other tasks before and after", with `before: echo running before`
- G2: not assessed.
- G3: not assessed.
- G4: not assessed.
- Final: exit at G1 (category mismatch: fan-out by discovery and tags, with no relationships between projects).

### fbecart/zinoma

- URL: https://github.com/fbecart/zinoma (schema docs
- https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/struct.Project.html)
- License: MIT (`LICENSE`)
- Latest release: 0.19.6 on 2023-12-12
- Archived: false
- G1: pass (monorepo manager, borderline confirmed).
  Named projects are imported explicitly, and targets depend on targets in other projects.
  From https://fbecart.github.io/zinoma/doc/zinoma/config/yaml/schema/struct.Project.html (HTTP 200):
  "imports should be an object, the keys being the project names and the values their respective paths."
  and "In this example, the target test_all depend from targets defined in different projects."
  (example `dependencies: [api::test, webapp::test]`).
- G2: pass (MIT `LICENSE`).
- G3: exit (coverage).
  - The docs are the README (https://github.com/fbecart/zinoma, HTTP 200, text present) plus the rustdoc schema pages
  - it links (HTTP 200, text present).
  - (a) Configuration reference: present (rustdoc `Project`, `Target`, `OutputResource` field docs).
  - (b) CLI reference: present (the README's `USAGE` block).
  - (c) Extension or plugin mechanism: none documented.
    The README and schema cover only targets, inputs, outputs, services and imports.
  - (d) Watch mode is claimed natively and documented at behavior level in the README "Watch mode (`--watch`)" section:
    "it will keep an eye open on the targets' `input`'s paths and will re-execute the relevant targets in case
    filesystem changes are detected."
  - The README's build-from-source sample output still prints "Žinoma 0.19.0", while the latest release is 0.19.6.
- G4: not assessed.
- Final: exit at G3 (coverage: no extension or plugin mechanism documented).

### Jomy10/beaver

- URL: https://github.com/Jomy10/beaver
- License: MIT (`LICENSE`)
- Latest release: 4.1.0 on 2025-10-28
- Archived: false
- G1: pass (monorepo manager, borderline confirmed).
  Named projects, with target dependencies across projects and imported projects.
  "When the target is not defined in the same project, we need to qualify the name by prepending it with the project
  name (i.e. `MyOtherProject:MyMathLibrary`)." (https://github.com/Jomy10/beaver/blob/master/docs/src/2_3_dependency.md)
- G2: pass (MIT `LICENSE`).
- G3: exit (confusion: examples disagree with the reference).
  - Trigger: the README (https://github.com/Jomy10/beaver, HTTP 200) example uses `language: :cpp` and
  - `include: "include/physics"`.
  - The guide https://github.com/Jomy10/beaver/blob/master/docs/src/2_2_target.md (HTTP 200) uses
  - `language: :cxx, # specify language` and `headers: "path/to/headers/directory"`.
  - The API reference https://github.com/Jomy10/beaver/blob/master/docs/src/3_2_C_target.md (HTTP 200) lists `headers:`
  - and has no `include:` or `language:` field at all.
  - The README also says "there is basic documentation in the docs directory. More extensive documentation is coming
  - soon when the project is more stable."
    The mdBook in `docs/` is not published anywhere the README links.
- G4: not assessed.
- Final: exit at G3 (confusion: the README example uses keys and values that the guide and the API reference do not).

### BobBuildTool/bob

- URL: https://github.com/BobBuildTool/bob (docs https://bob-build-tool.readthedocs.io/en/latest/)
- License: GPL-3.0 (`LICENSE`)
- Latest release: v1.2.0 on 2026-01-17 (PyPI `BobBuildTool` 1.2.0, same day)
- Archived: false
- G1: pass (monorepo manager, borderline confirmed).
  Recipes declare packages and their dependencies, and Bob decides what to rebuild from that package graph.
  - "When Bob parses the recipes he builds an internal package graph. The general dependency structure is derived from
  - the recipes." (https://bob-build-tool.readthedocs.io/en/latest/manpages/bobpaths.html)
  - "Bob closely tracks the input of all packages ... If something is changed Bob can accurately determine which
  - packages have to be rebuilt." (`doc/tutorial/fingerprints.rst`)
- G2: pass (GPL-3.0 `LICENSE`).
- G3: exit (confusion).
  - First trigger: the docs landing page (https://bob-build-tool.readthedocs.io/, redirected to `/en/latest/`, HTTP
  - 200) is titled "Welcome to Bob’s documentation! — Bob 1.3.dev999+unknown documentation", and all four Bob pages
  - fetched carry that version label.
    The latest release is v1.2.0, so I could not tell which release the docs describe.
  - An example disagrees with the syntax it illustrates:
  - https://bob-build-tool.readthedocs.io/en/latest/manpages/bobpaths.html (HTTP 200) gives the verbose form of `f*` as
  - "chils@f* selects all children of the context package starting with f".
    Every other verbose example on that page uses the axis `child@`.
  - The "Man Pages" navigation also puts `bobpaths` ("Specifying paths to Bob packages", a query-language description)
  - in one flat list with the command pages `bob-build`, `bob-dev`, `bob-ls` and the rest.
  - Coverage was not needed for the exit.
    "Extending Bob" (https://bob-build-tool.readthedocs.io/en/latest/manual/extending.html, HTTP 200) says "Bob may be
    extended through plugins."
- G4: not assessed.
- Final: exit at G3 (confusion: the docs are labeled with an unreleased version, and the bobpaths example has a wrong
- axis name).

### oxequa/realize

- URL: https://github.com/oxequa/realize
- License: GPL-3.0 (`LICENSE`)
- Latest release: v2.0.2 on 2018-04-14 (last push 2021-05-14)
- Archived: false
- G1: pass (component, borderline confirmed).
  It is a persistent watch process with an optional HTTP server.
  - "--server -> Enable the web server" and "Manage multiple projects at the same time." (README.md)
  - `realize/server.go` registers `e.GET("/ws", s.projects)`.
    That websocket handler sends `json.Marshal(s.Parent)` (project state) and writes received JSON through
    `s.Parent.Settings.Write(s.Parent)`.
- G2: pass (GPL-3.0 `LICENSE`).
- G3: exit (confusion: broken anchor).
  - The only docs are the README (https://github.com/oxequa/realize, HTTP 200, text present).
  - Trigger: the README's "Content" list links "💃🏻 Get started" to `#get-started`.
    The page has no `user-content-get-started` anchor; the matching section is headed "Quickstart"
    (`user-content-quickstart`).
  - Coverage would also fail.
    (c) no extension or plugin mechanism is documented.
    (e) the web server is claimed (`--server`, "Redesigned panel"), but nothing documents what its HTTP or websocket
    interface exposes.
    The "Config sample" is an annotated example, not a reference.
- G4: not assessed.
- Final: exit at G3 (confusion: broken "Get started" anchor; coverage gaps for extension and the HTTP interface).

## Exit counts

- Already screened: 1 (earthly/earthly).
- G1: 3 (meta-company/makex, metaist/ds, omio-labs/myke).
- G2: 1 (GriffinCanCode/bldr).
- G3: 10 (nadlejs/nadle, OctaHive/octa, no0dles/hammerkit, tuist/once, kraken-build/kraken, gregnazario/must,
- fbecart/zinoma, Jomy10/beaver, BobBuildTool/bob, oxequa/realize).
  Nine exited on confusion; fbecart/zinoma exited on coverage.

## Survivors

None.
All 14 candidates screened here exited, and `earthly/earthly` was skipped as already screened.
