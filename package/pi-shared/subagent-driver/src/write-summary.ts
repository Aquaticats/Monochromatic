/**
 Writes the summary so that a reader sees all of it or none, and so that nothing another attempt wrote in the folder is written over.

 The summary is the record of what a run did and said: its end reason, what Pi left, and the last thing the agent said. A run that
 wrote into another run's folder would lose that record, which is why the file appears whole or not at all, why a name already taken
 is an error and never a replacement, and why a run asks what its folder already holds before it starts rather than after it has
 worked.

 A process may be killed between any two instructions, so the states such a kill leaves are ordinary ones: a folder holding a whole
 summary with no second name of it, or holding a summary and the temporary name it was published under. Neither is one a later write
 may damage, which is why every write names its temporary file with bytes no other write can derive and creates it exclusively.

 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  link,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { SUMMARY_FILE_NAME, } from './newest-finished-run-of-a-name.ts';
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
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The name the summary is written under before it takes its own name.
 */
const UNFINISHED_NAME = '.summary.json.unfinished';

/**
 How many random bytes name that file, which is what keeps two writes of one folder apart: a name no other write can derive is a name no
 other write opens, and one it alone can remove.
 */
const UNFINISHED_NAME_BYTES = 8;

/**
 How many of a folder's records a refusal names, which is what a reader acts on; the rest are counted.
 */
const NAMED_RECORDS = 5;

/**
 One step of writing a summary, named where a kill may land between two instructions: the temporary file holds the whole summary but
 has no name a reader reads, or the summary has taken its own name while the temporary one is still a second name of it.
 */
export type SummaryWriteStep =
  | 'the-temporary-file-is-whole'
    | 'the-summary-has-taken-its-name';

/**
 What happens between two steps of one write, which a test takes over to stop the writer where a kill would leave it.
 */
export type BetweenSummaryWriteSteps = (step: SummaryWriteStep,) => Promise<void>;

/**
 A run whose folder already holds what an earlier attempt at a run of that name left there.
 */
export class RunFolderHoldsAnEarlierAttemptError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'RunFolderHoldsAnEarlierAttemptError';
}

/**
 Names the records a refusal found, naming a few of them and counting the rest.

 @param records - the names a run folder holds

 @returns the names, and how many more there are
 */
function namedRecordsOf(records: readonly string[],): string {
  /**
   The names a reader acts on first.
   */
  const named = records.slice(
    0,
    NAMED_RECORDS,
  )
    .join(', ',);
  return (records.length > NAMED_RECORDS)
    ? `${named}, and ${String(records.length - NAMED_RECORDS,)} more`
    : named;
}

/**
 Says what a run answers when the folder it was given holds what an earlier attempt left there, in the words a reader acts on.

 @param runFolder - the folder the run was given

 @param records - the names the folder holds

 @returns the message, naming the folder, the records that are kept, and every way of giving the run a folder of its own

 @example
 ```ts
 const problem = earlierAttemptProblemOf({ runFolder: '/runs/a-5', records: ['stderr.log',], },);
 ```
 */
export function earlierAttemptProblemOf({
  runFolder,
  records,
}: {
  readonly runFolder: string;
  readonly records: readonly string[];
},): string {
  return `${runFolder} holds what an earlier attempt at a run of this name left there (${namedRecordsOf(records,)}), and a run never writes over another attempt's records. This run stops before it writes anything, so those records are kept. Start a run with no --run=<number>, so its folder takes the next number no folder of that name holds; name a runs folder of its own; or move that folder aside if nobody needs what it holds any more.`;
}

/**
 Reads what an earlier attempt left in a run folder: every entry but nothing at all for a folder that is not there.

 @param runFolder - the folder the run was given

 @returns the names of the entries, in the order a reader reads them

 @example
 ```ts
 const records = await recordsInTheRunFolderOf({ runFolder: '/runs/a-5', },);
 ```
 */
async function recordsInTheRunFolderOf({ runFolder, }: { readonly runFolder: string },): Promise<readonly string[]> {
  try {
    return (await readdir(runFolder,)).toSorted();
  }
  catch (error) {
    // A folder that is not there holds nothing, which is what a run folder made for this run looks like before the run makes it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Says what stops a run from writing into the folder it was given, if anything does.

 A run folder is a run's own: a folder that holds anything holds the record of an attempt at this run, whether that attempt finished
 or was killed, and a run that wrote there would write over it. Both the launcher's checks and the driver ask this before anything is
 made, so a start that would write over another attempt's records is refused rather than spent.

 @param runFolder - the folder this run would write into

 @returns the problem in words, or nothing when the folder holds nothing

 @example
 ```ts
 const { problem } = await earlierAttemptProblemIn({ runFolder: '/runs/a-5', },);
 ```
 */
export async function earlierAttemptProblemIn({ runFolder, }: { readonly runFolder: string },): Promise<{ readonly problem?: string }> {
  /**
   What the folder holds, which is nothing for a folder a run may write into.
   */
  const records = await recordsInTheRunFolderOf({ runFolder, },);
  return (records.length === 0) ? {} : { problem: earlierAttemptProblemOf({
    runFolder,
    records,
  },), };
}

/**
 Refuses a run folder that holds what an earlier attempt left there, before the run starts.

 @param runFolder - the folder the run was given

 @throws RunFolderHoldsAnEarlierAttemptError when the folder holds anything

 @example
 ```ts
 await refuseWhenTheFolderHoldsAnEarlierAttempt({ runFolder: '/runs/a-5', },);
 ```
 */
export async function refuseWhenTheFolderHoldsAnEarlierAttempt({ runFolder, }: { readonly runFolder: string },): Promise<void> {
  /**
   What stops this run from writing there, when anything does.
   */
  const { problem, } = await earlierAttemptProblemIn({ runFolder, },);
  if (problem !== undefined) {
    throw new RunFolderHoldsAnEarlierAttemptError(problem,);
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

 @param betweenSteps - what happens between two steps of the write, which a test takes over to stop a writer where a kill would
 leave it: a process may be killed between any two instructions, so the state such a kill leaves is an ordinary one

 @throws RunFolderHoldsAnEarlierAttemptError when the folder already holds a summary, which is left exactly as it was

 @example
 ```ts
 await writeSummary({ runFolder: '/run', summary, },);
 ```
 */
export async function writeSummary({
  runFolder,
  summary,
  mask = MASKING_NOTHING,
  betweenSteps,
}: {
  readonly runFolder: string;
  readonly summary: RunSummary;
  readonly mask?: TextMasking;
  readonly betweenSteps?: BetweenSummaryWriteSteps;
},): Promise<void> {
  /**
   Where the summary is written first, under a name no other write of any folder can derive, so no other write opens it.
   */
  const unfinishedFile = join(
    runFolder,
    `${UNFINISHED_NAME}-${randomBytes(
      UNFINISHED_NAME_BYTES,
    )
      .toString('hex',)}`,
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
    // Created, never opened: a name this write did not just make is a name another write left, and truncating that one would write
    // through a summary that is already published.
    { flag: 'wx', },
  );
  await betweenSteps?.('the-temporary-file-is-whole',);
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
    if (errorCodeOf(error,) !== ALREADY_THERE_CODE) {
      throw error;
    }
    /**
     What the folder held when this write found the summary's name taken.
     */
    const records = await recordsInTheRunFolderOf({ runFolder, },);
    throw new RunFolderHoldsAnEarlierAttemptError(earlierAttemptProblemOf({
      runFolder,
      records,
    },),);
  }
  await betweenSteps?.('the-summary-has-taken-its-name',);
}
