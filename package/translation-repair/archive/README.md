# @monochromatic-dev/translation-repair-archive

Handling of the translation a page already carries in the archive:
the archive block review,
disputes over archive wording,
revision checks,
coverage verdicts,
non-translation screening,
the naturalness review,
and the preparation of sections and blocks.
It is its own package because the passes,
the artifact builder,
the consolidation lane and the page checks all read these records,
and it sits above `lane-contest` and `translate-stage`.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Archive block review:
  review one archive block that no source block claims,
  where reviewers keep it on source or editorial support or revise it,
  and judges choose between the revisions,
  one composite of revisions that changed disjoint stretches of lines,
  a revision being read as the hunks a line diff makes to the block,
  so a revision may add or remove lines,
  and the unchanged block;
  the evidence checks and the selection evidence for that choice.
  The round waits the writers' straggler window,
  since each revision is a whole candidate.
  The composite reaches the judges with its contributors' own assessments,
  each marked with the composite's lines that contributor put there,
  since no reviewer proposed the composite.
  Names:
  `runArchiveBlockReviewStage`,
  `buildArchiveBlockReviewMessages`,
  `isVerifiableEditorialArchiveBlock`,
  `archiveReplyVerdict`,
  `archiveBlockSelectionEvidence`,
  `withArchiveOriginal`,
  `CompositeNotOnSlateError`,
  `hunksBetween`,
  `hunkCollisions`,
  `spliceHunks`,
  `voicesAsRead`.
  Declared in `archive-block-review-stage.ts`,
  `archive-block-review-wire.ts`,
  `archive-block-evidence.ts`,
  `archive-reply-verdict.ts`,
  `archive-block-selection-evidence.ts`,
  `archive-composite-revision.ts`,
  `archive-composite-lines.ts`,
  `archive-composite-hunks.ts`,
  `archive-composite-splice.ts`,
  and `archive-voices-as-read.ts`.
- Archive disputes:
  read off the repair lane's chunks the slices whose archive rendering the adjudicators accepted a claim against,
  say which text stands in for the archive there and which wordings the slice refuses,
  and write the sheet note and the log line for each.
  Names:
  `archiveDisputesOf`,
  `archiveDisputeNotesOf`,
  `describeArchiveDispute`,
  `logArchiveDisputes`,
  `disputedWordingsOf`.
  Declared in `archive-dispute.ts`.
- Accepted repair claims:
  read off the same chunks every claim the adjudicators accepted,
  at the severity they settled it,
  leave out the claims a dispute note already carries,
  and write the sheet note the polish gate reads.
  Names:
  `acceptedRepairClaimsOf`,
  `acceptedRepairClaimsNote`,
  `acceptedRepairClaimNotesOf`,
  `repairClaimEvidenceOf`.
  Declared in `accepted-repair-claims.ts`.
- Revisions and footnotes:
  check a revision keeps the footnotes and shape of its block,
  and relabel and reorder footnote definitions.
  Names:
  `revisionFootnoteFindings`,
  `revisionShapeFindings`,
  `BlockOutsideArchiveError`,
  `closeFootnoteRelabel`,
  `reorderFootnoteDefinitions`.
  Declared in `archive-revision-footnotes.ts`,
  `archive-revision-shape.ts`,
  `archive-footnote-closure.ts`,
  and `archive-footnote-order.ts`.
- Coverage verdicts:
  list the passages no pairing covers,
  ask a roster whether the translation carries each one anywhere,
  weigh the replies into a verdict,
  and measure the untranslated tail,
  the source-only slices after the pairing's last agreed pair,
  and the English the translated part is missing against what its source predicts.
  Names:
  `runCoverageStage`,
  `judgeCoverage`,
  `buildCoverageMessages`,
  `listCoverageCandidates`,
  `readUntranslatedTail`,
  `interiorShortfall`.
  Declared in `coverage-stage.ts`,
  `coverage-verdict.ts`,
  `coverage-wire.ts`,
  `coverage-candidates.ts`,
  and `coverage-tail.ts`.
- Non-translation screening:
  test the critics' votes that a slice is not a translation of its source against deterministic evidence,
  rule whether the standing votes dominate the document,
  and word the finding that names the block.
  Names:
  `assessNonTranslationDominance`,
  `screenNonTranslationVotes`,
  `nonTranslationVotesStand`,
  `nonTranslationDominanceFinding`.
  Declared in `non-translation-evidence.ts` and `non-translation-finding.ts`.
- Naturalness review:
  review a candidate's naturalness before it ships,
  and confirm an acceptance with a second reading.
  Names:
  `reviewAbsoluteNaturalness`,
  `confirmAbsoluteNaturalness`,
  `buildAbsoluteNaturalnessReviewMessages`.
  Declared in `absolute-naturalness-review-stage.ts`,
  `absolute-naturalness-confirmation.ts`,
  and `absolute-naturalness-review-wire.ts`.
- Section and block preparation:
  pair the blocks and sections of an archive with a source before the lanes run.
  Names:
  `prepareBlockPairing`,
  `buySectionPairing`,
  `prepareDocumentPairWithRoster`.
  Declared in `prepare-block-pairing.ts`,
  `prepare-section-round.ts`,
  and `prepare-with-pairing.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `adjudication`
- `assembly`
- `corpus-source`
- `document-structure`
- `glossary`
- `house-rule`
- `lane-contest`
- `markup`
- `page-pass`
- `provider`
- `reference`
- `shared-contract`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `artifact`
- `consolidate`
- `corpus-pass`
- `coverage-census`
- `insertion`
- `page-check`
- `probe`
- `repair`
- `translate-lane`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/archive:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/archive:test:unit`,
or build and test together with `mise run //package/translation-repair/archive:buildAndTest`.
Pass test file paths to run only those files.

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
