/**
 Tests that serialize emits exactly one valid JSON line, colored or plain.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import { serializeSnapshot, } from './serialize.ts';
import type { EstimateSnapshot, } from './types.ts';

/**
 A representative snapshot with every optional field populated.
 */
const SNAPSHOT: EstimateSnapshot = {
  metric: 'm',
  scope: 's',
  basis: ['x',],
  pending: [],
  done: true,
  full: {
    confidence: 'low',
    point: { bytes: 100, human: '100 B', },
    lo: { bytes: 80, human: '80 B', },
    hi: { bytes: 200, human: '200 B', },
  },
  shallow: { bytes: 10, human: '10 B', },
  ratio: { point: 0.1, lo: 0.05, hi: 0.125, },
  savings: { point: 90, lo: 87.5, hi: 95, },
};

/**
 ESC byte marking an ANSI sequence.
 */
const ESC = String.fromCodePoint(27);

/**
 Removes `ESC ... m` ANSI sequences from a string via a linear scan.
 
 @param text - possibly-colored text
 
 @returns the text without ANSI escapes
 */
function stripAnsi(text: string): string {
  const cursor = { i: 0, };
  const out: string[] = [];
  while (cursor.i < text.length) {
    if (text[cursor.i] === ESC) {
      while ((cursor.i < text.length) && (text[cursor.i] !== 'm'))
        cursor.i += 1;
      cursor.i += 1;
      continue;
    }
    out.push(text[cursor.i] ?? '');
    cursor.i += 1;
  }
  return out.join('');
}

await describe({
  name: serializeSnapshot.name,
  children: [
    it({
      name: 'emits verbatim JSON with color off',
      fn: async ({ expect, }) => {
        const line = serializeSnapshot({ snapshot: SNAPSHOT, colorOn: false, });
        expect(line.includes(ESC)).toBe(false);
        expect(line).toBe(JSON.stringify(SNAPSHOT));
        expect(line.includes('\n')).toBe(false);
      },
    }),

    it({
      name: 'emits ANSI JSON with color on that parses back to the snapshot',
      fn: async ({ expect, }) => {
        const line = serializeSnapshot({ snapshot: SNAPSHOT, colorOn: true, });
        expect(line.includes(ESC)).toBe(true);
        expect(
          JSON.parse(stripAnsi(line)),
        ).toEqual(SNAPSHOT);
      },
    }),
  ],
});
