import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import {
  EveryProviderDryError,
  HOLD_POLL_MS,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  RUN_SEATS,
  SEAT_WITHHOLD_ASKS,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type JudgeSeats,
  judgeSeatsFor,
} from './run-seats.ts';
import {
  readDrynessPastShortBench,
  type SeatReadingClient,
  type WarnedState,
} from './run-seats-short-bench.ts';
import {
  readMeterStates,
  stateWordOf,
} from './run-seats-states.ts';
import { WritingBenchUnreachableError, } from './run-seats-floor.ts';
import type { JudgeSeatPhase, } from './run-seats-wait.ts';

export type {
  SeatReadingClient,
  WarnedState,
} from './run-seats-short-bench.ts';

//region Seat readings past a named hold
// THE READING EVERY PHASE AND EVERY CHUNK TAKES before it asks anyone.
//
// SPLIT OUT OF `run-seats.ts` when the thirteenth class grew a second face.
// The phase-boundary wait of `752bf9a9b` covered the third hakureico pass's
// case: a hold that began before a phase. The fourth pass met the other case:
// Hyper's daily limit named 923 s two minutes into consolidation, after the
// phase had seated, so eighteen chunks ran on the two Bedrock seats, slice 5's
// slate declined under quorum-not-met with a standing text the block floor
// had refused, and the entry stopped INCOMPLETE. So every phase that buys slice
// by slice (both lanes, the lane contest and the consolidation) now takes
// this reading again before a slice while a hold runs, which waits out the
// shortest hold, and the next while a bench the phase leans on is still short of
// quorum and a hold still runs, and seats the slice on what it read (`pass-reseat-hook.ts`, ledger H5 and X12).
// `awaitBenchQuorum` did the waiting alone until the last of those hooks moved
// onto the shared re-seat on 2026-09-28, and was removed.

/**
 Reads every provider's dryness and derives the benches for one phase of
 one entry, waiting out a named hold first when a bench the phase leans on
 would otherwise sit short of its quorum, and says which writing benches of a
 phase that goes on without its writers are below their floor.

 @param client - run client whose dryness view and holds are the router's own

 @param phase - phase about to start, which the reading seats

 @param signal - entry abort

 @param l - entry logger, which records the reading and the bench

 @param pollMs - how often a wait checks for abort

 @param warned - memory of the no-wait line last written, so a caller that reads again under the same holds says the
 same state once; absent for a caller that reads once

 @returns Benches for this phase, and one clause for each writing bench below the
 floor that the phase goes on without; none for a phase that stops for it

 @throws {@link WritingBenchUnreachableError} when a writing bench the phase
 leans on cannot write a slate, no hold promises it back, and the phase stops for it

 @throws {@link EveryProviderDryError} instead when every provider reads dry, which ends the run
 (`endsTheRun`)

 @example
 ```ts
 const { seats, unreachableWriters, } = await readPhaseSeats({ client, phase: 'page consistency', signal, l, },);
 ```
 */
export async function readPhaseSeats(
  {
    client,
    phase,
    signal,
    l,
    pollMs = HOLD_POLL_MS,
    warned,
  }: {
    readonly client: SeatReadingClient;
    readonly phase: JudgeSeatPhase;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly pollMs?: number;
    readonly warned?: WarnedState;
  },
): Promise<{
  readonly seats: JudgeSeats;
  readonly unreachableWriters: readonly string[];
}> {
  /**
   Dryness this phase seats on, any named hold waited out.
   */
  const {
    dry,
    waitMs,
    unreachableWriters,
  } = await readDrynessPastShortBench({
    client,
    phase,
    signal,
    l,
    pollMs,
    ...((warned === undefined) ? {} : { warned, }),
  },);
  /**
   Benches for this reading.
   */
  const seats = judgeSeatsFor({ dry, },);
  /**
   Each bench, named once for the line.
   */
  const {
    wideSeats,
    selectJudges,
    lateJudges,
    slateJudges,
    checkers,
    translators,
    readers,
    readersSeeing,
    writers,
    roster,
    withheld,
    unserved,
  } = seats;
  /**
   What each meter said in the reading the dryness came from.
   */
  const meters = await readMeterStates({
    client,
    signal,
    l,
  },);
  /**
   Each provider's state, for the line.
   */
  const states = PROVIDER_ORDER.map(function stateOf(provider: ProviderName,): string {
    return `${provider}=${
      stateWordOf({
        routedDry: dry[provider],
        meter: meters[provider],
      },)
    }`;
  },);
  /**
   Counts of the seats the run has withheld for throwing on every ask.
   */
  const withheldCounts = RUN_SEATS.withheld({ minAsks: SEAT_WITHHOLD_ASKS, },);
  /**
   Each withheld seat with the counts that withheld it, as the line names it.
   */
  const threwWithheld = withheldCounts.map(
    function reasonOf(count,): string {
      return `${count.modelId} (threw ${String(count.threw,)} of ${String(count.asked,)} ${
        wordForCount({
          count: count.asked,
          one: 'ask',
          many: 'asks',
        },)
      })`;
    },
  );
  l.info(
    `JUDGE SEATS phase=${phase} ${states.join(' ',)} wide=${String(wideSeats.length,)} `
      + `select=${String(selectJudges.length,)} late=${String(lateJudges.length,)} `
      + `slate=${String(slateJudges.length,)} checkers=${String(checkers.length,)} `
      + `translators=${String(translators.length,)} readers=${String(readers.length,)} seeing=${String(readersSeeing.length,)} `
      + `writers=${String(writers.length,)} roster=${String(roster.length,)} `
      + `withheld=${(withheld.length === 0) ? 'none' : withheld.join(',',)} `
      + `unserved=${(unserved.length === 0) ? 'none' : unserved.join(',',)} `
      + `threw-withheld=${(threwWithheld.length === 0) ? 'none' : threwWithheld.join(',',)} `
      + `waited=${String(waitMs,)}ms`,
  );
  return {
    seats,
    unreachableWriters,
  };
}

/**
 Reads every provider's dryness and derives the benches for one phase of
 one entry, waiting out a named hold first when a bench the phase leans on
 would otherwise sit short of its quorum.

 @param client - run client whose dryness view and holds are the router's own

 @param phase - phase about to start, which the reading seats

 @param signal - entry abort

 @param l - entry logger, which records the reading and the bench

 @param pollMs - how often a wait checks for abort

 @param warned - memory of the no-wait line last written, so a caller that reads again under the same holds says the
 same state once; absent for a caller that reads once

 @returns Benches for this phase

 @throws {@link WritingBenchUnreachableError} when a writing bench the phase
 leans on cannot write a slate and no hold promises it back, in a phase that
 stops for it

 @throws {@link EveryProviderDryError} instead when every provider reads dry, which ends the run
 (`endsTheRun`)

 @example
 ```ts
 const seats = await readJudgeSeats({ client, phase: 'lanes', signal, l, },);
 ```
 */
export async function readJudgeSeats(
  {
    client,
    phase,
    signal,
    l,
    pollMs = HOLD_POLL_MS,
    warned,
  }: {
    readonly client: SeatReadingClient;
    readonly phase: JudgeSeatPhase;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly pollMs?: number;
    readonly warned?: WarnedState;
  },
): Promise<JudgeSeats> {
  return (await readPhaseSeats({
    client,
    phase,
    signal,
    l,
    pollMs,
    ...((warned === undefined) ? {} : { warned, }),
  },))
    .seats;
}

//endregion Seat readings past a named hold
