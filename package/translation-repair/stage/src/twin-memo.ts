import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  failureWritten,
  markFailureWritten,
} from './overlapped-map.ts';

//region Twin memo
// In-run memoization of slices asking the same question, safe under overlap.
//
// WHAT THE SEQUENTIAL DRIVERS HAD. Two slices carrying identical text ask one
// question, and since their keys said so each driver memoized what it
// PERSISTED so the second twin reused it: a cold run then settled what a warm
// run resumes. The memo held persisted records only, on purpose: a record the
// driver refused to store (nobody heard, an unfilled passage) was not
// memoized, so the twin asked again, exactly as a warm run would.
//
// WHAT OVERLAP ADDS. The twin may arrive while the first is still buying. It
// must neither buy the same question twice nor reuse what will not be stored,
// so the memo holds a PROMISE of the persisted record: a twin waits for it,
// reuses the record if one was stored, and buys its own otherwise. The buyer
// withdraws its entry before resolving to nothing, so a third twin that wakes
// finds either the second's entry to wait on or no entry, and registers.
//
// REGISTERED WITHOUT AN AWAIT BETWEEN LOOKING AND SETTING. Two twins that
// both looked, both found nothing, and both yielded before registering would
// both buy; the lookup loop exits, the buy starts and the entry is set in one
// synchronous run.

/**
 What a buy left behind for its twins: the record it persisted, or nothing.

 TAGGED RATHER THAN NULLISH, which this repository requires of every absence
 it models. It also says the thing plainly: a buy that stored nothing is a
 fact about the buy, not a missing value.
 */
export type TwinStored<Settled,> = {
  readonly kind: 'stored';

  /**
   Record a warm run would resume for this key.
   */
  readonly record: Settled;
} | {
  readonly kind: 'nothing';
};

/**
 Per-key promise of what a buy left for its twins.
 */
export type TwinMemo<Settled,> = Map<string, Promise<TwinStored<Settled>>>;

/**
 What came of asking under the memo: a twin's stored record, or one's own
 purchase.
 */
export type TwinOrBought<Settled, Bought,> = {
  readonly kind: 'reused';

  /**
   Record a twin persisted for this key.
   */
  readonly twin: Settled;
} | {
  readonly kind: 'bought';

  /**
   What this slice's own buy returned, stored or not.
   */
  readonly bought: Bought;
};

/**
 Reuses what a twin persisted for this key, or buys and registers the buy so
 twins arriving meanwhile wait for it.

 @param key - question this slice asks, shared by its twins

 @param memo - promises of persisted records, one per key being bought

 @param buy - starts this slice's own purchase; called at most once, and only
 when no twin persisted a record for the key

 @param persistedOf - reads the persisted record off a purchase, answering
 `nothing` when the purchase was deliberately not stored

 @param l - logger for a purchase that failed while twins waited on it

 @param signal - caller's stop, the one `buy` honours, whose reason a failed
 purchase may end in and which is no failure of the purchase, so it gets no
 line

 @returns Twin's record, or the purchase

 @throws Whatever `buy` throws; a waiting twin sees nothing stored and buys
 its own, or throws under the same abort. The failure is written once on an
 abandoned-buy line unless it is the stopped caller's reason or a lower layer
 already wrote it (`markFailureWritten`)

 @example
 ```ts
 const asked = await reuseTwinOrBuy({
   key,
   memo: twins,
   buy: async function buyThisSlice() {
     return await attemptAndPersist();
   },
   persistedOf: function stored(bought,) {
     return bought.persisted ? { kind: 'stored', record: bought.record, } : { kind: 'nothing', };
   },
   l,
   signal,
 },);
 ```
 */
export async function reuseTwinOrBuy<Settled, Bought,>(
  {
    key,
    memo,
    buy,
    persistedOf,
    l,
    signal,
  }: {
    readonly key: string;
    readonly memo: TwinMemo<Settled>;
    readonly buy: () => Promise<Bought>;
    readonly persistedOf: (bought: Bought,) => TwinStored<Settled>;
    readonly l: Logger;
    readonly signal: AbortSignal;
  },
): Promise<TwinOrBought<Settled, Bought>> {
  for (
    let pending = memo.get(key,);
    pending !== undefined;
    pending = memo.get(key,)
  ) {
    /* oxlint-disable no-await-in-loop -- each wait is for a different twin's buy; the loop ends when the key has no buyer or when one persisted */
    /**
     What the twin buying this key left behind.
     */
    const twin = await pending;
    /* oxlint-enable no-await-in-loop */
    if (twin.kind === 'stored') {
      return {
        kind: 'reused',
        twin: twin.record,
      };
    }
  }

  // SYNCHRONOUS from the empty lookup to the registration, per the module
  // note: the buy runs to its first await and the entry is set before any
  // other slice can look.

  /**
   This slice's own purchase.
   */
  const bought = buy();
  memo.set(
    key,
    (async function untilStored(): Promise<TwinStored<Settled>> {
      try {
        /**
         What the purchase left for its twins.
         */
        const stored = persistedOf(await bought,);
        if (stored.kind === 'nothing')
          memo.delete(key,);
        return stored;
      }
      catch (error) {
        // WITHDRAWN, NOT PROPAGATED: the buyer throws this error on its own
        // path, and a twin waiting here must go and ask for itself, or throw
        // under the same abort when it tries.
        memo.delete(key,);
        // THE CALLER'S OWN STOP IS NO ABANDONED BUY (ledger B411): written as
        // one, every translate and repair buy the stop ended read as a buy
        // that failed. The withdrawal stands, so a waiting twin still asks for
        // itself, and its own buy meets the same stop.
        if (signal.aborted && (error === signal.reason))
          return { kind: 'nothing', };
        // A FAILURE A LOWER LAYER ALREADY WROTE keeps that line alone, such as
        // a repair re-ask `repairOneCandidate` wrote with its seat as the
        // caller stopped the consolidation (ledger B412). This line would add
        // the failure's words a second time; what it adds besides, that a
        // waiting twin asks for itself, holds for every withdrawn key and is
        // this function's documented behaviour.
        if (failureWritten({ error, },))
          return { kind: 'nothing', };
        l.warn(
          `twin memo: the buy for a shared key was abandoned (${
            exchangeFailureLogText({ error, },)
          }), so any twin waiting on it asks for itself`,
        );
        // THIS IS THE FAILURE'S LINE. The buyer throws the same failure on its
        // own path, and the overlapped map would write it a second time where
        // it goes unthrown (ledger B405).
        markFailureWritten({ error, },);
        return { kind: 'nothing', };
      }
    })(),
  );
  return {
    kind: 'bought',
    bought: await bought,
  };
}

//endregion Twin memo
