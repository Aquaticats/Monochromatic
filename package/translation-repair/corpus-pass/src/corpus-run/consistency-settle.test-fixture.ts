/**
 Fixtures the tests of settling a group share: a place a slice holds whole and
 a run-off heard from a bench. Cat-themed invention; no corpus content appears
 here.

 @module
 */

import type {
  ConsistencyPlace,
  ConsistencyRunOffAsked,
} from '../../dist/final/node/index.mjs';

/**
 The repeated words every place of a group holds.
 */
const WORDS = '小猫打盹';

/**
 A place a slice holds whole, the slice carrying the English it is given.

 @param position - position of the place in its group, which is also its slice

 @param english - the English the place's slice carries

 @returns The place with the facts of its slice

 @example
 ```ts
 const place = placeIn({ position: 0, english: 'The cat naps.', },);
 ```
 */
export function placeIn(
  {
    position,
    english,
  }: {
    readonly position: number;
    readonly english: string;
  },
): ConsistencyPlace {
  return {
    startOffset: position * 10,
    endOffset: (position * 10) + WORDS.length,
    line: position + 1,
    slice: {
      sliceIndex: position,
      sourceIndex: position,
      sourceText: '小猫打盹',
      sourceStart: position * 10,
      nodes: [],
      current: {
        kind: 'present',
        text: english,
      },
      lineStructured: false,
      disputedWordings: [],
    },
  };
}

/**
 What a run-off heard.

 @param choices - the label each seat chose, in seat order

 @param quorum - how many ballots the gather needed

 @param findings - what the gather reported

 @returns The run-off as the settling reads it

 @example
 ```ts
 const runOff = runOffOf({ choices: ['OCCURRENCE 2', 'OCCURRENCE 2',], },);
 ```
 */
export function runOffOf(
  {
    choices,
    quorum = 2,
    findings = [],
  }: {
    readonly choices: readonly string[];
    readonly quorum?: number;
    readonly findings?: readonly string[];
  },
): ConsistencyRunOffAsked {
  return {
    kind: 'choice',
    quorum,
    voices: choices.map(function choiceOf(
      choice,
      at,
    ) {
      return {
        seat: `seat-${String(at,)}`,
        reason: 'It renders the original best.',
        choice,
      };
    },),
    findings,
    resumed: false,
  };
}
