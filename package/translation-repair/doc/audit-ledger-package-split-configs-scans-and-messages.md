# Audit ledger: the package split, configs, scans and messages

The defects of the split of `package/module/translation-repair` into 36 packages under `package/translation-repair`
(the move commit is `4bd1aea8d5`,
of 2026-10-18),
found by the dry runs,
the slice agents and the checks that followed it:
a build config that lacked a plugin (B648),
a task that depended on a build the package lacks (B649),
exports of files that do not exist (B650),
a scan that read nothing after the move (B651),
refusals that named a task that no longer exists (B652),
audit queries that covered only the new paths (B653),
workspace cycles made by tests (B654),
and a lint ignore pattern that matched a package (B655).
The process mistakes of the same work are filed in
[Audit ledger: process mistakes, the package split](audit-ledger-process-the-package-split.md).
The plan of the split is `doc/planning/translation-repair-package-split.md`.

Written on 2026-10-18 (UTC) from the split lead's notes (`s245/split/ledger-notes.md`)
and the messages of the commits named in each entry.
Paths here are written in the layout after the split.
The notes called two of the entries open (B650 and B654),
and two more were being fixed when they were written (B652 and B653);
the commits named in the entries fixed all four.
One more finding of the notes belongs to another record and is not filed here:
the TSDoc of `projectSourceText` in the `prefer-readonly-parameter-type` plugin says that its cache entries are
"keyed by path and shared across projects",
while every entry's address includes its project key.
That is for the plugin's own ledger,
and the sentence still stood in the tree this file was written from.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B648: the scaffolded library configs did not copy the semantic-wrap config

Found on 2026-10-18 in the split's move run
(the message of the fix says so;
the fix was checked in a keyless worktree).

Outcome written on 2026-10-18.
Fixed in `2083d7bec1` (test `ef1bf117e4`).
The copy itself was added before the move by `76eb00d8fc` (test `3d844e0315`).

What the run showed:
no family build copied `semantic-wrap.config.jsonc`,
and every test that wraps a passage failed with `LintBinaryRunError`
(`monochromatic-lint` exiting 2).
The typography package's library build needs the copy beside its built module.

The cause:
`76eb00d8fc` made the package's builds copy the config through `nodeConfig`'s `extraPlugins`,
because a lookup relative to `../../../src` would point into the wrong package once the semantic-wrap module is
inlined into the runner bundles of several packages.
The scaffolded family library configs of the move had no way to pass that plugin:
`familyLibraryNodeConfig` did not take `extraPlugins`.

Fixed:
- `familyLibraryNodeConfig` in `package/config/rolldown` takes `extraPlugins`,
  passed through to `nodeConfig`,
  which appends them after the shared plugins (`2083d7bec1`).
  The case `ef1bf117e4` fails against the earlier function:
  the last plugin is a shared one.
- The copy that the family library builds are to carry is the one `76eb00d8fc` added:
  a `copiedAssetsPlugin` that copies `semantic-wrap.config.jsonc` into `dist/final/node`,
  read by `semanticWrapConfigPath` beside the running module.
  `3d844e0315` pins the copy as byte-identical to the source.

Prevention:
the notes name none.

## B649: the `source-scans` task depended on a build task the package lacks

Found on 2026-10-18 (the notes do not say by whom or by which run).

Outcome written on 2026-10-18.
Fixed in `cd9484b9dc`.

The task `source-scans` of `package/translation-repair/source-scan` declared `depends = ["build"]`,
and the package has no `build` task.
The scans load family packages by their built entries.

Fixed in `cd9484b9dc`:
the task depends on the builds of `document-structure`,
`markup`,
`run` and `typography`,
with a comment saying the scans load these packages by their built entries.

Prevention:
the notes name none.

## B650: two packages exported a file they do not have

Found on 2026-10-18 during the split (the notes call it open).

Outcome written on 2026-10-18.
Fixed in `3bab5e97fd` for `test-fixture` (the notes name `68f97bcded` as the commit cherry-picked as that fix)
and in `740f62a633` for `source-scan`.

The `package.json` of `package/translation-repair/test-fixture` and of `package/translation-repair/source-scan`
exported `./ts` to `./src/index.ts`,
a file neither package has.
Every importer reaches a fixture through `./ts/<file>`,
which `./ts/*` serves,
and no package in the family imports either package by the bare key.

Fixed:
the exact key is removed and `./ts/*` stays,
so the exports describe what exists;
no `index.ts` is added.
`740f62a633` also drops the devDependency of `source-scan` on `coverage-census`:
a search of the package finds the name only in `package.json`,
since the scans read the census sources as text and import nothing from it.
The entry is removed from `package.json` and `pnpm-lock.yaml`,
which `pnpm install` regenerated,
and from the two READMEs that listed it.

Prevention:
the notes name none.

## B651: the TSDoc example scan read only its own directory after the move

Found on 2026-10-18 by the scan slice agent.

Outcome written on 2026-10-18.
Fixed in `b9c4ffd985` (landed as `2bd2d2ab4f`).

`tsdoc-example-scan.unit.test.ts` listed the scan package's own directory through its own URL.
After the split its package-wide case therefore read only the scan tests,
and passed without reading any source of the family.
The case now reads every family package through `readPackageSource`,
tests and fixtures included as before,
and finds nothing.

A scan whose input set shrinks to nothing passes vacuously.
The controls of the census and of the cache audit are recorded as matching in `389f03897c`.

Prevention:
the controls of the split compare each scan's read set before and after.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Scans and controls across a move".

## B652: refusals and guard messages named a task that no longer exists

Found on 2026-10-18 during the split (the notes had it being fixed in `w173`).

Outcome written on 2026-10-18.
Red `15c42b9fe3`,
fixed in `5fe393041d`.

The refusals still told an operator to run `//package/module/translation-repair:<command>`,
a task path that stopped existing when the family was split,
and the tests pinned the stale text.
The task lives in the `mise.toml` of the package that builds the runner.

Fixed in `5fe393041d`:
- `taskRunnerVerdict` in the adjudication package takes the owner of the command as a required input and writes its
  task from it,
  since a guard that guessed a package would name a task that does not exist.
- The run package keeps `COMMAND_OWNERS`,
  one entry per command that `COMMAND_LINES` declares
  (the compiler holds the two to each other),
  and `taskPathOf`;
  the source scan holds each entry to the `build-entries.ts` that lists the runner.
- The refusals of the cache-account audit and of the census name their own packages' tasks,
  and the comments of the probe,
  bench and corpus-pass runners name theirs.
- The case `15c42b9fe3` asks every runner refusal to name the task of the owning package,
  and fails until the fix.
  The built guard test also passes the owner to its second `guardRefusal` call,
  which the tests commit had missed.

Two merged slices conflicted here:
`w171` wrote the new task paths literally and `w173` derived them,
and the derived paths were kept (`taskPathOf` over the `COMMAND_OWNERS` table,
checked by a scan case);
the comment text of `w171` also conflicted with that of `w169`,
and `w169`'s was kept,
since it describes the family reading.

Prevention:
the notes name none.

## B653: the cache-account audit's history queries covered only the new paths

Found on 2026-10-18 during the split (the notes had it being fixed in `w173`).

Outcome written on 2026-10-18.
Red `4c32646241`,
fixed in `4e64eed3ef`.

A cache version set before the split was set at `package/module/translation-repair/src`.
The git log and pickaxe queries of the audit named only the family's `src`,
so the commit that moved the file was reported as the setter of the version,
and the count of source commits began at the move.

Fixed in `4e64eed3ef`:
the pickaxe log,
the diff of each candidate and the log since the earliest setting
name the pre-split directory beside the family's `src`,
with its tests left out of the last.
`preSplitSourcesOf` and `preSplitWithoutTestsOf` add it only where the sources are the family's;
a package outside the family keeps its own `src`.
The listing of tracked files is unchanged,
since those files stand in the family now.
Both functions are exported and have cases of their own.

Prevention:
the notes name none.

## B654: tests that started the runners of other packages made workspace cycles

Found on 2026-10-18 and recorded by the split lead as added by slice B
(the notes recorded the question of where the tests belong as open).

Outcome written on 2026-10-18.
Fixed in `96d07d277b` (tests) and `57d2d51c4c` (dependencies).

The tests of `adjudication` started the runners of five other packages,
the tests of `run` started those of `probe`,
and the tests of `probe` started those of `bench`.
Each of those packages depends on the package whose tests started it,
and the test-only devDependencies and `test:unit` depends made workspace cycles.

Fixed:
- Each built command's cases now run in the package that owns the command,
  so no test needs a package that needs it (`96d07d277b`).
  Shared pieces moved to fixtures:
  `variablesSet` into `child-environment.test-fixture.ts`,
  the corpus pin suite as `pinReadersSuite` over a list of commands with the entry of the package that owns each,
  the guard suite as `guardedSuite` in `task-runner-guard-built.test-fixture.ts`,
  and the unopenable file pieces once in `unopenable-file.test-fixture.ts`.
  The case total by a count of `it(` calls over every `src` file of the family is 11,657 before and after.
- The five devDependencies of `adjudication`,
  the one of `probe` and the one of `run`,
  with the `test:unit` depends that built those packages first,
  are removed,
  and the lockfile changes by the seven importer links only (`57d2d51c4c`).
  `pnpm install` no longer warns of a cycle among the family's packages.

The first merge of the slices also conflicted over two shared fixtures,
`runBuiltCommand` and `runProbeOver` (M243).
The merged form requires the owning package's entry from every caller and drops the directory fallback
(`2ec8ff1581`).

Prevention:
the notes name none.

## B655: a package directory named like a repository-wide lint ignore was never linted

Found on 2026-10-18 by `w172`,
which cleared the lint output of the move,
after the lint task of the package had found no file.

Outcome written on 2026-10-18.
Fixed in `578df705ea` (a package-local config) and,
as the root cause,
in `8b103ac6e7`.

The shared oxlint base config ignores `**/test-fixture/**`,
a pattern written for the fixture packages under `package/test-fixture/`.
It also matched `package/translation-repair/test-fixture`,
so the lint task of that package found no file to lint,
and a lint of 0 files reports nothing.

Fixed:
- `578df705ea` gives the package a local `oxlint.config.ts` that spreads the shared one,
  since oxlint matches ignore patterns against paths relative to the loaded configuration;
  the package then lints 47 files with no finding.
- `8b103ac6e7` spells the pattern `**/package/test-fixture/**`:
  the root lint still finds 0 files in the fixture packages (probed with one invalid file)
  and lints the translation-repair package (46 files,
  0 warnings,
  0 errors).
  The per-package workaround config is removed.

Prevention:
a new package's first lint names a nonzero file count,
and "No files found to lint" is a failure and not a pass.
Recorded in `mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"A move's dry run".
