# Monorepo manager re-screen, chunk 1

Re-screened 2026-09-16.
Input: `rescreen-chunk-1.json` (11 candidates that earlier exited only on the extension or plugin requirement).
That requirement is withdrawn; G3 was re-run under the remaining rules.

Method:

- Earlier evidence came from `doc/audit/tech-monorepo-manager-vet-2026-09-16/screening-primary-chunk-1.md`,
- `screening-primary-chunk-2.md`, `screening-primary-chunk-3.md`, and `screening-registry-chunk-1.md`.
  G1 and G2 were not re-decided; one doubt is noted under Bolt.
- Every relied-on page was fetched once with
- `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/
- 130.0'`, with `--output` and `--write-out '%{http_code} %{url_effective}'` added only to save the body and record stat
- us.
  Bot-challenge markers (`cf-challenge`, `challenge-platform`, `captcha`, `Just a moment`, `Attention Required`,
  `Access denied`, `verify you are human`, `Enable JavaScript and cookies`) were searched outside `<script>` elements;
  none matched on any page.
  Visible text (scripts, styles, and tags removed) was read for each page, and the docs text was present on every 200
  response.
- One github.com page returned 429; per the rate-limiting exception, the same Markdown was read with
- `gh api repos/<o>/<r>/contents/<path>` (recorded under rman).
- Repository trees, releases, and a docs source clone (`colcon/colcon.readthedocs.org`, shallow) came from `gh`;
- release dates from `registry.npmjs.org` and PyPI JSON.
- No candidate tool was installed or executed.
- Rule numbers: rule 1 is the confusion cull, rule 2 is no-JS and bot blocking, rule 3 is coverage.
  A rule 1 trigger is an immediate exit; further triggers are listed only as corroboration.

### folke/ultra-runner

- Earlier reason: G3 coverage (c), README-only docs; (a) annotated "partial".
- Release: npm `ultra-runner` 3.10.5 published 2021-02-28 (GitHub release `v3.10.5`, same day).
- Rule 2: https://github.com/folke/ultra-runner returned 200; README text present, no block markers.
- Rule 1 trigger (broken links): in the README sections "⚡ Fast" ("you can configure which scripts should be ran
- concurrently") and "💫 Getting Started" ("See optional configuration"), both links point to
- `##gear-optional-configuration` (double `#`).
  The heading anchor is `#gear-optional-configuration` (`id="user-content-gear-optional-configuration"`), so neither
  link reaches the configuration section.
- Rule 3, also failing:
  - (a) "⚙️ Optional Configuration" is a single annotated `package.json` example
  - (`"ultra": { "concurrent": ["lint"] }`) plus `.ultraignore` prose; no key list or reference.
  - (b) present: `ultra --help` output under "🚀 Usage".
  - (d) to (f) not claimed natively: `--watch` appears only as a flag passed to child build commands, and `--monitor`
  - is a local process list, not IPC.
- Final: exit, G3 rule 1 (broken configuration anchor links); rule 3 (a) also fails.

### colcon/colcon-core

- Earlier reason: G3 coverage (c), extension-point page names extension points without registration or interface; (a)
- and (b) present.
- Release: no GitHub releases; newest tag `0.21.2` per earlier evidence.
- Rule 2:
  - https://colcon.readthedocs.io/en/released/ returned 200, text present.
  - https://colcon.readthedocs.io/en/released/user/configuration.html returned 200, text present.
  - https://colcon.readthedocs.io/en/released/reference/verb/build.html returned 200, text present.
- Rule 1 trigger (reference incomplete, feature location unknown), on
- https://colcon.readthedocs.io/en/released/user/configuration.html:
  - The `colcon.pkg` key `type` is described as "to explicitly declare which colcon extension should process the
  - package", with no accepted values.
    A search of every `.rst` file in the docs source (`colcon/colcon.readthedocs.org`) found no page listing package
    type values.
  - "Using .meta files" says "Some configuration files are being picked up by default. The following are a few examples
  - (see e.g. `colcon build --help`)", so the reference itself is a partial list that defers to CLI help.
- Corroboration: the build verb page renders flag cross-references as `–cmake-target-skip-unavailable` and
- `–cmake-target clean` (en dash) while the flag heading is `--cmake-target-skip-unavailable`.
- Rule 3: (b) present (build verb page); (d) to (f) not claimed natively.
- Final: exit, G3 rule 1 (configuration reference leaves `type` values undocumented and calls its default-file list "a
- few examples").

### chrismatix/grog

- Earlier reason: G3 coverage (c), no plugin or extension mechanism; (a) and (b) present.
- Release: GitHub `v0.46.1` published 2026-09-04.
- Rule 2:
  - https://grog.build/ returned 200, text present.
  - https://grog.build/reference/configuration/ returned 200, text present.
  - https://grog.build/reference/commands/ returned 200, text present.
  - https://grog.build/build-configuration/ returned 200, text present.
  - https://grog.build/reference/target-configuration/ returned 200, text present.
- Rule 1 trigger (reference table contradicts field details), on https://grog.build/reference/target-configuration/:
  - The "Fields Reference" table lists `bin_output` with type `Output`, but there is no `bin_output` field detail (no
  - `id="bin_output"` heading, absent from "On this page") and `Output` is not defined on the page.
  - "Field Details" documents `oci_push` (`id="oci_push"`), which the "Fields Reference" table omits.
- Corroboration:
  - https://grog.build/reference/configuration/ offers "a complete example of a grog configuration file", but
  - `skip_workspace_lock`, explained further down the same page, is not in it (one occurrence in the page HTML).
  - https://grog.build/build-configuration/ pins the Pkl package to `package://grog.build/releases/v0.44.0/grog@0.44.0`
  - while the latest release is `v0.46.1`.
- Rule 3: (a) and (b) otherwise present; (d) to (f) not claimed.
- Final: exit, G3 rule 1 (target configuration table and field details disagree).

### meslzy/outdo

- Earlier reason: G3 coverage, no extension or plugin mechanism ("no plugin ecosystem"); (a), (b), and (d) watch
- documented.
- Release: npm `@meslzy/outdo` 0.0.1 published 2026-08-26.
- Rule 2:
  - https://meslzy.github.io/outdo/ returned 200, text present.
  - https://meslzy.github.io/outdo/reference/task-api returned 200, text present.
  - https://meslzy.github.io/outdo/reference/cli returned 200, text present.
  - https://meslzy.github.io/outdo/guide/watch-and-services returned 200, text present.
- Rule 1 trigger (prose contradicts examples and registry), on https://meslzy.github.io/outdo/reference/task-api:
  - The page says "Everything in outdo's public surface is exported from "outdo"", but its own example imports
  - `defineTasks` and `task` from `"@meslzy/outdo"`, as does the home page example.
  - `https://registry.npmjs.org/outdo` returns 404; the published package is `@meslzy/outdo`.
    The home page quick start `bunx outdo init` names the unscoped package that does not exist.
- Corroboration: the same page, introduced as "Everything do.ts can export, in one place", truncates its type list with
- "…".
- Final: exit, G3 rule 1 (package specifier contradiction between prose, examples, and registry).

### Bolt

- Earlier reason: G3 coverage (c), README-only docs with no extension documentation; configuration documents only
- `bolt.workspaces`; several commands marked not implemented.
- Release: npm `bolt` 0.24.10 published 2021-09-28; repository last push 2024-06-01; not archived.
- Rule 2: https://github.com/boltpkg/bolt returned 200, README text present.
  `gh api repos/boltpkg/bolt/contents/` shows no docs directory, so the README is the only doc.
- Rule 1 triggers:
  - Undefined status marker: the "Commands" table marks commands ✅, ❌, or 🖌; 🖌 (on `bolt build`, `bolt test`,
  - `bolt format/fmt`, `bolt lint`, `bolt doc`, `bolt check`) is never explained, so whether those commands exist
  - cannot be read from the docs.
  - Obsolete docs: "But Bolt (will soon) do lots of other stuff ... Well that's what Bolt will do for you (once we get
  - around to implementing it)", with the last release in 2021.
- Rule 3 (a) also fails: "Configuration" is one example ("For examples, for declaring workspaces in sub-directories")
- of `bolt.workspaces`.
- G1 doubt, not re-decided: the README says `bolt ws run [script]` runs "a script in every package" and documents no
- dependency ordering.
- Final: exit, G3 rule 1 (undefined command status marker; obsolete roadmap text); rule 3 (a) also fails.

### drkns

- Earlier reason: G3 coverage (c); README has `drkns.yml` fields (a) and a command list (b).
- Release: PyPI `drkns` 4.0.0 uploaded 2026-08-25.
- Rule 2: https://github.com/frantzmiccoli/drkns returned 200, README text present.
- Rule 1 trigger (config keys only in test fixtures): README "Steps" says a step can "Have a command to run, string,
- required" and "Have a background flag", without naming the keys.
  The keys `command:` and `background: true` appear only in the test fixture
  `testprojects/nominalcase/project3/drkns.yml`, and the README sends readers to "a dummy project on which our tests
  are based".
- Corroboration:
  - "Troubleshooting" runs `drkns sync forget dir1_project2/fedd32a-adbcc445555`, while "drkns CLI > Commands"
  - documents `drkns forget UNIT_NAME[/HASH]` and `drkns sync DIRECTION` with DIRECTION `in` or `out`.
  - "In your CI" comments `drkns sync out --delete` as "Persist all execution statuses from S3".
- Final: exit, G3 rule 1 (step keys learnable only from test fixtures; `forget` command spelled two ways).

### @enspirit/emb

- Earlier reason: G3 coverage, configuration reference documents only built-in plugins.
- Release: npm `@enspirit/emb` 0.31.1 published 2026-08-17.
- Rule 2:
  - https://enspirit.github.io/emb/ returned 200, text present.
  - https://enspirit.github.io/emb/getting-started/introduction/ returned 200, text present.
  - https://enspirit.github.io/emb/reference/configuration/ returned 200, text present.
  - https://enspirit.github.io/emb/reference/cli/ returned 200, text present.
- Rule 1 trigger (self-described complete reference contradicts itself), on
- https://enspirit.github.io/emb/reference/configuration/, which says "This is the complete reference for EMB
- configuration files":
  - "Variable Expansion > Syntax" lists only the `${env:...}`, `${vars:...}`, and `${vault:...}` forms, while the same
  - page documents `${op:...}` expansion under `plugins` and uses `${op:Private/db-credentials#connection_string}` in
  - the `file` resource example.
  - "Rebuild triggers" names `flavors.<flavor>.defaults.rebuildPolicy['docker/image']`, but both `flavors` sections
  - (project and component) document only `patches`.
- Corroboration: https://enspirit.github.io/emb/reference/cli/ ("Complete reference for all EMB commands") states
- "`emb <command> --help` is always authoritative".
- Final: exit, G3 rule 1 (configuration reference omits keys and syntax it uses on the same page).

### workspace-utils

- Earlier reason: G3 coverage (c), no plugin, extension, or hook mechanism.
- Release: npm `workspace-utils` 2.1.2 published 2026-08-21.
- Rule 2:
  - https://torstendittmann.github.io/workspace-utils/ returned 200, text present.
  - https://torstendittmann.github.io/workspace-utils/configuration/ returned 200, text present.
  - https://torstendittmann.github.io/workspace-utils/commands/overview/ returned 200, text present.
  - https://torstendittmann.github.io/workspace-utils/commands/run/ returned 200, text present.
  - https://torstendittmann.github.io/workspace-utils/commands/build/ returned 200, text present.
- Rule 1 trigger (broken links): the configuration page's "Next Steps" links `./concepts/workspace-detection.md`,
- `./concepts/package-managers.md`, and `./advanced/performance.md`.
  The first resolves to
  https://torstendittmann.github.io/workspace-utils/configuration/concepts/workspace-detection.md, which returned 404;
  `docs/src` in the repository has no `concepts` or `advanced` directory.
- Corroboration:
  - The configuration page example `"typecheck": "workspace-utils run typecheck --parallel"` uses `--parallel`, which
  - none of the configuration, commands overview, run, or build pages document (only `--sequential`).
  - The configuration page calls `--filter` a "Repeatable relationship-aware package selector" (`--filter "app..."`),
  - while the run page says "Filter packages by glob pattern" and the configuration page's "Multiple Filters" says "Use
  - multiple invocations".
- Final: exit, G3 rule 1 (404 links from the configuration page; undocumented flag in its example).

### rman

- Earlier reason: G3 coverage (c), CLI and programmatic API documented; only pre/post script hooks.
- Release: npm `rman` 1.0.12 published 2026-09-16.
- Rule 2:
  - https://github.com/panates/rman returned 200, README text present.
  - https://github.com/panates/rman/blob/main/docs/cli.md returned 200, text present.
  - https://github.com/panates/rman/blob/main/docs/cli/run.md returned 200, text present.
  - https://github.com/panates/rman/blob/main/docs/api.md returned 429 (GitHub rate limiting of our request volume, not
  - a project bot wall); the same Markdown was read with `gh api repos/panates/rman/contents/docs/api.md`.
- Rule 1 trigger (contradiction between summary and reference):
  - README "Commands" table: `publish` "Publishes every package to its configured target(s) - npm, Docker and/or GitHub
  - Releases".
  - README section `rman github-release`: "It is deliberately neither a publish.target nor opt-in".
  - `docs/api.md` "Config keys reference", row `publish.target`: `'npm' | 'docker'`, "The repository's GitHub Release
  - is not a target here".
  - The README "Commands" table also omits `github-release`, which the README documents further down.
- Corroboration: in `docs/api.md`, the `run.<script>.concurrency` row says "See `RunService` below", but the
- `RunService` section never explains `concurrency` (its `Options` type has `parallel`), and its per-script YAML
- example declares `test:` twice.
  The raw file's header comment records `package-version: 1.0.11`, one release behind npm (not visible in rendered
  view).
- Final: exit, G3 rule 1 (README says GitHub Releases is a publish target; README section and config reference say it
- is not).

### lattice (@latticeandcompany/lattice)

- Earlier reason: G3 coverage (c), extension only by editing source; CLI and configuration references exist.
- Release: GitHub `v1.1.1` published 2026-09-10; npm `@latticeandcompany/lattice` 1.1.1 same day.
- Rule 2:
  - https://latticeandcompany.github.io/lattice returned 200 (redirected to `/lattice/`), text present.
  - https://latticeandcompany.github.io/lattice/docs returned 200 (redirected to `/docs/`), text present.
  - https://latticeandcompany.github.io/lattice/docs/configuration returned 200, text present.
  - https://latticeandcompany.github.io/lattice/docs/cli returned 200, text present.
  - https://latticeandcompany.github.io/lattice/docs/nested-repos returned 200, text present.
  - https://latticeandcompany.github.io/lattice/docs/dev-servers returned 200, text present.
- Rule 1 trigger (contradiction between pages): https://latticeandcompany.github.io/lattice/docs/ says "Nested repos
- covers a repo that contains another repo with its own `lattice.json`".
  The linked page, https://latticeandcompany.github.io/lattice/docs/nested-repos, is titled "Wrap a repo that has its
  own task runner" and wraps a turbo subtree; its only `lattice.json` is the root one (one occurrence on the page), and
  the repository example `examples/nested-repo` has a single root `lattice.json` with `frontend/turbo.json`.
  The configuration page says commands walk up "to the nearest lattice.json", so an inner `lattice.json` has behavior,
  but the page the introduction promises for it does not cover it.
- Context for the caller: this was the only trigger found; the configuration and CLI references read consistently
- otherwise.
- Rule 3 otherwise: (a) https://latticeandcompany.github.io/lattice/docs/configuration and (b)
- https://latticeandcompany.github.io/lattice/docs/cli are field-level and flag-level references; (d) to (f) not
- claimed natively (dev servers are `persistent: true` tasks; the dev servers example uses `cargo watch` as a user
- command).
- Final: exit, G3 rule 1 (introduction misdescribes the nested repos page).

### monist / monist-tools

- Earlier reason: G3 coverage (c), README documents `monistrc.json` and commands; README says Monist is being phased
- out.
- Release: npm `monist` 1.7.0 published 2020-02-10; npm `monist-tools` 2.0.0 published 2021-11-06.
- Rule 2:
  - https://github.com/lddubeau/monist returned 200, README text present.
  - https://github.com/lddubeau/monist-tools returned 200, README text present.
- Rule 1 trigger (obsolete docs): the monist README opens "IMPORTANT NOTE: Monist is being phased out in favor of npm
- workspaces".
- Corroboration:
  - "Issues With Parallelism" refers to `monist run-all some-script`, a command absent from "Usage" (`npm`, `run`,
  - `update-versions`, `set-script`, `verify-deps`).
  - `--local-deps=link` says "Monist 2 will remove this option", but npm `monist` has no 2.x version; the forked
  - `monist-tools` started at 2.0.0 and dropped `run`, so whether it is "Monist 2" is not stated.
  - "Usage" gives one-line summaries and says "Please use monist [cmd] --help to get a more comprehensive description",
  - deferring the CLI reference (rule 3 (b)) to the binary.
  - monist-tools: its README lists only `update-versions`, `set-script`, and `verify-deps` (no task running, so it does
  - not fit G1 alone) and says "You invoke monist with monist" while its commands are `monist-tools ...`.
- Final: exit, G3 rule 1 (docs declare the tool phased out).

## Survivors

None.
All 11 candidates exit at G3 rule 1; none reached G4.
Rule 2 blocked no candidate (one github.com 429 was handled by the rate-limit exception).
