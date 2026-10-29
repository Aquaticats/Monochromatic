/**
 Tests for moving a chosen place's quoted words over another place's by code:
 what a replacement is made of, which edges of a quote it adjusts, and how
 replacements in one slice meet each other.

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
  applyConsistencyEdits,
  bringsEdgeApostrophe,
  type ConsistencyPlan,
  type ConsistencyRendering,
  type ConsistencyVoice,
  countConsistency,
  markReadingsInEnglish,
  pageApostropheFormOf,
  planConsistencyEdit,
  provenApostrophesInEnglish,
} from '../dist/final/node/index.mjs';
import {
  attachedOffsetsIn,
  pageMarksOf,
  unprovenLooseMarksIn,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/consistency-marks-reading.test-fixture.ts';
import {
  type Located,
  type QuotedPlace,
  renderingsOf,
} from './consistency-renderings.test-fixture.ts';

/**
 Plans a splice the way the pass does, comparing the marks beside the two places' words.

 @param chosen - chosen place's located rendering

 @param target - receiving place's located rendering

 @param framed - whether the group is a frame

 @returns Plan

 @example
 ```ts
 const plan = planCompared({ chosen, target, framed: false, },);
 ```
 */
function planCompared(
  {
    chosen,
    target,
    framed,
  }: {
    readonly chosen: Located;
    readonly target: Located;
    readonly framed: boolean;
  },
): ConsistencyPlan {
  return planConsistencyEdit({
    chosen,
    target,
    framed,
    marks: 'compared',
  },);
}


/**
 The places of a grid that the counter locates, each beside its located rendering, counted alone from three ballots
 that quote it and choose it.

 @param places - places of the grid

 @returns Each place that is located with its rendering, in the order of the grid

 @example
 ```ts
 const located = locatedAmong({ places: [{ english: 'Fill the cats bowl.', quote: 'the cats', },], },);
 ```
 */
function locatedAmong(
  { places, }: { readonly places: readonly QuotedPlace[]; },
): readonly (readonly [QuotedPlace, Located])[] {
  return places.flatMap(function locatedAlone(place,): readonly (readonly [QuotedPlace, Located])[] {
    const [only,] = countConsistency({
      voices: ['a', 'b', 'c',].map(function voiceOf(seat,): ConsistencyVoice {
        return {
          seat,
          reason: 'The first rendering is the more faithful.',
          quotes: [{ label: 'OCCURRENCE 1', quote: place.quote, },],
          choice: 'OCCURRENCE 1',
          keep: [],
        };
      },),
      places: [{ label: 'OCCURRENCE 1', english: { kind: 'present', text: place.english, }, },],
      framed: false,
      quorum: 2,
    },).renderings;
    return (only?.kind === 'located') ? [[place, only,],] : [];
  },);
}

/**
 The plan for moving the chosen place's quoted words over the receiving place's, both counted from their English.

 @param chosen - chosen place's English and quote

 @param receiving - receiving place's English and quote

 @returns Plan

 @example
 ```ts
 const plan = planFor({ chosen, receiving, },);
 ```
 */
function planFor(
  {
    chosen,
    receiving,
  }: {
    readonly chosen: QuotedPlace;
    readonly receiving: QuotedPlace;
  },
): ConsistencyPlan {
  const [first, second,] = renderingsOf({
    chosen,
    receiving,
  },);
  return planCompared({
    chosen: first,
    target: second,
    framed: false,
  },);
}

/**
 A located rendering.

 @param core - words the quote keeps

 @param start - offset in the English where the quote starts

 @param over - fields this case adds, a frame's slot among them

 @returns Located rendering

 @example
 ```ts
 const rendering = locatedAt({ core: 'the cat naps', start: 3, },);
 ```
 */
function locatedAt(
  {
    core,
    start,
    over = {},
  }: {
    readonly core: string;
    readonly start: number;
    readonly over?: Record<string, unknown>;
  },
): Extract<ConsistencyRendering, { readonly kind: 'located'; }> {
  return {
    kind: 'located',
    quote: core,
    core,
    start,
    ...over,
  } as Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
}

/**
 The characters that are single marks.
 */
const SINGLE_MARKS: ReadonlySet<string> = new Set(['\'', '‘', '’', '\u{02BC}',],);

/**
 The loose marks of a text in order, each folded to the form the comparison reads: every double mark, and every single
 mark that does not have a letter or a digit directly on both sides and is no proven apostrophe. Read by the fixture,
 written apart from the code under test, for ASCII.

 @param text - text read

 @returns Loose marks, one text per mark

 @example
 ```ts
 const marks = looseMarksIn({ text: 'the cats\' "bowl"', },); // ['"', '"']
 ```
 */
function looseMarksIn({ text, }: { readonly text: string; },): readonly string[] {
  return unprovenLooseMarksIn({ text, },);
}

/**
 Whether words start or end with a single mark, which is the apostrophe a plan carries in when the receiving words hold
 none there: whether it reads as one on the page depends on the page's other marks, which the pass decides where the page
 is known (`pass-consistency-apply-gains.ts`) and a plan alone does not.

 @param text - words that go in

 @returns Whether the first or the last character is a single mark

 @example
 ```ts
 const carries = edgeApostropheIn({ text: 'the parents’', },); // true
 ```
 */
function edgeApostropheIn({ text, }: { readonly text: string; },): boolean {
  return SINGLE_MARKS.has(text.at(-1,) ?? '',) || SINGLE_MARKS.has(text.at(0,) ?? '',);
}

/**
 The single mark directly beside an offset, folded, as a list of zero or one.

 @param text - text read

 @param at - offset asked about, which may lie outside the text

 @returns Mark, or none

 @example
 ```ts
 const marks = markAt({ text: 'the cats\' bowl', at: 8, },); // ['\'',]
 ```
 */
function markAt({ text, at, }: { readonly text: string; readonly at: number; },): readonly string[] {
  const point = text.charAt(at,);
  if (!SINGLE_MARKS.has(point,))
    return [];
  return [(point === '‘') ? '‘' : '\'',];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: planConsistencyEdit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPLACES THE RECEIVING PLACE\'S WORDS with the chosen words, over the span the receiving quote stands in',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({ core: 'the kitten dozes', start: 40, },),
              target: locatedAt({ core: 'the kitten naps', start: 7, },),
              framed: false,
            },),).toEqual({
              kind: 'replace',
              start: 7,
              end: 22,
              text: 'the kitten dozes',
            },);
          },
        },),
        it({
          name: 'FINDS THE PLACE ALREADY AGREEING when the words are the same, and plans no edit',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({ core: 'the kitten dozes', start: 40, },),
              target: locatedAt({ core: 'the kitten dozes', start: 7, },),
              framed: false,
            },),).toEqual({ kind: 'same', },);
          },
        },),
        it({
          name: 'LEAVES THE RECEIVING PLACE\'S CLOSING PUNCTUATION WHERE IT IS, replacing the core and not the quote',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({ core: 'the kitten dozes', start: 40, over: { quote: 'the kitten dozes.', }, },),
              target: locatedAt({ core: 'the kitten naps', start: 7, over: { quote: 'the kitten naps!', }, },),
              framed: false,
            },),).toEqual({
              kind: 'replace',
              start: 7,
              end: 22,
              text: 'the kitten dozes',
            },);
          },
        },),
        it({
          name: 'GIVES THE FIRST LETTER THE RECEIVING PLACE\'S CASE when the two quotes begin with one letter, a '
            + 'sentence start into a clause and back',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: 'The kitten dozes', start: 0, },),
                target: locatedAt({ core: 'the kitten naps', start: 9, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the kitten dozes', start: 9, },),
                target: locatedAt({ core: 'The kitten naps', start: 0, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'replace', start: 9, end: 24, text: 'the kitten dozes', },
              { kind: 'replace', start: 0, end: 15, text: 'The kitten dozes', },
            ],);
          },
        },),
        it({
          name: 'KEEPS THE CHOSEN CASE when the quotes begin with different letters, however they are capitalised',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({ core: 'Sunny windows nap', start: 0, },),
              target: locatedAt({ core: 'the kitten naps', start: 9, },),
              framed: false,
            },),).toEqual({ kind: 'replace', start: 9, end: 24, text: 'Sunny windows nap', },);
          },
        },),
        it({
          name: 'GIVES A FRAME\'S RECEIVING PLACE THE CHOSEN PATTERN with its own slot words in the chosen slot',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({
                core: 'So this cat must be a chubby little treasure',
                start: 42,
                over: { slot: 'chubby', slotStart: 22, },
              },),
              target: locatedAt({
                core: 'So this cat is surely a lazy little treasure',
                start: 50,
                over: { slot: 'lazy', slotStart: 24, },
              },),
              framed: true,
            },),).toEqual({
              kind: 'replace',
              start: 50,
              end: 94,
              text: 'So this cat must be a lazy little treasure',
            },);
          },
        },),
        it({
          name: 'GIVES A LETTER BEYOND THE FIRST PLANE THE RECEIVING PLACE\'S CASE, reading the whole character and not '
            + 'the first half of its pair',
          fn: async () => {
            const small = String.fromCodePoint(0x1_04_28,);
            const capital = String.fromCodePoint(0x1_04_00,);
            expect(planCompared({
              chosen: locatedAt({ core: `${small} cat naps`, start: 0, },),
              target: locatedAt({ core: `${capital} cat sleeps`, start: 9, },),
              framed: false,
            },),).toEqual({
              kind: 'replace',
              start: 9,
              end: 9 + `${capital} cat sleeps`.length,
              text: `${capital} cat naps`,
            },);
          },
        },),
        it({
          name: 'LEAVES A QUOTE THAT OPENS ON A DIGIT ALONE, each digit being its own',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({ core: '3 cats nap', start: 0, },),
              target: locatedAt({ core: '4 cats sleep', start: 9, },),
              framed: false,
            },),).toEqual({ kind: 'replace', start: 9, end: 21, text: '3 cats nap', },);
          },
        },),
        it({
          name: 'GIVES THE FIRST LETTER AFTER AN OPENING MARK THE RECEIVING PLACE\'S CASE, a quotation mark or an emphasis '
            + 'mark opening both quotes',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: '“The cat naps”', start: 0, },),
                target: locatedAt({ core: '“the cat naps”', start: 9, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: '*The cat* naps', start: 0, },),
                target: locatedAt({ core: '*the cat* dozes', start: 9, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'same', },
              { kind: 'replace', start: 9, end: 24, text: '*the cat* naps', },
            ],);
          },
        },),
        it({
          name: 'LEAVES A QUOTE MADE OF OPENING MARKS ALONE AS IT IS, there being no letter to give a case',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: '((', start: 0, },),
                target: locatedAt({ core: 'the cat', start: 9, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'The cat', start: 0, },),
                target: locatedAt({ core: '**', start: 9, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'replace', start: 9, end: 16, text: '((', },
              { kind: 'replace', start: 9, end: 11, text: 'The cat', },
            ],);
          },
        },),
        it({
          name: 'REPLACES A RECEIVING QUOTE THAT BEGINS WITH THE CHOSEN QUOTE, and one at the start or the end of its English',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: 'the kitten', start: 30, },),
                target: locatedAt({ core: 'the kitten naps on the mat', start: 0, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the kitten', start: 0, },),
                target: locatedAt({ core: 'the kitten naps', start: 17, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'replace', start: 0, end: 26, text: 'the kitten', },
              { kind: 'replace', start: 17, end: 32, text: 'the kitten', },
            ],);
          },
        },),
        it({
          name: 'FINDS A FRAME\'S PLACE ALREADY IN THE CHOSEN PATTERN when only its slot differs',
          fn: async () => {
            expect(planCompared({
              chosen: locatedAt({
                core: 'So this cat must be a chubby little treasure',
                start: 42,
                over: { slot: 'chubby', slotStart: 22, },
              },),
              target: locatedAt({
                core: 'So this cat must be a lazy little treasure',
                start: 50,
                over: { slot: 'lazy', slotStart: 22, },
              },),
              framed: true,
            },),).toEqual({ kind: 'same', },);
          },
        },),
        it({
          name: 'FINDS A FRAME\'S PLACE UNAVAILABLE TO CODE when either place has no agreed slot, and the same when the words '
            + 'are already the same',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: 'So this cat must be a chubby little treasure', start: 42, },),
                target: locatedAt({
                  core: 'So this cat is surely a lazy little treasure',
                  start: 50,
                  over: { slot: 'lazy', slotStart: 24, },
                },),
                framed: true,
              },),
              planCompared({
                chosen: locatedAt({
                  core: 'So this cat must be a chubby little treasure',
                  start: 42,
                  over: { slot: 'chubby', slotStart: 22, },
                },),
                target: locatedAt({ core: 'So this cat is surely a lazy little treasure', start: 50, },),
                framed: true,
              },),
              planCompared({
                chosen: locatedAt({ core: 'So this cat must be a treasure', start: 42, },),
                target: locatedAt({ core: 'So this cat must be a treasure', start: 50, },),
                framed: true,
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'same', },
            ],);
          },
        },),
        it({
          name: 'PLANS A REPLACEMENT WHEN THE CHOSEN WORDS KEEP THE APOSTROPHE THEY WERE WRITTEN WITH: a dropped '
            + 'letter at the end or the start of the words, a plural possessive, which the pass then decides on the page',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'He was goin\' home.', quote: 'goin\'', },
                receiving: { english: 'He was going home.', quote: 'going', },
              },),
              planFor({
                chosen: { english: 'The cats\' bowl is full.', quote: 'The cats\'', },
                receiving: { english: 'The cat\'s bowl is full.', quote: 'The cat\'s', },
              },),
              planFor({
                chosen: { english: '\'Tis the season.', quote: '\'Tis the season', },
                receiving: { english: 'It is the season.', quote: 'It is the season', },
              },),
              planFor({
                chosen: { english: 'He was goin’ home.', quote: 'goin’', },
                receiving: { english: 'He was going home.', quote: 'going', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 7, end: 12, text: 'goin\'', },
              { kind: 'replace', start: 0, end: 9, text: 'The cats\'', },
              { kind: 'replace', start: 0, end: 16, text: '\'Tis the season', },
              { kind: 'replace', start: 7, end: 12, text: 'goin’', },
            ],);
          },
        },),
        it({
          name: 'TAKES A PROVEN APOSTROPHE OUT WITH THE WORDS IT BELONGS TO WHEN THE CHOSEN WORDS HOLD NONE: a plural '
            + 'possessive, a year, a clipped word, curly or straight, and finds the same splice unavailable where the '
            + 'receiving English holds an opener that could pair with the mark',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'The cat naps.', quote: 'The cat', },
                receiving: { english: 'The cats\' bowl is full.', quote: 'The cats\'', },
              },),
              planFor({
                chosen: { english: 'The cat naps.', quote: 'The cat', },
                receiving: { english: 'The cats’ bowl is full.', quote: 'The cats’', },
              },),
              planFor({
                chosen: { english: 'The 1990s cats nap.', quote: '1990s cats', },
                receiving: { english: 'The ’90s cats nap.', quote: '’90s cats', },
              },),
              planFor({
                chosen: { english: 'Give them the bowl.', quote: 'them', },
                receiving: { english: 'Give ’em the bowl.', quote: '’em', },
              },),
              planFor({
                chosen: { english: 'Wait until dark.', quote: 'until', },
                receiving: { english: 'Wait \'til dark.', quote: '\'til', },
              },),
              planFor({
                chosen: { english: 'The cat naps.', quote: 'The cat', },
                receiving: { english: 'She said ‘The cats’ bowl is full.’', quote: 'The cats', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 0, end: 9, text: 'The cat', },
              { kind: 'replace', start: 0, end: 9, text: 'The cat', },
              { kind: 'replace', start: 4, end: 13, text: '1990s cats', },
              { kind: 'replace', start: 5, end: 8, text: 'them', },
              { kind: 'replace', start: 5, end: 9, text: 'until', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN QUOTATION MARKS INSIDE THE WORDS DO NOT MATCH those of the receiving words, '
            + 'in either direction, double or single',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'He said, "So she was kind," she said.', quote: '"So she was kind," she said', },
                receiving: { english: 'Then "She was kind," she said.', quote: 'She was kind," she said', },
              },),
              planFor({
                chosen: { english: 'Then So she was kind, she said.', quote: 'So she was kind, she said', },
                receiving: { english: 'Then "She was kind," she said.', quote: 'She was kind," she said', },
              },),
              planFor({
                chosen: { english: 'She said ‘I am here.', quote: 'I am here', },
                receiving: { english: 'He said ‘We are near,’ and left.', quote: 'We are near,’ and left', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN THE CORNER BRACKETS INSIDE THE WORDS DO NOT MATCH those of the receiving '
            + 'words: one the chosen words would bring in, one they would delete, the other kind of bracket, and a double mark '
            + 'in the bracket\'s place',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'Then she was 「so kind」 today.', quote: 'she was 「so kind」 today', },
                receiving: { english: 'Later she was so merry today.', quote: 'she was so merry today', },
              },),
              planFor({
                chosen: { english: 'Then she was so kind today.', quote: 'she was so kind today', },
                receiving: { english: 'Later she was 「so merry」 today.', quote: 'she was 「so merry」 today', },
              },),
              planFor({
                chosen: { english: 'Then she was 「so kind」 today.', quote: 'she was 「so kind」 today', },
                receiving: { english: 'Later she was 『so merry』 today.', quote: 'she was 『so merry』 today', },
              },),
              planFor({
                chosen: { english: 'Then she was "so kind" today.', quote: 'she was "so kind" today', },
                receiving: { english: 'Later she was 「so merry」 today.', quote: 'she was 「so merry」 today', },
              },),
              planFor({
                chosen: { english: 'Then she was 「so kind」 today.', quote: 'she was 「so kind」 today', },
                receiving: { english: 'Later she was “so merry” today.', quote: 'she was “so merry” today', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN A LOW OR REVERSED DOUBLE MARK INSIDE THE WORDS IS BROUGHT IN, DELETED OR MOVED, as '
            + 'for any double mark: a lone one in the receiving words, one in the chosen words only, a reversed one in the '
            + 'receiving words, and a pair whose order against a corner bracket differs',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'She said hello to the cats.', quote: 'She said hello to the cats', },
                receiving: { english: 'She said „hello to the kittens.', quote: 'She said „hello to the kittens', },
              },),
              planFor({
                chosen: { english: 'She said „hello to the cats.', quote: 'She said „hello to the cats', },
                receiving: { english: 'She said hello to the kittens.', quote: 'She said hello to the kittens', },
              },),
              planFor({
                chosen: { english: 'She said hello to the cats.', quote: 'She said hello to the cats', },
                receiving: { english: 'She said ‟hello to the kittens.', quote: 'She said ‟hello to the kittens', },
              },),
              planFor({
                chosen: { english: 'She said „hello” 「to」 the cats.', quote: 'She said „hello” 「to」 the cats', },
                receiving: { english: 'She said 「hello」 „to” the kittens.', quote: 'She said 「hello」 „to” the kittens', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'STILL REPLACES WHEN BOTH PLACES HOLD A LOW DOUBLE MARK PAIR INSIDE THEIR WORDS, and writes it in the receiving '
            + 'place\'s form when the other place wrote a curly pair',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'She said „hello” to the cats.', quote: 'She said „hello” to the cats', },
                receiving: { english: 'He said „hi” to the kittens.', quote: 'He said „hi” to the kittens', },
              },),
              planFor({
                chosen: { english: 'She said “hello” to the cats.', quote: 'She said “hello” to the cats', },
                receiving: { english: 'He said „hi” to the kittens.', quote: 'He said „hi” to the kittens', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 0, end: 27, text: 'She said „hello” to the cats', },
              { kind: 'replace', start: 0, end: 27, text: 'She said „hello” to the cats', },
            ],);
          },
        },),
        it({
          name: 'STILL REPLACES WHEN BOTH PLACES HOLD THE SAME CORNER BRACKETS INSIDE THEIR WORDS, in the same order',
          fn: async () => {
            expect(planFor({
              chosen: { english: 'Then she was 「so kind」 today.', quote: 'she was 「so kind」 today', },
              receiving: { english: 'Later she was 「so merry」 today.', quote: 'she was 「so merry」 today', },
            },),).toEqual({ kind: 'replace', start: 6, end: 30, text: 'she was 「so kind」 today', },);
          },
        },),
        it({
          name: 'STILL REPLACES WHEN BOTH PLACES HOLD THE SAME MARKS: the same plural possessive, apostrophes inside the '
            + 'words, the same double-quoted clause, and the same proven apostrophe written straight at one place and curly '
            + 'or modifier-letter at the other, which goes in written in the receiving place\'s form, as an apostrophe '
            + 'inside a word does',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'The cats\' bowl is full.', quote: 'The cats\'', },
                receiving: { english: 'The kittens\' bowl is full.', quote: 'The kittens\'', },
              },),
              planFor({
                chosen: { english: 'The cat\'s bowl is full.', quote: 'The cat\'s bowl', },
                receiving: { english: 'The kitten\'s bowl is full.', quote: 'The kitten\'s bowl', },
              },),
              planFor({
                chosen: { english: 'He doesn\'t nap.', quote: 'doesn\'t nap', },
                receiving: { english: 'He won’t nap.', quote: 'won’t nap', },
              },),
              planFor({
                chosen: { english: 'Then "She was kind," she said.', quote: '"She was kind," she said', },
                receiving: { english: 'Later “She was merry,” she said.', quote: '“She was merry,” she said', },
              },),
              planFor({
                chosen: { english: 'Fill the cats\' bowl.', quote: 'the cats\'', },
                receiving: { english: 'Fill the kittens’ bowl.', quote: 'the kittens’', },
              },),
              planFor({
                chosen: { english: 'Fill the cats\' bowl.', quote: 'the cats\'', },
                receiving: { english: 'Fill the kittens\u{02BC} bowl.', quote: 'the kittens\u{02BC}', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 0, end: 12, text: 'The cats\'', },
              { kind: 'replace', start: 0, end: 17, text: 'The cat\'s bowl', },
              { kind: 'replace', start: 3, end: 12, text: 'doesn’t nap', },
              { kind: 'replace', start: 6, end: 31, text: '“She was kind,” she said', },
              { kind: 'replace', start: 5, end: 17, text: 'the cats’', },
              { kind: 'replace', start: 5, end: 17, text: 'the cats\u{02BC}', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN IT WOULD CHANGE WHICH APOSTROPHES ARE ATTACHED: the receiving words end in '
            + 'emphasis, a bracket or a double quotation mark before the mark beside them and the chosen words end in a '
            + 'letter, so the page would read `cats\'s`, in straight and curly forms; an opener before the places leaves '
            + 'the marks beside the words with a possible partner, so they stay outside the spans and are compared',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: '‘Fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill the **cat**\' bowl.', quote: 'the **cat**', },
              },),
              planFor({
                chosen: { english: '‘Fill the cats’ bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill the **cat**’ bowl.', quote: 'the **cat**', },
              },),
              planFor({
                chosen: { english: '‘Fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill *Tom*\' bowl.', quote: '*Tom*', },
              },),
              planFor({
                chosen: { english: '‘Fill the "grey" cats\' bowl.', quote: 'the "grey" cats', },
                receiving: { english: '‘Fill the "grey cat"\' bowl.', quote: 'the "grey cat"', },
              },),
              planFor({
                chosen: { english: '‘Fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill the (grey cat)\' bowl.', quote: 'the (grey cat)', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'READS THE EDGE OF THE WORDS ON THE SIDE OF EACH MARK BESIDE THEM: the start for a mark before the words, the '
            + 'end for a mark after, a side with no mark beside it at the receiving place decides nothing, and an empty edge is '
            + 'its own class that no letter equals',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: 'grey cats', start: 0, over: { markBefore: '\'', }, },),
                target: locatedAt({ core: '(grey cat)', start: 9, over: { markBefore: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'grey cats', start: 0, over: { markBefore: '\'', }, },),
                target: locatedAt({ core: 'white cat', start: 9, over: { markBefore: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'grey cats', start: 0, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: '(grey cats', start: 9, over: { markAfter: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'grey cats', start: 0, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: 'grey cat)', start: 9, over: { markAfter: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: '', start: 0, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: 'cats', start: 9, over: { markAfter: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: '', start: 0, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: '', start: 9, over: { markAfter: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: '(grey) cats', start: 0, },),
                target: locatedAt({ core: 'grey cat', start: 9, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'replace', start: 9, end: 18, text: 'grey cats', },
              { kind: 'replace', start: 9, end: 19, text: 'grey cats', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'same', },
              { kind: 'replace', start: 9, end: 17, text: '(grey) cats', },
            ],);
          },
        },),
        it({
          name: 'STILL REPLACES WHEN THE EDGES BESIDE THE MARK ARE OF ONE CLASS: a plural possessive onto another, a word in '
            + 'emphasis onto another word in emphasis, and a mark beside words in emphasis onto a mark beside words in '
            + 'emphasis',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: '‘Fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill the kittens\' bowl.', quote: 'the kittens', },
              },),
              planFor({
                chosen: { english: '‘Fill the **cat**\' bowl.', quote: 'the **cat**', },
                receiving: { english: '‘Fill the **dog**\' bowl.', quote: 'the **dog**', },
              },),
              planFor({
                chosen: { english: '‘Fill the (grey cat)\' bowl.', quote: 'the (grey cat)', },
                receiving: { english: '‘Fill the (white cat)’ bowl.', quote: 'the (white cat)', },
              },),
              planFor({
                chosen: { english: '‘Fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: '‘Fill *the* cats\' bowl.', quote: '*the* cats', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 6, end: 17, text: 'the cats', },
              { kind: 'replace', start: 6, end: 17, text: 'the **cat**', },
              { kind: 'replace', start: 6, end: 21, text: 'the (grey cat)', },
              { kind: 'replace', start: 6, end: 16, text: 'the cats', },
            ],);
          },
        },),
        it({
          name: 'LEAVES EVERY APOSTROPHE A LETTER ATTACHES OR DOES NOT ATTACH AS IT WAS, over a grid of words with letters, '
            + 'emphasis, brackets and quotation marks at their edges and a mark with a letter or none beyond it: any plan '
            + 'that replaces leaves the loose marks of the whole receiving English exactly as they were',
          fn: async () => {
            const cores = ['the cats nap', '*the cats nap*', '(the cats nap)', 'the "cats" nap', 'the cats "nap"', '3 cats',];
            const befores = ['', '\'', 'x\'', '’', 'x’',];
            const afters = ['', '\'', '\'s', '’s', '\' x', '’',];
            /**
             Every English of the grid: an English with a mark and a letter, or none, before and after the words.
             */
            const places = cores.flatMap(function ofCore(core,): readonly QuotedPlace[] {
              return befores.flatMap(function ofBefore(before,): readonly QuotedPlace[] {
                return afters.map(function ofAfter(after,): QuotedPlace {
                  return { english: `Fill ${before}${core}${after} bowl.`, quote: core, };
                },);
              },);
            },);
            /**
             The places of the grid the counter locates, which are all the property is about.
             */
            const located = locatedAmong({ places, },);
            /**
             What each pair of places came to, and every replacement that changed the loose marks of the receiving English.
             */
            const tally = { replaced: 0, refused: 0, broken: [] as string[], };
            for (const [chosen, first,] of located) {
              for (const [receiving, second,] of located) {
                const plan = planCompared({
                  chosen: first,
                  target: second,
                  framed: false,
                },);
                if (plan.kind === 'unavailable') {
                  tally.refused += 1;
                  continue;
                }
                if (plan.kind === 'same')
                  continue;
                tally.replaced += 1;
                const applied = applyConsistencyEdits({
                  english: receiving.english,
                  edits: [{ id: 'x', start: plan.start, end: plan.end, text: plan.text, },],
                },).text;
                // A plan that carries an edge apostrophe in is decided on the page by the pass, so it is no break here.
                if ((looseMarksIn({ text: applied, },).join('',) !== looseMarksIn({ text: receiving.english, },).join('',))
                  && (!edgeApostropheIn({ text: plan.text, })))
                  tally.broken.push(`${chosen.english} -> ${receiving.english}: ${applied}`,);
              }
            }
            expect(tally.broken,).toEqual([],);
            expect(tally.replaced > 0,).toBe(true,);
            expect(tally.refused > 0,).toBe(true,);
            expect(located.length > 50,).toBe(true,);
          },
        },),
        it({
          name: 'WRITES EACH LOOSE MARK OF THE WORDS THAT GO IN IN THE FORM THE RECEIVING PLACE WROTE AT THE SAME POSITION, '
            + 'so straight quotation marks carried onto a curly place leave it curly and the reverse, an apostrophe '
            + 'follows the receiving place\'s, and an apostrophe of a word, inside it or proven at its edge, is written in '
            + 'the receiving place\'s form',
          fn: async () => {
            /**
             The words a plan puts in, or the kind of plan where it puts none.
             */
            function textOrKind({ plan, }: { readonly plan: ConsistencyPlan; },): string {
              return (plan.kind === 'replace') ? plan.text : plan.kind;
            }
            expect([
              planFor({
                chosen: { english: 'Then "She was kind," she said.', quote: '"She was kind," she said', },
                receiving: { english: 'Later “He was merry,” he said.', quote: '“He was merry,” he said', },
              },),
              planFor({
                chosen: { english: 'Then “She was kind,” she said.', quote: '“She was kind,” she said', },
                receiving: { english: 'Later "He was merry," he said.', quote: '"He was merry," he said', },
              },),
              planFor({
                chosen: { english: 'Fill the cats\' bowl and the dogs\' dish.', quote: 'the cats\' bowl and the dogs', },
                receiving: { english: 'Fill the kittens’ bowl and the pups’ dish.', quote: 'the kittens’ bowl and the pups', },
              },),
              planFor({
                chosen: { english: 'He said "go" and \'run\' now.', quote: 'He said "go" and \'run\' now', },
                receiving: { english: 'She said “stop” and \'wait\' soon.', quote: 'She said “stop” and \'wait\' soon', },
              },),
              planFor({
                chosen: { english: 'Fill the cat\'s "bowl" now.', quote: 'the cat\'s "bowl"', },
                receiving: { english: 'Fill the kitten’s “dish” now.', quote: 'the kitten’s “dish”', },
              },),
            ].map(function textOf(plan,): string {
              return textOrKind({ plan, },);
            },),).toEqual([
              '“She was kind,” she said',
              '"She was kind," she said',
              'the cats’ bowl and the dogs’',
              'He said “go” and \'run\' now',
              'the cat’s “bowl”',
            ],);
          },
        },),
        it({
          name: 'CALLS WORDS THAT DIFFER ONLY IN THE FORM OF THEIR LOOSE MARKS THE SAME, and leaves the opening single '
            + 'quotation mark, the corner brackets and a record that kept no marks as they were written',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'Fill the "cats" bowl.', quote: 'the "cats" bowl', },
                receiving: { english: 'Fill the “cats” bowl.', quote: 'the “cats” bowl', },
              },),
              planFor({
                chosen: { english: 'He said ‘go’ now.', quote: 'He said ‘go’ now', },
                receiving: { english: 'She said ‘stop’ soon.', quote: 'She said ‘stop’ soon', },
              },),
              planFor({
                chosen: { english: 'Then she was 「so kind」 today.', quote: 'she was 「so kind」 today', },
                receiving: { english: 'Later she was 「so merry」 today.', quote: 'she was 「so merry」 today', },
              },),
              planConsistencyEdit({
                chosen: locatedAt({ core: 'the "cats" nap', start: 0, },),
                target: locatedAt({ core: 'the “dogs” nap', start: 9, },),
                framed: false,
                marks: 'unrecorded',
              },),
            ],).toEqual([
              { kind: 'same', },
              { kind: 'replace', start: 0, end: 20, text: 'He said ‘go’ now', },
              { kind: 'replace', start: 6, end: 30, text: 'she was 「so kind」 today', },
              { kind: 'replace', start: 9, end: 23, text: 'the "cats" nap', },
            ],);
          },
        },),
        it({
          name: 'FINDS TWO PLACES WHOSE WORDS AGREE AND WHOSE MARKS DIFFER UNAVAILABLE, not the same rendering, and keeps '
            + 'the one with equal marks the same',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'Say ‘go, then fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: 'Say ‘go, then fill the cats bowl.', quote: 'the cats', },
              },),
              planFor({
                chosen: { english: 'Say ‘go, then fill the cats bowl.', quote: 'the cats', },
                receiving: { english: 'Say ‘go, then fill the cats\' bowl.', quote: 'the cats', },
              },),
              planFor({
                chosen: { english: 'Say ‘the cats nap.', quote: 'the cats nap', },
                receiving: { english: 'Say the cats nap.', quote: 'the cats nap', },
              },),
              planFor({
                chosen: { english: 'Say ‘go, then fill the cats\' bowl.', quote: 'the cats', },
                receiving: { english: 'Say ‘go, then fill the cats’ bowl.', quote: 'the cats', },
              },),
              planFor({
                chosen: { english: 'Fill the cats bowl.', quote: 'the cats', },
                receiving: { english: 'Fill the cats bowl.', quote: 'the cats', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'same', },
              { kind: 'same', },
            ],);
          },
        },),
        it({
          name: 'FOLDS THE THREE APOSTROPHE-LIKE MARKS TO ONE WHEN COMPARING THE MARKS BESIDE THE WORDS, and keeps the '
            + 'opening single quotation mark apart, or absent from present',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markAfter: '’', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, over: { markAfter: '’', }, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markAfter: '\u{02BC}', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, over: { markBefore: '‘', }, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markBefore: '‘', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, over: { markBefore: '‘', }, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markBefore: '\'', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, over: { markBefore: '\'', }, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markBefore: '’', }, },),
                framed: false,
              },),
              planCompared({
                chosen: locatedAt({ core: 'the cats', start: 0, },),
                target: locatedAt({ core: 'the cats', start: 9, over: { markAfter: '’', }, },),
                framed: false,
              },),
            ],).toEqual([
              { kind: 'same', },
              { kind: 'same', },
              { kind: 'same', },
              { kind: 'unavailable', },
              { kind: 'same', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'COMPARES THE MARKS OF A FRAME\'S PLACES TOO, with a slot and without one, before it calls them the same',
          fn: async () => {
            expect([
              planCompared({
                chosen: locatedAt({
                  core: 'So this cat must be a chubby little treasure',
                  start: 42,
                  over: { slot: 'chubby', slotStart: 22, markAfter: '\'', },
                },),
                target: locatedAt({
                  core: 'So this cat is surely a lazy little treasure',
                  start: 50,
                  over: { slot: 'lazy', slotStart: 24, },
                },),
                framed: true,
              },),
              planCompared({
                chosen: locatedAt({ core: 'So this cat must be a treasure', start: 42, over: { markBefore: '‘', }, },),
                target: locatedAt({ core: 'So this cat must be a treasure', start: 50, },),
                framed: true,
              },),
              planCompared({
                chosen: locatedAt({
                  core: 'So this cat must be a chubby little treasure',
                  start: 42,
                  over: { slot: 'chubby', slotStart: 22, markAfter: '\'', },
                },),
                target: locatedAt({
                  core: 'So this cat is surely a lazy little treasure',
                  start: 50,
                  over: { slot: 'lazy', slotStart: 24, markAfter: '’', },
                },),
                framed: true,
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              {
                kind: 'replace',
                start: 50,
                end: 94,
                text: 'So this cat must be a lazy little treasure',
              },
            ],);
          },
        },),
        it({
          name: 'COMPARES NO MARK FOR A RECORD THAT HELD NONE: with the marks left unrecorded, places whose marks or '
            + 'quotation marks differ are planned as before',
          fn: async () => {
            expect([
              planConsistencyEdit({
                chosen: locatedAt({ core: 'goin', start: 7, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: 'going', start: 7, },),
                framed: false,
                marks: 'unrecorded',
              },),
              planConsistencyEdit({
                chosen: locatedAt({ core: 'the cats', start: 7, over: { markAfter: '\'', }, },),
                target: locatedAt({ core: 'the cats', start: 9, },),
                framed: false,
                marks: 'unrecorded',
              },),
              planConsistencyEdit({
                chosen: locatedAt({ core: 'So she was kind', start: 7, },),
                target: locatedAt({ core: 'She was kind," she said', start: 9, },),
                framed: false,
                marks: 'unrecorded',
              },),
            ],).toEqual([
              { kind: 'replace', start: 7, end: 12, text: 'goin', },
              { kind: 'same', },
              { kind: 'replace', start: 9, end: 32, text: 'So she was kind', },
            ],);
          },
        },),
        it({
          name: 'LEAVES EVERY LOOSE MARK OF THE RECEIVING PLACE WHERE IT IS AND BRINGS NONE IN, over a grid of words and '
            + 'marks before and after them at both places: any plan that replaces keeps the receiving English\'s '
            + 'quotation marks and the chosen place\'s',
          fn: async () => {
            const cores = ['the cats nap', 'a "quiet" cat nap', 'the cat\'s nap',];
            const befores = ['', '\'', '‘', '’', '\u{02BC}', '"',];
            const afters = ['', '\'', '’', '"',];
            /**
             Every place of the build: an English with the words, a mark before them and a mark after them.
             */
            const places = cores.flatMap(function ofCore(core,): readonly QuotedPlace[] {
              return befores.flatMap(function ofBefore(before,): readonly QuotedPlace[] {
                return afters.map(function ofAfter(after,): QuotedPlace {
                  return { english: `He said ${before}${core}${after} and left.`, quote: core, };
                },);
              },);
            },);
            /**
             What each pair of places came to, and every replacement that broke either guarantee.
             */
            const tally = { replaced: 0, refused: 0, same: 0, broken: [] as string[], };
            for (const chosen of places) {
              for (const receiving of places) {
                const [first, second,] = renderingsOf({ chosen, receiving, },);
                const plan = planCompared({
                  chosen: first,
                  target: second,
                  framed: false,
                },);
                if (plan.kind === 'unavailable') {
                  tally.refused += 1;
                  continue;
                }
                if (plan.kind === 'same') {
                  tally.same += 1;
                  continue;
                }
                tally.replaced += 1;
                const applied = applyConsistencyEdits({
                  english: receiving.english,
                  edits: [{ id: 'x', start: plan.start, end: plan.end, text: plan.text, },],
                },).text;
                const spanStart = plan.start;
                const spanEnd = plan.start + plan.text.length;
                const chosenStart = first.start;
                const chosenEnd = first.start + first.core.length;
                /**
                 What the receiving English holds, in loose marks, after and before the replacement.
                 */
                const loose = {
                  after: looseMarksIn({ text: applied, },)
                    .join('',),
                  before: looseMarksIn({ text: receiving.english, },)
                    .join('',),
                  span: looseMarksIn({ text: applied.slice(spanStart, spanEnd,), },)
                    .join('',),
                  chosenSpan: looseMarksIn({ text: chosen.english.slice(chosenStart, chosenEnd,), },)
                    .join('',),
                };
                /**
                 The single marks directly beside the replaced span, and beside the chosen span.
                 */
                const beside = {
                  newBefore: markAt({ text: applied, at: spanStart - 1, },)
                    .join('',),
                  newAfter: markAt({ text: applied, at: spanEnd, },)
                    .join('',),
                  chosenBefore: markAt({ text: chosen.english, at: chosenStart - 1, },)
                    .join('',),
                  chosenAfter: markAt({ text: chosen.english, at: chosenEnd, },)
                    .join('',),
                };
                // An apostrophe the words carry in may read as a quotation mark on a page that holds an opener, which the
                // pass catches where the page is known; any other plan keeps every loose mark.
                const keptAll = (loose.after === loose.before) || edgeApostropheIn({ text: plan.text, });
                const sameAsChosen = (loose.span === loose.chosenSpan)
                  && (beside.newBefore === beside.chosenBefore)
                  && (beside.newAfter === beside.chosenAfter);
                if (!(keptAll && sameAsChosen))
                  tally.broken.push(`${chosen.english} -> ${receiving.english}: ${applied}`,);
              }
            }
            expect(tally.broken,).toEqual([],);
            expect(tally.replaced > 0,).toBe(true,);
            expect(tally.refused > 0,).toBe(true,);
            expect(tally.same > 0,).toBe(true,);
          },
        },),
        it({
          name: 'CARRIES A PROVEN APOSTROPHE WITH THE WORDS IT BELONGS TO: a plural possessive replaces another and the page '
            + 'reads the chosen one\'s mark, one goes with the words it was in, and a dropped letter at the start is carried '
            + 'the same way',
          fn: async () => {
            /**
             The page after a plan, or the kind of plan where it changes none.
             */
            function pageAfter({ plan, english, }: { readonly plan: ConsistencyPlan; readonly english: string; },): string {
              return (plan.kind === 'replace')
                ? applyConsistencyEdits({
                  english,
                  edits: [{ id: 'x', start: plan.start, end: plan.end, text: plan.text, },],
                },).text
                : plan.kind;
            }
            /**
             The pairs of the cases, each the chosen place and the receiving place.
             */
            const pairs: readonly (readonly [QuotedPlace, QuotedPlace])[] = [
              [
                { english: 'Fill the parents’ bowl.', quote: 'the parents', },
                { english: 'Fill the folks’ bowl.', quote: 'the folks', },
              ],
              [
                { english: 'Fill the parents\' bowl.', quote: 'the parents', },
                { english: 'Fill the folks’ bowl.', quote: 'the folks', },
              ],
              [
                { english: 'Fill the parent bowl.', quote: 'the parent', },
                { english: 'Fill the parents’ preference.', quote: 'the parents', },
              ],
              [
                { english: 'Guard her ’round the cats.', quote: 'round the', },
                { english: 'Guard her ’bout the dogs.', quote: 'bout the', },
              ],
              [
                { english: 'Guard her round the cats.', quote: 'round the', },
                { english: 'Guard her ’bout the dogs.', quote: 'bout the', },
              ],
            ];
            expect(pairs.map(function pageOf([chosen, receiving,],): string {
              return pageAfter({
                plan: planFor({ chosen, receiving, },),
                english: receiving.english,
              },);
            },),).toEqual([
              'Fill the parents’ bowl.',
              'Fill the parents’ bowl.',
              'Fill the parent preference.',
              'Guard her ’round the dogs.',
              'Guard her round the dogs.',
            ],);
          },
        },),
        it({
          name: 'PLANS A PROVEN APOSTROPHE INTO A PLACE THAT HOLDS NONE AT THAT EDGE, the gain case: the plan is a '
            + 'replacement that carries the mark in, and the page decides whether it reads as an apostrophe there '
            + '(`pass-consistency-apply-gains.ts`); a quotation closer, and marks that differ beside the words, still refuse',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'Fill the parents’ bowl.', quote: 'the parents', },
                receiving: { english: 'Fill the family bowl.', quote: 'the family', },
              },),
              planFor({
                chosen: { english: 'Guard her ’round the cats.', quote: 'round the', },
                receiving: { english: 'Guard her about the dogs.', quote: 'about the', },
              },),
              planFor({
                chosen: { english: 'Fill the parents’ bowl.', quote: 'the parents', },
                receiving: { english: 'Guard her ’round the cats.', quote: 'round the', },
              },),
              planFor({
                chosen: { english: 'Fill the cats’ bowl.', quote: 'the cats', },
                receiving: { english: 'She said ‘the cats’ twice.', quote: 'the cats', },
              },),
              planFor({
                chosen: { english: 'She said ‘the dogs’ twice.', quote: 'the dogs', },
                receiving: { english: 'Fill the cats’ bowl.', quote: 'the cats', },
              },),
              planFor({
                chosen: { english: '‘Mio’ is here.', quote: 'Mio', },
                receiving: { english: 'Fill the parents’ bowl.', quote: 'the parents', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 5, end: 15, text: 'the parents’', },
              { kind: 'replace', start: 10, end: 19, text: '’round the', },
              { kind: 'replace', start: 10, end: 20, text: 'the parents’', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN A QUOTATION MARK COULD BE TAKEN FOR AN APOSTROPHE ANYWHERE IN THE ENGLISH, the '
            + 'reviewer\'s cases: a quotation that opens in another paragraph or inside a block quote, a straight one whose '
            + 'closer is in the next paragraph, an opener glued between letters, the low, reversed and angle single marks, '
            + 'and the control that a double-quoted speech beside a plural possessive does not forbid the splice',
          fn: async () => {
            /**
             The chosen words the cases carry, with no mark.
             */
            const chosen = { english: 'Then feed the dogs.', quote: 'Then feed the dogs', };
            const plain = { english: 'Fill the dogs bowl.', quote: 'the dogs', };
            expect([
              planFor({
                chosen,
                receiving: { english: '‘Fill the bowls.\n\nThen feed the kittens’.', quote: 'Then feed the kittens', },
              },),
              planFor({
                chosen: { english: 'Fill the bowls for the dogs.', quote: 'Fill the bowls for the dogs', },
                receiving: {
                  english: 'He said, \'Fill the bowls for the kittens.\n\nThen feed them.\'',
                  quote: 'Fill the bowls for the kittens',
                },
              },),
              planFor({
                chosen,
                receiving: { english: '> ‘Fill the bowls.\n>\n> Then feed the kittens’.', quote: 'Then feed the kittens', },
              },),
              planFor({
                chosen: { english: 'Tis for the birds and the dogs.', quote: 'Tis for the birds and the dogs', },
                receiving: {
                  english: '\'Tis for the kittens\n\nand for the cats\' bowl.',
                  quote: 'Tis for the kittens\n\nand for the cats',
                },
              },),
              planFor({
                chosen: plain,
                receiving: { english: 'She said‘fill the kittens’ bowl and left.', quote: 'the kittens', },
              },),
              planFor({
                chosen: plain,
                receiving: { english: 'She said ‚fill the kittens’ bowl and left.', quote: 'the kittens', },
              },),
              planFor({
                chosen: plain,
                receiving: { english: 'She said ‛fill the kittens’ bowl and left.', quote: 'the kittens', },
              },),
              planFor({
                chosen: plain,
                receiving: { english: 'She said ‹fill the kittens’ bowl and left.', quote: 'the kittens', },
              },),
              planFor({
                chosen: { english: 'Fill the dogs’ bowl.', quote: 'the dogs', },
                receiving: { english: '“Fill the kittens’ bowl,” said Mio.', quote: 'the kittens', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'replace', start: 6, end: 18, text: 'the dogs’', },
            ],);
          },
        },),
        it({
          name: 'KEEPS EVERY QUOTATION MARK OF THE PAGE WHEN A PLAN REPLACES, over a grid of quotation marks and apostrophes '
            + 'beside the words, in another paragraph or in a block quote, and a partner of any kind: any plan that replaces '
            + 'either keeps the sequence of all single marks outside the span or carries an apostrophe in',
          fn: async () => {
            const lefts = ['', '‘', '\'', '‚', '‹', '`',];
            const rights = ['', '’', '\'', '›',];
            const around = [
              ['Say ', ' now.',],
              ['Say ‘x. ', ' now.\n\nThen go.',],
              ['Say ‘x.\n\nThen ', ' now.',],
              ['> Say \'x.\n>\n> Then ', ' now.\'',],
              ['Say ', ' now.\n\nThen ’tis ‘go’.',],
            ] as const;
            const words = ['the kittens', 'Fill the bowls\n\nThen feed the kittens',];
            const places = words.flatMap(function ofWords(core,): readonly QuotedPlace[] {
              return around.flatMap(function ofAround([head, tail,],): readonly QuotedPlace[] {
                return lefts.flatMap(function ofLeft(left,): readonly QuotedPlace[] {
                  return rights.map(function ofRight(right,): QuotedPlace {
                    return { english: `${head}${left}${core}${right}${tail}`, quote: core, };
                  },);
                },);
              },);
            },);
            const located = locatedAmong({ places, },);
            /**
             Every character that is a quotation mark or an apostrophe, with its offset, in a page.
             */
            function quoteMarksIn({ text, }: { readonly text: string; },): string {
              return Array.from(text,)
                .filter(function isQuoteLike(character,): boolean {
                  return '\'‘’‚‛‹›`\u{02BC}'.includes(character,);
                },)
                .join('',);
            }
            const tally = { replaced: 0, broken: [] as string[], };
            for (const [chosen, first,] of located) {
              for (const [receiving, second,] of located) {
                const plan = planCompared({
                  chosen: first,
                  target: second,
                  framed: false,
                },);
                if (plan.kind !== 'replace')
                  continue;
                tally.replaced += 1;
                const applied = applyConsistencyEdits({
                  english: receiving.english,
                  edits: [{ id: 'x', start: plan.start, end: plan.end, text: plan.text, },],
                },).text;
                const outsideBefore = quoteMarksIn({
                  text: receiving.english.slice(0, plan.start,) + receiving.english.slice(plan.end,),
                });
                const outsideAfter = quoteMarksIn({
                  text: applied.slice(0, plan.start,) + applied.slice(plan.start + plan.text.length,),
                });
                // The fixture reads the page whole, so a mark the planner took for an apostrophe and deleted, whose partner
                // stood in another paragraph, shows as a loose mark that is gone.
                const looseKept = (unprovenLooseMarksIn({ text: applied, },)
                  .join('',)
                  === unprovenLooseMarksIn({ text: receiving.english, },)
                    .join('',)) || edgeApostropheIn({ text: plan.text, });
                if ((outsideBefore !== outsideAfter) || (!looseKept))
                  tally.broken.push(`${chosen.english} -> ${receiving.english}: ${applied}`,);
              }
            }
            expect(tally.broken.slice(0, 5,),).toEqual([],);
            expect(tally.replaced > 0,).toBe(true,);
            expect(located.length > 100,).toBe(true,);
          },
        },),
        it({
          name: 'WRITES AN APOSTROPHE THAT ARRIVES WITH THE CHOSEN WORDS IN THE FORM THE RECEIVING PLACE WROTE, the reviewer\'s '
            + 'D1 to D4: the apostrophes of the receiving words when they share one form, else those of the page outside '
            + 'the span when they share one, else as the chosen place wrote it; the modifier letter apostrophe is a form '
            + 'of its own; and words that differ only in that form are the same',
          fn: async () => {
            /**
             The words that go in, for a pair of places and the form of the page outside the receiving span.
             */
            function textOf(
              { chosen, receiving, page, }: {
                readonly chosen: QuotedPlace;
                readonly receiving: QuotedPlace;
                readonly page: readonly string[];
              },
            ): string {
              const [first, second,] = renderingsOf({ chosen, receiving, },);
              const plan = planConsistencyEdit({
                chosen: first,
                target: second,
                framed: false,
                marks: 'compared',
                pageApostrophe: function pageForm(): readonly string[] {
                  return page;
                },
              },);
              return (plan.kind === 'replace') ? plan.text : plan.kind;
            }
            expect([
              // D1: a straight edge apostrophe onto a place whose own words hold a curly one.
              textOf({
                chosen: { english: 'Fill the dogs\' bowl.', quote: 'the dogs', },
                receiving: { english: 'Fill the kittens’ bowl.', quote: 'the kittens', },
                page: [],
              },),
              // D2: an apostrophe inside a word, onto words that hold a curly one.
              textOf({
                chosen: { english: 'Fill the dog\'s bowl.', quote: 'the dog\'s bowl', },
                receiving: { english: 'Fill the kitten’s bowl.', quote: 'the kitten’s bowl', },
                page: [],
              },),
              // D3: the modifier letter apostrophe onto a curly place.
              textOf({
                chosen: { english: 'Fill the dogsʼ bowl.', quote: 'the dogs', },
                receiving: { english: 'Fill the kittens’ bowl.', quote: 'the kittens', },
                page: [],
              },),
              // D4: a curly apostrophe onto a straight place.
              textOf({
                chosen: { english: 'Fill the dogs’ bowl.', quote: 'the dogs', },
                receiving: { english: 'Fill the kittens\' bowl.', quote: 'the kittens', },
                page: [],
              },),
              // The receiving words hold none: the page outside the span gives the form.
              textOf({
                chosen: { english: 'Fill the dogs\' bowl.', quote: 'the dogs', },
                receiving: { english: 'Fill the family bowl.', quote: 'the family', },
                page: ['’',],
              },),
              // The receiving words hold none and the page's apostrophes are of mixed forms: as the chosen place wrote it.
              textOf({
                chosen: { english: 'Fill the dogs\' bowl.', quote: 'the dogs', },
                receiving: { english: 'Fill the family bowl.', quote: 'the family', },
                page: [],
              },),
              // An apostrophe inside a word and the page's form, onto words with none.
              textOf({
                chosen: { english: 'Fill the dog\'s bowl.', quote: 'the dog\'s bowl', },
                receiving: { english: 'Fill the family dish.', quote: 'the family dish', },
                page: ['\u{02BC}',],
              },),
              // Words that differ only in the form of an apostrophe are the same.
              textOf({
                chosen: { english: 'Fill the dog\'s bowl.', quote: 'the dog\'s bowl', },
                receiving: { english: 'Fill the dog’s bowl.', quote: 'the dog’s bowl', },
                page: [],
              },),
            ],).toEqual([
              'the dogs’',
              'the dog’s bowl',
              'the dogs’',
              'the dogs\'',
              'the dogs’',
              'the dogs\'',
              'the dogʼs bowl',
              'same',
            ],);
          },
        },),
        it({
          name: 'KEEPS THE FORM THE CHOSEN PLACE WROTE WHEN THE RECEIVING WORDS HOLD APOSTROPHES OF MORE THAN ONE FORM, and '
            + 'writes every arriving apostrophe in the one form the receiving words share when they share one',
          fn: async () => {
            const [chosen, receiving,] = renderingsOf({
              chosen: { english: 'Fill the dog\'s and the cat\'s bowl.', quote: 'the dog\'s and the cat\'s bowl', },
              receiving: { english: 'Fill the kitten’s and the pup\'s dish.', quote: 'the kitten’s and the pup\'s dish', },
            },);
            const [chosenOne, receivingOne,] = renderingsOf({
              chosen: { english: 'Fill the dog\'s and the cat\'s bowl.', quote: 'the dog\'s and the cat\'s bowl', },
              receiving: { english: 'Fill the kitten’s and the pup’s dish.', quote: 'the kitten’s and the pup’s dish', },
            },);
            expect([
              planConsistencyEdit({
                chosen,
                target: receiving,
                framed: false,
                marks: 'compared',
                pageApostrophe: function pageForm(): readonly string[] {
                  return [];
                },
              },),
              planConsistencyEdit({
                chosen: chosenOne,
                target: receivingOne,
                framed: false,
                marks: 'compared',
                pageApostrophe: function pageForm(): readonly string[] {
                  return ['\'',];
                },
              },),
            ],).toEqual([
              { kind: 'replace', start: 5, end: 36, text: 'the dog\'s and the cat\'s bowl', },
              { kind: 'replace', start: 5, end: 36, text: 'the dog’s and the cat’s bowl', },
            ],);
          },
        },),
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN THE WORDS DIFFER IN THE BLOCK BREAKS THEY HOLD, a blank line or a line that the '
            + 'parse reads as a new block: the page keeps its number of blocks only if the counts agree, the case of the '
            + 'review\'s G3 residue',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'The sun on the sill sent the kitten off to sleep.', quote: 'The sun on the sill sent the kitten off to sleep', },
                receiving: { english: 'The sunny window put the kitten\n\nto sleep said she.', quote: 'The sunny window put the kitten\n\nto sleep', },
              },),
              planFor({
                chosen: { english: 'The sun on the sill sent the kitten off to sleep.', quote: 'The sun on the sill sent the kitten off to sleep', },
                receiving: { english: 'The sunny window put the kitten\n\nto sleep\' said she.', quote: 'The sunny window put the kitten\n\nto sleep', },
              },),
              planFor({
                chosen: { english: 'The sun on the sill sent the kitten\n\noff to sleep.', quote: 'The sun on the sill sent the kitten\n\noff to sleep', },
                receiving: { english: 'The sunny window put the kitten to sleep.', quote: 'The sunny window put the kitten to sleep', },
              },),
              planFor({
                chosen: { english: 'The sun on the sill sent the kitten\n- off to sleep.', quote: 'The sun on the sill sent the kitten\n- off to sleep', },
                receiving: { english: 'The sunny window put the kitten to sleep.', quote: 'The sunny window put the kitten to sleep', },
              },),
            ],).toEqual([
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'STILL REPLACES WORDS THAT HOLD THE SAME NUMBER OF BLOCK BREAKS, a blank line on each side, or none',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'The sun on the sill\n\nsent the kitten off.', quote: 'The sun on the sill\n\nsent the kitten off', },
                receiving: { english: 'The sunny window\n\nput the kitten down.', quote: 'The sunny window\n\nput the kitten down', },
              },),
              planFor({
                chosen: { english: 'The sun on the sill sent the kitten off.', quote: 'The sun on the sill sent the kitten off', },
                receiving: { english: 'The sunny window put the kitten down.', quote: 'The sunny window put the kitten down', },
              },),
            ],).toEqual([
              { kind: 'replace', start: 0, end: 37, text: 'The sun on the sill\n\nsent the kitten off', },
              { kind: 'replace', start: 0, end: 36, text: 'The sun on the sill sent the kitten off', },
            ],);
          },
        },),
        it({
          name: 'LEAVES THE LOOSE MARKS THAT ARE NO PROVEN APOSTROPHE AS THEY WERE, over a grid of words with emphasis and '
            + 'quotation marks, a mark or none before and after them, and an opener before the place or a closer after it, at '
            + 'both places: any plan that replaces changes no text outside the span, keeps the sequence of loose marks that '
            + 'are no proven apostrophe, and attaches or detaches no mark outside the span',
          fn: async () => {
            const cores = ['the cats nap', 'the dogs nap', '*the cats nap*', 'the "cats" nap', 'the "dogs" nap',];
            const befores = ['', '\'', 'x’',];
            const afters = ['', '’', '’s', '\' x',];
            const leads = ['Fill ', '‘Fill ',];
            const tails = [' bowl.', ' bowl, the dogs’ nap.',];
            /**
             Every English of the grid, with the words judges quote.
             */
            const places = cores.flatMap(function ofCore(core,): readonly QuotedPlace[] {
              return befores.flatMap(function ofBefore(before,): readonly QuotedPlace[] {
                return afters.flatMap(function ofAfter(after,): readonly QuotedPlace[] {
                  return leads.flatMap(function ofLead(lead,): readonly QuotedPlace[] {
                    return tails.map(function ofTail(tail,): QuotedPlace {
                      return { english: `${lead}${before}${core}${after}${tail}`, quote: core, };
                    },);
                  },);
                },);
              },);
            },);
            /**
             The places of the grid the counter locates, which are all the property is about.
             */
            const located = locatedAmong({ places, },);
            /**
             What each pair came to, and every replacement that broke one of the three guarantees.
             */
            const tally = { replaced: 0, refused: 0, withEdgeMark: 0, broken: [] as string[], };
            for (const [chosen, first,] of located) {
              for (const [receiving, second,] of located) {
                const plan = planCompared({
                  chosen: first,
                  target: second,
                  framed: false,
                },);
                if (plan.kind === 'unavailable') {
                  tally.refused += 1;
                  continue;
                }
                if (plan.kind === 'same')
                  continue;
                tally.replaced += 1;
                if (SINGLE_MARKS.has(plan.text.at(-1,) ?? '',) || SINGLE_MARKS.has(plan.text.at(0,) ?? '',))
                  tally.withEdgeMark += 1;
                const applied = applyConsistencyEdits({
                  english: receiving.english,
                  edits: [{ id: 'x', start: plan.start, end: plan.end, text: plan.text, },],
                },).text;
                const shift = plan.text.length - (plan.end - plan.start);
                /**
                 Whether the text outside the replaced span is as it was.
                 */
                const outsideKept = (applied.slice(0, plan.start,) === receiving.english.slice(0, plan.start,))
                  && (applied.slice(plan.start + plan.text.length,) === receiving.english.slice(plan.end,));
                /**
                 Whether some mark outside the span was attached and is not, or the reverse.
                 */
                const attachedNow = new Set(attachedOffsetsIn({ text: applied, },),);
                const attachedBefore = new Set(attachedOffsetsIn({ text: receiving.english, },),);
                const turned = pageMarksOf({ text: receiving.english, },)
                  .filter(function standsOutside({ at, },): boolean {
                    return (at < plan.start) || (at >= plan.end);
                  },)
                  .some(function changedKind({ at, },): boolean {
                    return attachedBefore.has(at,) !== attachedNow.has((at < plan.start) ? at : (at + shift),);
                  },);
                // An apostrophe the words carry in may read as a quotation mark on a page that holds an opener, which the
                // pass catches where the page is known; any other plan keeps every loose mark.
                const sameLoose = (unprovenLooseMarksIn({ text: applied, },)
                  .join('',)
                  === unprovenLooseMarksIn({ text: receiving.english, },)
                    .join('',)) || edgeApostropheIn({ text: plan.text, });
                if (!(outsideKept && sameLoose && (!turned)))
                  tally.broken.push(`${chosen.english} -> ${receiving.english}: ${applied}`,);
              }
            }
            expect(tally.broken.slice(0, 5,),).toEqual([],);
            expect(tally.broken,).toEqual([],);
            expect(tally.replaced > 0,).toBe(true,);
            expect(tally.refused > 0,).toBe(true,);
            expect(tally.withEdgeMark > 0,).toBe(true,);
            expect(located.length > 100,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: `${planConsistencyEdit.name} beside a quotation mark that no reader takes for an apostrophe`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE MARK BESIDE A PICTOGRAPH, OR BEFORE OPENING MARKUP, AS A QUOTATION MARK IN EVERY READER, so the locator '
            + 'does not widen a span over the opener and no splice removes a quotation mark from a pair the page holds',
          fn: async () => {
            /**
             The Englishes of the examples: a quotation whose closing mark stands after a pictograph and before a digit, the same
             before `ss`, and a closing quotation mark before emphasis that opens after it.
             */
            const englishes = [
              'He said \'cats 🐱\'5 now.',
              'He said \'cats 🐱\'ss now.',
              '‘Go on.\n\nHe said ”’**cats**).',
            ];
            expect([
              englishes.map(function readingsOfEnglish(english,): readonly string[] {
                return [...markReadingsInEnglish({ english, },),].map(function shown([at, kind,],): string {
                  return `${String(at,)}:${kind}`;
                },);
              },),
              englishes.map(function provenIn(english,): readonly number[] {
                const { trailing, leading, } = provenApostrophesInEnglish({ english, },);
                return [...trailing, ...leading,];
              },),
            ],).toEqual([
              [['8:leading', '16:free',], ['8:leading', '16:free',], ['18:free',],],
              [[], [], [],],
            ],);
            const [first, second,] = renderingsOf({
              chosen: { english: 'Fill cats 🐱 bowl.', quote: 'cats 🐱', },
              receiving: { english: englishes[0] ?? '', quote: 'cats 🐱', },
            },);
            const [, third,] = renderingsOf({
              chosen: { english: 'Fill 猫 bowl.', quote: '猫', },
              receiving: { english: englishes[1] ?? '', quote: 'cats 🐱', },
            },);
            expect([
              // The span is the words alone and the opener stays outside it with the closer, so the pair is two kept marks.
              second.core,
              second.markBefore,
              second.markAfter,
              third.core,
              third.markBefore,
              third.markAfter,
              planCompared({ chosen: first, target: second, framed: false, },),
              planFor({
                chosen: { english: 'Fill 猫 bowl.', quote: '猫', },
                receiving: { english: englishes[1] ?? '', quote: 'cats 🐱', },
              },),
            ],).toEqual([
              'cats 🐱',
              '\'',
              '\'',
              'cats 🐱',
              '\'',
              '\'',
              { kind: 'unavailable', },
              { kind: 'unavailable', },
            ],);
          },
        },),
        it({
          name: 'CARRIES WORDS INTO EMPHASIS AFTER A CLOSING QUOTATION MARK WITHOUT MOVING THE MARK, an accepted outcome (the lead\'s '
            + 'ruling on the D7 c3 shape): the single mark before the emphasis is a free mark kept where it stands, and the '
            + '`’` the words add inside a word (`cat’s`) is a contraction, attached between two letters, and no quotation mark',
          fn: async () => {
            const english = '‘Go on.\n\nHe said ”’**cats**).';
            const plan = planFor({
              chosen: { english: 'Fill the cat’s bowl.', quote: 'cat’s', },
              receiving: { english, quote: 'cats', },
            },);
            /**
             The page the carry makes, none when the plan is no replacement.
             */
            const made = plan.kind === 'replace'
              ? applyConsistencyEdits({
                english,
                edits: [{ id: 'e', start: plan.start, end: plan.end, text: plan.text, },],
              },).text
              : plan.kind;
            expect(made,).toEqual('‘Go on.\n\nHe said ”’**cat’s**).',);
            // The mark before the emphasis stays free, and the one the words add reads attached, so no quotation mark is added.
            expect([...markReadingsInEnglish({ english: made, },),].map(function shown([at, kind,],): string {
              return `${String(at,)}:${kind}`;
            },),).toEqual(['18:free', '24:attached',],);
          },
        },),
      ],
    },),
    describe({
      name: `${planConsistencyEdit.name} when the receiving place's apostrophe form changes how the words parse`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A SPLICE UNAVAILABLE WHEN WRITING THE APOSTROPHE IN THE RECEIVING PLACE\'S FORM TURNS A CLOSING UNDERSCORE '
            + 'INTO A CHARACTER OF THE WORDS: the modifier letter apostrophe is a letter to the Markdown parse, so `_cats_ʼ` holds '
            + 'a loose mark that `_cats_’` (a mark past real emphasis) did not, and the receiving words held none',
          fn: async () => {
            /**
             The chosen place, one per way the grid wrote the mark after the underscored word, and the receiving place that
             writes its apostrophe as the modifier letter.
             */
            const cases = [
              ['He said *_cats_’ 5 now.', 'He said 5 ʼcats 🐱*猫 now.', 'cats 🐱',],
              ['He said *_cats_’ now.', 'He said ʼthe cats猫 now.', 'the cats猫',],
              ['He said 5 "_cats_’ now.', 'He said 5 ʼcats…s now.', 'cats…',],
              ['He said *_cats_\' now.', 'He said “cats…ʼ5 now.', 'cats…',],
              ['He said (_cats_\' 5 now.', 'He said 5 ʼcats…」s now.', 'cats…',],
            ] as const;
            expect(cases.map(function planOf([chosenEnglish, english, quote,],): ConsistencyPlan['kind'] {
              const [chosen, target,] = renderingsOf({
                chosen: { english: chosenEnglish, quote: '_cats_', },
                receiving: { english, quote, },
              },);
              // The pass hands the planner the receiving English and the form of the page's other apostrophes, as the grid did.
              return planConsistencyEdit({
                chosen,
                target,
                framed: false,
                marks: 'compared',
                english,
                pageApostrophe: () => pageApostropheFormOf({
                  english,
                  start: target.start,
                  end: target.start + target.core.length,
                },),
              },).kind;
            },),).toEqual(cases.map(function unavailable(): ConsistencyPlan['kind'] {
              return 'unavailable';
            },),);
          },
        },),
        it({
          name: 'STILL REPLACES WHEN THE APOSTROPHE FORM LEAVES THE PARSE OF THE WORDS AS IT WAS: real emphasis with an asterisk '
            + 'closes next to the modifier letter apostrophe, so the words hold the loose marks the receiving words hold',
          fn: async () => {
            /**
             The planned replacement for the asterisk twin of the underscored words.
             */
            const plan = planFor({
              chosen: { english: 'He said *cats*’ now.', quote: '*cats*', },
              receiving: { english: 'He said 5 ʼcats…s now.', quote: 'cats…', },
            },);
            expect(plan.kind === 'replace' ? plan.text : plan.kind,).toBe('*cats*ʼ',);
          },
        },),
      ],
    },),
    describe({
      name: `${planConsistencyEdit.name} beside a mark past a literal emphasis character`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A GAIN UNAVAILABLE BESIDE A MARK THAT A LITERAL `*`, `_` OR `~` SEPARATES FROM THE SPAN, the J1 shapes the '
            + 'grid found after the parse gate: the refusal run is wide, so an apostrophe that arrives beside a kept mark one literal '
            + 'emphasis character away would stand as a doubled mark (`\'*\'5 cats`), whatever the parse makes of the character',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'He said \'5 cats now. Then it’s fine.', quote: '5 cats', },
                receiving: { english: 'He said \'*tom now. Then ‘go’.', quote: 'tom', },
              },).kind,
              planFor({
                chosen: { english: 'He said \'rock\'n\'roll* now. Then it’s fine.', quote: 'rock\'n\'roll', },
                receiving: { english: 'He said \'*_cats_) now.\'', quote: '_cats_', },
              },).kind,
              planFor({
                chosen: { english: 'He said ’5 cats*s now.', quote: '5 cats', },
                receiving: { english: 'He said \'*cats) now.', quote: 'cats', },
              },).kind,
              planFor({
                chosen: { english: 'Mio’s dish. He said ʼgoin now.', quote: 'goin', },
                receiving: { english: 'He said \'*tom"s now.', quote: 'tom', },
              },).kind,
              planFor({
                chosen: { english: 'He said \'tom) now. Then it’s fine.', quote: 'tom', },
                receiving: { english: 'He said \'*cats” now. Then it’s fine.', quote: 'cats', },
              },).kind,
            ],).toEqual(['unavailable', 'unavailable', 'unavailable', 'unavailable', 'unavailable',],);
          },
        },),
      ],
    },),
    describe({
      name: `${planConsistencyEdit.name} beside the low, reversed and angle single marks`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A GAIN UNAVAILABLE WHEN ONE OF THE LOW, REVERSED OR ANGLE SINGLE MARKS STANDS A REFUSAL RUN AWAY, at either '
            + 'edge: the pass counts these as single marks, the record stores them, and an apostrophe that arrives beside one '
            + 'would stand as a doubled mark',
          fn: async () => {
            /**
             The four marks the counter did not read as marks beside a span before the record stored them.
             */
            const marks = ['\u{201A}', '\u{201B}', '‹', '›',] as const;
            expect(marks.flatMap(function plansOf(mark,): readonly ConsistencyPlan['kind'][] {
              return [
                planFor({
                  chosen: { english: 'Say **the parents’** now.', quote: 'the parents', },
                  receiving: { english: `Say **the family**${mark} now.`, quote: 'the family', },
                },).kind,
                planFor({
                  chosen: { english: 'Say **’round the family** now.', quote: 'round the family', },
                  receiving: { english: `Say ${mark}**around the family** now.`, quote: 'around the family', },
                },).kind,
              ];
            },),).toEqual(marks.flatMap(function unavailable(): readonly ConsistencyPlan['kind'][] {
              return ['unavailable', 'unavailable',];
            },),);
          },
        },),
        it({
          name: 'STILL PLANS A GAIN WHERE NO SINGLE MARK STANDS BESIDE THE SPAN OR A REFUSAL RUN AWAY, the guillemets being no '
            + 'single marks, so the refusal sees exactly the eight',
          fn: async () => {
            expect([
              planFor({
                chosen: { english: 'Say **the parents’** now.', quote: 'the parents', },
                receiving: { english: 'Say **the family**» now.', quote: 'the family', },
              },).kind,
              planFor({
                chosen: { english: 'Say **the parents’** now.', quote: 'the parents', },
                receiving: { english: 'Say **the family** now.', quote: 'the family', },
              },).kind,
            ],).toEqual(['replace', 'replace',],);
          },
        },),
      ],
    },),
    describe({
      name: bringsEdgeApostrophe.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN THE WORDS THAT GO IN HOLD AN APOSTROPHE AT AN EDGE WHERE THE RECEIVING WORDS HOLD NONE, at either '
            + 'edge, and false when the receiving words hold one there, or the words hold none',
          fn: async () => {
            expect([
              bringsEdgeApostrophe({ text: 'the parents’', target: locatedAt({ core: 'the family', start: 0, },), },),
              bringsEdgeApostrophe({ text: '’round the', target: locatedAt({ core: 'about the', start: 0, },), },),
              bringsEdgeApostrophe({ text: 'the parents’', target: locatedAt({ core: 'the folks’', start: 0, },), },),
              bringsEdgeApostrophe({ text: 'the parents', target: locatedAt({ core: 'the folks’', start: 0, },), },),
              bringsEdgeApostrophe({ text: 'the cat’s bowl', target: locatedAt({ core: 'the family', start: 0, },), },),
            ],).toEqual([true, true, false, false, false,],);
          },
        },),
        it({
          name: 'READS AN EDGE APOSTROPHE BESIDE A COMPATIBILITY LETTER AS BESIDE A WORD, by the neighbour test of the consistency '
            + 'pass: a mathematical script name (`𝓠𝓾𝓪𝓷’`) and a fullwidth letter bring one, and a micro sign brings a mark too '
            + '(the ruling on gains: a single mark at an edge arrives whatever stands inside it, and the page proves it or not)',
          fn: async () => {
            expect([
              bringsEdgeApostrophe({
                text: 'the \u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’',
                target: locatedAt({ core: 'the family', start: 0, },),
              },),
              bringsEdgeApostrophe({
                text: 'the \u{FF21}’',
                target: locatedAt({ core: 'the family', start: 0, },),
              },),
              bringsEdgeApostrophe({
                text: '’\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7} bowl',
                target: locatedAt({ core: 'about the', start: 0, },),
              },),
              bringsEdgeApostrophe({
                text: 'the \u{00B5}’',
                target: locatedAt({ core: 'the family', start: 0, },),
              },),
            ],).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'IS TRUE FOR A SINGLE MARK AT AN EDGE OF THE WORDS WHATEVER STANDS INSIDE IT, the parse of the words and the letter '
            + 'beside the mark making no difference: the receiving words hold no edge apostrophe at that edge (a mark beside a space '
            + 'is none), so the mark arrives and the page must prove it (`_cats_ʼ` holds no real emphasis, so its mark reads free)',
          fn: async () => {
            expect([
              bringsEdgeApostrophe({ text: '_cats_ʼ', target: locatedAt({ core: 'ʼcats 🐱', start: 0, },), },),
              bringsEdgeApostrophe({ text: '_cats_ʼ', target: locatedAt({ core: 'the family', start: 0, },), },),
              bringsEdgeApostrophe({ text: 'the cats ʼ', target: locatedAt({ core: 'the family', start: 0, },), },),
              bringsEdgeApostrophe({ text: 'ʼ 5 cats', target: locatedAt({ core: 'about the', start: 0, },), },),
              bringsEdgeApostrophe({ text: '_cats_ʼ', target: locatedAt({ core: 'the folks’', start: 0, },), },),
              bringsEdgeApostrophe({ text: '_cats_ʼ', target: locatedAt({ core: 'the folks ʼ', start: 0, },), },),
              bringsEdgeApostrophe({ text: '_cats_', target: locatedAt({ core: 'the family', start: 0, },), },),
            ],).toEqual([true, true, true, true, false, true, false,],);
          },
        },),
      ],
    },),
    describe({
      name: applyConsistencyEdits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES EVERY EDIT AGAINST THE ENGLISH IT WAS LOCATED IN, cutting the text once between the edits, so one edit '
            + 'moves no other',
          fn: async () => {
            const applied = applyConsistencyEdits({
              english: 'The kitten naps. The kitten dozes.',
              edits: [
                { id: 'first', start: 4, end: 10, text: 'cat', },
                { id: 'second', start: 21, end: 27, text: 'young cat', },
              ],
            },);
            expect(applied,).toEqual({
              text: 'The cat naps. The young cat dozes.',
              applied: ['first', 'second',],
              overlapped: [],
            },);
          },
        },),
        it({
          name: 'LEAVES OUT THE LATER OF TWO EDITS THAT OVERLAP, and reports it, the earlier edit winning',
          fn: async () => {
            const applied = applyConsistencyEdits({
              english: 'The kitten naps in the sun.',
              edits: [
                { id: 'earlier', start: 4, end: 15, text: 'cat sleeps', },
                { id: 'later', start: 11, end: 20, text: 'dozes in', },
              ],
            },);
            expect(applied,).toEqual({
              text: 'The cat sleeps in the sun.',
              applied: ['earlier',],
              overlapped: ['later',],
            },);
          },
        },),
        it({
          name: 'MAKES TWO EDITS THAT TOUCH WITHOUT OVERLAPPING, the span end being exclusive',
          fn: async () => {
            const applied = applyConsistencyEdits({
              english: 'abcdef',
              edits: [
                { id: 'left', start: 0, end: 3, text: 'XY', },
                { id: 'right', start: 3, end: 6, text: 'Z', },
              ],
            },);
            expect(applied,).toEqual({
              text: 'XYZ',
              applied: ['left', 'right',],
              overlapped: [],
            },);
          },
        },),
        it({
          name: 'CHANGES NOTHING for no edits',
          fn: async () => {
            expect(applyConsistencyEdits({ english: 'The cat naps.', edits: [], },),).toEqual({
              text: 'The cat naps.',
              applied: [],
              overlapped: [],
            },);
          },
        },),
        it({
          name: 'PLANS A MARK THAT STANDS PAST A RUN OF MARKUP OR CLOSING CHARACTERS AS A KEPT MARK, so an edge apostrophe is '
            + 'never written beside it: unavailable on the end side for a mark after emphasis, strong, strikethrough, a '
            + 'straight or curly double mark, a bracket or a link, and a straight mark kept as it stands',
          fn: async () => {
            /**
             The plan kind for chosen words that carry a trailing apostrophe onto a place whose own mark stands past markup.
             */
            function kindFor({ receiving, }: { readonly receiving: string; }): string {
              const [chosen, target,] = renderingsOf({
                chosen: { english: 'Fill the dogs’ bowl.', quote: 'dogs’', },
                receiving: { english: receiving, quote: 'kittens', },
              },);
              return planConsistencyEdit({ chosen, target, framed: false, marks: 'compared', },).kind;
            }
            expect([
              kindFor({ receiving: 'Fill the *kittens*’ bowl.', },),
              kindFor({ receiving: 'Fill the **kittens**’ bowl.', },),
              kindFor({ receiving: 'Fill the "kittens"’ bowl.', },),
              kindFor({ receiving: 'Fill the “kittens”’ bowl.', },),
              kindFor({ receiving: 'Fill the (kittens)’ bowl.', },),
              kindFor({ receiving: 'Fill the [kittens](https://cats.example/k)’ bowl.', },),
              kindFor({ receiving: 'Fill the ~~kittens~~’ bowl.', },),
              kindFor({ receiving: 'Fill the *kittens*\' bowl.', },),
              kindFor({ receiving: 'Fill the kittens’ bowl.', },),
            ],).toEqual([
              'unavailable',
              'unavailable',
              'unavailable',
              'unavailable',
              'unavailable',
              'unavailable',
              'unavailable',
              'unavailable',
              'replace',
            ],);
          },
        },),
        it({
          name: 'PLANS A MARK BEFORE A RUN OF MARKUP OR OPENING CHARACTERS AS A KEPT MARK TOO, so a leading apostrophe is never '
            + 'written beside it, a literal `*` between the mark and the words included (the refusal run is wide, the ruling on the '
            + 'J1 shapes), and plans the same words as a replacement where no such mark stands',
          fn: async () => {
            /**
             The plan kind for chosen words that carry a leading apostrophe onto a place.
             */
            function kindFor({ receiving, }: { readonly receiving: string; }): string {
              const [chosen, target,] = renderingsOf({
                chosen: { english: 'Fill ’5 cats now.', quote: '5 cats', },
                receiving: { english: receiving, quote: '5 cats', },
              },);
              return planConsistencyEdit({ chosen, target, framed: false, marks: 'compared', },).kind;
            }
            expect([
              kindFor({ receiving: 'He said \'*5 cats* now.', },),
              kindFor({ receiving: 'He said ’(_5 cats_ now.', },),
              kindFor({ receiving: 'He said ‘"5 cats now.', },),
              kindFor({ receiving: 'He said *5 cats* now.', },),
              kindFor({ receiving: 'He said 5 cats now.', },),
              kindFor({ receiving: 'He said \'*5 cats now.', },),
            ],).toEqual(['unavailable', 'unavailable', 'unavailable', 'replace', 'replace', 'unavailable',],);
          },
        },),
        it({
          name: 'PLANS A GAIN ONLY WHERE THE KEPT MARK PAST MARKUP IS ON THE GAINED SIDE, and a place with no mark to gain is '
            + 'unaffected: chosen words with no apostrophe over a place with emphasis and a mark are a replacement',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs bowl.', quote: 'dogs', },
              receiving: { english: 'Fill the *kittens*’ bowl.', quote: 'kittens', },
            },);
            expect(planConsistencyEdit({ chosen, target, framed: false, marks: 'compared', },).kind,).toBe('replace',);
          },
        },),
        it({
          name: 'PLANS A SPLICE THAT WOULD CHANGE HOW A MARK OUTSIDE THE WORDS IS READ AS UNAVAILABLE WHEN THE CALLER HOLDS THE '
            + 'ENGLISH, and as a replacement when it does not',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              // The mark after `()` has no letter before the run of closing characters, so it is a loose mark beside the words;
              // after `(here)` it would read as a trailing mark and be taken into the words.
              chosen: { english: 'Say Come ()\' now.', quote: 'Come ()', },
              receiving: { english: 'He said cats 🐱\'s now.', quote: 'cats 🐱', },
            },);
            expect([
              planConsistencyEdit({
                chosen,
                target,
                framed: false,
                marks: 'compared',
                english: 'He said cats 🐱\'s now.',
              },).kind,
              planConsistencyEdit({ chosen, target, framed: false, marks: 'compared', },).kind,
            ],).toEqual(['unavailable', 'replace',],);
          },
        },),
        it({
          name: 'PLANS EVERY SPLICE AS UNAVAILABLE WHEN THE PAGE HOLDS A REFERENCE THAT DECODES TO A QUOTATION-LIKE CHARACTER, '
            + 'wherever it stands and whether or not the words bring an edge apostrophe, asking for the references of any '
            + 'splice and of none that changes nothing, and plans the same splice as a replacement for a page with no such '
            + 'reference and for a caller that holds no page',
          fn: async () => {
            /**
             How many times the planner asked for the references of the page.
             */
            const asked = { times: 0, };
            /**
             The plan kind for chosen words over a place that keeps no mark, given what the page holds.
             */
            function kindFor({ chosenEnglish, chosenQuote, page, receiving = { english: 'Fill the kittens bowl.', quote: 'kittens', }, }: {
              readonly chosenEnglish: string;
              readonly chosenQuote: string;
              readonly page?: readonly string[];
              readonly receiving?: QuotedPlace;
            },): string {
              const [chosen, target,] = renderingsOf({
                chosen: { english: chosenEnglish, quote: chosenQuote, },
                receiving,
              },);
              return planConsistencyEdit({
                chosen,
                target,
                framed: false,
                marks: 'compared',
                ...((page === undefined)
                  ? {}
                  : {
                    pageReference: function pageReferences(): readonly string[] {
                      asked.times += 1;
                      return page;
                    },
                  }),
              },).kind;
            }
            expect([
              kindFor({ chosenEnglish: 'Fill the dogs’ bowl.', chosenQuote: 'dogs’', page: ['&#39;',], },),
              kindFor({ chosenEnglish: 'Fill the dogs’ bowl.', chosenQuote: 'dogs’', page: ['&lsquo;', '&rdquo;',], },),
              kindFor({ chosenEnglish: 'Fill the dogs’ bowl.', chosenQuote: 'dogs’', page: [], },),
              kindFor({ chosenEnglish: 'Fill the dogs’ bowl.', chosenQuote: 'dogs’', },),
              kindFor({
                chosenEnglish: 'Guard her ’round the cats.',
                chosenQuote: 'round the',
                receiving: { english: 'Guard her about the dogs.', quote: 'about the', },
                page: ['&#39;',],
              },),
            ],).toEqual(['unavailable', 'unavailable', 'replace', 'replace', 'unavailable',],);
            expect(asked.times,).toBe(4,);
            asked.times = 0;
            expect([
              kindFor({ chosenEnglish: 'Fill the dogs bowl.', chosenQuote: 'dogs', page: ['&#39;',], },),
              kindFor({ chosenEnglish: 'Fill the dog\'s bowl.', chosenQuote: 'dog\'s', page: ['&#39;',], },),
              kindFor({ chosenEnglish: 'Fill the dogs bowl.', chosenQuote: 'dogs', page: [], },),
              kindFor({ chosenEnglish: 'Fill the dogs bowl.', chosenQuote: 'dogs', },),
            ],).toEqual(['unavailable', 'unavailable', 'replace', 'replace',],);
            expect(asked.times,).toBe(3,);
            asked.times = 0;
            expect([
              kindFor({ chosenEnglish: 'Fill the kittens bowl.', chosenQuote: 'kittens', page: ['&#39;',], },),
            ],).toEqual(['same',],);
            expect(asked.times,).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES AN EDIT WHOSE SPAN LIES OUTSIDE THE ENGLISH OR IS EMPTY, since offsets located in that English '
            + 'cannot',
          fn: async () => {
            /**
             Applies one edit to a short English and reports the refusal.
             */
            function refusalOf({ start, end, }: { readonly start: number; readonly end: number; },): string {
              return String(caught(function act(): unknown {
                return applyConsistencyEdits({
                  english: 'The cat.',
                  edits: [{ id: 'x', start, end, text: 'y', },],
                },);
              },),);
            }
            /**
             The refusal for a span.
             */
            function message({ spans, }: { readonly spans: string; },): string {
              return `ConsistencyInvariantError: unreachable: edit x spans ${spans} in an English whose length is 8, and a span `
                + 'located in that English lies inside it and is not empty';
            }
            expect([
              refusalOf({ start: -1, end: 2, },),
              refusalOf({ start: 3, end: 9, },),
              refusalOf({ start: 4, end: 4, },),
            ],).toEqual([
              message({ spans: '-1 to 2', },),
              message({ spans: '3 to 9', },),
              message({ spans: '4 to 4', },),
            ],);
          },
        },),
      ],
    },),
  ],
},);
