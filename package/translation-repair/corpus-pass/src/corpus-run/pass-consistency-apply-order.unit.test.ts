/**
 Tests for putting the splices a check left out back in the order of the page, one pass at a time: every pass but the last
 puts back at least one splice, so the passes are at most one more than the splices left out.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConsistencyFloorVerdict,
  putBackInPageOrder,
  readmitLedgerOf,
} from '../../dist/final/node/index.mjs';
import {
  droppedRound,
  type SecondSliceSplice,
} from './consistency-readmit-page.test-fixture.ts';

//region Pass consistency apply order tests

/**
 The floors of a slice whose gains each pass only once the next one is made: a text that holds the gain of a number without the
 gain of the number after it is refused.

 @param candidateText - slice with its replacements made
 @returns Verdict
 @example
 ```ts
 const verdict = nextFirst({ candidateText: 'Mug 1 is here. Cup 2 is here.', },); // refused
 ```
 */
function nextFirst({ candidateText, }: { readonly candidateText: string; },): ConsistencyFloorVerdict {
  return [1, 2, 3,].every(function followedBy(number,): boolean {
    return (!candidateText.includes(`Mug ${String(number,)}`,))
      || candidateText.includes(`Mug ${String(number + 1,)}`,);
  },)
    ? { kind: 'passes', }
    : { kind: 'refused', findings: ['the next gain is not made',], };
}

/**
 The replacements of a slice of numbered cups, each taking a cup's name for a mug's.

 @param second - English of the slice, which names the cups
 @param count - how many cups the slice names, numbered from one
 @returns One replacement for each cup, in the order of the page
 @example
 ```ts
 const edits = cupEditsOf({ second: 'Cup 1 is here. Cup 2 is here.', count: 2, },);
 ```
 */
function cupEditsOf(
  {
    second,
    count,
  }: {
    readonly second: string;
    readonly count: number;
  },
): readonly SecondSliceSplice[] {
  return Array.from(
    { length: count, },
    function editOf(
      _slot,
      at,
    ): SecondSliceSplice {
      /**
       The cup's number.
       */
      const number = at + 1;
      /**
       Where the cup's name starts in the slice.
       */
      const start = second.indexOf(`Cup ${String(number,)}`,);
      return {
        quote: `Cup ${String(number,)}`,
        chosen: 'the parents’',
        start,
        end: start + 5,
        text: `Mug ${String(number,)}`,
      };
    },
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: putBackInPageOrder.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ENDS AFTER AT MOST ONE PASS MORE THAN THE GAINS LEFT OUT: in a slice of four gains each one the floors pass only '
            + 'once the next one is made, a pass puts back the last gain alone, the next pass the one before it, and so on, so '
            + 'four passes put back one gain each and a fifth puts back none',
          fn: async () => {
            const second = 'Cup 1 is here. Cup 2 is here. Cup 3 is here. Cup 4 is here.';
            const { page, round, } = droppedRound({
              second,
              edits: cupEditsOf({ second, count: 4, },),
              dropped: [1, 2, 3, 4,].map(function nameOf(number,): string {
                return `group ${String(number,)} OCCURRENCE 2`;
              },),
            },);
            const ledger = readmitLedgerOf({ page, round, refused: new Set(), },);
            const passes = putBackInPageOrder({
              ledger,
              asked: new Map(),
              floorsOn: nextFirst,
            },);
            expect(passes,).toBe(5,);
            expect(ledger.dropped.flat(),).toEqual([],);
          },
        },),
        it({
          name: 'ENDS AFTER TWO PASSES WHERE THE FLOORS PASS EVERY GAIN: the first pass puts both gains back and the second finds '
            + 'none left',
          fn: async () => {
            const second = 'Cup 1 is here. Cup 2 is here.';
            const { page, round, } = droppedRound({
              second,
              edits: cupEditsOf({ second, count: 2, },),
              dropped: ['group 1 OCCURRENCE 2', 'group 2 OCCURRENCE 2',],
            },);
            const ledger = readmitLedgerOf({ page, round, refused: new Set(), },);
            const passes = putBackInPageOrder({
              ledger,
              asked: new Map(),
              floorsOn: function passesAll(): ConsistencyFloorVerdict {
                return { kind: 'passes', };
              },
            },);
            expect(passes,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Pass consistency apply order tests
