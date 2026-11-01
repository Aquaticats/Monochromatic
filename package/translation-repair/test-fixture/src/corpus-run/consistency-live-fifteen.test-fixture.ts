/**
 The consistency section of the live artifact of generation fifteen (the entry TianqiChen666), as the file holds it:
 a refrain of four lines whose choices tied two and two and a maximal repeat whose winner's quote the first reader
 could not locate. The quotes and slots are the live judges' own, so they stay in the straight-against-curly and
 trimmed-text shapes that generation fifteen counted as different strings; the judges' reasons are abridged to the
 sentences that decide, since the reader checks a reason only for blankness.

 @module
 */

import { SENTENCE_BALLOTS, } from './consistency-live-fifteen-sentence.test-fixture.ts';

//region Live consistency section of generation fifteen

/**
 The quotes the four live judges gave for the refrain, each with its slot, written as they wrote them.
 */
const REFRAIN_BALLOTS = [
  {
    seat: 'hf:Qwen/Qwen3.8-27B',
    reason: 'The renderings differ in wording and force, not core meaning: 1 uses \'must have been\', 2 uses \'I suppose,\' and 3-4 use a confirming tag. The most faithful is OCCURRENCE 3, because it keeps the memorial past tense, stays third person, and renders 吧 as the natural confirming question \'wasn\'t she?\' in a flexible \'a girl who ...\' refrain. No surrounding speaker, tense, quotation, or wordplay forces any occurrence to keep its current wording.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'So she must have been a girl who loved life',
        slot: 'loved life',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'So she was a girl of extraordinary perseverance,\nI suppose',
        slot: 'extraordinary perseverance',
      },
      {
        label: 'OCCURRENCE 3',
        quote: 'So she was a girl who could be trusted,\nwasn\'t she',
        slot: 'could be trusted',
      },
      {
        label: 'OCCURRENCE 4',
        quote: 'So she was a gentle,\ntender-hearted girl,\nwasn\'t she',
        slot: 'gentle,\ntender-hearted',
      },
    ],
    choice: 'OCCURRENCE 3',
    keep: [],
  },
  {
    seat: 'hf:moonshotai/Kimi-K3',
    reason: 'The four places render the same sentence frame four ways: \'must have been\', \'I suppose\', and twice \'wasn\'t she?\'. OCCURRENCE 3 and OCCURRENCE 4 already agree on \'So she was a … girl, wasn\'t she?\', which keeps the structure and matches clause and tag correctly; OCCURRENCE 3 is the cleaner of the two parallel refrains, so the others should follow it.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'So she must have been a girl who loved life',
        slot: 'who loved life',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'So she was a girl of extraordinary perseverance,\nI suppose',
        slot: 'of extraordinary perseverance',
      },
      {
        label: 'OCCURRENCE 3',
        quote: 'So she was a girl who could be trusted,\nwasn\'t she',
        slot: 'who could be trusted',
      },
      {
        label: 'OCCURRENCE 4',
        quote: 'So she was a gentle,\ntender-hearted girl,\nwasn\'t she',
        slot: 'gentle,\ntender-hearted',
      },
    ],
    choice: 'OCCURRENCE 3',
    keep: [],
  },
  {
    seat: 'minimax-m3',
    reason: 'All four renderings place the slot as a modifier before the noun "girl", but the surrounding frame disagrees in verb tense and modal ending. The original is a soft hedged guess best carried into English by a tag question on a simple past copula. OCCURRENCE 4 keeps the past tense the house rule sets for a memorial life and gives a warm, conversational tone.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'So she must have been a girl who loved life.',
        slot: 'loved life',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'So she was a girl of extraordinary perseverance, I suppose.',
        slot: 'of extraordinary perseverance',
      },
      {
        label: 'OCCURRENCE 3',
        quote: 'So she was a girl who could be trusted, wasn\'t she?',
        slot: 'who could be trusted',
      },
      {
        label: 'OCCURRENCE 4',
        quote: 'So she was a gentle, tender-hearted girl, wasn\'t she?',
        slot: 'gentle, tender-hearted',
      },
    ],
    choice: 'OCCURRENCE 4',
    keep: [],
  },
  {
    seat: 'inception/mercury-2.5',
    reason: 'The renderings differ in how they capture the inferential particle. Occurrence 4 is the most faithful, using the natural phrasing to preserve both past tense and the original\'s gentle conclusion. None of the surroundings force a grammatical difference, so all should align with this choice.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'So she must have been a girl who loved life',
        slot: 'loved life',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'So she was a girl of extraordinary perseverance,\nI suppose',
        slot: 'extraordinary perseverance',
      },
      {
        label: 'OCCURRENCE 3',
        quote: 'So she was a girl who could be trusted,\nwasn’t she',
        slot: 'could be trusted',
      },
      {
        label: 'OCCURRENCE 4',
        quote: 'So she was a gentle,\ntender-hearted girl,\nwasn’t she',
        slot: 'gentle, tender-hearted',
      },
    ],
    choice: 'OCCURRENCE 4',
    keep: [],
  },
];

/**
 One place of the refrain, as the live artifact recorded it: no quote won, so it stays as it stands.

 @param position - position of the place in the group, counted from zero

 @param sliceIndex - slice that holds the place

 @param startOffset - where the place starts in the original

 @param endOffset - where the place ends in the original

 @param line - one-based line of the place

 @param slotStartOffset - where the slot starts in the original

 @param slotEndOffset - where the slot ends in the original

 @param reason - why the first reader could not locate it

 @returns The place as the file holds it

 @example
 ```ts
 const place = refrainPlace({ position: 0, sliceIndex: 1, startOffset: 232, endOffset: 245, line: 17, slotStartOffset: 237, slotEndOffset: 241, reason: 'slot-no-agreement', },);
 ```
 */
function refrainPlace(
  {
    position,
    sliceIndex,
    startOffset,
    endOffset,
    line,
    slotStartOffset,
    slotEndOffset,
    reason,
  }: {
    readonly position: number;
    readonly sliceIndex: number;
    readonly startOffset: number;
    readonly endOffset: number;
    readonly line: number;
    readonly slotStartOffset: number;
    readonly slotEndOffset: number;
    readonly reason: 'slot-no-agreement' | 'no-agreement';
  },
): Record<string, unknown> {
  return {
    label: `OCCURRENCE ${String(position + 1,)}`,
    sliceIndex,
    startOffset,
    endOffset,
    line,
    slotStartOffset,
    slotEndOffset,
    rendering: {
      kind: 'unlocated',
      reason,
    },
    outcome: 'untouched',
  };
}

/**
 The live consistency section of generation fifteen, with no changed slice and no finding.
 */
export const LIVE_CONSISTENCY_FIFTEEN: Readonly<Record<string, unknown>> = {
  groups: [
    {
      repeat: {
        kind: 'refrain',
        prefix: '所以她是个',
        suffix: '的女孩吧',
      },
      occurrences: [
        refrainPlace({
          position: 0,
          sliceIndex: 1,
          startOffset: 232,
          endOffset: 245,
          line: 17,
          slotStartOffset: 237,
          slotEndOffset: 241,
          reason: 'slot-no-agreement',
        },),
        refrainPlace({
          position: 1,
          sliceIndex: 4,
          startOffset: 558,
          endOffset: 571,
          line: 25,
          slotStartOffset: 563,
          slotEndOffset: 567,
          reason: 'slot-no-agreement',
        },),
        refrainPlace({
          position: 2,
          sliceIndex: 7,
          startOffset: 865,
          endOffset: 878,
          line: 33,
          slotStartOffset: 870,
          slotEndOffset: 874,
          reason: 'no-agreement',
        },),
        refrainPlace({
          position: 3,
          sliceIndex: 9,
          startOffset: 1_084,
          endOffset: 1_097,
          line: 37,
          slotStartOffset: 1_089,
          slotEndOffset: 1_093,
          reason: 'no-agreement',
        },),
      ],
      unplaced: [],
      verdict: {
        kind: 'asked',
        ballots: REFRAIN_BALLOTS,
        usable: 4,
        quorum: 3,
        winner: {
          kind: 'none',
          reason: 'no-majority',
        },
      },
    },
    {
      repeat: {
        kind: 'maximal',
        wording: '她想以可爱的样子陪伴在更多人的身边，为更多人带来阳光。',
      },
      occurrences: [
        {
          label: 'OCCURRENCE 1',
          sliceIndex: 8,
          startOffset: 1_055,
          endOffset: 1_082,
          line: 35,
          rendering: {
            kind: 'unlocated',
            reason: 'no-agreement',
          },
          outcome: 'untouched',
        },
        {
          label: 'OCCURRENCE 2',
          sliceIndex: 15,
          startOffset: 1_667,
          endOffset: 1_694,
          line: 55,
          rendering: {
            kind: 'unlocated',
            reason: 'no-agreement',
          },
          outcome: 'untouched',
        },
      ],
      unplaced: [],
      verdict: {
        kind: 'asked',
        ballots: SENTENCE_BALLOTS,
        usable: 4,
        quorum: 3,
        winner: {
          kind: 'none',
          reason: 'chosen-unlocated',
        },
      },
    },
  ],
  slices: [],
  findings: [],
};

//endregion Live consistency section of generation fifteen
