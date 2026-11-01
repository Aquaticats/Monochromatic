import type { BoundOwner, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RunClient, } from '@monochromatic-dev/translation-repair-run/ts';

//region Entry bound
// INSIDE AN ENTRY THE SEAT READER'S BOUND IS THE SMALLER OF WHAT THE PASS HAS LEFT AND WHAT THE ENTRY'S OWN HARD CAP
// HAS LEFT. The pass-level bound (`corpus-pass-run.ts`) counts the soft budget left, capped at the hard cap as a
// constant; but the hard cap is a per-entry timer armed when the entry starts, so an entry that has run for hours has
// less than the whole cap left, and a wait for a NeuralWatt return that fits the constant can still outlast the entry's
// timer and end in a timeout with no stated refusal. The entry's client therefore carries a bound that falls with the
// entry's own time. The gate's park runs between entries on a fresh timer and keeps the pass-level bound.

/**
 The client an entry runs on, its pass bound narrowed to the time the entry's own cap has left.

 @param client - pass's client, whose bound (when it carries one) counts the soft budget left

 @param hardCapMs - how long the entry may run before its timer aborts it

 @param startedAt - when the entry began, on the clock `now` reads

 @param now - monotonic clock in milliseconds

 @returns Client with a bound that is the smaller of its own and what the entry's cap has left, which may be zero
 or negative once the cap is spent

 @example
 ```ts
 const entryClient = withEntryBound({ client, hardCapMs: 25_200_000, startedAt: monotonicMs(), now: monotonicMs, },);
 ```
 */
export function withEntryBound(
  {
    client,
    hardCapMs,
    startedAt,
    now,
  }: {
    readonly client: RunClient;
    readonly hardCapMs: number;
    readonly startedAt: number;
    readonly now: () => number;
  },
): RunClient {
  /**
   What the entry's own cap has left now.

   @returns Milliseconds, zero or below once the cap is spent
   */
  function capLeftMs(): number {
    return hardCapMs - (now() - startedAt);
  }
  /**
   What the pass has left now, unbounded for a client that carries no bound.

   @returns Milliseconds
   */
  function passLeftMs(): number {
    return client.passLeftMs?.() ?? Number.POSITIVE_INFINITY;
  }
  return {
    ...client,
    passLeftMs: function entryLeftMs(): number {
      return Math.min(
        passLeftMs(),
        capLeftMs(),
      );
    },
    passBoundOwner: function entryBoundOwner(): BoundOwner {
      return (capLeftMs() < passLeftMs()) ? 'entry' : 'pass';
    },
  };
}

//endregion Entry bound
