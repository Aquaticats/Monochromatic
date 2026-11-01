/**
 Library surface of the yuku-based container-native mutation tester.
 
 @example
 ```ts
 import { enumerateMutants } from '@monochromatic-dev/mutation-test-cli/ts';
 ```
 */

export {
  enumerateMutants,
  type EnumerationResult,
  type IgnoredMutant,
} from './engine/enumerate.ts';
export {
  lineStarts,
  positionAt,
} from './engine/lines.ts';
export { mutantId, } from './engine/mutant-id.ts';
export { findOperatorToken, } from './engine/operator-token.ts';
export {
  allOperators,
  type OperatorFn,
} from './engine/operator/index.ts';
export { spliceReplacement, } from './engine/splice.ts';
export {
  matchingSuppressions,
  suppressionRules,
  type ParsedComment,
  type SuppressionRule,
} from './engine/suppression.ts';
export type {
  EstreeNode,
  Mutant,
  MutantStatus,
  OperatorName,
  Replacement,
} from './engine/types.ts';
export { isEstreeNode, } from './engine/node-access.ts';
export { effectiveTimeoutMs, } from './container/mutant-loop.ts';
/**
 Work-tree git marker shape and writer, exported so built-artifact tests
 can verify the shape against fs-path's `GIT_REPOSITORY` marker.

 @internal
 */
export {
  GIT_MARKER_DIRECTORIES,
  GIT_MARKER_FILES,
  materialiseGitMarker,
  RSYNC_EXCLUDES,
  rsyncSourceArgs,
} from './container/worktree.ts';
export {
  parseCliOptions,
  type CliOptions,
} from './host/cli-options.ts';
export { listSourceFiles, } from './host/source-file-list.ts';
export {
  chunk,
  composeReshard,
  composeShards,
  type MutantGroup,
} from './host/shards.ts';
export { sanitizeShardTag, } from './host/shard-tag.ts';
export {
  selectSources,
  selectTests,
  stemsRelated,
} from './host/selection.ts';
export {
  buildRunReport,
  formatTerminalSummary,
  RUN_REPORT_SCHEMA_VERSION,
  type RunMutantRecord,
  type RunReport,
  type RunTotals,
} from './host/report.ts';
export type {
  FinalMutantResult,
  OrchestrateOptions,
  RunOutcome,
} from './host/orchestrate-types.ts';
export {
  SHARD_SCHEMA_VERSION,
  type ShardBaseline,
  type ShardManifest,
  type ShardMutantResult,
  type ShardReport,
} from './shard-schema.ts';
export {
  BAKED_ENTRYPOINT,
  MANIFEST_MOUNT,
  REPORT_MOUNT,
  SOURCE_FILE_LIST_NAME,
  SOURCE_MOUNT,
  WORK_MOUNT,
} from './mounts.ts';
