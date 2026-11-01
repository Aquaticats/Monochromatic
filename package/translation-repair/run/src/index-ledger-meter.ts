//region Price, ledger and meter exports
// The provider prices, the ledger reader and report, and the meter reports.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  type CreditRates,
  creditsFor,
  HYPER_PRICE_READ_ON,
  ratesFor,
} from './corpus-run/hyper-price.ts';
export {
  type LedgerReading,
  readLedgerDirectory,
  refusalOf,
  type RefusedFile,
} from './corpus-run/ledger-directory.ts';
export {
  LedgerShapeError,
  parseLedgerRound,
  type ReadBallot,
  type ReadCandidate,
  type ReadRound,
} from './corpus-run/ledger-parse.ts';
export {
  type CandidateReading,
  type LedgerSummary,
  type ModelWork,
  summariseLedger,
  workOfModel,
} from './corpus-run/ledger-read.ts';
export {
  printReading,
  printRefusals,
  printSeat,
  printSummary,
} from './corpus-run/ledger-report-print.ts';
export { reportLedger, } from './corpus-run/ledger-report-run.ts';
export {
  countStates,
  type DrySpan,
  drySpans,
  dutyCycle,
  longestDrySpan,
  seriesFor,
  type StateCounts,
} from './corpus-run/meter-dry-span.ts';
export { levelLines, } from './corpus-run/meter-report-level.ts';
export { reportProvider, } from './corpus-run/meter-report-provider.ts';
export {
  mergeSamples,
  reportMeters,
} from './corpus-run/meter-report-run.ts';
export {
  outageLines,
  spanText,
  stampText,
} from './corpus-run/meter-report-text.ts';
export {
  type MeterLogReading,
  type MeterSample,
  readMeterLine,
  readMeterLog,
} from './corpus-run/meter-sample-read.ts';

//endregion Price, ledger and meter exports
