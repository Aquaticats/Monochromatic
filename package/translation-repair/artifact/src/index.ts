//region Public index
// The names `@monochromatic-dev/translation-repair-artifact` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  type ArtifactChangeSets,
  readArtifactChangeSets,
} from './artifact-change-sets.ts';
export {
  requireArtifactJsonRecord,
  requireArtifactJsonValue,
  requireExactKeys,
  requireKeyOf,
  requireOneOf,
  requireOpenRecord,
} from './artifact-exact-guard.ts';
export {
  type ArtifactKeyVocabulary,
  CHUNK_SPELLED_KEYS,
  keyVocabularyOf,
  SLICE_SPELLED_KEYS,
  UnknownArtifactGenerationError,
} from './artifact-key-vocabulary.ts';
export {
  type ArtifactProbeReading,
  type OwnedProbeReading,
  readArtifactProbe,
} from './artifact-probe-read.ts';
export { parseRegionTally, } from './artifact-probe-tally.ts';
export {
  type ParsedArtifactReading,
  readSettledArtifact,
} from './artifact-read.ts';
export {
  type RepairLaneRecords,
  repairLaneRecordsOf,
} from './artifact-repair-lane-records.ts';
export {
  ARTIFACT_SCHEMA_VERSION_V1,
  type ArtifactSchemaReading,
  KNOWN_ARTIFACT_SCHEMA_VERSIONS,
  readArtifactSchemaVersion,
} from './artifact-schema-version.ts';
export {
  type ParsedAcceptedIssue,
  type ParsedArtifact,
  parseSettledArtifact,
} from './artifact-v1-read.ts';
export { archiveFlawedByMajority, } from './consolidate-archive-flawed.ts';
export { archiveStandInFor, } from './consolidate-archive-stand-in.ts';
export {
  CONSOLIDATE_CACHE_VERSION,
  consolidateRunShape,
  consolidateSliceKey,
} from './consolidate-key.ts';
export { laneTextsForSlate, } from './consolidate-lane-offer.ts';
export { withShippedTenseCandidates, } from './consolidate-shipped-tense.ts';
export {
  type ConsolidateRoster,
  type ConsolidateSliceSeating,
} from './consolidate-slice-seating.ts';
export {
  OffRosterModelError,
  requireProducer,
  requireRosterModelId,
} from './corpus-run/artifact-producer-read.ts';
export {
  readRepairRounds,
  RoundsNotRecordedError,
} from './corpus-run/artifact-rounds-read.ts';
export { buildSettledTwoLaneArtifact, } from './corpus-run/artifact-two-lane-build.ts';
export {
  type ArtifactConsistency,
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyCandidate,
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyCarryDecision,
  type ArtifactConsistencyChoiceBallot,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyOccurrence,
  type ArtifactConsistencyPlaceName,
  type ArtifactConsistencyRendering,
  type ArtifactConsistencyRepeat,
  type ArtifactConsistencyRunOff,
  type ArtifactConsistencySlice,
  type ArtifactConsistencyUnplaced,
  type ArtifactConsistencyVerdict,
  CONSISTENCY_CANDIDATE_KINDS,
  CONSISTENCY_CARRY_DECISION_KINDS,
  CONSISTENCY_CARRY_NO_DECISION_REASONS,
  CONSISTENCY_KEPT_REASONS,
  type ConsistencyCarryNoDecisionReason,
  type ConsistencyKeptReason,
  type ConsistencyNotAskedReason,
  type ConsistencyOccurrenceOutcome,
  type ConsistencyOverride,
  consistencyOverrideAt,
  NO_CONSISTENCY,
} from './corpus-run/artifact-two-lane-consistency.ts';
export {
  type ArtifactConsolidateGate,
  type ArtifactConsolidateShipped,
  type ArtifactConsolidateSlice,
  type ArtifactConsolidation,
  type ArtifactConsolidationPolish,
  type ArtifactConsolidationTerminal,
  type ArtifactNaturalnessCorrection,
  type ArtifactNaturalnessFinding,
  type ArtifactNaturalnessReview,
  type ArtifactNaturalnessReviewRound,
  type ArtifactNaturalnessReviewSeat,
  describeConsolidateSlice,
} from './corpus-run/artifact-two-lane-consolidate.ts';
export {
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  ARTIFACT_SCHEMA_VERSION_V20,
  ARTIFACT_SCHEMA_VERSION_V21,
  ARTIFACT_SCHEMA_VERSION_V22,
  ARTIFACT_SCHEMA_VERSION_V2,
  ARTIFACT_SCHEMA_VERSION_V5,
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  type ArtifactJsonValue,
  type ArtifactSectionAlignment,
  type ArtifactSectionCorrespondence,
  isTwoLaneArtifactGeneration,
  type SettledArtifact,
  type SettledLane,
  type SettledPreparation,
  TWO_LANE_GENERATIONS,
  type TwoLaneArtifactGeneration,
} from './corpus-run/artifact-two-lane-contract.ts';
export { verifyArtifactMeasurements, } from './corpus-run/artifact-two-lane-corpus-verify.ts';
export {
  type ProjectedLanes,
  projectLanes,
} from './corpus-run/artifact-two-lane-derive.ts';
export {
  type ArtifactPageAssembly,
  NO_PAGE_ASSEMBLY,
  type PageAssemblyOverride,
  pageAssemblyOverrideAt,
  parsePageAssembly,
} from './corpus-run/artifact-two-lane-page-assembly.ts';
export {
  toArtifactComparisonRow,
  toArtifactDecisions,
  toArtifactDelivery,
  toArtifactOutcome,
  toArtifactRow,
} from './corpus-run/artifact-two-lane-project.ts';
export {
  assertCarriesRecount,
  outcomesAfter,
} from './corpus-run/artifact-two-lane-read-consistency-carry-recount.ts';
export {
  parseConsistencyCarry,
  parseConsistencyRunOff,
} from './corpus-run/artifact-two-lane-read-consistency-carry.ts';
export { assertAskedGroupCoherent, } from './corpus-run/artifact-two-lane-read-consistency-coherence.ts';
export { assertGroupFits, } from './corpus-run/artifact-two-lane-read-consistency-fit.ts';
export {
  asLocated,
  locatedContradicted,
  reasonContradicted,
} from './corpus-run/artifact-two-lane-read-consistency-located.ts';
export {
  keptReasonExpected,
  parseConsistencyOccurrence,
} from './corpus-run/artifact-two-lane-read-consistency-occurrence.ts';
export {
  parseConsistencyBallot,
  parseConsistencyRepeat,
  parseConsistencyUnplaced,
} from './corpus-run/artifact-two-lane-read-consistency-parts.ts';
export { parseConsistencyRendering, } from './corpus-run/artifact-two-lane-read-consistency-rendering.ts';
export { assertSlicesCoherent, } from './corpus-run/artifact-two-lane-read-consistency-slices.ts';
export {
  asLocatedBySpan,
  locatedContradictedBySpan,
  reasonContradictedBySpan,
} from './corpus-run/artifact-two-lane-read-consistency-span-located.ts';
export {
  assertCarriedOutcomes,
  type CarriedPlace,
  carriesSplice,
  spliceBuiltForSomePlace,
} from './corpus-run/artifact-two-lane-read-consistency-span-shared.ts';
export { assertSpanOutcomesFollow, } from './corpus-run/artifact-two-lane-read-consistency-span-outcomes.ts';
export { expectedSlotOf, } from './corpus-run/artifact-two-lane-read-consistency-span-slot.ts';
export {
  expectedWinnerOf,
  sameWinner,
} from './corpus-run/artifact-two-lane-read-consistency-span-winner.ts';
export { assertSpanGroupCoherent, } from './corpus-run/artifact-two-lane-read-consistency-span.ts';
export {
  assertSeatsHeardOnce,
  winningPositionOf,
} from './corpus-run/artifact-two-lane-read-consistency-tally.ts';
export { parseConsistencyVerdict, } from './corpus-run/artifact-two-lane-read-consistency-verdict.ts';
export { parseConsistency, } from './corpus-run/artifact-two-lane-read-consistency.ts';
export {
  parseGateBallot,
  parseShipped,
  parseVerdict,
} from './corpus-run/artifact-two-lane-read-consolidate-parts.ts';
export {
  parseConsolidation,
  type ParsedConsolidation,
} from './corpus-run/artifact-two-lane-read-consolidate.ts';
export { parseLaneSelection, } from './corpus-run/artifact-two-lane-read-contest.ts';
export { parseContestFloorMark, } from './corpus-run/artifact-two-lane-read-contest-floor.ts';
export {
  type ArtifactEvidenceRow,
  type ArtifactRepairEvidence,
  type ArtifactRepairStatus,
  type ArtifactTranslateEvidence,
  type ArtifactTranslateStatus,
  type ParsedArchiveText,
  type ParsedLane,
  type ParsedPreparation,
  type ParsedTwoLaneArtifact,
} from './corpus-run/artifact-two-lane-read-contract.ts';
export {
  parseRepairEvidence,
  parseTranslateEvidence,
} from './corpus-run/artifact-two-lane-read-evidence.ts';
export { parseLanes, } from './corpus-run/artifact-two-lane-read-lanes.ts';
export { parseNaturalnessReview, } from './corpus-run/artifact-two-lane-read-naturalness-review.ts';
export {
  parseBlockPairing,
  type ParsedBlockPairing,
} from './corpus-run/artifact-two-lane-read-pairing.ts';
export { parseConsolidationPolish, } from './corpus-run/artifact-two-lane-read-polish.ts';
export {
  assertEvidenceMatchesLedger,
  assertRowsCoherent,
} from './corpus-run/artifact-two-lane-read-row-relations.ts';
export {
  parseComparisonRow,
  parseDeliveryRow,
  parseEvidenceRow,
} from './corpus-run/artifact-two-lane-read-rows.ts';
export {
  type ParsedSectionPairing,
  parseSectionPairing,
} from './corpus-run/artifact-two-lane-read-section-pairing.ts';
export {
  assertBlockedCompatible,
  assertIndexSetsMatchLedger,
  assertTranslateCountsAgree,
} from './corpus-run/artifact-two-lane-read-set-relations.ts';
export { parseRecordedTenseCandidates, } from './corpus-run/artifact-two-lane-read-tense.ts';
export {
  parseDecisionComparison,
  parseSliceDelivery,
  parseSliceOutcome,
  type UnknownKeyPolicy,
} from './corpus-run/artifact-two-lane-read-vocabulary.ts';
export { parseSettledTwoLaneArtifact, } from './corpus-run/artifact-two-lane-read.ts';
export {
  type PairingRecipe,
  pairingSetAsideSentence,
  readSettledEntryInfo,
  rebuildPreparation,
  type RebuildReproduction,
  type RebuiltPreparation,
  type RecipeHalf,
  recipeOf,
} from './corpus-run/artifact-two-lane-rebuild.ts';
export {
  ArtifactPreparationMismatchError,
  assertFindingsDescribePreparation,
  assertLedgerDescribesPreparation,
  assertResultCountsPreparation,
} from './corpus-run/artifact-two-lane-verify.ts';
export {
  type ClassifiedSlice,
  classifyDisplacement,
  type DocumentDisplacement,
  type RelocationCandidate,
  type SliceClass,
} from './displacement-class.ts';
export {
  mostGiven,
  unwrappedRendering,
} from './page-title-lexicon-count.ts';
export {
  type PageTitleLexicon,
  pageTitleLines,
  type SettledPageTitle,
  settlePageTitles,
} from './page-title-lexicon-stage.ts';
export {
  buildPageTitleLexiconMessages,
  isPageTitleLexiconWire,
  PAGE_TITLE_IDENTITY_RULE,
  PAGE_TITLE_LEXICON_RESPONSE_FORMAT,
} from './page-title-lexicon-wire.ts';

//endregion Public index
