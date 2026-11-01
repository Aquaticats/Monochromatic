/**
 Tests for the rule that reads a log line's stamp back (ledger B73): only
 what `Date.prototype.toISOString` writes, which is how the logger stamps
 every line.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isIsoStampText, } from '../dist/final/node/index.mjs';
import {
  STAMPS_NOT_WRITTEN,
  WRITTEN_STAMP,
} from './iso-stamp-text.test-fixture.ts';

await describe({
  name: isIsoStampText.name,
  children: [
    it({
      name: 'TAKES EVERY STAMP toISOString WRITES, the six-digit form it writes past year 9999 included',
      fn: async () => {
        expect([
          WRITTEN_STAMP,
          new Date(0,).toISOString(),
          new Date(Date.UTC(
            10_000,
            0,
          ),).toISOString(),
        ].map(function taken(text,): boolean {
          return isIsoStampText({ text, },);
        },),).toEqual([
          true,
          true,
          true,
        ],);
      },
    },),
    it({
      name: 'REFUSES EVERY SPELLING THE LOGGER DOES NOT WRITE, those Date.parse reads as the written instant '
        + 'included, since a line carrying one did not come from the logger as written',
      fn: async () => {
        expect(STAMPS_NOT_WRITTEN.map(function taken(text,): boolean {
          return isIsoStampText({ text, },);
        },),).toEqual(STAMPS_NOT_WRITTEN.map(function refused(): boolean {
          return false;
        },),);
      },
    },),
  ],
},);
