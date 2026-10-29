/**
 Tests for reading the parts inside one recorded consistency group: the
 repeated wording or frame, a place no slice holds whole, and one heard ballot.
 Each is read key by key, so every case here breaks one valid record in one way
 and pins the whole refusal: the path it names and what was expected there.

 What the parts say about each other is the group reader's question and has its
 own cases (`artifact-two-lane-read-consistency.unit.test.ts`).

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
  parseConsistencyBallot,
  parseConsistencyRepeat,
  parseConsistencyUnplaced,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 The count refusal's reason, which every offset and every count of a ballot shares.
 */
const NOT_A_COUNT = 'a non-negative integer no larger than JSON carries exactly';

/**
 A frame as the writer records it.
 */
const FRAME = {
  kind: 'refrain',
  prefix: '所以这只猫',
  suffix: '的小宝贝',
};

/**
 A run repeated as it stands, as the writer records it.
 */
const RUN = {
  kind: 'maximal',
  wording: '小猫打盹',
};

/**
 A place no slice holds whole, as the writer records it.
 */
const STRAY = {
  startOffset: 6,
  endOffset: 18,
  line: 3,
  reason: 'in-no-slice',
};

/**
 A ballot with a quote that carries a slot and one that does not.
 */
const BALLOT = {
  seat: 'hyper-cat',
  reason: 'The first place words it best.',
  quotes: [
    {
      label: 'OCCURRENCE 1',
      quote: 'So this cat is a lazy little treasure.',
      slot: 'lazy',
    },
    {
      label: 'OCCURRENCE 2',
      quote: 'The kitten naps',
    },
  ],
  choice: 'OCCURRENCE 1',
  keep: ['OCCURRENCE 2',],
};

/**
 The text of the refusal a repeat raises.

 @param value - recorded repeat

 @returns Refusal as text

 @example
 ```ts
 const text = repeatRefusal({ value: 'naps', },);
 ```
 */
function repeatRefusal({ value, }: { readonly value: unknown; },): string {
  return refusalTextOf({
    read: function readsRepeat(): unknown {
      return parseConsistencyRepeat({
        value,
        path: 'cat.repeat',
      },);
    },
  },);
}

/**
 The text of the refusal a place no slice holds whole raises.

 @param value - recorded place

 @returns Refusal as text

 @example
 ```ts
 const text = strayRefusal({ value: 'naps', },);
 ```
 */
function strayRefusal({ value, }: { readonly value: unknown; },): string {
  return refusalTextOf({
    read: function readsStray(): unknown {
      return parseConsistencyUnplaced({
        value,
        path: 'cat.unplaced[0]',
      },);
    },
  },);
}

/**
 The text of the refusal a ballot raises.

 @param value - recorded ballot

 @returns Refusal as text

 @example
 ```ts
 const text = ballotRefusal({ value: 'naps', },);
 ```
 */
function ballotRefusal({ value, }: { readonly value: unknown; },): string {
  return refusalTextOf({
    read: function readsBallot(): unknown {
      return parseConsistencyBallot({
        value,
        path: 'cat.ballots[0]',
      },);
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistencyRepeat.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A RUN REPEATED AS IT STANDS AND A FRAME WITH A SLOT, each as the writer records it',
          fn: async () => {
            expect([RUN, FRAME,].map(function read(value,): unknown {
              return parseConsistencyRepeat({
                value,
                path: 'cat.repeat',
              },);
            },),).toEqual([RUN, FRAME,],);
          },
        },),
        it({
          name: 'REFUSES A REPEAT THAT IS NOT AN OBJECT AND A KIND OUTSIDE THE VOCABULARY, naming the path',
          fn: async () => {
            expect([
              repeatRefusal({ value: 'naps', },),
              repeatRefusal({ value: { kind: 'echo', }, },),
              repeatRefusal({ value: { kind: 7, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.repeat: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.repeat.kind: expected one of maximal, refrain.',
              'ArtifactParseError: artifact parse failed at cat.repeat.kind: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A RUN WITH A KEY OF A FRAME AND A RUN WHOSE WORDING IS NOT TEXT',
          fn: async () => {
            expect([
              repeatRefusal({ value: { ...RUN, prefix: '所以', }, },),
              repeatRefusal({ value: { kind: 'maximal', }, },),
              repeatRefusal({ value: { ...RUN, wording: 4, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.repeat.prefix: expected no key here beyond kind, wording.',
              'ArtifactParseError: artifact parse failed at cat.repeat.wording: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.repeat.wording: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A FRAME WITH THE WORDING OF A RUN AND A FRAME WHOSE PREFIX OR SUFFIX IS NOT TEXT',
          fn: async () => {
            expect([
              repeatRefusal({ value: { ...FRAME, wording: '小猫', }, },),
              repeatRefusal({ value: { kind: 'refrain', suffix: '的小宝贝', }, },),
              repeatRefusal({ value: { kind: 'refrain', prefix: '所以这只猫', }, },),
              repeatRefusal({ value: { ...FRAME, suffix: 4, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.repeat.wording: '
                + 'expected no key here beyond kind, prefix, suffix.',
              'ArtifactParseError: artifact parse failed at cat.repeat.prefix: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.repeat.suffix: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.repeat.suffix: expected a string.',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: parseConsistencyUnplaced.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A PLACE FOR EACH REASON NO SLICE HOLDS IT WHOLE, as written',
          fn: async () => {
            expect(['in-no-slice', 'crosses-slice-end',].map(function read(reason,): unknown {
              return parseConsistencyUnplaced({
                value: { ...STRAY, reason, },
                path: 'cat.unplaced[0]',
              },);
            },),).toEqual([
              { ...STRAY, reason: 'in-no-slice', },
              { ...STRAY, reason: 'crosses-slice-end', },
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THAT IS NOT AN OBJECT AND A KEY THE PLACE DOES NOT HAVE, naming the path',
          fn: async () => {
            expect([
              strayRefusal({ value: 'naps', },),
              strayRefusal({ value: { ...STRAY, label: 'OCCURRENCE 1', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.unplaced[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.unplaced[0].label: '
                + 'expected no key here beyond startOffset, endOffset, line, reason.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN OFFSET OR A LINE THAT IS NOT A COUNT, whether absent, text, negative or fractional',
          fn: async () => {
            expect([
              strayRefusal({ value: { ...STRAY, startOffset: '6', }, },),
              strayRefusal({ value: { ...STRAY, endOffset: -1, }, },),
              strayRefusal({ value: { ...STRAY, line: 1.5, }, },),
              strayRefusal({ value: { startOffset: 6, endOffset: 18, reason: 'in-no-slice', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.unplaced[0].startOffset: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.unplaced[0].endOffset: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.unplaced[0].line: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.unplaced[0].line: expected a number.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THAT ENDS WHERE IT STARTS OR BEFORE IT, and a place on a line before the first',
          fn: async () => {
            const refusal = 'ArtifactParseError: artifact parse failed at cat.unplaced[0]: '
              + 'expected a place that ends after it starts, on a line counted from one.';
            expect([
              strayRefusal({ value: { ...STRAY, endOffset: STRAY.startOffset, }, },),
              strayRefusal({ value: { ...STRAY, endOffset: STRAY.startOffset - 1, }, },),
              strayRefusal({ value: { ...STRAY, line: 0, }, },),
            ],).toEqual([refusal, refusal, refusal,],);
          },
        },),
        it({
          name: 'READS A PLACE THAT ENDS ONE CHARACTER AFTER IT STARTS ON THE FIRST LINE',
          fn: async () => {
            /**
             The smallest place a repeat can stand in.
             */
            const smallest = { ...STRAY, startOffset: 0, endOffset: 1, line: 1, };
            expect(parseConsistencyUnplaced({ value: smallest, path: 'cat.unplaced[0]', },),).toEqual(smallest,);
          },
        },),
        it({
          name: 'REFUSES A REASON OUTSIDE THE VOCABULARY AND A REASON THAT IS NOT TEXT',
          fn: async () => {
            expect([
              strayRefusal({ value: { ...STRAY, reason: 'lost', }, },),
              strayRefusal({ value: { ...STRAY, reason: 4, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.unplaced[0].reason: '
                + 'expected one of in-no-slice, crosses-slice-end.',
              'ArtifactParseError: artifact parse failed at cat.unplaced[0].reason: expected a string.',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: parseConsistencyBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A BALLOT WITH A SLOT ON ONE QUOTE AND NONE ON THE OTHER, and a ballot with no quotes and no kept places',
          fn: async () => {
            expect([
              BALLOT,
              {
                ...BALLOT,
                quotes: [],
                keep: [],
              },
            ].map(function read(value,): unknown {
              return parseConsistencyBallot({
                value,
                path: 'cat.ballots[0]',
              },);
            },),).toEqual([
              BALLOT,
              {
                ...BALLOT,
                quotes: [],
                keep: [],
              },
            ],);
          },
        },),
        it({
          name: 'REFUSES A BALLOT THAT IS NOT AN OBJECT AND A KEY THE BALLOT DOES NOT HAVE, naming the path',
          fn: async () => {
            expect([
              ballotRefusal({ value: 'naps', },),
              ballotRefusal({ value: { ...BALLOT, weight: 1, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.ballots[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].weight: '
                + 'expected no key here beyond seat, reason, quotes, choice, keep.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A SEAT, A REASON OR A CHOICE THAT IS NOT TEXT',
          fn: async () => {
            expect([
              ballotRefusal({ value: { ...BALLOT, seat: 4, }, },),
              ballotRefusal({ value: { ...BALLOT, reason: null, }, },),
              ballotRefusal({ value: { ...BALLOT, choice: 1, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.ballots[0].seat: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].reason: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].choice: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES QUOTES THAT ARE NOT A LIST OF OBJECTS, naming the quote by its position',
          fn: async () => {
            expect([
              ballotRefusal({ value: { ...BALLOT, quotes: 'naps', }, },),
              ballotRefusal({ value: { ...BALLOT, quotes: ['naps',], }, },),
              ballotRefusal({
                value: {
                  ...BALLOT,
                  quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps', place: 1, },],
                },
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes[0].place: '
                + 'expected no key here beyond label, quote, slot.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE WHOSE LABEL, WORDS OR SLOT ARE NOT TEXT, the second quote named by its position',
          fn: async () => {
            expect([
              ballotRefusal({ value: { ...BALLOT, quotes: [{ label: 1, quote: 'The cat naps', },], }, },),
              ballotRefusal({ value: { ...BALLOT, quotes: [{ label: 'OCCURRENCE 1', },], }, },),
              ballotRefusal({
                value: {
                  ...BALLOT,
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'The cat naps', },
                    { label: 'OCCURRENCE 2', quote: 'The kitten naps', slot: 2, },
                  ],
                },
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes[0].label: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes[0].quote: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].quotes[1].slot: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES KEPT PLACES THAT ARE NOT A LIST OF TEXT, naming the entry by its position',
          fn: async () => {
            expect([
              ballotRefusal({ value: { ...BALLOT, keep: 'OCCURRENCE 2', }, },),
              ballotRefusal({ value: { ...BALLOT, keep: ['OCCURRENCE 2', 3,], }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.ballots[0].keep: expected an array.',
              'ArtifactParseError: artifact parse failed at cat.ballots[0].keep[1]: expected a string.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
