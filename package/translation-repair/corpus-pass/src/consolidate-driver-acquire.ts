import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ConsolidationSettlement, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import {
  type BoughtConsolidation,
  storedConsolidationOf,
} from './consolidate-driver-records.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  reuseTwinOrBuy,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidation driver acquire
// How the consolidation driver comes by one slice's settlement: resumed from
// an earlier run's cache, reused from a twin slice asking the same question in
// this run, or bought. Split out of `consolidate-driver.ts` at the line cap
// when the per-slice log context wrapped its body (ledger A11, 2026-09-27).

/**
 One slice's settlement and how it was come by.

 @example
 ```ts
 const acquired: AcquiredConsolidation = { settlement, exit: 'resumed', };
 ```
 */
export type AcquiredConsolidation = {
  /**
   Settlement the slice ships from.
   */
  readonly settlement: ConsolidationSettlement;

  /**
   How it was come by, as the slice cost line reports it.
   */
  readonly exit: 'computed' | 'resumed' | 'reused';
};

/**
 Resumes, reuses or buys one slice's settlement, in that order.

 @param key - key the settlement resumes under

 @param cache - settlements earlier runs bought

 @param twins - purchases in this run, shared by identical questions

 @param buy - fresh purchase, made only where neither of the others answers

 @param l - driver logger

 @param signal - entry's stop, the one `buy` honours, whose reason the twin
 memo writes no abandoned-buy line for

 @returns Settlement and how it was come by

 @example
 ```ts
 const acquired = await acquireConsolidation({ key, cache, twins, buy, l: dl, signal, },);
 ```
 */
export async function acquireConsolidation(
  {
    key,
    cache,
    twins,
    buy,
    l,
    signal,
  }: {
    readonly key: string;
    readonly cache: SliceCache<ConsolidationSettlement>;
    readonly twins: TwinMemo<ConsolidationSettlement>;
    readonly buy: () => Promise<BoughtConsolidation>;
    readonly l: Logger;
    readonly signal: AbortSignal;
  },
): Promise<AcquiredConsolidation> {
  /**
   A settlement an earlier run already bought for this slice, if any.
   */
  const resumed = cache
    .resumed
    .get(key,);
  if (resumed !== undefined) {
    return {
      settlement: resumed,
      exit: 'resumed',
    };
  }

  /**
   Twin's persisted settlement or this row's fresh purchase.
   */
  const asked = await reuseTwinOrBuy({
    key,
    memo: twins,
    buy,
    persistedOf: storedConsolidationOf,
    l,
    signal,
  },);
  if (asked.kind === 'reused') {
    return {
      settlement: asked.twin,
      exit: 'reused',
    };
  }
  /**
   Fresh settlement unwrapped after memo accounting.
   */
  const { settlement, } = asked.bought;
  return {
    settlement,
    exit: 'computed',
  };
}

//endregion Consolidation driver acquire
