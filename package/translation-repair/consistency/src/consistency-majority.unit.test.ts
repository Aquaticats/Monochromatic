/**
 Tests for what a strict majority of a group's usable ballots agree on: a
 quote, a slot, a choice, a kept place. Strict means more than half of the
 usable ballots, so a tie agrees on nothing and a ballot that gave nothing
 counts against whatever the others gave.

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
  agreedQuoteOf,
  agreedSlotOf,
  agreedSlotWordsOf,
  agreedWordsOf,
  type ConsistencyVoice,
  consistencyLabels,
  isKeptByMajority,
  keepsByLabelOf,
  majorityChoiceOf,
  markEvidenceOf,
  quotesByLabelOf,
  runOffFinalistsOf,
  runOffTiedFinalistsOf,
  usableVoicesOf,
} from '../dist/final/node/index.mjs';
import { countedList, } from '@monochromatic-dev/translation-repair-test-fixture/ts/element-reads.test-fixture.ts';

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
 Builds one heard ballot over two places.

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
    quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat naps', slot: 'naps', },],
    choice: 'OCCURRENCE 1',
    keep: [],
    ...over,
  };
}

/**
 Builds ballots that choose the labels given, one each.

 @param choices - label each seat chose, in seat order

 @returns Ballots with their seats

 @example
 ```ts
 const voices = choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], },);
 ```
 */
function choosing({ choices, }: { readonly choices: readonly string[]; },): readonly ConsistencyVoice[] {
  return choices.map(function oneSeat(
    choice,
    at,
  ): ConsistencyVoice {
    return voiceOf({ seat: `seat-${String(at,)}`, over: { choice, }, },);
  },);
}

/**
 The mark evidence of ballots that each quote the first place by one of the given quotes.

 @param quotes - what each ballot quotes for the first place, one ballot for each

 @param words - folded words the quotes agreed on

 @returns On which sides the evidence holds

 @example
 ```ts
 const { after, } = evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats',], words: 'the cats', },); // true
 ```
 */
function evidenceFrom(
  {
    quotes,
    words,
  }: {
    readonly quotes: readonly string[];
    readonly words: string;
  },
): ReturnType<typeof markEvidenceOf> {
  return markEvidenceOf({
    voices: quotes.map(function ballotFor(quote, at,): ConsistencyVoice {
      return voiceOf({
        seat: `seat-${String(at,)}`,
        over: { quotes: [{ label: 'OCCURRENCE 1', quote, },], },
      },);
    },),
    label: 'OCCURRENCE 1',
    words,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: agreedQuoteOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE QUOTE MORE THAN HALF OF THE USABLE BALLOTS GAVE, and none when no text has more than half',
          fn: async () => {
            const same = [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', },), voiceOf({ seat: 'c', over: { quotes: [], }, },),];
            const split = [
              voiceOf({ seat: 'a', },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat dozes', },], }, },),
            ];
            expect([
              agreedQuoteOf({ voices: same, label: 'OCCURRENCE 1', },),
              agreedQuoteOf({ voices: split, label: 'OCCURRENCE 1', },),
              agreedQuoteOf({ voices: same, label: 'OCCURRENCE 2', },),
              agreedQuoteOf({ voices: [], label: 'OCCURRENCE 1', },),
            ],).toEqual([['the cat naps',], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: agreedSlotOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE SLOT MORE THAN HALF OF THE USABLE BALLOTS GAVE, a ballot that quoted no slot counting against it',
          fn: async () => {
            const slotted = [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', },), voiceOf({ seat: 'c', },),];
            const mostlyBare = [
              voiceOf({ seat: 'a', },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat naps', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat naps', },], }, },),
            ];
            expect([
              agreedSlotOf({ voices: slotted, label: 'OCCURRENCE 1', },),
              agreedSlotOf({ voices: mostlyBare, label: 'OCCURRENCE 1', },),
            ],).toEqual([['naps',], [],],);
          },
        },),
      ],
    },),
    describe({
      name: agreedWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE FOLDED WORDS MORE THAN HALF OF THE USABLE BALLOTS QUOTED, whatever their spacing, apostrophes and '
            + 'closing marks, and none when the words split',
          fn: async () => {
            const alike = [
              voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat wasn’t   asleep.', },], }, },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat wasn\'t\nasleep', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat dozes', },], }, },),
            ];
            const split = [
              voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps', },], }, },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat dozes', },], }, },),
            ];
            expect([
              agreedWordsOf({ voices: alike, label: 'OCCURRENCE 1', },),
              agreedWordsOf({ voices: split, label: 'OCCURRENCE 1', },),
              agreedWordsOf({ voices: alike, label: 'OCCURRENCE 2', },),
              agreedWordsOf({ voices: [], label: 'OCCURRENCE 1', },),
            ],).toEqual([['The cat wasn\'t asleep',], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: markEvidenceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS FOR A SIDE WHEN MORE THAN HALF OF THE USABLE BALLOTS QUOTED THE AGREED WORDS WITH A SINGLE MARK '
            + 'DIRECTLY BESIDE THEM ON IT, a ballot that copied no mark counting against it',
          fn: async () => {
            const words = 'the cats';
            expect([
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats\'',], words, },),
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats',], words, },),
              evidenceFrom({ quotes: ['the cats\'', 'the cats', 'the cats',], words, },),
              evidenceFrom({ quotes: ['the cats', 'the cats', 'the cats',], words, },),
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats', 'the cats',], words, },),
              evidenceFrom({ quotes: [], words, },),
            ],).toEqual([
              { before: false, after: true, },
              { before: false, after: true, },
              { before: false, after: false, },
              { before: false, after: false, },
              { before: false, after: false, },
              { before: false, after: false, },
            ],);
            expect([
              evidenceFrom({ quotes: ['\'Tis', '\'Tis', 'Tis',], words: 'Tis', },),
              evidenceFrom({ quotes: ['‘the cats’', '‘the cats', 'the cats’',], words, },),
              evidenceFrom({ quotes: ['‘the cats', '‘the cats', 'the cats’',], words, },),
            ],).toEqual([
              { before: true, after: false, },
              { before: true, after: true, },
              { before: true, after: false, },
            ],);
          },
        },),
        it({
          name: 'COUNTS ONLY THE BALLOTS WHOSE QUOTE FOLDS TO THE AGREED WORDS, against all the usable ballots: a ballot that '
            + 'gave other words or no quote adds no evidence and is not nothing',
          fn: async () => {
            const words = 'the cats';
            expect([
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats', 'the dogs\'', 'the dogs\'',], words, },),
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the dogs',], words, },),
              evidenceFrom({ quotes: ['the cats\'', 'the cats\'', 'the cats\'', 'the cats', 'the dogs',], words, },),
              markEvidenceOf({
                voices: [
                  voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', },], }, },),
                  voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', },], }, },),
                  voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 2', quote: 'the cats\'', },], }, },),
                ],
                label: 'OCCURRENCE 1',
                words,
              },),
            ],).toEqual([
              { before: false, after: false, },
              { before: false, after: true, },
              { before: false, after: true, },
              { before: false, after: true, },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: agreedSlotWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE FOLDED SLOT WORDS MORE THAN HALF OF THE USABLE BALLOTS GAVE beside a quote that contains the agreed words, '
            + 'a slot given beside a quote that does not contain them or not given counting against it',
          fn: async () => {
            const voices = [
              voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps', slot: 'cat', },], }, },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', slot: 'cat', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat', slot: 'cat', },], }, },),
            ];
            expect([
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The cat naps', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The cat', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The kitten', },),
            ],).toEqual([['cat',], ['cat',], [],],);
          },
        },),
        it({
          name: 'COUNTS ONLY A QUOTE THAT IS THE AGREED WORDS UNDER THE EXACT-QUOTE AGREEMENT of an artifact written before slots '
            + 'were counted beside containing quotes, so the slot beside the longer quotes is not carried to the shorter words',
          fn: async () => {
            const voices = [
              voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps', slot: 'cat', },], }, },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', slot: 'cat', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat', slot: 'cat', },], }, },),
            ];
            expect([
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The cat naps', slotAgreement: 'exact-quote', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The cat', slotAgreement: 'exact-quote', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'The cat', slotAgreement: 'containing-quote', },),
            ],).toEqual([['cat',], [], ['cat',],],);
          },
        },),
        it({
          name: 'COUNTS A SLOT GIVEN BESIDE A QUOTE THAT HOLDS THE AGREED WORDS BEFORE A JOINT only under the earlier reading of '
            + 'containment of an artifact written before joints counted: "cat" is no word of "cat\'s bowl" by default or by the locator\'s '
            + 'edges, and is one by the edges of letters',
          fn: async () => {
            const voices = [
              voiceOf({ seat: 'a', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'cat\'s bowl', slot: 'bowl', },], }, },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'cat\'s bowl', slot: 'bowl', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'cat', },], }, },),
            ];
            expect([
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'cat', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'cat', edges: 'locator', },),
              agreedSlotWordsOf({ voices, label: 'OCCURRENCE 1', words: 'cat', edges: 'letters', },),
            ],).toEqual([[], [], ['bowl',],],);
          },
        },),
      ],
    },),
    describe({
      name: runOffFinalistsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE LABELS SHARING THE HIGHEST CHOICE COUNT when two or more share it and none has more than half, in '
            + 'the order of the sheet, as a tie for first always has',
          fn: async () => {
            const labels = consistencyLabels({ count: 4, },);
            expect([
              runOffFinalistsOf({ voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2',], },), labels, },),
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 3', 'OCCURRENCE 1', 'OCCURRENCE 3', 'OCCURRENCE 1', 'OCCURRENCE 2',], },),
                labels,
              },),
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',], },),
                labels,
              },),
            ],).toEqual([
              ['OCCURRENCE 1', 'OCCURRENCE 2',],
              ['OCCURRENCE 1', 'OCCURRENCE 3',],
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',],
            ],);
          },
        },),
        it({
          name: 'ADDS THE LABELS WITH THE SECOND-HIGHEST COUNT TO A LONE LEADER that fell short of a majority: four ballots '
            + 'splitting two, one and one name the three labels, in the order of the sheet',
          fn: async () => {
            const labels = consistencyLabels({ count: 4, },);
            expect([
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 1',], },),
                labels,
              },),
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 3', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',], },),
                labels,
              },),
            ],).toEqual([
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
            ],);
          },
        },),
        it({
          name: 'ADDS EVERY LABEL THAT SHARES THE SECOND-HIGHEST COUNT, and no label behind them: five ballots splitting two, '
            + 'one, one and one name all four labels, seven splitting three, two, two and none name the leader and both seconds, '
            + 'and a label behind the seconds stays out',
          fn: async () => {
            const labels = consistencyLabels({ count: 4, },);
            expect([
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',], },),
                labels,
              },),
              runOffFinalistsOf({
                voices: choosing({
                  choices: ['OCCURRENCE 2', 'OCCURRENCE 2', 'OCCURRENCE 2', 'OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 3', 'OCCURRENCE 3',],
                },),
                labels,
              },),
              runOffFinalistsOf({
                voices: choosing({
                  choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',],
                },),
                labels,
              },),
            ],).toEqual([
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',],
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              ['OCCURRENCE 1', 'OCCURRENCE 2',],
            ],);
          },
        },),
        it({
          name: 'NAMES NONE WHEN A LABEL HAS A STRICT MAJORITY, such as two of three, WHEN NOTHING COUNTED, or WHEN NO OTHER '
            + 'SHEET LABEL DREW A BALLOT behind a lone leader',
          fn: async () => {
            const labels = consistencyLabels({ count: 3, },);
            expect([
              runOffFinalistsOf({ voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2',], },), labels, },),
              runOffFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 1',], },),
                labels,
              },),
              runOffFinalistsOf({ voices: choosing({ choices: ['OCCURRENCE 2',], },), labels, },),
              runOffFinalistsOf({ voices: [], labels, },),
              runOffFinalistsOf({ voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 9', 'OCCURRENCE 9',], },), labels, },),
            ],).toEqual([[], [], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: runOffTiedFinalistsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES ONLY THE LABELS TIED FOR THE HIGHEST COUNT, as the artifacts written before the lone leader\'s rule were run off: '
            + 'a lone leader without a majority names none, and a tie names the tied labels in the order of the sheet',
          fn: async () => {
            const labels = consistencyLabels({ count: 4, },);
            expect([
              runOffTiedFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 1',], },),
                labels,
              },),
              runOffTiedFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 3', 'OCCURRENCE 1', 'OCCURRENCE 3', 'OCCURRENCE 1', 'OCCURRENCE 2',], },),
                labels,
              },),
              runOffTiedFinalistsOf({
                voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',], },),
                labels,
              },),
              runOffTiedFinalistsOf({ voices: choosing({ choices: ['OCCURRENCE 1', 'OCCURRENCE 1', 'OCCURRENCE 2',], },), labels, },),
              runOffTiedFinalistsOf({ voices: [], labels, },),
            ],).toEqual([
              [],
              ['OCCURRENCE 1', 'OCCURRENCE 3',],
              ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3', 'OCCURRENCE 4',],
              [],
              [],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: majorityChoiceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE PLACE MORE THAN HALF OF THE BALLOTS CHOSE, and none for a tie or a plurality',
          fn: async () => {
            const two = [
              voiceOf({ seat: 'a', },),
              voiceOf({ seat: 'b', },),
              voiceOf({ seat: 'c', over: { choice: 'OCCURRENCE 2', }, },),
            ];
            const tied = [voiceOf({ seat: 'a', },), voiceOf({ seat: 'b', over: { choice: 'OCCURRENCE 2', }, },),];
            expect([majorityChoiceOf({ voices: two, },), majorityChoiceOf({ voices: tied, },), majorityChoiceOf({ voices: [], },),],)
              .toEqual([['OCCURRENCE 1',], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: isKeptByMajority.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS A PLACE ONLY WHEN MORE THAN HALF OF THE BALLOTS LIST IT, a tie keeping nothing',
          fn: async () => {
            const keeping = (count: number,): readonly ConsistencyVoice[] => ['a', 'b', 'c',].map(function oneSeat(
              seat,
              at,
            ): ConsistencyVoice {
              return voiceOf({ seat, over: { keep: (at < count) ? ['OCCURRENCE 2',] : [], }, },);
            },);
            expect([
              isKeptByMajority({ voices: keeping(3,), label: 'OCCURRENCE 2', },),
              isKeptByMajority({ voices: keeping(2,), label: 'OCCURRENCE 2', },),
              isKeptByMajority({ voices: keeping(1,), label: 'OCCURRENCE 2', },),
              isKeptByMajority({ voices: keeping(0,), label: 'OCCURRENCE 2', },),
              isKeptByMajority({ voices: [], label: 'OCCURRENCE 2', },),
            ],).toEqual([true, true, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: 'the strict-majority boundary',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'AGREES ONLY WHEN MORE THAN HALF OF THE USABLE BALLOTS GIVE THE SAME CHOICE, QUOTE, SLOT OR KEPT PLACE, '
            + 'at every count from one to six, odd and even',
          fn: async () => {
            /**
             Ballots of which `agreeing` give the text everyone is asked about and the rest give another.
             */
            function splitting(
              { usable, agreeing, }: { readonly usable: number; readonly agreeing: number; },
            ): readonly ConsistencyVoice[] {
              return Array.from(
                { length: usable, },
                function oneSeat(
                  _unused,
                  at,
                ): ConsistencyVoice {
                  const agrees = at < agreeing;
                  return voiceOf({
                    seat: `seat-${String(at,)}`,
                    over: {
                      choice: agrees ? 'OCCURRENCE 1' : 'OCCURRENCE 2',
                      quotes: [{ label: 'OCCURRENCE 1', quote: agrees ? 'the cat naps' : 'the cat dozes', slot: agrees ? 'naps' : 'dozes', },],
                      keep: agrees ? ['OCCURRENCE 2',] : [],
                    },
                  },);
                },
              );
            }
            const found: Record<string, readonly boolean[]> = {};
            for (const usable of [1, 2, 3, 4, 5, 6,]) {
              found[String(usable,)] = Array.from(
                { length: usable + 1, },
                function atAgreeing(
                  _unused,
                  agreeing,
                ): boolean {
                  const voices = splitting({ usable, agreeing, },);
                  return (majorityChoiceOf({ voices, },)[0] === 'OCCURRENCE 1')
                    && (agreedQuoteOf({ voices, label: 'OCCURRENCE 1', },)[0] === 'the cat naps')
                    && (agreedSlotOf({ voices, label: 'OCCURRENCE 1', },)[0] === 'naps')
                    && isKeptByMajority({ voices, label: 'OCCURRENCE 2', },);
                },
              );
            }
            expect(found,).toEqual({
              1: [false, true,],
              2: [false, false, true,],
              3: [false, false, true, true,],
              4: [false, false, false, true, true,],
              5: [false, false, false, true, true, true,],
              6: [false, false, false, false, true, true, true,],
            },);
          },
        },),
      ],
    },),
    describe({
      name: quotesByLabelOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GROUPS THE QUOTES OF THE BALLOTS BY THE PLACE THEY NAME, each group in the order of the ballots, and has none '
            + 'for a place nobody quoted',
          fn: async () => {
            const voices = [
              voiceOf({
                seat: 'a',
                over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat naps', }, { label: 'OCCURRENCE 2', quote: 'a cat', },], },
              },),
              voiceOf({ seat: 'b', over: { quotes: [{ label: 'OCCURRENCE 2', quote: 'the cat', slot: 'cat', },], }, },),
              voiceOf({ seat: 'c', over: { quotes: [], }, },),
            ];
            const grouped = quotesByLabelOf({ voices, },);
            expect([
              grouped.get('OCCURRENCE 1',),
              grouped.get('OCCURRENCE 2',),
              grouped.get('OCCURRENCE 3',),
            ],).toEqual([
              [{ label: 'OCCURRENCE 1', quote: 'the cat naps', },],
              [{ label: 'OCCURRENCE 2', quote: 'a cat', }, { label: 'OCCURRENCE 2', quote: 'the cat', slot: 'cat', },],
              undefined,
            ],);
          },
        },),
        it({
          name: 'GIVES THE SAME ANSWER FOR EVERY PLACE WITH THE GROUPED QUOTES AS WITHOUT THEM, for the agreed quote, the agreed '
            + 'words, the agreed slot, the slot beside words and the evidence of a mark',
          fn: async () => {
            const voices = [
              voiceOf({
                seat: 'a',
                over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', slot: 'cats', }, { label: 'OCCURRENCE 2', quote: 'a cat', },], },
              },),
              voiceOf({
                seat: 'b',
                over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cats\'', slot: 'cats', }, { label: 'OCCURRENCE 2', quote: 'a dog', },], },
              },),
              voiceOf({ seat: 'c', over: { quotes: [{ label: 'OCCURRENCE 2', quote: 'a cat', },], }, },),
            ];
            const quotesByLabel = quotesByLabelOf({ voices, },);
            const labels = ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',];
            expect(labels.map(function indexed(label,): unknown {
              return [
                agreedQuoteOf({ voices, label, quotesByLabel, },),
                agreedWordsOf({ voices, label, quotesByLabel, },),
                agreedSlotOf({ voices, label, quotesByLabel, },),
                agreedSlotWordsOf({ voices, label, words: 'the cats', quotesByLabel, },),
                markEvidenceOf({ voices, label, words: 'the cats', quotesByLabel, },),
              ];
            },),).toEqual(labels.map(function plain(label,): unknown {
              return [
                agreedQuoteOf({ voices, label, },),
                agreedWordsOf({ voices, label, },),
                agreedSlotOf({ voices, label, },),
                agreedSlotWordsOf({ voices, label, words: 'the cats', },),
                markEvidenceOf({ voices, label, words: 'the cats', },),
              ];
            },),);
          },
        },),
      ],
    },),
    describe({
      name: keepsByLabelOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE BALLOTS THAT KEEP EACH PLACE, a place listed twice by one ballot counting once, and gives the same '
            + 'keeping as the ballots read one by one',
          fn: async () => {
            const voices = [
              voiceOf({ seat: 'a', over: { keep: ['OCCURRENCE 2', 'OCCURRENCE 2',], }, },),
              voiceOf({ seat: 'b', over: { keep: ['OCCURRENCE 2', 'OCCURRENCE 1',], }, },),
              voiceOf({ seat: 'c', over: { keep: [], }, },),
            ];
            const keepsByLabel = keepsByLabelOf({ voices, },);
            expect([
              keepsByLabel.get('OCCURRENCE 1',),
              keepsByLabel.get('OCCURRENCE 2',),
              keepsByLabel.get('OCCURRENCE 3',),
              isKeptByMajority({ voices, label: 'OCCURRENCE 1', keepsByLabel, },),
              isKeptByMajority({ voices, label: 'OCCURRENCE 2', keepsByLabel, },),
              isKeptByMajority({ voices, label: 'OCCURRENCE 2', },),
            ],).toEqual([1, 2, undefined, false, true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: usableVoicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SELECTS THE BALLOTS THAT COUNT against the places shown, leaving out a blank reason and a place never shown',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            const voices = [
              voiceOf({ seat: 'a', },),
              voiceOf({ seat: 'b', over: { reason: '', }, },),
              voiceOf({ seat: 'c', over: { keep: ['OCCURRENCE 9',], }, },),
              voiceOf({ seat: 'd', over: { keep: ['OCCURRENCE 2',], }, },),
            ];
            expect(usableVoicesOf({ voices, labels, },).map(function seatOf(voice,): string {
              return voice.seat;
            },),).toEqual(['a', 'd',],);
          },
        },),
        it({
          name: 'LEAVES OUT A BALLOT THAT QUOTES OR KEEPS A PLACE TWICE, and counts one that names each place shown once',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            const voices = [
              voiceOf({ seat: 'a', },),
              voiceOf({
                seat: 'b',
                over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'the cat naps', }, { label: 'OCCURRENCE 1', quote: 'a cat', },], },
              },),
              voiceOf({ seat: 'c', over: { keep: ['OCCURRENCE 2', 'OCCURRENCE 2',], }, },),
              voiceOf({ seat: 'd', over: { keep: ['OCCURRENCE 1', 'OCCURRENCE 2',], }, },),
            ];
            expect(usableVoicesOf({ voices, labels, },).map(function seatOf(voice,): string {
              return voice.seat;
            },),).toEqual(['a', 'd',],);
          },
        },),
        it({
          name: 'READS THE LABELS SHOWN A BOUNDED NUMBER OF TIMES FOR EACH LABEL a ballot names, however many places the sheet '
            + 'showed: three ballots that quote and keep every place of a long sheet read it a few times for each place and never '
            + 'once for each pair of places',
          fn: async () => {
            /**
             The places the sheet shows.
             */
            const size = 600;
            /**
             The count of reads of the labels shown.
             */
            const reads = { count: 0, };
            /**
             The labels of the places, as plain text.
             */
            const names = consistencyLabels({ count: size, },);
            /**
             Three ballots alike, each quoting every place and keeping every place.
             */
            const voices = ['a', 'b', 'c',].map(function voiceBy(seat,): ConsistencyVoice {
              return voiceOf({
                seat,
                over: {
                  quotes: names.map(function quoteOf(label,): ConsistencyVoice['quotes'][number] {
                    return { label, quote: 'the cat naps', };
                  },),
                  keep: names,
                },
              },);
            },);
            expect([
              usableVoicesOf({ voices, labels: countedList({ items: names, reads, },), },).length,
              reads.count < (30 * size),
            ],).toEqual([3, true,],);
          },
        },),
      ],
    },),
  ],
},);
