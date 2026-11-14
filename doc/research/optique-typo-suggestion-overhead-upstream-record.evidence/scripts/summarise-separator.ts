// Summarises the separator-shape runs: per-arm medians and range, the
// unchanged-arm spread, and the 2971-over-1.3.6 ratio with its uncertainty.
// Host-side script; run with: node scripts/summarise-separator.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One separator record.
 */
type SeparatorRecord = {
  readonly kind: 'run';
  readonly arm: string;
  readonly tag: string;
  readonly round: number;
  readonly ms: readonly number[];
  readonly tokens: number;
  readonly ok: boolean;
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
  /** Sorted copy. */
  const sorted = [...values].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Middle index. */
  const middle = Math.floor(sorted.length / 2,);

  return sorted.length % 2 === 0
    ? ((sorted[middle - 1] ?? Number.NaN) + (sorted[middle] ?? Number.NaN)) / 2
    : (sorted[middle] ?? Number.NaN);
}

/**
 * Rounds for the report.
 *
 * @param value - Number to round.
 *
 * @param digits - Decimal places.
 *
 * @returns Rounded number.
 */
function round(value: number, digits: number,): number {
  /** Scale factor. */
  const factor = 10 ** digits;

  return Math.round(value * factor) / factor;
}

/**
 * Reads a JSONL file of separator records.
 *
 * @param path - File to read.
 *
 * @returns Every record in it.
 */
function readRecords(path: string,): SeparatorRecord[] {
  return readFileSync(
    path,
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as SeparatorRecord;
    },)
    .filter(function onlyRuns(record,) {
      return record.kind === 'run';
    },);
}

//endregion Statistics

//region Entry point

/**
 * Summarises the separator matrix and spread runs.
 */
function main(): void {
  /** Matrix records. */
  const matrix = readRecords('results-scaling/separator.jsonl',);
  /** Spread records. */
  const spread = readRecords('results-scaling/separator-spread.jsonl',);
  /** Per-arm samples from the matrix. */
  const perArm = new Map<string, number[]>();
  /** Per-round medians per arm. */
  const perArmRounds = new Map<string, number[]>();

  for (const record of matrix) {
    perArm.set(
      record.arm,
      [
        ...(perArm.get(record.arm,) ?? []),
        ...record.ms,
      ],
    );
    perArmRounds.set(
      record.arm,
      [
        ...(perArmRounds.get(record.arm,) ?? []),
        median(record.ms,),
      ],
    );
  }

  /** Per-arm summary. */
  const arms = [...perArm.entries(),].map(function summary(entry,) {
    /** Every sample for this arm. */
    const samples = entry[1];
    /** Round medians for this arm. */
    const roundMedians = perArmRounds.get(entry[0],) ?? [];

    return {
      arm: entry[0],
      samples: samples.length,
      medianMs: round(median(samples,), 3,),
      minMs: round(Math.min(...samples,), 3,),
      maxMs: round(Math.max(...samples,), 3,),
      roundMediansMs: roundMedians.map(function asRound(value,) {
        return round(value, 3,);
      },),
    };
  },);

  /** Copy medians per arm from the spread run. */
  const perCopy = new Map<string, number[]>();

  for (const record of spread) {
    /** Key combining arm and copy label. */
    const key = `${record.arm}|${record.tag}`;

    perCopy.set(
      key,
      [
        ...(perCopy.get(key,) ?? []),
        median(record.ms,),
      ],
    );
  }

  /** Spread summary per arm. */
  const spreadArms = [...new Set(spread.map(function armOf(record,) {
    return record.arm;
  },),),].map(function summary(arm,) {
    /** Copy medians for this arm. */
    const copyMedians = [...perCopy.entries(),]
      .filter(function belongs(entry,) {
        return entry[0].startsWith(`${arm}|`,);
      },)
      .map(function medianOf(entry,) {
        return median(entry[1],);
      },);
    /** Smallest copy median. */
    const smallest = Math.min(...copyMedians,);
    /** Largest copy median. */
    const largest = Math.max(...copyMedians,);

    return {
      arm,
      copies: copyMedians.length,
      medianMs: round(median(copyMedians,), 3,),
      copyMedianMinMs: round(smallest, 3,),
      copyMedianMaxMs: round(largest, 3,),
      copyMedianRangeMs: round(largest - smallest, 3,),
      copyMedianRangeRelative: round(
        (largest - smallest) / median(copyMedians,),
        4,
      ),
    };
  },);

  /** Ratio of 2971 to 1.3.6, with the spread's worst case. */
  const baseline = arms.find(function isBaseline(entry,) {
    return entry.arm === '1.3.6';
  },);
  /** Slowest arm's summary. */
  const slow = arms.find(function isSlow(entry,) {
    return entry.arm === '1.4.0-dev.2971';
  },);
  /** Ratio from the matrix. */
  const ratio = baseline === undefined || slow === undefined
    ? Number.NaN
    : slow.medianMs / baseline.medianMs;
  /** Baseline spread. */
  const baselineSpread = spreadArms.find(function isBaseline(entry,) {
    return entry.arm === '1.3.6';
  },);
  /** Slow arm spread. */
  const slowSpread = spreadArms.find(function isSlow(entry,) {
    return entry.arm === '1.4.0-dev.2971';
  },);

  /** The report. */
  const report = {
    generatedAt: new Date().toISOString(),
    arms,
    spreadArms,
    ratio2971Over136: round(ratio, 3,),
    ratioWorstCase: baselineSpread === undefined || slowSpread === undefined
      ? Number.NaN
      : round(
        slowSpread.copyMedianMaxMs / baselineSpread.copyMedianMinMs,
        3,
      ),
    ratioBestCase: baselineSpread === undefined || slowSpread === undefined
      ? Number.NaN
      : round(
        slowSpread.copyMedianMinMs / baselineSpread.copyMedianMaxMs,
        3,
      ),
  };

  writeFileSync(
    'results-scaling/separator-summary.json',
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
  );

  for (const arm of arms) {
    console.log(
      `separator ${arm.arm.padEnd(18,)} median=${arm.medianMs} ms min=${arm.minMs} max=${arm.maxMs} rounds=${
        JSON.stringify(arm.roundMediansMs)
      }`,
    );
  }

  for (const arm of spreadArms) {
    console.log(
      `separator spread ${arm.arm.padEnd(18,)} copies=${arm.copies} median=${arm.medianMs} range=${
        arm.copyMedianRangeMs
      } ms (${(arm.copyMedianRangeRelative * 100).toFixed(1)}%)`,
    );
  }

  console.log(
    `2971 over 1.3.6: ${report.ratio2971Over136}x (worst-case spread ${report.ratioWorstCase}x, best-case ${
      report.ratioBestCase
    }x)`,
  );
}

main();

//endregion Entry point
