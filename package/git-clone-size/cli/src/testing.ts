/**
 Built entry the unit tests import.

 The package's only entry is the `git-clone-size` bin, which parses argv and measures
 when it loads, so no suite can import it. This module re-exports exactly the behavior
 the suites drive, so they load what this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the bin bundle. Nothing in this package imports it, and the CLI never loads it.

 @module
 */

//region Sources

/**
 Input classification and remote URL parsing.

 @internal
 */
export {
  isRemoteInput,
  parseRemoteUrl,
  type Source,
} from './source.ts';

//endregion Sources

//region Measurement estimators

/**
 Per-strategy byte estimators and the priors they start from.

 @internal
 */
export {
  churnEstimate,
  deepenEstimate,
  priorAbsentEstimate,
  priorEstimate,
} from './estimators.ts';
export {
  DEFAULT_MAX_DEEPEN_COMMITS,
  DEFAULT_MAX_PACK_BYTES,
  DEFAULT_MAX_PROBE_SECONDS,
  PRIOR_ABSENT_TIP_BYTES,
  PRIOR_MULTIPLIER,
} from './constants.ts';

/**
 The unmeasured sentinel and its narrowing predicate.

 @internal
 */
export {
  isMeasured,
  UNMEASURED,
} from './measure.ts';

/**
 Arrival-order merging for concurrent estimator work.

 @internal
 */
export { mergeAsArrived, } from './async-queue.ts';

//endregion Measurement estimators

//region Assembly

/**
 Combining estimates into a fused snapshot and reporting what is still pending.

 @internal
 */
export {
  buildEstimates,
  buildSnapshot,
  computePending,
  type Signals,
} from './assemble.ts';
export { combineEstimates, } from './combine.ts';
export { serializeSnapshot, } from './serialize.ts';
export {
  type EstimateSnapshot,
  type FusionState,
} from './types.ts';
export {
  estimate,
  type EstimateOptions,
} from './stream.ts';

//endregion Assembly

//region Exact local measurement

/**
 Walking a local repository to measure it exactly, and the pack-objects walk.

 @internal
 */
export {
  countObjectsSizePack,
  localExact,
} from './local-exact.ts';

//endregion Exact local measurement

//region Output formatting

/**
 Byte and ratio formatting, JSON colouring, and the temporary directory helper.

 @internal
 */
export {
  clamp,
  computeRatio,
  computeSavings,
  roundTo,
  toSize,
} from './format.ts';
export {
  colorizeJson,
  shouldColor,
} from './color.ts';
export { makeTempDir, } from './temp-dir.ts';

//endregion Output formatting
