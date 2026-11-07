/**
 Library surface of the repository move tools, for the unit tests and the guided runner.

 @module
 */

export { MoveStopError, } from './errors.ts';
export {
  type MoveOptions,
  parseMoveOptions,
} from './move-options.ts';
export {
  parseInventory,
  resolveFullPaths,
} from './inventory.ts';
export { assignNames, } from './name-assignment.ts';
export {
  boundedSeconds,
  startDeadlineClock,
} from './deadline-clock.ts';
export { runMove, } from './run-move.ts';
export { buildUncommittedWorkTable, } from './uncommitted-work-table.ts';
export { createGuidedConfig, } from './guided-config.ts';
export {
  GuidedStop,
  runPhase,
} from './guided-engine.ts';
export {
  footprintOf,
  holderPatterns,
  preflight,
  prepareUnitEdit,
  selectVerify,
  windowMaximum,
} from './guided-internal.ts';
export {
  parseGuidedFlags,
  runGuided,
} from './guided-runner.ts';
export type {
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';
export { cutoverStages, } from './guided-cutover-stages.data.ts';
export { prepareStages, } from './guided-prepare-stages.data.ts';
export {
  rollbackStages,
  thawStages,
} from './guided-undo-stages.data.ts';
