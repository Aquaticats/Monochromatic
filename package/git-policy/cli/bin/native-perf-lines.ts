/**
 The lines `native-perf.ts` prints: one line of figures for each scenario, and one line for each requirement it broke.
 */
import type { BoundViolation } from './native-perf-judge.ts';
import type { ScenarioReport } from './native-perf-measure.ts';

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
