/**
 Tag a single parsed TOML scalar as a toml-test tagged value.
 
 Reads the kind-bearing `toml-eslint-parser` value node directly rather than
 the lossy `getStaticTOMLValue` projection, because the runner distinguishes
 the four datetime kinds and integers past 2^53 that the projection collapses.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import {
  KIND_TO_TAG,
  type TaggedValue,
} from './tagged-types.ts';

/**
 Render a finite-or-special float as the toml-test float payload.
 
 The runner parses the payload with Go's `ParseFloat` and compares
 numerically, so the non-finite spellings match TOML's `inf` / `-inf` / `nan`
 and finite values use the shortest round-tripping decimal.
 
 @param value - Parsed numeric value carried by the float node.
 
 @returns toml-test float payload string.
 
 @example
 ```ts
 floatToPayload(Number.POSITIVE_INFINITY); // 'inf'
 floatToPayload(3.5);                       // '3.5'
 ```
 */
function floatToPayload(value: number,): string {
  if (Number.isNaN(value,))
    return 'nan';
  if (value === Number.POSITIVE_INFINITY)
    return 'inf';
  if (value === Number.NEGATIVE_INFINITY)
    return '-inf';
  return String(value,);
}

/**
 Segment count of a colon-split time that has hours and minutes but no seconds.
 */
const TIME_SEGMENTS_WITHOUT_SECONDS = 2;

/**
 Length of a `YYYY-MM-DD` date plus its one `T`-or-space separator.
 */
const DATE_PREFIX_LENGTH = 11;

/**
 Ensure a `HH:MM[:SS]` time string carries seconds.
 
 TOML 1.1 makes seconds optional, but the toml-test runner compares times with
 an RFC 3339 layout that requires `HH:MM:SS`, so a seconds-less time is padded
 with `:00`. A time that already has seconds (and any fractional part) is left
 untouched.
 
 @param time - Time component, with or without seconds.
 
 @returns Time guaranteed to include a seconds field.
 
 @example
 ```ts
 withSeconds('13:37'); // '13:37:00'
 ```
 */
function withSeconds(time: string,): string {
  return (time.split(':',)
    .length
    === TIME_SEGMENTS_WITHOUT_SECONDS) ? `${time}:00` : time;
}

/**
 Normalize a local datetime / date / time spelling to a seconds-bearing form.
 
 Dates have no time component and pass through; times pad their seconds; local
 datetimes pad the seconds of the time that follows the date separator.
 
 @param kind - Local datetime kind of the node.
 
 @param datetime - Source spelling of the value.
 
 @returns Spelling whose time component, if any, includes seconds.
 
 @example
 ```ts
 localValueWithSeconds({ kind: 'local-time', datetime: '07:32' }); // '07:32:00'
 ```
 */
function localValueWithSeconds(
  {
    kind,
    datetime,
  }: {
    readonly kind: 'local-date-time' | 'local-date' | 'local-time';
    readonly datetime: string;
  },
): string {
  if (kind === 'local-date')
    return datetime;
  if (kind === 'local-time')
    return withSeconds(datetime,);
  return `${datetime.slice(
    0,
    DATE_PREFIX_LENGTH,
  )}${withSeconds(datetime.slice(DATE_PREFIX_LENGTH,),)}`;
}

/**
 Convert one parsed scalar node into a tagged value.
 
 Integers use the node's exact `bigint` so 64-bit values survive; offset
 datetimes normalize to RFC 3339 UTC via the parsed instant (the runner
 compares instants); local datetimes keep their source spelling but pad in a
 seconds field, which the runner's RFC 3339 wall-clock layout requires.
 
 @param node - Parsed scalar value node.
 
 @returns Tagged value mirroring the node's kind and payload.
 
 @throws Error when the node carries an unrecognized kind, so a future parser
         kind surfaces rather than being silently dropped.
 
 @example
 ```ts
 leafToTagged({ node, }); // { type: 'integer', value: '255' }
 ```
 */
export function leafToTagged({ node, }: { readonly node: AST.TOMLValue; },): TaggedValue {
  if (node.kind === 'string')
    return {
      type: KIND_TO_TAG[node.kind],
      value: node.value,
    };
  if (node.kind === 'integer')
    return {
      type: KIND_TO_TAG[node.kind],
      value: node.bigint
        .toString(),
    };
  if (node.kind === 'float')
    return {
      type: KIND_TO_TAG[node.kind],
      value: floatToPayload(node.value,),
    };
  if (node.kind === 'boolean')
    return {
      type: KIND_TO_TAG[node.kind],
      value: node.value ? 'true' : 'false',
    };
  if (node.kind === 'offset-date-time')
    return {
      type: KIND_TO_TAG[node.kind],
      value: node.value
        .toISOString(),
    };
  if ((node.kind === 'local-date-time') || (node.kind === 'local-date')
    || (node.kind === 'local-time'))
    return {
      type: KIND_TO_TAG[node.kind],
      value: localValueWithSeconds({
        kind: node.kind,
        datetime: node.datetime,
      },),
    };
  throw new Error(`Unhandled TOML value kind: ${String((node as { kind: unknown; }).kind,)}`,);
}
