/**
 Settled groups of the cat page for the tests of applying and recording the
 consistency replacements: a group whose first place wins and whose places stand
 in slices the case names, and the same group with a pending replacement.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  type PendingEdit,
  type SettledGroup,
  settleAsked,
} from '../../dist/final/node/index.mjs';
import { firstPlaceBallot, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/consistency-group.test-fixture.ts';
import { placeIn, } from './consistency-settle.test-fixture.ts';

/**
 The seats that judge a group of the cat page.
 */
const SEATS = [
  'hyper-cat',
  'bedrock-cat',
  'synthetic-cat',
] as const;

/**
 A group whose first place wins and whose places are settled in the slices it names.

 @param places - the slice and the English of each place, with the quote the judges give for it

 @returns The group, with no pending replacement

 @example
 ```ts
 const group = groupIn({ places: [{ slice: 1, english: 'Later the kitten naps in the sun.', quote: 'in the sun', },], });
 ```
 */
export function groupIn(
  { places, }: {
    readonly places: readonly {
      readonly slice: number;
      readonly english: string;
      readonly quote: string;
    }[];
  },
): SettledGroup {
  return settleAsked({
    repeat: {
      kind: 'maximal',
      wording: '小猫打盹',
    },
    places: places.map(function placeOf({
      slice,
      english,
    },) {
      return placeIn({
        position: slice,
        english,
      },);
    },),
    unplaced: [],
    asked: {
      quorum: 2,
      resumed: false,
      findings: [],
      voices: SEATS.map(function ballotOf(seat,) {
        return firstPlaceBallot({
          seat,
          quotes: places.map(function quoteOf(
            { quote, },
            position,
          ) {
            return {
              label: `OCCURRENCE ${String(position + 1,)}`,
              quote,
            };
          },),
        },);
      },),
    },
  },);
}

/**
 The group with one pending replacement.

 @param group - the group

 @param edit - the replacement

 @returns The group that would make it

 @example
 ```ts
 const pending = withEdit({ group, edit, },);
 ```
 */
export function withEdit(
  {
    group,
    edit,
  }: {
    readonly group: SettledGroup;
    readonly edit: PendingEdit;
  },
): SettledGroup {
  return {
    ...group,
    edits: [edit,],
  };
}
