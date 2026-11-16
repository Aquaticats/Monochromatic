// Reports the run-to-run spread of the unchanged arms measured in spread mode:
// for each configuration, the independent copies' medians, their range, their
// standard deviation, and the largest difference between any two copies. Any
// version difference smaller than this range is noise.
//
// Host-side script; run with: node scripts/spread.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One spread-mode record, which is an ordinary run record with a copy label.
 */
type SpreadRecord = {
  readonly kind: 'run';
  readonly arm: string;
  readonly tag: string;
  readonly round: number;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly ms: readonly number[];
  readonly ok: boolean;
};

/**
 * Spread statistics for one unchanged configuration.
 */
type ConfigSpread = {
  readonly arm: string;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly copies: number;
  readonly medianMs: number;
  readonly copyMediansMs: readonly number[];
  readonly copyMedianMinMs: number;
  readonly copyMedianMaxMs: number;
  readonly copyMedianRangeMs: number;
  readonly copyMedianStandardDeviationMs: number;
  readonly copyMedianRangeRelative: number;
  readonly largestPairwiseGapMs: number;
  readonly allOk: boolean;
};

//endregion Types

//region Statistics

/**
 * Interpolated median.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Median value.
 */
function median(values: readonly number[],): number {
  if (values.length === 0) {
    return Number.NaN;
  }

  /** Sorted copy. */
  const sorted = [...values].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Index of the middle sample. */
  const middleIndex = Math.floor(sorted.length / 2,);

  if (sorted.length % 2 === 1) {
    return sorted[middleIndex] ?? Number.NaN;
  }

  return ((sorted[middleIndex - 1] ?? Number.NaN) + (sorted[middleIndex] ?? Number.NaN)) / 2;
}

/**
 * Sample standard deviation.
 *
 * @param values - Numbers to summarise.
 *
 * @returns Standard deviation, or 0 for fewer than two values.
 */
function standardDeviation(values: readonly number[],): number {
  if (values.length < 2) {
    return 0;
  }

  /** Arithmetic mean. */
  const mean = values.reduce(function sum(total, value,) {
    return total + value;
  }, 0,) / values.length;
  /** Sum of squared deviations. */
  const squaredDeviationSum = values.reduce(function sum(total, value,) {
    return total + ((value - mean) ** 2);
  }, 0,);

  return Math.sqrt(squaredDeviationSum / (values.length - 1),);
}

/**
 * Rounds for the report.
 *
 * @param value - Number to round.
 *
 * @param digits - Decimal places to keep.
 *
 * @returns Rounded number.
 */
function round(value: number, digits: number,): number {
  /** Scale factor. */
  const factor = 10 ** digits;

  return Math.round(value * factor) / factor;
}

//endregion Statistics

//region Entry point

/**
 * Summarises every unchanged configuration's independent copies.
 */
function main(): void {
  /** Every spread record. */
  const records = readFileSync(
    'results/spread.jsonl',
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as SpreadRecord;
    },)
    .filter(function onlyRuns(record,) {
      return record.kind === 'run';
    },);

  /** One median per execution, keyed by configuration and copy label. */
  const perCopy = new Map<string, { configKey: string; arm: string; workload: string; pathSet: string; count: number; medians: number[]; ok: boolean; }>();

  for (const record of records) {
    /** Configuration identity without the copy label. */
    const configKey = `${record.arm}|${record.workload}|${record.pathSet}|${record.count}`;
    /** Copy identity inside the configuration. */
    const copyKey = `${configKey}|${record.tag}`;
    /** Running entry for this copy. */
    const entry = perCopy.get(copyKey,) ?? {
      configKey,
      arm: record.arm,
      workload: record.workload,
      pathSet: record.pathSet,
      count: record.count,
      medians: [],
      ok: true,
    };

    entry.medians.push(median(record.ms,),);
    entry.ok = entry.ok && record.ok;
    perCopy.set(
      copyKey,
      entry,
    );
  }

  /** Copy entries grouped per configuration. */
  const perConfig = new Map<string, { arm: string; workload: string; pathSet: string; count: number; copyMedians: number[]; allOk: boolean; }>();

  for (const entry of perCopy.values()) {
    /** Running configuration entry. */
    const config = perConfig.get(entry.configKey,) ?? {
      arm: entry.arm,
      workload: entry.workload,
      pathSet: entry.pathSet,
      count: entry.count,
      copyMedians: [],
      allOk: true,
    };

    config.copyMedians.push(median(entry.medians,),);
    config.allOk = config.allOk && entry.ok;
    perConfig.set(
      entry.configKey,
      config,
    );
  }

  /** One row per configuration. */
  const spreads: ConfigSpread[] = [...perConfig.values(),].map(function asSpread(config,) {
    /** Smallest copy median. */
    const minimum = Math.min(...config.copyMedians,);
    /** Largest copy median. */
    const maximum = Math.max(...config.copyMedians,);
    /** Median across copies. */
    const acrossCopies = median(config.copyMedians,);

    return {
      arm: config.arm,
      workload: config.workload,
      pathSet: config.pathSet,
      count: config.count,
      copies: config.copyMedians.length,
      medianMs: round(acrossCopies, 4,),
      copyMediansMs: config.copyMedians.map(function asRound(value,) {
        return round(value, 4,);
      },),
      copyMedianMinMs: round(minimum, 4,),
      copyMedianMaxMs: round(maximum, 4,),
      copyMedianRangeMs: round(maximum - minimum, 4,),
      copyMedianStandardDeviationMs: round(
        standardDeviation(config.copyMedians,),
        4,
      ),
      copyMedianRangeRelative: round(
        (maximum - minimum) / acrossCopies,
        4,
      ),
      largestPairwiseGapMs: round(maximum - minimum, 4,),
      allOk: config.allOk,
    };
  },);

  spreads.sort(function ordering(left, right,) {
    return (left.workload.localeCompare(right.workload,))
      || (left.pathSet.localeCompare(right.pathSet,))
      || (left.count - right.count)
      || (left.arm.localeCompare(right.arm,));
  },);

  writeFileSync(
    'results/spread-summary.json',
    `${JSON.stringify(
      {
        records: records.length,
        spreads,
      },
      null,
      2,
    )}\n`,
  );

  for (const spread of spreads) {
    console.log(
      `${spread.workload} ${spread.pathSet} n=${spread.count} ${spread.arm} copies=${
        spread.copies
      } median=${spread.medianMs}ms range=${spread.copyMedianRangeMs}ms (${
        (spread.copyMedianRangeRelative * 100).toFixed(1)
      }%) sd=${spread.copyMedianStandardDeviationMs}ms min=${
        spread.copyMedianMinMs
      } max=${spread.copyMedianMaxMs} allOk=${spread.allOk}`,
    );
    console.log(`  copy medians: ${spread.copyMediansMs.join(', ')}`);
  }
}

main();

//endregion Entry point
