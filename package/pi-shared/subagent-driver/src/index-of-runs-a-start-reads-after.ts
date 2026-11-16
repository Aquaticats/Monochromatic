/**
 What a start reads after a run has ended, or while a unit is becoming active: the finished runs a turn cap is sized from, the lines
 about a unit's state, and what a start reports. Re-exported from `index-of-starting-a-run.ts`, so the package's surface is unchanged.

 @module
 */

export {
  type EndedRun,
  endedRunIn,
  endedRunLineOf,
  endedRunsSince,
  timeIn,
} from './runs-that-ended-since.ts';
export {
  type UnitWaitForActive,
  ACTIVATING_STATE,
  waitedLineOf,
  waitForUnitToBecomeActive,
} from './waiting-for-a-unit-to-become-active.ts';
export { SUMMARY_FILE_NAME, } from './runs-that-ended-since.ts';
