/**
 The library surface of the provider pools a run shares: the table, the counts, the gate, the refusal a closed pool gives, the
 launcher's words, and the placement of a run on one pool or on Pi's virtual writer. `index-of-starting-a-run.ts` re-exports it,
 and `index.ts` re-exports that.

 @module
 */

export {
  AUDIT_RUN_PREFIX,
  CLOSED_POOL_LIMIT,
  KIMI_RUN_PREFIX,
  MODEL_OF_A_RUN_NAMING_NONE,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_KIMI_FLEX,
  NEURALWATT_STANDARD,
  OPENCODE_GO,
  POOL_LIMITS_VARIABLE,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  PROVIDER_POOLS,
  QWEN_PLAN_DEEPSEEK,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  VIRTUAL_POOLS,
  VIRTUAL_REVIEWER,
  VIRTUAL_WRITER,
  type PoolMembership,
  PoolLimitsError,
  type ProviderPool,
  type VirtualPool,
  NO_VIRTUAL_POOL,
  limitsIn,
  membershipOf,
  membershipOfAVirtualSelection,
  membershipOfSelection,
  poolNamed,
  poolOfThatNameIn,
  poolsARunOfThatNameCouldHold,
  poolsWithTheseLimits,
  poolTakesNoNewRun,
  virtualPoolNamed,
  virtualTextOf,
} from './pool-limits.ts';
export {
  NO_POOL_HAS_ROOM,
  type PoolTallies,
  type VirtualRoom,
  firstDefaultPoolWithRoom,
  poolHasRoom,
  poolsARunMayTake,
  poolsHeldBy,
  roomIn,
  summaryTextOf,
  talliesOf,
  virtualRoomOf,
  virtualSelectionHasRoom,
} from './pool-tallies.ts';
export {
  BUILD_VOLUME,
  BYTES_IN_A_GIBIBYTE,
  BYTES_IN_A_KIBIBYTE,
  type FreeBlocksOfAVolume,
  type FreeSpace,
  FREE_SPACE_FLOOR_BYTES,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceOnVolume,
  freeSpaceTextOf,
  wholeGibibytesOf,
} from './free-space-floor.ts';
export {
  type GoingRun,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
  goingRunsNow,
  launcherRunsIn,
  poolOfExecStart,
  unitRunNamesIn,
} from './going-runs.ts';
export {
  ALREADY_RUNNING_EXIT_STATUS,
  type GateOutcome,
  NO_SLOT_EXIT_STATUS,
  type PoolChoice,
  gateOutcomeOf,
  poolChoiceFor,
} from './pool-gate.ts';
export {
  CLOSED_POOL_EXIT_STATUS,
  ClosedPoolRefusalError,
  refuseARunOnAClosedPool,
} from './refusal-of-a-run-on-a-closed-pool.ts';
export {
  type PoolOfARunNamingNoProvider,
  poolOfARunNamingNoProvider,
  whyItTakesNoRun,
} from './pool-of-a-run-naming-no-provider.ts';
export {
  NoPoolTakesTheRunError,
  poolPlacementOfAWorktreeLaunch,
} from './pool-placement-of-a-worktree-launch.ts';
export type { WorktreeLaunchPlacement, } from './worktree-launch-placement.ts';
export {
  placeARunOnAVirtualSelection,
} from './place-a-run-on-a-virtual-selection.ts';
export {
  poolsWithTheCallersLimits,
  runsGoingForTheCount,
} from './pool-counts-before-a-launch.ts';
export {
  type LauncherArguments,
  launcherArgumentsOf,
  wallSecondsOn,
} from './launcher-arguments.ts';
