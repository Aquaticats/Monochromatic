import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { RepairSliceSeating, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type {
  TranslateModels,
  TranslateSliceSeating,
} from '@monochromatic-dev/translation-repair-translate-lane/ts';
import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { entryLoggerOf, } from './pass-entry-logger.ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import {
  type JudgeSeats,
  reachableSeats,
  readJudgeSeats,
  type SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Translate lane re-seating
// SPLIT OUT OF `pass-entry.ts` for the file-length cap when the thirteenth
// class added it. The lanes phase seats both lanes at once, and the translate
// lane starts after the repair lane has spent minutes, which on 2026-09-07 was
// long enough for Hyper to be held out by its daily limit: every Hyper-only
// writer was refused in the same millisecond and the footnote passage the
// archive lacks stayed unfilled. So the translate lane asks for its seats
// again when it is about to start, through the same reader every phase uses,
// which waits out a named hold when the bench cannot reach quorum.
//
// THE REPAIR LANE RE-SEATS PER CHUNK WHILE A HOLD RUNS (class one hundred
// three, zheermao7, 2026-09-23). Its benches were read once at the lanes
// boundary, and the checkers are not among the benches the lanes phase waits
// on, so when Synthetic's five-hour window ran out two minutes into the lane
// the two Synthetic-only checker seats stayed unreachable for the rest of it:
// ten of twelve checker rounds and their introduced-defect probes ran on one
// voice, short of quorum, while the substitute a dry reading seats
// (zheermao6, dry from the start: two of three on every round) sat idle. The
// per-chunk hook now reads the seats again while a hold is running, which is
// the signal a dry-out leaves (a refusal from a meter that reads dry holds
// the provider out), hands the driver the roster read, and keeps handing it
// over once the hold has ended, so the chunks after the dry-out never fall
// back to the roster read before it. It still costs nothing while nothing is
// held and the select reach it hands is at the floor or above: one synchronous read of the holds.

/**
 Builds the re-seating the lanes driver calls when the translate lane is
 about to start.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the reading honours

 @param entryId - entry the log line is tagged with

 @returns Reader of the translate lane's roster as of the moment it is called

 @example
 ```ts
 const reseatTranslate = translateReseatFor({ client, signal, entryId: entry.id, },);
 ```
 */
export function translateReseatFor(
  {
    client,
    signal,
    entryId,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly entryId: string;
  },
): () => Promise<TranslateModels> {
  return async function reseat(): Promise<TranslateModels> {
    /**
     Benches as of the translate lane's start.
     */
    const reseated = await readJudgeSeats({
      client,
      phase: 'translate lane',
      signal,
      l: entryLoggerOf({ entryId, },),
    },);
    return reseated.translateModels;
  };
}

/**
 What the repair lane takes from a seat reading: its roster, whose checkers
 the line names, and which of them are padding.

 A CHECKER NO WET PROVIDER SERVES IS PADDING (ledger B711): the bench is
 padded with ranked seats nobody serves only while fewer than its width are
 served, and the stage reads them as unreachable. The zheermao20 chunks were
 re-seated with Qwen3.8-27B as a checker after Synthetic ran dry and nothing
 in the line said it was padding, so the line names them beside the bench.

 @param seats - benches as of this chunk

 @returns Roster the chunk runs on, beside the line naming it

 @example
 ```ts
 const { seating, line, } = repairSeatingOf({ seats, },);
 ```
 */
function repairSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<RepairSliceSeating> {
  /**
   Checkers the chunk runs on, for the line.
   */
  const checkers = seats.repairModels
    .checkerModelIds
    .join(',',);
  /**
   Models of the reading that no wet provider serves.
   */
  const unserved = new Set(seats.unserved,);
  /**
   Checkers on the bench that no wet provider serves, for the line.
   */
  const padding = seats.repairModels
    .checkerModelIds
    .filter(function isUnserved(modelId,): boolean {
      return unserved.has(modelId,);
    },);
  return {
    seating: { repairModels: seats.repairModels, },
    line: `chunk re-seated under a hold: checkers=${checkers} unserved-checkers=${
      (padding.length === 0) ? 'none' : padding.join(',',)
    }`,
  };
}

/**
 Select judges of one reading a wet provider would serve, which the translate lane
 holds a slice that ended unfilled against before it asks the slice again.

 @param seats - benches as of this slice

 @returns Reachable select judges, counted

 @example
 ```ts
 const reachable = selectReachOf({ seats, },);
 ```
 */
export function selectReachOf(
  { seats, }: { readonly seats: JudgeSeats; },
): number {
  return reachableSeats({
    seats: seats.translateModels
      .judgeModelIds,
    dry: seats.dry,
  },)
    .length;
}

/**
 Whether the seating last handed over states a select reach below the floor.

 READ AGAIN EVEN WHEN NO HOLD RUNS, as the lane contest's and the consolidation's hooks do, since a provider whose
 hold ended is wet again and the reach handed over would otherwise stay the one read before it. A passage the archive
 never translated is bought no candidate at a reach under the floor, so a stale reach would keep skipping it after the
 bench had recovered.

 @param seating - seating the hook last handed over

 @returns True when a passage with no archive wording would be bought no candidate on it

 @example
 ```ts
 const stale = selectReachBelowFloor({ selectReachable: 1, },);
 ```
 */
function selectReachBelowFloor(seating: TranslateSliceSeating,): boolean {
  return (seating.selectReachable !== undefined) && (seating.selectReachable < MIN_SELECTION_BALLOTS);
}

/**
 What the translate lane takes from a seat reading: its writers and judges,
 whose writers the line names.

 THE TRANSLATE LANE RE-SEATS TOO (ledger H5, 2026-09-28): it only waited, so a
 dry-out inside the lane left every later slice on the roster read before it,
 the stage classes one hundred three, one hundred nine and one hundred
 thirteen had each fixed for one repair stage at a time.

 @param seats - benches as of this slice

 @returns Roster the slice runs on, beside the line naming it

 @example
 ```ts
 const { seating, line, } = translateSeatingOf({ seats, },);
 ```
 */
function translateSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<TranslateSliceSeating> {
  /**
   Writers the slice runs on, for the line.
   */
  const writers = seats.translateModels
    .translatorModelIds
    .join(',',);
  return {
    seating: {
      translateModels: seats.translateModels,
      selectReachable: selectReachOf({ seats, },),
    },
    line: `slice re-seated under a hold: writers=${writers}`,
  };
}

/**
 Hooks the lanes driver calls: the translate lane's re-seating when it is
 about to start, and the fresh seat reading each slice of either lane takes
 before it starts while a named hold runs, whose roster that lane's next
 slice runs on, kept once read (class one hundred three for the repair lane,
 ledger H5 for the translate lane).

 @example
 ```ts
 const hooks: LanesHooks = lanesHooksFor({ client, signal, entryId, },);
 ```
 */
export type LanesHooks = {
  readonly reseatTranslate: () => Promise<TranslateModels>;
  readonly beforeSlice: (
    args: { readonly lane: 'repair' | 'translate'; },
  ) => Promise<RepairSliceSeating & TranslateSliceSeating>;
};

/**
 Builds the hooks the lanes driver calls.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param entryId - entry the log lines are tagged with

 @param startedSelectReachable - reachable select judges in the reading the
 attempt started on, which a translate slice keeps until a hold makes the hook
 read again; absent where the caller did not count

 @param l - logger the lines go to, the entry's tagged logger when absent

 @returns Re-seating and per-chunk wait for both lanes

 @example
 ```ts
 const hooks = lanesHooksFor({ client, signal, entryId: entry.id, },);
 ```
 */
export function lanesHooksFor(
  {
    client,
    signal,
    entryId,
    startedSelectReachable,
    l = entryLoggerOf({ entryId, },),
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly entryId: string;
    readonly startedSelectReachable?: number;
    readonly l?: Logger;
  },
): LanesHooks {
  /**
   Each lane's own re-seating, so neither hands the other a roster it read
   (`pass-reseat-hook.ts`).
   */
  const reseatLane = {
    repair: reseatHookFor<RepairSliceSeating>({
      client,
      signal,
      phase: 'lanes',
      unseated: {},
      seatingOf: repairSeatingOf,
      l,
    },),
    translate: reseatHookFor<TranslateSliceSeating>({
      client,
      signal,
      phase: 'translate lane',
      unseated: (startedSelectReachable === undefined) ? {} : { selectReachable: startedSelectReachable, },
      staleWhen: selectReachBelowFloor,
      seatingOf: translateSeatingOf,
      l,
    },),
  };
  return {
    reseatTranslate: translateReseatFor({
      client,
      signal,
      entryId,
    },),
    beforeSlice: async function beforeSlice(
      { lane, }: { readonly lane: 'repair' | 'translate'; },
    ): Promise<RepairSliceSeating & TranslateSliceSeating> {
      return await reseatLane[lane]();
    },
  };
}

/**
 Reads the lanes' benches and builds the lanes driver's hooks in one call,
 so the pass entry names both in one place.

 The contest and the consolidation seams read their own benches: XIEPT2 on
 2026-09-03 ran Synthetic dry seven minutes into a 219-minute entry.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param entryId - entry the log lines are tagged with

 @returns Benches for both lanes as of now, and the hooks for when each
 lane and chunk starts

 @example
 ```ts
 const { seats, lanesHooks, } = await readLanesSeats({ client, signal, entryId: entry.id, },);
 ```
 */
export async function readLanesSeats(
  {
    client,
    signal,
    entryId,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly entryId: string;
  },
): Promise<{
  readonly seats: JudgeSeats;
  readonly lanesHooks: LanesHooks;
}> {
  /**
   Benches for both lanes as of now.
   */
  const seats = await readJudgeSeats({
    client,
    phase: 'lanes',
    signal,
    l: entryLoggerOf({ entryId, },),
  },);
  return {
    seats,
    lanesHooks: lanesHooksFor({
      client,
      signal,
      entryId,
      startedSelectReachable: selectReachOf({ seats, },),
    },),
  };
}

//endregion Translate lane re-seating
