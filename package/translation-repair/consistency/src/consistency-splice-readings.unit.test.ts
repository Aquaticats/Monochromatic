/**
 Tests for asking whether a splice changes how a single mark outside the replaced words is read: attached, joined
 after closing markup, trailing, leading or free, before and after the words are put in.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { readFileSync, } from 'node:fs';
import { runInNewContext, } from 'node:vm';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bringsAtEdge,
  bringsEdgeApostrophe,
  gainsBesideKeptMark,
  holdsTwoMarksAtEdge,
  readingsChangeOutside,
} from '../dist/final/node/index.mjs';

/**
 Whether replacing the one stand of some words in an English changes how a mark outside them is read (a stand is a place
 where words lie in an English, defined in `consistency-fold.ts`).

 @param english - English before the splice

 @param words - words replaced, which stand once in it

 @param written - words that go in

 @returns Whether a mark outside the words is read another way afterwards

 @example
 ```ts
 const changed = changesOutside({ english: 'The cat’s bowl.', words: 'cat', written: 'kitten', },); // false
 ```
 */
function changesOutside(
  {
    english,
    words,
    written,
  }: {
    readonly english: string;
    readonly words: string;
    readonly written: string;
  },
): boolean {
  /**
   Where the words start in the English.
   */
  const start = english.indexOf(words,);
  return readingsChangeOutside({
    english,
    start,
    end: start + words.length,
    written,
  },);
}

/**
 The receiving place's located rendering that the gain check reads.
 */
type Target = Parameters<typeof gainsBesideKeptMark>[0]['target'];

/**
 A hand-built located rendering for a gain check: the receiving words and the marks the counter would have recorded beside
 and past them.

 @param core - receiving words, `kittens` by default

 @param over - marks the counter records, none by default

 @returns Rendering

 @example
 ```ts
 const target = locatedWith({ over: { markAfter: '’', }, },);
 ```
 */
function locatedWith(
  {
    core = 'kittens',
    over,
  }: {
    readonly core?: string;
    readonly over: Record<string, unknown>;
  },
): Target {
  return {
    kind: 'located',
    quote: core,
    core,
    start: 0,
    ...over,
  } as Target;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readingsChangeOutside.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN THE WORDS THAT GO IN CHANGE A NEIGHBOUR OF A MARK ACROSS THE NEIGHBOUR TEST OF THE CONSISTENCY '
            + 'PASS, so a mark that was free becomes joined or leading, and one beside a letter of another script is read '
            + 'as beside none',
          fn: async () => {
            expect([
              changesOutside({ english: 'She said ”’猫 now.', words: '猫', written: 'goin', },),
              changesOutside({ english: 'She said ”’5 cats now.', words: '5 cats', written: '猫', },),
              changesOutside({ english: 'He said cats 🐱\'s now.', words: 'cats 🐱', written: 'Come (here)', },),
              changesOutside({ english: 'He said cats 🐱\'s now.', words: 'cats 🐱', written: 'cats', },),
              changesOutside({ english: 'He said cat\'s now.', words: 'cat', written: '猫', },),
            ],).toEqual([true, true, true, true, true,],);
          },
        },),
        it({
          name: 'READS A COMPATIBILITY LETTER OF THE WORDS THAT GO IN AS A LETTER BESIDE A MARK, by the neighbour test of the '
            + 'consistency pass: a mathematical script name or a fullwidth letter changes how the mark before it is read, a micro '
            + 'sign (NFKC Greek) beside it does not',
          fn: async () => {
            expect([
              changesOutside({ english: 'She said ”’猫 now.', words: '猫', written: '\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}', },),
              changesOutside({ english: 'She said ”’猫 now.', words: '猫', written: '\u{FF21}', },),
              changesOutside({ english: 'She said ”’猫 now.', words: '猫', written: '\u{00B5}', },),
            ],).toEqual([true, true, false,],);
          },
        },),
        it({
          name: 'IS FALSE WHEN EVERY MARK OUTSIDE THE WORDS IS READ AS IT WAS, whatever the words that go in say: no mark, a mark far '
            + 'from the words, a neighbour of the same kind and a stretch of another script for another',
          fn: async () => {
            expect([
              changesOutside({ english: 'The cat naps.', words: 'cat', written: 'kitten', },),
              changesOutside({ english: 'Mio’s dish. The cat naps.', words: 'cat', written: 'kitten', },),
              changesOutside({ english: 'She said ”’猫 now.', words: '猫', written: '狗', },),
              changesOutside({ english: 'He said cats 🐱\'s now.', words: 'cats 🐱', written: 'cats 🐶', },),
              changesOutside({ english: 'The cats’ bowl.', words: 'cats’', written: 'dogs’', },),
            ],).toEqual([false, false, false, false, false,],);
          },
        },),
        it({
          name: 'READS EVERY MARK OF THE ENGLISH WITH THE PARSE, so words that put markup in can change how a mark far from them '
            + 'is read, and a lone asterisk is a character that reads no apostrophe as joined',
          fn: async () => {
            expect([
              changesOutside({ english: 'The *cat naps. Then the dog*\'s bowl.', words: 'cat', written: 'kitten*', },),
              changesOutside({ english: 'The cat*\'s bowl.', words: 'cat', written: 'kitten', },),
              changesOutside({ english: 'The *cat*\'s bowl.', words: 'cat', written: 'kitten', },),
            ],).toEqual([true, false, false,],);
          },
        },),
        it({
          name: 'READS A MARK PAST CLOSING MARKUP BY THE LETTER ON THE FAR SIDE OF THE MARKUP, so words of another script put in '
            + 'before closing markup change how the mark is read, and words of the same kind do not; a mark before opening '
            + 'markup is read where it stands, so words after the markup change nothing',
          fn: async () => {
            expect([
              changesOutside({ english: 'The *kittens*’ bowl.', words: 'kittens', written: '猫', },),
              changesOutside({ english: 'The *kittens*’ bowl.', words: 'kittens', written: 'dogs', },),
              changesOutside({ english: 'He said ’*tis* now.', words: 'tis', written: '猫', },),
              changesOutside({ english: 'He said ’*tis* now.', words: 'tis', written: 'twas', },),
            ],).toEqual([true, false, false, false,],);
          },
        },),
        it({
          name: 'COMPARES THE MARKS AFTER THE WORDS AT THEIR OFFSET AFTER THE SPLICE, so words of another length shift them '
            + 'without changing what they read as',
          fn: async () => {
            expect([
              changesOutside({ english: 'The cat’s bowl, the pups’ bowl.', words: 'cat', written: 'a very small kitten', },),
              changesOutside({ english: 'The ’cat and the pups’ bowl.', words: 'cat', written: 'kitten', },),
            ],).toEqual([false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: bringsAtEdge.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN THE WORDS THAT GO IN HOLD AN APOSTROPHE AT THE EDGE ASKED ABOUT AND THE RECEIVING WORDS HOLD NONE THERE, '
            + 'at the end and at the start, directly beside a letter or, at the end, past closing emphasis beside one, and false '
            + 'when the receiving words hold an edge apostrophe there',
          fn: async () => {
            expect([
              bringsAtEdge({ text: 'the dogs’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: '’round the cats', core: 'around the cats', side: 'start', },),
              bringsAtEdge({ text: 'the dogs’', core: '猫’', side: 'end', },),
              bringsAtEdge({ text: 'the *dogs*’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: 'the *dogs*’', core: '*kittens*’', side: 'end', },),
            ],).toEqual([true, true, true, true, false,],);
          },
        },),
        it({
          name: 'IS TRUE FOR A SINGLE MARK AT THE EDGE ASKED ABOUT WHATEVER STANDS INSIDE IT, the ruling on gains: a mark before '
            + 'opening markup, after an underscore that closes nothing, beside a space or beside a pictograph arrives as a free '
            + 'mark the page must prove, and the receiving words\' own edge apostrophe is what makes it no gain',
          fn: async () => {
            expect([
              bringsAtEdge({ text: '’*round* the cats', core: 'around the cats', side: 'start', },),
              bringsAtEdge({ text: '_cats_ʼ', core: 'ʼcats 🐱', side: 'end', },),
              bringsAtEdge({ text: '_cats_ʼ', core: 'ʼcats 🐱', side: 'start', },),
              bringsAtEdge({ text: 'the cats ʼ', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: 'the 🐱’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: 'the 猫’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: '_cats_ʼ', core: 'kittens’', side: 'end', },),
              bringsAtEdge({ text: '_cats_', core: 'kittens', side: 'end', },),
            ],).toEqual([true, true, false, true, true, true, false, false,],);
          },
        },),
        it({
          name: 'READS AN APOSTROPHE BESIDE A COMPATIBILITY LETTER AS AT THE EDGE OF A WORD, by the neighbour test of the '
            + 'consistency pass: after a mathematical script name or before one it is brought, and after a micro sign a mark is '
            + 'brought too (the ruling on gains: any single mark at the edge, whatever stands inside it)',
          fn: async () => {
            expect([
              bringsAtEdge({ text: 'the \u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: '’\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7} bowl', core: 'around the cats', side: 'start', },),
              bringsAtEdge({ text: 'the \u{FF21}’', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: 'the \u{00B5}’', core: 'kittens', side: 'end', },),
            ],).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'IS FALSE WHEN THE RECEIVING WORDS ALREADY HOLD AN APOSTROPHE AT THAT EDGE, since the mark is replaced where it stands '
            + 'and none is brought',
          fn: async () => {
            expect([
              bringsAtEdge({ text: 'the dogs’', core: 'kittens’', side: 'end', },),
              bringsAtEdge({ text: '’round the cats', core: '’round', side: 'start', },),
            ],).toEqual([false, false,],);
          },
        },),
        it({
          name: 'IS FALSE WHEN THE WORDS THAT GO IN HOLD NO SINGLE MARK AT THE EDGE ASKED ABOUT, whatever the receiving words hold '
            + 'there or at the other edge',
          fn: async () => {
            expect([
              bringsAtEdge({ text: 'the dogs', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: 'the dogs’', core: 'kittens', side: 'start', },),
              bringsAtEdge({ text: 'the dogs', core: 'kittens’', side: 'end', },),
              bringsAtEdge({ text: 'the 猫', core: 'kittens', side: 'end', },),
              bringsAtEdge({ text: '', core: '', side: 'start', },),
            ],).toEqual([false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: gainsBesideKeptMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN THE WORDS BRING AN EDGE APOSTROPHE AND ANY SINGLE MARK STANDS DIRECTLY OUTSIDE THE SPAN ON THAT SIDE, '
            + 'whatever the mark reads as: a closing mark, an opening mark, a straight or a curly one',
          fn: async () => {
            expect([
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: { markAfter: '’', }, },), },),
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: { markAfter: '\'', }, },), },),
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: { markAfter: '‘', }, },), },),
              gainsBesideKeptMark({ text: '’round the cats', target: locatedWith({ over: { markBefore: '\'', }, },), },),
              gainsBesideKeptMark({ text: '’round the cats', target: locatedWith({ over: { markBefore: '‘', }, },), },),
            ],).toEqual([true, true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE WHEN THE WORDS BRING AN EDGE APOSTROPHE AND A SINGLE MARK STANDS A WIDE RUN AWAY ON THAT SIDE, past markup or '
            + 'a bracket',
          fn: async () => {
            expect([
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: { markPastAfter: '’', }, },), },),
              gainsBesideKeptMark({ text: '’round the cats', target: locatedWith({ over: { markPastBefore: '\'', }, },), },),
            ],).toEqual([true, true,],);
          },
        },),
        it({
          name: 'IS FALSE WHEN NO SINGLE MARK STANDS OUTSIDE THE SPAN ON THE GAIN SIDE, whatever stands on the other side, and when '
            + 'the words bring no edge apostrophe, whatever stands outside',
          fn: async () => {
            expect([
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: {}, },), },),
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: { markBefore: '\'', markPastBefore: '‘', }, },), },),
              gainsBesideKeptMark({ text: '’round the cats', target: locatedWith({ over: { markAfter: '’', markPastAfter: '’', }, },), },),
              gainsBesideKeptMark({ text: 'the dogs', target: locatedWith({ over: { markAfter: '’', markPastAfter: '’', }, },), },),
            ],).toEqual([false, false, false, false,],);
          },
        },),
        it({
          name: 'IS TRUE WHEN THE WORDS CARRY TWO SINGLE MARKS AT AN EDGE AND THE RECEIVING WORDS DO NOT, since one of the pair would '
            + 'stand outside the span as a doubled mark; it is false when the receiving words carry the pair too',
          fn: async () => {
            expect([
              gainsBesideKeptMark({ text: 'the dogs’’', target: locatedWith({ over: {}, },), },),
              gainsBesideKeptMark({ text: '\'\'cats', target: locatedWith({ over: {}, },), },),
              gainsBesideKeptMark({ text: 'the dogs’’', target: locatedWith({ core: 'kittens’’', over: {}, },), },),
              gainsBesideKeptMark({ text: 'the dogs’', target: locatedWith({ over: {}, },), },),
            ],).toEqual([true, true, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: holdsTwoMarksAtEdge.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR TWO SINGLE MARKS SIDE BY SIDE AT THE EDGE ASKED ABOUT, of any two of the four spellings, and false for '
            + 'one mark, a mark and a letter, a pair at the other edge or a text too short to hold two',
          fn: async () => {
            expect([
              holdsTwoMarksAtEdge({ text: 'the dogs’’', side: 'end', },),
              holdsTwoMarksAtEdge({ text: '\'’cats', side: 'start', },),
              holdsTwoMarksAtEdge({ text: 'the dogs\u{02BC}‘', side: 'end', },),
              holdsTwoMarksAtEdge({ text: 'the dogs’', side: 'end', },),
              holdsTwoMarksAtEdge({ text: 'the dogs’s', side: 'end', },),
              holdsTwoMarksAtEdge({ text: 'the dogs’’', side: 'start', },),
              holdsTwoMarksAtEdge({ text: '’', side: 'start', },),
              holdsTwoMarksAtEdge({ text: '', side: 'end', },),
            ],).toEqual([true, true, true, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'COUNTS THE LOW, REVERSED AND ANGLE SINGLE MARKS TOO, since the pass counts all eight as single marks: a pair of any '
            + 'two of the eight at the edge asked about is a doubled mark, and a guillemet is none',
          fn: async () => {
            expect([
              holdsTwoMarksAtEdge({ text: 'the dogs’‹', side: 'end', },),
              holdsTwoMarksAtEdge({ text: '\u{201A}\u{201B}cats', side: 'start', },),
              holdsTwoMarksAtEdge({ text: 'the dogs›’', side: 'end', },),
              holdsTwoMarksAtEdge({ text: 'the dogs’»', side: 'end', },),
              holdsTwoMarksAtEdge({ text: '«’cats', side: 'start', },),
            ],).toEqual([true, true, true, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: 'the examples of consistency-splice-readings.ts',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RUNS EVERY @example OF THE FILE AND GETS THE ANSWER IT COMMENTS, an example that comments none returning a '
            + 'boolean, so an example whose offsets drift from the text it documents fails here',
          fn: async () => {
            /**
             The file the examples are read from.
             */
            const source = readFileSync(
              new URL(
                'consistency-splice-readings.ts',
                import.meta.url,
              ),
              'utf8',
            );
            /**
             The code of each `@example` block, between its fences.
             */
            const examples = source.split('@example',)
              .slice(1,)
              .map(function codeOf(block,): string {
                /**
                 Where the code starts, after the opening fence line.
                 */
                const start = block.indexOf('```ts\n',) + '```ts\n'.length;
                return block.slice(
                  start,
                  block.indexOf(
                    '\n ```',
                    start,
                  ),
                );
              },);
            /**
             What each example's call answers, the call being the code after `=` up to its closing `;`.
             */
            const answers = examples.map(function answerOf(code,): unknown {
              /**
               The call, without the declaration and the comment.
               */
              const call = code.slice(
                code.indexOf(' = ',) + ' = '.length,
                code.lastIndexOf(');',) + ');'.length,
              );
              return runInNewContext(
                call,
                {
                  bringsAtEdge,
                  bringsEdgeApostrophe,
                  gainsBesideKeptMark,
                  holdsTwoMarksAtEdge,
                  readingsChangeOutside,
                  target: { kind: 'located', quote: 'kittens', core: 'kittens', start: 0, markPastAfter: '’', },
                },
              );
            },);
            /**
             What each example comments as its answer: `true` or `false` where it comments one, none otherwise.
             */
            const commented = examples.map(function commentOf(code,): unknown {
              /**
               The words after the comment marker, none where the example has no comment.
               */
              const [, comment = '',] = code.split(' // ',);
              return (comment === '') ? 'none' : comment.startsWith('true',) ? true : comment.startsWith('false',) ? false : 'other';
            },);
            expect([examples.length, answers.length,],).toEqual([5, 5,],);
            expect(answers.map(function agreeing(answer, position,): unknown {
              return (commented[position] === 'none') ? (typeof answer) : answer;
            },),).toEqual(examples.map(function expected(_code, position,): unknown {
              return (commented[position] === 'none') ? 'boolean' : commented[position];
            },),);
          },
        },),
      ],
    },),
  ],
},);
