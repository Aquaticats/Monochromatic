/**
 Tests for what a floor's refusal of a slice does to the check of the apostrophes the replacements carry in. The check reads
 the page as the replacements would make it, and a slice the floors refuse keeps its wording, so a closing mark that a
 refused splice took out of the page stands again and can pair with a mark carried in earlier on the page: the check and the
 floors decide together, on the edits that are really made.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { applyConsistency, } from '../../dist/final/node/index.mjs';
import {
  floorsFindingOfThird,
  ROUND_LEFT_OUT,
  roundApplied,
  THIRD_WITH_CLOSER,
  THIRD_WITHOUT_MARK,
} from './consistency-round-page.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applyConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES THE `’round` SPLICE OUT WHEN A LATER FLOOR REFUSAL PUTS A CLOSING MARK BACK AFTER IT: the place keeps its '
            + 'words (kept-current), the refused slice keeps its wording (kept-floor), and the page changes nowhere',
          fn: async () => {
            expect(roundApplied({ third: THIRD_WITH_CLOSER, declaredNames: ['parents',], },),).toEqual({
              outcomes: [['chosen', 'kept-current',], ['chosen', 'kept-floor',],],
              changed: [],
              findings: [ROUND_LEFT_OUT, floorsFindingOfThird(),],
            },);
          },
        },),
        it({
          name: 'MAKES THE `’round` SPLICE WHEN NO FLOOR REFUSES A SLICE, the control of the case where a refusal puts a closer '
            + 'back',
          fn: async () => {
            expect(roundApplied({ third: THIRD_WITH_CLOSER, declaredNames: [], },),).toEqual({
              outcomes: [['chosen', 'replaced',], ['chosen', 'replaced',],],
              changed: [[1, 'We went ’round the yard.',], [2, 'Fill the folks bowl now.',],],
              findings: [],
            },);
          },
        },),
        it({
          name: 'MAKES THE `’round` SPLICE WHEN A FLOOR REFUSES A SLICE THAT HOLDS NO MARK a refusal could put back, and keeps '
            + 'the refused slice as it was',
          fn: async () => {
            expect(roundApplied({ third: THIRD_WITHOUT_MARK, declaredNames: ['family',], },),).toMatchObject({
              outcomes: [['chosen', 'replaced',], ['chosen', 'kept-floor',],],
              changed: [[1, 'We went ’round the yard.',],],
            },);
          },
        },),
      ],
    },),
  ],
},);
