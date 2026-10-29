/**
 Plain-text report of scenario results.

 @module
 */

import type { ScenarioResult, } from './scenario-model-fixture.ts';

//region Formatting

/**
 Status label width so columns align.
 */
const STATUS_WIDTH = 5;

/**
 Workload digest prefix length in the report.
 */
const WORKLOAD_PREFIX = 12;

/**
 Detail lines shown per result, enough for an error's message and top frames.
 */
const MAX_DETAIL_LINES = 8;

/**
 Formats one result with its violations.

 @param result - scenario result

 @returns report lines

 @example
 ```ts
 formatResult(result);
 ```
 */
export function formatResult(result: ScenarioResult,): readonly string[] {
  /**
   Attempt exit summary.
   */
  const attempts = result.attempts
    .map(function attemptSummary(attempt,) {
      return `${attempt.label}=${attempt.killed ? 'killed' : String(attempt.exitCode,)}`;
    },)
    .join(' ',);
  return [
    `${result.status
      .toUpperCase()
      .padEnd(STATUS_WIDTH,)} git ${result.gitVersion} ${result.name} (${String(result.durationMs,)} ms)${result.workload === undefined ? '' : ` workload ${result.workload
        .slice(
          0,
          WORKLOAD_PREFIX,
        )}`}${attempts === '' ? '' : ` exits: ${attempts}`}`,
    ...result.violations
      .map(function violationLine(violation,) {
        return `      ${violation.invariant} [${violation.subject}]: ${violation.detail}`;
      },),
    ...(result.detail === undefined ? [] : result.detail
      .split('\n',)
      .slice(
      0,
      MAX_DETAIL_LINES,
      )
      .map(function detailLine(line,) {
        return `      ${line}`;
      },)),
  ];
}

/**
 Summarizes result counts per group and status.

 @param results - every result

 @returns summary lines

 @example
 ```ts
 summarize(results);
 ```
 */
export function summarize(results: readonly ScenarioResult[],): readonly string[] {
  /**
   Count per group and status.
   */
  const counts = [...Map.groupBy(
    results,
    function byGroupAndStatus(result,) {
      return `${result.group} ${result.status}`;
    },
  ),].map(function countEntry([key, grouped,]: readonly [
    string,
    readonly ScenarioResult[]
  ],) {
    return [
      key,
      grouped.length,
    ] as const;
  },);
  return [
    `summary: ${String(results.length,)} scenario runs`,
    ...counts.toSorted(function byKey(
      [left,],
      [right,],
    ) {
      return left.localeCompare(right,);
    },)
      .map(function countLine([key, value,],) {
        return `  ${key}: ${String(value,)}`;
      },),
  ];
}

/**
 Reports whether the run passed:
 no scenario failed or errored.

 @param results - every result

 @returns overall pass

 @example
 ```ts
 runPassed(results);
 ```
 */
export function runPassed(results: readonly ScenarioResult[],): boolean {
  return results.every(function passedOrSkipped(result,) {
    return (result.status === 'pass') || (result.status === 'skip');
  },);
}

//endregion Formatting
