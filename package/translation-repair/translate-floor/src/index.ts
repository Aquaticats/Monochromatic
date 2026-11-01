//region Public index
// The names `@monochromatic-dev/translation-repair-translate-floor` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  type BenchSeating,
  keepBench,
} from './bench-seating.ts';
export {
  type DisputedWording,
  disputedWordingFindings,
} from './disputed-wording.ts';
export {
  FRONT_MATTER_DECISION_RULE,
  validateFrontMatterTranslation,
} from './front-matter-translation.ts';
export {
  type AlignedRun,
  type GroupedRun,
  groupNodesSealed,
  type SealedRun,
} from './group-aligned.ts';
export { groupNodes, } from './group-nodes.ts';
export {
  laneCoverageSentence,
  LaneSliceCoverageError,
  type LaneSliceCoverageFault,
  type NamedSliceSetLabel,
} from './lane-slice-coverage-error.ts';
export {
  type NamedSliceSet,
  validateNamedSets,
} from './lane-slice-sets.ts';
export {
  buildLaneSliceTexts,
  type LaneSliceOutcome,
  type LaneSliceText,
  type UndecidedSlicePolicy,
} from './lane-slice-text.ts';
export { compareLineCounts, } from './line-structure-guard.ts';
export {
  MAX_PRODUCED_TO_SOURCE_RATIO,
  PRODUCED_VOLUME_FLOOR,
  producedVolumeBound,
} from './produced-volume-bound.ts';
export { describeSelfPreference, } from './self-preference-line.ts';
export {
  type SelectionRound,
  type SelfPreference,
  selfPreference,
  type SelfPreferenceCounts,
} from './self-preference.ts';
export { type SliceCache, } from './slice-cache.ts';
export {
  armSliceCost,
  SLICE_COST_LANES,
  SLICE_COST_MARKER,
  SLICE_START_MARKER,
  type SliceCostLane,
  type SliceCostSpan,
} from './slice-cost-log.ts';
export {
  readSliceCosts,
  type SliceCostReading,
  type SliceCostRow,
} from './slice-cost-read.ts';
export { type SliceDelivery, } from './slice-delivery-decide.ts';
export {
  deliverySentence,
  type DeliverySetName,
  SliceDeliveryError,
  type SliceDeliveryFault,
} from './slice-delivery-fault.ts';
export {
  buildSliceDelivery,
  type SliceDeliveryRecord,
} from './slice-delivery.ts';
export {
  isPlausibleSlice,
  type RatioImplausibility,
  ratioImplausibility,
  type SliceImplausibility,
  sliceImplausibility,
} from './slice-implausible.ts';
export {
  SLICE_CHAR_BUDGET,
  subdivideChunkPair,
  subdivideSealedChunkPair,
} from './slice-pair.ts';
export {
  assertSettledRecordAgrees,
  resumedSliceDiscardFinding,
  sliceRecordAgrees,
  SliceRecordContradictionError,
  type SliceRecordLane,
} from './slice-record-agreement.ts';
export { foldSoftBreaks, } from './soft-break-fold.ts';
export { sourceBreakDisplay, } from './source-break-display.ts';
export {
  absenceFinding,
  assertAbsentSliceFilled,
  blankAgainst,
  BlankSelectionError,
  type IncumbentKind,
  requireTranslateAbsence,
  type RunoffBasis,
  TranslateAbsenceError,
  type TranslateAbsenceReason,
  type UnfilledReason,
} from './translate-absence.ts';
export {
  atomFindings,
  type AtomRenderingPool,
  renderingPoolsOf,
} from './translate-atom-rendering.ts';
export {
  buildTranslateCandidates,
  laneCandidate,
  type LaneText,
  type TranslateCandidateSet,
  type TranslateCandidateValue,
  type TranslateOrigin,
} from './translate-candidates.ts';
export {
  type BlindGround,
  type ComparedVerdict,
  FloorGroundDisagreementError,
  type FloorReach,
  floorReach,
  type FrontMatterGround,
  type MarkdownGround,
  readFrontMatterGround,
  readMarkdownGround,
  requireComparedVerdict,
} from './translate-floor-ground.ts';
export { addedFormulaFindings, } from './translate-formula.ts';
export { guillemetFindings, } from './translate-guillemets.ts';
export {
  hanResidueFindings,
  type HanRun,
  hanRuns,
  untranslatedOrResidueFindings,
} from './translate-han-residue.ts';
export { neutralPronounFindings, } from './translate-neutral-pronoun.ts';
export {
  buildTranslateRepairMessages,
  isTranslateRepairWire,
  type RepairResolution,
  TRANSLATE_REPAIR_RESPONSE_FORMAT,
  type TranslateRepairWire,
} from './translate-repair-wire.ts';
export {
  repairInvalidCandidates,
  type RepairOutcome,
} from './translate-repair.ts';
export {
  TRANSLATE_CHALLENGE_TASK,
  TRANSLATE_FRONT_MATTER_CRITERIA,
  TRANSLATE_LINE_STRUCTURE_CRITERION,
  TRANSLATE_SELECTION_CRITERIA,
  TRANSLATE_SELECTION_TASK,
  type TranslateJudgeResponsibility,
  translateSelectionCriteria,
  translateSelectionTask,
} from './translate-selection-sheet.ts';
export { sheetLeakFindings, } from './translate-sheet-leak.ts';
export {
  describeSlate,
  NOT_ON_SLATE,
  positionOf,
  rotateCandidates,
  type TranslateSlateEntry,
} from './translate-slate.ts';
export { untranslatedFindings, } from './translate-untranslated.ts';
export { unwrappedLinkFindings, } from './translate-unwrapped-link.ts';
export {
  type SliceValidation,
  validateTranslatedSlice,
} from './translate-validate.ts';
export {
  buildTranslateMessages,
  isTranslateReportWire,
  JUDGE_PICTURE_SCOPE_RULE,
  TRANSLATE_ATTESTED_RULE,
  TRANSLATE_FRONT_MATTER_RULE,
  TRANSLATE_LINE_STRUCTURE_RULE,
  TRANSLATE_PICTURE_SCOPE_RULE,
  TRANSLATE_RESPONSE_FORMAT,
  type TranslateFollowupEvidence,
  type TranslatePromptPlan,
  type TranslateReportWire,
} from './translate-wire.ts';
export {
  type FoldedTranslation,
  type FoldedVoices,
  foldTranslation,
  foldTranslatorVoices,
  requireFoldedVoices,
  UnfoldedTranslationError,
} from './translator-answer-fold.ts';
export {
  assertWordingCoherent,
  WordingCoherenceError,
} from './wording-coherence.ts';
export {
  sameWording,
  wordingKey,
} from './wording-key.ts';

//endregion Public index
