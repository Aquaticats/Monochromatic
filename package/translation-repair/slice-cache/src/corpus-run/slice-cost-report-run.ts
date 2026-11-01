import { readFile, } from 'node:fs/promises';

import {
  isMissingPathError,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { readSliceCosts, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  bucketSliceCostsBySize,
  printSliceCostBucket,
} from './slice-cost-bands.ts';
import { printSliceCostLanes, } from './slice-cost-lanes.ts';
import { printSliceCostSpread, } from './slice-cost-spread.ts';

//region Slice cost report run
// Reads the per-slice cost telemetry a pass writes, and answers the one
// question it was written for: whether a slice's cost scales with its size.
//
// WHY IT EXISTS AT ALL: `slice-cost-log.ts` writes a line per slice per lane and
// `slice-cost-read.ts` parses them back, and until now NOTHING CALLED THE
// READER. Telemetry written and never read is a failure this package has named
// before, and it is worse than no telemetry, because it looks like the question
// is covered.
//
// THE QUESTION. The two-lane cost measurement of 2026-08-17 found that
// per-slice cost FALLS as entries get larger,
// 7.07 minutes a slice on the smallest entry against 2.47 on the largest, and
// could not say why. Two explanations fit that equally: larger entries have
// larger slices and size is what costs, or cost is mostly a fixed per-slice
// overhead that a larger entry amortises over more content. They imply opposite
// remedies. If size dominates, slicing differently changes the bill; if overhead
// dominates, only asking fewer times does.
//
// THE READING THAT SEPARATES THEM is cost per CHARACTER against slice size. Under
// a size-dominated cost that figure is flat; under a fixed overhead it falls
// steeply as slices grow, because the same overhead is divided by more
// characters.
//
// COSTS NOTHING. It reads a log file and prints.


/**
 Reads the log the command was named.

 @param path - log named on the command line

 @returns The log's whole text

 @throws StatedRefusalError when nothing is at the path, which is a mistyped
 name rather than a fault of the command

 @example
 ```ts
 const log = await logTextAt({ path: '/runs/pass.log', },);
 ```
 */
async function logTextAt({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!isMissingPathError({ error, },)) {
      throw error;
    }
    throw new StatedRefusalError({
      says: `${path} is not there; name a pass log that exists: slice-cost-report <log file>`,
      cause: error,
    },);
  }
}

/**
 Reads a pass log and reports what its slices cost.

 @param line - the report's command line, read whole by `reportingRefusals`,
 which refuses it when no log is named

 @throws StatedRefusalError when the log is named as an empty argument, or
 nothing is at the path named

 @example
 ```ts
 await reportSliceCost({ line, },);
 ```
 */
export async function reportSliceCost(
  { line, }: { readonly line: CommandLineOf<'slice-cost-report'>; },
): Promise<void> {
  /**
   Log to read, named on the command line.
   */
  const [path = '',] = line.positionals;
  if (path === '')
    throw new StatedRefusalError({ says: 'name a log file, not an empty argument: slice-cost-report <log file>', },);

  /**
   Everything its cost lines said.
   */
  const {
    rows,
    dropped,
  } = readSliceCosts({ log: await logTextAt({ path, },), },);

  console.log(
    `${path}\n${String(rows.length,)} cost ${
      wordForCount({
        count: rows.length,
        one: 'line',
        many: 'lines',
      },)
    }, ${String(dropped.length,)} dropped\n`,
  );
  if (rows.length === 0) {
    console.log('NOTHING TO READ YET. A pass writes these as it goes, so an empty'
      + ' reading means the run has not finished a slice rather than that slices are free.',);
    return;
  }

  console.log('WHAT A SLICE COSTS, BY THE SIZE OF ITS ORIGINAL',);
  /**
   Every band, smallest first.
   */
  const buckets = bucketSliceCostsBySize({ rows, },);
  buckets.forEach(function show(bucket,): void {
    printSliceCostBucket({ bucket, },);
  },);

  printSliceCostSpread({ rows, },);

  console.log(
    '\nREAD THE ms/char COLUMN. Flat across bands means size drives the cost and'
      + ' slicing differently changes the bill. Falling steeply as slices grow means'
      + ' a fixed per-slice overhead divided by more characters, and only asking'
      + ' fewer times changes anything.',
  );

  printSliceCostLanes({ rows, },);
}

//endregion Slice cost report run
