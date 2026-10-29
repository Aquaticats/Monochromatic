import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  type BudgetView,
  holdReasonOf,
  shortestHold,
  waitOutHold,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RunClient, } from './run-client-contract.ts';
import { judgeSeatsFor, } from './run-seats.ts';
import { ceilingWordsOf, } from './run-seats-ceiling.ts';
import {
  leaveOutFutileReturns,
  sayFutileSkips,
} from './run-seats-futile.ts';
import {
  holdsThatCureShortBenches,
  meterDryOf,
  shortBenchesWithCure,
} from './run-seats-hold-need.ts';
import { seatWithoutWaiting, } from './run-seats-no-wait.ts';
import {
  holdsWithinPass,
  passBoundOf,
} from './run-seats-pass-bound.ts';
import {
  benchesOf,
  PHASES_WITHOUT_WRITER_STOP,
  unreachableWritingBenches,
} from './run-seats-floor.ts';
import {
  neuralwattOf,
  withNeuralWattReturn,
  writtenRefusal,
} from './run-seats-refusal.ts';
import { readMeterStates, } from './run-seats-states.ts';
import {
  type JudgeSeatPhase,
  phaseBenches,
  shortBenches,
} from './run-seats-wait.ts';

//region Short bench wait
// THE WAIT A SEAT READING TAKES WHEN A BENCH A PHASE LEANS ON IS SHORT, split out of `run-seats-read.ts` at its line
// budget: the dryness view read past the holds that explain the shortfall.

/**
 Client surface the readings need: the router's dryness view and holds.

 @example
 ```ts
 const client: SeatReadingClient = createRunClient({ maxHoldMs: 25_200_000, },);
 ```
 */
export type SeatReadingClient = Pick<
  RunClient,
  'futileReturns'
    | 'neuralwattDryness'
    | 'neuralwattHeldByReturn'
    | 'neuralwattReturn'
    | 'passBoundOwner'
    | 'passLeftMs'
    | 'providerDryness'
    | 'providerHolds'
    | 'providerMeterStates'
    | 'writingProviders'
  >;

/**
 What a caller that reads the seats again and again under the same holds remembers of the last no-wait line it let
 through: which providers were held and which benches stood unmet, as one signature, so the same state is said once and
 never by its text (a text carries distances that shrink between readings). The signature is forgotten when a reading
 finds no unmet bench or has waited, so a bench that recovers and falls short again is said again.

 @example
 ```ts
 const warned: WarnedState = { signature: '', };
 ```
 */
export type WarnedState = {
  /**
   The providers held and the benches unmet when the line was last written, none before one was
   written or once it was forgotten.
   */
  signature: string;
};

/**
 The view when the budgets could not be read: nothing is dry.

 @returns Wet

 @example
 ```ts
 const dry = providerRecord({ of: wetWhenUnread, },);
 ```
 */
function wetWhenUnread(): boolean {
  return false;
}

/**
 Reads the dryness view, waiting out the shortest running hold when a
 bench the phase leans on cannot reach quorum among the seats a wet provider
 serves, reading again after the wait, and waiting for the next hold while a
 bench is still short and a hold still runs; when a judge bench is short and
 no provider has named a return the pass can wait for, the reading says so and seats what it
 read; when a writing bench is below its floor with nothing to wait for, or
 still below it after the waits, the entry stops, except in a phase that goes on
 without its writers (`PHASES_WITHOUT_WRITER_STOP`), where the reading says so in
 one error line naming the bench and returns the benches below the floor.

 @throws {@link WritingBenchUnreachableError} when a writing bench the phase
 leans on cannot write a slate and no hold promises it back, in a phase that
 stops for it

 @throws {@link EveryProviderDryError} instead when every provider reads dry, which ends the run
 (`endsTheRun`)

 @param client - run client whose dryness view and holds are the router's own

 @param phase - phase about to start or continue, which names the benches

 @param signal - entry abort the wait honours

 @param l - entry logger, which records the shortfall and the wait

 @param pollMs - how often the wait checks for abort

 @param warned - memory of the no-wait line last written, which the reading reads and updates so the same held
 providers and the same unmet benches are said once, and clears when it finds no unmet bench or has waited; absent for
 a caller that reads once

 @mutates warned - holds the signature of the state this reading found, and forgets it when a reading finds no unmet
 bench or has waited

 @returns Dryness the phase seats on, how long was waited for it, and one clause
 for each writing bench of a phase that goes on without its writers that is below
 its floor

 @example
 ```ts
 const { dry, waitMs, } = await readDrynessPastShortBench({ client, phase, signal, l, pollMs: HOLD_POLL_MS, },);
 ```
 */
export async function readDrynessPastShortBench(
  {
    client,
    phase,
    signal,
    l,
    pollMs,
    warned,
  }: {
    readonly client: SeatReadingClient;
    readonly phase: JudgeSeatPhase;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly pollMs: number;
    readonly warned?: WarnedState;
  },
): Promise<{
  readonly dry: BudgetView;
  readonly waitMs: number;
  readonly unreachableWriters: readonly string[];
}> {
  /**
   Reads which providers are dry, or none when the view could not be read.

   @returns Dryness per provider, holds folded in
   */
  async function readDryness(): Promise<BudgetView> {
    try {
      return await client.providerDryness({ signal, },);
    } catch (error) {
      l.warn(
        `judge seats: the budget view could not be read (${exchangeFailureLogText({ error, },)}); `
          + 'seating the full bench',
      );
      return providerRecord({ of: wetWhenUnread, },);
    }
  }
  /**
   Reads which providers the meters keep dry, only while a hold runs: the meters say whether a hold ending can bring
   its provider back, so a reading with no hold has nothing to ask them. NeuralWatt is left out while its hold is its
   day start, since the day starting again is what turns its meter wet.

   @param running - how long each provider's refusal still holds it out

   @returns Providers dry by their meter whose hold ending brings no return, none when no hold runs
   */
  async function readMeterDry(running: ProviderRecord<number>,): Promise<ReadonlySet<ProviderName>> {
    if (!PROVIDER_ORDER.some(function isHeld(provider,): boolean {
      return running[provider] > 0;
    },))
      return new Set();
    /**
     Providers the meters say are dry.
     */
    const dryByMeter = meterDryOf({
      meters: await readMeterStates({
        client,
        signal,
        l,
      },),
    },);
    if (client.neuralwattHeldByReturn?.({ returnBoundMs: passBoundOf({ client, }), },) ?? true)
      return new Set([...dryByMeter,].filter(function isNotNeuralWatt(provider,): boolean {
        return provider !== 'neuralwatt';
      },),);
    return dryByMeter;
  }
  /**
   Dryness as first read.
   */
  const first = await readDryness();
  /**
   Benches this phase leans on, in the order the line prints them.
   */
  const names = phaseBenches({ phase, },);
  /**
   Benches as first read.
   */
  const benches = benchesOf({ seats: judgeSeatsFor({ dry: first, },), },);
  /**
   How long each provider's refusal still holds it out, as the pass allows it to be waited for, without the holds of
   providers whose last return served nothing, which are said below when a bench is short.
   */
  const {
    held: running,
    skipped: skippedFirst,
  } = leaveOutFutileReturns({
    client,
    running: holdsWithinPass({ client, },),
  },);
  /**
   Providers the meters keep dry, whose holds ending bring no return.
   */
  const meterDry = await readMeterDry(running,);
  /**
   How long each provider's refusal still holds it out, for the providers whose return would lift a bench that stands
   short across its floor or its quorum; a hold on any other provider cannot change such a bench and is not waited for.
   */
  const holds = holdsThatCureShortBenches({
    holds: running,
    dry: first,
    meterDry,
    names,
  },);
  /**
   The first hold to end, zero when no provider that matters is held.
   */
  const shortest = shortestHold({ holds, },);
  /**
   Benches this phase leans on that cannot reach quorum as first read.
   */
  const short = shortBenches({
    benches,
    names,
    dry: first,
  },);
  /**
   Writing benches this phase leans on that cannot write a slate as first read.
   */
  const unreachable = unreachableWritingBenches({
    benches,
    names,
    dry: first,
  },);
  /**
   Whether this phase goes on when a writing bench it leans on is below its floor.
   */
  const goesOn = PHASES_WITHOUT_WRITER_STOP.has(phase,);
  if ((short.length === 0) && (unreachable.length === 0)) {
    // A CLEAN READING ENDS THE STATE the last warning was about: a bench that falls short again is said again.
    if (warned !== undefined)
      warned.signature = '';
    return {
      dry: first,
      waitMs: 0,
      unreachableWriters: [],
    };
  }
  // A BENCH IS SHORT, so a hold left out because its provider's last return served nothing is said before the reading
  // goes on without it or waits for another hold.
  sayFutileSkips({
    skipped: skippedFirst,
    phase,
    l,
  },);
  if (shortest === 0) {
    return seatWithoutWaiting({
      client,
      phase,
      l,
      ...((warned === undefined) ? {} : { warned, }),
      dry: first,
      running,
      holds,
      meterDry,
      names,
      benches,
      unreachable,
      goesOn,
    },);
  }
  /**
   What the waits have read so far: the dryness, the benches short of quorum, and the holds still running.
   */
  const waiting = {
    dry: first,
    short,
    running,
    meterDry,
    holds,
    shortest,
    waitedMs: 0,
  };
  // The loop ends by a break or a throw: every wait is a running hold, and a hold ends.
  for (;;) {
    /**
     Each provider's hold, for the line.
     */
    const held = PROVIDER_ORDER.map(function holdOf(provider,): string {
      return `${provider} ${String(waiting.holds[provider],)}ms`;
    },);
    /**
     Whose hold is the shortest, and for NeuralWatt when its day starts again, for the lines.
     */
    const reason = holdReasonOf({
      holds: waiting.holds,
      shortest: waiting.shortest,
      neuralwatt: neuralwattOf({ client, },),
      neuralwattHeldByReturn: client.neuralwattHeldByReturn?.({ returnBoundMs: passBoundOf({ client, }), },) ?? true,
    },);
    l.warn(
      `JUDGE SEATS phase=${phase} short of quorum: ${
        [
          ...waiting.short,
          ...ceilingWordsOf({
            running: waiting.running,
            dry: waiting.dry,
            meterDry: waiting.meterDry,
            names,
          },),
        ].join('; ',)
      }; holds ${held.join(', ',)}; `
        + `waiting ${
          String(waiting.shortest,)
        }ms for the shortest hold to end, ${reason}, rather than seating a bench short of its quorum`,
    );
    /* oxlint-disable no-await-in-loop -- the loop IS the wait: one hold at a time, then a fresh reading */
    await waitOutHold({
      ms: waiting.shortest,
      signal,
      pollMs,
    },);
    waiting.waitedMs += waiting.shortest;
    l.info(
      `JUDGE SEATS phase=${phase} waited ${String(waiting.shortest,)}ms for ${reason}; the benches are read again`,
    );
    // Dryness after the wait, the holds still running, and what the meters keep dry.
    waiting.dry = await readDryness();
    /**
     The holds running after the wait, without those of providers whose last return served nothing.
     */
    const rerun = leaveOutFutileReturns({
      client,
      running: holdsWithinPass({ client, },),
    },);
    waiting.running = rerun.held;
    waiting.meterDry = await readMeterDry(waiting.running,);
    /* oxlint-enable no-await-in-loop */
    /**
     Benches as read after the wait.
     */
    const benchesAfter = benchesOf({ seats: judgeSeatsFor({ dry: waiting.dry, },), },);
    waiting.short = shortBenches({
      benches: benchesAfter,
      names,
      dry: waiting.dry,
    },);
    /**
     Writing benches still below the floor after the wait.
     */
    const stillBelow = unreachableWritingBenches({
      benches: benchesAfter,
      names,
      dry: waiting.dry,
    },);
    if ((waiting.short
      .length
      === 0) && (stillBelow.length === 0))
      break;
    sayFutileSkips({
      skipped: rerun.skipped,
      phase,
      l,
    },);
    // A HOLD THAT ENDED FIRST IS NOT THE END OF THE WAITING: a bench still short while another hold runs (the
    // NeuralWatt return behind a rate-limit hold) is waited for too, bounded by the entry's abort, by the holds that
    // serve a short bench, and by the pass bound the client carries (a return past it is not a hold).
    waiting.holds = holdsThatCureShortBenches({
      holds: waiting.running,
      dry: waiting.dry,
      meterDry: waiting.meterDry,
      names,
    },);
    waiting.shortest = shortestHold({ holds: waiting.holds, },);
    if (waiting.shortest === 0)
      break;
  }
  // A READING THAT WAITED CHANGED THE SITUATION, whatever it found after: the next one to find a bench short says so.
  if (warned !== undefined)
    warned.signature = '';
  if (waiting.short
    .length
    > 0) {
    // SAID WHEN A BENCH IS SEATED SHORT after the waits, with the providers whose return would have reached its quorum,
    // so a reader of the run can tell a bench that waited for nobody from one nobody could cure.
    l.warn(
      `JUDGE SEATS phase=${phase} seated short after waiting ${String(waiting.waitedMs,)}ms: ${
        shortBenchesWithCure({
          dry: waiting.dry,
          names,
        },)
          .join('; ',)
      }`,
    );
  }
  /**
   Writing benches still below the floor after the waits this reading takes.
   */
  const stillUnreachable = unreachableWritingBenches({
    benches: benchesOf({ seats: judgeSeatsFor({ dry: waiting.dry, },), },),
    names,
    dry: waiting.dry,
  },);
  /**
   The same benches with why NeuralWatt still reads dry.
   */
  const statedAfter = withNeuralWattReturn({
    clauses: stillUnreachable,
    dry: waiting.dry,
    client,
    refusalFollows: !goesOn,
  },);
  if (stillUnreachable.length > 0) {
    l.error(
      `JUDGE SEATS phase=${phase} writing bench unreachable after waiting ${String(waiting.waitedMs,)}ms: `
        + `${statedAfter.join('; ',)}; ${goesOn ? 'the phase goes on without its writers' : 'stopping the entry'}`,
    );
    if (!goesOn) {
      throw writtenRefusal({
        phase,
        clauses: statedAfter,
        dry: waiting.dry,
        client,
      },);
    }
  }
  return {
    dry: waiting.dry,
    waitMs: waiting.waitedMs,
    unreachableWriters: stillUnreachable,
  };
}

//endregion Short bench wait
