//region Attribution and audit exports
// The attribution reader and report and the audit sensitivity probe.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  attributionEntryOf,
  type AttributionGather,
  gatherAttributionEntries,
  type MalformedArtifact,
} from './corpus-run/attribution-read.ts';
export {
  type AcceptedIssueView,
  type AttributionEntry,
  type AttributionReport,
  buildAttributionReport,
  type CriticTally,
  type ProposerView,
  type SliceCriticView,
} from './corpus-run/attribution-report.ts';
export {
  AUDIT_ARMS,
  SOURCE_TEXT as AUDIT_SOURCE_TEXT,
  ORACLE_CANDIDATE_SPAN,
  ORACLE_SOURCE_SPAN,
} from './corpus-run/audit-sensitivity-input.ts';
export {
  pointsAtOracle,
  shownText,
  sightedVoices,
} from './corpus-run/audit-sensitivity-oracle.ts';
export {
  armLines,
  keptLine,
  voiceLines,
} from './corpus-run/audit-sensitivity-print.ts';
export { runAuditSensitivity, } from './corpus-run/audit-sensitivity-run.ts';

//endregion Attribution and audit exports
