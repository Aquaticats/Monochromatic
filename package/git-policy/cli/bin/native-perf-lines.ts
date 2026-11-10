/**
 The lines `native-perf.ts` prints: one line of figures for each scenario, and the failure it ends with when a sample or
 a requirement was not met.
 */
import type { BoundViolation } from './native-perf-judge.ts';
import type {
  SampleFailure,
  ScenarioReport,
} from './native-perf-measure.ts';

/**
 Width of the scenario column in the printed lines, so the figures align.
 */
const SCENARIO_COLUMN_WIDTH = 22;

/**
 A millisecond figure as it is printed, with one decimal.

 @param milliseconds - the figure

 @returns the figure and its unit
 */
function formatted(milliseconds: number): string {
  return `${milliseconds
    .toFixed(1)} ms`;
}

/**
 One printed line of figures for a scenario. A commit also shows the median, p95 and maximum of its paired deltas, the
 last of which the commit requirement is judged on.

 @param entry - the scenario's report

 @returns the line

 @example
 ```ts
 console.log(reportLine(entry));
 ```
 */
export function reportLine(entry: ScenarioReport): string {
  return [
    entry.scenario
      .padEnd(SCENARIO_COLUMN_WIDTH),
    `direct median ${formatted(entry.direct
      .median)}`,
    `wrapped median ${formatted(entry.wrapped
      .median)}`,
    `added median ${formatted(entry.addedMedianMilliseconds)}`,
    `added p95 ${formatted(entry.addedP95Milliseconds)}`,
    ...(entry.kind === 'commit'
      ? [
        `paired delta median ${formatted(entry.pairedDelta
          .median)}`,
        `p95 ${formatted(entry.pairedDelta
          .p95)}`,
        `max ${formatted(entry.pairedDelta
          .maximum)}`,
      ]
      : []),
  ].join('  ');
}

/**
 One printed line for a requirement a scenario broke.

 @param violation - the broken requirement

 @returns the line

 @example
 ```ts
 console.log(violationLine(violation));
 ```
 */
export function violationLine(violation: BoundViolation): string {
  return `OVER BOUND: ${violation.scenario} ${violation.statistic} ${formatted(violation.value)} `
    + `>= ${formatted(violation.limit)}`;
}

/**
 Fails a run whose samples or requirements were not all met: one line naming how many samples answered with a failure
 status, then one for each requirement a scenario broke. `native-perf.ts` ends by calling it, so the failure reaches the
 error channel and the exit status instead of being printed beside an assigned standard exit code.

 @param failures - samples whose command answered with a failure status

 @param violations - requirements a scenario broke

 @throws Error when any sample failed or any requirement was broken, with every reason in the message

 @example
 ```ts
 assertSamplesSucceededAndBoundsHeld({ failures, violations });
 ```
 */
export function assertSamplesSucceededAndBoundsHeld({
  failures,
  violations,
}: {
  readonly failures: readonly SampleFailure[];
  readonly violations: readonly BoundViolation[];
}): void {
  if ((failures.length === 0) && (violations.length === 0)) return;
  /**
   Every reason the run failed, in the order the report states them.
   */
  const reasons: string[] = [];
  if (failures.length > 0) {
    reasons.push(`FAILED samples: ${String(failures.length)}; first: ${JSON.stringify(failures[0])}`);
  }
  for (const violation of violations) {
    reasons.push(violationLine(violation));
  }
  throw new Error(reasons.join('\n'));
}
