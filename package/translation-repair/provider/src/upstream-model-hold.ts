import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Upstream model hold
// THE SIXTY-FOURTH CLASS (XingZ613, 2026-09-19). Class sixty-two lets a
// model whose upstream endpoint rate-limited it lose its seat for the round
// without holding the whole provider out; but the next round asked the same
// model again, and every round waited its five-attempt ladder out in grace:
// on the stages Mercury 2.5 sat on the mean grace wait rose from 0 to 6 s to
// 14 to 32 s, and the repair lane took 1h39m against XingZ608's 35m. The
// provider hold of class fourteen scoped to the one model is what the
// aggregator's reply asks for ("retry shortly"): the model is held out for
// the rate-limit backoff and refused at call time as unreachable, so a
// round neither asks nor waits for it, and every other model on the
// provider is served meanwhile.

/**
 How long a model whose upstream endpoint rate-limited it stays off the
 bench before it is asked again; the provider hold's own backoff.
 */
export const UPSTREAM_MODEL_HOLD_MS = 60_000;

/**
 Holds on single models, keyed by model id.

 @example
 ```ts
 const holds = createUpstreamModelHolds({ holdMs: UPSTREAM_MODEL_HOLD_MS, now: monotonicMs, },);
 holds.hold({ modelId: 'inception/mercury-2.5', },);
 holds.remainingMs({ modelId: 'inception/mercury-2.5', },); // 60000
 ```
 */
export type UpstreamModelHolds = {
  /**
   Starts, or restarts, one model's hold.
   */
  readonly hold: (args: { readonly modelId: string; },) => void;

  /**
   Milliseconds one model's hold has left, zero when it is free.
   */
  readonly remainingMs: (args: { readonly modelId: string; },) => number;

  /**
   Refuses a call for a model whose hold has not ended, before any call is made, so a round counts the seat
   unreachable and neither asks nor waits for it (class sixty-four).

   @throws {@link NoProviderForModelError} while the model is held
   */
  readonly refuseWhileHeld: (args: { readonly modelId: string; },) => void;
};

/**
 Builds the per-model holds one router keeps.

 @param holdMs - how long a hold lasts

 @param now - clock the holds are read by, injectable for tests; the router passes `monotonicMs` (ledger B78)

 @returns Holds shared by every call the router makes

 @example
 ```ts
 const holds = createUpstreamModelHolds({ holdMs: UPSTREAM_MODEL_HOLD_MS, now: monotonicMs, },);
 ```
 */
export function createUpstreamModelHolds(
  {
    holdMs,
    now,
  }: {
    readonly holdMs: number;
    readonly now: () => number;
  },
): UpstreamModelHolds {
  /**
   When each held model's hold ends, by model id.
   */
  const until = new Map<string, number>();

  /**
   Milliseconds one model's hold has left, zero when it is free, a lapsed hold forgotten.

   @param modelId - model asked about

   @returns Milliseconds left
   */
  function remainingMs({ modelId, }: { readonly modelId: string; },): number {
    /**
     When this model's hold ends, absent when it was never held.
     */
    const ends = until.get(modelId,);
    if (ends === undefined)
      return 0;
    /**
     What is left of the hold, negative once it has ended.
     */
    const left = ends - now();
    if (left <= 0) {
      until.delete(modelId,);
      return 0;
    }
    return left;
  }
  return {
    hold: function hold({ modelId, }: { readonly modelId: string; },): void {
      until.set(
        modelId,
        now() + holdMs,
      );
    },
    remainingMs,
    refuseWhileHeld: function refuseWhileHeld({ modelId, }: { readonly modelId: string; },): void {
      /**
       What is left of this model's own hold, zero when it is free.
       */
      const heldMs = remainingMs({ modelId, },);
      if (heldMs > 0) {
        throw new NoProviderForModelError({
          modelId,
          reason: `its upstream endpoint rate-limited this model; held out for another ${String(heldMs,)}ms`,
        },);
      }
    },
  };
}

//endregion Upstream model hold
