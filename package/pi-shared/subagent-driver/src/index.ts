/**
 Library surface of the Pi subagent driver. The bin is `drive-pi-subagent.ts`.

 @module
 */

export {
  ConnectionToPi,
  type ConnectionListeners,
  type ConnectionRequest,
} from './connection-to-pi.ts';
export { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
export type {
  DriverOptions,
  EndReason,
  PiExit,
  PiRecord,
  RunSummary,
} from './driver-types.ts';
export {
  DRIVER_USAGE,
  DriverOptionsError,
  parseDriverOptions,
} from './parse-driver-options.ts';
export {
  preview,
  RecordOfPiRun,
} from './record-of-pi-run.ts';
export {
  NestedDriverError,
  PiRunEndedEarlyError,
  supervisePiRun,
} from './supervise-pi-run.ts';
