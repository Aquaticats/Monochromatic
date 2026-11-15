/**
 Reading an ordinary file: a named pipe or a device is refused rather than waited on, and the bound covers the whole of the read,
 from before the file is opened, so an open or a metadata read that never answers is answered by the bound.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NotAnOrdinaryFileError,
  type OpenAFile,
  ReadThatTookTooLongError,
  readOrdinaryFile,
  readOrdinaryFileWithin,
} from '../dist/final/node/index.mjs';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 How long the bound in these tests is, in milliseconds, which is far shorter than any file here takes to read.
 */
const SHORT_BOUND_MILLISECONDS = 100;

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-read-ordinary-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Opens a file that never answers, which is what a filesystem whose server stopped looks like from here.

 @returns a promise that never settles

 @example
 ```ts
 const openFile = neverAnsweringOpen();
 ```
 */
function neverAnsweringOpen(): OpenAFile {
  return async function held() {
    // The open is asked for and never answered, which is what a filesystem whose server stopped looks like from here.
    return await new Promise<never>(function never(_resolve) {
      void _resolve;
    },);
  };
}

/**
 Answers what it waited for, after the test's own bound has passed.

 @param request - how long to wait

 @param request.milliseconds - the test's bound

 @returns why the test stopped waiting

 @example
 ```ts
 const answer = await stillWaitingAfter({ milliseconds: 2_000, });
 ```
 */
async function stillWaitingAfter({ milliseconds, }: { readonly milliseconds: number },): Promise<{ readonly stillWaiting: true }> {
  await wait(milliseconds,);
  return { stillWaiting: true, };
}

/**
 Asks for a read and answers either what it gave or that it was still waiting when the test stopped waiting.

 @param read - the read to ask for

 @returns its answer, or the fact that the test's own bound passed first

 @example
 ```ts
 const answer = await boundedAnswer(async () => await readOrdinaryFile('/x'),);
 ```
 */
async function boundedAnswer(read: () => Promise<string>,): Promise<{
  readonly text?: string;
  readonly error?: unknown;
  readonly stillWaiting?: true;
}> {
  /**
   The test's own bound, longer than the read's, so a read the package bounds answers first.
   */
  const testBound = stillWaitingAfter({ milliseconds: 2_000, },);
  return await Promise.race([
    (async function attempt() {
      try {
        return { text: await read(), };
      }
      catch (error) {
        return { error, };
      }
    })(),
    testBound,
  ],);
}

await describe({
  name: '',
  children: [
    describe({
      name: readOrdinaryFile.name,
      children: [
        it({
          name: 'reads an ordinary file whole',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A file a run may leave, of a few kilobytes.
             */
            const file = join(folder.path, 'summary.json',);
            await writeFile(file, '{"endReason":"idle"}\n',);
            expect(await readOrdinaryFile(file,),).toBe('{"endReason":"idle"}\n',);
          },
        },),
        it({
          name: 'refuses a named pipe at once, naming it, instead of waiting for a writer that may never come',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A named pipe where a file a run reads is expected.
             */
            const pipe = join(folder.path, 'summary.json',);
            await execFileAsync('mkfifo', [pipe,], { timeout: 5_000, },);
            /**
             What the read answered, or the fact that it was still waiting when the test stopped waiting.
             */
            const answer = await boundedAnswer(async function read() {
              return await readOrdinaryFile(pipe,);
            },);
            expect(answer,).toHaveProperty('error',);
            expect(answer.error,).toBeInstanceOf(NotAnOrdinaryFileError,);
            expect(String(answer.error,),).toContain(pipe,);
          },
        },),
      ],
    },),
    describe({
      name: readOrdinaryFileWithin.name,
      children: [
        it({
          name: 'bounds the open itself, answering its own error for an open that never returns',
          fn: async () => {
            /**
             What the read answered, with an open that never returns and a bound far shorter than the test's own.
             */
            const answer = await boundedAnswer(async function read() {
              return await readOrdinaryFileWithin({
                path: '/a/file/whose/open/never/answers',
                milliseconds: SHORT_BOUND_MILLISECONDS,
                openFile: neverAnsweringOpen(),
              },);
            },);
            // The bound was made before the open, so it answers for the open too: without that, this read would wait forever.
            expect(answer,).toHaveProperty('error',);
            expect(answer.error,).toBeInstanceOf(ReadThatTookTooLongError,);
            expect(String(answer.error,),).toContain('/a/file/whose/open/never/answers',);
            expect(String(answer.error,),).toContain(String(SHORT_BOUND_MILLISECONDS,),);
            expect(answer.stillWaiting,).toBeUndefined();
          },
        },),
        it({
          name: 'reads a file inside the bound a run reads its own records with',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A file a run may leave.
             */
            const file = join(folder.path, 'feed.jsonl',);
            await writeFile(file, '{"t":0,"kind":"turn_start","n":1}\n',);
            expect(await readOrdinaryFileWithin({ path: file, },),).toBe('{"t":0,"kind":"turn_start","n":1}\n',);
          },
        },),
      ],
    },),
  ],
},);
