import { presentNamesOfKind, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { EntryOutcome, } from './pass-entry-contract.ts';
import { isSliceFileName, } from './slice-cache-namespace.ts';

//region Entry reattempt
// Why a capped entry gets another go INSIDE one invocation, and what stops it.
//
// The largest entries cannot settle inside `HARD_CAP_MINUTES` in one attempt.
// XingZ60 was measured at 45 repair-lane slices in 4 h 51 m of a 420-minute
// cap, with 119 slices prepared and a translate lane of the same size behind
// it, and recorded three ways out. Its first was to run such an entry as a
// SEQUENCE of attempts against a frozen build, noted as costing nothing to
// build and needing the build to stop moving.
//
// THE BUILD ALREADY STOPS MOVING, for exactly as long as one invocation. The
// pass digests `dist/final/node` once before its loop and every cache it opens
// is namespaced by that digest, so nothing landing in the repository can
// invalidate a cache while the loop runs. What actually prevented the sequence
// was scheduling alone: the loop ran over a list built before it started, so a
// capped entry was visited once and the invocation moved on to other entries
// for up to the three-day soft budget.
//
// So the sequence needs neither a frozen build nor operator discipline. It
// needs the loop to come back, and the cost of coming back is nothing, because
// the slices the capped attempt bought are already on disk.
//
// WHAT STOPS IT COMING BACK FOREVER is the harder half, and this codebase had
// already written the hazard down in `corpus-pass.ts`: no progress guarantee
// holds. An abort can land before the first persistence, and the slices a lane
// deliberately leaves uncached, the unfilled and the unheard, produce no cache
// entry however long they took. A loop that retried until settled would spend a
// three-day budget on one entry that never moves, which is worse than the cap.
//
// This decides on MEASURED progress instead. An attempt that left more slices
// cached than it found has moved the entry forward and earns another; one that
// left the same number has not and does not. That reads the thing a second
// attempt is actually for, since its whole value is the cache the first one
// filled.
//
// WHAT COUNTS AS PROGRESS IS EVERY CACHE RECORD, not only a translated slice,
// and that is deliberate rather than incidental. `slice-cache-namespace.ts`
// gives the pairing, contest, refine and translate records the same `.json`
// suffix in the same per-entry directory, so all of them are counted here.
//
// The first live run, 2026-08-24, is what this is written from. Its first
// attempt spent 45 seconds buying two block pairings and was then cut, and
// those two records are the whole of what it banked. They earned the
// re-attempt, and the re-attempt reached the lanes in 0.16 seconds instead of
// 45 by reading them back. Had setup cached anywhere else, the largest entries,
// which are the ones this exists for, would have stalled on every first
// attempt.

/**
 What one attempt earned, and so whether the invocation comes back to it.
 */
export type ReattemptVerdict =
  | {
    /**
     Entry reached its artifact, so nothing is owed.
     */
    readonly kind: 'settled';
  }
    | {
    /**
     Entry did not settle and bought no slice it did not already hold, so a
     further attempt inside this invocation would repeat this one.
     */
      readonly kind: 'stalled';

    /**
     Slices it holds, which is what it held before this attempt too.
     */
      readonly cached: number;
    }
    | {
    /**
     Incomplete work explicitly forbids fresh whole-entry attempt.
     */
      readonly kind: 'stopped';
    }
    | {
    /**
     The pipeline declined the entry by the archive's own note; its decline
     record stands where the artifact would and no pass attempts it again.
     */
      readonly kind: 'declined';
    }
    | {
    /**
     Entry did not settle and cached slices it did not have before, so the
     next attempt starts further along than this one did.
     */
      readonly kind: 'earned';

    /**
     Slices this attempt bought.
     */
      readonly gained: number;
    };

/**
 Counts slices one entry has cached, across every lane sharing its directory.

 COUNTS ALL LANES DELIBERATELY. Progress is progress whichever lane made it,
 and an entry whose repair lane is capped while its translate lane advances is
 moving forward exactly as much as one where both do.

 @param dir - per-entry cache directory, absent before its first slice

 FILES ONLY: a directory named like a slice was bought by no attempt (ledger
 B65).

 @returns How many slices sit there, zero when nothing has been cached

 @example
 ```ts
 const cached = await countCachedSlices({ dir: entryCacheDir, },);
 ```
 */
export async function countCachedSlices(
  { dir, }: { readonly dir: string; },
): Promise<number> {
  return (await presentNamesOfKind({
    dir,
    kind: 'file',
  },))
    .filter(function isSlice(name,): boolean {
      return isSliceFileName({ name, },);
    },)
    .length;
}

/**
 Slices one attempt bought, which is not always the difference in counts.

 A COUNT THAT FELL MEANS THE CACHE WAS RESET rather than that work was lost.
 An entry carrying slices from an earlier build has them discarded when its
 lane opens under this invocation's digest, so every slice present afterwards
 was bought by this attempt and the plain difference reads as negative.
 Subtracting alone would call that attempt stalled and drop the entry at
 precisely the moment it had started paying for a fresh generation.

 @param cachedBefore - slices present when this attempt started

 @param cachedAfter - slices present when it stopped

 @returns Slices this attempt is responsible for

 @example
 ```ts
 const bought = slicesBought({ cachedBefore: 45, cachedAfter: 64, },);
 ```
 */
function slicesBought(
  {
    cachedBefore,
    cachedAfter,
  }: {
    readonly cachedBefore: number;
    readonly cachedAfter: number;
  },
): number {
  if (cachedAfter >= cachedBefore)
    return cachedAfter - cachedBefore;
  return cachedAfter;
}

/**
 Reads one attempt into the decision of whether to make another.

 TAKES SETTLEMENT AS AN INPUT rather than inferring it from the cache, because
 a settled entry DISCARDS its slice cache on the way out. Inferring would read
 that discard as the sharpest possible stall and would be wrong about the one
 outcome the whole pass exists to reach.

 @param outcome - scheduling disposition from entry pipeline

 @param cachedBefore - slices present when this attempt started

 @param cachedAfter - slices present when it stopped

 @returns Verdict naming what happened and what it earns

 @example
 ```ts
 const verdict = readAttemptOutcome({ outcome: { kind: 'resumable-failure' }, cachedBefore: 45, cachedAfter: 64, },);
 ```
 */
export function readAttemptOutcome(
  {
    outcome,
    cachedBefore,
    cachedAfter,
  }: {
    readonly outcome: EntryOutcome;
    readonly cachedBefore: number;
    readonly cachedAfter: number;
  },
): ReattemptVerdict {
  if (outcome.kind === 'settled')
    return { kind: 'settled', };
  if (outcome.kind === 'stopped')
    return { kind: 'stopped', };
  if (outcome.kind === 'declined')
    return { kind: 'declined', };

  /**
   Slices this attempt is responsible for, reset-aware.
   */
  const gained = slicesBought({
    cachedBefore,
    cachedAfter,
  },);

  if (gained > 0)
    return {
      kind: 'earned',
      gained,
    };

  return {
    kind: 'stalled',
    cached: cachedAfter,
  };
}

//endregion Entry reattempt
