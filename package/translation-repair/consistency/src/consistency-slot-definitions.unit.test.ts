/**
 Tests for the page definitions that move the slot of a span: none where the page's definitions change nothing or the place
 carries none, the definition of the reference a slot sits beside, and no more of the page's definitions than that.

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
  type ConsistencyVoice,
  definitionsMovingSlotOf,
  foldedWordsOf,
  quotesByLabelOf,
} from '../dist/final/node/index.mjs';

/**
 The definitions that move the slot `slot` of the span `span`, when the ballots all gave it beside the span.

 @param span - span of the first place, as its English holds it

 @param slot - slot every ballot gave

 @param definitions - definitions of the page

 @returns What `definitionsMovingSlotOf` gives

 @example
 ```ts
 const moving = movingFor({ span: 'a[b]c', slot: 'b', definitions: '[b]: https://x.example/b', },);
 ```
 */
function movingFor(
  {
    span,
    slot,
    definitions,
  }: {
    readonly span: string;
    readonly slot: string;
    readonly definitions: string;
  },
): string {
  /**
   Three ballots that gave the slot beside the span.
   */
  const voices: readonly ConsistencyVoice[] = ['a', 'b', 'c',].map(function ballotFor(seat,): ConsistencyVoice {
    return {
      seat,
      reason: 'It words the first place best.',
      quotes: [{ label: 'OCCURRENCE 1', quote: span, slot, },],
      choice: 'OCCURRENCE 1',
      keep: [],
    };
  },);
  return definitionsMovingSlotOf({
    voices,
    label: 'OCCURRENCE 1',
    words: foldedWordsOf({ text: span, },),
    span,
    quotesByLabel: quotesByLabelOf({ voices, },),
    definitions,
  },);
}

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: definitionsMovingSlotOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES NONE WHERE THE PAGE HAS NO DEFINITIONS OR THEY CHANGE NOTHING for the span: the definition of another reference '
            + 'leaves `a[b]c` with its slot, and a page with none leaves the span read alone',
          fn: async () => {
            expect([
              movingFor({ span: 'a[b]c', slot: 'b', definitions: '', },),
              movingFor({ span: 'a[b]c', slot: 'b', definitions: '[z]: https://x.example/z', },),
              movingFor({ span: 'the cat naps', slot: 'naps', definitions: '[z]: https://x.example/z', },),
            ],).toEqual(['', '', '',],);
          },
        },),
        it({
          name: 'GIVES THE DEFINITION OF THE REFERENCE THE SLOT SITS BESIDE, which makes `a[b]c` one word and the slot `b` no stand',
          fn: async () => {
            expect(movingFor({ span: 'a[b]c', slot: 'b', definitions: '[b]: https://x.example/b', },),).toBe(
              '[b]: https://x.example/b',
            );
          },
        },),
        it({
          name: 'GIVES NO MORE OF THE PAGE\'S DEFINITIONS THAN THE SLOT NEEDS, in the order of the page, whole entries among them: a '
            + 'page with a footnote of two paragraphs and the definitions of other references records the one definition',
          fn: async () => {
            /**
             The definitions of a page: the definitions of three references, and a footnote of two paragraphs after them.
             */
            const definitions = '[x]: https://x.example/x\n[b]: https://x.example/b\n[z]: https://x.example/z\n'
              + '[^1]: First paragraph.\n\n    Second paragraph.';
            expect([
              movingFor({ span: 'a[b]c', slot: 'b', definitions, },),
              movingFor({ span: 'a[b]c [d]e', slot: 'b', definitions: `[d]: https://x.example/d\n${definitions}`, },),
            ],).toEqual(['[b]: https://x.example/b', '[b]: https://x.example/b',],);
          },
        },),
      ],
    },),
  ],
},);
