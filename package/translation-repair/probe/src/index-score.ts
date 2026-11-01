//region Score exports
// The artifact names a sample lists and the score readers and runs.
// Re-exported by `index.ts`, which is the package's one public entry.

export { listSettledNames, } from './corpus-run/sample-artifact-names.ts';
export {
  agreementRate,
  positionsOrNone,
  printAgreement,
  printPrecision,
} from './corpus-run/score-agreement-print.ts';
export {
  type FileReading,
  preGradeName,
  readNamedOrBeside,
  readOptional,
} from './corpus-run/score-agreement-read.ts';
export { printGradeReport, } from './corpus-run/score-agreement-run.ts';
export { requireArtifactsDir, } from './corpus-run/score-artifacts-dir.ts';
export { printAttribution, } from './corpus-run/score-attribution-run.ts';
export {
  attributionCriticLine,
  attributionHeaderLine,
} from './corpus-run/score-attribution-table.ts';
export {
  printMalformedArtifacts,
  printPartialJoinWarning,
  printUnattributedWarning,
} from './corpus-run/score-attribution-warnings.ts';
export {
  authorHeaderLine,
  authorLine,
  type AuthorRow,
  tallyAuthors,
} from './corpus-run/score-crosscheck-authors.ts';
export { printCrosscheck, } from './corpus-run/score-crosscheck-run.ts';
export {
  printCrosscheckMalformed,
  printJoinFailureWarning,
  printUnjudgeableWarning,
} from './corpus-run/score-crosscheck-warnings.ts';
export {
  countRefinedJoined,
  joinProbeGrades,
  printProbeAgreement,
} from './corpus-run/score-probe-agreement.ts';
export { gatherProbeReadings, } from './corpus-run/score-probe-gather.ts';
export { printProbeScore, } from './corpus-run/score-probe-run.ts';
export { readSheetText, } from './corpus-run/score-sheet-text.ts';
export {
  parseVerifyManifest,
  readVerifyManifest,
  type VerifyManifestItem,
} from './corpus-run/score-verify-manifest.ts';
export { printVerifyScore, } from './corpus-run/score-verify-run.ts';
export {
  type KindTally,
  tallyByKind,
  verifyKindLines,
} from './corpus-run/score-verify-tally.ts';

//endregion Score exports
