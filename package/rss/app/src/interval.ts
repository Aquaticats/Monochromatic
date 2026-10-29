import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';
import * as v from 'valibot';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the interval module.
 */
const l = tagged({
  tag: 'interval',
  l: parentLogger,
},);

/**
 Minutes per fetch interval span.
 */
const MINUTES_PER_INTERVAL = 5;

/**
 Default fetch cache interval: 5 minutes in milliseconds.
 */
const DEFAULT_FETCH_INTERVAL_MS = MINUTES_PER_INTERVAL
  * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 Duration in milliseconds for which fetched feed data is cached.
 Controls the time bucket size for the fetch memoize salt.
 Override with the `RSS_FETCH_INTERVAL_MS` environment variable.
 
 @see {@link getFetchSalt} for how this controls cache invalidation
 */
export const FETCH_INTERVAL_MS: number = v.parse(
  v.pipe(
    v.unknown(),
    v.transform(Number,),
    v.number(),
  ),
  process.env
    .RSS_FETCH_INTERVAL_MS
    ?? DEFAULT_FETCH_INTERVAL_MS,
);

l.debug(`fetch interval: ${String(FETCH_INTERVAL_MS,)}ms`,);

/**
 Computes a time-bucketed salt for fetch memoization.
 Returns the same value within each interval window of length
 {@link FETCH_INTERVAL_MS}, causing fetch results to be reused until
 the window advances.
 
 @returns String representation of the current time bucket
 
 @example
 ```ts
 const salt = getFetchSalt();
 ```
 */
export function getFetchSalt(): string {
  return String(Math.floor(Date.now()
    / FETCH_INTERVAL_MS,),);
}
