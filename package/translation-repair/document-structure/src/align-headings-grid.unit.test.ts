/**
 Tests for where the heading-affinity grid draws its trust line.

 WHY THIS FILE EXISTS. It is known what a wrong section pairing costs: on
 one entry every critic call compared the wrong original against the wrong
 translation, so every issue filed was noise and every repair damaged correct
 text. The grid is what stops that, and the type's own words are "pairings AT
 OR ABOVE threshold". On 2026-08-25 moving the comparison so that a pairing
 landing exactly ON the threshold became untrusted failed no test here.

 HALF IS NOT A CORNER CASE. Affinity is shared Latin runs over the smaller
 heading's run count, so a Chinese heading carrying a romanised name beside
 one other run, matched against an English heading carrying that name beside
 one other, scores exactly one half. That is the ordinary shape of a memorial
 page's section heading, not a constructed edge.

 BOTH SIDES OF THE LINE ARE PINNED, since a grid trusting everything would
 satisfy the first case alone.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { headingAffinity, } from '@monochromatic-dev/translation-repair-markup';
import { buildGrid, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Original-side heading carrying two Latin runs, one of which the English side
 also carries.
 */
const SOURCE_TWO_RUNS = '### 其七：mittens suki';

/**
 Translation-side heading carrying two runs, sharing exactly one.
 */
const TARGET_TWO_RUNS = '### mittens ann';

/**
 Original-side heading carrying three runs.
 */
const SOURCE_THREE_RUNS = '### 其八：mittens suki whiskers';

/**
 Translation-side heading carrying three runs and sharing one, so the smaller
 side has three and the pairing lands below the line rather than on it.
 */
const TARGET_THREE_RUNS = '### mittens annie tabby';

//endregion Fixtures

await describe({
  name: buildGrid.name,
  children: [
    it({
      name: 'TRUSTS a pairing landing exactly ON the threshold, which the type calls at or above, and '
        + 'which is the ordinary shape of a heading carrying one romanised name beside one other run',
      fn: async () => {
        /**
         Score of the only pairing, stated by the shipped scorer rather than
         assumed, so a fixture that stopped scoring one half would be reported
         as a broken fixture rather than passing as a clean null.
         */
        const score = headingAffinity({
          source: SOURCE_TWO_RUNS,
          target: TARGET_TWO_RUNS,
        },);

        expect(score,).toBe(1 / 2,);

        const grid = buildGrid({
          sourceHeadings: [SOURCE_TWO_RUNS,],
          targetHeadings: [TARGET_TWO_RUNS,],
        },);

        expect(grid.affinity[0]?.[0],).toBe(1 / 2,);
        expect(grid.trusted[0]?.[0],).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a pairing below the threshold, which is the control that makes the line above a '
        + 'line: a grid trusting every scored pairing would anchor a name repeated across headings',
      fn: async () => {
        /**
         Score of a pairing sharing one run of three, which sits under the
         line rather than on it.
         */
        const score = headingAffinity({
          source: SOURCE_THREE_RUNS,
          target: TARGET_THREE_RUNS,
        },);

        expect(score,).toBe(1 / 3,);

        const grid = buildGrid({
          sourceHeadings: [SOURCE_THREE_RUNS,],
          targetHeadings: [TARGET_THREE_RUNS,],
        },);

        expect(grid.affinity[0]?.[0],).toBe(1 / 3,);
        expect(grid.trusted[0]?.[0],).toBe(false,);
      },
    },),
  ],
},);
