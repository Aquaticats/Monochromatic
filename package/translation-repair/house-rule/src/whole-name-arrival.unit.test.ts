/**
 Tests whether a declared form that turns up in a replacement stands as a
 whole name (`whole-name-arrival.ts`): not run on across a joint, spelled as
 declared, and with no unexcused capitalised word fused beside it. Cat-themed
 invention throughout; the survival guard's own file tests the same rule
 through `findDroppedDeclaredNames`.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  arrivesAsWholeName,
  besideWordsOf,
  declaredWordsOf,
  excusedNeighboursOf,
  nameProjection,
  projectName,
} from '../dist/final/node/index.mjs';

/**
 Excused words for a case that has none beyond the ordinary ones.
 */
const ORDINARY_ONLY = excusedNeighboursOf({
  declaredWords: new Set<string>(),
  base: {
    before: [],
    after: [],
  },
},);

/**
 Whether a form arrives as a whole name in a text.

 @param text - candidate text

 @param form - declared form

 @param excused - words allowed beside it

 @returns Check's answer

 @example
 ```ts
 const whole = arrives({ text: 'Blossom naps.', form: 'Blossom', },);
 ```
 */
function arrives(
  {
    text,
    form,
    excused = ORDINARY_ONLY,
  }: {
    readonly text: string;
    readonly form: string;
    readonly excused?: typeof ORDINARY_ONLY;
  },
): boolean {
  return arrivesAsWholeName({
    projection: projectName({ text, },),
    form,
    key: nameProjection({ text: form, },),
    excused,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: arrivesAsWholeName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A FORM STANDING ALONE, in punctuation, in quotation marks or at either end of the text',
          fn: async () => {
            for (const text of [
              'Blossom naps.',
              'The cat (Blossom) naps.',
              '"Blossom" naps.',
              'The cat is Blossom',
              'Blossom',
            ]) {
              expect(arrives({ text, form: 'Blossom', },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'REFUSES A FORM RUN ON FROM OR INTO A LETTER, A DIGIT OR ACROSS A JOINING MARK, '
            + 'a Markdown-escaped underscore included',
          fn: async () => {
            for (const text of [
              'WoolyBlossom naps.',
              'BlossomWooly naps.',
              'Wooly-Blossom naps.',
              'Blossom-Wooly naps.',
              'Wooly_Blossom naps.',
              String.raw`Wooly\_Blossom naps.`,
              'Blossom2024 naps.',
              '2024Blossom naps.',
            ]) {
              expect(arrives({ text, form: 'Blossom', },),).toBe(false,);
            }
          },
        },),
        it({
          name: 'ACCEPTS A FORM BESIDE A JOINING MARK THAT HAS NO LETTER BEYOND IT, a dash or an underscore '
            + 'closing a phrase',
          fn: async () => {
            for (const text of [
              'Blossom - naps.',
              'Blossom_ naps.',
              '- Blossom naps.',
            ]) {
              expect(arrives({ text, form: 'Blossom', },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'REFUSES A SPELLING THE FORM WAS NOT DECLARED WITH, but takes case, a Markdown escape, '
            + 'a no-break space and a bracket at the end of the form',
          fn: async () => {
            expect(arrives({ text: 'Mi Mi naps.', form: 'Mimi', },),).toBe(false,);
            expect(arrives({ text: 'Wooly Cat naps.', form: 'Wooly_Cat', },),).toBe(false,);
            expect(arrives({ text: 'blossom naps.', form: 'Blossom', },),).toBe(true,);
            expect(arrives({ text: String.raw`Wooly\_Cat naps.`, form: 'Wooly_Cat', },),).toBe(true,);
            expect(arrives({ text: 'Wooly\u00A0Cat\u3000naps.', form: 'Wooly Cat', },),).toBe(true,);
            expect(arrives({ text: 'Wooly\n Cat naps.', form: 'Wooly Cat', },),).toBe(true,);
            expect(arrives({ text: 'Wooly\u{202F}Cat naps.', form: 'Wooly Cat', },),).toBe(true,);
            expect(arrives({ text: 'Blossom (Peanut) naps.', form: 'Blossom (Peanut)', },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES A CAPITALISED WORD FUSED ON BESIDE THE FORM, either side, through a space or a no-break space, '
            + 'unless the word is excused',
          fn: async () => {
            for (const text of [
              'Wooly Blossom naps.',
              'Blossom Wooly naps.',
              'Wooly\u00A0Blossom naps.',
              'The cat Wooly   Blossom naps.',
            ]) {
              expect(arrives({ text, form: 'Blossom', },),).toBe(false,);
            }
            expect(arrives({
              text: 'Wooly Blossom naps.',
              form: 'Blossom',
              excused: excusedNeighboursOf({
                declaredWords: new Set([ 'wooly', ],),
                base: {
                  before: [],
                  after: [],
                },
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'IGNORES A WORD THAT IS NOT CAPITALISED, a lower-case word, a word in capitals, a one-letter word '
            + 'or one with a digit, and any word across a punctuation mark',
          fn: async () => {
            for (const text of [
              'the Blossom naps.',
              'THE Blossom naps.',
              'I Blossom naps.',
              'Wooly, Blossom naps.',
              'Blossom, Wooly naps.',
              'Wooly. Blossom naps.',
              'Blossom naps Wooly.',
              'Wooly2 Blossom naps.',
            ]) {
              expect(arrives({ text, form: 'Blossom', },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'ACCEPTS WHEN ANY ONE OCCURRENCE STANDS WHOLE, since the other may be a fusion the candidate also '
            + 'wrote',
          fn: async () => {
            expect(arrives({ text: 'Wooly Blossom and Blossom nap.', form: 'Blossom', },),).toBe(true,);
          },
        },),
        it({
          name: 'LETS AN ORDINARY OPENER STAND BESIDE THE FORM but not a word left off the list',
          fn: async () => {
            expect(arrives({ text: 'When Blossom naps.', form: 'Blossom', },),).toBe(true,);
            expect(arrives({ text: 'Blossom Dear naps.', form: 'Blossom', },),).toBe(true,);
            expect(arrives({ text: 'Xing Blossom naps.', form: 'Blossom', },),).toBe(false,);
          },
        },),
        it({
          name: 'READS NO NEIGHBOUR BESIDE A HAN FORM, since Chinese is written without spaces',
          fn: async () => {
            expect(arrives({ text: 'Wooly \u732B\u5C0F\u59D0 naps.', form: '\u732B\u5C0F\u59D0', },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: besideWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE CAPITALISED WORDS BEFORE AND AFTER EACH OCCURRENCE, lower-cased',
          fn: async () => {
            expect(besideWordsOf({
              projection: projectName({ text: 'Aoba Mittens naps. Mittens Quill waits. Mittens', },),
              key: 'mittens',
            },),).toEqual({
              before: [ 'aoba', ],
              after: [ 'quill', ],
            },);
          },
        },),
        it({
          name: 'NAMES NOTHING WHERE NO CAPITALISED WORD STANDS BESIDE, through punctuation or a break',
          fn: async () => {
            expect(besideWordsOf({
              projection: projectName({ text: 'Mittens, Aoba. naps\nMittens', },),
              key: 'mittens',
            },),).toEqual({
              before: [],
              after: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: declaredWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS EACH FORM AT ANYTHING THAT IS NO LETTER OR DIGIT, lower-cased, once each',
          fn: async () => {
            expect([ ...declaredWordsOf({ forms: [ 'Wooly Blossom', 'Blossom (Peanut)', 'Wooly_Cat2', '', '--', ], },), ],)
              .toEqual([ 'wooly', 'blossom', 'peanut', 'cat2', ],);
          },
        },),
      ],
    },),

    describe({
      name: excusedNeighboursOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS THE DECLARED WORDS, THE ORDINARY WORDS AND THE BASE\'S OWN WORDS, side by side',
          fn: async () => {
            const excused = excusedNeighboursOf({
              declaredWords: new Set([ 'wooly', ],),
              base: {
                before: [ 'aoba', ],
                after: [ 'quill', ],
              },
            },);
            expect(excused.before.has('wooly',),).toBe(true,);
            expect(excused.before.has('when',),).toBe(true,);
            expect(excused.before.has('aoba',),).toBe(true,);
            expect(excused.before.has('quill',),).toBe(false,);
            expect(excused.after.has('quill',),).toBe(true,);
            expect(excused.after.has('aoba',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
