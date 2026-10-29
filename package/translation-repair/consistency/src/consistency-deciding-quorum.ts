import { MIN_STAGE_VOICES, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Consistency deciding quorum
// ONE JUDGE ALONE NEVER DECIDES (the lead's ruling of 2026-10-25, on the owner's rule). A gather on a seated bench of one
// or two seats closes on a quorum of one by design of the reachable quorum (`stage/src/stage-quorum.ts`), which this
// file leaves alone: what it bounds is the wait, not the count. A consistency round decides only on the usable ballots
// of two or more judges, so the quorum a count applies is the gather's, raised to the two-voice floor.

/**
 Usable ballots a consistency count needs before it decides anything: the quorum the gather closed on, never below
 the two voices a decision needs.

 @param quorum - voices the gather closed on, one on a seated bench of one or two seats

 @returns Quorum the usable ballots must reach

 @example
 ```ts
 const needed = decidingQuorumOf({ quorum: 1, },); // => 2
 ```
 */
export function decidingQuorumOf({ quorum, }: { readonly quorum: number; },): number {
  return Math.max(
    quorum,
    MIN_STAGE_VOICES,
  );
}

//endregion Consistency deciding quorum
