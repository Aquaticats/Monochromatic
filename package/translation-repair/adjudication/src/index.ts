//region Public index
// The names `@monochromatic-dev/translation-repair-adjudication` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { ADDITION_IS_REMOVED_NOT_SOFTENED, } from './addition-repair-rule.ts';
export {
  type AdjudicationPromptPlan,
  buildAdjudicationMessages,
} from './adjudicate-prompt.ts';
export {
  ADJUDICATION_RESPONSE_FORMAT,
  isPanelBallotWire,
  type PanelBallotWire,
  type PanelGroupWire,
  type PanelVerdictWire,
  resolvePanelBallot,
  usablePanelBallotFor,
} from './adjudicate-wire.ts';
export { logBallotIrregularities, } from './ballot-irregularity-log.ts';
export { countBallots, } from './candidate-select-count.ts';
export {
  NO_TYPED_ANSWER,
  selectDecision,
  TYPED_BALLOT_REASON,
} from './candidate-select-decision.ts';
export {
  buildCandidateSelectMessages,
  CANDIDATE_NONE,
  CANDIDATE_SELECT_RESPONSE_FORMAT,
  type CandidateBallotAsSent,
  type CandidateBallotWire,
  isCandidateBallotAsSent,
  isCandidateBallotWire,
  readCandidateBallotWire,
  type SelectEvidence,
} from './candidate-select-wire.ts';
export { countCandidateWeights, } from './candidate-weights.ts';
export {
  type IssueCheckerBallot,
  type IssueCheckerReading,
} from './checker-reading.ts';
export {
  buildSliceCriticRecords,
  type ClaimAttribution,
  type ClaimEmission,
  type ClaimProposer,
  collectClaimAttributions,
  retainAttributions,
  type SliceCriticRecord,
} from './critic-attribution.ts';
export {
  buildCriticMessages,
  criticOriginalsOf,
} from './critic-prompt.ts';
export {
  CRITIC_RESPONSE_FORMAT,
  type CriticIssueResolution,
  type CriticIssueWire,
  type CriticReportWire,
  isCriticReportWire,
  resolveCriticIssue,
} from './critic-wire.ts';
export {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from './declared-identity-rule.ts';
export {
  DECLARED_NAMES_EVIDENCE_LABEL,
  declaredNamesEvidence,
} from './declared-names-evidence.ts';
export { prepareDocumentPair, } from './document-preparation.ts';
export {
  EMPTY_INTRODUCED_DEFECT_REPORT,
  type IntroducedDefectReport,
  runIntroducedDefectProbe,
} from './introduced-defect-probe.ts';
export {
  type ClaimAdmissibility,
  type RegionDefectTally,
  type ResolvedProberChecks,
  resolveProberChecks,
  type ScreenedDefectClaim,
  screenEvidence,
  screenIntroducedDefects,
  UPHELD_ADMISSIBILITY,
} from './introduced-defect-screen.ts';
export {
  proberOriginalOf,
  proberShownTexts,
  type ProberShown,
  type ShownRegion,
} from './introduced-defect-shown.ts';
export {
  buildIntroducedDefectMessages,
  INTRODUCED_DEFECT_VERDICTS,
  type IntroducedDefectCheckWire,
  type IntroducedDefectPromptPlan,
  type IntroducedDefectReportWire,
  type IntroducedDefectVerdict,
  isIntroducedDefectReportWire,
  isIntroducedDefectVerdict,
  type PriorIssueDisclosure,
  type ProbedEditKind,
  PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
} from './introduced-defect-wire.ts';
export { INTRODUCED_DEFECT_RESPONSE_FORMAT, } from './introduced-defect-format.ts';
export { NaturalnessCompletenessError, } from './naturalness-completeness-error.ts';
export {
  NaturalnessQuorumError,
  validNaturalnessQuorum,
} from './naturalness-quorum.ts';
export { NaturalnessRepairInterruptedError, } from './naturalness-repair-interrupted-error.ts';
export {
  type PanelStageResult,
  runPanelStage,
} from './panel-stage.ts';
export {
  coverageGapLines,
  readStandingCoverage,
  type SeatAnswers,
  type StandingCoverage,
  UnseatedStandingError,
} from './producer-silence.ts';
export {
  rankStandings,
  standingLine,
} from './producer-standing-report.ts';
export {
  type PreferenceRate,
  preferenceRate,
  type ProducerStanding,
  producerStandings,
} from './producer-standing.ts';
export {
  type GradedRepairItem,
  parseGradedRepairSheet,
  readSheetIdentity,
  type RepairVerdict,
  type SheetIdentity,
} from './repair-grade-read.ts';
export {
  collectRepairRegions,
  type RepairRegion,
} from './repair-region.ts';
export { repairSelectionSourceEvidence, } from './repair-selection-evidence.ts';
export {
  buildResolutionMessages,
  isResolutionReportWire,
  isResolutionVerdict,
  RESOLUTION_RESPONSE_FORMAT,
  RESOLUTION_VERDICTS,
  type ResolutionCheckWire,
  type ResolutionPromptPlan,
  type ResolutionReportWire,
  type ResolutionVerdict,
  usableResolutionReportFor,
} from './resolution-wire.ts';
export { sealedNodeIds, } from './sealed-node-ids.ts';
export {
  type CandidateMeasurements,
  type CandidateSelection,
  CandidateSlateError,
  compareCandidates,
  type RepairCandidate,
  selectRepairCandidate,
  UNCHANGED_CANDIDATE_ID,
  UNCHANGED_MEASUREMENTS,
  winnerChangedText,
} from './select-candidate.ts';
export {
  CHUNK_DECLINE_CONSEQUENCE,
  declineConsequenceFor,
  KEEPS_TRUSTED_TEXT,
  LEAVES_PASSAGE_UNTRANSLATED,
  SHIPS_BY_PREFERENCE,
} from './select-decline-consequence.ts';
export { declaredSubjectNames, } from './subject-names.ts';
export {
  type IssueResolutionTally,
  type ResolutionBallot,
  resolveResolutionChecks,
  tallyResolutionChecks,
} from './tally-resolution.ts';
export {
  type AdjudicationResult,
  tallyVotes,
} from './tally-votes.ts';
export {
  STARTED_BY_VARIABLE,
  taskRunnerVerdict,
} from './task-runner-guard.ts';

//endregion Public index
