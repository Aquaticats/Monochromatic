//region Editor fidelity exports
// The editor fidelity probe's trials, score, arguments, ceiling, spend guard, client, trial, plan and run.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  DEFAULT_EDITOR_MAX_USD,
  DEFAULT_EDITOR_TRIAL_CAP,
  EDITOR_BENCH_MAX_USD,
  readEditorFidelityArguments,
} from './corpus-run/editor-fidelity-args.ts';
export {
  callCeilingUsd,
  ceilingLine,
  type EditorCeiling,
  editorCeilings,
  MAX_ASKS_PER_TRIAL,
  messageChars,
  pricesOf,
  tierFor,
  usdText,
} from './corpus-run/editor-fidelity-ceiling.ts';
export {
  cardsClientFrom,
  cardsOnlyClient,
  editorProbeEnv,
} from './corpus-run/editor-fidelity-client.ts';
export {
  drawPoolEntry,
  type DrawnEntry,
} from './corpus-run/editor-fidelity-pool-draw-entry.ts';
export {
  type PoolDrawRunInput,
  runEditorFidelityPoolDraw,
} from './corpus-run/editor-fidelity-pool-draw-run.ts';
export {
  drawPool,
  type EntryPages,
  type PoolDraw,
} from './corpus-run/editor-fidelity-pool-selection.ts';
export {
  manifestDigest,
  manifestLine,
  openingLines,
} from './corpus-run/editor-fidelity-opening.ts';
export { poolLine, } from './corpus-run/editor-fidelity-pool-line.ts';
export {
  type Section,
  sectionsOf,
} from './corpus-run/editor-fidelity-pool-sections.ts';
export {
  joinedBySection,
  specForDamage,
} from './corpus-run/editor-fidelity-pool-spec.ts';
export {
  patchedWith,
  scoreableVerdict,
} from './corpus-run/editor-fidelity-pool-verdict.ts';
export {
  type Answer,
  errorAnswer,
  exitAnswer,
  guardAnswer,
  manifestAnswer,
  poolAnswer,
  seatAnswer,
  spendAnswer,
} from './corpus-run/editor-fidelity-arm-answers.ts';
export {
  ceilingAnswer,
  editorAnswer,
  plannedAnswer,
  rowAnswer,
} from './corpus-run/editor-fidelity-arm-trial-answers.ts';
export { checkEditorBench, } from './corpus-run/editor-fidelity-arm-bench.ts';
export {
  type ArmFigures,
  armFiguresOf,
  type KindFigure,
  type RowStops,
} from './corpus-run/editor-fidelity-arm-figures.ts';
export { benchReportLines, } from './corpus-run/editor-fidelity-arm-report.ts';
export { fixRateLinesOf, } from './corpus-run/editor-fidelity-arm-fix-rate.ts';
export {
  REPEAT_ALPHA,
  type RepeatAgreementTest,
  repeatAgreementTest,
  type RepeatCounts,
  type RepeatShortfall,
  repeatShortfallsOf,
} from './corpus-run/editor-fidelity-arm-repeat-test.ts';
export { repeatLinesOf, } from './corpus-run/editor-fidelity-arm-repeats.ts';
export {
  stopsStatedAnswer,
  trialEndsAnswer,
  UNUSABLE_PLUS_REFUSED_WARNED_SHARE_PERCENT,
  unusableWarning,
} from './corpus-run/editor-fidelity-arm-stops.ts';
export {
  type AskEnd,
  askEndOf,
  recordingAsks,
  type RecordedAsks,
  stoppedWord,
} from './corpus-run/editor-fidelity-asks.ts';
export {
  decideEffort,
  decisionRuleText,
  MIN_SHARED_TRIALS,
} from './corpus-run/editor-fidelity-decide.ts';
export type {
  DecidingClause,
  EffortReading,
  EffortVerdict,
  KindEstimate,
  SharedTrial,
} from './corpus-run/editor-fidelity-decide-types.ts';
export {
  effortOf,
  type SharedBench,
  sharedBenchOf,
} from './corpus-run/editor-fidelity-arm-shared.ts';
export {
  ALPHA,
  BOOTSTRAP_REPLICATES,
  BOOTSTRAP_SEED,
  clusterBootstrapInterval,
  INTERVAL_LEVEL,
  type PairedCluster,
  type PairedInterval,
  pairedPermutationTest,
  PairedShapeError,
  type PairedTest,
  unitDraws,
} from './corpus-run/editor-fidelity-paired.ts';
export {
  passScopeOf,
  scopedToPass,
} from './corpus-run/editor-fidelity-pass-scope.ts';
export {
  type ArmCheckReport,
  checkEditorArms,
} from './corpus-run/editor-fidelity-arm-check.ts';
export {
  type ArmFacts,
  type ArmLog,
  bodyOf,
  countWithNoun,
  type ExitRecord,
  factsOf,
  firstStarting,
  numberAfter,
  usdSaid,
  wordAfter,
} from './corpus-run/editor-fidelity-arm-facts.ts';
export {
  type EditorCardTally,
  editorDecisionRuleLine,
  editorDefectOnlyLine,
  type EditorFidelityPlan,
  editorFidelityPlan,
  type EditorFidelityRow,
  editorFidelityRow,
  editorKeptRun,
  editorKindLines,
  type EditorKindTally,
  type EditorPair,
  type EditorPlannedRow,
  editorTallyLine,
  editorTallyOf,
} from './corpus-run/editor-fidelity-plan.ts';
export {
  editorKeptLine,
  editorPreflightLine,
} from './corpus-run/editor-fidelity-plan-lines.ts';
export {
  pairName,
  readingWord,
  reportTallies,
  rowLineOf,
  seatedCardsOf,
  sheetCharsOf,
} from './corpus-run/editor-fidelity-probe-read.ts';
export {
  cappedLine,
  plannedLine,
  trialsLine,
} from './corpus-run/editor-fidelity-progress-lines.ts';
export {
  type EditorProbeSink,
  runEditorFidelityProbe,
} from './corpus-run/editor-fidelity-probe-run.ts';
export {
  ConcurrentGuardCallError,
  type GuardedClient,
  guardLine,
  type GuardTotal,
  guardTotalLine,
  spendGuardedClient,
} from './corpus-run/editor-fidelity-spend-guard.ts';
export {
  type EditorTrialReading,
  runEditorTrial,
} from './corpus-run/editor-fidelity-trial.ts';
export {
  type EditorSheetContext,
  editorSheetContextOf,
  sheetFieldsOf,
} from './editor-fidelity-context.ts';
export {
  differingMiddle,
  digitRunAround,
} from './editor-fidelity-difference.ts';
export { DRAWN_EDITOR_SPECS, } from './editor-fidelity-pool-drawn.ts';
export { EDITOR_FIDELITY_POOL, } from './editor-fidelity-pool-manifest.ts';
export {
  DEFAULT_POOL_SEED,
  furthestFirst,
  isPlainEntryId,
  POOL_DRAWN_ON,
  POOL_TRIALS_WANTED,
  seededEntryOrder,
  SPECS_PER_PART,
} from './editor-fidelity-pool-rule.ts';
export {
  type PoolFile,
  poolSourceFiles,
} from './editor-fidelity-pool-source.ts';
export { insertionRegionWithNeighbours, } from './editor-fidelity-region.ts';
export {
  type EditorFidelityScore,
  type EditorFixKind,
  scoreEditedText,
} from './editor-fidelity-score.ts';
export {
  EDITOR_DAMAGE_KINDS,
  type EditorDamageKind,
  type EditorFidelityTrial,
  editorFidelityTrials,
} from './editor-fidelity-trials.ts';

//endregion Editor fidelity exports
