#!/usr/bin/env node
/**
 `page-weight` CLI.
 
 Walks a dist directory of static HTML, computes per-page transfer size
 (HTML plus every asset a browser would fetch to render it), and prints
 min/max/mean/median summary statistics.
 
 Wire sizes prefer a `.zst` companion when one exists, matching the
 precompressed variant served by file servers that speak zstd
 content-encoding.
 
 @example
 ```sh
 node package/page-weight/cli/src/index.ts ./dist
 ```
 */
import { resolve, } from 'node:path';

import readdir from 'tiny-readdir-glob';

import {
  type PageWeight,
  weighPage,
} from './collect.ts';
import { summarize, } from './stats.ts';

import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core/ts';



/**
 Human-readable binary size units, ordered by magnitude.
 */
const BYTE_UNITS = [
  'B',
  'KiB',
  'MiB',
  'GiB',
];

/**
 Exit code for missing or invalid arguments.
 */
const EXIT_USAGE = 1;

/**
 Exit code for summary succeeded but some references did not resolve.
 */
const EXIT_MISSING_REFS = 2;

/**
 Result of scaling a raw byte count into a chosen unit; pairs the scaled
 numeric value with the `BYTE_UNITS` index that names the unit.
 */
type Scaled = {
  /**
   Byte count divided down by `BYTES_PER_KIB` repeatedly until the unit fits.
   */
  value: number;
  /**
   Position in `BYTE_UNITS` reached by the scaling loop.
   */
  unitIndex: number;
};

/**
 Formats a byte count as a human-readable string using binary prefixes,
 dividing by {@link BYTES_PER_KIB} repeatedly until the value fits a unit.
 
 Keeps output readable in a small terminal; precision is capped at 1 decimal
 place to avoid noise when byte counts differ trivially between runs.
 
 @param bytes - byte count
 
 @returns formatted string (e.g. `"1.2 MiB"`)
 
 @example
 ```ts
 humanBytes(1536); // '1.5 KiB'
 ```
 */
function humanBytes(bytes: number,): string {
  /**
   Scaled byte count and the `BYTE_UNITS` index it lives in after dividing by `BYTES_PER_KIB` until the unit fits.
   */
  const {
    value,
    unitIndex,
  } = (function scale(): Scaled {
    /**
     Working value mutated in-place by the scaling loop; renamed to avoid clashing with the destructured `value`.
     */
    let scaledValue = bytes;
    /**
     Working position in `BYTE_UNITS`; renamed to avoid clashing with the destructured `unitIndex`.
     */
    let scaledIndex = 0;
    while ((scaledValue >= BYTES_PER_KIB) && (scaledIndex < (BYTE_UNITS.length
      - 1))) {
      scaledValue /= BYTES_PER_KIB;
      scaledIndex += 1;
    }
    return {
      value: scaledValue,
      unitIndex: scaledIndex,
    };
  })();
  /**
   Decimals shown in the formatted output; raw bytes are reported as integers.
   */
  const precision = unitIndex === 0 ? 0 : 1;
  return `${value.toFixed(precision,)} ${BYTE_UNITS[unitIndex]}`;
}

//region Main

/**
 Command-line arguments after `page-weight` or direct `node` execution.
 */
const args = process.argv
  .slice(2,);

/**
 First positional argument: the dist directory to scan.
 */
const [distArg,] = args;
if (distArg === undefined) {
  console.error('usage: page-weight <dist-dir>',);
  process.exitCode = EXIT_USAGE;
  throw new Error('missing dist directory argument',);
}

/**
 Resolved absolute dist directory.
 */
const root = resolve(distArg,);

/**
 Discover every `.html` file inside the dist root.
 */
const found = await readdir(
  '**/*.html',
  { cwd: root, },
);
if (found.files
  .length
  === 0) {
  console.error(`no HTML files under ${root}`,);
  process.exitCode = EXIT_USAGE;
  throw new Error('empty page set',);
}

/**
 Weighed-page results, computed via {@link weighPage} concurrently.
 */
const weights: PageWeight[] = await Promise.all(
  found.files
    .map(function weigh(htmlPath: string,): Promise<PageWeight> {
      return weighPage({
        htmlPath,
        root,
      },);
    },),
);

/**
 Page rows sorted largest-first for readability.
 */
const sorted = weights.toSorted(function byTotalDescending(
  a: PageWeight,
  b: PageWeight,
): number {
  return b.totalBytes
    - a
      .totalBytes;
},);

/**
 Aggregate min/max/mean/median summary over page totals.
 */
const totals = weights.map(function pageTotal(entry: PageWeight,): number {
  return entry.totalBytes;
},);
/**
 Summary statistics for the page-weight distribution, computed via
 {@link summarize}.
 */
const stats = summarize(totals,);

/**
 Column width for the page path.
 */
const pageColumnWidth = Math.max(
  'page'.length,
  ...sorted.map(function pageLength(entry: PageWeight,): number {
    return entry.page
      .length;
  },),
);
/**
 Column width for the bytes column.
 */
const bytesColumnWidth = Math.max(
  'bytes'.length,
  ...sorted.map(function bytesLength(entry: PageWeight,): number {
    return humanBytes(entry.totalBytes,)
      .length;
  },),
);
/**
 Column width for the resource count.
 */
const resourcesColumnWidth = Math.max(
  'assets'.length,
  ...sorted.map(function resourcesLength(entry: PageWeight,): number {
    return String(entry.resourceCount,)
      .length;
  },),
);

/**
 Formatted header row.
 */
const header = [
  'page'.padEnd(pageColumnWidth,),
  'bytes'.padStart(bytesColumnWidth,),
  'assets'.padStart(resourcesColumnWidth,),
]
  .join('  ',);
console.log(header,);
for (const entry of sorted) {
  /**
   Pre-padded report row joined for terminal-aligned columns.
   */
  const row = [
    entry.page
      .padEnd(pageColumnWidth,),
    humanBytes(entry.totalBytes,)
      .padStart(bytesColumnWidth,),
    String(entry.resourceCount,)
      .padStart(resourcesColumnWidth,),
  ]
    .join('  ',);
  console.log(row,);
}

console.log('',);
console.log(`pages:  ${stats.count}`,);
console.log(`min:    ${humanBytes(stats.min,)}`,);
console.log(`max:    ${humanBytes(stats.max,)}`,);
console.log(`mean:   ${humanBytes(stats.mean,)}`,);
console.log(`median: ${humanBytes(stats.median,)}`,);

/**
 Unique references that could not be resolved to a file under the root.
 */
const missingAll = new Set<string>();
for (const entry of weights) {
  for (const ref of entry.missing)
    missingAll.add(ref,);
}
if (missingAll.size
  > 0) {
  console.error('',);
  console.error(
    `warn: ${missingAll.size} unresolved references (dead links or external)`,
  );
  for (const ref of [...missingAll,].toSorted())
    console.error(`  ${ref}`,);
  process.exitCode = EXIT_MISSING_REFS;
}

//endregion Main
