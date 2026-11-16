// Builds the report tables from results/summary.json and
// results/spread-summary.json: per-cell medians with range, the factor over the
// owned parser, and adjacent-release differences judged against the measured
// spread of an unchanged arm.
//
// Host-side script; run with: node scripts/report-tables.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One arm's statistics for one cell, as written by summarise.ts.
 */
type ArmSummary = {
  readonly arm: string;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly medianMs: number;
  readonly minMs: number;
  readonly maxMs: number;
  readonly roundMedianRelativeSpread: number;
  readonly samples: number;
  readonly ok: boolean;
};

/**
 * One unchanged configuration's spread, as written by spread.ts.
 */
type SpreadSummary = {
  readonly arm: string;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly copies: number;
  readonly medianMs: number;
  readonly copyMedianRangeRelative: number;
  readonly copyMedianRangeMs: number;
};

//endregion Types

//region Config

/**
 * Arms in report order, ending with the baseline.
 */
const ARM_ORDER: readonly string[] = [
  '1.1.1',
  '1.3.1',
  '1.3.2',
  '1.3.6',
  '1.4.0-dev.2664',
  '1.4.0-dev.2665',
  '1.4.0-dev.2667',
  '1.4.0-dev.2971',
  'owned-4879a44e',
];

/**
 * Baseline arm label.
 */
const BASELINE = 'owned-4879a44e';

/**
 * Release pairs whose difference the report must judge against the spread.
 */
const ADJACENT_PAIRS: readonly (readonly [string, string, string])[] = [
  ['1.1.1', '1.3.1', 'before the 1.3.2 patch'],
  ['1.3.1', '1.3.2', 'the 1.3.2 patch (pull 992)'],
  ['1.3.2', '1.3.6', 'three later 1.3 maintenance releases'],
  ['1.3.6', '1.4.0-dev.2664', 'last dev build before pull 997'],
  ['1.4.0-dev.2664', '1.4.0-dev.2665', 'pull 997 (deferred diagnostics)'],
  ['1.4.0-dev.2665', '1.4.0-dev.2667', 'the pull-997 merge commit'],
  ['1.4.0-dev.2667', '1.4.0-dev.2971', 'main since 2667 up to current dev'],
];

//endregion Config

//region Helpers

/**
 * Formats a duration with a fixed number of decimals.
 *
 * @param value - Milliseconds.
 *
 * @param digits - Decimal places.
 *
 * @returns Formatted number.
 */
function formatMs(value: number, digits: number,): string {
  return value.toFixed(digits,);
}

/**
 * Per-cent change from one value to another.
 *
 * @param from - Earlier value.
 *
 * @param to - Later value.
 *
 * @returns Signed percentage change.
 */
function percentChange(from: number, to: number,): number {
  return ((to - from) / from) * 100;
}

//endregion Helpers

//region Entry point

/**
 * Builds every report table and writes `results/report-tables.md`.
 */
function main(): void {
  /** Per-arm summaries. */
  const summary = JSON.parse(
    readFileSync(
      'results/summary.json',
      'utf8',
    ),
  ) as { arms: readonly ArmSummary[]; };
  /** Spread summaries. */
  const spread = JSON.parse(
    readFileSync(
      'results/spread-summary.json',
      'utf8',
    ),
  ) as { spreads: readonly SpreadSummary[]; };

  /** Arm summaries keyed by cell and arm. */
  const byCell = new Map<string, Map<string, ArmSummary>>();

  for (const entry of summary.arms) {
    /** Cell key. */
    const cellKey = `${entry.workload}|${entry.pathSet}|${entry.count}`;
    /** Arm map for this cell. */
    const perArm = byCell.get(cellKey,) ?? new Map<string, ArmSummary>();

    perArm.set(
      entry.arm,
      entry,
    );
    byCell.set(
      cellKey,
      perArm,
    );
  }

  /** Noise threshold per cell, from the unchanged-arm spread. */
  const thresholds = new Map<string, number>();
  /** Largest spread seen for a workload and path set, used as a fallback. */
  const spreadByShape = new Map<string, number>();

  for (const entry of spread.spreads) {
    /** Cell key for this spread configuration. */
    const cellKey = `${entry.workload}|${entry.pathSet}|${entry.count}`;
    /** Shape key for the fallback. */
    const shapeKey = `${entry.workload}|${entry.pathSet}`;
    /** Current threshold for this cell. */
    const existing = thresholds.get(cellKey,);

    if (existing === undefined || entry.copyMedianRangeRelative > existing) {
      thresholds.set(
        cellKey,
        entry.copyMedianRangeRelative,
      );
    }

    /** Current fallback for this shape. */
    const shapeExisting = spreadByShape.get(shapeKey,) ?? 0;

    if (entry.copyMedianRangeRelative > shapeExisting) {
      spreadByShape.set(
        shapeKey,
        entry.copyMedianRangeRelative,
      );
    }
  }

  /** Threshold applying to a cell, with a shape-level fallback. */
  function thresholdFor(
    workload: string,
    pathSet: string,
    count: number,
  ): { value: number; fallback: boolean; } {
    /** Exact cell threshold. */
    const exact = thresholds.get(`${workload}|${pathSet}|${count}`,);

    if (exact !== undefined) {
      return {
        value: exact,
        fallback: false,
      };
    }

    /** Shape-level threshold, when one was measured for this workload. */
    const shape = spreadByShape.get(`${workload}|${pathSet}`,);

    if (shape !== undefined) {
      return {
        value: shape,
        fallback: true,
      };
    }

    /** Largest range any unchanged arm showed, used as the last resort. */
    const global = Math.max(...spread.spreads.map(function rangeOf(entry,) {
      return entry.copyMedianRangeRelative;
    },),);

    return {
      value: Number.isFinite(global,) ? global : 0,
      fallback: true,
    };
  }

  /** Markdown lines of the report. */
  const lines: string[] = [];

  lines.push('## Measured spread of an unchanged arm',);
  lines.push('',);
  for (const entry of spread.spreads) {
    lines.push(
      `- \`${entry.arm}\` ${entry.workload} ${entry.pathSet} n=${entry.count}: `,
      `${entry.copies} copies, median ${formatMs(
        entry.medianMs,
        2,
      )} ms, range ${formatMs(
        entry.copyMedianRangeMs,
        2,
      )} ms (${(entry.copyMedianRangeRelative * 100).toFixed(1)}%)`,);
  }
  lines.push('',);

  for (const [cellKey, perArm,] of byCell) {
    /** Baseline statistics for this cell. */
    const baseline = perArm.get(BASELINE,);
    /** Noise threshold for this cell. */
    const threshold = thresholdFor(
      baseline?.workload ?? '',
      baseline?.pathSet ?? '',
      baseline?.count ?? 0,
    );
    /** Distinct parts of the cell key, dropping duplicated workload and path set. */
    const cellParts = cellKey
      .split('|',)
      .filter(function distinct(value, index, all,) {
        return all.indexOf(value,) === index;
      },);

    lines.push(
      `## ${cellParts.join(' ')}${
        baseline === undefined ? '' : ` n=${baseline.count}`
      }`,
      '',
    );

    lines.push(
      'Measured medians, with the range of every timed sample:',
      '',
    );

    for (const arm of ARM_ORDER) {
      /** Statistics for this arm. */
      const entry = perArm.get(arm,);

      if (entry === undefined) {
        lines.push(`- \`${arm}\`: not measured`,);
        continue;
      }

      /** Factor over the owned parser. */
      const factor = baseline === undefined || baseline.medianMs === 0 || entry.arm === BASELINE
        ? ''
        : `, ${(entry.medianMs / baseline.medianMs).toFixed(1)}x the owned parser`;

      lines.push(
        `- \`${entry.arm}\`: median ${formatMs(
          entry.medianMs,
          3,
        )} ms (min ${formatMs(
          entry.minMs,
          3,
        )} ms, max ${formatMs(
          entry.maxMs,
          3,
        )} ms, ${entry.samples} timed parses${
          entry.ok ? '' : ', RESULT MISMATCH'
        })${factor}`,
      );
    }

    lines.push('',);
    lines.push(
      `Noise threshold from unchanged arms for this cell: ${
        (threshold.value * 100).toFixed(1)
      }%${
        threshold.fallback ? ' (nearest measured configuration, not this cell)' : ''
      }`,
    );
    lines.push('',);

    /** Adjacent-release deltas for this cell. */
    const pairLines: string[] = [];

    for (const [earlier, later, label,] of ADJACENT_PAIRS) {
      /** Earlier statistics. */
      const earlierEntry = perArm.get(earlier,);
      /** Later statistics. */
      const laterEntry = perArm.get(later,);

      if (earlierEntry === undefined || laterEntry === undefined) {
        continue;
      }

      /** Signed change from earlier to later. */
      const change = percentChange(
        earlierEntry.medianMs,
        laterEntry.medianMs,
      );
      /** Whether the change is larger than the measured spread. */
      const verdict = Math.abs(change) / 100 > threshold.value ? 'beyond noise' : 'within noise';

      pairLines.push(
        `- \`${earlier}\` to \`${later}\` (${label}): ${
          formatMs(
            earlierEntry.medianMs,
            3,
          )
        } ms to ${
          formatMs(
            laterEntry.medianMs,
            3,
          )
        } ms, ${change.toFixed(1)}%, ${verdict}`,
      );
    }

    lines.push(...pairLines,);
    lines.push('',);
  }

  writeFileSync(
    'results/report-tables.md',
    `${lines.join('\n',)}\n`,
  );
  process.stdout.write(`${lines.join('\n',)}\n`,);
}

main();

//endregion Entry point
