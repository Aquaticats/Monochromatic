/**
 Writes the summary so that a reader sees all of it or none.

 @module
 */

import {
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { RunSummary, } from './driver-types.ts';
import {
  MASKING_NOTHING,
  type TextMasking,
} from './secret-value-masking.ts';

/**
 A run's summary as the data it is written as, masked, or the little that is written when it could not be masked.

 @param summary - what to write

 @param mask - the masking

 @returns the summary as masked data, or the reason it was withheld and nothing of it
 */
function summaryAsData({
  summary,
  mask,
}: {
  readonly summary: RunSummary;
  readonly mask: TextMasking;
},): unknown {
  /**
   The summary as data.
   */
  const masked = mask.maskData(summary,);
  if ('value' in masked) {
    return masked.value;
  }
  return {
    endReason: summary.endReason,
    withheld: masked.cause,
  };
}

/**
 Writes a run's summary, appearing whole and last.

 @param runFolder - folder the run writes to

 @param summary - what to write

 @param mask - the masking every string of it goes through

 @example
 ```ts
 await writeSummary({ runFolder: '/run', summary, },);
 ```
 */
export async function writeSummary({
  runFolder,
  summary,
  mask = MASKING_NOTHING,
}: {
  readonly runFolder: string;
  readonly summary: RunSummary;
  readonly mask?: TextMasking;
},): Promise<void> {
  /**
   Where the summary is written first.
   */
  const unfinished = join(
    runFolder,
    '.summary.json.unfinished',
  );
  await writeFile(
    unfinished,
    `${JSON.stringify(
      // The summary carries what a tool error said, what the last reply said, and what Pi printed when a start failed, so it goes
      // through the same masking as every other file a run writes.
      summaryAsData({
        summary,
        mask,
      },),
      null,
      2,
    )}\n`,
  );
  await rename(
    unfinished,
    join(
      runFolder,
      'summary.json',
    ),
  );
}
