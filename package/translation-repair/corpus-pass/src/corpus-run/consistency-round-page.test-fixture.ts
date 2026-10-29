/**
 The round page for the tests of what the floors' refusal of a slice does to the check of the apostrophes the replacements
 carry in: the first group's splice puts `’round` in the second slice, the second group's splice takes the words of the third
 slice out for `the folks`, and whether `’round` stays an apostrophe depends on whether a closing mark stands after it, and so
 on whether the third slice is changed.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  applyConsistency,
  consistencyFloorVerdict,
  type ConsistencySliceFacts,
  type SettledGroup,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';

/**
 The English of the first two slices of the round page: the chosen words, which hold a leading `’round`, and the receiving
 words, which hold `round` without it.
 */
const ROUND_ENGLISH = [
  'The folks ran ’round.',
  'We went round the yard.',
] as const;

/**
 What the receiving place of the round page's second group holds: the slice it stands in and the span its splice replaces.
 */
type RoundThird = {
  /**
   The English of the third slice.
   */
  readonly english: string;

  /**
   The quote the judges give for its place.
   */
  readonly quote: string;

  /**
   Where the code splice over its place ends: the words start at the sixth character.
   */
  readonly end: number;
};

/**
 The third slice of the page whose closer comes back when a floor refuses it: a plural possessive that the splice takes out
 with its words.
 */
export const THIRD_WITH_CLOSER: RoundThird = {
  english: 'Fill the parents’ bowl now.',
  quote: 'the parents',
  end: 17,
};

/**
 The third slice of the page whose refusal restores no mark: its words hold none.
 */
export const THIRD_WITHOUT_MARK: RoundThird = {
  english: 'Fill the family bowl now.',
  quote: 'the family',
  end: 15,
};

/**
 What the pass reports when it leaves out the splice that puts `’round` in the second slice.
 */
export const ROUND_LEFT_OUT: string = 'slice 1: the consistency replacement of group 1 OCCURRENCE 2 was left out because the apostrophe it '
  + 'carries in would read as a quotation mark there, since another quotation mark of the page could pair with it; the place '
  + 'keeps its words';

/**
 What one slice of a page ships now.
 */
type SliceEnglish = {
  /**
   Index of the slice in page order.
   */
  readonly sliceIndex: number;

  /**
   The English the slice ships; empty for a slice that ships nothing.
   */
  readonly english: string;
};

/**
 The facts of slices of the cat page from what each ships now, an empty English being a slice that ships nothing.

 @param englishes - slice index and English of each slice, in page order

 @returns Facts of the slices

 @example
 ```ts
 const slices = factsOf({ englishes: [{ sliceIndex: 1, english: 'Later the kitten naps.', },], },);
 ```
 */
export function factsOf(
  { englishes, }: { readonly englishes: readonly SliceEnglish[]; },
): readonly ConsistencySliceFacts[] {
  return englishes.map(function sliceOf(
    {
      sliceIndex,
      english,
    },
  ): ConsistencySliceFacts {
    return {
      sliceIndex,
      sourceIndex: sliceIndex,
      sourceText: '小猫打盹。',
      sourceStart: 0,
      nodes: [],
      current: (english === '')
        ? { kind: 'absent', }
        : {
          kind: 'present',
          text: english,
        },
      lineStructured: false,
      disputedWordings: [],
    };
  },);
}

/**
 The settled groups and the facts of the slices of a page.
 */
type RoundPage = {
  /**
   The settled groups, in the order the pass applies them.
   */
  readonly settled: readonly SettledGroup[];

  /**
   The facts of the slices, in page order.
   */
  readonly slices: readonly ConsistencySliceFacts[];
};

/**
 The round page: the settled groups and the facts of the three slices.

 @param third - third slice and span its splice replaces

 @returns Settled groups and facts of the slices

 @example
 ```ts
 const page = roundPage({ third: THIRD_WITH_CLOSER, },);
 ```
 */
export function roundPage({ third, }: { readonly third: RoundThird; },): RoundPage {
  return {
    settled: [
      withEdit({
        group: groupIn({
          places: [
            {
              slice: 0,
              english: nonNullishOrThrow(ROUND_ENGLISH[0],),
              quote: 'round',
            },
            {
              slice: 1,
              english: nonNullishOrThrow(ROUND_ENGLISH[1],),
              quote: 'round',
            },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 1,
          start: 8,
          end: 13,
          text: '’round',
        },
      },),
      withEdit({
        group: groupIn({
          places: [
            {
              slice: 0,
              english: nonNullishOrThrow(ROUND_ENGLISH[0],),
              quote: 'The folks',
            },
            {
              slice: 2,
              english: third.english,
              quote: third.quote,
            },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 2,
          start: 5,
          end: third.end,
          text: 'the folks',
        },
      },),
    ],
    slices: factsOf({
      englishes: [
        {
          sliceIndex: 0,
          english: nonNullishOrThrow(ROUND_ENGLISH[0],),
        },
        {
          sliceIndex: 1,
          english: nonNullishOrThrow(ROUND_ENGLISH[1],),
        },
        {
          sliceIndex: 2,
          english: third.english,
        },
      ],
    },),
  };
}

/**
 The round page with a third group that changes the second slice beside the first group's splice.

 @returns Settled groups and facts of the slices

 @example
 ```ts
 const page = roundPageWithTwoEdits();
 ```
 */
export function roundPageWithTwoEdits(): RoundPage {
  /**
   The round page the third group is added to.
   */
  const page = roundPage({ third: THIRD_WITH_CLOSER, },);
  return {
    settled: [
      ...page.settled,
      withEdit({
        group: groupIn({
          places: [
            {
              slice: 0,
              english: nonNullishOrThrow(ROUND_ENGLISH[0],),
              quote: 'The folks',
            },
            {
              slice: 1,
              english: nonNullishOrThrow(ROUND_ENGLISH[1],),
              quote: 'yard',
            },
          ],
        },),
        edit: {
          kind: 'splice',
          placePositions: [1,],
          sliceIndex: 1,
          start: 18,
          end: 22,
          text: 'garden',
        },
      },),
    ],
    slices: page.slices,
  };
}

/**
 What applying the round page came to, for a case to compare.

 @param third - third slice and span its splice replaces

 @param declaredNames - declared name forms the floors keep in each slice

 @returns Outcome of each place of each group, slices that changed with their text, and findings

 @example
 ```ts
 const seen = roundApplied({ third: THIRD_WITH_CLOSER, declaredNames: ['parents',], },);
 ```
 */
export function roundApplied(
  {
    third,
    declaredNames,
  }: {
    readonly third: RoundThird;
    readonly declaredNames: readonly string[];
  },
): unknown {
  /**
   What the pass made of the round page.
   */
  const applied = applyConsistency({
    ...roundPage({ third, },),
    declared: [],
    declaredNames,
    l: levelCapturingLogger({ lines: [], },),
  },);
  /**
   The groups the pass settled, to read the outcome of each place.
   */
  const { groups, } = applied;
  /**
   The slices the pass changed, to read their text.
   */
  const { slices, } = applied;
  return {
    outcomes: groups.map(function outcomesOf({ occurrences, },): readonly string[] {
      return Array.from(
        occurrences,
        function outcomeOf({ outcome, },): string {
          return outcome;
        },
      );
    },),
    changed: slices.map(function pairOf(
      {
        sliceIndex,
        text,
      },
    ): readonly [
      number,
      string,
    ] {
      return [
        sliceIndex,
        text,
      ];
    },),
    findings: applied.findings,
  };
}

/**
 What the pass reports when the floors refuse the third slice of the page whose closer comes back, with its words taken out
 for `the folks`.

 @returns Finding

 @throws {@link Error} when the floors pass the slice, which would leave the case with no refusal to test

 @example
 ```ts
 const finding = floorsFindingOfThird();
 ```
 */
export function floorsFindingOfThird(): string {
  /**
   What the floors say of the third slice with its words taken out.
   */
  const verdict = consistencyFloorVerdict({
    sliceIndex: 2,
    sourceText: '小猫打盹。',
    currentText: THIRD_WITH_CLOSER.english,
    candidateText: 'Fill the folks bowl now.',
    lineStructured: false,
    declared: [],
    declaredNames: ['parents',],
    disputedWordings: [],
  },);
  if (verdict.kind !== 'refused')
    throw new Error('Expected the floors to refuse the slice that loses the declared name',);
  /**
   The reasons the floors gave, to put in the finding.
   */
  const { findings, } = verdict;
  return `slice 2: the floors refused the consistency replacement: ${findings.join(' | ',)}`;
}
