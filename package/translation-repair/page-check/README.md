# @monochromatic-dev/translation-repair-page-check

Published-page checks and the page-assembly passes:
Canadian spelling and dates,
the text a reader would see at each slice ("would ship"),
page assembly guards,
destination checks,
archive casing and the publication of the settled tree.
It is its own package because `corpus-pass`,
`probe`,
`run` and `slice-cache` all assemble and check pages the same way,
and it sits above the lane packages.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Canadian forms and small corrections:
  write dates and listed spellings the Canadian way,
  bring prose quote marks to the page's majority style,
  unwrap quoted blockquote paragraphs where the archive sets its blockquotes bare,
  and correct pinyin tones,
  each on every slice of the page,
  those no lane replaced included.
  Names:
  `canadianizePage`,
  `canadianizeText`,
  `unifyQuoteStyle`,
  `unwrapBlockquoteQuotes`,
  `correctPinyinPage`.
  Declared in `corpus-run/canadian-forms.ts`,
  `corpus-run/quote-style-unify.ts`,
  `corpus-run/blockquote-quote-unify.ts`,
  and `corpus-run/pinyin-tone.ts`.
- Would-ship text:
  derive what a reader would see at one slice from the decisions of the stages above the lanes.
  Names:
  `wouldShipTextFor`,
  `wouldShipTextPerSlice`,
  `UnansweredContestSliceError`,
  `WouldShipReading`.
  Declared in `corpus-run/would-ship-text.ts`.
- Source text under a note:
  read a slice with no archive text and no backed wording as shipping the original passage,
  one block-quote note before each contiguous run (owner,
  2026-10-23,
  "Source text + note").
  The text is each slice's original as the artifact records it,
  the slices of a run joined by one blank line,
  and text between slices is not carried.
  Names:
  `UNTRANSLATED_PASSAGE_NOTE`,
  `carryingSource`,
  `markRuns`,
  `carriesText`,
  `textOnPage`,
  `withoutSourceRecordAt`.
  Declared in `corpus-run/source-ships.ts`.
- Page composition under the grammar:
  ship each source run as recorded whenever the page parses,
  and otherwise write the runs that do not parse alone (and any run holding a comment that never closes) as literal
  text,
  then every run,
  then ship the page without the runs no writing makes parse (without every run where that does not help)
  and report `page-source-unparseable` naming the slices.
  The literal text is read off a CommonMark parse of the run with the site's math
  (`parseMarkdownMathBody` of `translation-repair-document-structure`),
  so a fence in a list item,
  a code span across lines,
  math,
  a link destination and an escape the original wrote are copied,
  and only a `{`,
  `}` or `<` the parse reads as text is escaped.
  A footnote definition that CommonMark takes for a link reference definition is read as the footnote it is,
  its body as blocks of its own,
  and a bare address that a less-than sign ends is written as a link with the same text and target.
  A page on disk is judged by the same composition run over what the artifact records,
  so a page shipped without runs reads as agreed
  (`PageAgreement.sourceDropped` names the slices).
  Names:
  `composeSourcePage`,
  `composeOverTiling`,
  `tilingOf`,
  `sourceRunsOf`,
  `slicesRefusedAsLiteral`,
  `droppedRunSets`,
  `spansOf`,
  `bareAddresses`,
  `addressAsLink`,
  `explicitLink`,
  `encodeAsLiteralMdx`,
  `SourcePageCompositionError`,
  `readPageGrammar`.
  Declared in `corpus-run/source-ships-compose.ts`,
  `corpus-run/source-ships-runs.ts`,
  `corpus-run/source-ships-tiling.ts`,
  `corpus-run/source-ships-spans.ts`,
  `corpus-run/source-ships-addresses.ts`,
  `corpus-run/source-ships-links.ts`,
  `corpus-run/source-ships-encoding.ts`
  and `corpus-run/page-grammar.ts`.
  `corpus-run/source-ships-footnote.ts`,
  `corpus-run/source-ships-module-line.ts`
  and `corpus-run/source-ships-ranges.ts` hold the helpers of the reading and are exported for their own tests.
- Shippable replacements:
  turn a settled entry's per-slice readings into the replacements that assemble its page.
  The original of a slice that ships it is left out unless the caller passes `includeSource: true`,
  which only the writer of the published page does.
  Names:
  `shippableReplacements`.
  Declared in `corpus-run/shippable-replacements.ts`.
- Page assembly guards:
  run the composed page through the guards each lane runs,
  and refuse a page that does not parse or breaks its footnotes.
  Names:
  `guardPageAssembly`,
  `assertPageParses`,
  `UnparseablePageError`,
  `assertPageFootnotesIntact`,
  `assertPageFootnotesResolve`,
  `PageFootnoteResolutionError`,
  `assertFrontMatterComplete`.
  The integrity guard weighs the page against its archive;
  the resolve check reads the page alone and is reported as a publish defect,
  never a refusal.
  Declared in `corpus-run/page-assembly-guard.ts`,
  `corpus-run/page-grammar.ts`,
  `corpus-run/page-footnote-integrity.ts`,
  `corpus-run/page-footnote-resolution.ts`,
  and `corpus-run/front-matter-completeness.ts`.
- Destinations:
  find the link destinations of the source that a page no longer carries,
  judge which of them the page may lack under the either-rendering rule,
  refuse a would-ship page that drops one,
  and point links the archive gave its own destination back at it.
  Names:
  `assertDestinationsComplete`,
  `DroppedDestinationError`,
  `droppedDestinations`,
  `judgeDestinationRenderings`,
  `restoreArchiveDestinations`.
  Declared in `corpus-run/destination-completeness.ts`,
  `corpus-run/dropped-destinations.ts`,
  `corpus-run/destination-renderings.ts`,
  and `corpus-run/archive-destination-restore.ts`.
- Archive casing and restores:
  restore casing,
  italic titles,
  JSX attributes,
  list spread and name glosses the archive had.
  Names:
  `restoreArchiveCasing`,
  `restoreArchiveNameCasing`,
  `restoreArchiveItalicTitles`,
  `restoreJsxAttributes`,
  `restoreListSpread`,
  `restoreNameGlossLines`.
  Declared in `corpus-run/archive-casing-restore.ts`,
  `corpus-run/archive-name-casing.ts`,
  `corpus-run/archive-italic-title-restore.ts`,
  `corpus-run/jsx-attribute-restore.ts`,
  `corpus-run/list-spread-restore.ts`,
  and `corpus-run/name-gloss-restore.ts`.
- Publishing and verification:
  write the settled pages,
  list defects,
  and check a published page against its artifact.
  Names:
  `publishFixedPage`,
  `publishDefects`,
  `republishSettledPages`,
  `pageCarriesEveryWording`,
  `refusePageThatDisagrees`,
  `PublishedPageDisagreesError`,
  `whatThereIsToVerify`.
  Declared in `corpus-run/publish-fixed.ts`,
  `corpus-run/publish-defects.ts`,
  `corpus-run/page-republish.ts`,
  `corpus-run/published-page-check.ts`,
  `corpus-run/published-page-disagreement.ts`,
  and `corpus-run/published-tree-listing.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `corpus-source`
- `document-structure`
- `glossary`
- `lane-contest`
- `markup`
- `page-pass`
- `provider`
- `reference`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Development dependencies other than `test-fixture` (`devDependencies`,
used by tests):

- `consolidate`

### Used by

Packages that list it in `dependencies`:

- `corpus-pass`
- `probe`
- `run`
- `slice-cache`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/page-check:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/page-check:test:unit`,
or build and test together with `mise run //package/translation-repair/page-check:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/published-run.test-fixture.ts`.

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
