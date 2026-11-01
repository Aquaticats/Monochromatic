/**
 What:
  The two latency requirements `native-perf.ts` enforces, and the verdict on a finished report.
 Why:
  `SPEC.md` ("Forbidden strings") requires that the added latency of a forwarded command stays strictly under two
  seconds, and the owner's measure for commits (2026-11-01) is that a commit through the wrapper must not take five
  seconds or more longer than the same commit with native Git. Keeping the judging pure lets a unit test feed it a
  recorded or edited sample set, which the benchmark itself cannot do without making the wrapper slower.

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
 The strict upper bound for how much longer a commit through the wrapper may take than the same commit with native Git,
 in milliseconds; the owner's measure of 2026-11-01 ("commits vs native git should not take 5s longer").
 */
export const COMMIT_DELTA_BOUND_MILLISECONDS = 5_000;

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
 What a scenario is judged by: which statistic of its report, and the bound that statistic must stay under.
 */
const JUDGED: Readonly<Record<ScenarioKind, (entry: ScenarioReport) => Omit<BoundViolation, 'scenario'>>> = {
  forwarded(entry) {
    return {
      statistic: 'added p95',
      value: entry.addedP95Milliseconds,
      limit: FORWARDED_BOUND_MILLISECONDS,
    };
  },
  commit(entry) {
    return {
      statistic: 'paired commit delta p95',
      value: entry.pairedDelta
        .p95,
      limit: COMMIT_DELTA_BOUND_MILLISECONDS,
    };
  },
};

/**
 Requirements that a finished report breaks. A forwarded command is judged by the difference of the p95 of its wrapped
 and direct samples, which is what the forwarded bound has always used; a commit is judged by the p95 of its paired
 deltas (each wrapped sample minus the direct sample of the same round), because the owner's measure compares one
 commit with its native twin.

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
     The statistic and bound this scenario's kind is judged by.
     */
    const judged = JUDGED[entry.kind](entry);
    return judged.value >= judged.limit
      ? [
        {
          scenario: entry.scenario,
          ...judged,
        },
      ]
      : [];
  });
}
