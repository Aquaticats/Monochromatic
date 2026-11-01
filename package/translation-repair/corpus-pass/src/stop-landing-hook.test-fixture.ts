import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  backfillWordsOf,
  benchesOf,
  judgeSeatsFor,
  phaseBenches,
  unreachableWritingBenches,
  WritingBenchUnreachableError,
} from '@monochromatic-dev/translation-repair-run';
import type { ProviderRecord, } from '@monochromatic-dev/translation-repair-shared-contract';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';
import type {
  BudgetView,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type Reseated,
  reseatHookFor,
} from '../dist/final/node/index.mjs';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

//region Stop-landing hook
// A PER-SLICE HOOK THAT ENDS THE FIRST TWO SLICES AROUND THE CALLER'S STOP,
// for cases that run a document driver's overlapped lanes and read what is
// written for a slice whose own failure the stop leaves unthrown.
//
// The first call is the first slice's: it waits for the stop and rejects
// with the caller's reason, as a hold wait the caller stops does. The second
// call is the later slice's: it stops the caller, then ends as the case says,
// by a failure of its own or by a seating the driver refuses. So the later
// slice fails of its own first and the first slice ends in the reason after
// it, an order no timer, child or disk decides.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The failures are invented and cat-themed.

/**
 Refusal a hold reading raises when a writing bench cannot write, which is
 what the pass's own per-slice hook throws (`readJudgeSeats`).
 */
export const BENCH_UNREACHABLE: WritingBenchUnreachableError = new WritingBenchUnreachableError({
  phase: 'lanes',
  clauses: ['tabby editors 0 of 3 reachable, floor 2',],
},);

/**
 Builds the hook, its first call waiting for the stop and its second making
 the stop.

 @param steering - caller's controller, which the second call aborts

 @param reason - why the caller stops

 @param afterStop - how the second call ends once the caller has stopped: a
 throw of the slice's own, or a seating the driver then refuses

 @returns Hook a driver awaits before each slice

 @throws Error on a third call, since a driver whose second slice failed
 starts no further slice

 @example
 ```ts
 const beforeSlice = stopLandingHook({ steering, reason, afterStop: throwBenchUnreachable, },);
 ```
 */
export function stopLandingHook<SeatingT,>(
  {
    steering,
    reason,
    afterStop,
  }: {
    readonly steering: AbortController;
    readonly reason: Error;
    readonly afterStop: () => SeatingT;
  },
): () => Promise<SeatingT> {
  /**
   Calls made so far.
   */
  const calls = { count: 0, };
  return async function beforeSlice(): Promise<SeatingT> {
    calls.count += 1;
    if (calls.count === 1) {
      await untilAborted({ signal: steering.signal, },);
      steering.signal
        .throwIfAborted();
      throw new Error('unreachable: the first slice waited for a stop and its signal had not aborted',);
    }
    if (calls.count === 2) {
      steering.abort(reason,);
      return afterStop();
    }
    throw new Error(
      'unreachable: the hook was called a third time, though the second slice failed and no slice starts after it',
    );
  };
}

/**
 Ends the later slice's hook with the bench refusal.

 @throws {@link WritingBenchUnreachableError} always, as {@link BENCH_UNREACHABLE}

 @example
 ```ts
 const beforeSlice = stopLandingHook({ steering, reason, afterStop: throwBenchUnreachable, },);
 ```
 */
export function throwBenchUnreachable(): never {
  throw BENCH_UNREACHABLE;
}

/**
 Refuses a model exchange a case expects none of, with an error saying a
 slice the hook failed asked a model.

 @example
 ```ts
 await refuseUnaskedExchange();
 ```
 */
function refuseUnaskedExchange(): Promise<never> {
  return Promise.reject(new Error('unreachable: a slice the hook failed asked a model',),);
}

/**
 Client for a driver whose slices all end in their hook before asking
 anybody.
 */
export const UNASKED_CLIENT: ModelClient = {
  chatText: refuseUnaskedExchange,
  chatJson: refuseUnaskedExchange,
  quotas: refuseUnaskedExchange,
};

//endregion Stop-landing hook

//region Stop-landing seat reading
// THE PASS'S OWN RE-SEAT (`reseatHookFor`, over `readJudgeSeats`) WITH THE
// STOP LANDING WHILE ITS DRYNESS READ IS IN FLIGHT, for cases that hand a
// module the hook a run hands it and read what is written once the reading
// stops the entry.
//
// The hook's own gate sees a hold, so it reads; by the time the reading asks
// for the holds again the hold has ended, so it waits for nothing. The
// dryness read stops the caller and then answers, as the run's budget view
// does from a reading already taken: every provider dry but DeepSeek, which serves
// one seat of the static editors and refiners and none the role order would add
// (Bedrock alone stood here until the owner's answer of 2026-11-01, "Backfill, and
// add sonnet", which seats both Gemma sizes Bedrock serves). So the reading finds the editors below their floor with
// nothing to wait for, writes its own line, and throws the refusal it marks
// written, while the caller's signal has aborted.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The holds and the view are invented.

/**
 Dryness the reading answers with: DeepSeek alone wet.
 */
const DEEPSEEK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Holds as the hook's gate reads them: Hyper held out a minute.
 */
const HYPER_HELD: ProviderRecord<number> = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  hyper: 60_000,
  openrouter: 0,
};

/**
 Holds as the reading reads them, the minute having passed.
 */
const NONE_HELD: ProviderRecord<number> = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  hyper: 0,
  openrouter: 0,
};

/**
 Line the reading writes as it stops the entry under one view.

 @param dry - view the lanes are read under

 @returns Line: every writing bench of the lanes under its floor, then the seats the role order gave

 @example
 ```ts
 const line = stopLineOf({ dry: DEEPSEEK_ALONE, },);
 ```
 */
function stopLineOf(
  { dry, }: { readonly dry: BudgetView; },
): string {
  /**
   Seats the view derives.
   */
  const seats = judgeSeatsFor({ dry, },);
  /**
   The clauses of the stop line, in the order the run states them.
   */
  const clauses = unreachableClausesOf({ dry, },);
  return `JUDGE SEATS phase=lanes writing bench unreachable: ${clauses.join('; ',)}; `
    + `backfilled by role order: ${backfillWordsOf({ backfilled: seats.backfilled, },)}; stopping the entry`;
}

/**
 The writing benches of the lanes under their floor in a view, each worded as the stop line words it.

 @param dry - view the lanes are read under

 @returns One clause for each bench of the lanes under its floor

 @example
 ```ts
 const clauses = unreachableClausesOf({ dry: DEEPSEEK_ALONE, },);
 ```
 */
function unreachableClausesOf(
  { dry, }: { readonly dry: BudgetView; },
): readonly string[] {
  return unreachableWritingBenches({
    benches: benchesOf({ seats: judgeSeatsFor({ dry, },), },),
    names: phaseBenches({ phase: 'lanes', },),
    dry,
  },);
}

/**
 What the stop line over {@link DEEPSEEK_ALONE} says is unreachable: each writing bench of the lanes under its floor,
 as a clause of the line, so a case that looks for the stop line looks for the benches it names and not for its first
 words alone.
 */
export const DEEPSEEK_ALONE_UNREACHABLE: readonly string[] = unreachableClausesOf({ dry: DEEPSEEK_ALONE, },);

/**
 Line the reading writes as it stops the entry over {@link DEEPSEEK_ALONE}: every writing bench of the lanes under its
 floor, as the benches the run derives from that view read, then the seats the role order gave.
 */
export const DEEPSEEK_ALONE_STOP_LINE: string = stopLineOf({ dry: DEEPSEEK_ALONE, },);

/**
 Refuses to seat anything, since the reading this fixture scripts stops the
 entry before any seating is taken from it.

 @returns Never

 @throws Error on any call

 @example
 ```ts
 const hook = reseatHookFor({ client, signal, phase: 'lanes', unseated: {}, seatingOf: refuseSeating, l, },);
 ```
 */
function refuseSeating<SeatingT,>(): Reseated<SeatingT> {
  throw new Error('unreachable: a reading that stopped the entry was asked for a seating',);
}

/**
 Builds the pass's re-seat for the repair lane over a client whose dryness
 read stops the caller and then reports the editors unreachable.

 @param steering - caller's controller, which the dryness read aborts

 @param reason - why the caller stops

 @param unseated - seating the hook would hand on before any hold

 @param l - logger the reading writes its line to

 @returns Hook a module calls as its re-seat; its first call throws the
 reading's marked refusal with the caller stopped

 @example
 ```ts
 const reseat = stopLandingSeatReading<RepairSliceSeating>({ steering, reason, unseated: {}, l, },);
 ```
 */
export function stopLandingSeatReading<SeatingT,>(
  {
    steering,
    reason,
    unseated,
    l,
  }: {
    readonly steering: AbortController;
    readonly reason: Error;
    readonly unseated: SeatingT;
    readonly l: Logger;
  },
): () => Promise<SeatingT> {
  /**
   Holds read so far.
   */
  const holdReads = { count: 0, };
  return reseatHookFor<SeatingT>({
    client: {
      providerMeterStates: metersAllWet,
      providerHolds: function holdsAsRead(): ProviderRecord<number> {
        holdReads.count += 1;
        return (holdReads.count === 1) ? HYPER_HELD : NONE_HELD;
      },
      providerDryness: function drynessAsTheStopLands(): Promise<BudgetView> {
        steering.abort(reason,);
        return Promise.resolve(DEEPSEEK_ALONE,);
      },
    },
    signal: steering.signal,
    phase: 'lanes',
    unseated,
    seatingOf: refuseSeating,
    l,
  },);
}

//endregion Stop-landing seat reading
