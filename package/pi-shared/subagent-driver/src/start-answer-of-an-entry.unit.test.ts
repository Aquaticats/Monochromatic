/**
 One entry's start answer: the value the batch that started it writes beside the runs, and the value the settling reads instead of
 the last line of what that batch printed.

 The runs folder is the only seam: a test writes its own folder and answers for nothing else.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
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
  NO_START_ANSWER,
  NO_SLOT_EXIT_STATUS,
  START_ANSWER_FILE_SUFFIX,
  type StartAnswerOfAnEntry,
  StartAnswerError,
  gaveUpWaitingForAPlaceIn,
  keepStartAnswersOfABatch,
  startAnswerFileOf,
  startAnswerOf,
  writeStartAnswer,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, removed when the test ends.

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-start-answer-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 One answer a batch would write for one entry.

 @param name - the entry's name

 @param status - the status its start ended with

 @returns the answer

 @example
 ```ts
 const answer = anAnswer({ name: 'a', status: 3 });
 ```
 */
function anAnswer({
  name,
  status,
}: {
  readonly name: string;
  readonly status?: number;
},): StartAnswerOfAnEntry {
  return {
    name,
    ...((status === undefined) ? {} : { status, }),
    gaveUpWaitingForAPlace: gaveUpWaitingForAPlaceIn(((status === undefined) ? {} : { status, })),
    line: `${name}: status ${String(status ?? 'none',)}`,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: gaveUpWaitingForAPlaceIn.name,
      children: [
        it({
          name: 'says a start gave up for want of a place only for the status the retrying start exits with at its deadline',
          fn: async () => {
            expect(gaveUpWaitingForAPlaceIn({ status: NO_SLOT_EXIT_STATUS, },),).toBe(true,);
            expect(gaveUpWaitingForAPlaceIn({ status: 0, },),).toBe(false,);
            expect(gaveUpWaitingForAPlaceIn({ status: 1, },),).toBe(false,);
            expect(gaveUpWaitingForAPlaceIn({},),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: writeStartAnswer.name,
      children: [
        it({
          name: 'writes the answer beside the runs, through a renamed file, so a reader never sees half of one',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Where the answer went.
             */
            const file = await writeStartAnswer({
              runsFolder: folder.path,
              answer: {
                ...anAnswer({
                  name: 'a',
                  status: NO_SLOT_EXIT_STATUS,
                },),
                keptOutputAt: '/runs/a-start-failure.log',
              },
            },);
            expect(file,).toBe(join(
              folder.path,
              `a${START_ANSWER_FILE_SUFFIX}`,
            ),);
            expect(
              JSON.parse(await readFile(
                file,
                'utf8',
              ),),
            ).toEqual({
              name: 'a',
              status: NO_SLOT_EXIT_STATUS,
              gaveUpWaitingForAPlace: true,
              line: 'a: status 3',
              keptOutputAt: '/runs/a-start-failure.log',
            },);
            /**
             What the write through left beside it, which is nothing once the rename is done.
             */
            expect((await stat(file,)).size,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'refuses a folder that cannot be written, naming the answer and the file',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder the answer cannot be written into.
             */
            const unwritable = join(folder.path, 'not-a-folder',);
            await writeFile(unwritable, 'a file where a folder would be\n',);
            /**
             What the write refused with.
             */
            let refused: unknown;
            try {
              await writeStartAnswer({
                runsFolder: unwritable,
                answer: anAnswer({ name: 'a', },),
              },);
            }
            catch (error) {
              refused = error;
            }
            expect(refused,).toBeInstanceOf(StartAnswerError,);
            expect((refused as Error).message,).toContain('the start answer of a could not be written',);
          },
        },),
      ],
    },),
    describe({
      name: startAnswerOf.name,
      children: [
        it({
          name: 'reads back what was written, and answers that no answer is there when the batch wrote none',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await startAnswerOf({
              runsFolder: folder.path,
              name: 'a',
            },),).toBe(NO_START_ANSWER,);
            await writeStartAnswer({
              runsFolder: folder.path,
              answer: anAnswer({
                name: 'a',
                status: 4,
              },),
            },);
            expect(await startAnswerOf({
              runsFolder: folder.path,
              name: 'a',
            },),).toEqual({
              name: 'a',
              status: 4,
              gaveUpWaitingForAPlace: false,
              line: 'a: status 4',
            },);
          },
        },),
        it({
          name: 'refuses a file that is not JSON, and one that is JSON but not an answer this package wrote',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The answer's path, which the test writes itself.
             */
            const file = startAnswerFileOf({
              runsFolder: folder.path,
              name: 'a',
            },);
            await writeFile(
              file,
              'not json\n',
            );
            /**
             What reading it refused with.
             */
            let notJson: unknown;
            try {
              await startAnswerOf({
                runsFolder: folder.path,
                name: 'a',
              },);
            }
            catch (error) {
              notJson = error;
            }
            expect(notJson,).toBeInstanceOf(StartAnswerError,);
            expect((notJson as Error).message,).toContain('is not JSON',);
            await writeFile(
              file,
              '{"name":"a"}\n',
            );
            /**
             What reading an answer that names no reason refused with.
             */
            let notAnAnswer: unknown;
            try {
              await startAnswerOf({
                runsFolder: folder.path,
                name: 'a',
              },);
            }
            catch (error) {
              notAnAnswer = error;
            }
            expect(notAnAnswer,).toBeInstanceOf(StartAnswerError,);
            expect((notAnAnswer as Error).message,).toContain('does not hold the answer of a start',);
          },
        },),
        it({
          name: 'refuses a file that is there and cannot be read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The answer's path, made unreadable for this account.
             */
            const file = startAnswerFileOf({
              runsFolder: folder.path,
              name: 'a',
            },);
            await writeFile(
              file,
              '{}\n',
            );
            await chmod(file, 0o000,);
            await using readableAgain = {
              async [Symbol.asyncDispose]() {
                await chmod(file, 0o600,);
              },
            };
            /**
             What reading it refused with.
             */
            let refused: unknown;
            try {
              await startAnswerOf({
                runsFolder: folder.path,
                name: 'a',
              },);
            }
            catch (error) {
              refused = error;
            }
            expect(refused,).toBeInstanceOf(StartAnswerError,);
            expect((refused as Error).message,).toContain('could not be read',);
          },
        },),
      ],
    },),
    describe({
      name: keepStartAnswersOfABatch.name,
      children: [
        it({
          name: 'writes one answer per entry, and reports an answer that cannot be written and goes on to the next',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder an answer cannot be written into, and one it can.
             */
            await mkdir(join(folder.path, 'runs',),);
            /**
             What the keeping reported.
             */
            const reported: string[] = [];
            await keepStartAnswersOfABatch({
              runsFolder: join(folder.path, 'not-a-folder',),
              answers: [
                anAnswer({
                  name: 'a',
                  status: 0,
                },),
                anAnswer({
                  name: 'b',
                  status: 1,
                },),
              ],
              report: function report(problem: string,) {
                reported.push(problem,);
              },
            },);
            expect(reported,).toHaveLength(2,);
            expect(reported[0],).toContain('its start answer could not be written',);
            /**
             What a folder that can be written holds once both answers went in.
             */
            const runsThatCanBeWritten = join(folder.path, 'runs',);
            await keepStartAnswersOfABatch({
              runsFolder: runsThatCanBeWritten,
              answers: [
                anAnswer({
                  name: 'a',
                  status: 0,
                },),
                anAnswer({
                  name: 'b',
                  status: 1,
                },),
              ],
              report: function report(problem: string,) {
                throw new Error(`nothing was expected to be reported: ${problem}`);
              },
            },);
            expect(await startAnswerOf({
              runsFolder: runsThatCanBeWritten,
              name: 'a',
            },),).not.toBe(NO_START_ANSWER,);
            expect(await startAnswerOf({
              runsFolder: runsThatCanBeWritten,
              name: 'b',
            },),).not.toBe(NO_START_ANSWER,);
          },
        },),
      ],
    },),
  ],
},);
