import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { UnfilledSlice, } from './translate-document-contract.ts';
import type { BoughtSlice, } from './translate-slice-buy.ts';
import type { TranslateSliceSettlement, } from './translate-slice-settle.ts';

//region Floor-limited slices
// A SLICE THAT ENDED UNFILLED BECAUSE THE BENCH THAT DECIDES IT COULD NOT REACH ITS FLOOR is not asked again by a later
// attempt of the same pass unless that bench has grown (XingZ6020, 2026-10-22: slices 88 to 105 were tried with one
// select judge reachable of seven, each ended unfilled for want of a second ballot, and the next attempt asked all
// eighteen again).
//
// THE PASS HOLDS THE MEMORY, one per entry, so a later attempt of the same entry finds what an earlier one left. A later
// pass starts with none and asks again as it always did, since the bench may be healthy by then.
//
// ONLY THE FLOOR-LIMITED CASE IS REMEMBERED. A panel that met its floor and backed nothing twice has said something
// about the slice, and a later attempt may still ask it as it does now. A reach that has grown asks again, even to a
// count still under the floor, because one more seat is the only thing a repeat could change.

/**
 What a pass remembers of one slice that ended unfilled on a bench below its floor.

 @example
 ```ts
 const record: FloorLimitedRecord = { reachable: 1, unfilled, findings: [], told: false, };
 ```
 */
export type FloorLimitedRecord = {
  /**
   Select judges a wet provider would serve when the slice ended.
   */
  readonly reachable: number;

  /**
   The outcome the slice keeps while the bench has not grown.
   */
  readonly unfilled: UnfilledSlice;

  /**
   The findings the slice reported when it ended, kept so a skipped attempt reports the same.
   */
  readonly findings: readonly string[];

  /**
   Whether a line has said why the slice is not asked again, which is said once per slice.
   */
  readonly told: boolean;
};

/**
 Slices of one entry that ended floor-limited, by the question each answered.

 @example
 ```ts
 const memory: FloorLimitedMemory = new Map();
 ```
 */
export type FloorLimitedMemory = Map<string, FloorLimitedRecord>;

/**
 Names the question a slice answered, which a later attempt of the same entry asks again unchanged.

 @param sliceIndex - slice within the entry

 @param sourceText - original of the slice

 @param incumbentText - wording the question stands on, the archive's or the repair text standing in for it

 @returns Key into the memory

 @example
 ```ts
 const key = floorLimitedKey({ sliceIndex: 4, sourceText, incumbentText: '', },);
 ```
 */
export function floorLimitedKey(
  {
    sliceIndex,
    sourceText,
    incumbentText,
  }: {
    readonly sliceIndex: number;
    readonly sourceText: string;
    readonly incumbentText: string;
  },
): string {
  return JSON.stringify([
    sliceIndex,
    sourceText,
    incumbentText,
  ],);
}

/**
 Records or forgets one slice after it was bought.

 @param memory - the entry's memory, absent for a caller that keeps none

 @param key - question the slice answered

 @param bought - what the purchase returned

 @param reachable - select judges a wet provider would serve, absent where the hook counted none

 @example
 ```ts
 rememberFloorLimited({ memory, key, bought, reachable: 1, },);
 ```
 */
export function rememberFloorLimited(
  {
    memory,
    key,
    bought,
    reachable,
  }: {
    readonly memory?: FloorLimitedMemory;
    readonly key: string;
    readonly bought: BoughtSlice;
    readonly reachable?: number;
  },
): void {
  if (memory === undefined)
    return;
  if ((bought.kind === 'unfilled') && (reachable !== undefined)) {
    /**
     What the slice ended as.
     */
    const { unfilled, } = bought;
    /**
     Whether the panel declined the slate twice, which is what a bench short of a second ballot leaves.
     */
    const declined = unfilled.reason === 'no-candidate-backed';
    if (declined && (reachable < MIN_SELECTION_BALLOTS)) {
      memory.set(
        key,
        {
          reachable,
          unfilled,
          findings: bought.findings,
          told: false,
        },
      );
      return;
    }
  }
  memory.delete(key,);
}

/**
 Whether a slice is asked or keeps the outcome an earlier attempt left it.

 @example
 ```ts
 const decision: AskDecision = { kind: 'ask', };
 ```
 */
export type AskDecision = {
  readonly kind: 'ask';
} | {
  readonly kind: 'kept';

  /**
   The unfilled outcome the slice keeps, reported as the earlier attempt reported it.
   */
  readonly settlement: TranslateSliceSettlement;
};

/**
 Decides whether a slice is asked again, or keeps the outcome an earlier attempt left it while its bench has not grown.

 @param memory - the entry's memory, absent for a caller that keeps none

 @param key - question the slice answers

 @param reachable - select judges a wet provider would serve now, absent where the hook counted none

 @param benchSize - select judges seated

 @param sliceIndex - slice within the entry

 @param l - lane logger

 @returns The unfilled outcome the slice keeps, or that it is to be asked

 @example
 ```ts
 const decision = notAskedAgain({ memory, key, reachable: 1, benchSize: 7, sliceIndex: 4, l, },);
 ```
 */
export function notAskedAgain(
  {
    memory,
    key,
    reachable,
    benchSize,
    sliceIndex,
    l,
  }: {
    readonly memory?: FloorLimitedMemory;
    readonly key: string;
    readonly reachable?: number;
    readonly benchSize: number;
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): AskDecision {
  if (memory === undefined)
    return { kind: 'ask', };
  /**
   What an earlier attempt of this pass left for the slice.
   */
  const remembered = memory.get(key,);
  if (remembered === undefined)
    return { kind: 'ask', };
  if (reachable === undefined)
    return { kind: 'ask', };
  if (reachable > remembered.reachable)
    return { kind: 'ask', };
  if (!remembered.told) {
    /**
     Why the earlier attempt left the slice unfilled.
     */
    const { reason, } = remembered.unfilled;
    l.info(
      [
        `slice ${String(sliceIndex,)}: not asked again; it ended unfilled (${reason}) with `,
        `${String(remembered.reachable,)} of ${String(benchSize,)} select judges reachable against a floor of `,
        `${String(MIN_SELECTION_BALLOTS,)}, and ${String(reachable,)} are reachable now; a later pass asks again`,
      ].join('',),
    );
    memory.set(
      key,
      {
        ...remembered,
        told: true,
      },
    );
  }
  /**
   The finding naming why this attempt did not ask.
   */
  const finding = [
    `translate-not-asked-again (chunk ${String(sliceIndex,)}, select reach ${String(reachable,)} of `,
    `${String(benchSize,)}, recorded ${String(remembered.reachable,)}, floor ${String(MIN_SELECTION_BALLOTS,)})`,
  ].join('',);
  return {
    kind: 'kept',
    settlement: {
      outcome: {
        kind: 'unfilled',
        unfilled: remembered.unfilled,
      },
      refusedCacheFindings: [],
      unfilledFindings: [
        ...remembered.findings,
        finding,
      ],
    },
  };
}

//endregion Floor-limited slices
