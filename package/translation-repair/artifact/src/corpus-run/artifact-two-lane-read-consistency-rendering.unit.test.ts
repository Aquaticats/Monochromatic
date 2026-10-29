/**
 Tests for reading the rendering of one place of a recorded consistency group
 under the generation the section was written in: the keys a located rendering
 may hold, the offset of its span, the single marks its English holds beside the
 span, and the reasons a place stays unlocated. Each case breaks one valid record
 in one way and pins the whole refusal.

 The same rendering read through a whole place is in
 `artifact-two-lane-read-consistency-occurrence.unit.test.ts`.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseConsistencyRendering, } from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 Where the rendering sits, for the refusals.
 */
const WHERE = 'cat.rendering';

/**
 A rendering read as written.

 @param value - recorded rendering

 @param generation - generation the section was written under

 @returns What the reader makes of it

 @example
 ```ts
 const rendering = renderingRead({ value: { kind: 'unlocated', reason: 'not-found', }, generation: 16, },);
 ```
 */
function renderingRead(
  {
    value,
    generation,
  }: {
    readonly value: unknown;
    readonly generation: 15 | 16 | 17 | 18;
  },
): unknown {
  return parseConsistencyRendering({
    value,
    path: WHERE,
    generation,
  },);
}

/**
 The text of the refusal a rendering raises.

 @param value - recorded rendering

 @param generation - generation the section was written under

 @returns Refusal as text

 @example
 ```ts
 const text = renderingRefusal({ value: 4, generation: 16, },);
 ```
 */
function renderingRefusal(
  {
    value,
    generation,
  }: {
    readonly value: unknown;
    readonly generation: 15 | 16 | 17 | 18;
  },
): string {
  return refusalTextOf({
    read: function readsRendering(): unknown {
      return renderingRead({
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
      name: parseConsistencyRendering.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A LOCATED RENDERING IN EACH GENERATION WITH THE KEYS IT RECORDS: the quote and slot of fifteen, the '
            + 'offset of sixteen, the single marks beside the span of seventeen',
          fn: async () => {
            const fifteen = { kind: 'located', quote: 'The cat naps.', slot: 'cat', };
            const sixteen = { kind: 'located', quote: 'The cat naps', start: 4, slot: 'cat', };
            const seventeen = { kind: 'located', quote: 'The cat naps', start: 4, markBefore: '‘', markAfter: '’', slot: 'cat', };
            expect([
              renderingRead({ value: fifteen, generation: 15, },),
              renderingRead({ value: sixteen, generation: 16, },),
              renderingRead({ value: seventeen, generation: 17, },),
              renderingRead({ value: { kind: 'located', quote: 'The cat naps', start: 4, }, generation: 17, },),
            ],).toEqual([
              fifteen,
              sixteen,
              seventeen,
              { kind: 'located', quote: 'The cat naps', start: 4, },
            ],);
          },
        },),
        it({
          name: 'READS THE DEFINITIONS OF THE PAGE A GENERATION EIGHTEEN LOCATED RENDERING RECORDS, and refuses the key before '
            + 'eighteen, naming the keys that generation allows',
          fn: async () => {
            /**
             A rendering that records the definitions that move its slot.
             */
            const recorded = { kind: 'located', quote: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b', };
            expect([
              renderingRead({ value: recorded, generation: 18, },),
              renderingRead({ value: { kind: 'located', quote: 'a[b]c', start: 0, markBefore: '\'', slot: 'b', }, generation: 18, },),
              renderingRefusal({ value: recorded, generation: 17, },),
            ],).toEqual([
              recorded,
              { kind: 'located', quote: 'a[b]c', start: 0, markBefore: '\'', slot: 'b', },
              'ArtifactParseError: artifact parse failed at cat.rendering.definitions: '
                + 'expected no key here beyond kind, quote, start, markBefore, markAfter, slot.',
            ],);
          },
        },),
        it({
          name: 'REFUSES DEFINITIONS THAT ARE NOT TEXT, ARE EMPTY, OR ARE NOT IN THE FORM A PAGE\'S DEFINITIONS ARE READ IN: a number, '
            + 'an empty string, a sentence, and definitions with other text among them',
          fn: async () => {
            const reason = 'ArtifactParseError: artifact parse failed at cat.rendering.definitions: expected link and footnote '
              + 'definitions in the form a page\'s definitions are read in, and at least one.';
            expect([
              renderingRefusal({ value: { kind: 'located', quote: 'a[b]c', start: 0, definitions: 7, }, generation: 18, },),
              renderingRefusal({ value: { kind: 'located', quote: 'a[b]c', start: 0, definitions: '', }, generation: 18, },),
              renderingRefusal({ value: { kind: 'located', quote: 'a[b]c', start: 0, definitions: 'not a definition', }, generation: 18, },),
              renderingRefusal({
                value: { kind: 'located', quote: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b\nstray text', },
                generation: 18,
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.rendering.definitions: expected a string.',
              reason,
              reason,
              reason,
            ],);
          },
        },),
        it({
          name: 'READS AN UNLOCATED RENDERING WITH ITS REASON, and refuses a slot reason in generation sixteen and after',
          fn: async () => {
            expect([
              renderingRead({ value: { kind: 'unlocated', reason: 'slot-empty', }, generation: 15, },),
              renderingRead({ value: { kind: 'unlocated', reason: 'not-found', }, generation: 16, },),
              renderingRead({ value: { kind: 'unlocated', reason: 'ambiguous', }, generation: 17, },),
            ],).toEqual([
              { kind: 'unlocated', reason: 'slot-empty', },
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'unlocated', reason: 'ambiguous', },
            ],);
            expect(renderingRefusal({ value: { kind: 'unlocated', reason: 'slot-empty', }, generation: 17, },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.rendering.reason: expected a reason generation sixteen gives, '
                + 'which locates a place on its quote and not its slot.',
            );
          },
        },),
        it({
          name: 'REFUSES THE KEYS A GENERATION DOES NOT RECORD, the offset before sixteen and the marks beside the span before '
            + 'seventeen, naming the keys it allows',
          fn: async () => {
            expect([
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 4, }, generation: 15, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', markBefore: '\'', }, generation: 15, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 4, markAfter: '\'', }, generation: 16, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 4, reason: 'not-found', }, generation: 17, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.rendering.start: expected no key here beyond kind, quote, slot.',
              'ArtifactParseError: artifact parse failed at cat.rendering.markBefore: '
                + 'expected no key here beyond kind, quote, slot.',
              'ArtifactParseError: artifact parse failed at cat.rendering.markAfter: '
                + 'expected no key here beyond kind, quote, start, slot.',
              'ArtifactParseError: artifact parse failed at cat.rendering.reason: '
                + 'expected no key here beyond kind, quote, start, markBefore, markAfter, slot.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A MARK BESIDE THE SPAN THAT IS NOT EXACTLY ONE OF THE EIGHT SINGLE MARKS, and reads each of the eight, the '
            + 'low, the reversed and the two angle marks (`‚ ‛ ‹ ›`) among them, before and after the span',
          fn: async () => {
            const reason = 'expected one single mark, an apostrophe or a single quotation mark.';
            /**
             The eight single marks the pass counts and the record stores.
             */
            const eight = ['\'', '‘', '’', '\u{02BC}', '\u{201A}', '\u{201B}', '‹', '›',] as const;
            expect(eight.flatMap(function read(mark,): readonly unknown[] {
              return [
                renderingRead({
                  value: { kind: 'located', quote: 'x', start: 0, markBefore: mark, },
                  generation: 17,
                },),
                renderingRead({
                  value: { kind: 'located', quote: 'x', start: 0, markAfter: mark, },
                  generation: 17,
                },),
              ];
            },),).toEqual(eight.flatMap(function written(mark,): readonly unknown[] {
              return [
                { kind: 'located', quote: 'x', start: 0, markBefore: mark, },
                { kind: 'located', quote: 'x', start: 0, markAfter: mark, },
              ];
            },),);
            expect([
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markAfter: '', }, generation: 17, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markAfter: '\'s', }, generation: 17, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markBefore: '“', }, generation: 17, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markBefore: '«', }, generation: 17, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markAfter: '»', }, generation: 17, },),
              renderingRefusal({ value: { kind: 'located', quote: 'x', start: 0, markBefore: 7, }, generation: 17, },),
            ],).toEqual([
              `ArtifactParseError: artifact parse failed at cat.rendering.markAfter: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.rendering.markAfter: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.rendering.markBefore: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.rendering.markBefore: ${reason}`,
              `ArtifactParseError: artifact parse failed at cat.rendering.markAfter: ${reason}`,
              'ArtifactParseError: artifact parse failed at cat.rendering.markBefore: expected a string.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A RENDERING OF NO KNOWN KIND OR OF NO RECORD',
          fn: async () => {
            expect([
              renderingRefusal({ value: { kind: 'shy', }, generation: 17, },),
              renderingRefusal({ value: 4, generation: 17, },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.rendering.kind: expected one of located, unlocated.',
              'ArtifactParseError: artifact parse failed at cat.rendering: expected an object.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
