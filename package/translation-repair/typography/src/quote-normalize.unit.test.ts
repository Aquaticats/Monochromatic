/**
 Tests for punctuation normalization:
 every mapped variant collapses to its canonical character while
 length is preserved unit for unit, so offsets found in normalized
 text index the original exactly. The typography fold straightens curly
 quotes only and leaves the corner brackets and the no-break space alone
 (ledger B24).

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  collapseLineBreaks,
  normalizePunctuation,
  straightenProseQuotes,
  straightenQuotes,
} from '../dist/final/node/index.mjs';

/**
 Variant-to-canonical pairs the normalizer must collapse.
 */
const VARIANT_CASES = [
  ['‘', "'",],
  ['’', "'",],
  ['“', '"',],
  ['”', '"',],
  ['「', '"',],
  ['」', '"',],
  ['『', "'",],
  ['』', "'",],
  [' ', ' ',],
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: normalizePunctuation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        ...VARIANT_CASES.map(function toCase([variant, canonical,],) {
          return it({
            name: `maps ${JSON.stringify(variant,)} onto ${JSON.stringify(canonical,)}`,
            fn: async () => {
              expect(normalizePunctuation({ text: variant, },),).toBe(canonical,);
            },
          },);
        },),
        it({
          name: 'leaves ASCII and CJK text unchanged',
          fn: async () => {
            expect(normalizePunctuation({ text: "the cat's 猫窝 [^1]", },),)
              .toBe("the cat's 猫窝 [^1]",);
          },
        },),
        it({
          name: 'preserves length over mixed text so offsets transfer',
          fn: async () => {
            /**
             Mixed sample with every variant class plus surrounding prose.
             */
            const mixed = '老猫说：“打盹最舒服。”小猫写『喵』，又写「喵」，还写‘喵’和’喵‘。';
            expect(normalizePunctuation({ text: mixed, },),).toHaveLength(mixed.length,);
          },
        },),
      ],
    },),

    describe({
      name: collapseLineBreaks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a soft wrap as one space',
          fn: async () => {
            expect(collapseLineBreaks({ text: '小猫打盹，\n阳光很暖和。', },),).toBe(
              '小猫打盹， 阳光很暖和。',
            );
          },
        },),
        it({
          name: 'leaves a blank line as two spaces, so a joined quote still misses',
          fn: async () => {
            expect(collapseLineBreaks({ text: '第一段。\n\n第二段。', },),).toBe(
              '第一段。  第二段。',
            );
          },
        },),
        it({
          name: 'reads a carriage return as a space too',
          fn: async () => {
            expect(collapseLineBreaks({ text: '小猫打盹，\r\n阳光很暖和。', },),).toBe(
              '小猫打盹，  阳光很暖和。',
            );
          },
        },),
        it({
          name: 'preserves length so offsets transfer',
          fn: async () => {
            /**
             Sample mixing wraps, a blank line, and punctuation left alone.
             */
            const mixed = '小猫打盹，\n阳光很暖和。\n\n老猫说：“喵。”\n';
            expect(collapseLineBreaks({ text: mixed, },),).toHaveLength(mixed.length,);
          },
        },),
        it({
          name: 'leaves punctuation variants alone, unlike normalizePunctuation',
          fn: async () => {
            expect(collapseLineBreaks({ text: '老猫说：“喵。”', },),).toBe('老猫说：“喵。”',);
          },
        },),
      ],
    },),

    describe({
      name: straightenQuotes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STRAIGHTENS EVERY CURLY QUOTE, as the typography restoration would read it',
          fn: async () => {
            expect(straightenQuotes({ text: 'the cat’s ‘nap’ and “purr”', },),).toBe('the cat\'s \'nap\' and "purr"',);
          },
        },),
        it({
          name: 'LEAVES THE CORNER BRACKETS AND THE NO-BREAK SPACE ALONE, which the evidence fold folds',
          fn: async () => {
            /**
             Text carrying every mark the evidence fold maps and this one does not.
             */
            const kept = '「猫」『喵』\u{00A0}nap';
            expect(straightenQuotes({ text: kept, },),).toBe(kept,);
            expect(normalizePunctuation({ text: kept, },),).not.toBe(kept,);
          },
        },),
      ],
    },),

    describe({
      name: straightenProseQuotes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STRAIGHTENS THE QUOTES OF PROSE ONLY, leaving a backtick span and a tag as written, where the '
            + 'typography restoration never reaches',
          fn: async () => {
            expect(straightenProseQuotes({ text: 'the cat’s “nap”, then `say “purr”`', },),)
              .toBe('the cat\'s "nap", then `say “purr”`',);
            expect(straightenProseQuotes({ text: '<Cat name="“Mittens”">she didn’t</Cat>', },),)
              .toBe('<Cat name="“Mittens”">she didn\'t</Cat>',);
            // A tag with a curly attribute quote is no tag the grammar reads, so the text has no prose at all.
            expect(straightenProseQuotes({ text: '<Cat name=“Mittens”>she didn’t</Cat>', },),)
              .toBe('<Cat name=“Mittens”>she didn’t</Cat>',);
          },
        },),
        it({
          name: 'KEEPS THE LENGTH AND EVERY CHARACTER PAST THE FIRST PLANE, so an offset still indexes the input',
          fn: async () => {
            /**
             Prose with an astral ideograph beside a curly apostrophe.
             */
            const text = '𠀀’s “nap”';
            /**
             What the fold makes of it.
             */
            const folded = straightenProseQuotes({ text, },);
            expect(folded,).toBe('𠀀\'s "nap"',);
            expect(folded.length,).toBe(text.length,);
          },
        },),
      ],
    },),
  ],
},);
