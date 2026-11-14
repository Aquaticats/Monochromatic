// Turns results/runs.jsonl into per-arm statistics: pooled median, minimum,
// maximum, the spread of round medians, and whether every arm returned the
// same CommitRegion record. Writes results/summary.json and prints a table.
//
// Host-side script; run with: node scripts/summarise.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One timing record printed by the arm runner.
 */
type RunRecord = {
  readonly kind: 'run';
  readonly arm: string;
  readonly tag: string;
  readonly round: number;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly ms: readonly number[];
  readonly ok: boolean;
  readonly digest: string;
  readonly handled: number;
  readonly node: string;
  readonly load1: number;
};

/**
 * Statistics for one arm on one cell.
 */
type ArmCellSummary = {
  readonly arm: string;
  readonly workload: string;
  readonly pathSet: string;
  readonly count: number;
  readonly executions: number;
  readonly samples: number;
  readonly medianMs: number;
  readonly minMs: number;
  readonly maxMs: number;
  readonly roundMediansMs: readonly number[];
  readonly roundMedianMinMs: number;
  readonly roundMedianMaxMs: number;
  readonly roundMedianRelativeSpread: number;
  readonly ok: boolean;
  readonly digests: readonly string[];
};

//endregion Types

//region Statistics

/**
 * Interpolated median: the middle sample, or the mean of the two middle ones.
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

  /** Lower middle sample. */
  const lower = sorted[middleIndex - 1] ?? Number.NaN;
  /** Upper middle sample. */
  const upper = sorted[middleIndex] ?? Number.NaN;

  return (lower + upper) / 2;
}

/**
 * Rounds a number for the report without losing grade.
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
 * Reads every run record, summarises each arm and cell, and reports digest
 * agreement across arms for the same cell.
 */
function main(): void {
  /** Raw JSONL text. */
  const text = readFileSync(
    'results/runs.jsonl',
    'utf8',
  );
  /** Every run record in file order. */
  const records = text
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as RunRecord;
    },)
    .filter(function onlyRuns(record,) {
      return record.kind === 'run';
    },);

  /** Records grouped by cell and arm. */
  const grouped = new Map<string, RunRecord[]>();

  for (const record of records) {
    /** Group key: cell plus arm. */
    const key = `${record.workload}|${record.pathSet}|${record.count}|${record.arm}|${record.tag}`;
    /** Existing group. */
    const group = grouped.get(key,);

    if (group === undefined) {
      grouped.set(
        key,
        [record,],
      );
    }
    else {
      group.push(record,);
    }
  }

  /** Summary per arm and cell. */
  const summaries: ArmCellSummary[] = [];

  for (const group of grouped.values()) {
    /** First record, which carries the cell identity. */
    const [first] = group;

    if (first === undefined) {
      continue;
    }

    /** Every timed sample across executions. */
    const samples = group.flatMap(function samplesOf(record,) {
      return [...record.ms,];
    },);
    /** Median of each execution. */
    const roundMedians = group.map(function medianOf(record,) {
      return median(record.ms,);
    },);
    /** Pooled median. */
    const pooledMedian = median(samples,);
    /** Smallest execution median. */
    const roundMedianMin = Math.min(...roundMedians,);
    /** Largest execution median. */
    const roundMedianMax = Math.max(...roundMedians,);
    /** Distinct digests this arm produced for the cell. */
    const digests = [...new Set(group.map(function digestOf(record,) {
      return record.digest;
    },),),];

    summaries.push({
      arm: first.arm,
      workload: first.workload,
      pathSet: first.pathSet,
      count: first.count,
      executions: group.length,
      samples: samples.length,
      medianMs: round(pooledMedian, 4,),
      minMs: round(Math.min(...samples,), 4,),
      maxMs: round(Math.max(...samples,), 4,),
      roundMediansMs: roundMedians.map(function asRound(value,) {
        return round(value, 4,);
      },),
      roundMedianMinMs: round(roundMedianMin, 4,),
      roundMedianMaxMs: round(roundMedianMax, 4,),
      roundMedianRelativeSpread: round(
        (roundMedianMax - roundMedianMin) / pooledMedian,
        4,
      ),
      ok: group.every(function okOf(record,) {
        return record.ok;
      },),
      digests,
    });
  }

  /** Cells in a stable order. */
  const cellOrder = [
    'no-separator',
    'separator',
  ];
  /** Arms in a stable order. */
  const armOrder = [
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

  summaries.sort(function ordering(left, right,) {
    /** Workload rank. */
    const leftWorkload = cellOrder.indexOf(left.workload,);
    /** Workload rank. */
    const rightWorkload = cellOrder.indexOf(right.workload,);
    /** Arm rank. */
    const leftArm = armOrder.indexOf(left.arm,);
    /** Arm rank. */
    const rightArm = armOrder.indexOf(right.arm,);

    return (leftWorkload - rightWorkload) || (left.pathSet.localeCompare(right.pathSet,))
      || (left.count - right.count) || (leftArm - rightArm);
  },);

  /** Digest variants per cell, across arms. */
  const cellDigests = new Map<string, Map<string, string[]>>();

  for (const summary of summaries) {
    /** Cell key without the arm. */
    const cellKey = `${summary.workload}|${summary.pathSet}|${summary.count}`;
    /** Digest map for this cell. */
    const perDigest = cellDigests.get(cellKey,) ?? new Map<string, string[]>();

    for (const digest of summary.digests) {
      /** Arms producing this digest. */
      const arms = perDigest.get(digest,) ?? [];

      arms.push(summary.arm,);
      perDigest.set(
        digest,
        arms,
      );
    }
    cellDigests.set(
      cellKey,
      perDigest,
    );
  }

  /** Cell-level behavior check. */
  const behavior = [...cellDigests.entries(),].map(function asEntry(entry,) {
    return {
      cell: entry[0],
      distinctResults: entry[1].size,
      groups: [...entry[1].entries(),].map(function asGroup(group,) {
        return {
          digest: group[0],
          arms: group[1].sort(),
        };
      },),
    };
  },);

  /** Whole summary document. */
  const summary = {
    generatedAt: new Date().toISOString(),
    records: records.length,
    arms: summaries,
    behavior,
    allOk: summaries.every(function okOf(summary,) {
      return summary.ok;
    },),
  };

  writeFileSync(
    'results/summary.json',
    `${JSON.stringify(
      summary,
      null,
      2,
    )}\n`,
  );

  for (const entry of summaries) {
    console.log(
      `${
        entry.workload.padEnd(12,)
      } ${entry.pathSet.padEnd(5,)} n=${
        String(entry.count,).padStart(4,)
      } ${entry.arm.padEnd(18,)} median=${
        String(entry.medianMs,).padStart(9,)
      }ms min=${
        String(entry.minMs,).padStart(9,)
      }ms max=${
        String(entry.maxMs,).padStart(9,)
      }ms roundSpread=${
        String(entry.roundMedianRelativeSpread,).padStart(6,)
      } ok=${entry.ok}`,
    );
  }

  console.log('');

  for (const entry of behavior) {
    console.log(
      `behavior ${entry.cell}: distinctResults=${entry.distinctResults}`,
    );
    if (entry.distinctResults > 1) {
      for (const group of entry.groups) {
        console.log(`  ${group.digest.slice(
          0,
          12,
        )}: ${group.arms.join(', ')}`,);
      }
    }
  }

  console.log(`allOk=${summary.allOk} records=${records.length}`);
}

main();

//endregion Entry point
