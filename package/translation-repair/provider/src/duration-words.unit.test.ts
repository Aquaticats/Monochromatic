/**
 Tests for the duration a refusal states in words: hours, minutes and seconds, each named only where it is not zero.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { durationWordsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: durationWordsOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'STATES 301 000 MS AS 5 min 1 s and 48 000 ms as 48 s',
      fn: async () => {
        expect([
          durationWordsOf({ ms: 301_000, },),
          durationWordsOf({ ms: 48_000, },),
        ],).toEqual(['5 min 1 s', '48 s',],);
      },
    },),

    it({
      name: 'NAMES ALL THREE UNITS for an hour and more, and skips a zero unit between them',
      fn: async () => {
        expect([
          durationWordsOf({ ms: 3_661_000, },),
          durationWordsOf({ ms: 3_601_000, },),
          durationWordsOf({ ms: 82_800_000, },),
        ],).toEqual(['1 h 1 min 1 s', '1 h 1 s', '23 h',],);
      },
    },),

    it({
      name: 'ROUNDS UP TO A WHOLE SECOND so a wait is never stated shorter than it is, and states no time as 0 s',
      fn: async () => {
        expect([
          durationWordsOf({ ms: 1, },),
          durationWordsOf({ ms: 1_001, },),
          durationWordsOf({ ms: 0, },),
          durationWordsOf({ ms: -5, },),
        ],).toEqual(['1 s', '2 s', '0 s', '0 s',],);
      },
    },),
  ],
},);
