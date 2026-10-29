import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { FutileReturn, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RunClient, } from './run-client-contract.ts';
import { passBoundOf, } from './run-seats-pass-bound.ts';

//region Returns that served nothing
// A PROVIDER WHOSE LAST RETURN SERVED NOTHING IS NOT WAITED FOR (the owner's answer of 2026-10-29, "Remember futile
// returns"; measured on XingZ6020, where Synthetic refused again each time it came back and the waits for it summed to
// 31,509.8 s and bought 3 gates). The budgets keep the evidence (`provider-budget-futile-returns.ts`): a hold ended in
// the entry, no call the provider answered came after the end, and it refused again. Here the seat reader leaves such a
// provider's hold out of the holds it waits for, so a stage short of a bench goes on short and says why. The provider
// stays routable and a call it answers clears the mark, so it is waited for again after it serves.
//
// THE HOLDS ARE LEFT OUT, NOT THE LIFT: the reader's later questions (which holds cure a short bench, the ceiling the
// holds could lift it to, the words for a reading that takes no wait) are asked of the holds that remain, so they stay
// as they were and none of them learns a new input. A provider whose return would let the entry finish is not listed
// by the budgets, so it is never left out.
//
// NEURALWATT'S DAY START IS NOT A RETURN THAT CAN SERVE NOTHING: while the hold that keeps NeuralWatt out is its day
// start, a mark from a refusal hold is not read, as the meters' dryness is not read for it either.

/**
 A hold the reader left out, with the return that explains it.
 */
export type FutileSkip = FutileReturn & {
  /**
   How long the hold would have been waited for.
   */
  readonly holdLeftMs: number;
};

/**
 What the reader left out of the holds it waits for.

 @example
 ```ts
 const skipped: FutileSkips = new Map();
 ```
 */
export type FutileSkips = ReadonlyMap<ProviderName, FutileSkip>;

/**
 Leaves out of the running holds those of providers whose last return served nothing.

 @param client - run client, whose futile returns and NeuralWatt day start are read when it carries them

 @param running - how long each provider's refusal still holds it out, as the pass allows it to be waited for

 @returns The holds to wait for, and the holds left out with the return behind each

 @example
 ```ts
 const { held, skipped, } = leaveOutFutileReturns({ client, running: holdsWithinPass({ client, },), },);
 ```
 */
export function leaveOutFutileReturns(
  {
    client,
    running,
  }: {
    readonly client: Pick<RunClient, 'futileReturns' | 'neuralwattHeldByReturn' | 'passLeftMs'>;
    readonly running: ProviderRecord<number>;
  },
): {
  readonly held: ProviderRecord<number>;
  readonly skipped: FutileSkips;
} {
  /**
   Nothing left out.
   */
  const none = {
    held: running,
    skipped: new Map<ProviderName, FutileSkip>(),
  };
  if (client.futileReturns === undefined)
    return none;
  if (!PROVIDER_ORDER.some(function isHeld(provider,): boolean {
    return running[provider] > 0;
  },))
    return none;
  /**
   What the budgets know of each provider's last return.
   */
  const returns = client.futileReturns();
  /**
   Whether the hold that keeps NeuralWatt out is its day start, which no refusal's mark speaks for.
   */
  const dayStart = client.neuralwattHeldByReturn?.({ returnBoundMs: passBoundOf({ client, }), },) ?? true;
  /**
   The holds left out, with the return behind each.
   */
  const skipped = new Map<ProviderName, FutileSkip>();
  for (const provider of PROVIDER_ORDER) {
    /**
     This provider's last return, when it served nothing.
     */
    const mark = returns.get(provider,);
    if ((running[provider] === 0) || (mark === undefined))
      continue;
    if ((provider === 'neuralwatt') && dayStart)
      continue;
    skipped.set(
      provider,
      {
        ...mark,
        holdLeftMs: running[provider],
      },
    );
  }
  if (skipped.size === 0)
    return none;
  return {
    held: providerRecord({
      of: function holdWorthWaiting(provider,): number {
        return skipped.has(provider,) ? 0 : running[provider];
      },
    },),
    skipped,
  };
}

/**
 Says which providers' holds the reader does not wait for, and why, for a reading that found a bench short.

 @param skipped - holds the reader left out

 @param phase - phase the reading seats, which the line names

 @param l - entry logger

 @example
 ```ts
 sayFutileSkips({ skipped, phase, l, },);
 ```
 */
export function sayFutileSkips(
  {
    skipped,
    phase,
    l,
  }: {
    readonly skipped: FutileSkips;
    readonly phase: string;
    readonly l: Logger;
  },
): void {
  for (const [provider, skip,] of skipped) {
    l.warn(
      `JUDGE SEATS phase=${phase} not waiting for ${provider}: its last return served nothing (its ${
        String(skip.holdMs,)
      }ms hold ended ${
        String(skip.refusedAfterMs,)
      }ms before it refused again, and no call it answered came after), so its hold of ${
        String(skip.holdLeftMs,)
      }ms is not waited for while a bench is short; it is waited for again once it answers a call`,
    );
  }
}

//endregion Returns that served nothing
