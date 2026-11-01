import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  LANE_CONTEST_QUORUM,
  type LaneContestOutcome,
} from './lane-contest-stage.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TwinStored, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Lane contest persistence
// Which bought contest outcomes may be written down for a later run to resume,
// and the record a twin may reuse. Split out of `lane-contest-driver.ts` at the
// line cap (ledger L6): the driver decides what to ask, this module decides what
// is worth keeping once it has been asked.

/**
 Whether a bought outcome is worth keeping across runs.

 SETTLED VERDICTS ONLY. An unheard roster is a transient fact about a provider
 on one night, not a property of the question, and writing it to the cache
 would freeze that night into every later resume of the entry. The ballots
 that did arrive are discarded with it, which is the point: half a panel is
 not an answer, and re-asking is what the quorum is for.

 @param outcome - what the roster settled

 @param choiceMayShip - whether selected lane passes publication invariants

 @returns Whether to persist it

 @example
 ```ts
 const keep = worthResuming({ outcome, },);
 ```
 */
function worthResuming(
  {
    outcome,
    choiceMayShip = true,
  }: {
    readonly outcome: LaneContestOutcome;
    readonly choiceMayShip?: boolean;
  },
): boolean {
  return choiceMayShip && (outcome.usable >= LANE_CONTEST_QUORUM);
}

/**
 Fresh contest outcome beside whether it became warm-run evidence.

 @example
 ```ts
 const bought: BoughtLaneContest = { outcome, persisted: true, };
 ```
 */
export type BoughtLaneContest = {
  readonly outcome: LaneContestOutcome;
  readonly persisted: boolean;
};

/**
 Reads cache-eligible record from fresh contest purchase.

 @param bought - fresh result beside persistence status

 @returns Record a twin may reuse, or deliberate nothing

 @example
 ```ts
 const stored = storedContestOf({ outcome, persisted: true, },);
 ```
 */
export function storedContestOf(
  bought: BoughtLaneContest,
): TwinStored<LaneContestOutcome> {
  return bought.persisted
    ? {
      kind: 'stored',
      record: bought.outcome,
    }
    : { kind: 'nothing', };
}

/**
 Persists a bought contest only when caller remains live and quorum made its
 ballots reusable.

 Kept as a testable boundary because gather rounds normally surface an abort
 before returning, making the final pre-write defense unreachable in a
 transport fixture.

 @param key - exact contest question this outcome answers

 @param outcome - bought ballots and their settled choice

 @param cache - contest persistence boundary

 @param choiceMayShip - whether selected lane passes publication invariants

 @param signal - caller abort checked before write

 @returns Whether outcome was persisted and may be reused by a twin

 @throws Whatever caller abort reason or persistence throws

 @example
 ```ts
 await persistLaneContestOutcome({ key, outcome, cache, signal, },);
 ```

 @internal
 */
export async function persistLaneContestOutcome(
  {
    key,
    outcome,
    cache,
    choiceMayShip = true,
    signal,
  }: ForeignBorrowed<{
    readonly key: string;
    readonly outcome: LaneContestOutcome;
    readonly cache: SliceCache<LaneContestOutcome>;
    readonly choiceMayShip?: boolean;
    readonly signal: AbortSignal;
  }>,
): Promise<boolean> {
  signal.throwIfAborted();
  if (!worthResuming({
    outcome,
    choiceMayShip,
  },))
    return false;
  await cache.persist({
    key,
    serialized: JSON.stringify(outcome,),
  },);
  return true;
}

//endregion Lane contest persistence
