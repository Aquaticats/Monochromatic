/**
 Tests for counting a repeated wording's ballots: which quote a strict majority
 of the usable ballots located, which place won, which places a majority keeps,
 and why a group stays as it stands.

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
  type ConsistencyEnglish,
  type ConsistencyVoice,
  countConsistency,
  type PageEnglish,
  winnerOf,
} from '../dist/final/node/index.mjs';
import { countedList, } from '@monochromatic-dev/translation-repair-test-fixture/ts/element-reads.test-fixture.ts';

/**
 English of the first place.
 */
const FIRST: ConsistencyEnglish = {
  kind: 'present',
  text: 'The sunny window put the kitten to sleep. So this cat must be a chubby little treasure.',
};

/**
 English of the second place.
 */
const SECOND: ConsistencyEnglish = {
  kind: 'present',
  text: 'In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too.',
};

/**
 The two places of a repeated sentence.
 */
const PLACES: readonly { readonly label: string; readonly english: ConsistencyEnglish; }[] = [
  { label: 'OCCURRENCE 1', english: FIRST, },
  { label: 'OCCURRENCE 2', english: SECOND, },
];

/**
 Quote of the first place that stands once in its English.
 */
const QUOTE_ONE = 'The sunny window put the kitten to sleep';

/**
 Quote of the second place that stands once in its English.
 */
const QUOTE_TWO = 'the sunlight on the sill lulled the kitten to sleep';

/**
 Fields of a ballot a case replaces.
 */
type VoiceOverrides = {
  readonly reason?: string;
  readonly quotes?: ConsistencyVoice['quotes'];
  readonly choice?: string;
  readonly keep?: readonly string[];
};

/**
 Builds one heard ballot, by default usable, choosing the first place with
 both quotes given.

 @param seat - seat that answered

 @param over - fields this case replaces

 @returns Ballot with its seat

 @example
 ```ts
 const voice = voiceOf({ seat: 'a', over: { choice: 'OCCURRENCE 2', }, },);
 ```
 */
function voiceOf(
  {
    seat,
    over = {},
  }: {
    readonly seat: string;
    readonly over?: VoiceOverrides;
  },
): ConsistencyVoice {
  return {
    seat,
    reason: 'The first rendering is the more faithful.',
    quotes: [
      { label: 'OCCURRENCE 1', quote: QUOTE_ONE, },
      { label: 'OCCURRENCE 2', quote: QUOTE_TWO, },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
    ...over,
  };
}

/**
 Three ballots alike.

 @param over - fields this case replaces in each

 @returns Three ballots from three seats

 @example
 ```ts
 const voices = threeAlike({ over: { choice: 'OCCURRENCE 2', }, },);
 ```
 */
function threeAlike({ over = {}, }: { readonly over?: VoiceOverrides; } = {},): readonly ConsistencyVoice[] {
  return ['a', 'b', 'c',].map(function oneSeat(seat,): ConsistencyVoice {
    return voiceOf({ seat, over, },);
  },);
}

/**
 Ballots that each quote the first place by one of the given quotes.

 @param quotes - what each ballot quotes for the first place, one ballot for each

 @returns Ballots, from distinct seats

 @example
 ```ts
 const voices = ballotsQuoting({ quotes: ['the cats\'', 'the cats',], },);
 ```
 */
function ballotsQuoting({ quotes, }: { readonly quotes: readonly string[]; },): readonly ConsistencyVoice[] {
  return quotes.map(function ballotFor(quote, at,): ConsistencyVoice {
    return voiceOf({
      seat: `ballot-${String(at,)}`,
      over: { quotes: [{ label: 'OCCURRENCE 1', quote, },], },
    },);
  },);
}

/**
 Counts three alike ballots over the two places, a repeated run.

 @param voices - ballots heard

 @param quorum - voices the gather closed on

 @returns Count

 @example
 ```ts
 const count = countOver({ voices: threeAlike(), quorum: 2, },);
 ```
 */
function countOver(
  {
    voices,
    quorum = 2,
    places = PLACES,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly quorum?: number;
    readonly places?: readonly {
      readonly label: string;
      readonly english: ConsistencyEnglish;
      readonly page?: PageEnglish;
    }[];
  },
): ReturnType<typeof countConsistency> {
  return countConsistency({
    voices,
    places,
    framed: false,
    quorum,
  },);
}

/**
 The rendering of the first place when every ballot quotes it as given.

 @param english - first place's English

 @param quote - what every ballot quotes

 @returns Rendering of the first place

 @example
 ```ts
 const rendering = firstRenderingOf({ english: 'Fill the parents’ bowl.', quote: 'the parents', },);
 ```
 */
function firstRenderingOf(
  {
    english,
    quote,
    page,
  }: {
    readonly english: string;
    readonly quote: string;
    readonly page?: PageEnglish;
  },
): unknown {
  return countOver({
    voices: ballotsQuoting({ quotes: [quote, quote, quote,], },),
    places: [
      {
        label: 'OCCURRENCE 1',
        english: { kind: 'present', text: english, },
        ...((page === undefined) ? {} : { page, }),
      },
      PLACES[1] as (typeof PLACES)[number],
    ],
  },).renderings[0];
}

/**
 One input of the review's finding about places located once before the apostrophe became no part of a span: an English
 where the same words stand twice, once beside a single mark and once without, and what a judge quoting the first
 stand copies.
 */
type RepeatedStand = {
  /**
   The English of the place.
   */
  readonly english: string;

  /**
   The quote with the mark beside the words, as a judge who copied it writes it.
   */
  readonly marked: string;

  /**
   The quote without the mark.
   */
  readonly bare: string;

  /**
   The rendering the counter gives when the ballots choose the stand with the mark.
   */
  readonly located: unknown;
};

/**
 The five inputs of finding F6 of the review.
 */
const F6_INPUTS: readonly RepeatedStand[] = [
  {
    english: 'Fill the cats\' bowl. Then the cats nap.',
    marked: 'the cats\'',
    bare: 'the cats',
    located: { kind: 'located', quote: 'the cats\'', core: 'the cats\'', start: 5, },
  },
  {
    english: 'He was goin\' home, and goin there.',
    marked: 'goin\'',
    bare: 'goin',
    located: { kind: 'located', quote: 'goin\'', core: 'goin\'', start: 7, },
  },
  {
    english: '\'Tis the season. Tis the season.',
    marked: '\'Tis',
    bare: 'Tis',
    located: { kind: 'located', quote: '\'Tis', core: '\'Tis', start: 0, },
  },
  {
    english: 'The \'90s cats and the 90s cats.',
    marked: '\'90s cats',
    bare: '90s cats',
    located: { kind: 'located', quote: '\'90s cats', core: '\'90s cats', start: 4, },
  },
  {
    english: 'She said \'em all and em none.',
    marked: '\'em',
    bare: 'em',
    located: { kind: 'located', quote: '\'em', core: '\'em', start: 9, },
  },
];

/**
 The rendering the counter gives the first place when some ballots copied the mark beside the words and the others did not.

 @param input - English and two ways of quoting it

 @param copying - how many ballots copied the mark

 @param ballots - how many ballots there are, all usable

 @param framed - whether the group is a frame with a slot

 @returns First place's rendering

 @example
 ```ts
 const rendering = locatedBy({ input: F6_INPUTS[0], copying: 2, ballots: 3, framed: false, },);
 ```
 */
function locatedBy(
  {
    input,
    copying,
    ballots,
    framed,
  }: {
    readonly input: RepeatedStand;
    readonly copying: number;
    readonly ballots: number;
    readonly framed: boolean;
  },
): unknown {
  return countConsistency({
    voices: Array.from({ length: ballots, },).map(function voiceFor(_none, at,): ConsistencyVoice {
      return voiceOf({
        seat: `seat-${String(at,)}`,
        over: { quotes: [{ label: 'OCCURRENCE 1', quote: (at < copying) ? input.marked : input.bare, },], },
      },);
    },),
    places: [
      { label: 'OCCURRENCE 1', english: { kind: 'present', text: input.english, }, },
      PLACES[1] as (typeof PLACES)[number],
    ],
    framed,
    quorum: 2,
  },).renderings[0];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: countConsistency.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LOCATES A QUOTE A STRICT MAJORITY GAVE that stands once in the place\'s English, with its offset',
          fn: async () => {
            const count = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', },),
                voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'something else', },], }, },),
              ],
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: QUOTE_ONE,
              core: QUOTE_ONE,
              start: 0,
            },);
            expect(count.renderings[1],).toEqual({
              kind: 'located',
              quote: QUOTE_TWO,
              core: QUOTE_TWO,
              start: 16,
            },);
          },
        },),
        it({
          name: 'LOCATES A QUOTE THAT CARRIES ITS CLOSING PUNCTUATION by its words, the place\'s own span being the quote and the core',
          fn: async () => {
            const withStop = `${QUOTE_ONE}.`;
            const count = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: withStop, },], }, },),
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: QUOTE_ONE,
              core: QUOTE_ONE,
              start: 0,
            },);
          },
        },),
        it({
          name: 'RECORDS THE SINGLE MARK THE ENGLISH HOLDS DIRECTLY BEFORE AND AFTER A LOCATED SPAN, as the English writes it, '
            + 'the span itself covering it only where it is a proven apostrophe, and records none where none stands there or '
            + 'only a double mark does',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'the cats\'', },
                    { label: 'OCCURRENCE 2', quote: 'Tis the season', },
                    { label: 'OCCURRENCE 3', quote: 'the cats', },
                    { label: 'OCCURRENCE 4', quote: 'the cats', },
                  ],
                },
              },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'Fill the cats’ bowl.', }, },
                { label: 'OCCURRENCE 2', english: { kind: 'present', text: 'He said ‘Tis the season’ again.', }, },
                { label: 'OCCURRENCE 3', english: { kind: 'present', text: 'Fill the cats bowl.', }, },
                { label: 'OCCURRENCE 4', english: { kind: 'present', text: 'Fill "the cats" bowl.', }, },
              ],
            },);
            expect(count.renderings,).toEqual([
              { kind: 'located', quote: 'the cats’', core: 'the cats’', start: 5, },
              { kind: 'located', quote: 'Tis the season', core: 'Tis the season', start: 9, markBefore: '‘', markAfter: '’', },
              { kind: 'located', quote: 'the cats', core: 'the cats', start: 5, },
              { kind: 'located', quote: 'the cats', core: 'the cats', start: 6, },
            ],);
          },
        },),
        it({
          name: 'RECORDS THE SINGLE MARK PAST A RUN OF MARKUP OR CLOSING CHARACTERS AFTER A SPAN, AND PAST A RUN OF MARKUP OR '
            + 'OPENING CHARACTERS BEFORE IT, apart from the mark directly beside, which it leaves as it was',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'the kittens', },
                    { label: 'OCCURRENCE 2', quote: '5 cats', },
                    { label: 'OCCURRENCE 3', quote: 'the cats', },
                    { label: 'OCCURRENCE 4', quote: 'the pups', },
                  ],
                },
              },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'Fill *the kittens*’ bowl.', }, },
                { label: 'OCCURRENCE 2', english: { kind: 'present', text: 'He said \'*5 cats* now.', }, },
                { label: 'OCCURRENCE 3', english: { kind: 'present', text: 'Fill the cats bowl.', }, },
                { label: 'OCCURRENCE 4', english: { kind: 'present', text: 'Fill (“the pups”)’ bowl.', }, },
              ],
            },);
            expect(count.renderings,).toEqual([
              { kind: 'located', quote: 'the kittens', core: 'the kittens', start: 6, markPastAfter: '’', },
              { kind: 'located', quote: '5 cats', core: '5 cats', start: 10, markPastBefore: '\'', },
              { kind: 'located', quote: 'the cats', core: 'the cats', start: 5, },
              { kind: 'located', quote: 'the pups', core: 'the pups', start: 7, markPastAfter: '’', },
            ],);
          },
        },),
        it({
          name: 'RECORDS THE MARK PAST A LONE EMPHASIS CHARACTER TOO, on either side of a span: the refusal run stays wide (the ruling '
            + 'on the J1 shapes), so a `*`, `_` or `~` the parse makes no delimiter is still a character of the run, and a gain '
            + 'is refused beside the mark beyond it, whatever the mark reads as',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: '5 cats', },
                    { label: 'OCCURRENCE 2', quote: 'the kittens', },
                    { label: 'OCCURRENCE 3', quote: 'cats', },
                    { label: 'OCCURRENCE 4', quote: 'tom', },
                  ],
                },
              },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'He said \'*5 cats now.', }, },
                { label: 'OCCURRENCE 2', english: { kind: 'present', text: 'Fill the kittens*’ bowl and *', }, },
                { label: 'OCCURRENCE 3', english: { kind: 'present', text: 'He said 猫_cats_’ now.', }, },
                { label: 'OCCURRENCE 4', english: { kind: 'present', text: 'He said \'*tom now. Then ‘go’.', }, },
              ],
            },);
            expect(count.renderings,).toEqual([
              { kind: 'located', quote: '5 cats', core: '5 cats', start: 10, markPastBefore: '\'', },
              { kind: 'located', quote: 'the kittens', core: 'the kittens', start: 5, markPastAfter: '’', },
              { kind: 'located', quote: 'cats', core: 'cats', start: 10, markPastAfter: '’', },
              { kind: 'located', quote: 'tom', core: 'tom', start: 10, markPastBefore: '\'', },
            ],);
          },
        },),
        it({
          name: 'RECORDS THE MARKS BESIDE A FRAME\'S SPAN TOO, beside its slot',
          fn: async () => {
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [{ label: 'OCCURRENCE 1', quote: 'So this cat is a chubby little treasure', slot: 'chubby', },],
                },
              },),
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: { kind: 'present', text: 'He said ‘So this cat is a chubby little treasure’ again.', },
                },
                PLACES[1] as (typeof PLACES)[number],
              ],
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'So this cat is a chubby little treasure',
              core: 'So this cat is a chubby little treasure',
              start: 9,
              markBefore: '‘',
              markAfter: '’',
              slot: 'chubby',
              slotStart: 17,
            },);
          },
        },),
        it({
          name: 'LOCATES A REPEATED STAND BY THE MARK THE BALLOTS COPIED BESIDE IT when every ballot copied it, the five inputs '
            + 'of the review, a plain group and a frame alike',
          fn: async () => {
            expect(F6_INPUTS.flatMap(function bothBranches(input,): readonly unknown[] {
              return [false, true,].map(function branchOf(framed,): unknown {
                return locatedBy({ input, copying: 3, ballots: 3, framed, },);
              },);
            },),).toEqual(F6_INPUTS.flatMap(function located(input,): readonly unknown[] {
              return [false, true,].map(function branchOf(): unknown {
                return input.located;
              },);
            },),);
          },
        },),
        it({
          name: 'LOCATES A REPEATED STAND WHEN TWO OF THREE BALLOTS COPIED THE MARK BESIDE IT, the one that copied none being '
            + 'no evidence against it',
          fn: async () => {
            expect(F6_INPUTS.flatMap(function bothBranches(input,): readonly unknown[] {
              return [false, true,].map(function branchOf(framed,): unknown {
                return locatedBy({ input, copying: 2, ballots: 3, framed, },);
              },);
            },),).toEqual(F6_INPUTS.flatMap(function located(input,): readonly unknown[] {
              return [false, true,].map(function branchOf(): unknown {
                return input.located;
              },);
            },),);
          },
        },),
        it({
          name: 'LEAVES A REPEATED STAND AMBIGUOUS WHEN ONE OF THREE BALLOTS COPIED THE MARK BESIDE IT, or none did',
          fn: async () => {
            /**
             The rendering a place that stands twice is left with.
             */
            const ambiguous = { kind: 'unlocated', reason: 'ambiguous', };
            expect(F6_INPUTS.flatMap(function bothBranches(input,): readonly unknown[] {
              return [false, true,].flatMap(function branchOf(framed,): readonly unknown[] {
                return [
                  locatedBy({ input, copying: 1, ballots: 3, framed, },),
                  locatedBy({ input, copying: 0, ballots: 3, framed, },),
                ];
              },);
            },),).toEqual(F6_INPUTS.flatMap(function unlocated(): readonly unknown[] {
              return [ambiguous, ambiguous, ambiguous, ambiguous,];
            },),);
          },
        },),
        it({
          name: 'COUNTS THE MARK AMONG THE USABLE BALLOTS AND NOT AMONG THE ONES THAT AGREED ON THE WORDS: three of five agreed '
            + 'and all three copied it, so it holds; three of five agreed and two copied it, so it does not',
          fn: async () => {
            const english: ConsistencyEnglish = { kind: 'present', text: 'Fill the cats\' bowl. Then the cats nap.', };
            /**
             The rendering of the place when the ballots quote it as given, two other ballots quoting something else.

             @param quotes - quotes of the three ballots that agree on the words

             @returns Rendering
             */
            function renderingOf({ quotes, }: { readonly quotes: readonly string[]; },): unknown {
              return countOver({
                voices: ballotsQuoting({ quotes: [...quotes, 'the dogs', 'the dogs',], },),
                places: [{ label: 'OCCURRENCE 1', english, }, PLACES[1] as (typeof PLACES)[number],],
              },).renderings[0];
            }
            expect([
              renderingOf({ quotes: ['the cats\'', 'the cats\'', 'the cats\'',], },),
              renderingOf({ quotes: ['the cats\'', 'the cats\'', 'the cats',], },),
            ],).toEqual([
              { kind: 'located', quote: 'the cats\'', core: 'the cats\'', start: 5, },
              { kind: 'unlocated', reason: 'ambiguous', },
            ],);
          },
        },),
        it({
          name: 'COUNTS EACH SIDE APART, and never takes the mark into the span: two ballots that copied the mark before the '
            + 'words and one that copied the mark after them choose by the mark before',
          fn: async () => {
            const english: ConsistencyEnglish = { kind: 'present', text: 'Fill ‘the cats bowl. Then the cats’ nap.', };
            const count = countOver({
              voices: ballotsQuoting({ quotes: ['‘the cats', '‘the cats', 'the cats’',], },),
              places: [{ label: 'OCCURRENCE 1', english, }, PLACES[1] as (typeof PLACES)[number],],
            },);
            expect(count.renderings[0],).toEqual({ kind: 'located', quote: 'the cats', core: 'the cats', start: 6, markBefore: '‘', },);
          },
        },),
        it({
          name: 'TAKES A PROVEN APOSTROPHE INTO THE SPAN AND RECORDS NO MARK BESIDE IT, whether or not any judge copied it: '
            + 'a plural possessive at the end of the words and a dropped letter at the start of them',
          fn: async () => {
            expect([
              firstRenderingOf({ english: 'Fill the parents’ bowl.', quote: 'the parents', },),
              firstRenderingOf({ english: 'Fill the parents’ bowl.', quote: 'the parents’', },),
              firstRenderingOf({ english: 'Guard her ’round the cats.', quote: 'round the', },),
              firstRenderingOf({ english: 'Guard her ’round the cats.', quote: '’round the', },),
            ],).toEqual([
              { kind: 'located', quote: 'the parents’', core: 'the parents’', start: 5, },
              { kind: 'located', quote: 'the parents’', core: 'the parents’', start: 5, },
              { kind: 'located', quote: '’round the', core: '’round the', start: 10, },
              { kind: 'located', quote: '’round the', core: '’round the', start: 10, },
            ],);
          },
        },),
        it({
          name: 'PROVES AN APOSTROPHE OVER THE PAGE A PLACE BELONGS TO: a closing mark whose opener stands in an earlier slice '
            + 'is kept outside the span and recorded beside it, and the same place on a page with no opener takes it into '
            + 'the span',
          fn: async () => {
            /**
             The English of the place, the second slice of its page.
             */
            const english = 'Then feed the kittens’.';
            expect([
              firstRenderingOf({
                english,
                quote: 'the kittens',
                page: { text: `‘Fill the bowls.\n\n${english}`, offset: 18, },
              },),
              firstRenderingOf({
                english,
                quote: 'the kittens',
                page: { text: `Fill the bowls.\n\n${english}`, offset: 17, },
              },),
              firstRenderingOf({ english, quote: 'the kittens', },),
            ],).toEqual([
              { kind: 'located', quote: 'the kittens', core: 'the kittens', start: 10, markAfter: '’', },
              { kind: 'located', quote: 'the kittens’', core: 'the kittens’', start: 10, },
              { kind: 'located', quote: 'the kittens’', core: 'the kittens’', start: 10, },
            ],);
          },
        },),
        it({
          name: 'KEEPS A MARK WITH A POSSIBLE PARTNER OUTSIDE THE SPAN AND RECORDS IT: the closing mark of a quotation of one '
            + 'word, and a mark beside words that follow an opener in the same paragraph',
          fn: async () => {
            expect([
              firstRenderingOf({ english: '‘Mio’ is here.', quote: 'Mio', },),
              firstRenderingOf({ english: 'Guard her ’round the cats’ bowl.', quote: 'round the', },),
              firstRenderingOf({ english: 'She said ‘look at the parents’ bowl’ twice.', quote: 'the parents', },),
            ],).toEqual([
              { kind: 'located', quote: 'Mio', core: 'Mio', start: 1, markBefore: '‘', markAfter: '’', },
              { kind: 'located', quote: 'round the', core: 'round the', start: 11, markBefore: '’', },
              { kind: 'located', quote: 'the parents', core: 'the parents', start: 18, markAfter: '’', },
            ],);
          },
        },),
        it({
          name: 'LETS THE MARK EVIDENCE READ THE ENGLISH AS BEFORE WHERE A STAND HOLDS A PROVEN APOSTROPHE AND ANOTHER STAND OF '
            + 'THE SAME WORDS HOLDS NONE: no evidence leaves them ambiguous, evidence of a mark after the words chooses the '
            + 'stand with the apostrophe, and the span is widened only once the stand is chosen',
          fn: async () => {
            const english: ConsistencyEnglish = { kind: 'present', text: 'Fill the parents’ bowl. Then the parents nap.', };
            const both: ConsistencyEnglish = { kind: 'present', text: 'Fill the parents’ bowl. Then the parents’ nap.', };
            /**
             The first place's rendering when every ballot quotes it as given, in an English.
             */
            function renderingOf({ text, quote, }: { readonly text: ConsistencyEnglish; readonly quote: string; },): unknown {
              return countOver({
                voices: ballotsQuoting({ quotes: [quote, quote, quote,], },),
                places: [{ label: 'OCCURRENCE 1', english: text, }, PLACES[1] as (typeof PLACES)[number],],
              },).renderings[0];
            }
            expect([
              renderingOf({ text: english, quote: 'the parents', },),
              renderingOf({ text: english, quote: 'the parents’', },),
              renderingOf({ text: both, quote: 'the parents’', },),
            ],).toEqual([
              { kind: 'unlocated', reason: 'ambiguous', },
              { kind: 'located', quote: 'the parents’', core: 'the parents’', start: 5, },
              { kind: 'unlocated', reason: 'ambiguous', },
            ],);
          },
        },),
        it({
          name: 'LEAVES A STAND AMBIGUOUS WHEN THE MARK THE BALLOTS COPIED STANDS BESIDE BOTH, and a stand that stands once '
            + 'is located whatever the ballots copied',
          fn: async () => {
            const both = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'Fill the cats\' bowl. Then the cats\' nap.', }, },
                PLACES[1] as (typeof PLACES)[number],
              ],
            },);
            const once = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'Fill the cats bowl.', }, },
                PLACES[1] as (typeof PLACES)[number],
              ],
            },);
            expect([both.renderings[0], once.renderings[0],],).toEqual([
              { kind: 'unlocated', reason: 'ambiguous', },
              { kind: 'located', quote: 'the cats', core: 'the cats', start: 5, },
            ],);
          },
        },),
        it({
          name: 'LEAVES A PLACE UNLOCATED WHEN ITS WORDS START OR END BESIDE AN APOSTROPHE THAT A DIGIT OR A LETTER JOINS TO '
            + 'A DIGIT, so a replacement cannot strand the mark between them, and locates the same words where a space stands there',
          fn: async () => {
            /**
             The first place's rendering when every ballot quotes the words in the English.

             @param text - English of the place

             @param quote - what every ballot quotes

             @returns Rendering
             */
            function renderingFor({ text, quote, }: { readonly text: string; readonly quote: string; },): unknown {
              return countOver({
                voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote, },], }, },),
                places: [{ label: 'OCCURRENCE 1', english: { kind: 'present', text, }, }, PLACES[1] as (typeof PLACES)[number],],
              },).renderings[0];
            }
            expect([
              renderingFor({ text: 'He said 5\'2 tall.', quote: '2 tall', },),
              renderingFor({ text: 'He said 5’2 tall.', quote: '2 tall', },),
              renderingFor({ text: 'The cat\'5 naps.', quote: '5 naps', },),
              renderingFor({ text: 'In mid\'90s she napped.', quote: '90s', },),
              renderingFor({ text: 'He said 5 2 tall.', quote: '2 tall', },),
              renderingFor({ text: 'The \'90s cats napped.', quote: '90s cats', },),
            ],).toEqual([
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'located', quote: '2 tall', core: '2 tall', start: 10, },
              { kind: 'located', quote: '\'90s cats', core: '\'90s cats', start: 4, },
            ],);
          },
        },),
        it({
          name: 'LEAVES A PLACE UNLOCATED when no text is given by more than half of the usable ballots, a tie included',
          fn: async () => {
            const split = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'a chubby little treasure', },], }, },),
              ],
            },);
            const three = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'So this cat', },], }, },),
                voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'a chubby little treasure', },], }, },),
              ],
            },);
            expect([split.renderings[0], three.renderings[0],],).toEqual([
              { kind: 'unlocated', reason: 'no-agreement', },
              { kind: 'unlocated', reason: 'no-agreement', },
            ],);
          },
        },),
        it({
          name: 'COUNTS A BALLOT THAT GAVE NO QUOTE against the majority, so two of three is a majority and one of three is not',
          fn: async () => {
            const noQuote = { quotes: [] as ConsistencyVoice['quotes'], };
            const twoOfThree = countOver({
              voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', },), voiceOf({ seat: 'c', over: noQuote, },),],
            },);
            const oneOfThree = countOver({
              voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', over: noQuote, },), voiceOf({ seat: 'c', over: noQuote, },),],
            },);
            expect([twoOfThree.renderings[0]?.kind, oneOfThree.renderings[0],],).toEqual([
              'located',
              { kind: 'unlocated', reason: 'no-agreement', },
            ],);
          },
        },),
        it({
          name: 'REFUSES AN EMPTY QUOTE, and a quote that is only closing punctuation, as an empty one',
          fn: async () => {
            const empty = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: '', },], }, },),
            },);
            const stops = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: '...', },], }, },),
            },);
            expect([empty.renderings[0], stops.renderings[0],],).toEqual([
              { kind: 'unlocated', reason: 'empty', },
              { kind: 'unlocated', reason: 'empty', },
            ],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE THE ENGLISH DOES NOT HOLD, as a missing one',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The kitten slept', },], }, },),
            },);
            expect(count.renderings[0],).toEqual({ kind: 'unlocated', reason: 'not-found', },);
          },
        },),
        it({
          name: 'NEVER LOCATES A CORE HOLDING A NOTE BETWEEN TRANSLATORS: a quote copied across the place a note was cut '
            + 'from leaves its place unlocated, and a quote either side of the note locates as it did',
          fn: async () => {
            // The count reads the stored English (`consistency-count.ts`), where the note stands. A core is copied to
            // other places and ends at `start + core.length`, so a core spanning the note would carry the note
            // into them; a place the quote cannot be found in is left as it stands instead.
            /**
             English whose first sentence ends with a note.
             */
            const noted = 'The sunny window put the kitten to sleep.{/* 这句不太对劲 */}\nSo this cat must be a chubby little treasure.';
            /**
             The count of the first place when every ballot quotes it as given.
             */
            const renderingOf = (quote: string,): unknown => countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote, },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: noted, }, },
                PLACES[1] as { readonly label: string; readonly english: ConsistencyEnglish; },
              ],
            },).renderings[0];

            expect([
              renderingOf('put the kitten to sleep.\nSo this cat',),
              renderingOf('The sunny window put the kitten to sleep',),
              renderingOf('So this cat must be a chubby little treasure',),
            ],).toEqual([
              { kind: 'unlocated', reason: 'not-found', },
              {
                kind: 'located',
                quote: 'The sunny window put the kitten to sleep',
                core: 'The sunny window put the kitten to sleep',
                start: 0,
              },
              {
                kind: 'located',
                quote: 'So this cat must be a chubby little treasure',
                core: 'So this cat must be a chubby little treasure',
                start: noted.indexOf('So this cat',),
              },
            ],);
          },
        },),
        it({
          name: 'REFUSES A QUOTE THAT STANDS MORE THAN ONCE in the English, overlapping stands included, as an ambiguous one',
          fn: async () => {
            const twice = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'this cat', },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'this cat and this cat', }, },
                PLACES[1] as { readonly label: string; readonly english: ConsistencyEnglish; },
              ],
            },);
            const overlapping = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: '猫猫', },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: '猫猫猫', }, },
                PLACES[1] as { readonly label: string; readonly english: ConsistencyEnglish; },
              ],
            },);
            expect([twice.renderings[0], overlapping.renderings[0],],).toEqual([
              { kind: 'unlocated', reason: 'ambiguous', },
              { kind: 'unlocated', reason: 'ambiguous', },
            ],);
          },
        },),
        it({
          name: 'LEAVES A PLACE WITHOUT ENGLISH UNLOCATED, whatever the ballots quote for it',
          fn: async () => {
            const count = countOver({
              voices: threeAlike(),
              places: [
                PLACES[0] as { readonly label: string; readonly english: ConsistencyEnglish; },
                { label: 'OCCURRENCE 2', english: { kind: 'absent', }, },
              ],
            },);
            expect(count.renderings[1],).toEqual({ kind: 'unlocated', reason: 'no-english', },);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S SLOT inside the located quote, and the slot offset is inside the quote\'s words',
          fn: async () => {
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'So this cat must be a chubby little treasure.', slot: 'chubby', },
                    { label: 'OCCURRENCE 2', quote: 'the sunlight on the sill lulled the kitten', slot: 'sill', },
                  ],
                },
              },),
              places: PLACES,
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'So this cat must be a chubby little treasure',
              core: 'So this cat must be a chubby little treasure',
              start: 42,
              slot: 'chubby',
              slotStart: 22,
            },);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S PLACE WITHOUT A SLOT when its slot has no majority, is empty, is not in the span or is in it '
            + 'twice, a slot being no condition of a span',
          fn: async () => {
            /**
             Counts a frame whose every ballot gives the first place this slot.
             */
            function slotted({ slot, }: { readonly slot?: string; },): ReturnType<typeof countConsistency> {
              return countConsistency({
                voices: threeAlike({
                  over: {
                    quotes: [{
                      label: 'OCCURRENCE 1',
                      quote: 'So this cat must be a chubby little treasure',
                      ...((slot === undefined) ? {} : { slot, }),
                    },],
                  },
                },),
                places: PLACES,
                framed: true,
                quorum: 2,
              },);
            }
            /**
             The located rendering of the first place, without a slot.
             */
            const bare = {
              kind: 'located',
              quote: 'So this cat must be a chubby little treasure',
              core: 'So this cat must be a chubby little treasure',
              start: 42,
            };
            expect([
              slotted({}).renderings[0],
              slotted({ slot: '', }).renderings[0],
              slotted({ slot: 'happy', }).renderings[0],
              slotted({ slot: 'e', }).renderings[0],
            ],).toEqual([bare, bare, bare, bare,],);
          },
        },),
        it({
          name: 'IGNORES SLOTS for a repeated run, whatever the ballots give',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({
                over: { quotes: [{ label: 'OCCURRENCE 1', quote: QUOTE_ONE, slot: 'window', },], },
              },),
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: QUOTE_ONE,
              core: QUOTE_ONE,
              start: 0,
            },);
          },
        },),
        it({
          name: 'LOCATES A QUOTE AT THE START AND AT THE END OF ITS ENGLISH, and one holding a character beyond the first '
            + 'plane, offsets counted in the units the English is cut in',
          fn: async () => {
            const cat = String.fromCodePoint(0x1_F6_3A,);
            const places = [
              { label: 'OCCURRENCE 1', english: { kind: 'present', text: `The cat naps ${cat} all day`, }, },
              { label: 'OCCURRENCE 2', english: { kind: 'present', text: `${cat} Then the cat dozed`, }, },
            ] as const;
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: `The cat naps ${cat} all day`, },
                    { label: 'OCCURRENCE 2', quote: `${cat} Then the cat dozed`, },
                  ],
                },
              },),
              places,
              framed: false,
              quorum: 2,
            },);
            expect(count.renderings,).toEqual([
              {
                kind: 'located',
                quote: `The cat naps ${cat} all day`,
                core: `The cat naps ${cat} all day`,
                start: 0,
              },
              {
                kind: 'located',
                quote: `${cat} Then the cat dozed`,
                core: `${cat} Then the cat dozed`,
                start: 0,
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES A LEADING SPACE OF A QUOTE OUT OF THE SPAN, the offset naming the first visible character',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: ' sunny window put the kitten to sleep', },
                    { label: 'OCCURRENCE 2', quote: QUOTE_TWO, },
                  ],
                },
              },),
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'sunny window put the kitten to sleep',
              core: 'sunny window put the kitten to sleep',
              start: 4,
            },);
          },
        },),
        it({
          name: 'COUNTS A SLOT ONLY AMONG THE BALLOTS THAT GAVE THE AGREED SPAN, so a slot a majority gave beside other '
            + 'quotes is no slot of this one, and the place is located without one',
          fn: async () => {
            /**
             Ballot quoting the first place with this quote and slot.
             */
            function quoting({ seat, quote, slot, }: { readonly seat: string; readonly quote: string; readonly slot: string; },): ConsistencyVoice {
              return voiceOf({ seat, over: { quotes: [{ label: 'OCCURRENCE 1', quote, slot, },], }, },);
            }
            const count = countConsistency({
              voices: [
                quoting({ seat: 'a', quote: 'So this cat must be a chubby little treasure', slot: 'chubby', },),
                quoting({ seat: 'b', quote: 'So this cat must be a chubby little treasure', slot: 'little', },),
                quoting({ seat: 'c', quote: 'So this cat must be a chubby', slot: 'chubby', },),
              ],
              places: PLACES,
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'So this cat must be a chubby little treasure',
              core: 'So this cat must be a chubby little treasure',
              start: 42,
            },);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S PLACE WITHOUT A SLOT WHEN ITS SLOT IS THE WHOLE OF ITS SPAN, since no word of the frame '
            + 'is left for the pattern to carry',
          fn: async () => {
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [{ label: 'OCCURRENCE 1', quote: 'chubby little treasure.', slot: 'chubby little treasure', },],
                },
              },),
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: { kind: 'present', text: 'So this cat is a chubby little treasure.', },
                },
                PLACES[1] as (typeof PLACES)[number],
              ],
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'chubby little treasure',
              core: 'chubby little treasure',
              start: 17,
            },);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S PLACE WITHOUT A SLOT WHEN ITS SLOT CUTS MARKUP INSIDE A SPAN THAT HOLDS ITS MARKUP IN PAIRS',
          fn: async () => {
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [{
                    label: 'OCCURRENCE 1',
                    quote: 'So this cat is a **chubby** little treasure',
                    slot: 'chubby** little',
                  },],
                },
              },),
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: { kind: 'present', text: 'So this cat is a **chubby** little treasure.', },
                },
                PLACES[1] as (typeof PLACES)[number],
              ],
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'So this cat is a **chubby** little treasure',
              core: 'So this cat is a **chubby** little treasure',
              start: 0,
            },);
          },
        },),
        it({
          name: 'LOCATES A FRAME\'S PLACE WITHOUT A SLOT WHEN THE SLOT IS ONLY A PIECE OF A WORD of its span, a slot being '
            + 'located at word boundaries like a span',
          fn: async () => {
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [{
                    label: 'OCCURRENCE 1',
                    quote: 'So the kitten is a smart girl',
                    slot: 'art',
                  },],
                },
              },),
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: { kind: 'present', text: 'So the kitten is a smart girl.', },
                },
                PLACES[1] as (typeof PLACES)[number],
              ],
              framed: true,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'So the kitten is a smart girl',
              core: 'So the kitten is a smart girl',
              start: 0,
            },);
          },
        },),
        it({
          name: 'LEAVES A QUOTE UNLOCATED WHEN IT IS ONLY A PIECE OF A WORD, so a judge who quoted `so` does not point at the '
            + 'middle of `also`',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'so', },], }, },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: 'The kitten also napped.', }, },
                PLACES[1] as (typeof PLACES)[number],
              ],
            },);
            expect(count.renderings[0],).toEqual({ kind: 'unlocated', reason: 'not-found', },);
          },
        },),
        it({
          name: 'READS THE BALLOTS AND THE PLACES OF A GROUP A BOUNDED NUMBER OF TIMES FOR EACH PLACE, however many places the '
            + 'group holds: the quotes and the kept places of three ballots and the places themselves are read a few times for '
            + 'each place and never once for each pair of places',
          fn: async () => {
            /**
             The places of the group, each in a slice of its own.
             */
            const size = 600;
            /**
             The count of reads of every list the count takes, shared.
             */
            const reads = { count: 0, };
            /**
             The labels of the places.
             */
            const labels = Array.from(
              { length: size, },
              function labelOf(_, position,): string {
                return `OCCURRENCE ${String(position + 1,)}`;
              },
            );
            /**
             Three ballots alike, each quoting every place and keeping every place.
             */
            const voices = ['a', 'b', 'c',].map(function voiceBy(seat,): ConsistencyVoice {
              return voiceOf({
                seat,
                over: {
                  quotes: countedList({
                    items: labels.map(function quoteOf(label,): ConsistencyVoice['quotes'][number] {
                      return { label, quote: 'the kitten bowl', };
                    },),
                    reads,
                  },),
                  keep: countedList({ items: labels, reads, },),
                },
              },);
            },);
            const count = countConsistency({
              voices,
              places: countedList({
                items: labels.map(function placeOf(
                  label,
                  position,
                ): {
                  readonly label: string;
                  readonly english: ConsistencyEnglish;
                  readonly sliceIndex: number;
                } {
                  return {
                    label,
                    english: { kind: 'present', text: `Fill the kitten bowl ${String(position,)}.`, },
                    sliceIndex: position,
                  };
                },),
                reads,
              },),
              framed: false,
              quorum: 2,
            },);
            expect([
              count.renderings.filter(function isLocated({ kind, },): boolean {
                return kind === 'located';
              },).length,
              count.kept.every(Boolean,),
              reads.count < (100 * size),
            ],).toEqual([size, true, true,],);
          },
        },),
        it({
          name: 'LEAVES UNLOCATED EXACTLY THE PLACES WHOSE WORDS OVERLAP ANOTHER PLACE OF THEIR SLICE, over every three ranges of '
            + 'words of one English, in any order, whatever the ranges share',
          fn: async () => {
            /**
             The words of the English, each standing once.
             */
            const words = ['one', 'two', 'three', 'four', 'five', 'six',];
            /**
             The English of every place.
             */
            const english: ConsistencyEnglish = { kind: 'present', text: words.join(' ',), };
            /**
             Every range of whole words, as its first and last word.
             */
            const ranges = words.flatMap(function rangesFrom(
              _first,
              first,
            ): readonly (readonly [number, number])[] {
              return words.slice(first,)
                .map(function rangeTo(
                  _last,
                  length,
                ): readonly [number, number] {
                  return [first, first + length,];
                },);
            },);
            /**
             What each triple of ranges comes to: the places the count leaves unlocated against the places that share a word.
             */
            const outcomes = ranges.flatMap(function triplesFrom(first,): readonly (readonly [readonly boolean[], readonly boolean[]])[] {
              return ranges.flatMap(function triplesWith(second,): readonly (readonly [readonly boolean[], readonly boolean[]])[] {
                return ranges.map(function triple(third,): readonly [readonly boolean[], readonly boolean[]] {
                  /**
                   The three ranges, one for each place.
                   */
                  const chosen = [first, second, third,];
                  const count = countConsistency({
                    voices: threeAlike({
                      over: {
                        quotes: chosen.map(function quoteOf(
                          [from, to,],
                          position,
                        ): ConsistencyVoice['quotes'][number] {
                          return { label: `OCCURRENCE ${String(position + 1,)}`, quote: words.slice(from, to + 1,).join(' ',), };
                        },),
                        choice: 'OCCURRENCE 1',
                      },
                    },),
                    places: chosen.map(function placeOf(_range, position,): {
                      readonly label: string;
                      readonly english: ConsistencyEnglish;
                      readonly sliceIndex: number;
                    } {
                      return { label: `OCCURRENCE ${String(position + 1,)}`, english, sliceIndex: 0, };
                    },),
                    framed: false,
                    quorum: 2,
                  },);
                  return [
                    count.renderings.map(function isUnlocated({ kind, },): boolean {
                      return kind === 'unlocated';
                    },),
                    chosen.map(function sharesAWord(
                      [from, to,],
                      position,
                    ): boolean {
                      return chosen.some(function overlapsIt(
                        [otherFrom, otherTo,],
                        other,
                      ): boolean {
                        return (other !== position) && (from <= otherTo) && (otherFrom <= to);
                      },);
                    },),
                  ];
                },);
              },);
            },);
            expect(outcomes.every(function agrees([left, right,],): boolean {
              return left.every(function sameAt(unlocated, position,): boolean {
                return unlocated === right[position];
              },);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'LOCATES TWO PLACES OF ONE SLICE WHOSE WORDS TOUCH, since words that share no character overlap nothing',
          fn: async () => {
            const sleepy = '小猫睡觉';
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: '小猫', },
                    { label: 'OCCURRENCE 2', quote: '睡觉', },
                  ],
                },
              },),
              places: [
                { label: 'OCCURRENCE 1', english: { kind: 'present', text: sleepy, }, sliceIndex: 0, },
                { label: 'OCCURRENCE 2', english: { kind: 'present', text: sleepy, }, sliceIndex: 0, },
              ],
              framed: false,
              quorum: 2,
            },);
            expect(count.renderings,).toEqual([
              { kind: 'located', quote: '小猫', core: '小猫', start: 0, },
              { kind: 'located', quote: '睡觉', core: '睡觉', start: 2, },
            ],);
          },
        },),
        it({
          name: 'LEAVES TWO PLACES OF ONE SLICE UNLOCATED WHEN THEIR WORDS OVERLAP, one run of English being no two renderings',
          fn: async () => {
            const english: ConsistencyEnglish = { kind: 'present', text: 'The cat naps loudly today.', };
            const count = countConsistency({
              voices: threeAlike({
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'cat naps loudly', },
                    { label: 'OCCURRENCE 2', quote: 'naps loudly today', },
                  ],
                },
              },),
              places: [
                { label: 'OCCURRENCE 1', english, sliceIndex: 0, },
                { label: 'OCCURRENCE 2', english, sliceIndex: 0, },
              ],
              framed: false,
              quorum: 2,
            },);
            expect(count.renderings,).toEqual([
              { kind: 'unlocated', reason: 'overlaps-another', },
              { kind: 'unlocated', reason: 'overlaps-another', },
            ],);
          },
        },),
        it({
          name: 'LOCATES TWO PLACES WHOSE WORDS OVERLAP WHEN THEY STAND IN DIFFERENT SLICES or in no slice the caller names, '
            + 'since each is cut from an English of its own',
          fn: async () => {
            const english: ConsistencyEnglish = { kind: 'present', text: 'The cat naps loudly today.', };
            const quotes = [
              { label: 'OCCURRENCE 1', quote: 'cat naps loudly', },
              { label: 'OCCURRENCE 2', quote: 'naps loudly today', },
            ];
            const apart = countConsistency({
              voices: threeAlike({ over: { quotes, }, },),
              places: [
                { label: 'OCCURRENCE 1', english, sliceIndex: 0, },
                { label: 'OCCURRENCE 2', english, sliceIndex: 1, },
              ],
              framed: false,
              quorum: 2,
            },);
            const unnamed = countConsistency({
              voices: threeAlike({ over: { quotes, }, },),
              places: [
                { label: 'OCCURRENCE 1', english, },
                { label: 'OCCURRENCE 2', english, },
              ],
              framed: false,
              quorum: 2,
            },);
            const located = [
              { kind: 'located', quote: 'cat naps loudly', core: 'cat naps loudly', start: 4, },
              { kind: 'located', quote: 'naps loudly today', core: 'naps loudly today', start: 8, },
            ];
            expect([apart.renderings, unnamed.renderings,],).toEqual([located, located,],);
          },
        },),
        it({
          name: 'REFUSES A CHOSEN LABEL THE SHEET NEVER SHOWED, since a usable ballot cannot name one and a winner at '
            + 'index minus one would read as a place',
          fn: async () => {
            const refusal = caught(function winnerOfUnknown(): unknown {
              return winnerOf({
                usable: 3,
                quorum: 2,
                chosen: ['OCCURRENCE 9',],
                labels: ['OCCURRENCE 1', 'OCCURRENCE 2',],
                renderings: [],
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the label OCCURRENCE 9 a majority chose is no label of the sheet, '
                + 'and a usable ballot chooses a label the sheet showed',
            );
          },
        },),
        it({
          name: 'NAMES THE WINNER on a strict majority of the usable ballots that reaches the quorum, whose own quote is located',
          fn: async () => {
            const count = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', },),
                voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 2', }, },),
              ],
            },);
            expect({ usable: count.usable, winner: count.winner, },).toEqual({
              usable: 3,
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
            },);
          },
        },),
        it({
          name: 'KEEPS THE GROUP AS IT STANDS on a tie between two places, as the consolidation gate keeps the standing',
          fn: async () => {
            const count = countOver({
              voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 2', }, },),],
            },);
            expect(count.winner,).toEqual({ kind: 'none', reason: 'no-majority', },);
          },
        },),
        it({
          name: 'KEEPS THE GROUP AS IT STANDS when the choices have no majority, a plurality being none',
          fn: async () => {
            const count = countConsistency({
              voices: [
                voiceOf({ seat: 'a', over: { choice: 'OCCURRENCE 1', }, },),
                voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 2', }, },),
                voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 3', }, },),
                voiceOf({ seat: 'd', over: { choice: 'OCCURRENCE 3', }, },),
                voiceOf({ seat: 'e', over: { choice: 'OCCURRENCE 1', }, },),
              ],
              places: [
                ...PLACES,
                { label: 'OCCURRENCE 3', english: FIRST, },
              ],
              framed: false,
              quorum: 3,
            },);
            expect(count.winner,).toEqual({ kind: 'none', reason: 'no-majority', },);
          },
        },),
        it({
          name: 'NAMES NO WINNER BELOW THE QUORUM the gather closed on, though every usable ballot agrees',
          fn: async () => {
            const count = countOver({ voices: [voiceOf({ seat: 'a', },),], quorum: 2, },);
            expect({ usable: count.usable, winner: count.winner, },).toEqual({
              usable: 1,
              winner: { kind: 'none', reason: 'no-quorum', },
            },);
          },
        },),
        it({
          name: 'NAMES NO WINNER ON ONE USABLE BALLOT though the gather closed on a quorum of one (a bench of one or two '
            + 'seats), since one judge alone never decides (the lead\'s ruling of 2026-10-25, on the owner\'s rule), and '
            + 'runs nothing off',
          fn: async () => {
            const count = countOver({ voices: [voiceOf({ seat: 'a', },),], quorum: 1, },);
            expect({ usable: count.usable, winner: count.winner, finalists: count.finalists, },).toEqual({
              usable: 1,
              winner: { kind: 'none', reason: 'no-quorum', },
              finalists: [],
            },);
          },
        },),
        it({
          name: 'STILL NAMES A WINNER ON TWO USABLE BALLOTS under a quorum of one, the positive control: two judges '
            + 'decide',
          fn: async () => {
            const count = countOver({ voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', },),], quorum: 1, },);
            expect({ usable: count.usable, winner: count.winner, },).toEqual({
              usable: 2,
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
            },);
          },
        },),
        it({
          name: 'NAMES NO WINNER when nobody was heard',
          fn: async () => {
            expect(countOver({ voices: [], },).winner,).toEqual({ kind: 'none', reason: 'no-quorum', },);
          },
        },),
        it({
          name: 'NAMES NO WINNER when the place a majority chose has no located quote',
          fn: async () => {
            const count = countOver({
              voices: threeAlike({ over: { quotes: [{ label: 'OCCURRENCE 2', quote: QUOTE_TWO, },], }, },),
            },);
            expect(count.winner,).toEqual({ kind: 'none', reason: 'chosen-unlocated', },);
          },
        },),
        it({
          name: 'LEAVES UNUSABLE BALLOTS OUT of the usable count, the majority and the quotes: a blank reason, an '
            + 'unknown choice and an unknown kept place',
          fn: async () => {
            const count = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', over: { reason: ' ', choice: 'OCCURRENCE 2', }, },),
                voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 9', }, },),
                voiceOf({ seat: 'd', over: { keep: ['OCCURRENCE 9',], }, },),
                voiceOf({ seat: 'e', },),
              ],
              quorum: 1,
            },);
            expect({ usable: count.usable, winner: count.winner, },).toEqual({
              usable: 2,
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
            },);
          },
        },),
        it({
          name: 'KEEPS A PLACE only when more than half of the usable ballots list it',
          fn: async () => {
            const kept = countOver({
              voices: [
                voiceOf({ seat: 'a', over: { keep: ['OCCURRENCE 2',], }, },),
                voiceOf({ seat: 'b', over: { keep: ['OCCURRENCE 2',], }, },),
                voiceOf({ seat: 'c', },),
              ],
            },);
            const tied = countOver({
              voices: [
                voiceOf({ seat: 'a', over: { keep: ['OCCURRENCE 2',], }, },),
                voiceOf({ seat: 'b', },),
              ],
            },);
            expect([kept.kept, tied.kept,],).toEqual([[false, true,], [false, false,],],);
          },
        },),
        it({
          name: 'AGREES BY LOCATED SPAN, not by the string a judge typed: a line break written as a space, a curly apostrophe '
            + 'written straight and a closing full stop kept or left off are one span',
          fn: async () => {
            const count = countConsistency({
              voices: [
                voiceOf({
                  seat: 'a',
                  over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The kitten wasn’t asleep,\nwas it', },], },
                },),
                voiceOf({
                  seat: 'b',
                  over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The kitten wasn\'t asleep, was it?', },], },
                },),
                voiceOf({
                  seat: 'c',
                  over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The kitten  wasn\'t asleep,  was it', },], },
                },),
              ],
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: { kind: 'present', text: 'It purred.\nThe kitten wasn\'t asleep,\nwas it?\nIt blinked.', },
                },
                PLACES[1] as (typeof PLACES)[number],
              ],
              framed: false,
              quorum: 2,
            },);
            expect(count.renderings[0],).toEqual({
              kind: 'located',
              quote: 'The kitten wasn\'t asleep,\nwas it',
              core: 'The kitten wasn\'t asleep,\nwas it',
              start: 11,
            },);
          },
        },),
        it({
          name: 'COUNTS A BALLOT WHOSE QUOTE STANDS NOWHERE OR TWICE among the usable ballots that did not agree, so two spans '
            + 'out of four do not make a strict majority',
          fn: async () => {
            const count = countOver({
              voices: [
                voiceOf({ seat: 'a', },),
                voiceOf({ seat: 'b', },),
                voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The kitten slept', },], }, },),
                voiceOf({ seat: 'd', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'purred loudly', },], }, },),
              ],
              quorum: 3,
            },);
            expect(count.renderings[0],).toEqual({ kind: 'unlocated', reason: 'no-agreement', },);
          },
        },),
        it({
          name: 'NAMES THE PLACES THAT TIED FOR FIRST as the finalists, and a lone leader with the places next behind it (four '
            + 'ballots splitting two, one and one name three places), when the usable ballots reach the quorum and no choice '
            + 'has a strict majority, and no finalists otherwise',
          fn: async () => {
            const tiedTwo = countOver({
              voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 2', }, },),],
            },);
            const tiedThree = countConsistency({
              voices: [
                voiceOf({ seat: 'a', over: { choice: 'OCCURRENCE 1', }, },),
                voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 2', }, },),
                voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 3', }, },),
                voiceOf({ seat: 'd', over: { choice: 'OCCURRENCE 3', }, },),
                voiceOf({ seat: 'e', over: { choice: 'OCCURRENCE 1', }, },),
              ],
              places: [
                ...PLACES,
                { label: 'OCCURRENCE 3', english: FIRST, },
              ],
              framed: false,
              quorum: 3,
            },);
            const majority = countOver({
              voices: [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', },), voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 2', }, },),],
            },);
            const belowQuorum = countOver({ voices: [voiceOf({ seat: 'a', },),], quorum: 2, },);
            const plurality = countConsistency({
              voices: [
                voiceOf({ seat: 'a', over: { choice: 'OCCURRENCE 1', }, },),
                voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 1', }, },),
                voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 2', }, },),
                voiceOf({ seat: 'd', over: { choice: 'OCCURRENCE 3', }, },),
              ],
              places: [
                ...PLACES,
                { label: 'OCCURRENCE 3', english: FIRST, },
              ],
              framed: false,
              quorum: 3,
            },);
            const nobody = countOver({ voices: [], },);
            expect([
              tiedTwo.finalists,
              tiedThree.finalists,
              majority.finalists,
              belowQuorum.finalists,
              plurality.finalists,
              nobody.finalists,
            ],).toEqual([
              ['OCCURRENCE 1', 'OCCURRENCE 2',],
              ['OCCURRENCE 1', 'OCCURRENCE 3',],
              [],
              [],
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              [],
            ],);
          },
        },),
        it({
          name: 'REPLAYS THE LIVE GROUP TWO BALLOTS (the maximal repeat): the four quotes, two with their closing full stop, '
            + 'agree on the English\'s own span, and three choices of four name the first place the winner; a recurrence of '
            + 'the straight-against-curly and trimmed-text families',
          fn: async () => {
            const wanted = 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people';
            const accompany = 'She wanted to accompany more people with her cuteness and bring sunshine to their lives';
            /**
             One live ballot: the quotes given and the place chosen.
             */
            function liveBallot(
              { seat, stop, choice, }: { readonly seat: string; readonly stop: string; readonly choice: string; },
            ): ConsistencyVoice {
              return voiceOf({
                seat,
                over: {
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: `${wanted}${stop}`, },
                    { label: 'OCCURRENCE 2', quote: `${accompany}${stop}`, },
                  ],
                  choice,
                },
              },);
            }
            const count = countConsistency({
              voices: [
                liveBallot({ seat: 'qwen', stop: '', choice: 'OCCURRENCE 1', },),
                liveBallot({ seat: 'kimi', stop: '', choice: 'OCCURRENCE 1', },),
                liveBallot({ seat: 'deepseek', stop: '.', choice: 'OCCURRENCE 1', },),
                liveBallot({ seat: 'mercury', stop: '.', choice: 'OCCURRENCE 2', },),
              ],
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: {
                    kind: 'present',
                    text: 'She was not the stereotypical timid child;\nshe was a “senior” everyone could truly rely on,\n'
                      + 'and,\neven more,\na “teacher” worthy of trust.\n'
                      + `${wanted}.`,
                  },
                },
                {
                  label: 'OCCURRENCE 2',
                  english: {
                    kind: 'present',
                    text: `with every blade of grass and every shrub.\n\n${accompany}.`,
                  },
                },
              ],
              framed: false,
              quorum: 3,
            },);
            expect({
              usable: count.usable,
              winner: count.winner,
              finalists: count.finalists,
              renderings: count.renderings,
            },).toEqual({
              usable: 4,
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
              finalists: [],
              renderings: [
                { kind: 'located', quote: wanted, core: wanted, start: 137, },
                { kind: 'located', quote: accompany, core: accompany, start: 44, },
              ],
            },);
          },
        },),
        it({
          name: 'REPLAYS THE LIVE GROUP ONE BALLOTS (the refrain): the quotes written with a line break, a space, a straight or a '
            + 'curly apostrophe and a closing mark agree on the English\'s own span, the slot spans are agreed by words, and '
            + 'the two choices that split the ballots evenly are the finalists; a recurrence of the straight-against-curly and '
            + 'trimmed-text families',
          fn: async () => {
            /**
             One live ballot: its four quotes with slots, and the place chosen.
             */
            function liveBallot(
              {
                seat,
                quotes,
                choice,
              }: {
                readonly seat: string;
                readonly quotes: readonly (readonly [string, string,])[];
                readonly choice: string;
              },
            ): ConsistencyVoice {
              return {
                seat,
                reason: 'The refrain is rendered four ways.',
                quotes: quotes.map(function quoteOf([quote, slot,], at,): ConsistencyVoice['quotes'][number] {
                  return { label: `OCCURRENCE ${String(at + 1,)}`, quote, slot, };
                },),
                choice,
                keep: [],
              };
            }
            const voices = [
              liveBallot({
                seat: 'qwen',
                quotes: [
                  ['So she must have been a girl who loved life', 'loved life',],
                  ['So she was a girl of extraordinary perseverance,\nI suppose', 'extraordinary perseverance',],
                  ['So she was a girl who could be trusted,\nwasn\'t she', 'could be trusted',],
                  ['So she was a gentle,\ntender-hearted girl,\nwasn\'t she', 'gentle,\ntender-hearted',],
                ],
                choice: 'OCCURRENCE 3',
              },),
              liveBallot({
                seat: 'kimi',
                quotes: [
                  ['So she must have been a girl who loved life', 'who loved life',],
                  ['So she was a girl of extraordinary perseverance,\nI suppose', 'of extraordinary perseverance',],
                  ['So she was a girl who could be trusted,\nwasn\'t she', 'who could be trusted',],
                  ['So she was a gentle,\ntender-hearted girl,\nwasn\'t she', 'gentle,\ntender-hearted',],
                ],
                choice: 'OCCURRENCE 3',
              },),
              liveBallot({
                seat: 'minimax',
                quotes: [
                  ['So she must have been a girl who loved life.', 'loved life',],
                  ['So she was a girl of extraordinary perseverance, I suppose.', 'of extraordinary perseverance',],
                  ['So she was a girl who could be trusted, wasn\'t she?', 'who could be trusted',],
                  ['So she was a gentle, tender-hearted girl, wasn\'t she?', 'gentle, tender-hearted',],
                ],
                choice: 'OCCURRENCE 4',
              },),
              liveBallot({
                seat: 'mercury',
                quotes: [
                  ['So she must have been a girl who loved life', 'loved life',],
                  ['So she was a girl of extraordinary perseverance,\nI suppose', 'extraordinary perseverance',],
                  ['So she was a girl who could be trusted,\nwasn’t she', 'could be trusted',],
                  ['So she was a gentle,\ntender-hearted girl,\nwasn’t she', 'gentle, tender-hearted',],
                ],
                choice: 'OCCURRENCE 4',
              },),
            ];
            const count = countConsistency({
              voices,
              places: [
                {
                  label: 'OCCURRENCE 1',
                  english: {
                    kind: 'present',
                    text: '## Wish\n\nShe wanted to share her story as her true self with more people,\n'
                      + 'and to bring joy to more people.\n\nSo she must have been a girl who loved life.\n'
                      + 'Under the lights,\nin the plainest of lives,\nshe held the most distant of dreams close to her heart.',
                  },
                },
                {
                  label: 'OCCURRENCE 2',
                  english: {
                    kind: 'present',
                    text: 'So she was a girl of extraordinary perseverance,\nI suppose.\nYou see,\namid the starry night,\n'
                      + 'in the frailest of bodies,\nshe bore the most unyielding pressure.',
                  },
                },
                {
                  label: 'OCCURRENCE 3',
                  english: {
                    kind: 'present',
                    text: 'She wanted,\nwith her warm and friendly manner,\nto look after more people and bring a faint '
                      + 'glimmer of light into more lives.\n\nSo she was a girl who could be trusted,\nwasn\'t she?\nYou see,\n'
                      + 'far across the mountains and seas,\nshe gave the clearest directions in the most complicated of situations.',
                  },
                },
                {
                  label: 'OCCURRENCE 4',
                  english: {
                    kind: 'present',
                    text: 'So she was a gentle,\ntender-hearted girl,\nwasn\'t she?\nJust look:\ninside the headpiece,\n'
                      + 'in the narrowest of spaces,\nshe held up the most perfect world.',
                  },
                },
              ],
              framed: true,
              quorum: 3,
            },);
            expect({
              usable: count.usable,
              winner: count.winner,
              finalists: count.finalists,
              renderings: count.renderings,
            },).toEqual({
              usable: 4,
              winner: { kind: 'none', reason: 'no-majority', },
              finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',],
              renderings: [
                {
                  kind: 'located',
                  quote: 'So she must have been a girl who loved life',
                  core: 'So she must have been a girl who loved life',
                  start: 108,
                  slot: 'loved life',
                  slotStart: 33,
                },
                {
                  kind: 'located',
                  quote: 'So she was a girl of extraordinary perseverance,\nI suppose',
                  core: 'So she was a girl of extraordinary perseverance,\nI suppose',
                  start: 0,
                },
                {
                  kind: 'located',
                  quote: 'So she was a girl who could be trusted,\nwasn\'t she',
                  core: 'So she was a girl who could be trusted,\nwasn\'t she',
                  start: 126,
                },
                {
                  kind: 'located',
                  quote: 'So she was a gentle,\ntender-hearted girl,\nwasn\'t she',
                  core: 'So she was a gentle,\ntender-hearted girl,\nwasn\'t she',
                  start: 0,
                  slot: 'gentle,\ntender-hearted',
                  slotStart: 13,
                },
              ],
            },);
          },
        },),
      ],
    },),

    describe({
      name: winnerOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DECIDES FROM THE FACTS ALONE: the quorum first, then the majority, then the chosen place\'s quote',
          fn: async () => {
            const labels = ['OCCURRENCE 1', 'OCCURRENCE 2',];
            const located = { kind: 'located', quote: 'x', core: 'x', start: 0, } as const;
            const lost = { kind: 'unlocated', reason: 'not-found', } as const;
            expect([
              winnerOf({ usable: 1, quorum: 2, chosen: ['OCCURRENCE 1',], labels, renderings: [located, located,], },),
              winnerOf({ usable: 2, quorum: 2, chosen: [], labels, renderings: [located, located,], },),
              winnerOf({ usable: 2, quorum: 2, chosen: ['OCCURRENCE 2',], labels, renderings: [located, lost,], },),
              winnerOf({ usable: 2, quorum: 2, chosen: ['OCCURRENCE 2',], labels, renderings: [lost, located,], },),
            ],).toEqual([
              { kind: 'none', reason: 'no-quorum', },
              { kind: 'none', reason: 'no-majority', },
              { kind: 'none', reason: 'chosen-unlocated', },
              { kind: 'chosen', label: 'OCCURRENCE 2', },
            ],);
          },
        },),
      ],
    },),

  ],
},);
