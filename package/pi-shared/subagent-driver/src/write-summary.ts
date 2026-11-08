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

/**
 Writes a run's summary, appearing whole and last.

 @param runFolder - folder the run writes to

 @param summary - what to write

 @example
 ```ts
 await writeSummary({ runFolder: '/run', summary, },);
 ```
 */
export async function writeSummary({
  runFolder,
  summary,
}: {
  readonly runFolder: string;
  readonly summary: RunSummary
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
      summary,
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
