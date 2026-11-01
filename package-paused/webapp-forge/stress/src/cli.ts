#!/usr/bin/env node
/**
 * Stress harness CLI.
 *
 * `bun src/cli.ts --scenario=hot-repo|bursty-comment|all [--out=results.json]`
 *
 * The harness imports the server package's runtime module so it shares
 * the same in-memory storage adapter and event cursor as a freshly
 * booted server. We force `DB_PATH=:memory:` unless overridden so each
 * run starts with a clean libSQL.
 */

import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';
import { writeFileSync, } from 'node:fs';

import { getFlag, } from './scenarios/shared.ts';
import type {
  Scenario,
  ScenarioResult,
} from './types.ts';

process.env
  .DB_PATH ??= ':memory:';

/**
 * Lazy import keeps the runtime module from initialising before `DB_PATH` is set.
 */
const { hotRepo, } = await import('./scenarios/hot-repo.ts');

/**
 * Sister lazy import for the second scenario.
 */
const { burstyComment, } = await import('./scenarios/bursty-comment.ts');

/**
 * Phase 2 scenario: many sparse repos, parallel writers.
 */
const { wideService, } = await import('./scenarios/wide-service.ts');

/**
 * Phase 2 scenario: force-push iterations against the smart-HTTP receive-pack code path.
 */
const { forcePush, } = await import('./scenarios/force-push.ts');

/**
 * Tagged logger scoped to the stress CLI.
 */
const l = tagged({
  tag: 'stress',
  l: logger,
},);

/**
 * Scenarios known to the CLI (Phase 1 + Phase 2).
 */
const SCENARIOS: readonly Scenario[] = [
  hotRepo,
  burstyComment,
  wideService,
  forcePush,
];

/**
 * Renders a markdown summary table from the given scenario results.
 *
 * @param results - per-scenario results
 *
 * @returns markdown table
 *
 * @example
 * ```ts
 * const md = formatSummary(results);
 * ```
 */
function formatSummary(results: readonly ScenarioResult[],): string {
  /**
   * Accumulator seeded with the markdown table header.
   */
  const lines: string[] = [
    '| scenario | duration ms | events | p50 ms | p99 ms | violations |',
    '| --- | ---: | ---: | ---: | ---: | --- |',
  ];
  for (const r of results) {
    /**
     * Rendered cell, joined for readability when a scenario reports multiple violations.
     */
    const violationText = r.invariantViolations
      .length
      === 0
      ? 'none'
      : r.invariantViolations
        .join('; ',);
    lines.push(
      `| ${r.scenario} | ${String(r.durationMs,)} | ${String(r.eventCount,)} | ${
        String(r.p50,)
      } | ${String(r.p99,)} | ${violationText} |`,
    );
  }
  return lines.join('\n',);
}

/**
 * Resolved `--scenario=` flag, defaulting to `all`.
 */
const target = getFlag('scenario',)
  ?? 'all';

/**
 * Optional `--out=` flag for a JSON results file.
 */
const outFile = getFlag('out',);

/**
 * Filtered scenario list to actually run.
 */
const toRun: Scenario[] = target === 'all'
  ? [...SCENARIOS,]
  : SCENARIOS.filter(function matchesTarget(s,) {
    return s.name
      === target;
  },);

if (toRun.length
  === 0) {
  l.error(
    `no matching scenarios; unknown target: ${target}. Choose one of: ${
      SCENARIOS
        .map(function pickName(s,) {
          return s.name;
        },)
        .join(', ',)
    } or 'all'.`,
  );
  process.exitCode = 1;
}
else {
  /**
   * Aggregated per-scenario results for the report.
   */
  const results: ScenarioResult[] = [];
  for (const scenario of toRun) {
    l.info(`running scenario: ${scenario.name}`,);
    /* oxlint-disable no-await-in-loop -- scenarios share the runtime; serial run by design */
    /**
     * Single-scenario outcome appended to the aggregated report.
     */
    const result = await scenario.run();
    /* oxlint-enable no-await-in-loop */
    results.push(result,);
    l.info(
      `scenario ${result.scenario} done durationMs=${String(result.durationMs,)} p50=${
        String(result.p50,)
      } p99=${String(result.p99,)} violations=${
        String(result.invariantViolations
          .length,)
      }`,
    );
  }

  /**
   * Markdown table emitted to stdout for piping into reports or CI artifacts.
   */
  const summary = formatSummary(results,);
  // CLI output goes through stdout for piping into reports / CI artifacts.
  process.stdout
    .write(`${summary}\n`,);
  if (outFile !== undefined) {
    writeFileSync(
      outFile,
      JSON.stringify(
        results,
        null,
        2,
      ),
    );
  }

  /**
   * True when any scenario reported invariant violations.
   */
  const failed = results.some(function hasViolation(r,) {
    return r.invariantViolations
      .length
      > 0;
  },);
  if (failed)
    process.exitCode = 1;
}
