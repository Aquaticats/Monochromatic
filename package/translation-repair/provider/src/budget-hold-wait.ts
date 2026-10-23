import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  EVERY_PROVIDER_COUNTS,
  everyWritingProviderDry,
  EveryProviderDryError,
  idleWetProviders,
  meterWord,
  type NeuralWattReturn,
  type WritingReach,
} from './budget-routing.ts';
import { holdReasonOf, } from './hold-reason.ts';
import {
  NEURALWATT_RETURN_WAIT_BOUND_MS,
  type NeuralWattDryness,
} from './neuralwatt-return-hold.ts';
import type {
  BudgetView,
  ProviderBudgets,
} from './provider-budget.ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Budget hold wait
// A budget reading that waits out a refusal hold before it calls every
// provider dry.
//
// ALL HELD IS NOT ALL DRY. A hold is process state set by a refusal, and the
// pin pass of 2026-09-02 ended for every remaining entry inside one
// second because two holds were read as two empty meters while both meters
// read wet. When every provider reads dry and at least one is held, the
// shortest hold is waited out and the budgets are read again, and again while
// a hold still runs; only an all-dry reading with no hold left, which no hold
// can explain, ends the run. A refuser held out for no time is not dry at all,
// and the call goes back to it (ledger B124).
//
// SPLIT FROM `provider-router.ts` at its line budget, and along a real seam:
// this is about WHEN to read the budgets, and the router's slot arithmetic must
// stay synchronous with the decision it feeds, which an await in the middle of
// it would break.

/**
 Logger root for the wait.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What the run knows of NeuralWatt's day for a budget view that reports none.
 */
const UNKNOWN_DAY: NeuralWattReturn = { kind: 'unanchored', };

/**
 How often a call waiting out a provider hold checks whether it was aborted.

 THE GRANULARITY OF ABORT, NOT A PACING CHOICE: the wait itself is the hold's
 remaining length, which the budget layer measured, and this only bounds how
 long an aborted call keeps sleeping past its abort.
 */
export const HOLD_POLL_MS = 1_000;

/**
 Reading given when no provider has just refused the call being routed.
 */
export const NOBODY_REFUSED = 'nobody';

/**
 Shortest hold still running across the providers, zero when none is.

 @param holds - milliseconds of hold left per provider

 @returns Milliseconds until the first held provider comes back

 @example
 ```ts
 shortestHold({ holds: { synthetic: 0, hyper: 4_000, openrouter: 0, }, },);
 // => 4000
 ```
 */
export function shortestHold(
  { holds, }: { readonly holds: ProviderRecord<number>; },
): number {
  /**
   Holds that are actually running.
   */
  const running = Object.values(holds,)
    .filter(function isRunning(ms,): boolean {
      return ms > 0;
    },);
  if (running.length === 0)
    return 0;
  return Math.min(...running,);
}

/**
 Sleeps out a hold, waking to check for abort at the poll interval.

 @param ms - hold left to wait out

 @param signal - the call's abort, which ends the wait with its reason

 @param pollMs - how often the abort is checked

 @example
 ```ts
 await waitOutHold({ ms: 4_000, signal, pollMs: HOLD_POLL_MS, },);
 ```
 */
export async function waitOutHold(
  {
    ms,
    signal,
    pollMs,
  }: {
    readonly ms: number;
    readonly signal: AbortSignal;
    readonly pollMs: number;
  },
): Promise<void> {
  for (
    let remaining = ms;
    remaining > 0;
    remaining -= Math.min(
      remaining,
      pollMs,
    )
  ) {
    signal.throwIfAborted();
    // oxlint-disable-next-line no-await-in-loop -- the loop IS the wait, sliced only so an abort is noticed
    await wait(Math.min(
      remaining,
      pollMs,
    ),);
  }
  signal.throwIfAborted();
}

/**
 States what the budgets read and what held them, for the error that ends
 the run.

 @param view - meter reading with holds folded in

 @param refused - provider that had just refused this call, or nobody

 @param holds - hold left per provider

 @returns One clause a reader can tell exhaustion from holds by

 @example
 ```ts
 measuredAt({ view, refused: NOBODY_REFUSED, holds: budgets.holds(), },);
 // => 'meters read synthetic dry, hyper dry, openrouter dry; holds synthetic 0ms, hyper 0ms, openrouter 0ms'
 ```
 */
function measuredAt(
  {
    view,
    refused,
    holds,
  }: {
    readonly view: BudgetView;
    readonly refused: ProviderName | typeof NOBODY_REFUSED;
    readonly holds: ProviderRecord<number>;
  },
): string {
  /**
   Each meter's reading, the refusal that routed here folded into its provider's clause. A provider that reads wet here
   serves no seat that writes, since `readBudgetsPastHolds` calls this only once every provider that does reads dry.
   */
  const meters = PROVIDER_ORDER.map(function clauseOf(provider,): string {
    return `${provider} ${meterWord({
      view,
      provider,
    },)}${(refused === provider) ? ' (just refused this call)' : ''}`;
  },);
  /**
   Each provider's hold.
   */
  const held = PROVIDER_ORDER.map(function holdOf(provider,): string {
    return `${provider} ${String(holds[provider],)}ms`;
  },);
  return `meters read ${meters.join(', ',)}; holds ${held.join(', ',)}`;
}

/**
 What NeuralWatt's dryness adds to the refusal that ends the call: the cause the last reading found, and the distance
 to a spent day's return while it still lies ahead, which the pass reads to park until it.

 @param budgets - shared budget view, whose NeuralWatt cause is read when it reports one

 @param waited - whether the call waited for a hold before this refusal

 @returns Options to spread into the refusal, none for a view that knows nothing of NeuralWatt

 @example
 ```ts
 new EveryProviderDryError({ measured, ...dryCauseOf({ budgets, waited: false, },), },);
 ```
 */
function dryCauseOf(
  {
    budgets,
    waited,
  }: {
    readonly budgets: ProviderBudgets;
    readonly waited: boolean;
  },
): {
  readonly neuralwattDryness?: NeuralWattDryness;
  readonly neuralwattReturnsInMs?: number;
  readonly afterWaiting?: boolean;
} {
  /**
   Why NeuralWatt reads dry in the reading the refusal is made on.
   */
  const dryness = budgets.neuralwattDryness?.();
  if (dryness === undefined)
    return waited ? { afterWaiting: true, } : {};
  return {
    neuralwattDryness: dryness,
    // THE DISTANCE IS NAMED WHENEVER THE RETURN STILL LIES AHEAD, waited or not: a call that waited out another hold
    // has not been through NeuralWatt's return, and dropping the distance there left the pass unable to park for it
    // (R1). A return that has passed names none (the mark reads it as lag), and `waited` only words the message.
    ...(((dryness.kind === 'spent-day') && (dryness.returnsInMs !== undefined)
      && (dryness.returnsInMs > 0))
      ? { neuralwattReturnsInMs: dryness.returnsInMs, }
      : {}),
    ...(waited ? { afterWaiting: true, } : {}),
  };
}

/**
 Waits out the shortest running hold, reads the budgets again, and waits again while every provider still reads dry and
 some hold still runs.

 EVERY HOLD IS WAITED FOR, NOT THE FIRST ALONE. One wait for the shortest hold ended the wait at a hold on a provider
 that still read dry, with a longer hold (NeuralWatt's day starting again) a few minutes behind it, and the run ended
 short of a return it had named. The only bounds are the call's abort, the holds themselves (each of which ends), and a
 NeuralWatt return past what the router's bound leaves.

 @param budgets - shared budget view

 @param modelId - model being routed, for the log lines

 @param signal - call's abort, which ends a wait with its reason

 @param pollMs - how often a wait checks for abort

 @param holds - holds as the caller read them, with at least one running

 @param returnBoundMs - longest the waits of this call take in all, for a NeuralWatt return: every millisecond
 already waited comes off it before the holds are asked again, so a return inside the bound alone but past what is left
 is not waited for

 @param writing - providers whose wet reading keeps the run going

 @returns First reading in which some provider that keeps the run going no longer reads dry

 @throws {@link EveryProviderDryError} when every provider still reads dry and no hold is left to wait out

 @example
 ```ts
 const view = await readAfterHolds({
   budgets, modelId, signal, pollMs: HOLD_POLL_MS, holds: budgets.holds(), returnBoundMs: 300_000, writing,
 },);
 ```
 */
async function readAfterHolds(
  {
    budgets,
    modelId,
    signal,
    pollMs,
    holds,
    returnBoundMs,
    writing,
  }: {
    readonly budgets: ProviderBudgets;
    readonly modelId: RosterModelId;
    readonly signal: AbortSignal;
    readonly pollMs: number;
    readonly holds: ProviderRecord<number>;
    readonly returnBoundMs: number;
    readonly writing: WritingReach;
  },
): Promise<BudgetView> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readAfterHolds.name,
    l,
  },);

  /**
   The holds the next wait is taken from, how long that wait is, the milliseconds waited so far for the refusal that
   ends the run after the waits, and how much of the NeuralWatt return bound those waits left.
   */
  const next = {
    holds,
    shortest: shortestHold({ holds, },),
    waitedMs: 0,
    boundLeftMs: returnBoundMs,
  };
  // The loop ends by a return or a throw: every wait is a running hold, and a hold ends.
  for (;;) {
    /**
     Each provider's hold, for the line.
     */
    const held = PROVIDER_ORDER.map(function holdOf(provider,): string {
      return `${provider} ${String(next.holds[provider],)}ms`;
    },);
    /**
     Whose hold is the shortest, and for NeuralWatt when its day starts again, for the lines.
     */
    const reason = holdReasonOf({
      holds: next.holds,
      shortest: next.shortest,
      neuralwatt: budgets.neuralwattReturn?.() ?? UNKNOWN_DAY,
      neuralwattHeldByReturn: budgets.neuralwattHeldByReturn?.({ returnBoundMs: next.boundLeftMs, }) ?? true,
    },);
    rl.warn(
      `${modelId}: every provider is out for now (${held.join(', ',)}); `
        + `waiting ${String(next.shortest,)}ms for the shortest hold to end, ${reason}, rather than ending the run`,
    );
    /* oxlint-disable no-await-in-loop -- the loop IS the wait: one hold at a time, then a fresh reading */
    await waitOutHold({
      ms: next.shortest,
      signal,
      pollMs,
    },);
    next.waitedMs += next.shortest;
    rl.info(`${modelId}: waited ${String(next.shortest,)}ms for ${reason}; reading the budgets again`,);

    /**
     What the budgets look like once the shortest hold has ended.

     THE ROUTING REFUSAL IS NOT FOLDED IN AGAIN. `refused` says a provider refused THIS call a moment ago; its hold is
     what that refusal became, and a hold that has just expired is the provider coming back. Folding the refusal in a
     second time would send the call to an all-dry error after waiting for exactly the hold that would have cleared it.
     */
    const again = await budgets.read({ signal, },);
    /* oxlint-enable no-await-in-loop */
    if (!everyWritingProviderDry({
      view: again,
      writing,
    },))
      return again;

    // THE BOUND IS A SUM: what the waits so far took comes off it, so a return that lay inside the bound alone but lies
    // past what is left ends the call with its refusal (and the distance) and is not waited for on top of them (R3).
    next.boundLeftMs = Math.max(
      0,
      returnBoundMs - next.waitedMs,
    );
    next.holds = budgets.holds({ returnBoundMs: next.boundLeftMs, },);
    next.shortest = shortestHold({ holds: next.holds, },);
    if (next.shortest === 0) {
      /**
       What the run knows of NeuralWatt's day after this reading, which is where its keeper learns a day start the first
       reading could not give: read here and not carried from before the waits, so a start learned meanwhile is stated
       and the refusal does not say that none can be.
       */
      const neuralwatt = budgets.neuralwattReturn?.();
      throw new EveryProviderDryError({
        measured: `after waiting ${String(next.waitedMs,)}ms, ${
          measuredAt({
            view: again,
            refused: NOBODY_REFUSED,
            holds: next.holds,
          },)
        }`,
        idle: idleWetProviders({
          view: again,
          writing,
        },),
        ...dryCauseOf({
          budgets,
          waited: true,
        },),
        // Conditional spread keeps the day absent on budgets that report none.
        ...(neuralwatt === undefined ? {} : { neuralwatt, }),
      },);
    }
  }
}

/**
 Reads the budgets, waiting out holds while every provider reads dry and a
 hold explains it, and going back to a refuser the budgets gave no hold when it
 is the only provider with budget.

 @param budgets - shared budget view

 @param modelId - model being routed, for the log line

 @param signal - the call's abort

 @param refused - provider that has just refused this call, which counts as
 dry for the first reading unless it is the only provider left and was held
 out for no time, and not after its hold has been waited out; or nobody

 @param pollMs - how often the wait checks for abort

 @param writing - providers whose wet reading keeps the run going, because they serve a model seated in a writing
 bench (`WritingReach`); every provider where the caller knows no roster

 @param returnBoundMs - longest NeuralWatt return this call waits for; the router's own bound by default, since a
 call carries a deadline that cuts anything longer (`NEURALWATT_RETURN_WAIT_BOUND_MS`)

 @returns Every provider's dryness, holds waited out

 @throws {@link EveryProviderDryError} when every provider reads dry with no
 hold left to wait out, at the first reading or after the holds that ran have ended

 @example
 ```ts
 const view = await readBudgetsPastHolds({ budgets, modelId, signal, refused: NOBODY_REFUSED, pollMs: HOLD_POLL_MS, },);
 ```
 */
export async function readBudgetsPastHolds(
  {
    budgets,
    modelId,
    signal,
    refused,
    pollMs,
    writing = EVERY_PROVIDER_COUNTS,
    returnBoundMs = NEURALWATT_RETURN_WAIT_BOUND_MS,
  }: {
    readonly budgets: ProviderBudgets;
    readonly modelId: RosterModelId;
    readonly signal: AbortSignal;
    readonly refused: ProviderName | typeof NOBODY_REFUSED;
    readonly pollMs: number;
    readonly writing?: WritingReach;
    readonly returnBoundMs?: number;
  },
): Promise<BudgetView> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readBudgetsPastHolds.name,
    l,
  },);

  /**
   What each provider's budget looks like right now.
   */
  const first = await budgets.read({ signal, },);
  /**
   The same view with the refusal that routed here folded in.
   */
  const folded = providerRecord({
    of: function dryOf(provider,): boolean {
      return first[provider] || (refused === provider);
    },
  },);
  if (!everyWritingProviderDry({
    view: folded,
    writing,
  },))
    return folded;

  /**
   How long each provider's refusal still holds it out.
   */
  const holds = budgets.holds({ returnBoundMs, },);

  // A REFUSAL THE BUDGETS GAVE NO HOLD HAS ALREADY ENDED (ledger B124).
  // `markRefused` holds a refuser whose meter reads wet only when another
  // provider is wet, so the calls move there; with every other provider dry it
  // holds nothing and leaves the pacing to the transport ladder. Folding the
  // refusal in then read every provider dry with no hold to wait out, and the
  // call ended as if no budget were left anywhere: 166 calls on
  // Uekawakuyuurei, 2026-09-04, while Synthetic's meter read wet. The refuser
  // is the one provider with budget, so the call goes back to it.
  /**
   Whether the provider that just refused reads wet by meter and is held out
   for no time.
   */
  const refuserBack = (refused !== NOBODY_REFUSED)
    && writing[refused]
    && (!first[refused])
    && (holds[refused] === 0);
  if (refuserBack) {
    rl.info(
      `${modelId}: ${refused} refused this call and is held out for no time, every other provider reading `
        + `dry; asking ${refused} again rather than ending the run`,
    );
    return first;
  }

  /**
   The first hold to end, zero when no provider is held.
   */
  const shortest = shortestHold({ holds, },);
  if (shortest === 0) {
    /**
     What the run knows of NeuralWatt's day after the reading this refusal is made on, for the refusal that ends the
     run to state the return time from.
     */
    const neuralwatt = budgets.neuralwattReturn?.();
    throw new EveryProviderDryError({
      measured: measuredAt({
        view: first,
        refused,
        holds,
      },),
      idle: idleWetProviders({
        view: first,
        writing,
      },),
      ...dryCauseOf({
        budgets,
        waited: false,
      },),
      // Conditional spread keeps the day absent on budgets that report none.
      ...(neuralwatt === undefined ? {} : { neuralwatt, }),
    },);
  }

  return await readAfterHolds({
    budgets,
    modelId,
    signal,
    pollMs,
    holds,
    returnBoundMs,
    writing,
  },);
}

//endregion Budget hold wait
