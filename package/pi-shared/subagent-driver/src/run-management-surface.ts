/**
 What the run-management commands of this package offer, gathered in one module so the package's own surface stays readable.

 @module
 */

/**
 Where a run's folders, worktree and unit name come from, which every command of this package shares.
 */
export {
  defaultKimiReviewsFolder,
  defaultRepository,
  defaultRunsFolder,
  driverEntryOf,
  inboxOfARun,
  promptFileOf,
  runFolderNameOf,
  runFolderPathOf,
  worktreeOfARun,
} from './pi-runs-folders.ts';

/**
 The provider pools, their limits, and which pool a new run goes to.
 */
export {
  ALREADY_RUNNING_EXIT_STATUS,
  AUDIT_RUN_PREFIX,
  countsOfPools,
  countsSummaryOf,
  hasRoom,
  KIMI_RUN_PREFIX,
  NO_LAUNCHER,
  NO_POOL,
  NO_SLOT_EXIT_STATUS,
  PI_POOLS,
  POOL_OF_A_RUN_WITH_NO_LAUNCHER,
  type RunUnderAPool,
  runIsAlreadyGoing,
  type SlotPool,
  poolNamed,
  poolWithRoom,
  wallSecondsInPool,
} from './pools-of-pi-runs.ts';

/**
 The starts of runs in flight, which is half of what a pool holds.
 */
export {
  commandLinesUnderProcessFolder,
  commandNameOf,
  launcherIn,
  launchersOf,
  NOT_A_LAUNCHER,
  type ProcessCommandLine,
  readRunLaunchers,
  type RunLauncher,
  STARTER_COMMAND_NAME,
} from './run-launchers.ts';

/**
 How many turns a run is given, sized from what finished runs used.
 */
export {
  type FinishedRun,
  finishedRunsIn,
  kindOfRun,
  NoFinishedRunToSizeATurnCapError,
  type SizedTurnCap,
  sizedTurnCap,
  type TurnCapKind,
} from './turn-caps-from-finished-runs.ts';

/**
 Starting one run: its command line, its worktree, and its unit.
 */
export {
  NO_WORKTREE_BRANCH,
  parseStartPiRunOptions,
  START_USAGE,
  StartPiRunOptionsError,
  type StartPiRunOptions,
} from './start-pi-run-options.ts';
export {
  PiRunNotStartedError,
  startAPiRun,
  type SizeATurnCap,
  type StartAPiRunRequest,
  type StartOutcome,
} from './start-a-pi-run.ts';
export {
  type RunToTheEnd,
  runToTheEnd,
} from './run-command-to-the-end.ts';
export {
  type UnitEnvironment,
  unitEnvironmentOf,
  UnitEnvironmentUnreadableError,
  unitWordsOf,
} from './unit-of-a-pi-run.ts';

/**
 Waiting for a slot, and a batch of runs that each wait for one.
 */
export {
  type RetryAStartRequest,
  retryAStartUntilASlotFrees,
  type RetryOutcome,
  type StartAttempt,
} from './retry-a-start-until-a-slot-frees.ts';
export {
  BATCH_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  type BatchEntry,
  BatchListError,
  DEFAULT_BATCH_CPU_PERCENT,
  DEFAULT_BATCH_MEMORY,
  DEFAULT_BATCH_TASKS,
  DEFAULT_BATCH_WALL_SECONDS,
  parseBatchList,
  type StartAnEntry,
  startABatch,
  type StartedEntry,
} from './batch-of-pi-runs.ts';

/**
 What the runs folder says: each agent's newest report, and the note a coordinator drops into a run's inbox.
 */
export {
  latestFinishedRunOf,
  latestReportsOf,
  NO_FINISHED_RUN,
  parseReportOptions,
  type ReportOptions,
  ReportOptionsError,
  REPORTS_USAGE,
} from './final-reports-of-pi-runs.ts';
export {
  A_USABLE_NAME,
  noteNameProblemOf,
  notesToRuns,
  publishNote,
} from './note-to-running-pi-runs.ts';

/**
 Which runs serve the Git wrapper, and the stop file the others are asked to stop with.
 */
export {
  isWrapperWork,
  lineOf,
  RUNS_KEPT_DESPITE_THEIR_NAMES,
  type RunToSort,
  sortRunsForTheWrapper,
  STOP_MESSAGE,
  stopRunsNotOnTheWrapper,
  WRAPPER_BRANCH_PART,
  writeStopFile,
} from './wrapper-runs.ts';

/**
 A round of Kimi K3.1 Pro reviews, and the fallback that keeps one going when Synthetic rate-limits it.
 */
export {
  briefOf,
  DEFAULT_REVIEW_SOURCE,
  KIMI_FALLBACK_MODEL,
  KIMI_FALLBACK_PROVIDER,
  KIMI_REVIEW_MODEL,
  KIMI_REVIEW_PROVIDER,
  type KimiReview,
  KimiReviewListError,
  parseKimiReviewList,
} from './kimi-reviews.ts';
export {
  fallbackAfterARateLimit,
  isReviewed,
  NO_SUMMARY,
  type PreparedReview,
  prepareKimiReview,
  type ReviewFallback,
  ReviewNotPreparedError,
} from './prepare-a-kimi-review.ts';
export {
  type AttemptEnding,
  type PrepareAReview,
  type RoundOfReviewsRequest,
  type StartAReviewAttempt,
  startARoundOfKimiReviews,
  type SummaryTextOfARun,
} from './round-of-kimi-reviews.ts';

/**
 Reads whether text is a whole number, which a run folder's number and a process number are read with.
 */
export { isWholeNumberText, } from './whole-number-text.ts';
