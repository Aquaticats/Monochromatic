import type { RunClient, } from './run-client-contract.ts';

//region Seat reader pass bound
// WHAT THE SEAT READER'S WAIT FOR A NEURALWATT RETURN IS BOUNDED BY, split out of `run-seats-short-bench.ts` and
// `run-seats-refusal.ts` so both read the bound one way: the pass's own (inside an entry, the smaller of the pass's and
// the entry's cap, `pass-entry-bound.ts`), and unbounded for a client built outside a pass.

/**
 The bound a client carries on the wait for a return, unbounded where it carries none.

 @param client - run client, whose pass bound is read when it carries one

 @returns Milliseconds the reader may wait, which is zero or below once the pass or the entry's cap is spent

 @example
 ```ts
 const boundMs = passBoundOf({ client, },);
 ```
 */
export function passBoundOf({ client, }: { readonly client: Pick<RunClient, 'passLeftMs'>; },): number {
  return client.passLeftMs?.() ?? Number.POSITIVE_INFINITY;
}

/**
 How long each provider is held out as the seat reader waits for it: a NeuralWatt return counts as a hold only when it
 lies inside what the pass has left, so a return past it is not waited for and the reading refuses at once.

 @param client - run client, whose pass bound is read when it carries one

 @returns Milliseconds of hold left per provider

 @example
 ```ts
 const holds = holdsWithinPass({ client, },);
 ```
 */
export function holdsWithinPass(
  { client, }: { readonly client: Pick<RunClient, 'passLeftMs' | 'providerHolds'>; },
): ReturnType<RunClient['providerHolds']> {
  return client.providerHolds({ returnBoundMs: passBoundOf({ client, }), },);
}

//endregion Seat reader pass bound
