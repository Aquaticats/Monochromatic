/**
 Tests for the readers of an English's emphasis that read it with the page's link and footnote definitions: a reference
 whose definition stands in another slice is a link there, and an unresolved `[b*]` is plain text whose `*` can pair with an
 outer one, so the same English pairs its delimiters one way without the definition and another way with it (n173).

 Each case is an English whose pairing differs with and without a definition, and each result with the definition given is
 the result for the same text with the definition written into it.

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
  apostropheMarksOf,
  holdsEdgeApostrophe,
  joinedMarksOf,
  looseMarksOf,
  markReadingsInEnglish,
  marksPastMarkupOf,
  planConsistencyEdit,
  proofViewOf,
  provenApostrophesInEnglish,
  readingsChangeOutside,
  withApostropheForm,
} from '../dist/final/node/index.mjs';
import { renderingsOf, } from './consistency-renderings.test-fixture.ts';

/**
 The definition of the shortcut reference `[b*]`, which stands in another slice of the page.
 */
const DEFINITION = '[b*]: https://cats.example/b';

/**
 The definition of the shortcut reference `[cat*]`.
 */
const CAT_DEFINITION = '[cat*]: https://cats.example/c';

/**
 An English that closes its emphasis with a `*` after the reference `[b*]`, then an apostrophe and a letter: with the
 definition the `*` inside the reference is no delimiter, so the last `*` closes the emphasis and the apostrophe joins the
 letter after it.
 */
const JOINED = '*a [b*] c*\'s';

/**
 The same pairing with a typographic apostrophe past the closing mark and a word after it.
 */
const PAST = '*a [b*] c*’ x';

/**
 The same pairing with the typographic apostrophe last, at the edge of the words.
 */
const EDGE = '*a [b*] c*’';

/**
 An English whose last word closes an emphasis that the reference `[b*]` stands inside.
 */
const SENTENCE = 'x *a [b*] cat*\'s bowl';

/**
 An English with a plural possessive past the closing mark and the reference inside the emphasis.
 */
const POSSESSIVE = 'x *a [b*] cats*’ bowl';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: joinedMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE EMPHASIS WITH THE PAGE\'S DEFINITIONS: the apostrophe after the last `*` joins the letter after it when the '
            + 'reference inside the emphasis is defined elsewhere on the page, and joins none when it is not',
          fn: async () => {
            expect([
              [...joinedMarksOf({ english: JOINED, },),],
              [...joinedMarksOf({ english: JOINED, definitions: DEFINITION, },),],
              [...joinedMarksOf({ english: `${JOINED}\n\n${DEFINITION}`, },),].filter(function inEnglish(at,): boolean {
                return at < JOINED.length;
              },),
            ],).toEqual([[], [10,], [10,],],);
          },
        },),
      ],
    },),
    describe({
      name: marksPastMarkupOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE EMPHASIS WITH THE PAGE\'S DEFINITIONS: the mark after the last `*` stands past closing markup when the '
            + 'reference inside the emphasis is defined elsewhere on the page, and does not when it is not',
          fn: async () => {
            expect([
              [...marksPastMarkupOf({ english: PAST, },).trailing,],
              [...marksPastMarkupOf({ english: PAST, definitions: DEFINITION, },).trailing,],
              [...marksPastMarkupOf({ english: `${PAST}\n\n${DEFINITION}`, },).trailing,].filter(function inEnglish(at,): boolean {
                return at < PAST.length;
              },),
            ],).toEqual([[], [10,], [10,],],);
          },
        },),
      ],
    },),
    describe({
      name: holdsEdgeApostrophe.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE WORDS\' EMPHASIS WITH THE PAGE\'S DEFINITIONS: a mark last in the words ends a reading run of closing '
            + 'emphasis when the reference inside it is defined elsewhere on the page, and is no edge apostrophe when it is not',
          fn: async () => {
            expect([
              holdsEdgeApostrophe({ text: EDGE, side: 'end', },),
              holdsEdgeApostrophe({ text: EDGE, side: 'end', definitions: DEFINITION, },),
              holdsEdgeApostrophe({ text: `${DEFINITION}\n\n${EDGE}`, side: 'end', },),
            ],).toEqual([false, true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: looseMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE WORDS\' EMPHASIS WITH THE PAGE\'S DEFINITIONS: the mark that ends a reading run of closing emphasis is part '
            + 'of its word and no loose mark, and is a loose mark when the definition is not given',
          fn: async () => {
            expect([
              looseMarksOf({ text: EDGE, },),
              looseMarksOf({ text: EDGE, definitions: DEFINITION, },),
              looseMarksOf({ text: `${DEFINITION}\n\n${EDGE}`, },),
            ],).toEqual([['\'',], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: apostropheMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE WORDS\' EMPHASIS WITH THE PAGE\'S DEFINITIONS: the mark past closing emphasis is an apostrophe of a word '
            + 'when the definition is given, and none when it is not',
          fn: async () => {
            expect([
              apostropheMarksOf({ text: EDGE, },),
              apostropheMarksOf({ text: EDGE, definitions: DEFINITION, },),
              apostropheMarksOf({ text: `${DEFINITION}\n\n${EDGE}`, },),
            ],).toEqual([[], ['’',], ['’',],],);
          },
        },),
      ],
    },),
    describe({
      name: withApostropheForm.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE WORDS\' EMPHASIS WITH THE PAGE\'S DEFINITIONS: the apostrophe of a word past closing emphasis is written in '
            + 'the form asked when the definition is given, and left as it stands when it is not',
          fn: async () => {
            /**
             The words with a straight apostrophe past the closing emphasis.
             */
            const text = '*a [b*] c*\'';
            expect([
              withApostropheForm({ text, form: '’', },),
              withApostropheForm({ text, form: '’', definitions: DEFINITION, },),
              withApostropheForm({ text: `${DEFINITION}\n\n${text}`, form: '’', },).slice(DEFINITION.length + 2,),
            ],).toEqual(['*a [b*] c*\'', '*a [b*] c*’', '*a [b*] c*’',],);
          },
        },),
      ],
    },),
    describe({
      name: proofViewOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE ENGLISH WITH THE PAGE\'S DEFINITIONS, and remembers a view for the English together with the definitions '
            + 'it was read with: the same English asked without, with and without again gives the reading of each',
          fn: async () => {
            /**
             The marks read as joined to the word after them, for an English and the definitions it is read with.
             */
            function joinedFor({ english, definitions, }: { readonly english: string; readonly definitions?: string; },): number[] {
              return [...proofViewOf({ english, ...((definitions === undefined) ? {} : { definitions, }), },).joined,];
            }
            expect([
              joinedFor({ english: SENTENCE, },),
              joinedFor({ english: SENTENCE, definitions: DEFINITION, },),
              joinedFor({ english: SENTENCE, },),
              joinedFor({ english: `${SENTENCE}\n\n${DEFINITION}`, },),
            ],).toEqual([[], [14,], [], [14,],],);
          },
        },),
      ],
    },),
    describe({
      name: provenApostrophesInEnglish.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE ENGLISH WITH THE PAGE\'S DEFINITIONS: the plural possessive past the closing emphasis is proven when the '
            + 'reference inside the emphasis is defined elsewhere on the page, and is not when it is not',
          fn: async () => {
            expect([
              [...provenApostrophesInEnglish({ english: POSSESSIVE, },).trailing,],
              [...provenApostrophesInEnglish({ english: POSSESSIVE, definitions: DEFINITION, },).trailing,],
              [...provenApostrophesInEnglish({ english: `${POSSESSIVE}\n\n${DEFINITION}`, },).trailing,],
            ],).toEqual([[], [15,], [15,],],);
          },
        },),
      ],
    },),
    describe({
      name: markReadingsInEnglish.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE ENGLISH WITH THE PAGE\'S DEFINITIONS: the apostrophe after the closing emphasis reads as a joint when the '
            + 'reference inside the emphasis is defined elsewhere on the page, and as a leading mark when it is not',
          fn: async () => {
            expect([
              [...markReadingsInEnglish({ english: SENTENCE, },),],
              [...markReadingsInEnglish({ english: SENTENCE, definitions: DEFINITION, },),],
              [...markReadingsInEnglish({ english: `${SENTENCE}\n\n${DEFINITION}`, },),],
            ],).toEqual([[[14, 'leading',],], [[14, 'joint',],], [[14, 'joint',],],],);
          },
        },),
      ],
    },),
    describe({
      name: readingsChangeOutside.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE MARKS OUTSIDE THE SPAN WITH THE PAGE\'S DEFINITIONS: words that make a defined reference an undefined one '
            + 'change the pairing of the emphasis around it, so the apostrophe after it reads another way, which the English alone '
            + 'cannot show',
          fn: async () => {
            /**
             The English: a reference the page defines inside an emphasis that an apostrophe follows.
             */
            const english = '*a [cat*] d*\'s bowl';
            /**
             Where the words that the replacement takes out start.
             */
            const start = english.indexOf('cat',);
            /**
             The change `dog` makes, for the English and the definitions it is read with.
             */
            function changesFor({ read, definitions, }: { readonly read: string; readonly definitions?: string; },): boolean {
              return readingsChangeOutside({
                english: read,
                start,
                end: start + 3,
                written: 'dog',
                ...((definitions === undefined) ? {} : { definitions, }),
              },);
            }
            expect([
              changesFor({ read: english, },),
              changesFor({ read: english, definitions: CAT_DEFINITION, },),
              changesFor({ read: `${english}\n\n${CAT_DEFINITION}`, },),
            ],).toEqual([false, true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: planConsistencyEdit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE ENGLISH AROUND THE SPLICE WITH THE PAGE\'S DEFINITIONS: words that make a defined reference an undefined one '
            + 'are refused where the page defines it, and planned where the English alone shows no definition',
          fn: async () => {
            /**
             The receiving place's English: a reference the page defines inside an emphasis that an apostrophe follows.
             */
            const english = '*a [cat*] d*\'s bowl';
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dog', quote: 'dog', },
              receiving: { english, quote: 'cat', },
            },);
            /**
             What the planner makes of the splice for the definitions it is given.
             */
            function kindFor({ definitions, }: { readonly definitions?: string; },): string {
              return planConsistencyEdit({
                chosen,
                target,
                framed: false,
                marks: 'compared',
                english,
                ...((definitions === undefined) ? {} : { definitions, }),
              },).kind;
            }
            expect([
              kindFor({},),
              kindFor({ definitions: CAT_DEFINITION, },),
            ],).toEqual(['replace', 'unavailable',],);
          },
        },),
      ],
    },),
  ],
},);
