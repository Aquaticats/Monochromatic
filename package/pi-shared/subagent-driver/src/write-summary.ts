/**
 Writes the summary so that a reader sees all of it or none, and so that a summary that is already there is never written over.

 The summary is the record of what a run did and said: its end reason, what Pi left, and the last thing the agent said. A run that
 wrote into another run's folder would lose that record, which is why the file appears whole or not at all, why a name already taken
 is an error and never a replacement, and why a run asks its folder before it starts rather than after it has worked.

 @module
 */

import {
  link,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  SUMMARY_FILE_NAME,
  isThere,
} from './newest-finished-run-of-a-name.ts';
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
 The code the operating system gives a name that is already taken.
 */
const ALREADY_THERE_CODE = 'EEXIST';

/**
 The name the summary is written under before it takes its own name, and the run's own process number, so two runs that were given
 one folder never publish the bytes the other wrote.
 */
const UNFINISHED_NAME = '.summary.json.unfinished';

/**
 A run whose folder already holds the summary of another run.
 */
export class SummaryAlreadyThereError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SummaryAlreadyThereError';
}

/**
 Says what a run answers when the folder it was given holds a summary, in the words a reader acts on.

 @param runFolder - the folder the run was given

 @returns the message, naming the summary that is kept and how to give the run a folder of its own

 @example
 ```ts
 const problem = summaryIsThereProblemOf({ runFolder: '/runs/a-5', },);
 ```
 */
export function summaryIsThereProblemOf({ runFolder, }: { readonly runFolder: string },): string {
  return `${join(
    runFolder,
    SUMMARY_FILE_NAME,
  )} is the summary of a run that already finished, and a run never writes over one. This run stops before it writes anything, so that summary is kept. Start a run with no --run=<number>, so its folder takes the next number no folder of that name holds, or name a runs folder of its own.`;
}

/**
 Refuses a run folder that already holds a summary, before the run starts.

 @param runFolder - the folder the run was given

 @throws SummaryAlreadyThereError when the folder holds a summary

 @example
 ```ts
 await refuseWhenTheFolderHoldsASummary({ runFolder: '/runs/a-5', },);
 ```
 */
export async function refuseWhenTheFolderHoldsASummary({ runFolder, }: { readonly runFolder: string },): Promise<void> {
  if (await isThere(join(
    runFolder,
    SUMMARY_FILE_NAME,
  ),)) {
    throw new SummaryAlreadyThereError(summaryIsThereProblemOf({ runFolder, },),);
  }
}

/**
 Writes a run's summary, appearing whole and last.

 The summary takes its own name by one link of a file already written whole, so a name that is taken makes the link fail rather
 than replace the summary there: the file system must support hard links for that, which every file system a runs folder has been
 put on does.

 @param runFolder - folder the run writes to

 @param summary - what to write

 @param mask - the masking every string of it goes through

 @throws SummaryAlreadyThereError when the folder already holds a summary, which is left exactly as it was

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
   Where the summary is written first, under a name of this run's own.
   */
  const unfinishedFile = join(
    runFolder,
    `${UNFINISHED_NAME}-${String(process.pid,)}`,
  );
  /**
   The file the summary is written under before it takes its own name, removed however this function ends: when the summary took its
   own name, and when that name was already taken and this run refused to replace it.
   */
  await using theUnfinishedFile = {
    async [Symbol.asyncDispose](): Promise<void> {
      // Forced, so a removal that finds no file cannot take the place of the failure that kept this function from publishing.
      await rm(
        unfinishedFile,
        { force: true, },
      );
    },
  };
  await writeFile(
    unfinishedFile,
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
  try {
    await link(
      unfinishedFile,
      join(
        runFolder,
        SUMMARY_FILE_NAME,
      ),
    );
  }
  catch (error) {
    throw (errorCodeOf(error,) === ALREADY_THERE_CODE)
      ? new SummaryAlreadyThereError(summaryIsThereProblemOf({ runFolder, },),)
      : error;
  }
}
