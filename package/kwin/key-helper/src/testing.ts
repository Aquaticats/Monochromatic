/**
 Built entry the unit tests import.

 The test harness loads the artifact this package builds rather than sibling
 source, as `test-import/require-eventual-artifact` requires, so this module
 re-exports exactly the behavior the tests drive. The daemon entry stays out:
 importing `src/index.ts` claims the D-Bus name and opens the real
 `/dev/input` devices, and a test must do neither.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`,
 beside the daemon bundle.

 @module
 */

import { logger, } from '@monochromatic-dev/logger-core/ts';

/**
 Drain the daemon's pending log writes, so a test can assert what was reported.

 The logger hands each record to its sinks without waiting, so a case that reads
 `process.stderr.write` must let those writes finish first.

 @internal
 */
export async function flushLogs(): Promise<void> {
  await logger.flush();
}

/**
 Running one command to completion, the daemon's single child-process shape.

 @internal
 */
export {
  runCommand,
  type RunCommandOptions,
} from './child-process.ts';

/**
 The daemon's error classes, so tests can assert by type.

 @internal
 */
export {
  CommandFailedError,
  KeyHelperError,
  NoAppIdentityError,
  NoNvimSocketError,
  NoRuntimeDirectoryError,
  UnknownKeyError,
} from './errors.ts';

/**
 Combo-to-evdev translation and its ydotool runner.

 @internal
 */
export {
  EVDEV_KEYS,
  keysToEvdev,
  sendKeys,
} from './keys.ts';

/**
 Neovim listen-socket discovery and the msgpack-rpc input sender.

 @internal
 */
export {
  findNewestNvimSocket,
  nvimSocketDirectory,
  sendNvimInput,
} from './nvim.ts';

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
 New-instance command selection and its launcher.

 @internal
 */
export {
  launchNewInstance,
  NEW_WINDOW_OVERRIDES,
  selectLaunchCommand,
  type LaunchCommand,
} from './launch.ts';
