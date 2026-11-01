# @monochromatic-dev/translation-repair-typography

Prose typography for the pipeline:
quote normalization,
semantic line wrapping through the pinned lint binary,
prose masks,
log context,
and small process helpers.
It is its own package because every text package that builds on it needs it and it needs only `markup`,
`document-structure` and `shared-contract` below it.
It also owns `src/semantic-wrap.config.jsonc`,
the only non-TypeScript file under any family `src`,
which the build copies into `dist/final/node` beside the built module.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Quote normalization:
  straighten and normalize quotation marks and punctuation,
  and locate a quote inside a text.
  Names:
  `straightenQuotes`,
  `straightenProseQuotes`,
  `normalizePunctuation`,
  `collapseLineBreaks`,
  `nestedSinglePairs`.
  Declared in `quote-normalize.ts` and `nested-single-quotes.ts`.
- Semantic wrapping:
  wrap replacement text at semantic breaks by running the pinned lint binary,
  and refuse a missing or mismatched binary.
  Names:
  `wrapReplacementText`,
  `wrapWithLintBinary`,
  `PINNED_LINT_BINARY_VERSION`,
  `LintBinaryMissingError`,
  `LintBinaryVersionError`,
  `LintBinaryRunError`.
  Declared in `semantic-wrap.ts`,
  `semantic-wrap-binary.ts`,
  `semantic-wrap-binary-constants.ts`,
  `semantic-wrap-binary-error.ts`,
  and `semantic-wrap-run-error.ts`.
- Prose masks and quote location:
  mask everything but prose,
  write an edit on the shown view back to the source,
  and find a critic's quote in a document.
  Names:
  `fragmentProseMask`,
  `proseMaskReading`,
  `shownViewOf`,
  `writtenBack`,
  `locateQuote`.
  Declared in `typography-prose-mask.ts`,
  `typography-shown-view.ts`,
  and `locate-quote.ts`.
- Log context:
  carry the entry,
  attempt and slice a log line belongs to through async work.
  Names:
  `currentLogContext`,
  `inEntryLogContext`,
  `inSliceLogContext`,
  `contextRoot`,
  `sliceTagged`.
  Declared in `log-context.ts`.
- Process and file helpers:
  a child environment that holds no credential or package setting,
  program start with a checked exit,
  and missing-path handling.
  Names:
  `childEnvironment`,
  `isCredentialName`,
  `startProgram`,
  `refuseFailedRun`,
  `isMissingPathError`,
  `rethrowUnlessMissingPath`,
  `readTextOrEmptyIfMissing`.
  Declared in `child-process-environment.ts`,
  `lint-program.ts`,
  `missing-path-error.ts`,
  and `read-text-if-present.ts`.
- Character and count helpers:
  case classes,
  code points that render as nothing,
  lone surrogates,
  and count words for log lines.
  Names:
  `isCapitalLetter`,
  `isCasedLetter`,
  `rendersAsNothing`,
  `loneSurrogateIndex`,
  `wordForCount`,
  `howOften`.
  Declared in `cased-letters.ts`,
  `renders-as-nothing.ts`,
  `lone-surrogate.ts`,
  and `count-word.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `markup`
- `shared-contract`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `page-check`
- `page-pass`
- `probe`
- `provider`
- `provider-stream`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`

Packages that list it only in `devDependencies`:

- `source-scan`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/typography:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/typography:test:unit`,
or build and test together with `mise run //package/translation-repair/typography:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `console-warn-lines.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Why the semantic wrap starts its linter synchronously](doc/semantic-wrap-sync-spawn.md):
  the one synchronous process start and the lint suppression that covers it.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
