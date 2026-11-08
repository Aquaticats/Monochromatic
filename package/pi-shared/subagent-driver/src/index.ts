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
  FEED_PREVIEW_CHARACTERS,
  NESTED_RUN_VARIABLE,
  NO_RUN_FAILURE,
  POLL_MILLISECONDS,
  SHUTDOWN_GRACE_MILLISECONDS,
  SHUTDOWN_STAGES,
  TEARDOWN_ANSWER_MILLISECONDS,
  TEARDOWN_MILLISECONDS,
} from './driver-constants.ts';
export type {
  DriverOptions,
  EndReason,
  EventObservation,
  PiExit,
  PiRecord,
  RecordOfPiRun,
  RunEnding,
  RunFailure,
  RunProgress,
  RunSummary,
} from './driver-types.ts';
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
  finalEndReason,
} from './end-reasons.ts';
export {
  NestedDriverError,
  PiRunEndedEarlyError,
  supervisePiRun,
} from './supervise-pi-run.ts';
export {
  type WatchRequest,
  watchUntilEnded,
} from './watch-pi-run.ts';
