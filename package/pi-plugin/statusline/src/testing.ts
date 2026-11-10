/**
 Built entry the unit tests import.

 The package entry registers this pi extension's event handlers when it loads, so a
 test cannot import it. This module re-exports the rate-limit parsing, the types it
 produces, and the warning formatting the suites drive, so they load what this
 package builds rather than sibling source, as `test-import/require-eventual-artifact`
 requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the extension bundle. Nothing in this package imports it, and pi never loads it.

 @module
 */

//region Rate-limit headers

/**
 Provider header sets to usage snapshots, and the shape they produce.

 @internal
 */
export { parseRateLimitSnapshots, } from './rate-limit-headers.ts';
export {
  PLAIN_USAGE_WARNING_STYLE,
  RATE_LIMIT_WINDOW_SECONDS,
  type RateLimitSnapshot,
  type UsageWarningStyle,
} from './rate-limit-types.ts';

//endregion Rate-limit headers

//region Warning formatting

/**
 The statusline's usage-warning segment: its bands, projection, and marker.

 @internal
 */
export {
  formatRelativeTime,
  formatUsageWarningSegment,
  formatUsageWarningStatus,
  projectUsagePercent,
} from './usage-warning.ts';

//endregion Warning formatting
