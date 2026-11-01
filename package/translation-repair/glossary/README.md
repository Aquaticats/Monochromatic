# @monochromatic-dev/translation-repair-glossary

The community glossary and the rules that read a page against it,
with the address and skeleton rules of the translate floors,
sheet paths,
directory ids,
and several small run records that no higher layer owns.
It is its own package because the translate,
repair and report layers share these rules and records,
and it needs only `document-structure`,
`markup`,
`shared-contract` and `typography` below it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Community glossary and matching:
  the glossary data,
  the block of preferred renderings a prompt shows,
  and the matching that finds a form in a text.
  Names:
  `COMMUNITY_GLOSSARY`,
  `communityTermsIn`,
  `communityRenderingsBlock`,
  `communityRenderingDepartures`,
  `foldForGlossary`,
  `textCarriesForm`,
  `renderingSpans`.
  Declared in `community-glossary.ts` and `glossary-match.ts`.
- Address and skeleton floors:
  read the block skeleton of a slice or a page,
  and report candidates that do not carry their floor's blocks or quoted passages,
  that render a second-person address of the original in the third person,
  that set a Latin-letter title in the Chinese title marks,
  or that carry a closing quotation mark with no opening one.
  Names:
  `readSliceSkeleton`,
  `readPageSkeleton`,
  `compareBlocks`,
  `droppedAddressFindings`,
  `latinTitleFindings`,
  `strayClosingQuoteFindings`,
  `quotedPassageFloorFindings`.
  Declared in `translate-skeleton.ts`,
  `translate-skeleton-page.ts`,
  `translate-validate-blocks.ts`,
  `translate-address-drop.ts`,
  `translate-latin-title.ts`,
  and `translate-quote-balance.ts`.
- Sheet paths and directory ids:
  where graded sheets are written and how a directory id relates to a name.
  Names:
  `resolveSheetPath`,
  `writeSheetPair`,
  `assertSheetPairFree`,
  `GradedSheetExistsError`,
  `namesDirectoryId`,
  `directoryIdNameStands`.
  Declared in `corpus-run/sheet-path.ts`,
  `corpus-run/sheet-write.ts`,
  and `corpus-run/directory-id-name.ts`.
- Pipeline digest and attempt records:
  the digest of the built pipeline that artifacts and caches key their generation on,
  and the per-entry attempt store.
  Names:
  `digestPipeline`,
  `assertPipelineDigest`,
  `PipelineDigestError`,
  `groupByDigest`,
  `readAttemptMap`,
  `writeAttemptMap`,
  `countAttempt`.
  Declared in `corpus-run/pipeline-digest.ts`,
  `corpus-run/digest-group.ts`,
  and `corpus-run/attempt-store.ts`.
- Declined entries and file helpers:
  the record of entries the pass declined,
  atomic file writes,
  git lookup and directory listings.
  Names:
  `declinedEntryIds`,
  `writeDeclinedEntry`,
  `DeclinedEntriesUnreadableError`,
  `writeFileAtomic`,
  `resolveGit`,
  `namesIn`,
  `presentNamesOfKind`.
  Declared in `corpus-run/declined-entries.ts`,
  `corpus-run/atomic-write.ts`,
  `corpus-run/git-command.ts`,
  and `corpus-run/directory-listing.ts`.
- Grace dials:
  the environment settings that move the straggler window of stage rounds,
  and the longer window of writer rounds,
  without a rebuild.
  Names:
  `resolveStragglerGraceMs`,
  `STRAGGLER_GRACE_VAR`,
  `readWindowDial`,
  `resolveWriterGraceMs`,
  `WRITER_GRACE_VAR`,
  `readWriterGrace`.
  Declared in `grace-override.ts` and `writer-grace-override.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `markup`
- `shared-contract`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `bench`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
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

Packages that list it only in `devDependencies`:

- `translate-stage`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/glossary:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/glossary:test:unit`,
or build and test together with `mise run //package/translation-repair/glossary:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `stated-refusal-message.test-fixture.ts`,
`corpus-run/cap-argument-refusal.test-fixture.ts`,
and `corpus-run/built-pipeline-digest.test-fixture.ts`.

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
