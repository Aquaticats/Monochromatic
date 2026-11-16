// Summarises the 0.7.0 search-cost probe: the cost of one suggestion search,
// the margin against the design note's "less than 1ms", and the counted number
// of searches one thousand-path successful parse performs.
// Host-side script; run with: node scripts/summarise-search.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One search-cost record.
 */
type SearchRecord = {
  readonly kind: 'search-cost';
  readonly arm: string;
  readonly optionCount: number;
  readonly nameLength: number;
  readonly mistyped: string;
  readonly editDistanceCells: number;
  readonly search: {
    readonly repeats: number;
    readonly medianMs: number;
    readonly minMs: number;
    readonly suggestionsForMistyped: readonly string[];
  };
  readonly failingParse: {
    readonly repeats: number;
    readonly medianMs: number;
    readonly minMs: number;
  };
  readonly succeedingParse: {
    readonly repeats: number;
    readonly medianMs: number;
    readonly minMs: number;
  };
  readonly countedParse: {
    readonly pathCount: number;
    readonly success: boolean;
    readonly probes: number;
    readonly acceptedPaths: number;
    readonly ms: number;
  };
};

//endregion Types

//region Entry point

/**
 * Summarises every search-cost record and writes the Part 2 report.
 */
function main(): void {
  /** Every record from the probe's fresh processes. */
  const records = readFileSync(
    'results-scaling/search.jsonl',
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as SearchRecord;
    },);
  /** First record, which carries the shape. */
  const [first] = records;

  if (first === undefined) {
    throw new Error('no search-cost records',);
  }

  /** Median search time across the fresh processes. */
  const searchMedian = records
    .map(function searchOf(record,) {
      return record.search.medianMs;
    },)
    .sort(function ascending(left, right,) {
      return left - right;
    },)[Math.floor(records.length / 2,)] ?? Number.NaN;
  /** Median failing-parse time across the fresh processes. */
  const failingMedian = records
    .map(function failingOf(record,) {
      return record.failingParse.medianMs;
    },)
    .sort(function ascending(left, right,) {
      return left - right;
    },)[Math.floor(records.length / 2,)] ?? Number.NaN;
  /** Median succeeding-parse time across the fresh processes. */
  const succeedingMedian = records
    .map(function succeedingOf(record,) {
      return record.succeedingParse.medianMs;
    },)
    .sort(function ascending(left, right,) {
      return left - right;
    },)[Math.floor(records.length / 2,)] ?? Number.NaN;
  /** Nanoseconds per edit-distance cell implied by the isolated search. */
  const nanosecondsPerCell = (searchMedian * 1e6) / first.editDistanceCells;
  /** Margin of one search below the note's one millisecond. */
  const marginBelowOneMillisecond = 1 / searchMedian;

  /** The report. */
  const report = {
    generatedAt: new Date().toISOString(),
    arm: first.arm,
    parameters: {
      optionCount: first.optionCount,
      nameLength: first.nameLength,
      mistyped: first.mistyped,
      editDistanceCellsPerSearch: first.editDistanceCells,
    },
    processes: records.length,
    searchMedianMs: searchMedian,
    searchMinMs: Math.min(...records.map(function searchOf(record,) {
      return record.search.minMs;
    },),),
    nanosecondsPerCell: Number(nanosecondsPerCell.toFixed(3),),
    marginBelowOneMillisecond: Number(marginBelowOneMillisecond.toFixed(2),),
    failingParseMedianMs: failingMedian,
    failingParseSearchesImplied: Number((failingMedian / searchMedian).toFixed(1),),
    succeedingParseMedianMs: succeedingMedian,
    countedParse: first.countedParse,
    suggestions: first.search.suggestionsForMistyped,
  };

  writeFileSync(
    'results-scaling/search-summary.json',
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
  );

  console.log(`one search (${first.optionCount} names x ${first.nameLength} chars): ${
    searchMedian.toFixed(4)
  } ms = ${nanosecondsPerCell.toFixed(2)} ns per edit-distance cell`);
  console.log(`margin against the note's "less than 1ms": ${marginBelowOneMillisecond.toFixed(2)}x`);
  console.log(
    `one failing parse with a mistyped option: ${failingMedian.toFixed(3)} ms (=${
      (failingMedian / searchMedian).toFixed(1)
    } searches), succeeding parse: ${succeedingMedian.toFixed(4)} ms`,
  );
  console.log(
    `1000-path parse on 0.7.0: probes=${first.countedParse.probes}, accepted=${
      first.countedParse.acceptedPaths
    }, wall=${first.countedParse.ms.toFixed(1)} ms`,
  );
}

main();

//endregion Entry point
