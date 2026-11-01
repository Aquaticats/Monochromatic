# @monochromatic-dev/translation-repair-page-pass

Page passes that restore or splice text around the model lanes:
the deterministic gate that applies an editor's patch operations,
passes that unify title references and heading series,
restore contributor names,
ordinals and quote characters,
and the splicing of slices back into a page.
It is its own package because the assembly,
repair,
translate and page-check layers all run the same passes,
and it depends on no lane package.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Patch application and preservation:
  apply a set of patch operations to a text,
  and check that the preservation rules still hold.
  Names:
  `applyPatchOperations`,
  `PatchOperation`,
  `PatchOutcome`,
  `EnvelopeOverlapError`,
  `checkPreservation`,
  `PreservationVerdict`.
  Declared in `apply-patch.ts` and `preservation-check.ts`.
- Title references and headings:
  unify how a page refers to titles,
  render a numbered heading series in one style,
  restore the archive's heading where renderings repeat another section's,
  and refuse a page on which two different source headings read the same.
  Names:
  `unifyTitleReferences`,
  `unifyHeadingSeries`,
  `restoreCollidingHeadings`,
  `assertHeadingsStayDistinct`,
  `CollapsedHeadingError`,
  `headingTitles`.
  Declared in `corpus-run/title-reference-unify.ts`,
  `corpus-run/heading-series-unify.ts`,
  `corpus-run/heading-collision-restore.ts`,
  `corpus-run/heading-distinctness.ts`,
  and `corpus-run/heading-title-lines.ts`.
- Contributor names and handles:
  restore contributor names to their authorized forms and place glosses for handles.
  Names:
  `restoreContributorNames`,
  `assertContributorNamesComplete`,
  `ContributorCompletenessError`,
  `nameAuthorities`,
  `placeHandleGlosses`.
  Declared in `corpus-run/contributor-name-restore.ts`,
  `corpus-run/contributor-completeness.ts`,
  `corpus-run/contributor-name-authorities.ts`,
  and `corpus-run/handle-gloss-place.ts`.
- Ordinals:
  read how a page writes ordinals and render them in one style.
  Names:
  `readOrdinalStyle`,
  `renderOrdinal`,
  `readHanNumeral`,
  `OrdinalStyle`.
  Declared in `corpus-run/ordinal-style.ts`.
- Typography restore:
  put back the quote characters and ellipses an editor flattened.
  Names:
  `restoreTypography`,
  `restoreEllipsis`.
  Declared in `restore-typography.ts` and `restore-ellipsis.ts`.
- Footnote relabel and slice splicing:
  apply a footnote relabel,
  and splice slice replacements into the text of a page.
  Names:
  `applyFootnoteRelabel`,
  `retainedFootnoteLabels`,
  `spliceSlices`,
  `placeSlices`,
  `SliceSpliceError`,
  `pageTextOf`.
  Declared in `apply-footnote-relabel.ts`,
  `footnote-retained-labels.ts`,
  `splice-slices.ts`,
  and `corpus-run/assembly-page-text.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `provider-stream`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `assembly`
- `bench`
- `consistency`
- `corpus-pass`
- `lane-contest`
- `page-check`
- `repair`
- `slice-cache`
- `translate-floor`
- `translate-lane`

Packages that list it only in `devDependencies`:

- `insertion`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/page-pass:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/page-pass:test:unit`,
or build and test together with `mise run //package/translation-repair/page-pass:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/title-reference.test-fixture.ts` and `whole-chunk-candidate.test-fixture.ts`.

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
