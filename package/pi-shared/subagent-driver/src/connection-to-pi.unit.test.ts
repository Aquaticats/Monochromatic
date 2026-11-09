/**
 The channel to Pi against a stand-in `pi`: answers, silence, odd output, and every stage of ending. Each test uses its own
 disposable folder.

 @module
 */

import { constants, } from 'node:fs';
import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  open,
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
  type PiRecord,
  startConnectionToPi,
} from '../dist/final/node/index.mjs';
import {
  COMPACTION_FAILURE_TEXT,
  COMPACTION_MILLISECONDS,
} from './stand-in-pi-compaction-plays-fixture.ts';
import { STANDARD_ERROR_TEXT, } from './stand-in-pi-records-fixture.ts';

const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 Milliseconds Pi gets at each stage of ending in the tests about escalating past a stage: the scenario there ignores its input
 closing or its first signal, so the stage bound is the thing being waited out, and it is short so the test is quick.
 */
const ESCALATION_GRACE_MILLISECONDS = 1_500;
/**
 Milliseconds to wait for an answer that must arrive. The stand-in starts, answers and leaves in a fraction of this even with a
 capped processor; the bound only matters when the connection, not the machine, is what failed, so it is generous.
 */
const ANSWER_MILLISECONDS = 60_000;
/** Milliseconds to wait for an answer that will never come, far below the bound the driver ships. */
const SHORT_ANSWER_MILLISECONDS = 50;
/** Milliseconds between looks at what a test waits for. */
const LOOK_MILLISECONDS = 20;
/**
 Milliseconds a test waits for something the connection reports before giving up on it. What it waits for is a report the
 connection makes as the event arrives, so only a starved host can spend this.
 */
const REPORT_DEADLINE_MILLISECONDS = 60_000;

type Heard = {
  readonly events: PiRecord[];
  readonly unreadable: string[];
  readonly problems: string[];
};

type Opened = {
  readonly connection: ConnectionToPi;
  readonly heard: Heard;
  readonly log: string;
  readonly standardErrorFile: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a named pipe, which waits for a writer.

 @param path - where the pipe is made
 */
async function makePipe(path: string,): Promise<void> {
  await execFileAsync('mkfifo', [path,],);
}

/**
 Waits until what the connection reported contains a text.

 @param heard - what the connection reported

 @param text - text to wait for
 */
async function waitForProblem({
  heard,
  text,
}: {
  readonly heard: Heard;
  readonly text: string
},): Promise<void> {
  /**
   When to give up.
   */
  const deadline = Date.now() + REPORT_DEADLINE_MILLISECONDS;
  while (Date.now() <= deadline) {
    if (heard.problems.some((problem,) => problem.includes(text,))) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- a report arrives on its own schedule, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
  throw new Error(`no problem naming ${text}; the ones reported were ${heard.problems.join(', ')}`,);
}

/**
 Sends one command under the bound these tests wait with, so a test that is not about the deadline still gets one.

 @param connection - the running Pi

 @param request - the command, and any shorter silence bound or deadline the test is about

 @returns Pi's answer, or how waiting for it ended
 */
async function sendTo(
  connection: ConnectionToPi,
  request: {
    readonly command: PiRecord;
    readonly answerMilliseconds?: number;
    readonly deadline?: number;
  },
): Promise<PiRecord> {
  return await connection.send({
    ...request,
    deadline: request.deadline ?? (Date.now() + ANSWER_MILLISECONDS),
  },);
}

/**
 Reads the stand-in's log, which does not exist when Pi never started.

 @param log - file the stand-in writes

 @returns the log's text, or nothing when it was never written
 */
async function readLog(log: string,): Promise<string> {
  try {
    return await readFile(log, 'utf8',);
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
  const deadline = Date.now() + REPORT_DEADLINE_MILLISECONDS;
  while (Date.now() <= deadline) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the log is read again after each wait, which is the whole point of the loop; see doc/lint/sequential-awaits.md
    if ((await readLog(log,)).includes(text,)) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the stand-in writes its log as it goes, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
  throw new Error(`the log never showed ${text}`,);
}

/**
 Ends the tool a scenario started, so no fixture outlives the test.

 @param log - file the stand-in logs the tool's process number to
 */
async function stopToolIn(log: string,): Promise<void> {
  /**
   The log, which holds no tool line when the scenario started none, and does not exist when Pi never started.
   */
  const logged = await readLog(log,);
  /**
   The tool's process number, when the log holds one.
   */
  const toolLine = logged.split('\n',).find((line,) => line.startsWith('tool ',));
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

async function openConnection(
  {
    scenario,
    withoutPi = false,
    standardErrorIsPipe = false,
    graceMilliseconds,
  }: {
    readonly scenario: string;
    readonly withoutPi?: boolean;
    readonly standardErrorIsPipe?: boolean;

    /**
     How long Pi gets at each stage of ending. Absent means the bound the driver ships, which is what a test about Pi leaving
     on its own terms needs: a starved host has to be allowed the same leave a real run would take.
     */
    readonly graceMilliseconds?: number;
  },
): Promise<Opened> {
  const folder = await mkdtemp(join(scratchRoot(), 'pi-connection-',),);
  const log = join(folder, 'commands.log',);
  const standardErrorFile = join(folder, 'stderr.log',);
  /**
   A folder holding no executable, so a test that takes Pi away cannot reach a real one.
   */
  const withoutExecutables = join(folder, 'no-executables',);
  await mkdir(withoutExecutables,);
  if (standardErrorIsPipe) {
    await makePipe(standardErrorFile,);
  }
  if (!withoutPi) {
    await writeFile(join(folder, 'pi',), `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,);
    await chmod(join(folder, 'pi',), 0o700,);
  }
  const heard: Heard = { events: [], unreadable: [], problems: [], };
  const connection = startConnectionToPi({
    piArguments: [],
    workingFolder: folder,
    environment: {
      PATH: withoutPi ? withoutExecutables : [folder, dirname(process.execPath,), '/usr/bin',].join(delimiter,),
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
    },
    standardErrorFile,
    ...(graceMilliseconds === undefined ? {} : { graceMilliseconds, }),
    listeners: {
      onEvent: (record,) => {
        heard.events.push(record,);
      },
      onUnreadableLine: (line,) => {
        heard.unreadable.push(line,);
      },
      onExit: () => {
        heard.problems.push('exit noted',);
      },
      onProblem: (problem,) => {
        heard.problems.push(problem,);
      },
    },
  },);
  return {
    connection,
    heard,
    log,
    standardErrorFile,
    async [Symbol.asyncDispose]() {
      await connection.shutDown();
      await stopToolIn(log,);
      if (standardErrorIsPipe) {
        /**
         A reader of the pipe, so the copy's open finishes and the test process holds nothing open.
         */
        const reader = await open(
          standardErrorFile,
          constants.O_RDONLY | constants.O_NONBLOCK,
        );
        await reader.close();
      }
      await rm(folder, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'the connection to Pi',
  children: [
    it({
      name: 'matches an answer to its command and ends Pi by closing its input',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'normal', },);
        const answer = await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        expect(answer.success,).toBe(true,);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(opened.connection.hasExited(),).toBe(true,);
        expect(opened.connection.exitDetails().code,).toBe(0,);
      },
    },),
    it({
      name: 'matches an answer to a command that brought an identifier of its own',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'normal', },);
        const answer = await sendTo(opened.connection, {
          command: {
            id: 'custom',
            type: 'get_state',
          },
          answerMilliseconds: ANSWER_MILLISECONDS,
        },);
        expect(answer.success,).toBe(true,);
      },
    },),
    it({
      name: 'a silent Pi gives a failed answer naming the bound asked for, so that bound is the one used',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'exit-later', },);
        const answer = await sendTo(opened.connection, {
          command: { type: 'get_state', },
          answerMilliseconds: SHORT_ANSWER_MILLISECONDS,
        },);
        expect(answer.ending,).toBe('silence',);
        // The answer that named no bound, or Pi's own leaving, would be a different text; this one proves the bound asked for
        // was the bound used, whatever the host's schedule made of the two timers.
        expect(answer.problem,).toBe(`no answer in ${String(SHORT_ANSWER_MILLISECONDS,)} ms`,);
      },
    },),
    it({
      name: 'a compaction under way is not silence, so the wait outlives the bound asked for and still takes the answer',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'compaction-then-answer', },);
        const started = Date.now();
        const answer = await sendTo(opened.connection, {
          command: {
            type: 'prompt',
            message: 'hello',
          },
          answerMilliseconds: SHORT_ANSWER_MILLISECONDS,
          deadline: started + ANSWER_MILLISECONDS,
        },);
        expect(answer.ending,).toBe('answered',);
        expect(answer.record?.success,).toBe(true,);
        // The compaction held the answer past the bound, which is what proves the bound stopped counting while Pi compacted.
        expect(Date.now() - started,).toBeGreaterThan(COMPACTION_MILLISECONDS,);
      },
    },),
    it({
      name: 'a compaction that never ends is ended by the deadline, and the answer names the compaction it waited on',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'compaction-never-ends', },);
        const started = Date.now();
        const answer = await sendTo(opened.connection, {
          command: {
            type: 'prompt',
            message: 'hello',
          },
          deadline: started + COMPACTION_MILLISECONDS,
        },);
        expect(answer.ending,).toBe('deadline',);
        expect(answer.problem,).toContain('compacting',);
        expect(answer.compactionProblem,).toContain('compacting',);
        // The wall bound the caller gave is the one that ended it, not the far longer silence bound the driver ships.
        expect(Date.now() - started,).toBeLessThan(ANSWER_MILLISECONDS / 2,);
      },
    },),
    it({
      name: 'a failed compaction is named in the answer that stops waiting, and the silence bound still ends that wait',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'compaction-fails', },);
        const started = Date.now();
        const answer = await sendTo(opened.connection, {
          command: {
            type: 'prompt',
            message: 'hello',
          },
          answerMilliseconds: SHORT_ANSWER_MILLISECONDS,
          deadline: started + ANSWER_MILLISECONDS,
        },);
        expect(answer.ending,).toBe('silence',);
        expect(answer.problem,).toBe(`no answer in ${String(SHORT_ANSWER_MILLISECONDS,)} ms`,);
        expect(answer.compactionProblem,).toContain(COMPACTION_FAILURE_TEXT,);
        // The bound is measured from the end of the compaction, so the wait outlives the compaction and then some.
        expect(Date.now() - started,).toBeGreaterThan(COMPACTION_MILLISECONDS,);
      },
    },),
    it({
      name: 'a command waiting when Pi exits gets a failed answer, and later commands fail at once',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'exits-silently', },);
        const answer = await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        expect(answer.error,).toBe('pi exited',);
        expect(opened.connection.exitDetails().code,).toBe(3,);
        const later = await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        expect(later.error,).toBe('pi exited',);
        opened.connection.write({ type: 'get_state', },);
      },
    },),
    it({
      name: 'a Pi that stays after its input closes is ended with SIGTERM',
      fn: async () => {
        await using opened = await openConnection({
          scenario: 'ignores-input-close',
          graceMilliseconds: ESCALATION_GRACE_MILLISECONDS,
        },);
        await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(opened.connection.exitDetails().signal,).toBe('SIGTERM',);
      },
    },),
    it({
      name: 'a Pi that ignores SIGTERM is ended with SIGKILL',
      fn: async () => {
        await using opened = await openConnection({
          scenario: 'ignores-sigterm',
          graceMilliseconds: ESCALATION_GRACE_MILLISECONDS,
        },);
        await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(opened.connection.exitDetails().signal,).toBe('SIGKILL',);
      },
    },),
    it({
      name: 'a tool still holding the output open does not keep the driver waiting',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'keeps-output-open', },);
        await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(opened.connection.hasExited(),).toBe(true,);
        expect(opened.connection.exitDetails().code,).toBe(0,);
      },
    },),
    it({
      name: 'skips a blank line, reports a line that is not a record, and treats a response without a text id as an event',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'odd-lines', },);
        const answer = await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        expect(answer.success,).toBe(true,);
        expect(opened.heard.unreadable,).toEqual(['[1]',],);
        expect(opened.heard.events,).toEqual([{ type: 'response', id: 7, },],);
      },
    },),
    it({
      name: 'a Pi that cannot be started is reported, and a command sent to it fails',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'normal', withoutPi: true, },);
        const answer = await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        expect(answer.error,).toBe('pi exited',);
        expect(String(opened.connection.exitDetails().processError,),).toContain('ENOENT',);
      },
    },),
    it({
      name: 'keeps what Pi writes to its standard error',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'stderr-note', },);
        await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(await readFile(opened.standardErrorFile, 'utf8',),).toContain(STANDARD_ERROR_TEXT,);
      },
    },),
    it({
      name: 'a standard error copy that cannot finish is reported, and ending the run still returns',
      fn: async () => {
        await using opened = await openConnection({
          scenario: 'normal',
          standardErrorIsPipe: true,
          graceMilliseconds: ESCALATION_GRACE_MILLISECONDS,
        },);
        const outcome = await opened.connection.shutDown();
        expect(outcome.stopped,).toBe(true,);
        expect(opened.heard.problems.some((problem,) => problem.includes('standard error',)),).toBe(true,);
      },
    },),
    it({
      name: 'a write to a Pi that closed its input is reported, and is not taken for an exit',
      fn: async () => {
        await using opened = await openConnection({ scenario: 'closes-input', },);
        await sendTo(opened.connection, { command: { type: 'get_state', }, },);
        await waitForLogLine({ log: opened.log, text: 'marker input-closed', },);
        opened.connection.write({ type: 'get_state', },);
        await waitForProblem({ heard: opened.heard, text: 'could not write', },);
        expect(opened.connection.hasExited(),).toBe(false,);
      },
    },),
  ],
},);
