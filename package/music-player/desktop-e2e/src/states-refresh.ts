/**
 The checks of the refresh session (D113): the library and its true peaks follow the disk by themselves while the app runs.
 No key is pressed and nothing is clicked: the checks change files on disk and read what the app logs about it.
 
 @packageDocumentation
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { waveBytes, } from './library.ts';
import { pollUntilDeadline, } from './poll.ts';
import type { Session, } from './session.ts';
import { isLibraryReady, } from './states-conditions.ts';
import { SESSION_READY_MS, } from './timing.ts';

/**
 Log line of one finished decode whose decision was stored.
 */
const MEASURED_LINE = 'measured; upsert queued';

/**
 Log line of a window update that names files that cannot be played.
 */
const REPORTED_LINE = 'files that could not be opened are reported to the window';

/**
 How long the app gets to notice a change on disk and act on it, in milliseconds.
 */
const FOLLOWS_DISK_MS = 15_000;

/**
 How often the log is read while waiting, in milliseconds.
 */
const LOOK_EVERY_MS = 200;

/**
 Time given to a change that must not cause anything, in milliseconds.
 */
const QUIET_MS = 3_000;

/**
 Counts the log lines that contain a text.
 
 @param session - the session whose log is read
 
 @param text - text a line must contain
 
 @returns how many lines contain it
 
 @example
 ```ts
 await countLines({ session, text: 'measured', },);
 ```
 */
async function countLines(
  {
    session,
    text,
  }: {
    readonly session: Session;
    readonly text: string;
  },
): Promise<number> {
  return (await session.lines())
    .filter(function hasText(line,): boolean {
      return line.includes(text,);
    },)
    .length;
}

/**
 Waits until the log holds at least a number of lines with a text.
 
 @param session - the session whose log is read
 
 @param text - text a line must contain
 
 @param atLeast - the number of lines to wait for
 
 @returns whether the number was reached in time
 
 @example
 ```ts
 await waitForLines({ session, text: 'measured', atLeast: 3, },);
 ```
 */
async function waitForLines(
  {
    session,
    text,
    atLeast,
  }: {
    readonly atLeast: number;
    readonly session: Session;
    readonly text: string;
  },
): Promise<boolean> {
  return await pollUntilDeadline({
    timeoutMs: FOLLOWS_DISK_MS,
    intervalMs: LOOK_EVERY_MS,
    check: async function reached(): Promise<boolean> {
      return await countLines({
        session,
        text,
      },) >= atLeast;
    },
  },);
}

/**
 Records the refresh checks.
 
 @param context - the run's context
 
 @param session - the session on the refresh library, started with the library already measured
 
 @param library - the library directory the app watches
 
 @example
 ```ts
 await checkRefresh({ context, session, library, },);
 ```
 */
export async function checkRefresh(
  {
    context,
    session,
    library,
  }: {
    readonly context: RunContext;
    readonly library: string;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('states',);
  await recorder.check({
    id: 'states/refresh-initial',
    title: 'the library the app starts on is measured by itself, once per file',
    body: async function startsMeasured(): Promise<string> {
      await session.waitState({
        predicate: isLibraryReady,
        what: 'the library shown',
        timeoutMs: SESSION_READY_MS,
      },);
      expect({
        condition: await waitForLines({
          session,
          text: MEASURED_LINE,
          atLeast: 2,
        },),
        message: 'the two files of the library were not measured',
      },);
      return `${await countLines({
        session,
        text: MEASURED_LINE,
      },)} measured`;
    },
  },);
  await recorder.check({
    id: 'states/refresh-replaced-file',
    title: 'a file replaced on disk with different audio is measured again while the app runs, with no key pressed',
    body: async function replacedFileIsMeasured(): Promise<string> {
      /**
       Decodes finished before the file changed.
       */
      const before = await countLines({
        session,
        text: MEASURED_LINE,
      },);
      await writeFile(
        join(
          library,
          'a.wav',
        ),
        waveBytes({
          seconds: 2,
          hertz: 523,
        },),
      );
      expect({
        condition: await waitForLines({
          session,
          text: MEASURED_LINE,
          atLeast: before + 1,
        },),
        message: `the replaced file was not measured again (still ${before} decodes)`,
      },);
      return `${before} decodes before, then one more`;
    },
  },);
  await recorder.check({
    id: 'states/refresh-new-file',
    title: 'a file added to the folder is listed and measured by itself',
    body: async function newFileIsMeasured(): Promise<string> {
      /**
       Decodes finished before the file appeared.
       */
      const before = await countLines({
        session,
        text: MEASURED_LINE,
      },);
      await writeFile(
        join(
          library,
          'c.wav',
        ),
        waveBytes({
          seconds: 1,
          hertz: 262,
        },),
      );
      expect({
        condition: await waitForLines({
          session,
          text: MEASURED_LINE,
          atLeast: before + 1,
        },),
        message: 'the new file was not measured',
      },);
      return 'measured';
    },
  },);
  await recorder.check({
    id: 'states/refresh-broken-file-once',
    title: 'a file that cannot be decoded is reported once, however often the library changes afterwards',
    body: async function brokenFileIsReportedOnce(): Promise<string> {
      await writeFile(
        join(
          library,
          'd.wav',
        ),
        'this is not audio',
      );
      expect({
        condition: await waitForLines({
          session,
          text: REPORTED_LINE,
          atLeast: 1,
        },),
        message: 'the broken file was not reported',
      },);
      await session.shot('refresh-broken-file',);

      // Two more changes of the folder: each makes the app try the broken file again.
      await writeFile(
        join(
          library,
          'e.wav',
        ),
        waveBytes({
          seconds: 1,
          hertz: 294,
        },),
      );
      await waitForLines({
        session,
        text: MEASURED_LINE,
        atLeast: 1,
      },);
      await writeFile(
        join(
          library,
          'f.wav',
        ),
        waveBytes({
          seconds: 1,
          hertz: 330,
        },),
      );
      await wait(QUIET_MS,);

      /**
       How often the broken file was reported in all.
       */
      const reported = await countLines({
        session,
        text: REPORTED_LINE,
      },);
      expect({
        condition: reported === 1,
        message: `the broken file was reported ${reported} times`,
      },);
      return 'reported once';
    },
  },);
}
