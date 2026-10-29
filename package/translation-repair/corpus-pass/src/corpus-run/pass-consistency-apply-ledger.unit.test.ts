/**
 Tests for what the step that puts dropped splices back knows of the page while it works: the slices that carry English
 by their place, the plan, the replacements that stand made, the splices left out, and the small helpers that name and
 replace entries.

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
  gainSliceOf,
  idsOf,
  keyOf,
  NO_POSITION,
  pageFrameOf,
  positionOfSlice,
  replacedAt,
  readmitLedgerOf,
} from '../../dist/final/node/index.mjs';
import { droppedRound, } from './consistency-readmit-page.test-fixture.ts';

/**
 The name of the one splice the pages of these cases replace.
 */
const SPLICE = 'group 1 OCCURRENCE 2';

/**
 The one splice of the second slice, putting a trailing apostrophe on the family.
 */
const FAMILY = { quote: 'the family', chosen: 'the parents’', start: 5, end: 15, text: 'the parents’', };

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readmitLedgerOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE PAGE BY THE PLACE OF EACH SLICE THAT CARRIES ENGLISH: the plan, what stands made, the splice left out, '
            + 'and a blank line between the slices where the facts carry no page',
          fn: async () => {
            const { page, round, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [SPLICE,], },);
            const ledger = readmitLedgerOf({ page, round, refused: new Set(), },);
            expect({
              between: ledger.between,
              plans: [...ledger.plans.keys(),],
              refused: [...ledger.refused,],
              made: ledger.made.map(function textOf({ text, edits, },): unknown {
                return [text, edits.length,];
              },),
              toMake: ledger.toMake.map(function namesOf(edits,): unknown {
                return idsOf({ edits, },).size;
              },),
              dropped: ledger.dropped.map(function namesOf(edits,): readonly string[] {
                return [...idsOf({ edits, },),];
              },),
              verdicts: ledger.verdictLog.length,
              blocked: ledger.blocked.size,
              reasons: ledger.reasonLog.length,
            },).toEqual({
              between: ['', '\n\n', '',],
              plans: [1,],
              refused: [],
              made: [['Fill the parents’ dish and the pups’ bed.', 0,], ['Fill the family bowl.', 0,],],
              toMake: [0, 0,],
              dropped: [[], [SPLICE,],],
              verdicts: 0,
              blocked: 0,
              reasons: 0,
            },);
          },
        },),
        it({
          name: 'HOLDS THE TEXT THE FACTS CARRY BETWEEN THE SLICES, and the verdict the floors gave a slice that was put through them',
          fn: async () => {
            const { page, round, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [], },);
            const [slice,] = round;
            if (slice === undefined)
              throw new Error('the round has no slice',);
            const ledger = readmitLedgerOf({
              page: { ...page, between: ['<!-- a -->', '\n\n', '\n',], },
              round: [{ ...slice, verdict: { kind: 'passes', }, },],
              refused: new Set(),
            },);
            expect({
              between: ledger.between,
              made: ledger.made.map(function textOf({ text, },): string {
                return text;
              },),
              verdicts: ledger.verdictLog.map(function pairOf({ position, verdict, },): unknown {
                return [position, verdict,];
              },),
            },).toEqual({
              between: ['<!-- a -->', '\n\n', '\n',],
              made: ['Fill the parents’ dish and the pups’ bed.', 'Fill the parents’ bowl.',],
              verdicts: [[1, { kind: 'passes', },],],
            },);
          },
        },),
        it({
          name: 'READS A SLICE THE FLOORS REFUSED AS IT STANDS, whole: the page holds none of its replacements',
          fn: async () => {
            const { page, round, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [], },);
            const ledger = readmitLedgerOf({ page, round, refused: new Set([1,]), },);
            expect({
              refused: [...ledger.refused,],
              made: ledger.made.map(function textOf({ text, },): string {
                return text;
              },),
            },).toEqual({
              refused: [1,],
              made: ['Fill the parents’ dish and the pups’ bed.', 'Fill the family bowl.',],
            },);
          },
        },),
      ],
    },),
    describe({
      name: pageFrameOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE TEXT AROUND THE SLICES AND THE INDEX EACH SLICE IS NAMED BY, and the layout of the published page when '
            + 'the facts carry it, which readmitLedgerOf keeps from the page it was given',
          fn: async () => {
            const { page, round, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [], },);
            /**
             What the published page is assembled from, whose content the frame only hands on.
             */
            const layout = { targetText: 'x', chunks: [], silent: [], };
            const plain = readmitLedgerOf({ page, round, refused: new Set(), },);
            const laidOut = readmitLedgerOf({ page: { ...page, layout, }, round, refused: new Set(), },);
            expect([
              pageFrameOf({ ledger: plain, },),
              pageFrameOf({ ledger: laidOut, },),
            ],).toEqual([
              { between: ['', '\n\n', '',], sliceIndexes: plain.worked.map(function indexOfSlice({ facts, },): number {
                return facts.sliceIndex;
              },), },
              { between: ['', '\n\n', '',], layout, sliceIndexes: plain.worked.map(function indexOfSlice({ facts, },): number {
                return facts.sliceIndex;
              },), },
            ],);
            expect(plain.layout,).toBeUndefined();
            expect(laidOut.layout,).toBe(layout,);
          },
        },),
      ],
    },),
    describe({
      name: positionOfSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE PLACE OF A SLICE THAT CARRIES ENGLISH, and NO_POSITION for one that does not',
          fn: async () => {
            const { page, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [], },);
            const { slices: worked, } = page;
            expect([
              positionOfSlice({ worked, sliceIndex: 0, },),
              positionOfSlice({ worked, sliceIndex: 1, },),
              positionOfSlice({ worked, sliceIndex: 5, },),
            ],).toEqual([0, 1, NO_POSITION,],);
            expect(NO_POSITION,).toBe(-1,);
          },
        },),
      ],
    },),
    describe({
      name: gainSliceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A SLICE WITH THE REPLACEMENTS THAT WERE NOT LEFT OUT FOR ANOTHER GROUP\'S WORDS, and which of them are splices',
          fn: async () => {
            const { page, } = droppedRound({ second: 'Fill the family bowl.', edits: [FAMILY,], dropped: [], },);
            const [, worked,] = page.slices;
            if (worked === undefined)
              throw new Error('the page has no second slice',);
            const { sliceIndex, english, edits, splices, } = gainSliceOf({ worked, },);
            expect({
              sliceIndex,
              english,
              edits: [...idsOf({ edits, },),],
              splices: [...splices,],
            },).toEqual({
              sliceIndex: 1,
              english: 'Fill the family bowl.',
              edits: [SPLICE,],
              splices: [SPLICE,],
            },);
          },
        },),
      ],
    },),
    describe({
      name: idsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE NAMES OF SOME REPLACEMENTS AS A SET, each once, and an empty set for none',
          fn: async () => {
            expect([
              [...idsOf({ edits: [{ id: 'a', start: 0, end: 1, text: 'x', }, { id: 'b', start: 2, end: 3, text: 'y', }, { id: 'a', start: 4, end: 5, text: 'z', },], },),],
              [...idsOf({ edits: [], },),],
            ],).toEqual([['a', 'b',], [],],);
          },
        },),
      ],
    },),
    describe({
      name: replacedAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COPIES A LIST WITH ONE ENTRY REPLACED, leaving the list it was given as it was',
          fn: async () => {
            const list = ['a', 'b', 'c',] as const;
            expect([
              replacedAt({ list, position: 1, entry: 'x', },),
              list,
            ],).toEqual([['a', 'x', 'c',], ['a', 'b', 'c',],],);
          },
        },),
      ],
    },),
    describe({
      name: keyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A SPLICE BY THE PLACE OF ITS SLICE AND ITS OWN NAME, so the same name in two slices is two keys',
          fn: async () => {
            expect([
              keyOf({ position: 2, id: SPLICE, },),
              keyOf({ position: 3, id: SPLICE, },),
            ],).toEqual(['2 group 1 OCCURRENCE 2', '3 group 1 OCCURRENCE 2',],);
          },
        },),
      ],
    },),
  ],
},);
