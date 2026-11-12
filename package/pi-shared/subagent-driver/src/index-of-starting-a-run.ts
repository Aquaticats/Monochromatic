/**
 The library surface of the commands that start a run: the provider pools, the worktree, the unit, the batch and the retrying
 start. `index.ts` re-exports it.

 @module
 */

/**
 Starting a run: the provider pools it shares, the worktree it works in, the unit it runs as, the batch that starts several, and
 the retrying start that waits for a slot. The commands are `start-pi-run-when-a-pool-has-room.ts`,
 `start-pi-run-in-a-worktree.ts`, `retry-pi-run-start-until-a-pool-has-room.ts` and `start-pi-runs-from-a-batch-list.ts`.
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
  type PoolMembership,
  PoolLimitsError,
  type ProviderPool,
  limitsIn,
  membershipOf,
  poolNamed,
  poolsARunOfThatNameCouldHold,
  poolsWithTheseLimits,
  poolTakesNoNewRun,
} from './pool-limits.ts';
export {
  NO_POOL_HAS_ROOM,
  type PoolTallies,
  firstDefaultPoolWithRoom,
  poolHasRoom,
  poolsHeldBy,
  roomIn,
  summaryTextOf,
  talliesOf,
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
  type LauncherArguments,
  launcherArgumentsOf,
  wallSecondsOn,
} from './launcher-arguments.ts';
export {
  COUNTS_ONLY_OPTION,
  MODEL_OPTION,
  NAME_OPTION,
  POOL_GATE_USAGE,
  PROVIDER_OPTION,
  type PoolGateOptions,
  PoolGateOptionsError,
  RUNS_FOLDER_OPTION,
  SESSION_OF_OPTION,
  WALL_OPTION,
  parsePoolGateOptions,
} from './parse-pool-gate-options.ts';
export {
  IDLE_SECONDS_OF_A_RUN,
  NO_WORKTREE_BRANCH,
  type WorktreeLauncherOptions,
  WORKTREE_LAUNCHER_USAGE,
  WorktreeLauncherOptionsError,
  parseWorktreeLauncherOptions,
  worktreeFolderOf,
} from './parse-worktree-launcher-options.ts';
export {
  SessionFileUnreadableError,
  sessionFileInSummary,
  sessionFileOfRun,
} from './session-file-of-a-run.ts';
export { nextRunNumber, } from './next-run-number.ts';
export {
  CHECKOUT_MARKER,
  CheckoutNotFoundError,
  checkoutFolderOf,
} from './checkout-of-a-command.ts';
export {
  PI_LOAD_FAILURE_TEXT,
  PI_START_CHECK_MEMORY_SECONDS,
  type PiStartAnswer,
  type PiStartCheckOutcome,
  type PiStartCheckRecord,
  type StartChecks,
  StartRefusedError,
  answersIn,
  baseProblemIn,
  piProblemIn,
  piStartCheckFileIn,
  nameProblemIn,
  promptProblemIn,
  startRefusalsIn,
} from './checks-before-a-start.ts';
export {
  ENTRY_REASON_CHARACTERS,
  reasonInFailedStart,
} from './reason-in-a-failed-start.ts';
export {
  PI_STANDARD_ERROR_TAIL_BYTES,
  PI_STANDARD_ERROR_TAIL_LINES,
  type PiStandardErrorTail,
  lastOfPiStandardError,
} from './last-of-pi-standard-error.ts';
export {
  type FinishedRunTurns,
  NO_FINISHED_RUN,
  RUNS_OF_ONE_KIND_TO_TRUST,
  TURN_CAP_MARGIN,
  type TurnCapSizing,
  finishedTurnsIn,
  kindOfRun,
  turnCapSizing,
} from './turn-cap-of-finished-runs.ts';
export {
  UNIT_GRACE_SECONDS,
  UNIT_IO_WEIGHT,
  type UnitCommandRequest,
  runtimeMaxSecondsOf,
  unitCommandOf,
} from './unit-command-of-a-run.ts';
export {
  BASE_OF_A_RUN_NAMING_NONE,
  type GitRunner,
  GitCommandFailedError,
  type WorktreeOutcome,
  WorktreeFolderOccupiedError,
  branchExists,
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  makeWorktree,
  removeWorktree,
  worktreeOutcomeOf,
} from './worktree-of-a-run.ts';
export {
  GATE_OUTPUT_TAIL_CHARACTERS,
  type RetriedStart,
  retryStartUntilAPoolHasRoom,
} from './retry-start.ts';
export {
  RETRY_START_USAGE,
  type RetryStartOptions,
  RetryStartOptionsError,
  parseRetryStartOptions,
} from './parse-retry-start-options.ts';
export {
  BATCH_DEFAULT_CPU_PERCENT,
  BATCH_DEFAULT_DEADLINE_SECONDS,
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_TASK_COUNT,
  BATCH_DEFAULT_WALL_SECONDS,
  BATCH_ENTRY_FIELDS,
  BATCH_RETRY_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  BatchListError,
  type BatchEntry,
  ENTRY_LINE_CHARACTERS,
  ENTRY_OUTPUT_KEPT_CHARACTERS,
  type StartedBatch,
  type StartedEntry,
  batchEntriesIn,
  entryLineOf,
  promptFileOf,
  retryWordsOf,
  startBatch,
} from './batch-list.ts';
export {
  BATCH_USAGE,
  type BatchOptions,
  BatchOptionsError,
  parseBatchOptions,
} from './parse-batch-options.ts';
export {
  type ShownCommandEnding,
  runCommandShowingItsOutput,
} from './run-command.ts';
export {
  CommandAnswerMissingError,
  requiredOutputOf,
} from './required-output-of-a-command.ts';
export {
  GIT_ENVIRONMENT,
  GIT_TIMEOUT_SECONDS,
  gitRunnerOf,
} from './git-command.ts';
export {
  GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS,
  GitWrapperPreflightError,
  WRAPPER_COMMAND,
  refuseUnlessTheWrapperAnswersGit,
} from './git-wrapper-preflight.ts';
export {
  NO_SUCH_PROGRAM,
  programOnAPath,
} from './program-on-a-path.ts';
export { siblingCommandEntry, } from './sibling-command-entry.ts';
export {
  type TailKeeper,
  tailKeeper,
} from './tail-of-a-command.ts';
export {
  firstWordOf,
  isDigitsOnly,
  valueOfOption,
} from './text-of-an-option.ts';
export { optionIn, } from './option-in.ts';

/**
 A review whose run wrote no review file, and the command that saves its last message:
 `save-the-final-message-of-a-review-that-wrote-no-review-file.ts`.
 */
export {
  SAVED_REVIEW_NOTE,
  type SavedReview,
  lastAssistantTextIn,
  newestRunOf,
  saveTheLastMessage,
} from './last-message-of-a-review.ts';
export {
  SAVE_REVIEW_USAGE,
  SaveReviewOptionsError,
  parseSaveReviewOptions,
} from './parse-save-review-options.ts';
export {
  NOT_RUNNING_LOOKS_TO_END,
  type UnitWaitOutcome,
  UNIT_STATE_INTERVAL_MILLISECONDS,
  UNIT_WAIT_GRACE_SECONDS,
  unitStateOf,
  waitForUnitToEnd,
} from './wait-for-a-unit-to-end.ts';
