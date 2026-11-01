# @monochromatic-dev/translation-repair-source-scan

The package-wide source scans of the translation-repair family:
unit tests that read the source,
docs and task files of the family and fail when a new class,
function,
export,
link,
command-line read,
wall-clock read or similar appears without the allowance its rule asks for.
It holds no production code and has no build.
It is its own package because the scans measure the family as one program,
so a function one package exports and another uses still counts as reached.

## What it provides

- The scans:
  each `*.unit.test.ts` file is one scan,
  for example `dead-code.unit.test.ts`,
  `unused-imports.unit.test.ts`,
  `duplicate-bodies.unit.test.ts`,
  `wall-clock-reads.unit.test.ts`,
  `message-names-only.unit.test.ts`,
  `position-references.unit.test.ts`,
  `living-doc-links.unit.test.ts`,
  `markdown-lint-clean.unit.test.ts` and `own-unit-tests.unit.test.ts`.
  Every scan reads the family as one program:
  `readPackageSource` reads the TypeScript under the `src` of every family package,
  and `readPackageTexts` adds each family package's README,
  docs and task file,
  plus the family docs and README.
  Files are keyed by package directory and path under that package's `src`,
  such as `run/corpus-run/cli-refusal.ts`.
  `readRepositoryTexts` reads the living repository docs from the repository root,
  four levels above `src`.
- Scan fixtures:
  files named `*.test-fixture.ts` that only scans import.
  `family-source.test-fixture.ts` reads the family's source and task files and resolves imports between packages:
  `familyPackages`,
  `readPackageSource`,
  `readTaskFiles`,
  `runnerEntriesOf` and `declaringModule`.
  `source-scan.test-fixture.ts` parses TypeScript with the parser rolldown exposes and walks the syntax tree.
  `corpus-pass` imports it too,
  for the search over the family's source that keeps its list of answer-schema producers honest:
  `parseSource` parses each file,
  and `nodesUnder`,
  `parentsOf`,
  `isTreeNode`,
  `identifierName`,
  `memberName`,
  `literalText` and `unwrapped` read the tree.
  `source-text-scan.test-fixture.ts` holds the text helpers `matchingClose`,
  `topLevelPieces` and `withoutBlockAndLineComments`.
  `prose-texts.test-fixture.ts` reads package and repository texts through `readPackageTexts` and
  `readRepositoryTexts`.
  `living-docs.test-fixture.ts` lists the living repository docs through `readLivingRepositoryDocs`.
  `tsdoc-example-scan.test-fixture.ts` checks `@example` blocks against their declarations through
  `exampleFindingsOf`.
  The `position-references*.test-fixture.ts` files hold the vocabulary,
  text,
  context and exemption lists of the position-reference scan.
- The `source-scans` task:
  runs the listed scans through `test:unit`,
  to be run beside the named tests of every commit that touches the package's source,
  docs,
  README or task file,
  because the clock-time,
  task-list-number,
  living-doc-link,
  Markdown lint and position-reference scans read the docs too.
  Its description says it spends no quota.
- The `test:rotation-sweep` task (ledger B701):
  runs every unit test file of every family package that can reach the stage package once per salt from 0 to 9,
  each salt moving the start of `benchRotation` by that many seats,
  and lists the cases a salt fails that salt 0 passes,
  with the file,
  the case and the salts.
  The move is made by a loader under `src/rotation-sweep/` that rewrites `benchRotation` as the built bundle loads,
  so no production source reads the environment and no test-only seam sits in `stage`.
  It builds the packages first,
  costs ten full runs of the family's unit tests,
  and is run before a sheet text change lands and in the readiness pre-launch checks,
  never per commit.
  Narrow it with `-- --package <directory>` and `-- --salt <n>`,
  each repeatable.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

No family package is a production dependency,
because there is no production code.

Development dependencies other than `test-fixture` (`devDependencies`):

- `adjudication`
  (its keyless child runner,
  for the rotation sweep and the Markdown lint scan)
- `document-structure`
- `markup`
- `run`
- `typography`

### Used by

No family package lists it in `dependencies`.

Packages that list it only in `devDependencies`:

- `corpus-pass`
  (its schema cases import the scan fixtures through `@monochromatic-dev/translation-repair-source-scan/ts/`)

## Tests

This package has no build and no `dist`:
its tests are the scans,
and `test:unit` runs them.
The scans import the built entries of `document-structure`,
`markup`,
`run` and `typography` through their package names,
which resolve to each sibling's `dist/final/node/index.mjs`.
The scans read the source of every family package as text,
`coverage-census` among them,
so none of them is a dependency.
The `test:unit` task declares no `depends`,
so build those four first with `mise run //package/translation-repair/<sibling>:build`.
The unit tests of the rotation sweep and the Markdown lint scan also load the built `adjudication`,
which the `test:rotation-sweep` task builds with every other package it sweeps
and the `source-scans` task builds beside the four.
The `source-scans` task builds them first.

Run the scans with `mise run //package/translation-repair/source-scan:test:unit`,
or run the listed set with `mise run //package/translation-repair/source-scan:source-scans`.
The shared helper `scan-findings.test-fixture.ts` comes from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
