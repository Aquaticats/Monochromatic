//region Settled artifact keys
// The top-level keys a settled two-lane artifact is written with, in the
// order the builder writes them. Split out of `artifact-two-lane-read.ts` at
// its line budget when generation thirteen added the page assembly.

/**
 Top-level keys a settled artifact may carry, every generation's together;
 which of them a given generation requires is each section's reader's
 question.
 */
export const SETTLED_ARTIFACT_KEYS: readonly string[] = [
  'artifactSchemaVersion',
  'id',
  'tip',
  'pipelineDigest',
  'corpusSha',
  'callConfig',
  'durationMs',
  'timestamp',
  'preparation',
  'lanes',
  'comparison',
  'laneSelection',
  'consolidation',
  'pageAssembly',
  'consistency',
];

//endregion Settled artifact keys
