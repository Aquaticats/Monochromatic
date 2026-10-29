/**
 Tests for reading one place of a recorded consistency group: its label, slice,
 offsets and line, the slot offsets of a frame's place, how its rendering was
 found, and what became of it. Each is read key by key, so every case breaks
 one valid record in one way and pins the whole refusal.

 What the places say beside the kind of repeat and the slices the comparison
 names is the fit check's question
 (`artifact-two-lane-read-consistency-fit.unit.test.ts`).

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
  keptReasonExpected,
  parseConsistencyOccurrence,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 The count refusal's reason, which every offset, line and slice index shares.
 */
const NOT_A_COUNT = 'a non-negative integer no larger than JSON carries exactly';

/**
 The refusal text of the reason an offset pair or a line is out of order.
 */
const OUT_OF_ORDER = 'ArtifactParseError: artifact parse failed at cat.occurrences[0]: '
  + 'expected a place that ends after it starts, on a line counted from one.';

/**
 A place whose rendering was located, as the writer records it.
 */
const LOCATED = {
  label: 'OCCURRENCE 1',
  sliceIndex: 0,
  startOffset: 6,
  endOffset: 18,
  line: 3,
  rendering: { kind: 'located', quote: 'The cat naps.', },
  outcome: 'chosen',
};

/**
 A place of a frame, whose rendering was located with its slot.
 */
const FRAMED = {
  ...LOCATED,
  slotStartOffset: 9,
  slotEndOffset: 11,
  rendering: { kind: 'located', quote: 'So this cat is a lazy little treasure.', slot: 'lazy', },
};

/**
 A place whose rendering could not be pinned down.
 */
const UNLOCATED = {
  ...LOCATED,
  rendering: { kind: 'unlocated', reason: 'not-found', },
  outcome: 'unlocated',
};

/**
 Every reason a rendering can fail to be located.
 */
const UNLOCATED_REASONS = [
  'not-asked',
  'no-english',
  'no-agreement',
  'empty',
  'not-found',
  'ambiguous',
  'markup',
  'overlaps-another',
  'slot-no-agreement',
  'slot-empty',
  'slot-not-found',
  'slot-ambiguous',
  'slot-whole',
];

/**
 Every outcome the vocabulary names, in the order the refusal lists them.
 */
const OUTCOMES = [
  'chosen',
  'replaced',
  'consistent',
  'kept-context',
  'kept-floor',
  'kept-current',
  'overlapped',
  'unlocated',
  'untouched',
];

/**
 Every reason the pass leaves a place as it was for, in the order the refusal lists them: a gain left out on the final page
 with the reason it was left out for, and a splice refused at the edge of the span.
 */
const KEPT_REASONS = [
  'gain-pairs',
  'gain-unproves',
  'gain-floors',
  'gain-overlaps',
  'gain-refused',
  'edge-second-mark',
  'edge-reading-changes',
  'edge-reference-mark',
];

/**
 Every outcome a place of generation fifteen can have, which has no carry to end in `kept-current`.
 */
const OUTCOMES_OF_FIFTEEN = OUTCOMES.filter(function notKeptCurrent(outcome,): boolean {
  return outcome !== 'kept-current';
});

/**
 A place of generation sixteen whose rendering was located, which records the offset of its words.
 */
const LOCATED_BY_SPAN = {
  ...LOCATED,
  rendering: { kind: 'located', quote: 'The cat naps', start: 4, },
};

/**
 A place of a frame of generation sixteen, located with its slot.
 */
const FRAMED_BY_SPAN = {
  ...FRAMED,
  rendering: { kind: 'located', quote: 'So this cat is a lazy little treasure', start: 4, slot: 'lazy', },
};

/**
 A place read as written.

 @param value - recorded place

 @param generation - generation the section was written under

 @returns What the reader makes of it

 @example
 ```ts
 const place = placeRead({ value: LOCATED, },);
 ```
 */
function placeRead(
  {
    value,
    generation = 15,
  }: {
    readonly value: unknown;
    readonly generation?: 15 | 16 | 17;
  },
): unknown {
  return parseConsistencyOccurrence({
    value,
    path: 'cat.occurrences[0]',
    generation,
  },);
}

/**
 The text of the refusal a place raises.

 @param value - recorded place

 @param generation - generation the section was written under

 @returns Refusal as text

 @example
 ```ts
 const text = placeRefusal({ value: 'naps', },);
 ```
 */
function placeRefusal(
  {
    value,
    generation = 15,
  }: {
    readonly value: unknown;
    readonly generation?: 15 | 16 | 17;
  },
): string {
  return refusalTextOf({
    read: function readsPlace(): unknown {
      return placeRead({
        value,
        generation,
      },);
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseConsistencyOccurrence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A PLACE LOCATED WITH NO SLOT, A PLACE OF A FRAME LOCATED WITH ITS SLOT, AND AN UNLOCATED PLACE, as written',
          fn: async () => {
            expect([LOCATED, FRAMED, UNLOCATED,].map(function read(value,): unknown {
              return placeRead({ value, },);
            },),).toEqual([LOCATED, FRAMED, UNLOCATED,],);
          },
        },),
        it({
          name: 'READS A PLACE FOR EACH REASON ITS RENDERING WAS NOT LOCATED',
          fn: async () => {
            expect(UNLOCATED_REASONS.map(function read(reason,): unknown {
              return placeRead({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', reason, }, }, },);
            },),).toEqual(UNLOCATED_REASONS.map(function written(reason,): unknown {
              return { ...UNLOCATED, rendering: { kind: 'unlocated', reason, }, };
            },),);
          },
        },),
        it({
          name: 'READS A PLACE FOR EACH OUTCOME A PLACE OF GENERATION FIFTEEN CAN HAVE',
          fn: async () => {
            expect(OUTCOMES_OF_FIFTEEN.map(function read(outcome,): unknown {
              return placeRead({ value: { ...LOCATED, outcome, }, },);
            },),).toEqual(OUTCOMES_OF_FIFTEEN.map(function written(outcome,): unknown {
              return { ...LOCATED, outcome, };
            },),);
          },
        },),
        it({
          name: 'READS A PLACE FOR EACH OUTCOME A PLACE OF GENERATION SIXTEEN CAN HAVE, kept-current among them, and refuses '
            + 'kept-current in generation fifteen, which has no carry for a place to keep its English in',
          fn: async () => {
            expect(OUTCOMES.map(function read(outcome,): unknown {
              return placeRead({ value: { ...LOCATED_BY_SPAN, outcome, }, generation: 16, },);
            },),).toEqual(OUTCOMES.map(function written(outcome,): unknown {
              return { ...LOCATED_BY_SPAN, outcome, };
            },),);
            expect(placeRefusal({ value: { ...LOCATED, outcome: 'kept-current', }, },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].outcome: expected an outcome generation fifteen '
                + 'gives, which has no carry to end in kept-current.',
            );
          },
        },),
        it({
          name: 'READS A LOCATED PLACE OF GENERATION SIXTEEN WITH THE OFFSET OF ITS WORDS, with and without a slot, and refuses '
            + 'one without the offset and an offset in generation fifteen',
          fn: async () => {
            expect([LOCATED_BY_SPAN, FRAMED_BY_SPAN,].map(function read(value,): unknown {
              return placeRead({ value, generation: 16, },);
            },),).toEqual([LOCATED_BY_SPAN, FRAMED_BY_SPAN,],);
            expect([
              placeRefusal({ value: LOCATED, generation: 16, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, rendering: { kind: 'located', quote: 'x', start: -1, }, }, generation: 16, },),
              placeRefusal({ value: LOCATED_BY_SPAN, generation: 15, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.start: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.start: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.start: '
                + 'expected no key here beyond kind, quote, slot.',
            ],);
          },
        },),
        it({
          name: 'READS THE SLOT REASONS OF GENERATION FIFTEEN AS WRITTEN THERE, and refuses each in generation sixteen, which '
            + 'locates a place on its quote and not its slot',
          fn: async () => {
            const slotReasons = UNLOCATED_REASONS.filter(function isSlotReason(reason,): boolean {
              return reason.startsWith('slot-',);
            },);
            expect(slotReasons.map(function readFifteen(reason,): unknown {
              return placeRead({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', reason, }, }, generation: 15, },);
            },),).toEqual(slotReasons.map(function written(reason,): unknown {
              return { ...UNLOCATED, rendering: { kind: 'unlocated', reason, }, };
            },),);
            expect(slotReasons.map(function refuseSixteen(reason,): string {
              return placeRefusal({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', reason, }, }, generation: 16, },);
            },),).toEqual(slotReasons.map(function refused(): string {
              return 'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.reason: expected a reason '
                + 'generation sixteen gives, which locates a place on its quote and not its slot.';
            },),);
          },
        },),
        it({
          name: 'READS A PLACE OF GENERATION SEVENTEEN FOR EACH OUTCOME, an overlapped place with the place it gave way to',
          fn: async () => {
            /**
             The place an overlapped place gave way to.
             */
            const gaveWayTo = { group: 1, label: 'OCCURRENCE 2', };
            /**
             Each outcome's place, written as generation seventeen writes it.
             */
            const written = OUTCOMES.map(function writtenFor(outcome,): unknown {
              return (outcome === 'overlapped')
                ? { ...LOCATED_BY_SPAN, outcome, gaveWayTo, }
                : { ...LOCATED_BY_SPAN, outcome, };
            },);
            expect(written.map(function read(value,): unknown {
              return placeRead({ value, generation: 17, },);
            },),).toEqual(written,);
          },
        },),
        it({
          name: 'READS A LOCATED PLACE OF GENERATION SEVENTEEN THAT RECORDS THE SINGLE MARKS BESIDE ITS WORDS, one, both or '
            + 'neither, whichever of the four single marks they are, with and without a slot',
          fn: async () => {
            /**
             A located place of generation seventeen with the fields of its rendering added.
             */
            function withRendering({ over, }: { readonly over: Record<string, unknown>; },): unknown {
              return { ...LOCATED_BY_SPAN, rendering: { ...LOCATED_BY_SPAN.rendering, ...over, }, };
            }
            const written = [
              withRendering({ over: { markBefore: '‘', markAfter: '’', }, },),
              withRendering({ over: { markBefore: '\'', }, },),
              withRendering({ over: { markAfter: '\u{02BC}', }, },),
              withRendering({ over: {}, },),
              {
                ...FRAMED_BY_SPAN,
                rendering: { ...FRAMED_BY_SPAN.rendering, markAfter: '\'', },
              },
            ];
            expect(written.map(function read(value,): unknown {
              return placeRead({ value, generation: 17, },);
            },),).toEqual(written,);
          },
        },),
        it({
          name: 'REFUSES A MARK BESIDE THE WORDS THAT IS NOT ONE SINGLE MARK in generation seventeen: empty, two marks, a '
            + 'double quotation mark, a letter, and a value that is not text',
          fn: async () => {
            /**
             The refusal for a rendering of generation seventeen holding one value under one mark key.
             */
            function refusalFor({ key, value, }: { readonly key: string; readonly value: unknown; },): string {
              return placeRefusal({
                value: { ...LOCATED_BY_SPAN, rendering: { ...LOCATED_BY_SPAN.rendering, [key]: value, }, },
                generation: 17,
              },);
            }
            const reason = 'expected one single mark, an apostrophe or a single quotation mark.';
            expect([
              refusalFor({ key: 'markBefore', value: '', },),
              refusalFor({ key: 'markBefore', value: '\'\'', },),
              refusalFor({ key: 'markAfter', value: '"', },),
              refusalFor({ key: 'markAfter', value: 'x', },),
              refusalFor({ key: 'markAfter', value: 4, },),
            ],).toEqual([
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markBefore: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markBefore: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markAfter: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markAfter: ${reason}`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markAfter: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES THE MARKS BESIDE THE WORDS IN GENERATIONS FIFTEEN AND SIXTEEN, which recorded none',
          fn: async () => {
            expect([
              placeRefusal({
                value: { ...LOCATED, rendering: { ...LOCATED.rendering, markBefore: '\'', }, },
                generation: 15,
              },),
              placeRefusal({
                value: { ...LOCATED_BY_SPAN, rendering: { ...LOCATED_BY_SPAN.rendering, markAfter: '\'', }, },
                generation: 16,
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markBefore: '
                + 'expected no key here beyond kind, quote, slot.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.markAfter: '
                + 'expected no key here beyond kind, quote, start, slot.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN OVERLAPPED PLACE OF GENERATION SEVENTEEN THAT NAMES NO PLACE IT GAVE WAY TO, a place of another outcome '
            + 'that names one, and the place in generations that record none',
          fn: async () => {
            const gaveWayTo = { group: 1, label: 'OCCURRENCE 2', };
            expect([
              placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'overlapped', }, generation: 17, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'replaced', gaveWayTo, }, generation: 17, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'overlapped', gaveWayTo, }, generation: 16, },),
              placeRefusal({ value: { ...LOCATED, outcome: 'overlapped', gaveWayTo, }, generation: 15, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, slot: 'lazy', }, generation: 17, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo: expected the place an overlapped place '
                + 'gave way to, which generation seventeen records.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo: expected an overlapped place, the only '
                + 'kind that gave way to another place.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo: expected no key here beyond label, '
                + 'sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo: expected no key here beyond label, '
                + 'sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].slot: expected no key here beyond label, '
                + 'sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome, gaveWayTo, '
                + 'keptBecause.',
            ],);
          },
        },),
        it({
          name: 'READS THE REASON A KEPT-CURRENT PLACE OF GENERATION SEVENTEEN WAS LEFT AS IT WAS, each reason of the closed set',
          fn: async () => {
            /**
             Each place, recorded as generation seventeen records a place the pass left as it was.
             */
            const written = KEPT_REASONS.map(function writtenFor(keptBecause,): unknown {
              return { ...LOCATED_BY_SPAN, outcome: 'kept-current', keptBecause, };
            },);
            expect(written.map(function read(value,): unknown {
              return placeRead({ value, generation: 17, },);
            },),).toEqual(written,);
          },
        },),
        it({
          name: 'WORDS WHAT IS EXPECTED OF THE REASON OF A PLACE, naming the place and the closed set and saying what the record '
            + 'holds: none, or the text it holds as text',
          fn: async () => {
            expect([
              keptReasonExpected({ label: 'OCCURRENCE 3', },),
              keptReasonExpected({ label: 'OCCURRENCE 3', read: 'a "cat"', },),
            ],).toEqual([
              `a reason OCCURRENCE 3 was left as it was for, one of ${KEPT_REASONS.join(', ',)}; the record has none`,
              `a reason OCCURRENCE 3 was left as it was for, one of ${KEPT_REASONS.join(', ',)}; the record has "a \\"cat\\""`,
            ],);
          },
        },),
        it({
          name: 'REFUSES A REASON OUTSIDE THE CLOSED SET, naming the place and the reason read, one that is not text, a reason on '
            + 'a place that is not kept-current, and the reason in generations that record none',
          fn: async () => {
            /**
             A kept-current place of generation seventeen holding a value as its reason.
             */
            function keptWith({ keptBecause, }: { readonly keptBecause: unknown; },): string {
              return placeRefusal({
                value: { ...LOCATED_BY_SPAN, outcome: 'kept-current', keptBecause, },
                generation: 17,
              },);
            }
            expect([
              keptWith({ keptBecause: 'invented', },),
              keptWith({ keptBecause: 'edge-reading-change', },),
              keptWith({ keptBecause: 3, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'replaced', keptBecause: 'gain-pairs', }, generation: 17, },),
              placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'kept-current', keptBecause: 'gain-pairs', }, generation: 16, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].keptBecause: expected a reason OCCURRENCE 1 was left '
                + 'as it was for, one of gain-pairs, gain-unproves, gain-floors, gain-overlaps, gain-refused, edge-second-mark, '
                + 'edge-reading-changes, edge-reference-mark; the record has "invented".',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].keptBecause: expected a reason OCCURRENCE 1 was left '
                + 'as it was for, one of gain-pairs, gain-unproves, gain-floors, gain-overlaps, gain-refused, edge-second-mark, '
                + 'edge-reading-changes, edge-reference-mark; the record has "edge-reading-change".',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].keptBecause: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].keptBecause: expected a kept-current place, the only '
                + 'kind the pass leaves as it was for a reason.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].keptBecause: expected no key here beyond label, '
                + 'sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE IT GAVE WAY TO THAT IS NOT AN OBJECT, HOLDS A KEY BEYOND GROUP AND LABEL, OR HAS A GROUP THAT IS NOT A '
            + 'COUNT OR A LABEL THAT IS NOT TEXT',
          fn: async () => {
            /**
             The refusal of an overlapped place of generation seventeen naming a place that is wrong in one way.
             */
            function namingRefusal({ gaveWayTo, }: { readonly gaveWayTo: unknown; },): string {
              return placeRefusal({ value: { ...LOCATED_BY_SPAN, outcome: 'overlapped', gaveWayTo, }, generation: 17, },);
            }
            expect([
              namingRefusal({ gaveWayTo: 'OCCURRENCE 2', },),
              namingRefusal({ gaveWayTo: { group: 1, label: 'OCCURRENCE 2', start: 3, }, },),
              namingRefusal({ gaveWayTo: { group: -1, label: 'OCCURRENCE 2', }, },),
              namingRefusal({ gaveWayTo: { group: 1.5, label: 'OCCURRENCE 2', }, },),
              namingRefusal({ gaveWayTo: { group: 1, label: 2, }, },),
              namingRefusal({ gaveWayTo: { label: 'OCCURRENCE 2', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo.start: expected no key here beyond group, label.',
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo.group: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo.group: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo.label: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].gaveWayTo.group: expected a number.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THAT IS NOT AN OBJECT AND A KEY THE PLACE DOES NOT HAVE, naming the path',
          fn: async () => {
            expect([
              placeRefusal({ value: 'naps', },),
              placeRefusal({ value: { ...LOCATED, slot: 'lazy', }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0]: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].slot: expected no key here beyond label, '
                + 'sliceIndex, startOffset, endOffset, line, slotStartOffset, slotEndOffset, rendering, outcome.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN OFFSET, A LINE OR A SLICE THAT IS NOT A COUNT, whether absent, text, negative or fractional',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...LOCATED, startOffset: '6', }, },),
              placeRefusal({ value: { ...LOCATED, endOffset: -1, }, },),
              placeRefusal({ value: { ...LOCATED, line: 1.5, }, },),
              placeRefusal({ value: { ...LOCATED, sliceIndex: -1, }, },),
              placeRefusal({ value: { ...LOCATED, sliceIndex: undefined, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].startOffset: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].endOffset: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].line: expected ${NOT_A_COUNT}.`,
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].sliceIndex: expected ${NOT_A_COUNT}.`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].sliceIndex: expected a number.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THAT ENDS WHERE IT STARTS OR BEFORE IT, and a place on a line before the first',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...LOCATED, endOffset: LOCATED.startOffset, }, },),
              placeRefusal({ value: { ...LOCATED, endOffset: LOCATED.startOffset - 1, }, },),
              placeRefusal({ value: { ...LOCATED, line: 0, }, },),
            ],).toEqual([OUT_OF_ORDER, OUT_OF_ORDER, OUT_OF_ORDER,],);
          },
        },),
        it({
          name: 'READS A PLACE THAT ENDS ONE CHARACTER AFTER IT STARTS ON THE FIRST LINE',
          fn: async () => {
            expect(placeRead({
              value: {
                ...LOCATED,
                startOffset: 0,
                endOffset: 1,
                line: 1,
              },
            },),).toEqual({
              ...LOCATED,
              startOffset: 0,
              endOffset: 1,
              line: 1,
            },);
          },
        },),
        it({
          name: 'REFUSES A LABEL THAT IS NOT TEXT AND SLOT OFFSETS THAT ARE NOT COUNTS',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...LOCATED, label: 1, }, },),
              placeRefusal({ value: { ...FRAMED, slotStartOffset: '9', }, },),
              placeRefusal({ value: { ...FRAMED, slotEndOffset: -2, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].label: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].slotStartOffset: expected a number.',
              `ArtifactParseError: artifact parse failed at cat.occurrences[0].slotEndOffset: expected ${NOT_A_COUNT}.`,
            ],);
          },
        },),
        it({
          name: 'REFUSES A RENDERING THAT IS NOT AN OBJECT AND A KIND OUTSIDE THE VOCABULARY, naming the path',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...LOCATED, rendering: 'naps', }, },),
              placeRefusal({ value: { ...LOCATED, rendering: { kind: 'maybe', }, }, },),
              placeRefusal({ value: { ...LOCATED, rendering: { kind: 3, }, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering: expected an object.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.kind: '
                + 'expected one of located, unlocated.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.kind: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN UNLOCATED RENDERING THAT CARRIES A QUOTE AND A REASON OUTSIDE THE VOCABULARY, naming every reason',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', reason: 'not-found', quote: 'x', }, }, },),
              placeRefusal({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', reason: 'shy', }, }, },),
              placeRefusal({ value: { ...UNLOCATED, rendering: { kind: 'unlocated', }, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.quote: '
                + 'expected no key here beyond kind, reason.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.reason: expected one of '
                + `${UNLOCATED_REASONS.join(', ',)}.`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.reason: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING THAT CARRIES A REASON, HAS NO QUOTE, OR HAS A QUOTE OR A SLOT THAT IS NOT TEXT',
          fn: async () => {
            expect([
              placeRefusal({
                value: { ...LOCATED, rendering: { kind: 'located', quote: 'The cat naps.', reason: 'not-found', }, },
              },),
              placeRefusal({ value: { ...LOCATED, rendering: { kind: 'located', }, }, },),
              placeRefusal({ value: { ...LOCATED, rendering: { kind: 'located', quote: 4, }, }, },),
              placeRefusal({
                value: { ...LOCATED, rendering: { kind: 'located', quote: 'The cat naps.', slot: 4, }, },
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.reason: '
                + 'expected no key here beyond kind, quote, slot.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.quote: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.quote: expected a string.',
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].rendering.slot: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN OUTCOME OUTSIDE THE VOCABULARY, naming every outcome, and an outcome that is not text',
          fn: async () => {
            expect([
              placeRefusal({ value: { ...LOCATED, outcome: 'liked', }, },),
              placeRefusal({ value: { ...LOCATED, outcome: 4, }, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].outcome: expected one of '
                + `${OUTCOMES.join(', ',)}.`,
              'ArtifactParseError: artifact parse failed at cat.occurrences[0].outcome: expected a string.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
