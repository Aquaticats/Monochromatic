# Translation-repair package split

## Purpose

The owner asked on 2026-10-18 why the package's lint takes so long,
and to split `package/module/translation-repair` (its path before this split) into packages under
`package/translation-repair/<name>`,
each of a manageable size,
before the other queued work.
This document records the measurements behind the answer,
the module map the split follows,
the rules the move keeps,
and the controls that prove the move changed no behaviour.
It was written before any file moved,
so the map and the rules could be read and vetoed as they stood;
the sections headed "After the move,
measured" and "Measured after the move" were added after the move.

## Lint cost, measured

All runs below are oxlint 1.87.0 on the package as it stood at `94ebefa1e3`,
in the keyless worktree `w165` after a fresh build,
timed with `/usr/bin/time`.

- The whole package (3004 files,
  378k production lines,
  1552 test files):
  24.6 s,
  23.9 s and 24.1 s wall over three warm runs,
  33 s user each time;
  0 warnings and 0 errors.
- The 92.1 s in gate `g99b` was a cold run on a loaded host:
  the first run in the branch worktree took 76 s wall and the second 25 s.
- 31 files at the bottom of the import graph (the modules every runner reaches):
  3.7 s and 3.6 s wall.
- 40 `corpus-run/pass-*` files at the top of the graph:
  3.1 s and 3.0 s wall.
- All 120 `consistency-*` files,
  sources and tests together:
  3.5 s and 3.6 s wall.
- The type check (`lint:types`) took 2.1 s in `g99b`,
  so oxlint is almost all of the lint time.

Most of a small run is fixed start-up,
and a run near the top of the graph costs what a run near the bottom does.
A package of 5k to 25k lines should lint in about 3 to 5 s against the 24 s the whole package takes now.
A gate that lints every package of the family pays the start-up once per package,
so it will take longer in total than one lint of the whole package does today.
That cost is measured after the move,
not estimated here.

### After the move, measured

The estimate in the paragraph just before this heading was wrong.
All runs are at `8d09e8e092` in the keyless worktree `w165`,
with the pre-split package at `76eb00d8fc` in `w175`,
on a host shared with another session (one-minute load 3 to 14);
the outputs are under `/var/home/user/temp/agent/tr-fan/s245/split/lint/`.

- Three rounds,
  each linting the pre-split package and then every family package once,
  back to back (`times-paired.txt`):
  the pre-split package 25.99 s and 25.46 s
  (72.60 s in the first round,
  the first run in a new worktree);
  the 36 packages 352.19 s,
  350.60 s and 332.89 s in total.
  Leaf packages took 2.4 to 10 s,
  but `corpus-pass` took 31.6 to 35.1 s,
  `probe` 24.4 to 25.4 s,
  `bench` 21.8 to 23.6 s,
  `artifact` 15.7 to 17.9 s.
  File count does not predict the time:
  `translate-lane` (44 files) took 13 s and `artifact-pool` (36 files) 3 s.
- Type-aware rules are not the cost (`cost-probe.txt`):
  one oxlint run over the whole family took 232.71 s and 263.07 s with the root config,
  and 259.98 s and 257.32 s with `typeAware` and `typeCheck` off.
- The `prefer-readonly-parameter-type` JS plugin is the whole difference (`cost-probe2.txt`),
  measured with a scratch config that drops the plugin and its four rules and changes nothing else:
  the whole family lints in 11.54 s,
  the pre-split package in 11.36 s,
  and `corpus-pass` in 2.26 s and 2.20 s.
- With the plugin,
  `corpus-pass` takes 34.95 s with the plugin's cache emptied and 4.11 s run again straight after.
  The plugin keeps one persistent cache for the repository
  (`node_modules/.cache/prefer-readonly-parameter-type` at the lockfile's directory),
  bounded at 256 MiB and 4096 entries;
  after the runs it held 3582 entries in 46 project directories and 276,255,501 bytes,
  past its byte bound,
  so each package's lint evicts what the others cached,
  and in the paired rounds every package ran cold.
- Each package is its own TypeScript project for the plugin,
  and a package's project reaches the source of every family package it imports,
  so a module shared by many packages is analysed once per project that reaches it.
  The pre-split package was one project.
  Measured (`cost-probe3.txt`):
  after `corpus-pass`,
  `probe` and `bench` linted in turn on a cache holding only `corpus-pass`'s entries,
  615 of the 1257 source files in the cache sat under more than one project,
  and the cache went from 991 to 2407 entries.
  `corpus-pass` ran 4.24 s before the other two and 4.12 s after,
  so three packages do not evict each other;
  the eviction in the paired rounds needs more of the family than that.
- The bound is the cause of the warm cost (`cost-probe4b.txt`):
  with the bounds raised to 4096 MiB and 65,536 entries in `w165`
  (uncommitted,
  in the plugin source
  and in the rebuilt `@monochromatic-dev/config-oxlint` bundle the root config loads),
  the cache emptied,
  and every package linted once and then again,
  the first round took 311.07 s and the second 70.93 s,
  with `corpus-pass` 4.87 s,
  `probe` 3.33 s and `bench` 3.13 s.
  The family's warm cache is 8269 entries and 508,090,059 bytes,
  about twice both current bounds.
  A first attempt (`cost-probe4.txt`,
  second round 312.84 s) edited only the plugin source,
  which the bundle the root config loads does not read,
  so it measured the old bounds.
- The cold round's cost is the analysis run once per project that reaches a module,
  which a larger bound does not change.

### A finding on the way

Lint of the branch worktree reported 331 warnings and 96 errors
(`TS2353`,
`TS2339`,
`TS2305`,
`TS2724` and the `no-unsafe-*` family),
all in test files.
Its `dist` was built on 2026-10-15 and the tests take their types from `dist/final/node/index.d.mts`,
so lint of tests reads the last build,
not the source.
The gate builds first,
which is why gates stay clean.
The rule for the split keeps this in mind:
a package's lint runs after its build.

## How the map was made

The map was computed from the import graph,
not from file names alone,
because names proved a poor guide to layering.

- **Graph.**
  Every tracked `.ts` under `src` was parsed with the oxc parser rolldown exposes.
  Each named import was resolved through re-export chains to the module that declares the name,
  so barrels (66 modules that declare nothing) drop out and their consumers point at the real owners.
  1384 production modules remain,
  with no name left unresolved.
- **Components.**
  The graph has 1345 strongly connected components;
  the largest holds 6 files.
  Each component stays whole in one package.
- **Topics.**
  Every module got a topic from its name family
  (text,
  provider,
  panel,
  translate,
  repair,
  consolidate,
  consistency,
  artifact,
  corpus-run,
  corpus-pass,
  corpus-probe,
  coverage-census,
  and a few more),
  with exact overrides for orchestration files whose names put them in the wrong layer
  (`document-lanes.ts`,
  `document-preparation.ts`,
  the `*-driver` files,
  the `corpus-pass*` runner files).
- **No total order on topics exists.**
  A layered assignment by topic order displaced 145k to 193k lines from their topic for the orders tried by hand,
  and 96k for the best order a search found,
  which it reached only by pulling most of `panel` into lower packages,
  because the panel,
  translate and repair families import each other's rule files in both directions.
- **Clustering.**
  Components were merged greedily by import affinity,
  first only within a topic,
  then across topics only into the cluster that imports the smaller side most and only while one topic keeps 80% of the lines,
  then clusters under 4000 lines joined a neighbour.
  No merge may close a cycle between clusters,
  so the result is acyclic by construction,
  and no cluster may pass 25k lines.

The result is 34 packages of 4,055 to 23,968 lines.
The module-to-package list is `doc/audit/translation-repair-package-split-map-2026-10-18.json`.

## The packages

Listed from the bottom of the dependency order up.
Each line gives production lines,
production files,
how many other packages of the family it imports,
and what it holds.

- `markup`:
  7,981 lines,
  34 files,
  imports none.
  Markup atoms,
  MDX tags,
  nesting bounds,
  letter and code-point classes.
- `document-structure`:
  9,909 lines,
  35 files,
  imports 1.
  The MDX parse,
  footnote graph,
  container masks,
  heading and block alignment.
- `shared-contract`:
  8,021 lines,
  35 files,
  imports 2.
  Small shared shapes many layers read:
  model cards,
  aggregate claims,
  patch and issue models,
  roster ids.
  It mixes families on purpose:
  each module is imported from several layers.
- `typography`:
  5,093 lines,
  24 files,
  imports 3.
  Quote normalization,
  semantic wrapping,
  prose typography,
  log context.
- `glossary`:
  8,222 lines,
  36 files,
  imports 4.
  The community glossary and its matching,
  address and skeleton rules,
  sheet paths,
  directory ids.
- `house-rule`:
  5,727 lines,
  23 files,
  imports 5.
  House policy,
  identity context,
  declared names,
  grading and resolution sheets.
- `provider-stream`:
  8,434 lines,
  30 files,
  imports 5.
  Credential masking,
  stream deltas and their failure detectors,
  synthetic transport.
- `provider`:
  16,317 lines,
  52 files,
  imports 7.
  Provider routing,
  budgets,
  and the OpenRouter,
  Bedrock,
  Anthropic,
  Hyper and synthetic clients.
- `stage`:
  10,034 lines,
  39 files,
  imports 8.
  Panel stage rounds,
  quorum,
  seat tally,
  prompt payloads,
  the section pairing stage
  and the block pairing stage's wire and prompt pieces.
- `slice-reading`:
  10,016 lines,
  43 files,
  imports 9.
  Image reading and OCR,
  chunked documents,
  slice coverage and pictures,
  and the block pairing stage (`pair-blocks-stage.ts`).
- `rendering-audit`:
  5,360 lines,
  20 files,
  imports 9.
  The rendering audit library,
  rendering glossary,
  page names and headings.
- `reference`:
  4,651 lines,
  15 files,
  imports 10.
  Cited references,
  attestation,
  work titles,
  lookup caches.
- `corpus-source`:
  8,204 lines,
  26 files,
  imports 10.
  The corpus reader and its git access,
  repeat groups,
  preparation identity.
- `page-pass`:
  9,530 lines,
  32 files,
  imports 10.
  Page passes that restore or splice text:
  title references,
  contributor names,
  ordinals,
  typography restore.
- `translate-floor`:
  16,212 lines,
  56 files,
  imports 13.
  The translate lane's floors and validation,
  slice delivery and cost.
- `adjudication`:
  9,876 lines,
  40 files,
  imports 14.
  Critics,
  adjudication,
  tallying,
  candidate selection,
  introduced-defect screening.
- `assembly`:
  7,957 lines,
  23 files,
  imports 9.
  Assembly integrity and repetition checks,
  and the repair records assembly reads.
- `artifact-pool`:
  4,602 lines,
  15 files,
  imports 8.
  Artifact pool,
  eligibility,
  generations,
  provenance and placement.
- `translate-stage`:
  6,281 lines,
  23 files,
  imports 12.
  The translate stage,
  run-offs,
  retries,
  and tense candidates.
- `lane-contest`:
  7,066 lines,
  21 files,
  imports 16.
  Lane comparison and contests,
  seeded errors,
  delivery coherence.
- `archive`:
  7,580 lines,
  28 files,
  imports 17.
  Archive blocks,
  disputes,
  revisions,
  coverage verdicts,
  section and block preparation.
- `consistency`:
  19,546 lines,
  55 files,
  imports 13.
  The page-level consistency pass.
- `repair`:
  16,684 lines,
  60 files,
  imports 19.
  The repair and refine lanes and the editor ensemble.
- `consolidate`:
  7,148 lines,
  26 files,
  imports 16.
  Consolidation:
  settle,
  gate,
  polish,
  standing.
- `insertion`:
  4,055 lines,
  16 files,
  imports 12.
  Insertion checks on carried and contained text,
  runner closure,
  band order.
- `translate-lane`:
  5,524 lines,
  19 files,
  imports 20.
  The translate lane over a document:
  slices,
  alignment,
  archive splicing.
- `artifact`:
  23,759 lines,
  74 files,
  imports 23.
  The two-lane artifact:
  building,
  reading,
  rebuilding,
  row equality.
- `page-check`:
  17,936 lines,
  60 files,
  imports 21.
  Published-page checks:
  Canadian spelling and dates,
  would-ship,
  page assembly,
  destinations,
  archive casing.
- `run`:
  19,610 lines,
  75 files,
  imports 21.
  Run configuration,
  seats,
  the runs lock,
  the command line,
  meters,
  spend and the ledger.
  Runners:
  `budget-sample`,
  `meter-report`,
  `cap-census`,
  `ledger-report`,
  `repeat-census`,
  `run-timing-report`,
  `spend-report`,
  `verify-published`.
- `slice-cache`:
  10,342 lines,
  50 files,
  imports 23.
  Slice and consolidation caches,
  the cache-account audit,
  slice census,
  pass schema and eligibility.
  Runners:
  `cache-account-audit`,
  `slice-census`,
  `slice-cost-report`.
- `corpus-pass`:
  23,968 lines,
  84 files,
  imports 29.
  The corpus pass and its per-entry steps.
  Runner:
  `corpus-pass`.
- `probe`:
  21,325 lines,
  105 files,
  imports 24.
  Probes and their reports:
  rendering audit,
  relabel,
  sensitivity,
  recall,
  damage,
  attribution,
  scores.
  Runners:
  `damage-sample`,
  `draw-sample`,
  `audit-sensitivity`,
  `rendering-audit-settled`,
  `rendering-audit-settled-report`,
  `probe-relabel`,
  `probe-sensitivity`,
  `probe-verify`,
  `recall-benchmark`,
  `score-agreement`,
  `score-attribution`,
  `score-crosscheck`,
  `score-probe`,
  `score-verify`,
  `judge-fidelity-probe`.
- `bench`:
  15,867 lines,
  78 files,
  imports 23.
  Benches and calibration:
  window trial,
  editor width and calibration,
  rosters,
  model health,
  sentinel.
  Runners:
  `checker-sensitivity`,
  `model-catalog`,
  `model-health`,
  `roster-card`,
  `producer-calibrate`,
  `roster-bench`,
  `sentinel-probe`,
  `displacement-probe`,
  `editor-calibrate`,
  `editor-standing-read`,
  `editor-width-probe`,
  `translate-probe`,
  `window-trial-probe`.
- `coverage-census`:
  9,375 lines,
  32 files,
  imports 15.
  The coverage census and its control and probe.
  Runners:
  `coverage-census`,
  `coverage-control-probe`,
  `coverage-probe`.

Every package name is `@monochromatic-dev/translation-repair-<name>`,
after the `package/figma/kiwi` precedent for a category directory,
and every directory segment is singular.

## Rules the move keeps

### No behaviour change

The move changes paths,
import specifiers,
package manifests and build configuration only.
No function body changes in the move commits.
Barrels are dissolved:
every import names the package that declares the name.
Each package gets its own `src/index.ts` exporting what its siblings and its tests use.

### One copy of every module in a process

Production code checks identity on family classes:
46 `instanceof` sites,
9 of them on `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) and others on the stream errors,
`StatedRefusalError`,
`ArtifactParseError` and their kin.
63 test fixtures that use production code are imported by tests in more than one package
(`stage-settings.test-fixture.ts` alone by tests in 24).
If a package's build inlined its siblings,
as `doc/decision/workspace-ts-source-imports.md` makes the default (`deps.alwaysBundle`),
a test would load its own build's inlined copy of a sibling next to the copy a shared fixture loads,
and an error thrown by one copy would fail `instanceof` against the other.

So inside the family,
library builds keep family siblings external:
a package's `dist` imports `@monochromatic-dev/translation-repair-<sibling>`,
which resolves through the sibling's `"."` export to the sibling's own `dist`.
Every test and fixture then reaches each module through exactly one file.
Runner bundles still inline everything,
so the shipped commands stay self-contained.
This is a family-scoped exception to the decision document's inlining default,
taken because nothing outside the family imports these packages and the package was never published
(npm answers 404 for `@monochromatic-dev/module-translation-repair`).
The cost is a build order:
a package's build and tests depend on its siblings' builds,
declared through `depends` in each `mise.toml`.
Lint of a package's tests already depends on its own build (see "A finding on the way"),
so the order adds sibling builds to a dependency that exists.

### Tests

- A test keeps importing built output,
  which the `test-import` rule requires
  (`package/oxlint-plugin/test-import/README.md`).
- A test lives in the package of the module it tests,
  found by its base name (`foo.unit.test.ts` tests `foo.ts`).
- 209 tests also need a name from a package that imports their subject's package.
  Each lives instead in the lowest package that reaches every name it needs,
  measured per test against the map.
- One test,
  `corpus-run/cli-refusal.unit.test.ts`,
  needs names from `bench`,
  `coverage-census` and `artifact` at once,
  and no package reaches all three.
  It splits into one file per runner,
  each beside the runner it checks.
- 111 test files have no subject module:
  the source scans,
  fixtures that import no production code (`scratch-dir`,
  `capturing-logger` and their kin),
  and runner tests that start built commands.
  The source scans move to a package of their own (`package/translation-repair/source-scan`)
  that reads every family package's `src`.
  Fixtures with no production imports move to `package/translation-repair/test-fixture`,
  exported through its `/ts` subpath.
  Runner tests go beside their runner.
- The suite after the move must hold the same 2482 passing cases under the same names.

### Runners, the coverage census and the scans

- Each runner entry stays in the package that owns its entry file,
  with its `mise` task and its built path moved with it.
  Docs and runbooks that start runners name the new task.
- A runner's pipeline digest (`corpus-run/pipeline-digest.ts`) hashes the runner's own `dist/final/node` directory,
  and artifacts and slice caches key their generation on it.
  A runner that imported sibling builds at run time would leave sibling code out of its digest,
  so a sibling change could resume foreign slices silently.
  Runner bundles therefore keep inlining every family package,
  in a build of their own;
  only the library index build keeps family siblings external.
- The coverage census builds coverage bundles of the whole family,
  so it reads every package's sources,
  not only its own.
- Production-reach guards (dead code,
  unused values) and the other whole-package source scans
  measure the family as one program,
  so a function one package exports and another uses still counts as reached.
- `build-entries.ts` moves to `coverage-census` (the map said `stage`;
  only the census reads it in production).
  The move gives every runner package its own `src/build-entries.ts` listing its own runners,
  so the census must read every package's list;
  until it does it sees a subset,
  and a control holds it to the pre-split runner source set.

### Readers that find a file by path

The move rewrites import specifiers only.
A scan over every string literal and template text in the package
(`literal-paths.ts` in the split's scratch directory)
lists the code that names a family file as text and resolves it by path or key at run time,
which a move breaks without a type or build error:

- Source scans and their allowlist fixtures,
  keyed by `src/<file>` path.
  The 48 scans (the `source-scans` task plus every importer of the scan fixtures)
  and the 11 fixtures only scans import
  (the `position-references-*` allowlists and the scan helpers)
  all move to `source-scan`,
  which reads every family package's `src`.
- The coverage census,
  through `build-entries.ts`.
- The cache-account audit (`corpus-run/cache-account-run.ts`),
  which lists its own package's `src` with `git ls-files` to find the cache versions;
  it must list every family package's `src`.
- `semantic-wrap-binary.ts`,
  which finds `semantic-wrap.config.jsonc` at `../../../src` from the running file.
  Runner bundles in other packages inline it,
  so the build copies the config into `dist/final/node` and the built module reads it there;
  the pipeline digest then covers the config's bytes,
  which it did not before although the config changes output line breaks.
  This lands before the move as its own change.
  The config itself moves to `typography`,
  the only file under `src` that is not TypeScript.
- Tests and fixtures that start a child over the built package
  (`built-child.test-fixture.ts`,
   `child-environment.test-fixture.ts`,
   `published-run.test-fixture.ts`,
   `parse-mdx-cold-refusal.unit.test.ts`,
   `console-warn-lines.unit.test.ts`
   and the census and runs-lock tests,
   15 files):
  they join `dist/final/node` or a sibling file to their own directory,
  and some read classes off the built index by name inside the child's source text.
  Each must resolve the package that defines what it needs,
  by `import.meta.resolve('<package name>')` for a built entry
  and `<package name>/ts/<path>` for a source file.
- `typography-prose-mask.unit.test.ts` reads five rule sources by path;
  it moves with the scans' path reading.

Messages and case names that mention a file by its base name stay true after the move
and need no change.

## The pilot

Run in a keyless worktree at `fee1f64f15` with `markup` and `test-fixture` moved out
and the rest left in place:

- Both packages build;
  the remaining library index imports 83 names from `@monochromatic-dev/translation-repair-markup`
  and declares none of markup's 104 exports,
  and its declarations file imports the same bare name.
  Runner bundles inline markup.
- Markup's tests pass 45 cases,
  the fixture package's 6,
  the remaining package's 2429.
  Every case the baseline gate g99b passed and the pilot did not
  belongs to a reader in the list above:
  the source scans,
  the message-class inventory,
  the task-runner guard reach
  and the stack-edge child test.

## The full dry run

Run in keyless worktree `w165` at `2e79bef413` with every file moved by the map
(36 packages;
`source-scan` and `test-fixture` own no production module and have no build):

- Every package with production code builds.
- Tests fail in 9 packages:
  `adjudication`,
  `bench`,
  `corpus-pass`,
  `coverage-census`,
  `probe`,
  `run`,
  `slice-cache`,
  `source-scan`
  and `typography`.
  Measured per describe against gate g99b,
  100 suites lose cases;
  every one belongs to a reader in the list in "Readers that find a file by path":
  the runner tests that start a built entry ("as built"),
  the source scans and their positive controls,
  the census and cache-account audit runs,
  the child-environment and keyless-child tests,
  and `proseMaskReading`.
- Every task of the old `mise.toml` is carried by one of the new packages.
- Outside the old package,
  only two files name it,
  and file-enforcer generates both:
  the root `mise.toml` PATH list,
  globbed from installed `node_modules/.bin` directories,
  and the pnpr publish list.
  The move runs file-enforcer in the branch worktree after its install.

The semantic-wrap config change (`3d844e0315`,
`76eb00d8fc`) landed after the dry run,
so the move runs again at `76eb00d8fc`,
and that commit's gate g100 (suite 2484 passing,
lint,
types and scans clean)
replaces `94ebefa1e3` as the suite baseline in "Controls".

## Docs

Done on 2026-10-18 in the commits that follow the source move.

- Docs about the family as a whole
  (the ledger,
   the prevention doc,
   the writer brief and the rest of the 133 package-local `.md` files)
  moved with `git mv`,
  contents unchanged,
  to `package/translation-repair/doc/` under the same file names,
  beside the packages,
  after the `package/music-player/README.md` precedent for a category-level doc.
  The old README became the family README `package/translation-repair/README.md`,
  rewritten for the split layout:
  it keeps the standing constraints,
  the English variety rule and the contract,
  and adds the list of the 36 packages and where the docs live.
- A doc about one package moves into that package's `doc/`.
  A doc that names code in several packages stays at family level.
  Two docs left the family level:
  - `pictures.md` moved to `package/translation-repair/slice-reading/doc/pictures.md`.
    It describes the picture reading that `slice-reading` declares
    (the OCR gate,
    the corroboration threshold,
    what is sent to a reader),
    and names `run` and `corpus-pass` only for the reader seats and a configuration constant.
  - `semantic-wrap-sync-spawn.md` moved to `package/translation-repair/typography/doc/semantic-wrap-sync-spawn.md`.
    It justifies the one `oxlint-disable` in `typography/src/lint-program.ts`,
    which cites it,
    and names the callers in other packages only as the reason the spawn stays synchronous.
- Docs that were considered and stay at family level:
  `configuration.md` (credentials and settings read by `run`,
  `provider`,
  `corpus-source`,
  `probe` and others),
  `provider-availability.md`,
  `redaction-timing.md` (a policy for all the family's docs),
  `repetition.md` (names `assembly`,
  the `probe` instruments and the parse in `markup`),
  `roster-changes.md`,
  `seats-and-calibration.md` and `slice-context.md`.
- The old `LICENSES/` directory was removed.
  It had no manifest,
  every family package carries its own texts,
  and `package/music-player` has none at its root either.
- Living docs that cite a moved path are rewritten to the new path:
  runbooks,
  troubleshooting notes,
  decisions,
  the document map,
  the canonical handover,
  the prevention doc,
  the design commitments,
  the configuration doc,
  TSDoc and comments,
  and the package READMEs.
  A source file is named `package/translation-repair/<package>/src/<path>`,
  with the package taken from the map file or the tree;
  a directory or glob is `package/translation-repair/*/src/...`;
  a task is `//package/translation-repair/<package>:<task>` for its owning package,
  and `<package>` stands for the tasks every package has (`build`,
  `lint`,
  `test:unit`);
  `dist/final/node/<command>.mjs` goes with the package that owns the command.
  A file that left the tree before the split keeps its written path with the note `(removed before 2026-10-18)`.
  Relative Markdown links are recomputed from the new location.
- Historical records keep the path as written,
  with an in-place note after the path or its code span:
  `(moved on 2026-10-18 to \`<new path>\`)`.
  A path inside a fenced block gets no note inside the block,
  since the block holds commands and code,
  and a sentence after the block names each old path and its new one.
  A Markdown link target in a historical record is the one exception:
  it is retargeted so the link resolves,
  and its visible text is left as written.
  A bare `src/<path>` in a moved record names the old package's `src`:
  a record with fewer than four such paths gets a note on each,
  and a record with four or more gets one note at the top instead.
  Twenty-two audit-ledger parts carry that note,
  and the map file `doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.
- JSON audit records under `doc/audit/` stay as written,
  and so does every citation of `package/module/translation-repair/node_modules/.monochromatic`,
  the kept evidence directory,
  which is ignored by git and was not moved.
- The source-scan docs reader (`readPackageDocPaths`) reads the family README,
  `package/translation-repair/doc/`,
  and each package's `README.md` and `doc/`.
  Control:
  the Markdown set it returns was 169 paths before the move and is 169 after;
  mapped through the move table the two sets are identical.
  Scan keys of the family-level docs (`doc/<name>.md`,
  `README.md`) are unchanged,
  since they were relative to the old package root and are now relative to the family directory.

## Controls

The pre-split state is recorded at `94ebefa1e3`,
and the suite again at `76eb00d8fc` (gate g100),
whose semantic-wrap change moves the pipeline digest:

- suite 2482 passing cases and the blankness scan green;
  2484 at `76eb00d8fc`;
- oxlint 0 warnings and 0 errors;
- the TianqiChen66625 decisions replay byte-identical to the n171 build's output except for its label;
- the corpus measure over the 92 pinned pages.

After the move,
in a fresh keyless worktree:

- every package builds,
  lints clean and type-checks;
- the suite holds the same case names and counts;
- every source scan passes;
- the coverage census and the cache-account audit read the same source files as before the move,
  under their new paths;
- the replay and the corpus measure,
  run against the new `corpus-pass` runner bundle and the new library builds,
  match the pre-split outputs byte for byte except for labels;
- full-family lint time and single-package lint time are measured and recorded here.

### Measured after the move

All at `8d09e8e092` in the keyless worktree `w165`,
by the family gate g101 (`s245/split/gate-family.ts`)
and the controls after it (`s245/split/control/run-controls.ts`);
a path written `s245/...` is under `/var/home/user/temp/agent/tr-fan`.

- Every step of g101 exited 0:
  install,
  34 builds,
  and each of the 36 packages' `lint:oxlint`,
  `lint:types` and `test:unit`,
  then `source-scans`.
- The suites printed 2506 `[PASS]` lines and no `[FAIL]` line,
  against 2484 in g100.
  A `[PASS]` line names a top-level describe with its cases,
  so the case-name comparison is by line,
  as multisets (`s245/split/compare-names.ts`):
  6 lines of g100 are gone and 28 are new.
  Each gone line is still there in another shape:
  three gained a case the split added
  (the start guard names its owner's task,
  every runner has an owner,
  the audit finds a version set before the split),
  and three were divided between the packages whose runners or files they test
  (the 17 unreadable-corpus-commit refusals into four lines that hold all 17,
  the unopenable-file cases into two,
  the keyless-child cases into two).
  The other new lines test code the split added:
  the census's family reading,
  the audit's family sources and the command-owner table.
- The coverage census and the cache-account audit,
  read against the single package at `2083d7bec1`
  (`s245/split/control/census-audit-control.ts`,
  output in `s245/split/control/out2/control-output.txt`):
  runner sources 43 before and 43 after,
  and the 43 the build-entry lists name by owner are the 43 the bundles carry;
  library sources 1310 before and 1319 after,
  every after file mapped or new by construction
  (the build-entry lists,
  the family reading,
  the cache-account sources and the command-owner table);
  the audit reads 1639 files before and after,
  with none missing;
  the 8 cache-version constants read the same declarations and values.
  The first run of this control said DIFFER for the audit:
  it read git's renames end to end,
  and two fixtures moved as exact renames by `4bd1aea8d5` were edited afterwards below git's rename threshold;
  read at the move commit,
  the audit matches.
- The TianqiChen66625 decisions replay,
  run against the family's library builds
  (every function it calls comes from `consistency`),
  is byte-identical to the pre-split replay except for its label.
- The corpus measure over the 92 pinned pages,
  the pre-split build against the family's builds
  (1237 slices,
  55,605 word-locator comparisons),
  finds no difference in any corpus row;
  its typography control,
  which hands the new side different definitions,
  shows its difference.

## Order of work

1.  Commit this document and the map.
2.  One commit moves every file with `git mv` and rewrites every import,
    by a scratch script that reads the map;
    the tree may not build at that commit,
    and its message says so.
3.  Package scaffolding:
    `package.json`,
    `mise.toml`,
    `tsconfig.json`,
    rolldown configs and `README.md` for each package.
4.  Tests re-homed per the test rules,
    the scan and fixture packages,
    the coverage census reading the family.
5.  Docs moved and path references rewritten.
6.  The controls,
    then this document updated with the measured lint times.

## Open questions

None block the move.
The externals exception is open to veto;
the alternative is tests importing family source through `/ts`,
which removes the build order but stops testing built output,
which the `test-import` rule exists to prevent.
