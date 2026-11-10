/**
 Built entry the unit tests import.

 The package's only entry is the `android-exempt-unused` bin, which drives an
 interactive prompt when it loads, so no test can import it. This module re-exports
 exactly the behavior the unit suites drive, so they load what this package builds
 rather than sibling source, as `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the bin bundle. Nothing in this package imports it, and the CLI never loads it.

 @module
 */

//region adb output parsing

/**
 The pure parsers for `adb` output, and the package-name grammar they enforce.

 @internal
 */
export {
  isValidPackageName,
  parseDevices,
  parseExemptedQuery,
  parsePackageList,
  type Device,
} from './parse.ts';

//endregion adb output parsing

//region Selection diff

/**
 Current-versus-selected diff, the one step between the multiselect and the apply.

 @internal
 */
export {
  computeChanges,
  type Changes,
} from './changes.ts';

//endregion Selection diff

//region Failures

/**
 The error a failed `adb` invocation raises, so tests assert by type.

 @internal
 */
export { AdbCommandError, } from './errors.ts';

//endregion Failures
