//region Public index
// The names `@monochromatic-dev/translation-repair-archive` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  confirmAbsoluteNaturalness,
  type ConfirmedAbsoluteNaturalness,
} from './absolute-naturalness-confirmation.ts';
export {
  type AbsoluteNaturalnessReviewOutcome,
  type AbsoluteNaturalnessReviewSeat,
  type AbsoluteNaturalnessReviewVerdict,
  reviewAbsoluteNaturalness,
} from './absolute-naturalness-review-stage.ts';
export {
  ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
  type AbsoluteNaturalnessFinding,
  type AbsoluteNaturalnessReviewPerspective,
  type AbsoluteNaturalnessReviewSubject,
  type AbsoluteNaturalnessReviewWire,
  buildAbsoluteNaturalnessReviewMessages,
  isAbsoluteNaturalnessReviewWire,
} from './absolute-naturalness-review-wire.ts';
export {
  isArchiveReferenceQuoteAnchored,
  isArchiveSourceQuoteAnchored,
  isVerifiableEditorialArchiveBlock,
} from './archive-block-evidence.ts';
export {
  type ArchiveBlockReviewOutcome,
  runArchiveBlockReviewStage,
} from './archive-block-review-stage.ts';
export {
  archiveReplyVerdict,
  type ArchiveReplyVerdict,
} from './archive-reply-verdict.ts';
export {
  ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
  ARCHIVE_BLOCK_IDENTITY_RULE,
  ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  ARCHIVE_BLOCK_SELECTION_TASK,
  buildArchiveBlockReviewMessages,
  isArchiveBlockReviewWire,
} from './archive-block-review-wire.ts';
export {
  archiveBlockSelectionEvidence,
  CompositeNotOnSlateError,
  withArchiveOriginal,
} from './archive-block-selection-evidence.ts';
export {
  counted,
  linesOf,
  numbered,
  sharedLines,
} from './archive-composite-lines.ts';
export {
  ALIGNMENT_CELL_LIMIT,
  describeCollisions,
  describeHunk,
  hunkCollisions,
  hunksBetween,
  type HunkCollisions,
  type LineHunk,
  placed,
  replacedLines,
} from './archive-composite-hunks.ts';
export {
  spliceHunks,
  type SplicedBlock,
} from './archive-composite-splice.ts';
export { voicesAsRead, } from './archive-voices-as-read.ts';
export {
  acceptedRepairClaimNotesOf,
  acceptedRepairClaimsNote,
  acceptedRepairClaimsOf,
  repairClaimEvidenceOf,
} from './accepted-repair-claims.ts';
export {
  type ArchiveDispute,
  archiveDisputeNote,
  archiveDisputeNotesOf,
  archiveDisputesOf,
  archiveDisputesOfRepair,
  describeArchiveDispute,
  type DisputableChunk,
  disputedWordingsOf,
  logArchiveDisputes,
} from './archive-dispute.ts';
export {
  closeFootnoteRelabel,
  documentLabels,
} from './archive-footnote-closure.ts';
export {
  definitionLabelOrder,
  type ReorderedDefinitions,
  reorderFootnoteDefinitions,
} from './archive-footnote-order.ts';
export {
  BlockOutsideArchiveError,
  revisionFootnoteFindings,
} from './archive-revision-footnotes.ts';
export {
  REVISION_SHAPE_REFUSED,
  revisionShapeFindings,
} from './archive-revision-shape.ts';
export {
  type CoverageCandidate,
  listCoverageCandidates,
} from './coverage-candidates.ts';
export { type TargetRegion, } from './coverage-foreign-region.ts';
export {
  type CoverageAnswer,
  runCoverageStage,
} from './coverage-stage.ts';
export {
  interiorShortfall,
  readUntranslatedTail,
  type UntranslatedTail,
} from './coverage-tail.ts';
export {
  type CoverageVerdict,
  judgeCoverage,
} from './coverage-verdict.ts';
export {
  buildCoverageMessages,
  COVERAGE_IDENTITY_RULE,
  COVERAGE_RESPONSE_FORMAT,
  type CoverageDegree,
  type CoverageFollowupEvidence,
  type CoveragePromptPlan,
  type CoverageReportWire,
  isCoverageReportWire,
} from './coverage-wire.ts';
export {
  assessNonTranslationDominance,
  assessNonTranslationEvidence,
  NON_TRANSLATION_BLOCK_VOTES,
  NON_TRANSLATION_CONTRADICTION_MIN,
  type NonTranslationDominance,
  type NonTranslationEvidence,
  type NonTranslationScreening,
  nonTranslationVotesStand,
  screenNonTranslationVotes,
  sliceAnchorsTranslation,
} from './non-translation-evidence.ts';
export { nonTranslationDominanceFinding, } from './non-translation-finding.ts';
export { type PreparedBlockPairing, } from './prepare-block-pairing-model.ts';
export { prepareBlockPairing, } from './prepare-block-pairing.ts';
export { type BenchmarkEntry, } from './prepare-entry.ts';
export {
  buySectionPairing,
  type SectionRoundOutcome,
} from './prepare-section-round.ts';
export {
  type PairedPreparation,
  prepareDocumentPairWithRoster,
} from './prepare-with-pairing.ts';

//endregion Public index
