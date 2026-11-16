/**
 A run's summary is the record of what it did and said, so a folder holds one and it is never written over: the writer answers a
 folder that already holds a summary with an error, and leaves the bytes that were there exactly as they were.

 The folder is the seam: each case writes its own folder under this run's scratch folder, and the summary it puts there first is a
 whole run's summary, so the bytes a case compares are the record of a run and not a placeholder. The writer of a summary is also
 run as its own process and killed between two of its steps, because a process may be killed between any two instructions and the
 state such a kill leaves is one a later writer has to survive.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
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
  type RunSummary,
  type SummaryWriteStep,
  RunFolderHoldsAnEarlierAttemptError,
  type BetweenSummaryWriteSteps,
  earlierAttemptProblemIn,
  earlierAttemptProblemOf,
  refuseWhenTheFolderHoldsAnEarlierAttempt,
  summaryTheStoppingWriterPublishes,
  writeSummary,
} from '../dist/final/node/index.mjs';

/**
 The name a finished run's summary carries in its folder, which is the last thing a run writes.
 */
const SUMMARY_FILE_NAME = 'summary.json';

/**
 The program that writes one summary and stops where it is told to, run as a process this test kills.
 */
const STOPPING_WRITER = new URL('writer-of-a-summary-that-stops-between-its-steps-for-tests.ts', import.meta.url,).pathname;

/**
 How long a test waits for that writer to reach its step, and for the kill of it to be reaped.
 */
const PATIENCE_MILLISECONDS = 30_000;

/**
 How large a second writer's summary is, so that its writing still runs while the first writer publishes: both calls are made before
 either is awaited, and a write this size cannot finish before the other's does.
 */
const SECOND_WRITER_BYTES = 32 * 1_024 * 1_024;

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
    summaryTextOf({ summary, },),
    { encoding: 'utf8', },
  );
}

/**
 Reads the bytes a summary is written as, which is the whole of what a reader parses.

 @param request - the summary a writer would publish

 @returns its text, byte for byte

 @example
 ```ts
 const text = summaryTextOf({ summary: FIRST_SUMMARY });
 ```
 */
function summaryTextOf({ summary, }: { readonly summary: RunSummary },): string {
  return `${JSON.stringify(
    summary,
    null,
    2,
  )}\n`;
}

/**
 A thing a test waits for and keeps when it chooses.

 @returns the promise, and what keeps it

 @example
 ```ts
 const gate = aThingToWaitFor();
 gate.keep();
 await gate.waitedFor;
 ```
 */
function aThingToWaitFor(): {
  readonly waitedFor: Promise<void>;
  readonly keep: () => void;
} {
  /**
   What keeps the promise, which the promise itself fills as soon as it is made.
   */
  const settle: { keep?: () => void } = {};
  /**
   The promise itself.
   */
  const waitedFor = new Promise<void>(function holding(resolve,) {
    settle.keep = resolve;
  },);
  return {
    waitedFor,
    keep: function keepIt() {
      settle.keep?.();
    },
  };
}

/**
 Writes one summary as its own process, stops it at one of its steps, and kills it there: the folder is left in the state a kill
 between two instructions leaves.

 @param request - the folder, the step to stop at, and the marker the writer's last message carries

 @example
 ```ts
 await killTheWriterAt({ folder, step: 'the-summary-has-taken-its-name', marker: 'KILLED-WRITER' });
 ```
 */
async function killTheWriterAt({
  folder,
  step,
  marker,
}: {
  readonly folder: string;
  readonly step: SummaryWriteStep;
  readonly marker: string;
},): Promise<void> {
  /**
   The writer, which is a process this test started and was given the number of.
   */
  const writer = spawn(
    process.execPath,
    [
      STOPPING_WRITER,
      folder,
      step,
      marker,
    ],
    { stdio: [
      'ignore',
      'ignore',
      'pipe',
    ], },
  );
  /**
   What the writer says on its standard error, which says whether it reached the step.
   */
  const reported = writer.stderr;
  if (reported === null) {
    throw new Error('the writer was started without a standard error this test can read',);
  }
  reported.setEncoding('utf8',);
  /**
   The line the writer prints when it reaches the step it was told to stop at.
   */
  const stoppingLine = `stopped-at ${step}`;
  /**
   What the writer has said, in the pieces it said them in.
   */
  const heard: string[] = [];
  for await (const word of reported) {
    heard.push(String(word,),);
    if (heard.join('',).includes(stoppingLine,)) {
      break;
    }
  }
  if (!heard.join('',).includes(stoppingLine,)) {
    throw new Error(`the writer ended before it reached ${step}: ${heard.join('',)}`,);
  }
  writer.kill('SIGKILL',);
  await once(
    writer,
    'exit',
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
            expect(caught,).toBeInstanceOf(RunFolderHoldsAnEarlierAttemptError,);
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
              await refuseWhenTheFolderHoldsAnEarlierAttempt({ runFolder: folder.path, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RunFolderHoldsAnEarlierAttemptError,);
          },
        },),
        it({
          name: 'refuses nothing for a folder that holds no summary, and writes none of its own',
          fn: async () => {
            await using folder = await makeFolder();
            await refuseWhenTheFolderHoldsAnEarlierAttempt({ runFolder: folder.path, },);
            expect(await readdir(folder.path,),).toEqual([],);
          },
        },),
        it({
          name: 'names the folder, the records an earlier attempt left in it, and every way of giving a run a folder of its own',
          fn: async () => {
            /**
             What a run answers a folder that holds what an earlier attempt left with.
             */
            const problem = earlierAttemptProblemOf({
              runFolder: '/runs/a-5',
              records: ['feed.jsonl', 'stderr.log',],
            },);
            expect(problem,).toContain('/runs/a-5',);
            expect(problem,).toContain('feed.jsonl',);
            expect(problem,).toContain('stderr.log',);
            expect(problem,).toContain('--run=<number>',);
            expect(problem,).toContain('runs folder of its own',);
            expect(problem,).toContain('move that folder aside',);
          },
        },),
        it({
          name: 'counts the records it does not name, so a folder of many is still one line',
          fn: async () => {
            /**
             What a run answers a folder holding more records than a refusal names.
             */
            const problem = earlierAttemptProblemOf({
              runFolder: '/runs/a-5',
              records: [
                'record-1',
                'record-2',
                'record-3',
                'record-4',
                'record-5',
                'record-6',
                'record-7',
              ],
            },);
            expect(problem,).toContain('record-5',);
            expect(problem,).not.toContain('record-6',);
            expect(problem,).toContain('and 2 more',);
          },
        },),
        it({
          name: 'publishes the bytes of the writer that took the name, when two writers ask for one folder at once',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The summary of the second writer, which is large enough that its own writing still runs while the first publishes:
             both calls are made before either is awaited.
             */
            const secondSummary: RunSummary = {
              ...FIRST_SUMMARY,
              lastAssistantText: 'the second writer'.padEnd(SECOND_WRITER_BYTES, '2',),
            };
            /**
             What each writer answered, in the order the two were made.
             */
            const answers = await Promise.allSettled([
              writeSummary({
                runFolder: folder.path,
                summary: FIRST_SUMMARY,
              },),
              writeSummary({
                runFolder: folder.path,
                summary: secondSummary,
              },),
            ],);
            /**
             Which writer took the summary's name, which is one writer only.
             */
            const took = answers.flatMap(function tookIt(answer, index,) {
              return (answer.status === 'fulfilled') ? [index,] : [];
            },);
            expect(took,).toHaveLength(1,);
            /**
             The summary the writer that took the name asked to publish.
             */
            const askedToPublish = (answers[0].status === 'fulfilled') ? FIRST_SUMMARY : secondSummary;
            /**
             What that writer published.
             */
            const published = await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            );
            expect(published,).toBe(summaryTextOf({ summary: askedToPublish, },),);
          },
        },),
        it({
          name: 'publishes the later writer bytes whole, and leaves the temporary file a writer killed before it took the name exactly as it was',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The marker the killed writer's last message carries, so a reader of the bytes says whose they are.
             */
            const killedMarker = 'KILLED-BEFORE-TAKING-THE-NAME';
            await killTheWriterAt({
              folder: folder.path,
              step: 'the-temporary-file-is-whole',
              marker: killedMarker,
            },);
            /**
             What the killed writer left: one file of its own, whole, under a name that is not the summary's.
             */
            const left = await readdir(folder.path,);
            expect(left,).toHaveLength(1,);
            const leftBehind = left[0] ?? '';
            expect(leftBehind,).not.toBe(SUMMARY_FILE_NAME,);
            /**
             What the killed writer left in that file.
             */
            const killedText = await readFile(
              join(
                folder.path,
                leftBehind,
              ),
              'utf8',
            );
            expect(killedText,).toBe(summaryTextOf({ summary: summaryTheStoppingWriterPublishes({ marker: killedMarker, },), },),);
            await writeSummary({
              runFolder: folder.path,
              summary: SECOND_SUMMARY,
            },);
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(summaryTextOf({ summary: SECOND_SUMMARY, },),);
            // Nothing opened the killed writer's file for truncation, so its bytes are still the ones it left.
            expect(await readFile(
              join(
                folder.path,
                leftBehind,
              ),
              'utf8',
            ),).toBe(killedText,);
          },
        },),
        it({
          name: 'answers the later writer with an error, and leaves the summary a writer killed after it took the name byte for byte as it was',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The marker the killed writer's last message carries, so a reader of the bytes says whose they are.
             */
            const killedMarker = 'KILLED-AFTER-TAKING-THE-NAME';
            await killTheWriterAt({
              folder: folder.path,
              step: 'the-summary-has-taken-its-name',
              marker: killedMarker,
            },);
            /**
             What the killed writer published, which is its own summary whole.
             */
            const killedText = summaryTextOf({ summary: summaryTheStoppingWriterPublishes({ marker: killedMarker, },), },);
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(killedText,);
            /**
             What the later writer answered.
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
            expect(caught,).toBeInstanceOf(RunFolderHoldsAnEarlierAttemptError,);
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(killedText,);
          },
        },),
        it({
          name: 'leaves the published summary byte for byte as it was, when a later writer in a process with the writer process number asks for the folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The step the first writer is held at, a step a kill may land on: its summary has taken its name while the temporary
             one is still a second name of it.
             */
            const reachedTheStep = aThingToWaitFor();
            /**
             What lets that writer go on, once the later writer has asked.
             */
            const letItGoOn = aThingToWaitFor();
            /**
             The first writer, which is held where a kill would have left it.
             */
            const held = writeSummary({
              runFolder: folder.path,
              summary: FIRST_SUMMARY,
              betweenSteps: async function holdTheWriter(step,) {
                if (step !== 'the-summary-has-taken-its-name') {
                  return;
                }
                reachedTheStep.keep();
                await letItGoOn.waitedFor;
              },
            },);
            await reachedTheStep.waitedFor;
            /**
             What the held writer published before the later one asked.
             */
            const publishedByTheHeldWriter = summaryTextOf({ summary: FIRST_SUMMARY, },);
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(publishedByTheHeldWriter,);
            /**
             What the later writer answered.
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
            expect(caught,).toBeInstanceOf(RunFolderHoldsAnEarlierAttemptError,);
            // The later writer was in a process whose number follows from the same place the held writer's temporary name did, so a
            // name derived from the process lets one writer write through the other's published summary.
            expect(await readFile(
              join(
                folder.path,
                SUMMARY_FILE_NAME,
              ),
              'utf8',
            ),).toBe(publishedByTheHeldWriter,);
            letItGoOn.keep();
            await held;
          },
        },),
      ],
    },),
  ],
},);
