//region Public index
// The names `@monochromatic-dev/translation-repair-artifact-pool` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  ArtifactParseError,
  requireArray,
  requireArtifactParseRefusal,
  requireBoolean,
  requireCount,
  requireFinite,
  requireRecord,
  requireString,
  requireStringList,
} from './artifact-guard.ts';
export {
  parseRecordRepair,
  type RecordRepairReading,
} from './artifact-repair-read.ts';
export {
  type EligibleEntries,
  selectEligible,
} from './corpus-run/artifact-eligible.ts';
export {
  type ArtifactFileName,
  artifactFileNameOf,
  artifactFilesIn,
  ARTIFACTS_DIR,
  artifactsDirOf,
  entryIdOfArtifact,
  isArtifactFileName,
  listArtifactFiles,
} from './corpus-run/artifact-file-name.ts';
export {
  censusByGeneration,
  type GenerationCensus,
  type GenerationGroup,
  resolveCommit,
  tipContains,
} from './corpus-run/artifact-generation.ts';
export {
  type Placement,
  readPlacement,
} from './corpus-run/artifact-placement.ts';
export {
  EmptyPoolError,
  generationLines,
  MixedGenerationError,
  pluralEntries,
} from './corpus-run/artifact-pool-refusal.ts';
export {
  keepEligible,
  type PoolPolicy,
  readPoolPolicy,
  resolvePool,
} from './corpus-run/artifact-pool.ts';
export {
  abbreviate,
  ArtifactProvenanceError,
  assertArtifactProvenance,
  type ExpectedProvenance,
  type GenerationSelection,
} from './corpus-run/artifact-provenance.ts';
export {
  ArtifactComparisonError,
  type ArtifactComparisonFault,
} from './corpus-run/artifact-two-lane-comparison-fault.ts';
export {
  assertDerivationsAgree,
  compareLanes,
} from './corpus-run/artifact-two-lane-comparison.ts';
export { assertRecordedComparisonMatches, } from './corpus-run/artifact-two-lane-read-comparison.ts';
export { carveDivergence, } from './corpus-run/artifact-two-lane-rebuild-rows.ts';
export {
  comparisonRowDifferences,
  type ComparisonRowField,
  decisionsEqual,
  deliveriesEqual,
  outcomesEqual,
} from './corpus-run/artifact-two-lane-row-equality.ts';
export {
  type ArtifactComparisonRow,
  type ArtifactDecisionComparison,
  type ArtifactDeliveryRow,
  type ArtifactLaneRelation,
  type ArtifactSliceDelivery,
  type ArtifactSliceOutcome,
} from './corpus-run/artifact-two-lane-vocabulary.ts';

//endregion Public index
