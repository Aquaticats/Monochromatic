import type { RelabelCase, } from '../dist/final/node/index.mjs';

//region Relabel case
// A REBUILT REGION CASE, CAT-THEMED, for the cases that run a probe over
// regions the gatherers would rebuild from a corpus clone.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The gatherers have their own cases over a
// throwaway clone; these cases need only the shape they return.

/**
 Replaced wording of the nap edit.
 */
const NAP_BEFORE = 'The cat is doing the sleeping, and she wakes at dusk.';

/**
 Wording that replaced it.
 */
const NAP_AFTER = 'The cat sleeps.';

/**
 Builds the case of one nap edit in one entry.

 @param entryId - entry the edit belongs to

 @param positions - sample positions that drew it, none for a control

 @param recorded - what the run recorded about the edit

 @returns The case

 @example
 ```ts
 const relabelCase = napCase({ entryId: 'whiskers', positions: [2,], recorded: 'corroborated=0', },);
 ```
 */
export function napCase(
  {
    entryId,
    positions,
    recorded,
  }: {
    readonly entryId: string;
    readonly positions: readonly number[];
    readonly recorded: string;
  },
): RelabelCase {
  return {
    entryId,
    positions,
    region: {
      envelopeId: `envelope/nap-${entryId}`,
      issueIds: ['adjudicated/nap',],
      before: NAP_BEFORE,
      editorAfter: NAP_AFTER,
    },
    issues: [
      {
        issueId: 'adjudicated/nap',
        status: 'accepted',
        severity: 'major',
        claims: [],
        tallies: {},
      },
    ],
    sourceText: '猫猫在睡觉，黄昏时她会醒来。',
    baselineText: NAP_BEFORE,
    recorded,
  };
}

//endregion Relabel case
