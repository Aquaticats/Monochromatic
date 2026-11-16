/**
 Putting a run away against a stand-in `pi`: what the ending leaves for the summary, and what it leaves for a Pi whose exit is
 seen only once the ending's own bounds are spent. Each test uses its own disposable folder.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { kill, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type ConnectionToPi,
  MASKING_NOTHING,
  type PiRecord,
  putTheRunAway,
  type RecordOfPiRun,
  SHUTDOWN_GRACE_MILLISECONDS,
  startConnectionToPi,
  startRecordOfPiRun,
} from '../dist/final/node/index.mjs';

const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 The scenario in which Pi holds its output open, ignores its input closing and SIGTERM, and leaves by itself with a success status
 a little after that signal: an exit the driver can hear of only after the ending's last bound, because nothing closes its output.
 */
const LEAVES_AFTER_SIGTERM = 'leaves-after-sigterm-with-output-held';

/**
 The scenario in which Pi stays after its input closes until a signal ends it, which is a Pi the ending needed a signal for.
 */
const IGNORES_INPUT_CLOSE = 'ignores-input-close';

/**
 How long a test waits for a line the stand-in logs before giving up on it: the tool's line is written while the stand-in starts,
 so only a starved host can spend this.
 */
const LOG_DEADLINE_MILLISECONDS = 60_000;

/**
 Milliseconds between looks at what a test waits for.
 */
const LOOK_MILLISECONDS = 20;

/**
 How long the connection waits for Pi's output to end before it reports the exit it was already told of, which is longer than one
 stage of the ending and shorter than the two a test needs it to be heard within.

 The ending gives Pi `SHUTDOWN_STAGES` stages of `SHUTDOWN_GRACE_MILLISECONDS` each, so a Pi that leaves `x` after the first
 signal is heard of `x + this` after the ending began. The stand-in leaves four seconds after that signal, so this value puts the
 exit one second after the ending's last bound and three seconds before its copy of Pi's standard error is given up on.
 */
const EXIT_DEADLINE_MILLISECONDS = (SHUTDOWN_GRACE_MILLISECONDS * 8) / 5;

/**
 One Pi, its run's record, and the folder they live in.
 */
type RunUnderTest = {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly runFolder: string;
  readonly log: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Reads a text file, or nothing when it is not there yet.

 @param path - file to read

 @returns its text, or an empty string
 */
async function textOrNothingIn(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return '';
    }
    throw error;
  }
}

/**
 Waits until the stand-in's log holds a line.

 @param log - file the stand-in writes

 @param text - text to wait for
 */
async function waitForLogLine({
  log,
  text,
}: {
  readonly log: string;
  readonly text: string
},): Promise<void> {
  /**
   When to give up.
   */
  const deadline = Date.now() + LOG_DEADLINE_MILLISECONDS;
  while (Date.now() <= deadline) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the stand-in writes its log as it goes, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    if ((await textOrNothingIn(log,)).includes(text,)) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- a fixture's line arrives on its own schedule, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
  throw new Error(`the log never showed ${text}`,);
}

/**
 Ends the tool a scenario started, so no fixture outlives its test.

 @param log - file the stand-in logs the tool's process number to
 */
async function stopToolIn(log: string,): Promise<void> {
  /**
   The log, which holds no tool line when the scenario started none.
   */
  const logged = await textOrNothingIn(log,);
  /**
   The tool's process number, when the log holds one.
   */
  const toolLine = logged.split('\n',)
    .find((line,) => line.startsWith('tool ',));
  if (toolLine !== undefined) {
    try {
      kill(
        Number(toolLine.slice('tool '.length,),),
      );
    }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ESRCH') {
        throw error;
      }
    }
  }
}

/**
 Starts a stand-in Pi and the record of its run.

 @param scenario - which behaviour of the stand-in this test asks for

 @returns the Pi, the record and their folder

 @example
 ```ts
 await using run = await startRun({ scenario: 'normal', },);
 ```
 */
async function startRun({ scenario, }: { readonly scenario: string },): Promise<RunUnderTest> {
  /**
   The folder this run's files live in.
   */
  const folder = await mkdtemp(join(scratchRoot(), 'pi-put-away-',),);
  /**
   Where the stand-in logs what it was given and what it started.
   */
  const log = join(folder, 'commands.log',);
  /**
   The run's folder, which is where the feed and the summary go.
   */
  const runFolder = join(folder, 'run',);
  await writeFile(
    join(folder, 'pi',),
    `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,
  );
  await chmod(
    join(folder, 'pi',),
    0o700,
  );
  const record = startRecordOfPiRun({
    feedFile: join(runFolder, 'feed.jsonl',),
    mask: MASKING_NOTHING,
  },);
  const connection = startConnectionToPi({
    piArguments: [],
    workingFolder: folder,
    environment: {
      PATH: [folder, dirname(process.execPath,), '/usr/bin',].join(delimiter,),
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
    },
    standardErrorFile: join(runFolder, 'stderr.log',),
    graceMilliseconds: EXIT_DEADLINE_MILLISECONDS,
    listeners: {
      onEvent: function onEvent() {
        // The ending reads nothing of what Pi reports while it runs.
      },
      onUnreadableLine: function onUnreadableLine() {
        // A record this test does not read is not a failure of the ending.
      },
      onExit: function onExit(exit,) {
        record.note({
          kind: 'exit',
          ...exit,
        },);
      },
      onProblem: function onProblem(problem,) {
        record.note({
          kind: 'problem',
          problem,
        },);
      },
    },
  },);
  return {
    connection,
    record,
    runFolder,
    log,
    async [Symbol.asyncDispose]() {
      await connection.shutDown();
      await stopToolIn(log,);
      await record.close();
      await rm(
        folder,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Every line of a run's feed.

 @param runFolder - the run's folder

 @returns the lines, each parsed

 @example
 ```ts
 const lines = await feedIn('/runs/a-1');
 ```
 */
async function feedIn(runFolder: string,): Promise<readonly PiRecord[]> {
  /**
   The feed, which this test asks for only after the ending wrote its last line.
   */
  const shown = await textOrNothingIn(join(runFolder, 'feed.jsonl',),);
  return shown.split('\n',)
    .filter((line,) => line !== '')
    .map(function parsed(line,) {
      return JSON.parse(line,) as PiRecord;
    },);
}

await describe({
  name: 'the way a run is put away',
  children: [
    it({
      name: 'an ordinary ending takes nothing, and the statistics Pi answered are the summary\'s',
      fn: async () => {
        await using run = await startRun({ scenario: 'normal', },);
        const putAway = await putTheRunAway({
          connection: run.connection,
          record: run.record,
        },);
        expect(putAway.stopped,).toBe(true,);
        expect(putAway.stopProblem,).toBeUndefined();
        expect(putAway.sessionStats,).toMatchObject({ cost: 0.5, },);
      },
    },),
    it({
      name: 'a Pi that stays after its input closes is one the ending needed a signal for, and the ending says so',
      fn: async () => {
        await using run = await startRun({ scenario: IGNORES_INPUT_CLOSE, },);
        const putAway = await putTheRunAway({
          connection: run.connection,
          record: run.record,
        },);
        expect(run.connection.exitDetails().signal,).toBe('SIGTERM',);
        expect(putAway.stopped,).toBe(true,);
        expect(putAway.stopProblem,).toContain('SIGTERM',);
      },
    },),
    it({
      name: 'a Pi whose exit is seen only as the ending\'s last bound runs out is still a Pi that left, and the ending says what it took',
      fn: async () => {
        await using run = await startRun({ scenario: LEAVES_AFTER_SIGTERM, },);
        // The tool holds the stand-in's output before the ending starts, so nothing the stand-in's leaving does can close it.
        await waitForLogLine({
          log: run.log,
          text: 'tool ',
        },);
        const putAway = await putTheRunAway({
          connection: run.connection,
          record: run.record,
        },);
        // What the driver knew of Pi when the ending returned: it left by itself, with a success status, and no signal ended it.
        expect(run.connection.hasExited(),).toBe(true,);
        expect(run.connection.exitDetails(),).toEqual({ code: 0, },);
        // What it recorded of that same ending: a Pi that left, and the bound the ending ran out waiting for it.
        expect(putAway.stopped,).toBe(true,);
        expect(putAway.stopProblem,).toContain('SIGTERM',);
        expect(putAway.stopProblem,).toContain('last bound',);
        /**
         The lines of the run's feed after the ending.
         */
        const feed = await feedIn(run.runFolder,);
        expect(feed.at(-1,),).toMatchObject({
          kind: 'problem',
          problem: putAway.stopProblem,
        },);
      },
    },),
  ],
},);
