import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  endsTheRun,
  parksTheRun,
} from './stage-run-end.ts';

//region Overlapped map
// Runs one job per item with a bounded number in flight, and hands the results
// back in item order.
//
// WHY THIS EXISTS. Every per-slice driver in the pipeline ran its slices one
// after another under the rule that aggregate concurrency beyond one stream
// per model collapses throughput. The calibration arms of 2026-08-26 measured
// that rule on four bench slices and found the opposite
// (`doc/decision/translation-repair-calibration-overlap.md`): four slices in
// flight took the same call time in 0.23 of the wall clock that one at a time
// took in 0.41. This is the dial those drivers turn so a corpus pass can be
// measured the same way, and `overlap: 1` is exactly the loop it
// replaces.
//
// WHAT A DRIVER'S LOOP HAD THAT THIS KEEPS. Items START in item order, so a
// twin memoized by an earlier slice is registered before its twin looks for
// it. A failure stops further items from starting; the items already in
// flight finish, or throw under the same abort; and the error thrown is the
// lowest position's, which is the one the sequential loop would have thrown,
// unless a failure ends the run (`endsTheRun`), which is thrown instead, or else one carries a NeuralWatt return the
// pass may park on (`parksTheRun`), which is thrown instead of the lowest position's.
// Results come back in item order whatever order the items finished in, so
// records and findings aggregate the same way at any overlap.
//
// NOT `p-limit`, which the calibration uses. Its queue keeps draining after
// one job fails, so a pass that lost an entry to a fault would go on buying
// every remaining slice of it.
//
// WHAT IS LEFT UNTHROWN IS WRITTEN HERE (ledger B405 left this open under a
// stop, B413 without one). Once the caller has stopped, a lower job often
// ends in the caller's reason while a higher one ends in a failure of its
// own: a hook's refusal, a roster refusal, a ledger gap, a cache write. And
// with no stop, two jobs in flight can each fail of their own. Only the lowest
// position's failure is thrown, so the higher one reached no line. Each
// failure the helper does not throw is written, in position order whichever
// ended first, through `exchangeFailureLogText`, unless it is the stopped
// caller's reason itself, or a lower layer already wrote it and said so
// through `markFailureWritten` (the twin memo does for a failed buy it writes,
// `repairOneCandidate` for a re-ask it writes under a stop, and the seat
// reading for a writing bench it stops the entry over). The rule this follows
// is `mistake-prevention.md`, "Output decided by arrival order".

/**
 Failures a lower layer already wrote on a line of their own, held by identity
 so the failure itself is untouched and nothing is kept alive by being listed.
 */
const writtenFailures = new WeakSet<Error>();

/**
 Records that a failure already stands on a line of its own, so the
 overlapped map, and any outer layer that would write the same failure
 again, leaves it to that line.

 A failure that is no `Error` is not recorded, so the map would write it
 again; the writers that call this write errors their own code caught.

 @param error - caught value just written by the caller of this

 @example
 ```ts
 l.warn(`the buy was abandoned (${exchangeFailureLogText({ error, },)})`,);
 markFailureWritten({ error, },);
 ```
 */
export function markFailureWritten({ error, }: { readonly error: unknown; },): void {
  if (Error.isError(error,))
    writtenFailures.add(error,);
}

/**
 Whether a lower layer already wrote a failure on a line of its own.

 @param error - failure a job, a buy or an attempt ended in

 @returns Whether {@link markFailureWritten} recorded it

 @example
 ```ts
 if (!failureWritten({ error: failure, },)) ml.warn(text,);
 ```
 */
export function failureWritten({ error, }: { readonly error: unknown; },): boolean {
  return Error.isError(error,) && writtenFailures.has(error,);
}

/**
 Refusal for an overlap that cannot bound anything.

 REFUSED HERE AS WELL AS AT THE DIAL. `readOverlap` refuses what an invoker
 mistypes; this refuses what a caller computes, since zero lanes would settle
 a document with no slices in it and report every position missing.
 */
export class OverlapRefusedError extends Error {
  /**
   Declares this message safe to print whole at a boundary: it carries one
   number and nothing from any text.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param overlap - what the caller asked for
   */
  constructor({ overlap, }: { readonly overlap: number; },) {
    super(
      `overlap must be a whole number of at least one, and ${String(overlap,)} is not`,
    );
    this.name = 'OverlapRefusedError';
  }
}

/**
 Refuses an overlap that cannot describe a lane count.

 Exposed to drivers with a no-work branch, so disabled stages cannot make an
 invalid overlap valid merely by returning before {@link mapOverlapped}.

 @param overlap - value requiring validation

 @throws OverlapRefusedError when value is fractional or below one

 @example
 ```ts
 assertOverlap({ overlap: 4, },);
 ```

 @internal
 */
export function assertOverlap(
  { overlap, }: { readonly overlap: number; },
): void {
  if ((!Number.isInteger(overlap,)) || (overlap < 1))
    throw new OverlapRefusedError({ overlap, },);
}

/**
 One item beside where it sits, which every driver needs for its window and
 for the index its records carry.
 */
export type OverlappedRow<Item,> = {
  readonly item: Item;
  readonly position: number;
};

/**
 Runs `oneItem` over every item with at most `overlap` in flight, starting
 them in item order and returning their results in item order.

 @param items - what to run over, in the order results come back

 @param overlap - most items in flight at once; one reproduces a sequential
 loop

 @param oneItem - job for one item, handed the item and its position; its
 result must not be nullish, since a missing result is indistinguishable from
 a job that never ran

 @param signal - caller's stop, whose reason is never written, being the stop
 itself, and which the line for a failure left unthrown names as the ending
 once it has aborted

 @param l - caller's logger, so the line for a failure left unthrown carries
 the caller's tags

 @param nameOf - names an item on that line, since the line is written outside
 the job and no slice context of the job's reaches it

 @returns One result per item, in item order

 @throws OverlapRefusedError when `overlap` is not a whole number of at least
 one

 @throws Whatever the lowest-positioned failing job threw, once every job
 already in flight has finished; no job past it is started. Every other
 failure is first written on a line naming its item, in item order, whether
 or not `signal` has aborted, unless it is the aborted signal's reason or a
 lower layer marked it written (`markFailureWritten`)

 @example
 ```ts
 const records = await mapOverlapped({
   items: prepared.slices,
   overlap: 4,
   oneItem: async function settleOne({ item, position, },) {
     return await settleSlice({ slice: item, slicePosition: position, },);
   },
   signal,
   l,
   nameOf: function sliceName({ item, },) {
     return `slice ${String(item.target.sliceIndex,)}`;
   },
 },);
 ```
 */
export async function mapOverlapped<Item, Result,>(
  {
    items,
    overlap,
    oneItem,
    signal,
    l,
    nameOf,
  }: {
    readonly items: readonly Item[];
    readonly overlap: number;
    readonly oneItem: (row: OverlappedRow<Item>,) => Promise<Result>;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly nameOf: (row: OverlappedRow<Item>,) => string;
  },
): Promise<readonly Result[]> {
  assertOverlap({ overlap, },);

  /**
   Every item beside its position, which is what each job is handed.
   */
  const rows = items.map(function toRow(
    item,
    position,
  ): OverlappedRow<Item> {
    return {
      item,
      position,
    };
  },);

  /**
   What each finished job returned, by position.
   */
  const results = new Map<number, Result>();

  /**
   What each failed job threw, by position.
   */
  const failures = new Map<number, unknown>();

  /**
   Next row nobody has taken yet.
   */
  const cursor = { next: 0, };

  /**
   One lane: takes the next row while there is one and nothing has failed,
   and runs it to the end before taking another.
   */
  async function drain(): Promise<void> {
    while ((cursor.next < rows.length) && (failures.size === 0)) {
      /**
       Row this lane runs now, taken before the first await so no other lane
       takes the same one.
       */
      const row = nonNullishOrThrow(rows[cursor.next],);
      cursor.next += 1;
      try {
        /* oxlint-disable no-await-in-loop -- one lane is sequential by construction; the overlap is the number of lanes, not the number of awaits */
        results.set(
          row.position,
          await oneItem(row,),
        );
        /* oxlint-enable no-await-in-loop */
      }
      catch (error) {
        failures.set(
          row.position,
          error,
        );
      }
    }
  }

  await Promise.all(Array.from(
    { length: Math.min(
      overlap,
      rows.length,
    ), },
    function openLane(): Promise<void> {
      return drain();
    },
  ),);

  if (failures.size > 0) {
    /**
     Position of the first failure in item order, whose error the sequential
     loop would have thrown.
     */
    const earliest = Math.min(...failures.keys(),);
    /**
     The failure to throw in place of the lowest position's, when any qualifies: the first to arrive that ends the run
     (`endsTheRun`, which the every-provider-dry refusal holds), and only when none does the first that carries a
     NeuralWatt return the pass may park on (`parksTheRun`). Searched in that order, so the refusal wins whichever
     arrived first and a bench error that carries a return is not hidden behind an earlier item's ordinary failure.
     */
    const endingEntry = [...failures.entries(),]
      .find(function endsRun([, error,],): boolean {
        return endsTheRun({ error, },);
      },)
      ?? [...failures.entries(),]
        .find(function mayPark([, error,],): boolean {
          return parksTheRun({ error, },);
        },);
    /**
     Position whose failure is thrown: one that ends the run wins over an earlier failure of one item, since no item
     could be served once every provider is dry and the earlier item's own failure is not what ended the work.
     */
    const thrownAt = (endingEntry === undefined) ? earliest : endingEntry[0];
    /**
     The failure thrown, which reaches the caller's own catch.
     */
    const thrown = failures.get(thrownAt,);
    /**
     Logger naming this helper beneath the caller's tags.
     */
    const ml = tagged({
      tag: mapOverlapped.name,
      l,
    },);
    /**
     Why the thrown failure outranks an earlier one: it ends the run, or it names a NeuralWatt return that the run's
     gate may park on, which this map only hands over and does not decide.
     */
    const winsBecause = endsTheRun({ error: thrown, },)
      ? 'ends the run'
      : 'names a NeuralWatt return the run\'s gate may park on';
    /**
     Why the work ended, as each line about an unthrown failure says it: the
     caller's stop, or the failure thrown, named by its item.
     */
    const ending = signal.aborted
      ? 'and the caller stopped the work'
      : `and the failure thrown is ${nameOf(nonNullishOrThrow(rows[thrownAt],),)}'s, which ${
        (thrownAt === earliest) ? 'comes before it' : winsBecause
      }`;
    // Read once every lane has ended, so every failure is written in position
    // order whichever ended first, including one that ended before a stop.
    for (const row of rows) {
      /**
       What this row's job ended in, absent where it finished or never ran.
       */
      const failure = failures.get(row.position,);
      if (failures.has(row.position,)
        && (failure !== thrown)
        // Only a stopped caller has a reason; an unstopped signal's reads as
        // `undefined`, which a job may throw of its own.
        && (!(signal.aborted && (failure === signal.reason)))
        && (!failureWritten({ error: failure, },))) {
        ml.warn(`${nameOf(row,)} failed (${exchangeFailureLogText({ error: failure, },)}), ${ending}`,);
        // The same failure at a later position is the same line.
        markFailureWritten({ error: failure, },);
      }
    }
    throw thrown;
  }

  return rows.map(function toResult(row,): Result {
    return nonNullishOrThrow(results.get(row.position,),);
  },);
}

//endregion Overlapped map
