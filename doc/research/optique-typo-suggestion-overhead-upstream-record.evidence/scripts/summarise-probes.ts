// Summarises the two extra shape probes: the token sweep at one option parser
// (does the quadratic token floor survive without option probing?) and the
// separator profile at two token counts (does InputTrace.set carry it?).
// Host-side script; run with: node scripts/summarise-probes.ts

import { readFileSync, writeFileSync, } from 'node:fs';

//region Types

/**
 * One timed record from the single-option token probe.
 */
type ProbeRecord = {
  readonly kind: 'run';
  readonly arm: string;
  readonly level: number;
  readonly ms: readonly number[];
};

/**
 * One profile analysis from the separator size probe.
 */
type SizeProfile = {
  readonly kind: 'profile';
  readonly arm: string;
  readonly tokenCount: number;
  readonly repeat: number;
  readonly parseMs: number;
  readonly topFiles: readonly { readonly file: string; readonly ms: number; readonly shareOfParse: number; }[];
  readonly topFunctions: readonly { readonly function: string; readonly ms: number; readonly shareOfParse: number; }[];
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
 * Reads a JSONL file.
 *
 * @param path - File to read.
 *
 * @returns Parsed records.
 */
function readJsonl<T>(path: string,): T[] {
  return readFileSync(
    path,
    'utf8',
  )
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim() !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as T;
    },);
}

/**
 * Log-log slope of y on x.
 *
 * @param xs - Independent values, positive.
 *
 * @param ys - Dependent values, positive.
 *
 * @returns Slope.
 */
function logSlope(xs: readonly number[], ys: readonly number[],): number {
  /** Mean of log x. */
  const xMean = xs.reduce(function sum(total, value,) {
    return total + Math.log(value,);
  }, 0,) / xs.length;
  /** Mean of log y. */
  const yMean = ys.reduce(function sum(total, value,) {
    return total + Math.log(value,);
  }, 0,) / ys.length;
  /** Covariance. */
  let numerator = 0;
  /** Variance of log x. */
  let denominator = 0;

  for (const [index, x,] of xs.entries()) {
    /** Deviation of log x. */
    const dx = Math.log(x,) - xMean;
    /** Deviation of log y. */
    const dy = Math.log(ys[index] ?? Number.NaN,) - yMean;

    numerator += dx * dy;
    denominator += dx * dx;
  }

  return numerator / denominator;
}

//endregion Statistics

//region Entry point

/**
 * Summarises both probes and writes the report.
 */
function main(): void {
  /** Token-probe records. */
  const tokenRecords = readJsonl<ProbeRecord>('results-scaling/token-probe.jsonl',);
  /** Size profiles. */
  const sizeProfiles = readJsonl<SizeProfile>('results-scaling/profile-sizes.jsonl',);
  /** Arms in the token probe. */
  const tokenArms = [...new Set(tokenRecords.map(function armOf(record,) {
    return record.arm;
  },),),];
  /** Token counts in the token probe. */
  const tokenCounts = [...new Set(tokenRecords.map(function levelOf(record,) {
    return record.level;
  },),),].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Token-probe summary per arm. */
  const tokenProbe = tokenArms.map(function summary(arm,) {
    /** Minimum per token count. */
    const minima = tokenCounts.map(function minimumOf(level,) {
      /** Samples at this level. */
      const samples = tokenRecords
        .filter(function matches(record,) {
          return record.arm === arm && record.level === level;
        },)
        .flatMap(function msOf(record,) {
          return [...record.ms,];
        },);

      return Math.min(...samples,);
    },);

    return {
      arm,
      minimaMs: minima.map(function rounded(value,) {
        return Number(value.toFixed(3),);
      },),
      microsecondsPerToken: minima.map(function perToken(value, index,) {
        return Number(((value * 1000) / (tokenCounts[index] ?? 1)).toFixed(2),);
      },),
      exponentOnMinima: Number(logSlope(
        tokenCounts,
        minima,
      ).toFixed(3),),
    };
  },);

  /** Size-profile summary. */
  const sizes = sizeProfiles.map(function summary(profile,) {
    /** InputTrace file row. */
    const trace = profile.topFiles.find(function isTrace(file,) {
      return file.file === 'input-trace.js';
    },);

    return {
      arm: profile.arm,
      tokenCount: profile.tokenCount,
      parseMsPerParse: Number((profile.parseMs / profile.repeat).toFixed(3),),
      inputTraceMsPerParse: trace === undefined
        ? null
        : Number((trace.ms / profile.repeat).toFixed(3),),
      inputTraceShare: trace?.shareOfParse ?? null,
      setShare: profile.topFunctions.find(function isSet(entry,) {
        return entry.function === 'set';
      },)?.shareOfParse ?? null,
    };
  },);

  /** Exponent of each series across the two sizes. */
  const sizeExponents = sizeProfiles.length === 2
    ? {
      parseExponent: Number(logSlope(
        sizes.map(function countOf(size,) {
          return size.tokenCount;
        },),
        sizes.map(function parseOf(size,) {
          return size.parseMsPerParse;
        },),
      ).toFixed(3),),
      inputTraceExponent: sizes.every(function hasTrace(size,) {
        return size.inputTraceMsPerParse !== null;
      },)
        ? Number(logSlope(
          sizes.map(function countOf(size,) {
            return size.tokenCount;
          },),
          sizes.map(function traceOf(size,) {
            return size.inputTraceMsPerParse ?? 1;
          },),
        ).toFixed(3),)
        : null,
    }
    : undefined;

  /** The report. */
  const report = {
    generatedAt: new Date().toISOString(),
    tokenCounts,
    tokenProbe,
    sizes,
    sizeExponents,
  };

  writeFileSync(
    'results-scaling/probes-summary.json',
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
  );

  for (const arm of tokenProbe) {
    console.log(
      `one-option token probe ${arm.arm.padEnd(18,)} minima=${JSON.stringify(arm.minimaMs)} us/token=${
        JSON.stringify(arm.microsecondsPerToken)
      } exponent=${arm.exponentOnMinima}`,
    );
  }

  for (const size of sizes) {
    console.log(
      `separator profile N=${size.tokenCount} ${size.arm}: ${size.parseMsPerParse} ms/parse, input-trace ${
        size.inputTraceMsPerParse ?? 'n/a'
      } ms (${((size.inputTraceShare ?? 0) * 100).toFixed(1)}%), set ${((size.setShare ?? 0) * 100).toFixed(1)}%`,
    );
  }

  if (sizeExponents !== undefined) {
    console.log(
      `across 1000 to 4000 tokens: parse exponent ${sizeExponents.parseExponent}, input-trace exponent ${
        sizeExponents.inputTraceExponent ?? 'n/a'
      }`,
    );
  }
}

main();

//endregion Entry point
