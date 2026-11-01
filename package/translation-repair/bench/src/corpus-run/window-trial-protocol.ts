import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { RUN_ROSTER, } from '@monochromatic-dev/translation-repair-run/ts';

//region Window trial protocol
// The two rules of the trial that decide nothing about models and can be held
// by a test: what a run buys under, and when a run of refusals is the run's
// fault. Lifted out of `window-trial-probe.ts`, which is the command and stays
// out of the bundle's barrel the way every command does.

/**
 Version of this trial's protocol, bumped when what a row MEANS changes.

 Folded into the protocol digest, so a change here stops a later run resuming
 rows bought under the older meaning rather than silently pooling them.

 Version 2: the arms are judged on the sheets the pass asks, carrying the
 identity block, the date of death and the tense candidates where the entry
 records one, where version 1 judged on sheets without any of them.
 */
const TRIAL_PROTOCOL_VERSION = 2;

/**
 Digest of everything this run buys under.

 ROSTER, CORPUS PIN, CODE AND PROTOCOL TOGETHER. A trial re-run after any of
 them moved is asking a different question, and resuming across that boundary
 would pool two experiments into one tally. The ledger skips on this, so
 getting it wrong is what silently mixes them.

 @internal

 @param headSha - commit the pipeline is running at

 @param pin - corpus clone and commit the trial reads its slices at, the
 run's own; until 2026-10-10 the digest folded in the run's module constant
 whatever pin the trial was handed (ledger B302, B381). Its two halves are
 folded in the order the pin reader writes them, clone then commit, so a
 ledger keyed before the move still resumes at the same pin

 @returns Digest the ledger keys resumption on

 @example
 ```ts
 const protocol = protocolDigest({ headSha, pin, },);
 ```
 */
export function protocolDigest(
  {
    headSha,
    pin,
  }: {
    readonly headSha: string;
    readonly pin: CorpusPin;
  },
): string {
  return hashContent({
    content: JSON.stringify([
      'window-trial',
      TRIAL_PROTOCOL_VERSION,
      RUN_ROSTER,
      {
        cloneDir: pin.cloneDir,
        commitSha: pin.commitSha,
      },
      headSha,
    ],),
  },);
}

/**
 What one drawn slice yielded, as far as the refusal streak reads it.

 @example
 ```ts
 const yielded: SliceYield = 'already-held';
 ```
 */
export type SliceYield = 'refused' | 'already-held' | 'bought';

/**
 Refusals in a row after one slice.

 A SLICE THE LEDGER ALREADY HELD LEAVES THE STREAK WHERE IT WAS. It bought
 nothing, so it says nothing about whether the run can still buy; resetting on
 it let a resumed run refuse every new slice without ever reaching the stop,
 each refusal still paying for a producer slate.

 @internal

 @param refusedInARow - streak before this slice

 @param yielded - what the slice yielded

 @returns Streak after it

 @example
 ```ts
 const streak = streakAfter({ refusedInARow: 2, yielded: 'already-held', },);
 ```
 */
export function streakAfter(
  {
    refusedInARow,
    yielded,
  }: {
    readonly refusedInARow: number;
    readonly yielded: SliceYield;
  },
): number {
  if (yielded === 'refused')
    return refusedInARow + 1;
  if (yielded === 'already-held')
    return refusedInARow;
  return 0;
}

//endregion Window trial protocol
