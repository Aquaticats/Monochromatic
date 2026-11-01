/**
 Tests for how the words of a located place are read against the page's link and footnote definitions. The words are read as
 the span's own text, and a definition cannot supply a delimiter that stands outside the span: in `Take *[*]a*’ now.` with `[*]`
 defined, the `*` after `a` closes the emphasis the `*` before `[*]` opened, so the page reads the mark after it as an apostrophe
 of its word, the span `[*]a*’` reads the same alone, and reading that span with the definition makes the `*` inside `[*]` a plain
 character and the `*` after `a` a closer of nothing, so the mark reads loose against the page. The cases pin that a place of a
 run records no definitions for its words, and that a locator given definitions proves the apostrophe beside the words with them.

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
  holdsEdgeApostrophe,
  locateWords,
  markReadingsInEnglish,
  withProvenApostrophes,
} from '../dist/final/node/index.mjs';
import { renderingsOf, } from './consistency-renderings.test-fixture.ts';

/**
 The definition of the reference `[*]`, which stands in another slice of the page.
 */
const STAR_DEFINITION = '[*]: https://cats.example/star';

/**
 The English whose emphasis opens before the reference and closes after the word that follows it.
 */
const ENGLISH = 'Take *[*]a*’ now.';

/**
 The definition of the reference `[b*]`, which stands in another slice of the page.
 */
const DEFINITION = '[b*]: https://cats.example/b';

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: 'the words of a located place',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS NO DEFINITIONS FOR THE WORDS OF A PLACE OF A RUN, though the span reads another way as a fragment with the '
            + 'definition than alone: the opener of the emphasis stands outside the span, so the page reads the mark as an apostrophe '
            + 'of its word and the span read alone agrees with the page, which the span read with the definition does not',
          fn: async () => {
            const [, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs’ bowl.', quote: 'the dogs’', },
              receiving: { english: ENGLISH, quote: '[*]a*’', page: `${ENGLISH}\n\n${STAR_DEFINITION}`, },
            },);
            expect([
              target.core,
              target.start,
              target.definitions,
              holdsEdgeApostrophe({ text: target.core, side: 'end', },),
              holdsEdgeApostrophe({ text: target.core, side: 'end', definitions: STAR_DEFINITION, },),
              [...markReadingsInEnglish({ english: ENGLISH, definitions: STAR_DEFINITION, },).values(),],
            ],).toEqual(['[*]a*’', 6, undefined, true, false, ['trailing',],],);
          },
        },),
      ],
    },),
    describe({
      name: 'the locator given definitions',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PROVES THE APOSTROPHE BESIDE THE WORDS WITH THE DEFINITIONS IT IS GIVEN, not only the word test: the `*` after `c` closes '
            + 'the emphasis the `*` before `a` opened once `[b*]` is a reference, so the mark after it is an apostrophe of the word and '
            + 'the span takes it in; read alone the mark is free and the span ends before it',
          fn: async () => {
            /**
             The English the words stand in.
             */
            const haystack = 'Later *a [b*] c*’ naps.';
            expect([
              locateWords({ haystack, words: 'a [b*] c*', },),
              locateWords({ haystack, words: 'a [b*] c*', definitions: DEFINITION, },),
              withProvenApostrophes({ haystack, start: 7, end: 16, },),
              withProvenApostrophes({ haystack, start: 7, end: 16, definitions: DEFINITION, },),
            ],).toEqual([
              { kind: 'once', start: 7, end: 16, },
              { kind: 'once', start: 7, end: 17, },
              { start: 7, end: 16, },
              { start: 7, end: 17, },
            ],);
          },
        },),
        it({
          name: 'PROVES A LEADING APOSTROPHE WITH THE DEFINITIONS IT IS GIVEN: the mark after `c*` before `s` is a leading mark when '
            + '`[b*]` is plain text, which the span takes in, and a joint of its word when it is a reference, which it leaves out',
          fn: async () => {
            /**
             The English: the words `s bowl` begin right after the mark.
             */
            const haystack = '*a [b*] c*’s bowl';
            expect([
              withProvenApostrophes({ haystack, start: 11, end: 17, },),
              withProvenApostrophes({ haystack, start: 11, end: 17, definitions: DEFINITION, },),
            ],).toEqual([{ start: 10, end: 17, }, { start: 11, end: 17, },],);
          },
        },),
      ],
    },),
  ],
},);
