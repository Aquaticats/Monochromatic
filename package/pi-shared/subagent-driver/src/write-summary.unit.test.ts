/**
 A run's summary is the record of what it did and said, so a folder holds one and it is never written over: the writer answers a
 folder that already holds a summary with an error, and leaves the bytes that were there exactly as they were.

 The folder is the seam: each case writes its own folder under this run's scratch folder, and the summary it puts there first is a
 whole run's summary, so the bytes a case compares are the record of a run and not a placeholder.

 @module
 */

import {
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  SummaryAlreadyThereError,
  type RunSummary,
  refuseWhenTheFolderHoldsASummary,
  summaryIsThereProblemOf,
  writeSummary,
} from '../dist/final/node/index.mjs';

/**
 The name a finished run's summary carries in its folder, which is the last thing a run writes.
 */
const SUMMARY_FILE_NAME = 'summary.json';

/**
 A summary of a run that ended because its agent finished its turn, which is the record a case below must find untouched.
 */
const FIRST_SUMMARY: RunSummary = {
  endReason: 'idle',
  totalMilliseconds: 1_800_000,
  turns: 40,
  settled: 39,
  toolHistogram: { bash: 30, read: 60, },
  toolErrors: [],
  dialogs: [],
  sessionStats: { sessionFile: '/sessions/a.jsonl', cost: 1.25, },
  lastAssistantText: 'the review is written',
  retries: 0,
  rateLimitErrors: 0,
  piStartAttempts: 1,
};

/**
 A summary of a different run, which is what a writer asked to write over the first one would leave behind.
 */
const SECOND_SUMMARY: RunSummary = {
  ...FIRST_SUMMARY,
  endReason: 'wall-clock',
  turns: 120,
  lastAssistantText: 'the run ran out of its wall clock',
};

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-write-summary-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes a run folder holding one run's summary, as a run that finished left it.

 @param request - the folder and the summary to put there

 @example
 ```ts
 await writeARunFolder({ folder: '/runs/a-1', summary: FIRST_SUMMARY });
 ```
 */
async function writeARunFolder({
  folder,
  summary,
}: {
  readonly folder: string;
  readonly summary: RunSummary;
}): Promise<void> {
  await writeFile(
    join(
      folder,
      SUMMARY_FILE_NAME,
    ),
    `${JSON.stringify(
      summary,
      null,
      2,
    )}\n`,
    { encoding: 'utf8', },
  );
}

await describe({
  name: '',
  children: [
    describe({
      name: writeSummary.name,
      children: [
        it({
          name: 'writes the summary of a folder that holds none, as the whole of what a reader parses',
          fn: async () => {
            await using folder = await makeFolder();
            await writeSummary({
              runFolder: folder.path,
              summary: FIRST_SUMMARY,
            },);
            /**
             What the writer left, as a reader reads it.
             */
            const written = await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            );
            expect(JSON.parse(written,),).toEqual(FIRST_SUMMARY,);
            expect(await readdir(folder.path,),).toEqual([SUMMARY_FILE_NAME,],);
          },
        },),
        it({
          name: 'answers a folder that already holds a summary with an error, and leaves the summary that was there exactly as it was',
          fn: async () => {
            await using folder = await makeFolder();
            await writeARunFolder({
              folder: folder.path,
              summary: FIRST_SUMMARY,
            },);
            /**
             The summary that was there before the second writer asked, in the bytes a reader would parse.
             */
            const before = await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            );
            /**
             What the second writer answered.
             */
            let caught: unknown;
            try {
              await writeSummary({
                runFolder: folder.path,
                summary: SECOND_SUMMARY,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SummaryAlreadyThereError,);
            expect((caught as Error).message,).toContain(folder.path,);
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(before,);
          },
        },),
        it({
          name: 'writes no summary and refuses a folder that already holds one at the start of a run',
          fn: async () => {
            await using folder = await makeFolder();
            await writeARunFolder({
              folder: folder.path,
              summary: FIRST_SUMMARY,
            },);
            /**
             What refusing a folder that holds a summary answered.
             */
            let caught: unknown;
            try {
              await refuseWhenTheFolderHoldsASummary({ runFolder: folder.path, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SummaryAlreadyThereError,);
          },
        },),
        it({
          name: 'refuses nothing for a folder that holds no summary, and writes none of its own',
          fn: async () => {
            await using folder = await makeFolder();
            await refuseWhenTheFolderHoldsASummary({ runFolder: folder.path, },);
            expect(await readdir(folder.path,),).toEqual([],);
          },
        },),
        it({
          name: 'names the summary that is kept, the folder it is in, and both ways of giving a run a folder of its own',
          fn: async () => {
            /**
             What a run answers a folder that holds a summary with.
             */
            const problem = summaryIsThereProblemOf({ runFolder: '/runs/a-5', },);
            expect(problem,).toContain(join(
              '/runs/a-5',
              SUMMARY_FILE_NAME,
            ),);
            expect(problem,).toContain('--run=<number>',);
            expect(problem,).toContain('runs folder of its own',);
          },
        },),
      ],
    },),
  ],
},);
