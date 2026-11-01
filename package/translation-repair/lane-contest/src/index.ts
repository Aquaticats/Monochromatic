//region Public index
// The names `@monochromatic-dev/translation-repair-lane-contest` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  CONTEST_POLICY,
  CONTEST_REFUSAL,
  contestResponseFormat,
  findingsOrNone,
  isStringList,
  namesOneOf,
  readCandidateNames,
} from './contest-ballot-wire.ts';
export {
  type ContestRendering,
  contestSizeNote,
  SIZE_NOTE_POLICY,
} from './contest-size-note.ts';
export {
  type ArtifactContestSlice,
  type ArtifactContestVerdict,
  type ArtifactLaneSelection,
  contestEligibleIndexes,
  describeContestSlice,
  describeSoleLaneSlice,
  soleLaneOf,
  type SoleLaneReading,
} from './corpus-run/artifact-two-lane-contest.ts';
export {
  assertDeliveryCoherent,
  coherenceSentence,
  DeliveryCoherenceError,
  type DeliveryCoherenceFault,
} from './delivery-coherence.ts';
export {
  assertDeliveryAgreesWithDocument,
  DeliveryInvariantError,
  type DeliveryInvariantFault,
  deliveryInvariantSentence,
} from './delivery-invariants.ts';
export {
  type ComparedLane,
  comparisonSentence,
  type LaneComparisonFault,
} from './lane-comparison-fault.ts';
export {
  compareDocumentLanes,
  type DecisionComparison,
  type IdentifiedDeliveryLedger,
  type LaneComparison,
  LaneComparisonError,
  type SliceLaneComparison,
  type SliceLaneVerdict,
} from './lane-comparison.ts';
export {
  applyLaneContestEligibility,
  describeInadmissibleLanes,
  frontMatterContestEligibility,
  ineligibleCandidatesOf,
  LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,
  type LaneContestChoiceVerdict,
  laneContestChoiceVerdict,
  type LaneContestEligibility,
  settleEligibleLaneContestBallots,
} from './lane-contest-eligibility.ts';
export {
  LANE_CONTEST_CACHE_VERSION,
  laneContestRunShape,
  laneContestSliceKey,
} from './lane-contest-key.ts';
export {
  type BoughtLaneContest,
  persistLaneContestOutcome,
  storedContestOf,
} from './lane-contest-persist.ts';
export {
  type ArchiveOutcome,
  contestLaneSlice,
  LANE_CONTEST_QUORUM,
  LANE_CONTEST_RESPONSE_FORMAT,
  type LaneContestOutcome,
  settleArchiveBallots,
  settleLaneContestBallots,
} from './lane-contest-stage.ts';
export {
  laneContestTenseGroups,
  withTenseCandidates,
} from './lane-contest-tense.ts';
export {
  ARCHIVE_VERDICTS,
  type ArchiveVerdict,
  buildLaneContestMessages,
  CANDIDATE_NAMES,
  isLaneContestWire,
  type LaneChoice,
  type LaneContestBallot,
  type LaneContestSubject,
  type LaneContestWire,
  readLaneContestBallot,
} from './lane-contest-wire.ts';
export {
  type ProbeAgreement,
  type ProbeAgreementItem,
  probeFlaggedIssue,
  scoreProbeAgainstGrades,
} from './probe-agreement.ts';
export {
  type ProbeClaimAttribution,
  type TelemetryProbeReading,
  type TelemetryRegionTally,
} from './probe-attribution.ts';
export {
  corroboratingProberCount,
  judgeRegionProbe,
  ProbeTelemetryError,
  type ProbeTelemetrySummary,
  type RegionProbeVerdict,
  summarizeProbeTelemetry,
} from './probe-telemetry.ts';
export {
  computeDrawDigest,
  DRAW_IDENTITY_DOMAIN,
  type DrawIdentityItem,
} from './sample-draw-identity.ts';
export {
  buildSampleManifest,
  parseSampleManifest,
  type SampleGeneration,
  type SampleManifest,
  type SampleManifestItem,
} from './sample-manifest.ts';
export {
  gradeSeedDetection,
  type SeedDetectionVerdict,
} from './seed-detection.ts';
export {
  applySeededErrors,
  SEED_MATCH_TOLERANCE,
  SeedApplicationError,
  type SeededDocumentResult,
  type SeededErrorApplication,
  type SeededErrorKind,
  type SeededErrorSpec,
  seedHitByRegion,
} from './seeded-error.ts';
export {
  assertSheetMatchesManifest,
  HEADER_ONLY_BINDING_NOTE,
  requireSheetSeed,
  SheetBindingError,
  type SheetBindingStrength,
} from './sheet-binding.ts';

//endregion Public index
