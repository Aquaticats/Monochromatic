import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { benchBelowFloorStageFinding, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  absenceFinding,
  type IncumbentKind,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { BoughtSlice, } from './translate-slice-buy.ts';

//region Floor-limited slices
// A PASSAGE THE ARCHIVE NEVER TRANSLATED IS NOT BOUGHT AT ALL UNDER THE FLOOR. A winner of a selection round needs
// `MIN_SELECTION_BALLOTS` ballots whatever the weight (`candidate-select.ts`), and a round over one reachable seat
// casts at most one, so every candidate such a slice buys can only be declined, and the slice ends
// `no-candidate-backed` (XingZ6020 bought slices 88 to 117 that way). The skip sits where a purchase would start,
// after every path that can answer the slice without one: a stored record resumes before it, and a twin's record is
// reused before it (`reuseTwinOrBuy` runs the purchase only on a miss). No other path backs a candidate under the floor
// for a passage with no archive wording: the sole-candidate rule ships an incumbent, which the slice lacks; a challenge
// round's tie-break needs two candidates drawing weight and the preference shipped past a decline needs archive
// wording that cannot ship; the lane has no payload store. A slice the archive translates ends as a settled slice
// whatever its bench, so it is bought as before.
//
// THE SKIP NEEDS NO MEMORY. It reads the bench before each purchase, so a later attempt at the same reach skips the same
// slice again, and one at a reach of two or more buys it. The pass once kept a record of the slices it had skipped so a
// later attempt would not ask them again; the owner removed it on 2026-10-29 because the payload store already replays
// the repeat and the memory saved about nothing (B834 in the audit ledger, B1011).

/**
 Whether a passage's select bench reads under the floor, so that nothing bought for it could be backed.

 @param incumbentKind - whether the archive holds a translation here

 @param reachable - select judges a wet provider would serve, absent where the hook counted none

 @returns True for a passage the archive never translated read at a reach under the floor

 @example
 ```ts
 const skip = selectBelowFloor({ incumbentKind: 'absent', reachable: 1, },);
 ```
 */
export function selectBelowFloor(
  {
    incumbentKind,
    reachable,
  }: {
    readonly incumbentKind: IncumbentKind;
    readonly reachable?: number;
  },
): boolean {
  if (incumbentKind !== 'absent')
    return false;
  if (reachable === undefined)
    return false;
  return reachable < MIN_SELECTION_BALLOTS;
}

/**
 The outcome of a passage bought no candidate because its select bench reads under the floor: the unfilled slice it
 would have ended as, with the finding that names the bench and the line that says why.

 @param reachable - select judges a wet provider would serve

 @param benchSize - select judges seated

 @param sliceIndex - slice within the entry

 @param l - lane logger

 @returns Unfilled purchase, with no call made

 @example
 ```ts
 const bought = skippedBelowFloor({ reachable: 1, benchSize: 7, sliceIndex: 4, l, },);
 ```
 */
export function skippedBelowFloor(
  {
    reachable,
    benchSize,
    sliceIndex,
    l,
  }: {
    readonly reachable: number;
    readonly benchSize: number;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): BoughtSlice {
  /**
   Logger pre-tagged with this function's name.
   */
  const sl = tagged({
    tag: skippedBelowFloor.name,
    l,
  },);

  sl.warn(
    [
      `slice ${String(sliceIndex,)}: ${String(reachable,)} of ${String(benchSize,)} select `,
      `${
        wordForCount({
          count: benchSize,
          one: 'judge',
          many: 'judges',
        },)
      } reachable, below the floor of `,
      `${String(MIN_SELECTION_BALLOTS,)} `,
      `${
        wordForCount({
          count: MIN_SELECTION_BALLOTS,
          one: 'ballot',
          many: 'ballots',
        },)
      } a winner needs; `,
      'no candidate is bought and the passage stays unfilled (no-candidate-backed)',
    ].join('',),
  );
  /**
   What the slice reports of the bench.
   */
  const findings = [
    benchBelowFloorStageFinding({
      stage: 'translate-select',
      reachable,
      benchSize,
      floor: MIN_SELECTION_BALLOTS,
    },),
  ];
  return {
    kind: 'unfilled',
    unfilled: {
      sliceIndex,
      reason: 'no-candidate-backed',
      findings,
      benchBelowFloor: true,
    },
    findings: [
      ...findings,
      `${absenceFinding({ reason: 'no-candidate-backed', },)} chunk ${String(sliceIndex,)}`,
    ],
  };
}

//endregion Floor-limited slices
