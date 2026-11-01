import {
  type ConsistencyEnglish,
  type ConsistencyRendering,
  type ConsistencyVoice,
  countConsistency,
} from '../dist/final/node/index.mjs';

//region Consistency renderings
// THE RENDERINGS OF A PAIR OF PLACES AS THE PASS COUNTS THEM, for tests of what code does with a chosen place's words and a
// receiving place's words (the planner, the edge refusal), which read exactly what the pass hands them.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 A located rendering of the kind the planner reads.
 */
export type Located = Extract<ConsistencyRendering, { readonly kind: 'located'; }>;

/**
 One place of a pair: its English and the quote the judges gave for it.
 */
export type QuotedPlace = {
  /**
   What the place says on the page.
   */
  readonly english: string;

  /**
   The words the judges quoted for it.
   */
  readonly quote: string;

  /**
   The English of the whole page the place stands in, with its offset taken as zero, none for a place read alone: a reference in
   the place's English resolves against the definitions the page holds, whichever slice holds them.
   */
  readonly page?: string;
};

/**
 The seats of the three ballots.
 */
const SEATS = [
  'a',
  'b',
  'c',
] as const;

/**
 A place as the pass hands it to the count: the label the ballots quote it by, and the English it holds.
 */
type LabelledPlace = {
  /**
   The label the ballots quote the place by.
   */
  readonly label: string;

  /**
   What the place says on the page.
   */
  readonly english: ConsistencyEnglish;

  /**
   The English of the whole page and where the place's English starts in it, none for a place read alone.
   */
  readonly page?: {
    readonly text: string;
    readonly offset: number;
  };
};

/**
 The located renderings of a chosen place and a receiving place, counted from three ballots that quote both and choose the
 first, so the planner reads exactly what the pass hands it.

 @param chosen - the chosen place's English and quote

 @param receiving - the receiving place's English and quote

 @returns The two renderings, chosen first

 @throws Error when either place is not located

 @example
 ```ts
 const [chosen, target,] = renderingsOf({ chosen, receiving, },);
 ```
 */
export function renderingsOf(
  {
    chosen,
    receiving,
  }: {
    readonly chosen: QuotedPlace;
    readonly receiving: QuotedPlace;
  },
): readonly [
  Located,
  Located,
] {
  /**
   Three ballots that quote both places and choose the first.
   */
  const voices: readonly ConsistencyVoice[] = SEATS.map(function voiceOf(seat,): ConsistencyVoice {
    return {
      seat,
      reason: 'The first rendering is the more faithful.',
      quotes: [
        {
          label: 'OCCURRENCE 1',
          quote: chosen.quote,
        },
        {
          label: 'OCCURRENCE 2',
          quote: receiving.quote,
        },
      ],
      choice: 'OCCURRENCE 1',
      keep: [],
    };
  },);
  /**
   The two places as the pass hands them over.
   */
  const places: readonly LabelledPlace[] = [
    {
      label: 'OCCURRENCE 1',
      english: {
        kind: 'present',
        text: chosen.english,
      },
      ...((chosen.page === undefined)
        ? {}
        : {
          page: {
            text: chosen.page,
            offset: 0,
          },
        }),
    },
    {
      label: 'OCCURRENCE 2',
      english: {
        kind: 'present',
        text: receiving.english,
      },
      ...((receiving.page === undefined)
        ? {}
        : {
          page: {
            text: receiving.page,
            offset: 0,
          },
        }),
    },
  ];
  /**
   What the count made of the two places.
   */
  const { renderings, } = countConsistency({
    voices,
    places,
    framed: false,
    quorum: 2,
  },);
  /**
   The renderings of the chosen place and the receiving place.
   */
  const [first, second,] = renderings;
  if ((first?.kind !== 'located') || (second?.kind !== 'located')) {
    throw new Error(
      `a place is not located: ${JSON.stringify([
        first,
        second,
      ],)}`,
    );
  }
  return [
    first,
    second,
  ];
}

//endregion Consistency renderings
