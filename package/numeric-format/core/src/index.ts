/**
 Numeric formatters for the gaps left by `Intl.DurationFormat` and
 `Intl.NumberFormat` for developer-facing output.
 
 The package mirrors the shape of `@monochromatic-dev/const-core`
 (one file per category, re-exported from `index.ts`), and consumes its
 constants instead of redefining ratios locally.
 
 @example
 ```ts
 import {
   formatDuration,
 } from '@monochromatic-dev/numeric-format-core';
 ```
 
 @packageDocumentation
 */

//region byte

export { formatBytes, } from './byte.ts';

//endregion byte

//region duration

export {
  formatDuration,
  formatTrackedDuration,
} from './duration.ts';

//endregion duration
