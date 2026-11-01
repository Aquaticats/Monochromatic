//region Recall and rendering audit exports
// The recall benchmark and the settled rendering audit.
// Re-exported by `index.ts`, which is the package's one public entry.

export { chooseRecallEntries, } from './corpus-run/recall-benchmark-choose.ts';
export { seedRecallEntry, } from './corpus-run/recall-benchmark-entry.ts';
export {
  recallPlanLine,
  recallScorecardLines,
  recallStartLine,
  refuseUnmeasuredScorecard,
} from './corpus-run/recall-benchmark-report.ts';
export { runRecallBenchmark, } from './corpus-run/recall-benchmark-run.ts';
export {
  persistRecallScorecard,
  RECALL_SCORECARD_DIR,
  type RecallScorecardRecord,
} from './corpus-run/recall-scorecard-store.ts';
export {
  type AuditArguments,
  readAuditArguments,
  readReportArguments,
  type ReportArguments,
} from './corpus-run/rendering-audit-settled-args.ts';
export {
  type AuditRepeatBand,
  repeatBandOf,
} from './corpus-run/rendering-audit-settled-band.ts';
export {
  auditOne,
  capped,
  type CitedSubject,
  eligibleSubjects,
  printPopulation,
  withCitedReferences,
} from './corpus-run/rendering-audit-settled-buy.ts';
export {
  digestAuditedText,
  sameAuditedText,
  textIdentityOf,
} from './corpus-run/rendering-audit-settled-digest.ts';
export {
  runSettledAudit,
  settledArtifactRecord,
  settledBuyingLine,
} from './corpus-run/rendering-audit-settled-drive.ts';
export {
  readArchiveSubjects,
  readArtifactSubjects,
  type SettledArtifactReading,
  type SettledVerification,
} from './corpus-run/rendering-audit-settled-input.ts';
export {
  printBand,
  printRelations,
  printRelocations,
  printSplit,
  printVoices,
} from './corpus-run/rendering-audit-settled-print.ts';
export { printSettledRow, } from './corpus-run/rendering-audit-settled-progress.ts';
export {
  type AudienceSplit,
  type AuditRelocationPair,
  rateByVoice,
  splitFor,
  type VoiceRate,
} from './corpus-run/rendering-audit-settled-read.ts';
export {
  pageRelationFor,
  pageRelationLabel,
  pageRelationOf,
  type PageRelationTally,
  relationTallyOf,
  type SettledPageRelation,
} from './corpus-run/rendering-audit-settled-relation.ts';
export {
  auditRelocationPairs,
  distinctSlicePairs,
} from './corpus-run/rendering-audit-settled-relocation.ts';
export {
  type AuditRepeatPair,
  auditRepeatsAcross,
  type AuditRepeatSide,
  auditRepeatsWithin,
} from './corpus-run/rendering-audit-settled-repeat.ts';
export { runSettledReport, } from './corpus-run/rendering-audit-settled-report-run.ts';
export {
  type AuditedTextIdentity,
  type SettledAuditRow,
} from './corpus-run/rendering-audit-settled-row.ts';
export {
  newestRun,
  printAcross,
  readRunRows,
} from './corpus-run/rendering-audit-settled-runs.ts';
export {
  identityOf,
  type SettledAuditSubject,
  type SettledIdentity,
  type SettledReferences,
  subjectsOf,
} from './corpus-run/rendering-audit-settled-subject.ts';

//endregion Recall and rendering audit exports
