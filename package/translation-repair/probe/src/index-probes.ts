//region Judge and probe exports
// The judge cross-check and independence, and the relabel, sensitivity and verify probes.
// Re-exported by `index.ts`, which is the package's one public entry.

export {
  buildCrosscheckCensus,
  type CrosscheckArm,
  type CrosscheckCensus,
  type CrosscheckItem,
  statusBreakdown,
} from './corpus-run/judge-crosscheck.ts';
export {
  DAMAGE_KINDS,
  DEFAULT_TRIAL_CAP,
  readFidelityArguments,
} from './corpus-run/judge-fidelity-args.ts';
export {
  type BoundedFidelityMatrix,
  boundFidelityMatrix,
  fidelityJudgedLine,
  fidelityKeptLine,
  fidelityKeptRun,
  fidelityPartialWarning,
  type FidelityPlan,
  fidelityPlan,
  type FidelityPlannedRow,
  fidelityPreflightLine,
  type FidelityRowResult,
  fidelityRowResult,
} from './corpus-run/judge-fidelity-probe-plan.ts';
export {
  type FidelityProbeSink,
  runFidelityProbe,
} from './corpus-run/judge-fidelity-probe-run.ts';
export {
  type JudgeSeating,
  MIN_JUDGED_CLAIMS,
  seatJudges,
} from './corpus-run/judge-independence.ts';
export {
  probeRosterWith,
  readCandidateIds,
  readCandidatesAlone,
} from './corpus-run/probe-candidates.ts';
export {
  type ArtifactRecord,
  readArtifactRecords,
} from './corpus-run/probe-relabel-artifact.ts';
export {
  DAMAGED_CASES,
  gatherRelabelCases,
  locateSlice,
  type RelabelCase,
} from './corpus-run/probe-relabel-case.ts';
export { gatherControlCases, } from './corpus-run/probe-relabel-control.ts';
export {
  otherDisclosure,
  probePair,
} from './corpus-run/probe-relabel-pair.ts';
export {
  relabelCaseLines,
  relabelClaimLines,
  relabelCounts,
  relabelGathered,
  relabelNotes,
  relabelRebuilt,
} from './corpus-run/probe-relabel-print.ts';
export { runProbeRelabel, } from './corpus-run/probe-relabel-run.ts';
export { nowAsIso, } from './corpus-run/probe-run-clock.ts';
export {
  type IssueLabel,
  type PriorIssueList,
  PRODUCTION_LIST,
  SENSITIVITY_ARMS,
  type SensitivityArm,
} from './corpus-run/probe-sensitivity-arms.ts';
export {
  sensitivityArmLines,
  sensitivityNotes,
  sensitivityOpening,
} from './corpus-run/probe-sensitivity-print.ts';
export { runSensitivity, } from './corpus-run/probe-sensitivity-run.ts';
export { singleRegionTally, } from './corpus-run/probe-single-tally.ts';
export {
  persistProbeRun,
  type ProbeRun,
} from './corpus-run/probe-store.ts';
export {
  type GatheredProbe,
  reportProbeTelemetry,
} from './corpus-run/probe-telemetry-report.ts';
export {
  collectFlagged,
  keepAdmissible,
} from './corpus-run/probe-verify-collect.ts';
export {
  runProbeVerify,
  VERIFY_BLIND_NOTE,
  verifyOpening,
  verifyWrote,
} from './corpus-run/probe-verify-run.ts';
export {
  formatVerifyManifest,
  formatVerifySheet,
  orderBlind,
  type SheetFraming,
  type VerifyItem,
} from './corpus-run/probe-verify-sheet.ts';

//endregion Judge and probe exports
