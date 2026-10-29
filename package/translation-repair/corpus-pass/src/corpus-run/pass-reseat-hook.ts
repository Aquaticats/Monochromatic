import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { shortestHold, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type JudgeSeatPhase,
  type JudgeSeats,
  readJudgeSeats,
  type SeatReadingClient,
  type WarnedState,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Per-slice re-seating under a hold
// ONE SHAPE FOR EVERY PHASE THAT RE-SEATS ITS SLICES (ledger X12). The repair
// lane's chunks (class one hundred three), the translate lane's slices and the
// consolidation's (ledger H5) and the lane contest's (ledger X12) each read
// their seats again before a slice while a named hold runs, which is the
// signal a dry-out leaves, hand the slice the roster read, and keep handing it
// over once the hold has ended, so the slices after a dry-out never fall back
// to the roster read before it. Three copies of that memo had grown before
// this one; a fourth phase takes this instead of a fifth copy.
//
// IT COSTS NOTHING WHILE NOTHING IS HELD AND THE REACH IS AT OR ABOVE THE FLOOR: one synchronous read of the holds,
// no dryness read. A phase that passes `staleWhen` (the lane contest, the translate lane and the consolidation) also
// reads again with no hold running while the seating it last handed over states a reach under the floor, since a
// provider returning after its hold ended would lift it; that read costs a cached budget reading, trusted for 60
// seconds (`BUDGET_FRESH_MS` in `provider-budget.ts`), and the reading's own `JUDGE SEATS` line, for every slice the
// shortfall lasts. Under a hold the reading itself waits out the shortest hold once when its return would lift a bench
// the phase leans on over its floor or to its quorum (`run-seats-read.ts`).
//
// A WARNING IS WRITTEN ONCE PER STATE. A hold whose return would change no bench is not waited for, so
// every slice under it reads the same short bench and the reading warns of it each time; a Xiaomi or DeepSeek hold of
// hours over hundreds of slices wrote hundreds of identical lines. The hook still reads the seats for every slice,
// since a provider going dry changes the seating a slice must run on whether or not its hold would cure anything. It
// cannot skip the read on "no running hold would cure": that is also the state of a hold with no short bench at all,
// which is the state the seating is for.
//
// THE MEMORY IS WHAT THE WARNING IS ABOUT, NEVER ITS TEXT. The reading keeps one signature, the providers held and the
// benches unmet (`WarnedState` in `run-seats-short-bench.ts`), and writes its no-wait line only when that changes. The
// text carries the time each skipped hold has left and, with NeuralWatt dry, the distance to its return and the
// figure past what the pass has left, all of which shrink between slices, so a memory of text writes a line per
// slice. The signature is forgotten when no hold runs (the hook does it), when a reading finds no unmet bench, and when
// a reading has waited (the reading does it), so a hold that comes back, or a bench that recovers and falls short
// again, is said again.

/**
 What a phase takes from one seat reading: the seating its slices run on, and
 the line naming it.

 @example
 ```ts
 const reseated: Reseated<BenchSeating> = { seating: { modelIds: seats.lateJudges, }, line: 'slice re-seated under a hold: judges=a,b', };
 ```
 */
export type Reseated<SeatingT,> = {
  readonly seating: SeatingT;
  readonly line: string;
};

/**
 Builds the per-slice re-seating one phase's driver calls before each slice.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param phase - phase whose benches the reading waits on

 @param unseated - seating a slice takes before any hold has run, which keeps
 the roster the phase started on

 @param seatingOf - what this phase takes from a seat reading

 @param staleWhen - whether the seating last handed over must be read again
 although no hold runs, for a phase whose seating states a shortfall that a
 provider returning after its hold ended would cure; absent keeps the memo
 until a hold runs. Such a read costs a cached budget reading (trusted for
 sixty seconds, `BUDGET_FRESH_MS` in `provider-budget.ts`) and the reading's
 own `JUDGE SEATS` line, so it holds the file header's no-read guarantee only
 while the reach is at or above the floor

 @param l - entry logger, which records every re-seating

 @returns Per-slice reader of the seating the slice runs on

 @example
 ```ts
 const beforeSlice = reseatHookFor({ client, signal, phase: 'lane contest', unseated: {}, seatingOf, l, },);
 ```
 */
export function reseatHookFor<SeatingT,>(
  {
    client,
    signal,
    phase,
    unseated,
    seatingOf,
    staleWhen,
    l,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly phase: JudgeSeatPhase;
    readonly unseated: SeatingT;
    readonly seatingOf: (args: { readonly seats: JudgeSeats; },) => Reseated<SeatingT>;
    readonly staleWhen?: (seating: SeatingT,) => boolean;
    readonly l: Logger;
  },
): () => Promise<SeatingT> {
  /**
   Seating as last read under a hold, handed to every slice after it.
   */
  const latest: { seating: SeatingT; } = { seating: unseated, };
  /**
   The state the last no-wait line was about, which the reading updates; empty when no hold has run since.
   */
  const warned: WarnedState = { signature: '', };
  return async function reseatBeforeSlice(): Promise<SeatingT> {
    /**
     Whether a hold runs, which is what the memo of the last no-wait line is forgotten on.
     */
    const held = shortestHold({ holds: client.providerHolds(), },) !== 0;
    if ((!held) && ((staleWhen === undefined) || (!staleWhen(latest.seating,)))) {
      warned.signature = '';
      return latest.seating;
    }
    /**
     Benches as of this slice, the reading waiting out a named hold when a
     bench the phase leans on cannot reach quorum.
     */
    const seats = await readJudgeSeats({
      client,
      phase,
      signal,
      l,
      warned,
    },);
    /**
     What this phase takes from that reading.
     */
    const reseated = seatingOf({ seats, },);
    latest.seating = reseated.seating;
    // A READING TAKEN WITH NO HOLD RUNNING says nothing of a hold: the reading wrote its own JUDGE SEATS line, and
    // this one would name a hold that is not there, once for every slice the shortfall lasts.
    if (held)
      l.info(`JUDGE SEATS phase=${phase} ${reseated.line}`,);
    return latest.seating;
  };
}

//endregion Per-slice re-seating under a hold
