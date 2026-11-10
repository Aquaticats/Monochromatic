/**
 Coercion of a feed-supplied publication timestamp into a `Date`.

 A feed's date field is external data: RFC 822 or RFC 3339 text, a Unix
 timestamp, or a `Date` the parser already understood. Text no calendar can
 read must not abort a page, so the coercion falls back to the epoch that both
 call sites already use for a missing date.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import * as v from 'valibot';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the publication-date module.
 */
const l = tagged({
  tag: 'publication-date',
  l: parentLogger,
},);

/**
 Coerces string, number, or `Date` publication timestamps into a `Date`,
 falling back to the epoch when the value names no instant.
 
 Logging the rejected raw value at debug level keeps the diagnostic without
 repeating it once per sort comparison.
 
 @example
 ```ts
 v.parse(coercePublicationDateSchema, 'Tue, 32 Jan 2025 10:00:00 GMT'); // 1970-01-01T00:00:00.000Z
 ```
 */
export const coercePublicationDateSchema: v.GenericSchema<string | number | Date, Date> = v.pipe(
  v.union([
    v.string(),
    v.number(),
    v.date(),
  ],),
  v.transform(function toPublicationDate(input,) {
    /**
     Parsed instant, invalid when the input names no instant.
     */
    const parsed = new Date(input,);
    if (Number.isNaN(parsed.getTime(),)) {
      l.debug(`unparseable publication date ${String(input,)}; using the epoch`,);
      return new Date(0,);
    }
    return parsed;
  },),
  v.date(),
);
