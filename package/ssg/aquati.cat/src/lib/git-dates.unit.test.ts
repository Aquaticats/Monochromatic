/**
 Tests for reading a timestamp's UTC author offset.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { authorOffsetMinutes, } from '../../dist/final/node/index.mjs';

/**
 Timestamp/offset pairs covering every shape the parser has to tell apart.
 */
const OFFSET_CASES = [
  {
    timestamp: '2026-05-14T06:00:00+08:00',
    expected: 480,
    reason: 'an eastern offset written by git log --format=%aI',
  },
  {
    timestamp: '2026-05-14T06:00:00-05:00',
    expected: -300,
    reason: 'a western offset',
  },
  {
    timestamp: '2026-05-14T06:00:00+00:00',
    expected: 0,
    reason: 'UTC written as a zero offset',
  },
  {
    timestamp: '2026-05-14T06:00:00Z',
    expected: 0,
    reason: 'UTC written with the Z designator, as the GitHub API does',
  },
  {
    timestamp: '2026-05-14T06:00:00',
    expected: 0,
    reason: 'a timestamp carrying no offset at all',
  },
  {
    timestamp: '2026-05-14T06:00:00+05:30',
    expected: 330,
    reason: 'a half-hour offset',
  },
  {
    timestamp: '2026-05-14T06:00:00+8:00',
    expected: 0,
    reason: 'a malformed offset whose hour field is not two digits',
  },
] as const;

await describe({
  name: authorOffsetMinutes.name,
  children: OFFSET_CASES.map(function offsetCase({ timestamp, expected, reason, },) {
    return it({
      name: `reads ${String(expected,)} minutes from ${timestamp} (${reason})`,
      fn: async function readsOffset(): Promise<void> {
        expect(authorOffsetMinutes(timestamp,),).toBe(expected,);
      },
    },);
  },),
},);
