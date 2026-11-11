/**
 Library surface of the Pi subagent driver. The bin is `drive-pi-subagent.ts`.

 @module
 */

export {
  type CommandRequest,
  type ConnectionListeners,
  type ConnectionRequest,
  type ConnectionToPi,
  startConnectionToPi,
  type ShutDownOutcome,
  type ShutDownRequest,
} from './connection-to-pi.ts';
export {
  type CheckedRuns,
  defaultCheckedRunsFile,
  readCheckedRuns,
  writeCheckedRuns,
} from './checked-runs.ts';
export {
  COMMAND_ANSWER_MILLISECONDS,
  FEED_PREVIEW_CHARACTERS,
  MODEL_ERROR_RETRY_BASE_SECONDS,
  MODEL_ERROR_RETRY_CAP,
  MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
  NESTED_RUN_VARIABLE,
  NO_RUN_FAILURE,
  POLL_MILLISECONDS,
  SHUTDOWN_GRACE_MILLISECONDS,
  SHUTDOWN_STAGES,
  START_UP_ANSWER_MILLISECONDS,
  TEARDOWN_ANSWER_MILLISECONDS,
  TEARDOWN_MILLISECONDS,
} from './driver-constants.ts';
export type {
  CommandAnswer,
  CommandEnding,
  CompactionOfPi,
  DriverOptions,
  EndReason,
  EventObservation,
  PiExit,
  PiRecord,
  RecordOfPiRun,
  RetryTaken,
  RunEnding,
  RunFailure,
  RunProgress,
  RunSummary,
} from './driver-types.ts';
export {
  anotherAttemptMayFix,
  continuationPromptOf,
  isRateLimit,
  kindOfModelError,
  type ModelErrorKind,
  retryDelayMilliseconds,
  type WaitAskedByProvider,
  waitAskedByProvider,
} from './retries-of-a-model-error.ts';
export {
  DRIVER_USAGE,
  DriverOptionsError,
  parseDriverOptions,
} from './parse-driver-options.ts';
export {
  preview,
  textOfParts,
  thinkingLengthOf,
} from './previews-of-pi-messages.ts';
export {
  fieldsOf,
  isRecord,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';
export {
  type RecordOfPiRunRequest,
  startRecordOfPiRun,
} from './record-of-pi-run.ts';
export {
  CLEAN_END_REASONS,
  endReasonOfPromptWait,
  finalEndReason,
} from './end-reasons.ts';
export {
  carriesCredentialName,
  CREDENTIAL_NAME_WORDS,
  type EnvironmentWithoutCredentials,
  stripCredentialsFromEnvironment,
  type VariableKeptWithItsReason,
  VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME,
} from './environment-without-credentials.ts';
export {
  NestedDriverError,
  PiRunEndedEarlyError,
  supervisePiRun,
} from './supervise-pi-run.ts';
export { watchUntilEnded, } from './watch-pi-run.ts';
export type { WatchRequest, } from './watch-request.ts';
export {
  type StartWaitingRequest,
  startWaitingForAnAnswer,
  type WaitingForAnAnswer,
} from './wait-for-an-answer.ts';
export {
  type Ending,
  endReasonOfRun,
} from './take-the-first-prompt.ts';
export {
  putTheRunAway,
  type RunPutAway,
} from './put-the-run-away.ts';
export {
  answerModelError,
  type RetryWaiting,
} from './answer-a-model-error.ts';

/**
 What the watch for runs that should have come back reads, and the lines it answers with. The command is
 `watch-overdue-pi-runs.ts`.
 */
export {
  END_REASONS_WITHOUT_A_JOB,
  type FinishedPiRun,
  finishedPiRunsIn,
} from './finished-pi-runs.ts';
export {
  type OverdueLook,
  type OverdueLookRequest,
  lookAtPiRuns,
} from './look-at-pi-runs.ts';
export {
  LOOK_INTERVAL_MILLISECONDS,
  MILLISECONDS_PER_MINUTE,
  OPEN_CALL_PERCENTILE,
  OVERDUE_PERCENTILE,
  RECHECK_MILLISECONDS,
} from './overdue-constants.ts';
export {
  comparedAgainstTextOf,
  deadlineTextOf,
  limitsTextOf,
  minutesOf,
  percentTextOf,
} from './overdue-line-text.ts';
export {
  type FinishedRunEvidence,
  evidenceFromRunsFolder,
  NO_SAMPLE,
  percentileOf,
} from './overdue-limits.ts';
export {
  CHECK_IN_USAGE,
  type CheckInOptions,
  CheckInOptionsError,
  parseCheckInOptions,
} from './parse-check-in-options.ts';
export {
  type WatchOptions,
  WATCH_USAGE,
  WatchOptionsError,
  parseWatchOptions,
} from './parse-watch-options.ts';
export {
  parseProcessListing,
  type ProcessUnderARun,
  ProcessListingUnreadableError,
  processesUnder,
  startProcessTable,
} from './processes-under-a-pi-run.ts';
export {
  type CommandOutput,
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
export { latestRunFolderOf, } from './run-folder-of-a-unit.ts';
export {
  monotonicMicrosecondsNow,
  parseRunningPiUnits,
  PiUnitsUnreadableError,
  type RunningPiUnit,
  runNameOfUnit,
  startRunningPiUnits,
  unitNameOfGiven,
} from './running-pi-units.ts';
export {
  type OpenToolCall,
  type ToolCallTimes,
  toolCallTimesOfFeed,
} from './tool-call-times-of-a-feed.ts';
export {
  type OverdueWatchRequest,
  watchForOverdueRuns,
} from './watch-for-overdue-runs.ts';

/**
 The commands that read what is going on: the processes whose command line holds a text, the running units whose worktree a
 removal or a merge took away, and the final report of each agent's newest finished run.
 */
export {
  COMMAND_LINE_CHARACTERS,
  type CommandLineMatch,
  commandHolds,
  matchLineOf,
  processesWhoseCommandLineHolds,
} from './processes-whose-command-line-holds-text.ts';
export {
  type UnitWithoutItsWorktree,
  countLineOf,
  goneLineOf,
  serviceUnitsIn,
  unitsWhoseWorktreeIsGone,
} from './units-whose-worktree-is-gone.ts';
export {
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  type AttemptOfARun,
  attemptsOfARun,
  entriesOrNothingIn,
  isThere,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
export {
  REPORT_LINES,
  REPORT_MORE_LINES_SUFFIX,
  REPORT_WIDTH,
  type SummaryOfARun,
  noFinishedRunLineOf,
  reportLinesIn,
  reportLinesOf,
  summaryIn,
} from './report-of-a-run.ts';

/**
 A runs folder a test writes its own runs into, so no test reads the real one.

 @internal
 */
export {
  type DisposableRunsFolder,
  type RunFixture,
  standInRunsFolder,
} from './stand-in-runs-folder-for-tests.ts';

/**
 Exported so a test can hand the splitter the chunks it chooses, instead of hoping a scheduler splits a record.
 @internal
 */
export {
  startLineSplitter,
  type SplitPiOutputRequest,
} from './split-pi-output.ts';

/**
 The library surface of starting a run, kept in a file of its own so that neither list outgrows the line budget.
 */
export * from './index-of-starting-a-run.ts';
