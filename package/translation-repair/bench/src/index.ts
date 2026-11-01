//region Public index
// The names `@monochromatic-dev/translation-repair-bench` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { nodeEntries, } from './build-entries.ts';
export {
  BenchDrawError,
  type DrawableSlice,
  orderBySourceSize,
  pickSpreadSample,
} from './corpus-run/bench-draw.ts';
export {
  type BenchCall,
  type CallTokens,
  recordingClient,
} from './corpus-run/bench-record.ts';
export {
  BenchReportError,
  benchWidths,
  summarizeBench,
  writeBenchReport,
} from './corpus-run/bench-report.ts';
export {
  type BenchSlice,
  sampleBenchSlices,
  sliceListedEntries,
} from './corpus-run/bench-sample.ts';
export {
  ABSENT_ISSUE,
  ALL_FIXED_PATCHED_TEXT,
  ALL_FIXED_SOURCE_TEXT,
  SOURCE_TEXT as CHECKER_SOURCE_TEXT,
  DEFECTIVE_TEXT,
  MEANING_ISSUE,
  MIXED_SHEET_PATCHED_TEXT,
  SINGLE_ISSUE_CASES,
  TENSE_ISSUE,
} from './corpus-run/checker-sensitivity-input.ts';
export {
  CHECKER_NOTE,
  sheetCheckLine,
  singleCheckLine,
} from './corpus-run/checker-sensitivity-print.ts';
export {
  checkAllFixedSheet,
  checkMixedSheet,
  checkOne,
  runCheckerSensitivity,
} from './corpus-run/checker-sensitivity-run.ts';
export { reportLines, } from './corpus-run/displacement-probe-report.ts';
export {
  type EntryDisplacement,
  readEntry,
} from './corpus-run/displacement-probe-row.ts';
export { probeDisplacement, } from './corpus-run/displacement-probe-run.ts';
export {
  type CorpusTotals,
  corpusTotals,
  isNotable,
} from './corpus-run/displacement-probe-totals.ts';
export {
  printEditorCalibrateRefineReach,
  printEditorCalibrateShipped,
} from './corpus-run/editor-calibrate-closing.ts';
export {
  type EditorChair,
  editorChairFor,
} from './corpus-run/editor-calibrate-chair.ts';
export { driveEditorCalibrate, } from './corpus-run/editor-calibrate-drive.ts';
export {
  editorLaneChairOf,
  EDITOR_LANE_EDITORS,
  EDITOR_LANE_REFINERS,
  runEditorLane,
} from './corpus-run/editor-calibrate-lane.ts';
export { runEditorCalibrate, } from './corpus-run/editor-calibrate-run.ts';
export {
  shippedAuthors,
  sliceProgressLine,
  type SliceRounds,
} from './corpus-run/editor-calibrate-slice.ts';
export {
  slatedAuthors,
  sliceStandingLines,
  standingReportLines,
} from './corpus-run/editor-calibrate-standing.ts';
export { printEditorCalibrateStandings, } from './corpus-run/editor-calibrate-standings.ts';
export { noJudgedRoundLine, } from './corpus-run/editor-standing-absence.ts';
export {
  readStandingArtifact,
  type StandingReading,
} from './corpus-run/editor-standing-read-artifact.ts';
export { standingArtifactPaths, } from './corpus-run/editor-standing-read-listing.ts';
export { reportStandings, } from './corpus-run/editor-standing-read-run.ts';
export {
  everyStandingRound,
  printDigestStanding,
} from './corpus-run/editor-standing-read-seat.ts';
export {
  type ArmOutcome,
  runArm,
} from './corpus-run/editor-width-arm.ts';
export {
  armInSeat,
  bothOrders,
  seatThatWon,
} from './corpus-run/editor-width-contest.ts';
export {
  widthControlHolds,
  withoutASentence,
} from './corpus-run/editor-width-control.ts';
export {
  gatherWidthInput,
  type WidthInputOutcome,
  type WidthProbeInput,
} from './corpus-run/editor-width-input.ts';
export {
  classifyWidths,
  type HeadToHeadVerdict,
  readHeadToHead,
  summarizeWidths,
  type WidthArm,
  type WidthComparison,
  type WidthDraw,
  type WidthRow,
  type WidthSummary,
} from './corpus-run/editor-width-model.ts';
export {
  halfOfSample,
  readWidthDraw,
} from './corpus-run/editor-width-probe-draw.ts';
export { runWidthDraw, } from './corpus-run/editor-width-probe-loop.ts';
export { runEditorWidthProbe, } from './corpus-run/editor-width-probe-run.ts';
export { writeWidthReport, } from './corpus-run/editor-width-report.ts';
export { runWidthSlice, } from './corpus-run/editor-width-slice.ts';
export {
  CATALOG_MODEL_IDS,
  type CatalogComparison,
  compareCatalog,
  decodeModelList,
  formatCatalogReport,
  type ServedModel,
} from './corpus-run/model-catalog-compare.ts';
export { printModelCatalog, } from './corpus-run/model-catalog-print.ts';
export { askModelHealth, } from './corpus-run/model-health-probe.ts';
export { reportModelHealth, } from './corpus-run/model-health-report.ts';
export { runCalibrationRound, } from './corpus-run/producer-calibrate-round.ts';
export { runProducerCalibrate, } from './corpus-run/producer-calibrate-run.ts';
export { runBenchRow, } from './corpus-run/roster-bench-round.ts';
export { type BenchRow, } from './corpus-run/roster-bench-row.ts';
export { runRosterBench, } from './corpus-run/roster-bench-run.ts';
export {
  type MeasureAsk,
  readMeasureAsk,
} from './corpus-run/reader-measure-ask.ts';
export { runReaderMeasure, } from './corpus-run/reader-measure-command.ts';
export { filesUnder, } from './corpus-run/reader-measure-files.ts';
export { loadPictures, } from './corpus-run/reader-measure-pictures.ts';
export {
  matchRecord,
  readTrustedRecords,
  type RecordMatch,
} from './corpus-run/reader-measure-records.ts';
export { renderMeasureReport, } from './corpus-run/reader-measure-report.ts';
export {
  type CandidateOutcome,
  measureReader,
  type MeasuredPicture,
  type MeasuredRow,
} from './corpus-run/reader-measure-run.ts';
export {
  seatedBandOf,
  type CandidateRates,
  candidateRatesOf,
  type MeasureVerdict,
  MINIMUM_PRODUCED_READINGS,
  type SeatedBand,
  type TrustedReading,
  type TrustedRecord,
  verdictOf,
} from './corpus-run/reader-measure-score.ts';
export { readAsk, } from './corpus-run/roster-card-ask.ts';
export {
  fetchListing,
  LISTING_URL,
} from './corpus-run/roster-card-listing.ts';
export { printRosterCard, } from './corpus-run/roster-card-print.ts';
export {
  type CardFields,
  cardFieldsFrom,
  fieldAt,
  type Listed,
  listingRowFor,
  NOT_LISTED,
  renderProviderCard,
} from './corpus-run/roster-card-render.ts';
export {
  type ProbedResult,
  probeErrorLine,
  probeResultLine,
} from './corpus-run/sentinel-probe-line.ts';
export { probeCorpusEntries, } from './corpus-run/sentinel-probe-run.ts';
export {
  coverageOf,
  sparsestPair,
} from './corpus-run/translate-probe-coverage.ts';
export {
  heardLines,
  PROBE_SLICES,
  sectionLine,
  sliceHeading,
  slicesLine,
} from './corpus-run/translate-probe-lines.ts';
export { probeTranslate, } from './corpus-run/translate-probe-run.ts';
export {
  CONTROL_CLASS,
  controlSlices,
  flaggedSlices,
  RELOCATION_CLASSES,
  type TrialSlice,
} from './corpus-run/window-trial-draw.ts';
export {
  accountTrialLedger,
  appendTrialRow,
  completedArms,
  readTrialLedger,
  trialKey,
  type WindowTrialRow,
} from './corpus-run/window-trial-ledger.ts';
export {
  armOrderFor,
  TRIAL_ARM_SET,
} from './corpus-run/window-trial-order.ts';
export {
  type PickOutcome,
  runPick,
} from './corpus-run/window-trial-pick.ts';
export {
  checkWindowReached,
  WINDOW_UNCHECKED,
} from './corpus-run/window-trial-probe-check.ts';
export { drawEntry, } from './corpus-run/window-trial-probe-draw.ts';
export {
  boughtLine,
  ledgerReadLine,
  openingLine,
  walkEndLine,
} from './corpus-run/window-trial-probe-lines.ts';
export { runWindowTrial, } from './corpus-run/window-trial-probe-run.ts';
export {
  NOTHING_BOUGHT,
  tallyRefusal,
  tallyRows,
} from './corpus-run/window-trial-probe-tally.ts';
export {
  protocolDigest,
  type SliceYield,
  streakAfter,
} from './corpus-run/window-trial-protocol.ts';
export { TrialSliceRefusalError, } from './corpus-run/window-trial-refusal.ts';
export {
  type ArmRate,
  type ClassReport,
  reportWindowTrial,
  type Transitions,
  TRIAL_ARMS,
  windowTrialReportLine,
} from './corpus-run/window-trial-report.ts';
export { runSliceArms, } from './corpus-run/window-trial-slice.ts';
export {
  assertWindowReachedJudges,
  type SheetWitness,
  WINDOW_LABEL,
  WindowEvidenceError,
  witnessSheets,
} from './corpus-run/window-trial-witness.ts';

//endregion Public index
