//region Consistency pass exports
// The consistency pass: its ask, carry, apply, settle and bench steps.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  editedStartOf,
  gainedBy,
  type GainSlice,
  type GainVerdict,
  madeIn,
  unprovenOnPage,
  withoutUnprovenGains,
} from './corpus-run/pass-consistency-apply-gains.ts';
export {
  type GroupEdit,
  replacedKeyOf,
  type SettledIndex,
  settledIndexOf,
} from './corpus-run/pass-consistency-apply-index.ts';
export {
  gainSliceOf,
  idsOf,
  keyOf,
  NO_POSITION,
  pageFrameOf,
  positionOfSlice,
  type ReadmitLedger,
  readmitLedgerOf,
  replacedAt,
} from './corpus-run/pass-consistency-apply-ledger.ts';
export {
  editsChangingLocatedWords,
  type LeftOutEdit,
} from './corpus-run/pass-consistency-apply-located.ts';
export { putBackInPageOrder, } from './corpus-run/pass-consistency-apply-order.ts';
export {
  plansOnPage,
  settledSlicesOf,
} from './corpus-run/pass-consistency-apply-plan.ts';
export { readmittedSlicesOf, } from './corpus-run/pass-consistency-apply-readmit.ts';
export {
  carriedPlaceName,
  editId,
  finalGroupsOf,
  pendingNamed,
  placeKey,
  recordedRendering,
  type SliceEdits,
  sliceEditsOf,
  winnerOver,
} from './corpus-run/pass-consistency-apply-record.ts';
export { screenSide, } from './corpus-run/pass-consistency-apply-screen.ts';
export {
  type FloorsAsked,
  floorsQuestionOf,
  gainFindingOf,
  type GainReason,
  keptReasonOfGain,
  type SettledSlice,
  sliceIndexOf,
  type SlicePlan,
} from './corpus-run/pass-consistency-apply-settled.ts';
export {
  floorsWithSplice,
  madeWithSplice,
  type Trial,
  trialOf,
  type WithSplice,
} from './corpus-run/pass-consistency-apply-trial.ts';
export {
  type WorkedPage,
  workedPageOf,
} from './corpus-run/pass-consistency-apply-worked.ts';
export {
  type AppliedConsistency,
  applyConsistency,
} from './corpus-run/pass-consistency-apply.ts';
export {
  askConsistencyBench,
  type ConsistencyAsked,
} from './corpus-run/pass-consistency-ask.ts';
export {
  type AskedDecision,
  decideAskedGroup,
  type PreparedGroup,
} from './corpus-run/pass-consistency-bench-decide.ts';
export {
  askAll,
  writersOf,
} from './corpus-run/pass-consistency-bench.ts';
export {
  type ConsistencyCacheValue,
  type ConsistencyChoiceRecord,
  consistencyKey,
  type ConsistencyRecord,
  type ConsistencyStage,
  type ConsistencyWritersRecord,
  type ConsistencyWriterVoice,
  openConsistencyCache,
} from './corpus-run/pass-consistency-cache.ts';
export {
  askCarryChoice,
  askCarryWriters,
  type CarryChoiceAsked,
  type CarryWritersAsked,
} from './corpus-run/pass-consistency-carry-ask.ts';
export {
  composeSlice,
  floorRefusalOf,
  writtenParagraphsOf,
} from './corpus-run/pass-consistency-carry-candidates.ts';
export {
  carryWithoutChoice,
  choose,
} from './corpus-run/pass-consistency-carry-choose.ts';
export {
  crossingOfUnit,
  crossingWordsLost,
  crossingWordsOf,
} from './corpus-run/pass-consistency-carry-crossing.ts';
export {
  editsOf,
  partlyCarriedFindingOf,
} from './corpus-run/pass-consistency-carry-edits.ts';
export {
  type CarryShown,
  shownOf,
  splicedParagraphOf,
  writtenOffers,
} from './corpus-run/pass-consistency-carry-offers.ts';
export {
  englishOf,
  englishOfUnit,
  type HeldReceiver,
  isKeptOutcome,
  type KeptWords,
  keptWordsOf,
  sliceOfUnit,
  unitLabelsOf,
  unitsOf,
} from './corpus-run/pass-consistency-carry-units.ts';
export {
  carryAll,
  carryToUnit,
} from './corpus-run/pass-consistency-carry.ts';
export {
  afterRunOff,
  chosenPlaceOf,
  plannedPlace,
} from './corpus-run/pass-consistency-decide.ts';
export {
  type EdgeRefusal,
  edgeRefusalOf,
  type EdgeVerdict,
} from './corpus-run/pass-consistency-edge-refusal.ts';
export {
  type ConsistencyFloorVerdict,
  consistencyFloorVerdict,
} from './corpus-run/pass-consistency-floors.ts';
export {
  gatherThroughCache,
  recordFits,
} from './corpus-run/pass-consistency-gather.ts';
export { groupLine, } from './corpus-run/pass-consistency-group-line.ts';
export {
  type ConsistencyPlace,
  dispositionOf,
  type GroupDisposition,
  occurrenceViewOf,
  placesOf,
  repeatOf,
} from './corpus-run/pass-consistency-group.ts';
export { runOffResultLine, } from './corpus-run/pass-consistency-run-off-line.ts';
export {
  askRunOff,
  choiceRecordOf,
  type ConsistencyRunOffAsked,
  storedChoices,
} from './corpus-run/pass-consistency-run-off.ts';
export {
  countAsked,
  runOffReasonOf,
  settleAsked,
  settleCarried,
} from './corpus-run/pass-consistency-settle.ts';
export {
  type ConsistencySliceFacts,
  consistencySlicesOf,
} from './corpus-run/pass-consistency-slices.ts';
export {
  type CarryContext,
  type CarryReceiver,
  type CarryUnit,
  type CarryWriters,
  type DecidedGroup,
  type LocatedPlace,
  type PendingEdit,
  type SettledGroup,
  type UnitCarried,
} from './corpus-run/pass-consistency-types.ts';
export {
  settleFailed,
  settleUnasked,
  unplacedOf,
} from './corpus-run/pass-consistency-unasked.ts';
export { runPassConsistency, } from './corpus-run/pass-consistency.ts';

//endregion Consistency pass exports
