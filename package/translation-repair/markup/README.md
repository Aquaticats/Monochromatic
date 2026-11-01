# @monochromatic-dev/translation-repair-markup

The lowest layer of the translation-repair pipeline:
the atoms every other package reads text through.
It holds the character and script classes,
code-point helpers,
MDX tag and markup-atom scanners and the nesting bounds that keep the parsers from exhausting the stack.
It imports no other family package,
so it comes first in the build order,
and every other package with production code lists it,
`consolidate` in `devDependencies` only because its tests are the only files that import it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Letter and script classes:
  decide whether a character is ASCII,
  Latin or Han,
  and fold Latin words for comparison.
  Names:
  `isAsciiLetter`,
  `isAsciiDigits`,
  `isHanCharacter`,
  `isHanOnly`,
  `isLatinLetter`,
  `latinWordSpans`,
  `foldLatinWord`.
  Declared in `ascii-letters.ts`,
  `han-only-text.ts`,
  and `latin-letters.ts`.
- Code points:
  measure and order text by code point rather than by UTF-16 unit.
  Names:
  `codePointAt`,
  `codePointLength`,
  `codePointCount`,
  `compareCodePoints`,
  `textsInCodePointOrder`.
  Declared in `code-points.ts`.
- MDX tags and markup atoms:
  recognize where an MDX tag opens,
  read tag names,
  find the openers and closers of named inline container tags,
  scan the markup atoms an edit must carry through,
  measure what an edit lost,
  and refuse the edits of a patch that lose markup the patch writes nowhere else.
  Names:
  `opensMdxTag`,
  `readTagName`,
  `inlineContainerTags`,
  `scanMarkupAtoms`,
  `markupDelta`,
  `settleMarkupMoves`.
  Declared in `mdx-tag-start.ts`,
  `mdx-tag-name.ts`,
  `inline-container-tags.ts`,
  `markup-atom-scan.ts`,
  and `markup-atom-preservation.ts`.
- Nesting bounds:
  refuse text nested deeper than the parsers can survive,
  and tell a real stack overflow from a bound hit.
  Names:
  `firstNestingExcess`,
  `nestingAccount`,
  `NestingBoundError`,
  `isStackOverflow`,
  `NESTING_BOUND`,
  `DELIMITER_BOUND`.
  Declared in `nesting-bound.ts` and `nesting-vocabulary.ts`.
- Lifecycle stamps:
  write the wall clock time that a line of the pass carries,
  and the attempt where the line has one (ledger B960).
  Names:
  `lifecycleStamp`,
  `lifecycleClockStamp`,
  `wallClockNow`,
  `LifecycleAttemptError`.
  Declared in `lifecycle-stamp.ts`.
- Edits and spans:
  apply disjoint edits to a text,
  find closed mark spans and fenced lines,
  and walk mdast nodes.
  Names:
  `spliceDisjointEdits`,
  `OverlappingEditsError`,
  `closedMarkSpans`,
  `fencedLineFlags`,
  `treeNodes`.
  Declared in `disjoint-splice.ts`,
  `closed-mark-spans.ts`,
  `code-fence-lines.ts`,
  and `mdast-tree-nodes.ts`.
- Reading checks and small shared helpers:
  judge whether model readings of a picture agree or amount to a refusal,
  plus the error-name,
  clock,
  status and routing-refusal helpers many packages share.
  Names:
  `readsAsRefusal`,
  `readingsCorroborate`,
  `clusterReadings`,
  `errorName`,
  `monotonicMs`,
  `isSuccessStatus`,
  `NoProviderForModelError`.
  Declared in `reading-refusal.ts`,
  `reading-corroboration.ts`,
  `reading-cluster.ts`,
  `error-name.ts`,
  `monotonic-clock.ts`,
  `http-success.ts`,
  and `no-provider-for-model-error.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

No other family package is a production dependency.

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `bench`
- `consistency`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `document-structure`
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
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Packages that list it only in `devDependencies`:

- `consolidate`
- `source-scan`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`.

Build before testing:
`mise run //package/translation-repair/markup:build` builds this package,
which has no family sibling to build first.

Run the tests with `mise run //package/translation-repair/markup:test:unit`,
or build and test together with `mise run //package/translation-repair/markup:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `model-order.test-fixture.ts` and `iso-stamp-text.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
