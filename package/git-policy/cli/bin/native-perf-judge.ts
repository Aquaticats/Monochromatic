/**
 What:
  The two latency requirements `native-perf.ts` enforces, and the verdict on a finished report.
 Why:
  `SPEC.md` ("Forbidden strings") requires that the added latency of a forwarded command stays strictly under two
  seconds, and the owner's measure for commits (2026-11-01) is that what a commit through the wrapper adds over real
  Git's own time for that same commit stays under the larger of five seconds and twice that time (changed on
  2026-11-12: "Change the 5s rule to max(5s, double what native git takes)", read as the extra over real Git). Keeping
  the judging pure lets a unit test feed it a recorded or edited sample set, which the benchmark itself cannot do
  without making the wrapper slower.

 In TS you'd write (pseudocode):
 ```ts
 const violations = judge(report).filter((violation) => violation.value >= violation.limit);
 ```
 */
import type {
  ScenarioKind,
  ScenarioReport,
} from './native-perf-measure.ts';

/**
 The strict upper bound for wrapper-added latency of a forwarded command, in milliseconds.
 */
export const FORWARDED_BOUND_MILLISECONDS = 2_000;

/**
 The smallest bound on what a commit through the wrapper may add over real Git's own time for that same commit, in
 milliseconds: the five seconds of the owner's measure of 2026-11-01 ("commits vs native git should not take 5s
 longer"), which the change of 2026-11-12 keeps wherever real Git is fast.
 */
export const COMMIT_DELTA_BOUND_MILLISECONDS = 5_000;

/**
 The bound one commit round must stay under: what the wrapper may add over real Git's own time for that same commit,
 which is the larger of five seconds and twice that time.

 @param directMilliseconds - real Git's own time for the commit of this round

 @returns the milliseconds the wrapper may add in this round

 @example
 ```ts
 commitDeltaBound({ directMilliseconds: 50 }); // 5_000
 ```
 */
export function commitDeltaBound({
  directMilliseconds,
}: {
  readonly directMilliseconds: number;
}): number {
  return Math.max(
    COMMIT_DELTA_BOUND_MILLISECONDS,
    directMilliseconds * 2,
  );
}

/**
 One requirement a scenario failed: which scenario, which statistic reached its bound, and by how much.
 */
export type BoundViolation = {
  readonly scenario: string;
  readonly statistic: string;
  readonly value: number;
  readonly limit: number;
};

/**
 What a scenario's report is judged by: which statistic of its report, and the bound that statistic must stay under.
 A commit is judged elsewhere, because its bound belongs to each round rather than to the scenario.
 */
const JUDGED: Readonly<Record<Exclude<ScenarioKind, 'commit'>, (entry: ScenarioReport) => Omit<BoundViolation, 'scenario'>>> = {
  forwarded(entry) {
    return {
      statistic: 'added p95',
      value: entry.addedP95Milliseconds,
      limit: FORWARDED_BOUND_MILLISECONDS,
    };
  },
};

/**
 The commit round that breaks the measure furthest, as the violation to print, or nothing when every round holds.

 @param entry - a commit scenario's report, whose paired rounds carry both sides' times

 @returns the round whose delta stands furthest past the bound its own direct time gives

 @example
 ```ts
 worstCommitRound(entry);
 ```
 */
function commitRoundsOverTheirBound(entry: ScenarioReport): readonly Omit<BoundViolation, 'scenario'>[] {
  /**
   Every round that reached the bound its own direct time gives, furthest past it first.
   A report whose two arrays somehow differ in length reads a missing direct time as zero, which is the strictest bound
   this rule can give and never lets a round pass on a figure that is not there.
   */
  const broken = entry.rawPairedDeltas
    .flatMap(function aboveItsBound(
      delta,
      index,
    ): readonly {
      readonly over: number;
      readonly value: number;
      readonly limit: number;
    }[] {
      /**
       Real Git's own time for this round.
       */
      const direct: number = entry.rawPairedDirectMilliseconds[index] ?? 0;
      /**
       The bound this round's own direct time gives it.
       */
      const limit: number = commitDeltaBound({
        directMilliseconds: direct,
      });
      return delta < limit
        ? []
        : [
          {
            over: delta - limit,
            value: delta,
            limit,
          },
        ];
    })
    .toSorted(function furthestFirst(
      left,
      right,
    ) {
      return right.over - left.over;
    });
  /**
   The round that stands furthest past its own bound, absent when every round read holds.
   */
  const [worst] = broken;
  return worst === undefined
    ? []
    : [
      {
        statistic: 'paired commit delta',
        value: worst.value,
        limit: worst.limit,
      },
    ];
}

/**
 Requirements that a finished report breaks. A forwarded command is judged by the difference of the p95 of its wrapped
 and direct samples, which is what the forwarded bound has always used; a commit is judged round by round, because the
 owner's measure compares every commit with its own native twin and the bound of a round is twice that twin's time once
 that is larger than five seconds: one round decides, not the 95th percentile.

 @param report - every scenario that had a successful sample on both sides

 @returns the scenarios whose judged statistic reached its bound, in report order; empty when every requirement holds

 @example
 ```ts
 judge(report).map((violation) => violation.scenario);
 ```
 */
export function judge(report: readonly ScenarioReport[]): readonly BoundViolation[] {
  return report.flatMap(function violationsOf(entry): readonly BoundViolation[] {
    /**
     What this scenario is judged by: a commit by the rounds its own direct times are too short for, a forwarded
     command by the one statistic its kind is judged on.
     */
    const judged: readonly Omit<BoundViolation, 'scenario'>[] = entry.kind === 'commit'
      ? commitRoundsOverTheirBound(entry)
      : [JUDGED.forwarded(entry)];
    return judged
      .filter(function reachedItsBound(violation) {
        return violation.value >= violation.limit;
      })
      .map(function named(violation) {
        return {
          scenario: entry.scenario,
          ...violation,
        };
      });
  });
}
