/**
 Parsing utilities for task-depends item resolution.
 
 Provides timestamp parsing and item classification helpers
 used by the resolution pipeline.
 
 @module
 */

import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

//region Constants

/**
 Prefix that identifies a shell command item
 */
export const SH_PREFIX = 'sh:';

/**
 Sentinel returned by {@link parseTimestamp} when its input is not a parseable timestamp.
 
 A unique `Symbol` keeps the absent case out of the numeric domain: every real
 timestamp (including `Infinity`/`-Infinity`) is a `number`, so callers
 distinguish "unparseable" by identity rather than a nullish union.
 
 @example
 ```ts
 parseTimestamp('') === UNPARSEABLE_TIMESTAMP // true
 ```
 */
export const UNPARSEABLE_TIMESTAMP: unique symbol = Symbol('timestamp input cannot be parsed',);

/**
 Boundary for distinguishing unix seconds from milliseconds.
 
 Numbers \>= 1e12 are treated as milliseconds (dates after 2001 in ms).
 Numbers \< 1e12 are treated as seconds and multiplied by 1000.
 */
const SECONDS_MS_BOUNDARY = 1e12;

//endregion Constants

//region Item classification

/**
 Checks whether an item is a shell command (prefixed with `sh:`).
 
 @param item - Source or output item string
 
 @returns `true` when the item starts with `sh:`
 
 @example
 ```ts
 isShellCommand('sh:podman image exists foo') // true
 isShellCommand('src/*.ts') // false
 ```
 */
export function isShellCommand(item: string,): boolean {
  return item.startsWith(SH_PREFIX,);
}

/**
 Strips the `sh:` prefix from a shell command item.
 
 @param item - Shell command item with `sh:` prefix
 
 @returns Command string without prefix
 
 @example
 ```ts
 extractCommand('sh:podman image exists foo') // 'podman image exists foo'
 ```
 */
export function extractCommand(item: string,): string {
  return item.slice(SH_PREFIX.length,);
}

//endregion Item classification

//region Timestamp parsing

/**
 Parses a string as a timestamp.
 
 Supports unix epoch (seconds or milliseconds), ISO 8601 dates,
 and the sentinel strings `Infinity` and `-Infinity`.
 Numbers \>= 1e12 are treated as milliseconds; smaller numbers as seconds.
 
 @param value - Trimmed stdout from a shell command
 
 @returns Timestamp in milliseconds (possibly `Infinity` or `-Infinity`),
 or {@link UNPARSEABLE_TIMESTAMP} when not parseable
 
 @example
 ```ts
 parseTimestamp('1710000000') // 1710000000000 (seconds -> ms)
 parseTimestamp('Infinity') // Infinity
 parseTimestamp('-Infinity') // -Infinity
 parseTimestamp('') // UNPARSEABLE_TIMESTAMP
 ```
 */
export function parseTimestamp(value: string,): number | typeof UNPARSEABLE_TIMESTAMP {
  if (value === '')
    return UNPARSEABLE_TIMESTAMP;
  if (value === 'Infinity')
    return Infinity;
  if (value === '-Infinity')
    return -Infinity;

  /**
   Numeric coercion of the input; finite values are interpreted as a Unix timestamp before falling through to date parsing.
   */
  const num = Number(value,);
  if ((!Number.isNaN(num,)) && Number
    .isFinite(num,))
    return (num >= SECONDS_MS_BOUNDARY) ? num : (num * MS_PER_SECOND);

  /**
   Date coercion of the input; used as a fallback when the value is a date string rather than a numeric timestamp.
   */
  const date = new Date(value,);
  if (!Number.isNaN(date.getTime(),))
    return date.getTime();

  return UNPARSEABLE_TIMESTAMP;
}

//endregion Timestamp parsing
