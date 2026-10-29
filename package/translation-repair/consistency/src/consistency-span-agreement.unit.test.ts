/**
 Tests for how a chosen place is located when the judges quoted different spans of it (the lead's ruling of 2026-10-15 on
 the run of TianqiChen66625, group 2, where three judges quoted three nested spans of one place and none was a strict
 majority of the text): the place is located by the longest of its quotes that stands exactly once in its English and that,
 after the fold, the quotes of more than half of all the usable ballots contain.

 Fixtures are cat-themed invention, except the three quotes and the two English sentences of group 2, which are the
 run's own words (English only, as the artifact and the page hold them).

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { once, } from 'node:events';
import { Worker, } from 'node:worker_threads';

import {
  type ConsistencyVoice,
  containedQuoteWordsOf,
  countConsistency,
} from '../dist/final/node/index.mjs';
import { environmentWithoutKeys, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 The built module's address, for a worker that imports it.
 */
const BUILT_INDEX = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).href;

/**
 The worker's script: it reads the candidates for the quotes handed to it and posts them back. A worker, since a call that never
 returns holds the thread it runs on and no timer in that thread can end it.
 */
const CANDIDATES_SCRIPT = `
const { parentPort, workerData, } = require('node:worker_threads');
async function post() {
  const pkg = await import(workerData.index);
  const voices = workerData.quotes.map(function ballotOf(quote, index) {
    return { seat: 'seat ' + index, reason: 'The first place words it best.', quotes: [{ label: 'OCCURRENCE 1', quote, },],
      choice: 'OCCURRENCE 1', keep: [], };
  });
  parentPort.postMessage(pkg.containedQuoteWordsOf({ voices, label: 'OCCURRENCE 1', }));
}
post();
`;

/**
 The candidate spans of a place each ballot quoted once, read in a worker that is ended at the hang stop.

 @param quotes - one quote of the first place for each ballot

 @returns Candidates, longest first, or the word `hung` when the worker had not answered by the hang stop

 @example
 ```ts
 const candidates = await candidatesInWorker({ quotes: ['the cat', 'the cat',], },);
 ```
 */
async function candidatesInWorker({ quotes, }: { readonly quotes: readonly string[]; },): Promise<unknown> {
  /**
   The keyless environment, its removed variables left out, since a worker takes every value it is handed as text.
   */
  const env = Object.fromEntries(Object.entries(environmentWithoutKeys(),).filter(function isSet([, value,],): boolean {
    return value !== undefined;
  },),);
  /**
   The worker reading the candidates.
   */
  const worker = new Worker(CANDIDATES_SCRIPT, {
    eval: true,
    workerData: {
      index: BUILT_INDEX,
      quotes,
    },
    env,
  },);
  // Ending the worker ends a call that never returns, on every way out of this function.
  await using _ended = {
    [Symbol.asyncDispose]: async function endWorker(): Promise<void> {
      await worker.terminate();
    },
  };
  /**
   The stop for a hang, which no case waits for.
   */
  const stop = AbortSignal.timeout(HANG_STOP_MS,);
  try {
    /**
     The arguments of the worker's one message, which an error event the worker raises rejects.
     */
    const message: readonly unknown[] = await once(
      worker,
      'message',
      { signal: stop, },
    );
    return message[0];
  }
  catch (error) {
    if (stop.aborted)
      return 'hung';
    throw error;
  }
}

/**
 Builds one ballot that chooses the first place and quotes the given spans.

 @param seat - seat that answered
 @param quotes - quote given for each place, by label
 @param choice - label of the place chosen

 @returns Ballot

 @example
 ```ts
 const voice = ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },);
 ```
 */
function ballotOf(
  {
    seat,
    quotes,
    choice,
  }: {
    readonly seat: string;
    readonly quotes: Readonly<Record<string, string>>;
    readonly choice: string;
  },
): ConsistencyVoice {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes: Object.entries(quotes,)
      .map(function quoteOf([label, quote,],) {
        return { label, quote, };
      },),
    choice,
    keep: [],
  };
}

/**
 English of the two places of group 2 of the run of TianqiChen66625, as the page holds them.
 */
const GROUP_TWO_ENGLISH = [
  'She also wanted to stay beside more people in her adorable form,\nbringing sunshine into more lives.',
  'She wanted to stay by more people’s sides in her cute form and bring sunshine to more of them.',
] as const;

/**
 The three ballots of group 2, rebuilt from the artifact (`artifacts/TianqiChen666.json`, group 2, `verdict.ballots`).
 */
const GROUP_TWO_BALLOTS: readonly ConsistencyVoice[] = [
  ballotOf({
    seat: 'hf:moonshotai/Kimi-K3',
    quotes: {
      'OCCURRENCE 1': 'She wanted to stay beside more people in her adorable form,\nbringing sunshine into more lives',
      'OCCURRENCE 2': 'She wanted to stay by more people\'s sides in her cute form and bring sunshine to more of them',
    },
    choice: 'OCCURRENCE 1',
  },),
  {
    ...ballotOf({
      seat: 'minimax-m3',
      quotes: {
        'OCCURRENCE 1': 'stay beside more people in her adorable form, bringing sunshine into more lives',
        'OCCURRENCE 2': 'stay by more people\'s sides in her cute form and bring sunshine to more of them',
      },
      choice: 'OCCURRENCE 1',
    },),
    keep: ['OCCURRENCE 2',],
  },
  {
    ...ballotOf({
      seat: 'inception/mercury-2.5',
      quotes: {
        'OCCURRENCE 1': 'wanted to stay beside more people in her adorable form, bringing sunshine into more lives.',
        'OCCURRENCE 2': 'wanted to stay by more people\'s sides in her cute form and bring sunshine to more of them.',
      },
      choice: 'OCCURRENCE 2',
    },),
    keep: ['OCCURRENCE 1',],
  },
];

/**
 Counts ballots against two places of the given English.

 @param voices - ballots
 @param english - English of each place, in order
 @param quorum - quorum the gather closed on
 @param framed - whether the group is a frame with a slot, a plain group by default

 @param pageText - text of the page the first place belongs to, none by default, in which the first place starts at 0

 @returns What the ballots come to

 @example
 ```ts
 const count = countAgainst({ voices, english: ['the cat naps', 'the kitten naps',], quorum: 2, },);
 ```
 */
function countAgainst(
  {
    voices,
    english,
    quorum,
    framed = false,
    pageText,
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly english: readonly string[];
    readonly quorum: number;
    readonly framed?: boolean;
    readonly pageText?: string;
  },
): ReturnType<typeof countConsistency> {
  return countConsistency({
    voices,
    places: english.map(function placeOf(text, position,) {
      return {
        label: `OCCURRENCE ${String(position + 1,)}`,
        english: { kind: 'present' as const, text, },
        ...(((position > 0) || (pageText === undefined)) ? {} : { page: { text: pageText, offset: 0, }, }),
      };
    },),
    framed,
    quorum,
  },);
}

/**
 Builds one ballot of a frame that chooses the first place and quotes one span of it with a slot beside it.

 @param seat - seat that answered
 @param quote - span quoted
 @param slot - slot words given beside it

 @returns Ballot

 @example
 ```ts
 const voice = framedBallotOf({ seat: 'a', quote: 'cat naps in the sun', slot: 'naps', },);
 ```
 */
function framedBallotOf(
  {
    seat,
    quote,
    slot,
  }: {
    readonly seat: string;
    readonly quote: string;
    readonly slot: string;
  },
): ConsistencyVoice {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes: [{ label: 'OCCURRENCE 1', quote, slot, },],
    choice: 'OCCURRENCE 1',
    keep: [],
  };
}

await describe({
  name: countConsistency.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'LOCATES GROUP 2 OF THE RUN OF TianqiChen66625 BY THE LONGEST QUOTE A STRICT MAJORITY CONTAIN: the first judge\'s span '
        + 'stands nowhere on the page, the third judge\'s span is inside the first and contains the second, and so the third '
        + 'judge\'s span locates the chosen place and the winner is no longer lost to an unlocated place',
      fn: async () => {
        const count = countAgainst({ voices: GROUP_TWO_BALLOTS, english: GROUP_TWO_ENGLISH, quorum: 3, },);
        expect({
          first: count.renderings[0]?.kind === 'located' ? count.renderings[0].quote : count.renderings[0],
          second: count.renderings[1]?.kind === 'located' ? count.renderings[1].quote : count.renderings[1],
          winner: count.winner,
        },).toEqual({
          first: 'wanted to stay beside more people in her adorable form,\nbringing sunshine into more lives',
          second: 'wanted to stay by more people’s sides in her cute form and bring sunshine to more of them',
          winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
        },);
      },
    },),
    it({
      name: 'CHOOSES THE LONGER OF TWO QUOTES WHEN A STRICT MAJORITY OF THE QUOTES CONTAIN IT, and the shorter when they do not',
      fn: async () => {
        // Two quotes: the shorter is inside both, the longer inside one of two, which is no strict majority.
        const split = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['In the evening the cat naps in the sun.', 'The kitten naps.',],
          quorum: 2,
        },);
        expect(split.renderings[0]?.kind === 'located' ? split.renderings[0].quote : split.renderings[0],).toBe(
          'cat naps in the sun',
        );
        // Three quotes: the longer is inside two of three.
        const nested = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['In the evening the cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
        },);
        expect(nested.renderings[0]?.kind === 'located' ? nested.renderings[0].quote : nested.renderings[0],).toBe(
          'the cat naps in the sun',
        );
      },
    },),
    it({
      name: 'LOCATES NOTHING WHEN NO QUOTE IS CONTAINED IN A STRICT MAJORITY OF THE QUOTES, the place staying as it stands',
      fn: async () => {
        const count = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'every evening', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['Every evening the cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
        },);
        expect(count.renderings[0],).toEqual({ kind: 'unlocated', reason: 'no-agreement', },);
      },
    },),
    it({
      name: 'COUNTS A MAJORITY OF EVERY USABLE BALLOT, not only of the quotes naming the place: two of four ballots quoting a '
        + 'place the same way, the other two quoting nothing of it, do not agree on it',
      fn: async () => {
        const count = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps', 'OCCURRENCE 2': 'the kitten naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps', 'OCCURRENCE 2': 'the kitten naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'd', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['the cat naps in the sun.', 'the kitten naps in the sun.',],
          quorum: 3,
        },);
        expect(count.renderings[1],).toEqual({ kind: 'unlocated', reason: 'no-agreement', },);
      },
    },),
    it({
      name: 'SKIPS A QUOTE THAT STANDS NOWHERE IN THE PLACE FOR A SHORTER QUOTE A STRICT MAJORITY CONTAIN THAT STANDS ONCE, and '
        + 'reports the longest quote\'s own failure when none stands once',
      fn: async () => {
        // "the cat naps in the shade" is inside two of three quotes and stands nowhere; "the cat naps" is inside all three.
        const skipped = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps in the shade', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps in the shade', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['the cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
        },);
        const twice = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['the cat naps. the cat naps again.', 'The kitten naps.',],
          quorum: 2,
        },);
        const nowhere = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['The kitten naps.', 'The kitten naps.',],
          quorum: 2,
        },);
        expect([
          skipped.renderings[0]?.kind === 'located' ? skipped.renderings[0].quote : skipped.renderings[0],
          twice.renderings[0],
          nowhere.renderings[0],
        ],).toEqual([
          'the cat naps',
          { kind: 'unlocated', reason: 'ambiguous', },
          { kind: 'unlocated', reason: 'not-found', },
        ],);
      },
    },),
    it({
      name: 'REPORTS THE FAILURE OF THE LONGEST CANDIDATE, not of a shorter one: the longest quote stands nowhere and the shorter '
        + 'stands twice, so the place is not found, and where the longest stands twice the place is ambiguous',
      fn: async () => {
        // "the cat naps in the shade" is inside two of three quotes; "the cat naps" is inside all three.
        const longestNowhere = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps in the shade', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps in the shade', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['the cat naps in the sun. the cat naps again.', 'The kitten naps.',],
          quorum: 3,
        },);
        const longestTwice = countAgainst({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'the cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'the cat naps in the sun', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'the cat naps', }, choice: 'OCCURRENCE 1', },),
          ],
          english: ['the cat naps in the sun. the cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
        },);
        expect([
          longestNowhere.renderings[0],
          longestTwice.renderings[0],
        ],).toEqual([
          { kind: 'unlocated', reason: 'not-found', },
          { kind: 'unlocated', reason: 'ambiguous', },
        ],);
      },
    },),
    it({
      name: 'READS A COMBINING MARK AS PART OF THE WORD IT FOLLOWS, as the locator does: a quote "cafe" is no word of '
        + '"cafe" written with a combining acute accent, so it is contained only in itself and is no candidate',
      fn: async () => {
        // The fold leaves the decomposed accent as it is, so the accent follows the "e" in the longer quotes.
        const candidates = containedQuoteWordsOf({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'cafe', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'cafe\u{0301} au lait', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'cafe\u{0301} au lait', }, choice: 'OCCURRENCE 1', },),
          ],
          label: 'OCCURRENCE 1',
        },);
        expect(candidates,).toEqual(['cafe\u{0301} au lait',],);
      },
    },),
    it({
      name: 'READS A JOINT OF A LONGER WORD AS PART OF IT, as the locator does: a quote "cat" is no word of "cat\'s bowl", "cat-nap" '
        + 'or "0.5" written with a joint beside it, so it is contained only in itself and is no candidate, and "cat" beside a space '
        + 'is one',
      fn: async () => {
        /**
         The candidates two ballots leave when one quotes the short words and the other quotes the longer text.
         */
        function candidatesOf({ short, long, }: { readonly short: string; readonly long: string; },): unknown {
          return containedQuoteWordsOf({
            voices: [
              ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': short, }, choice: 'OCCURRENCE 1', },),
              ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': long, }, choice: 'OCCURRENCE 1', },),
            ],
            label: 'OCCURRENCE 1',
          },);
        }
        expect([
          candidatesOf({ short: 'cat', long: 'cat\'s bowl', },),
          candidatesOf({ short: 'cat', long: 'cat-nap', },),
          candidatesOf({ short: '5', long: '0.5', },),
          candidatesOf({ short: 'bowl', long: 'cat\'s bowl', },),
          candidatesOf({ short: 'cat', long: 'cat naps', },),
        ],).toEqual([[], [], [], ['bowl',], ['cat',],],);
      },
    },),
    it({
      name: 'READS A JOINT OF A LONGER WORD AS NO PART OF IT when the caller names the earlier reading of an artifact written before '
        + 'joints counted: "cat" is a word of "cat\'s bowl" there, so two ballots that quoted the longer text and one that quoted '
        + '"cat" agree on "cat", which the reading by default leaves out',
      fn: async () => {
        /**
         The ballots: two quoted the longer text, one the short words.
         */
        const voices = [
          ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'cat\'s bowl', }, choice: 'OCCURRENCE 1', },),
          ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'cat\'s bowl', }, choice: 'OCCURRENCE 1', },),
          ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'cat', }, choice: 'OCCURRENCE 1', },),
        ];
        expect({
          byDefault: containedQuoteWordsOf({ voices, label: 'OCCURRENCE 1', },),
          locator: containedQuoteWordsOf({ voices, label: 'OCCURRENCE 1', edges: 'locator', },),
          letters: containedQuoteWordsOf({ voices, label: 'OCCURRENCE 1', edges: 'letters', },),
        },).toEqual({
          byDefault: ['cat\'s bowl',],
          locator: ['cat\'s bowl',],
          letters: ['cat\'s bowl', 'cat',],
        },);
      },
    },),
    it({
      name: 'ENDS ON A BLANK QUOTE, which is contained only in another blank quote: a blank quote beside two that end in a '
        + 'letter is no candidate, and two blank quotes beside one of words are the one candidate',
      fn: async () => {
        // The search for an empty text finds it at every offset and once more at the end, so a loop stepping one offset past
        // each find never ran out; a reply whose quote folds to nothing is one a judge can send.
        expect({
          lone: await candidatesInWorker({ quotes: ['', 'the cat', 'the cat',], },),
          most: await candidatesInWorker({ quotes: [' ', '', 'the cat',], },),
        },).toEqual({
          lone: ['the cat',],
          most: ['',],
        },);
      },
    },),
    it({
      name: 'ORDERS CANDIDATES BY CODE POINTS, NOT UTF-16 UNITS: a quote of four letters is longer than one of a letter and two '
        + 'cats beyond the first plane, though the second takes five units',
      fn: async () => {
        const candidates = containedQuoteWordsOf({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'abcd', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'x🐈🐈', }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'abcd x🐈🐈', }, choice: 'OCCURRENCE 1', },),
          ],
          label: 'OCCURRENCE 1',
        },);
        expect(candidates,).toEqual(['abcd', 'x🐈🐈',],);
      },
    },),
    it({
      name: 'TELLS APART TWO CANDIDATES OF ONE LENGTH BY THEIR CODE POINTS, whichever ballot quoted them first, so the same ballots '
        + 'always give the same order',
      fn: async () => {
        expect({
          beeFirst: containedQuoteWordsOf({
            voices: [
              ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'bee', }, choice: 'OCCURRENCE 1', },),
              ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'ant', }, choice: 'OCCURRENCE 1', },),
              ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'ant bee', }, choice: 'OCCURRENCE 1', },),
            ],
            label: 'OCCURRENCE 1',
          },),
          antFirst: containedQuoteWordsOf({
            voices: [
              ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': 'ant', }, choice: 'OCCURRENCE 1', },),
              ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': 'bee', }, choice: 'OCCURRENCE 1', },),
              ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': 'ant bee', }, choice: 'OCCURRENCE 1', },),
            ],
            label: 'OCCURRENCE 1',
          },),
        },).toEqual({
          beeFirst: ['ant', 'bee',],
          antFirst: ['ant', 'bee',],
        },);
      },
    },),
    it({
      name: 'ORDERS CANDIDATES OF ONE LENGTH BY CODE POINTS, NOT UTF-16 UNITS: a character beyond the first plane follows one of the '
        + 'first plane above the surrogates, though its first code unit is the lesser',
      fn: async () => {
        /**
         A character of the first plane whose single code unit is greater than the first code unit of any character beyond it.
         */
        const plain = 'a\u{E000}';
        /**
         A character beyond the first plane after the same letter.
         */
        const beyond = 'a🐈';
        expect(containedQuoteWordsOf({
          voices: [
            ballotOf({ seat: 'a', quotes: { 'OCCURRENCE 1': beyond, }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'b', quotes: { 'OCCURRENCE 1': plain, }, choice: 'OCCURRENCE 1', },),
            ballotOf({ seat: 'c', quotes: { 'OCCURRENCE 1': `${beyond} ${plain}`, }, choice: 'OCCURRENCE 1', },),
          ],
          label: 'OCCURRENCE 1',
        },),).toEqual([plain, beyond,],);
      },
    },),
    it({
      name: 'COUNTS A SLOT GIVEN BESIDE ANY QUOTE THAT CONTAINS THE LOCATED WORDS, as the place is located by containment: '
        + 'nested quotes Q1 (the big cat naps in the sun), Q2 (cat naps in the sun) and Q3 (cat naps) locate the place by Q2, '
        + 'and the slot the two judges who quoted Q1 and Q2 gave agrees, two of three usable ballots',
      fn: async () => {
        const count = countAgainst({
          voices: [
            framedBallotOf({ seat: 'a', quote: 'the big cat naps in the sun', slot: 'naps', },),
            framedBallotOf({ seat: 'b', quote: 'cat naps in the sun', slot: 'naps', },),
            framedBallotOf({ seat: 'c', quote: 'cat naps', slot: 'cat', },),
          ],
          english: ['Today the big cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
          framed: true,
        },);
        expect(count.renderings[0],).toEqual({
          kind: 'located',
          quote: 'cat naps in the sun',
          core: 'cat naps in the sun',
          start: 14,
          slot: 'naps',
          slotStart: 4,
        },);
      },
    },),
    it({
      name: 'COUNTS NO SLOT GIVEN BESIDE A QUOTE THAT DOES NOT CONTAIN THE LOCATED WORDS: the shorter quote Q3 (cat naps) is '
        + 'not evidence for the slot of Q2 (cat naps in the sun), so one slot beside Q1 and two beside Q2 and Q3 agree on none',
      fn: async () => {
        const count = countAgainst({
          voices: [
            framedBallotOf({ seat: 'a', quote: 'the big cat naps in the sun', slot: 'naps', },),
            framedBallotOf({ seat: 'b', quote: 'cat naps in the sun', slot: 'sun', },),
            framedBallotOf({ seat: 'c', quote: 'cat naps', slot: 'sun', },),
          ],
          english: ['Today the big cat naps in the sun.', 'The kitten naps.',],
          quorum: 3,
          framed: true,
        },);
        expect(count.renderings[0],).toEqual({
          kind: 'located',
          quote: 'cat naps in the sun',
          core: 'cat naps in the sun',
          start: 14,
        },);
      },
    },),
    it({
      name: 'RECORDS THE DEFINITIONS OF THE PAGE ON A LOCATED RENDERING ONLY WHERE THEY MOVE ITS SLOT, and only the ones that do: '
        + 'with the definition of `[b]` on the page `a[b]c` is one word and has no slot, a definition of another reference moves '
        + 'nothing, and a page that holds both records the first alone',
      fn: async () => {
        /**
         The ballots of a frame that all gave the slot `b` beside the span `a[b]c`.
         */
        const voices = ['a', 'b', 'c',].map(function ballotFor(seat,): ConsistencyVoice {
          return framedBallotOf({ seat, quote: 'a[b]c', slot: 'b', },);
        },);
        /**
         The first place's rendering for a page, none for the span read alone.
         */
        function renderingFor({ pageText, }: { readonly pageText?: string; },): unknown {
          return countAgainst({
            voices,
            english: ['a[b]c', 'The kitten naps.',],
            quorum: 3,
            framed: true,
            ...((pageText === undefined) ? {} : { pageText, }),
          },).renderings[0];
        }
        expect([
          renderingFor({},),
          renderingFor({ pageText: 'a[b]c\n\n[b]: https://x.example/b', },),
          renderingFor({ pageText: 'a[b]c\n\n[z]: https://x.example/z', },),
          renderingFor({ pageText: 'a[b]c\n\n[b]: https://x.example/b\n\n[z]: https://x.example/z', },),
        ],).toEqual([
          { kind: 'located', quote: 'a[b]c', core: 'a[b]c', start: 0, slot: 'b', slotStart: 2, },
          { kind: 'located', quote: 'a[b]c', core: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b', },
          { kind: 'located', quote: 'a[b]c', core: 'a[b]c', start: 0, slot: 'b', slotStart: 2, },
          { kind: 'located', quote: 'a[b]c', core: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b', },
        ],);
      },
    },),
  ],
},);
