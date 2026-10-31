//region Timing, runs directory and spend exports
// Run timing, the runs directory and its lock, the spend report and the published-page check.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  type CallReading,
  type CallTiming,
  readCallTiming,
  readRoundTiming,
  type RoundReading,
  type RoundTiming,
  STREAM_MARKER,
  TimingFieldError,
  TimingLineError,
} from './corpus-run/run-timing-parse.ts';
export {
  type InFlight,
  measureInFlight,
  NothingInFlightError,
  readRunTiming,
  type RunTiming,
  summariseRounds,
} from './corpus-run/run-timing-read.ts';
export {
  asSpan,
  printInFlight,
  printRounds,
} from './corpus-run/run-timing-report-print.ts';
export { printRunRecords, } from './corpus-run/run-records-run.ts';
export { reportRunTiming, } from './corpus-run/run-timing-report-run.ts';
export {
  defaultRunsDirIn,
  prepareRunsLayout,
  PROMPT_PAYLOADS_DIR,
  RUNS_DIR_NAME,
  type RunsLayout,
  SLICE_CACHE_DIR,
} from './corpus-run/runs-layout.ts';
export {
  holderLiveness,
  lockFileText,
} from './corpus-run/runs-lock-holder.ts';
export {
  evictStaleLock,
  lockRunsDir,
  releaseIfOwned,
  RunsDirectoryBusyError,
} from './corpus-run/runs-lock.ts';
export {
  resolveSpendCeilingUsd,
  SPEND_CEILING_PROVIDERS,
  SPEND_CEILING_USD,
  SPEND_CEILING_VAR,
  spendCeilingNote,
  SpendCeilingOverrideError,
  spendCeilingOverrideNote,
  spendCeilingReached,
} from './corpus-run/spend-ceiling.ts';
export {
  type PricedSeat,
  priceTally,
  type SpendCost,
} from './corpus-run/spend-cost.ts';
export {
  readSpendLine,
  type SeatSpend,
  type SpendCount,
  type SpendLineReading,
  type SpendRecord,
  type SpendTally,
  type SpendUsd,
  tallySpend,
} from './corpus-run/spend-read.ts';
export {
  asCredits,
  asUsd,
  pricedLine,
  tokensOnlyLine,
  usdLine,
} from './corpus-run/spend-report-line.ts';
export { printCost, } from './corpus-run/spend-report-print.ts';
export { reportSpendCost, } from './corpus-run/spend-report-run.ts';
export {
  consistencyColumnOf,
  type JudgedEntry,
  judgePublishedEntry,
} from './corpus-run/verify-published-entry.ts';
export { sourceDroppedLinesOf, } from './corpus-run/verify-published-source-dropped.ts';
export {
  judgePublishedEntries,
  verifyPublishedRun,
} from './corpus-run/verify-published-run.ts';

//endregion Timing, runs directory and spend exports
