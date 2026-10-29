/**
 Tests for reading a recorded rendering of the sixteenth shape against the
 ballots: the counter's form of a located rendering, when the ballots contradict
 a reason a place was not located for, and when the words of a located rendering
 are ones the pass would not have located.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ConsistencyInvariantError,
  type UnlocatedReason,
} from '@monochromatic-dev/translation-repair-consistency';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  ARTIFACT_SCHEMA_VERSION_V19,
  asLocatedBySpan,
  locatedContradictedBySpan,
  reasonContradictedBySpan,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: asLocatedBySpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A LOCATED RENDERING WITHOUT A SLOT as the counter\'s form, the quote being the words and the core, and '
            + 'the recorded offset the start',
          fn: async () => {
            expect(asLocatedBySpan({
              rendering: { kind: 'located', quote: 'The cat naps', start: 9, },
              path: 'cat.rendering',
              generation: ARTIFACT_SCHEMA_VERSION_V19,
            },),).toEqual({ kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 9, },);
          },
        },),
        it({
          name: 'READS A LOCATED RENDERING WITH A SLOT, locating the slot once in the words, whatever the spacing and the '
            + 'apostrophes the words write',
          fn: async () => {
            expect(asLocatedBySpan({
              rendering: {
                kind: 'located',
                quote: 'So she was a gentle,\ntender-hearted girl,\nwasn’t she',
                start: 0,
                slot: 'gentle, tender-hearted',
              },
              path: 'cat.rendering',
              generation: ARTIFACT_SCHEMA_VERSION_V19,
            },),).toEqual({
              kind: 'located',
              quote: 'So she was a gentle,\ntender-hearted girl,\nwasn’t she',
              core: 'So she was a gentle,\ntender-hearted girl,\nwasn’t she',
              start: 0,
              slot: 'gentle, tender-hearted',
              slotStart: 13,
            },);
          },
        },),
        it({
          name: 'READS THE SINGLE MARKS A RENDERING RECORDS BESIDE ITS WORDS into the counter\'s form, with and without a '
            + 'slot, and adds none for a rendering that records none',
          fn: async () => {
            expect([
              asLocatedBySpan({
                rendering: { kind: 'located', quote: 'The cat naps', start: 9, markBefore: '‘', markAfter: '’', },
                path: 'cat.rendering',
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },),
              asLocatedBySpan({
                rendering: { kind: 'located', quote: 'the cat naps', start: 9, markAfter: '\'', slot: 'cat', },
                path: 'cat.rendering',
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },),
              asLocatedBySpan({
                rendering: { kind: 'located', quote: 'The cat naps', start: 9, },
                path: 'cat.rendering',
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },),
            ],).toEqual([
              { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 9, markBefore: '‘', markAfter: '’', },
              {
                kind: 'located',
                quote: 'the cat naps',
                core: 'the cat naps',
                start: 9,
                markAfter: '\'',
                slot: 'cat',
                slotStart: 4,
              },
              { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 9, },
            ],);
          },
        },),
        it({
          name: 'READS THE DEFINITIONS A RENDERING RECORDS INTO THE COUNTER\'S FORM, and locates the slot of a rendering with the '
            + 'definitions it was recorded with, where a recorded slot beside a reference is no stand without them',
          fn: async () => {
            expect(asLocatedBySpan({
              rendering: { kind: 'located', quote: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b', },
              path: 'cat.rendering',
              generation: ARTIFACT_SCHEMA_VERSION_V19,
            },),).toEqual({
              kind: 'located',
              quote: 'a[b]c',
              core: 'a[b]c',
              start: 0,
              definitions: '[b]: https://x.example/b',
            },);
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING THAT RECORDS NO OFFSET as a broken invariant, not as an offset of zero, since the '
            + 'sixteenth shape records one for every located rendering',
          fn: async () => {
            const refusal = caught(function readsWithoutOffset(): unknown {
              return asLocatedBySpan({
                rendering: { kind: 'located', quote: 'The cat naps', },
                path: 'cat.rendering',
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the located rendering at cat.rendering records no offset, and the '
                + 'reader of the sixteenth shape requires one',
            );
          },
        },),
        it({
          name: 'REFUSES A SLOT THAT STANDS NOWHERE OR TWICE IN THE WORDS, naming the path',
          fn: async () => {
            /**
             The refusal reading a slot raises.
             */
            function refusalOf({ slot, }: { readonly slot: string; },): string {
              return String(caught(function reads(): unknown {
                return asLocatedBySpan({
                  rendering: { kind: 'located', quote: 'the cat naps and the cat dozes', start: 0, slot, },
                  path: 'cat.rendering',
                  generation: ARTIFACT_SCHEMA_VERSION_V19,
                },);
              },),);
            }
            expect([refusalOf({ slot: 'bored', },), refusalOf({ slot: 'the cat', },),],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.rendering: '
                + 'expected a slot that stands exactly once in the words of its quote.',
              'ArtifactParseError: artifact parse failed at cat.rendering: '
                + 'expected a slot that stands exactly once in the words of its quote.',
            ],);
            expect(caught(function reads(): unknown {
              return asLocatedBySpan({
                rendering: { kind: 'located', quote: 'the cat naps', start: 0, slot: 'dozes', },
                path: 'cat.rendering',
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },);
            },),).toBeInstanceOf(ArtifactParseError,);
          },
        },),
      ],
    },),
    describe({
      name: reasonContradictedBySpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CONTRADICTS NOT-ASKED FOR A GROUP THE BENCH WAS ASKED ABOUT, and never contradicts no-english',
          fn: async () => {
            expect([
              reasonContradictedBySpan({ reason: 'not-asked', words: [], },),
              reasonContradictedBySpan({ reason: 'no-english', words: [], },),
              reasonContradictedBySpan({ reason: 'no-english', words: ['the cat naps',], },),
            ],).toEqual([true, false, false,],);
          },
        },),
        it({
          name: 'CONTRADICTS NO-AGREEMENT WHEN THE BALLOTS AGREED ON WORDS, and every other reason when they agreed on none',
          fn: async () => {
            const others: readonly UnlocatedReason[] = ['empty', 'not-found', 'ambiguous', 'markup', 'overlaps-another',];
            expect([
              reasonContradictedBySpan({ reason: 'no-agreement', words: [], },),
              reasonContradictedBySpan({ reason: 'no-agreement', words: ['the cat naps',], },),
              ...others.map(function contradicted(reason,): boolean {
                return reasonContradictedBySpan({ reason, words: [], },);
              },),
            ],).toEqual([false, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'CONTRADICTS EMPTY WHEN THE AGREED WORDS SHOW A READER SOMETHING, and any reason beside agreed words that show '
            + 'nothing, but not the reasons about where the words stand in an English the file does not hold',
          fn: async () => {
            const standing: readonly UnlocatedReason[] = ['not-found', 'ambiguous', 'markup', 'overlaps-another',];
            expect([
              reasonContradictedBySpan({ reason: 'empty', words: ['the cat naps',], },),
              reasonContradictedBySpan({ reason: 'empty', words: ['',], },),
              reasonContradictedBySpan({ reason: 'not-found', words: ['',], },),
              ...standing.map(function contradicted(reason,): boolean {
                return reasonContradictedBySpan({ reason, words: ['the cat naps',], },);
              },),
              reasonContradictedBySpan({ reason: 'slot-empty', words: ['the cat naps',], },),
            ],).toEqual([true, false, true, false, false, false, false, true,],);
            expect(reasonContradictedBySpan({ reason: 'empty', words: ['...',], },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: locatedContradictedBySpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS WORDS THAT SHOW A READER SOMETHING AND HOLD THEIR MARKUP IN PAIRS, with a slot that does the same and '
            + 'is not all of them, or with none',
          fn: async () => {
            expect([
              locatedContradictedBySpan({ rendering: { kind: 'located', quote: 'The cat naps', start: 0, }, },),
              locatedContradictedBySpan({
                rendering: { kind: 'located', quote: 'So this cat is a lazy little treasure', start: 0, slot: 'lazy', },
              },),
              locatedContradictedBySpan({
                rendering: { kind: 'located', quote: 'So this cat is a **lazy** treasure', start: 0, slot: 'a **lazy**', },
              },),
            ],).toEqual([false, false, false,],);
          },
        },),
        it({
          name: 'CONTRADICTS WORDS THAT SHOW A READER NOTHING OR CUT MARKUP, a slot that shows nothing, is all of the words or '
            + 'cuts markup',
          fn: async () => {
            expect([
              locatedContradictedBySpan({ rendering: { kind: 'located', quote: '  ', start: 0, }, },),
              locatedContradictedBySpan({ rendering: { kind: 'located', quote: 'on the sill** lulled', start: 0, }, },),
              locatedContradictedBySpan({
                rendering: { kind: 'located', quote: 'So this cat is a lazy treasure', start: 0, slot: '  ', },
              },),
              locatedContradictedBySpan({
                rendering: { kind: 'located', quote: 'lazy little treasure', start: 0, slot: 'lazy little treasure', },
              },),
              locatedContradictedBySpan({
                rendering: { kind: 'located', quote: 'So this cat is a **lazy** treasure', start: 0, slot: 'lazy** treasure', },
              },),
            ],).toEqual([true, true, true, true, true,],);
          },
        },),
      ],
    },),
  ],
},);
