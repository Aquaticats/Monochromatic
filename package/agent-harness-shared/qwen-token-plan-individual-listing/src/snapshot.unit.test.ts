import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  qwenTokenPlanIndividualFacts,
  qwenTokenPlanIndividualListingSnapshot,
} from '../dist/final/node/index.mjs';

/**
 Digit characters the timestamp shape check accepts.
 */
const DIGIT_CHARACTERS = '0123456789';

/**
 Whether one text is a second-precise ISO 8601 timestamp with an offset:
 `YYYY-MM-DDThh:mm:ssZ` or `YYYY-MM-DDThh:mm:ss±hh:mm`, the shape the snapshot
 header carries. Character scan, since the repo lints regular expressions out
 of source.
 */
function isSecondPreciseIsoTimestamp(text: string): boolean {
  function digitsAt(position: number, count: number): boolean {
    for (let offset = 0; offset < count; offset += 1) {
      if (!DIGIT_CHARACTERS.includes(text.charAt(position + offset))) {
        return false;
      }
    }
    return true;
  }
  if ((text.length !== 20) && (text.length !== 25)) {
    return false;
  }
  if (!digitsAt(0, 4)) {
    return false;
  }
  if (text.charAt(4) !== '-') {
    return false;
  }
  if (!digitsAt(5, 2)) {
    return false;
  }
  if (text.charAt(7) !== '-') {
    return false;
  }
  if (!digitsAt(8, 2)) {
    return false;
  }
  if (text.charAt(10) !== 'T') {
    return false;
  }
  if (!digitsAt(11, 2)) {
    return false;
  }
  if (text.charAt(13) !== ':') {
    return false;
  }
  if (!digitsAt(14, 2)) {
    return false;
  }
  if (text.charAt(16) !== ':') {
    return false;
  }
  if (!digitsAt(17, 2)) {
    return false;
  }
  if (text.length === 20) {
    return text.charAt(19) === 'Z';
  }
  if ((text.charAt(19) !== '+') && (text.charAt(19) !== '-')) {
    return false;
  }
  return digitsAt(20, 2) && (text.charAt(22) === ':') && digitsAt(23, 2);
}

await describe({
  name: 'listing membership snapshot',
  children: [
    it({
      name: 'stamps the header with a second-precise ISO 8601 fetch time',
      fn: async () => {
        expect(isSecondPreciseIsoTimestamp(
          qwenTokenPlanIndividualListingSnapshot.fetchedAt,
        )).toBe(true);
      },
    }),
    it({
      name: 'carries one entry per id with an integer created stamp',
      fn: async () => {
        for (const entry of qwenTokenPlanIndividualListingSnapshot.entries) {
          expect(entry.id.length).toBeGreaterThan(0);
          if (entry.created !== undefined) {
            expect(Number.isInteger(entry.created)).toBe(true);
          }
        }
      },
    }),
    it({
      name: 'lists every id once',
      fn: async () => {
        const ids = qwenTokenPlanIndividualListingSnapshot.entries.map(
          function toId(entry) {
            return entry.id;
          },
        );
        expect(new Set(ids).size).toBe(ids.length);
      },
    }),
    it({
      name: 'covers every facts row id the curated table carries',
      fn: async () => {
        const snapshotIds = new Set(
          qwenTokenPlanIndividualListingSnapshot.entries.map(
            function toId(entry) {
              return entry.id;
            },
          ),
        );
        for (const row of qwenTokenPlanIndividualFacts) {
          expect(snapshotIds.has(row.id)).toBe(true);
        }
      },
    }),
  ],
});
