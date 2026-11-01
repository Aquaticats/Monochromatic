/**
 Counters proving that a property's generated inputs reach the branches it
 exists to check.

 A property that never draws an input for a branch passes vacuously there.
 Each widened generator branch therefore names an event, the property body
 records it whenever a run reaches that branch, and after `assert` returns
 the test requires every event to have occurred a minimum number of times.
 A shortfall fails the bounded layer instead of silently narrowing it.

 @module
 */

/**
 Hit counter over a fixed set of event names.
 */
export type ReachTally = {
  /**
   Count one run reaching `event`; repeated hits within one run count once,
   and events outside the tally are ignored, so one observer can feed
   tallies over different subsets.
   */
  readonly hit: (event: string,) => void;
  /**
   Close the current run so its hits count; call once per property run.
   */
  readonly endRun: () => void;
  /**
   Events reached in fewer than `minimum` runs, with their counts.
   */
  readonly shortfalls: (minimum: number,) => readonly string[];
};

/**
 Runs per required hit: every event must occur in at least one run per this
 many, so a campaign round with more runs demands proportionally more hits.
 */
const RUNS_PER_REQUIRED_HIT = 100;

/**
 Smallest acceptable hit count for a property of `numRuns` runs.

 @param numRuns - Runs the property performed.

 @returns At least one, growing with the run count.

 @example
 ```ts
 minimumReach(400); // 4
 ```
 */
export function minimumReach(numRuns: number,): number {
  return Math.max(
    1,
    Math.floor(numRuns / RUNS_PER_REQUIRED_HIT,),
  );
}

/**
 Create a tally over `events`.

 @param events - Every event the property must reach.

 @returns Fresh tally with every count at zero.

 @example
 ```ts
 const tally = reachTally(['intoDropAllKept',] as const);
 tally.hit('intoDropAllKept');
 tally.endRun();
 tally.shortfalls(1); // []
 ```
 */
export function reachTally(events: readonly string[],): ReachTally {
  /**
   Runs that reached each event.
   */
  const counts = new Map<string, number>(events.map(function zero(event,) {
    return [
      event,
      0,
    ] as const;
  },),);
  /**
   Events hit in the run still open.
   */
  const current = new Set<string>();
  return {
    endRun() {
      for (const event of current) {
        counts.set(
          event,
          (counts.get(event,) ?? 0) + 1,
        );
      }
      current.clear();
    },
    hit(event,) {
      if (counts.has(event,))
        current.add(event,);
    },
    shortfalls(minimum,) {
      return [...counts,]
        .filter(function below([, count,],) {
          return count < minimum;
        },)
        .map(function describe([event, count,],) {
          return `${event}: ${String(count,)} of ${String(minimum,)} runs`;
        },);
    },
  };
}
