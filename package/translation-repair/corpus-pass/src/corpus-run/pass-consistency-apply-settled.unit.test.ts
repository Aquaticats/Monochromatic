/**
 Tests for what a settled slice carries and how the floors are asked about it: the reasons a splice is left out for,
 worded for the log, the question the floors are asked, and the index of the slice a settled slice stands for.

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
  floorsQuestionOf,
  gainFindingOf,
  type GainReason,
  keptReasonOfGain,
  type SettledSlice,
  sliceIndexOf,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: gainFindingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WORDS THE REASON THAT HOLDS ON THE FINAL PAGE for each kind of reason, naming the slice and the replacement, '
            + 'and ends each with the place keeping its words',
          fn: async () => {
            /**
             The edit the findings are about is named by its id alone.
             */
            const id = 'group 1 OCCURRENCE 2';
            const reasons: readonly GainReason[] = [
              { kind: 'pairs', },
              { kind: 'unproves', },
              { kind: 'floors', findings: ['the cat left', 'the dog came',], },
              { kind: 'overlaps', },
              { kind: 'refused', },
            ];
            expect(reasons.map(function findingOf(reason,): string {
              return gainFindingOf({ sliceIndex: 3, id, reason, },);
            },),).toEqual([
              'slice 3: the consistency replacement of group 1 OCCURRENCE 2 was left out because the apostrophe it carries in '
                + 'would read as a quotation mark there, since another quotation mark of the page could pair with it; the place '
                + 'keeps its words',
              'slice 3: the consistency replacement of group 1 OCCURRENCE 2 was left out because the apostrophe it carries in '
                + 'would make the apostrophe that another replacement carries in lose its proof; the place keeps its words',
              'slice 3: the consistency replacement of group 1 OCCURRENCE 2 was left out because the floors refuse the slice '
                + 'with it: the cat left | the dog came; the place keeps its words',
              'slice 3: the consistency replacement of group 1 OCCURRENCE 2 was left out because its words overlap the words '
                + 'of another replacement that is made; the place keeps its words',
              'slice 3: the consistency replacement of group 1 OCCURRENCE 2 was left out because the floors refused the other '
                + 'replacements of its slice, which keeps its wording whole; the place keeps its words',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: keptReasonOfGain.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE REASON THE RECORD KEEPS FOR EACH KIND OF REASON A GAIN IS LEFT OUT FOR, one name for each kind and none '
            + 'twice',
          fn: async () => {
            /**
             One reason of each kind.
             */
            const reasons: readonly GainReason[] = [
              { kind: 'pairs', },
              { kind: 'unproves', },
              { kind: 'floors', findings: ['the cat left',], },
              { kind: 'overlaps', },
              { kind: 'refused', },
            ];
            /**
             The record's name for each.
             */
            const named = reasons.map(function nameOf(reason,): string {
              return keptReasonOfGain({ reason, },);
            },);
            expect(named,).toEqual(['gain-pairs', 'gain-unproves', 'gain-floors', 'gain-overlaps', 'gain-refused',],);
            expect(new Set(named,).size,).toBe(reasons.length,);
          },
        },),
      ],
    },),
    describe({
      name: floorsQuestionOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES THE SAME QUESTION OF THE SAME SLICE AND REPLACEMENTS, and another of another set or another slice',
          fn: async () => {
            /**
             Two replacements, named by id alone.
             */
            const first = { id: 'group 1 OCCURRENCE 2', start: 0, end: 4, text: 'cats', };
            const second = { id: 'group 2 OCCURRENCE 2', start: 5, end: 9, text: 'naps', };
            expect([
              floorsQuestionOf({ sliceIndex: 1, edits: [first, second,], },),
              floorsQuestionOf({ sliceIndex: 1, edits: [first, second,], },),
              floorsQuestionOf({ sliceIndex: 1, edits: [first,], },),
              floorsQuestionOf({ sliceIndex: 2, edits: [first, second,], },),
              floorsQuestionOf({ sliceIndex: 2, edits: [], },),
            ],).toEqual([
              '1 group 1 OCCURRENCE 2 | group 2 OCCURRENCE 2',
              '1 group 1 OCCURRENCE 2 | group 2 OCCURRENCE 2',
              '1 group 1 OCCURRENCE 2',
              '2 group 1 OCCURRENCE 2 | group 2 OCCURRENCE 2',
              '2 ',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: sliceIndexOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE INDEX OF THE SLICE THE PLAN OF A SETTLED SLICE IS FOR',
          fn: async () => {
            /**
             A settled slice, whose plan is only as long as the facts it names.
             */
            const slice = { plan: { facts: { sliceIndex: 6, }, }, gainReasons: new Map(), } as unknown as SettledSlice;
            expect(sliceIndexOf({ slice, },),).toBe(6,);
          },
        },),
      ],
    },),
  ],
},);
