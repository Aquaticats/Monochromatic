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
} from './connection-to-pi.ts';
export {
  NESTED_RUN_VARIABLE,
  POLL_MILLISECONDS,
} from './driver-constants.ts';
export type {
  DriverOptions,
  EndReason,
  PiExit,
  PiRecord,
  RunProgress,
  RunSummary,
} from './driver-types.ts';
export {
  DRIVER_USAGE,
  DriverOptionsError,
  parseDriverOptions,
} from './parse-driver-options.ts';
export {
  fieldsOf,
  isRecord,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';
export {
  type EventObservation,
  preview,
  type RecordOfPiRun,
  type RunEnding,
  startRecordOfPiRun,
} from './record-of-pi-run.ts';
export {
  NestedDriverError,
  PiRunEndedEarlyError,
  supervisePiRun,
} from './supervise-pi-run.ts';
export {
  type WatchRequest,
  watchUntilEnded,
  WatchEndedWithoutReasonError,
} from './watch-pi-run.ts';
