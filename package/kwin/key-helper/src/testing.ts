/**
 Built entry the unit tests import.

 @module
 */

/**
 Combo-to-evdev translation.

 @internal
 */
export { keysToEvdev, } from './keys.ts';

/**
 The daemon's error classes.

 @internal
 */
export {
  NoAppIdentityError,
  UnknownKeyError,
} from './errors.ts';

/**
 The pure double-shift detector and sysfs bitmap parser.

 @internal
 */
export {
  INITIAL_SHIFT_STATE,
  parseBitmap,
  reduceShiftEvent,
  type ShiftEvent,
  type ShiftKey,
  type ShiftResult,
  type ShiftState,
} from './evdev-parse.ts';

/**
 New-instance command selection.

 @internal
 */
export { selectLaunchCommand, } from './launch.ts';
