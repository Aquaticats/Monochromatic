import {
  mkdir,
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Milliseconds a command the tests start is given before its deadline stops it, far below the time it would take to end
 by itself.
 */
const shortDeadlineMilliseconds = 300;

/**
 Milliseconds a test's own deadline allows, which is far more than a `node --eval` needs.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Milliseconds the command of the group-stop control is given, which is long enough for node to start and start a child of
 its own on a host running other agents' builds.
 */
const groupStopDeadlineMilliseconds = 1_500;

/**
 Milliseconds after it starts that the command's own child writes the record saying it outlived that deadline.

 This is three times the deadline, so a child the deadline stopped cannot reach the write, and the margin covers a host
 whose timer runs late.
 */
const survivalWriteMilliseconds = 4_500;

/**
 Milliseconds the test waits before it reads whether the record of a surviving child or the marker of a started one is
 there.
 */
const survivalWaitMilliseconds = survivalWriteMilliseconds + 500;

/**
 Milliseconds the wait of the grace control may last, which is its deadline, the grace the stop gives after it and room
 for a host running other agents' builds; the child that holds the wait open would otherwise hold it for a minute.
 */
const escapedChildGivesUpMilliseconds = 30_000;

/**
 Create one disposable tree under the private scratch parent.

 @param name - prefix of the tree's directory name

 @returns the tree, removed when the caller's scope ends
 */
async function scratchTree({ name, }: {
  readonly name: string;
},) {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  return mkdtempDisposable(join(
    scratchParent,
    name,
  ),);
}

/**
 Whether a file the fixture writes is there.

 @param path - file the command, or the child it started, writes

 @returns true when the file is there
 */
async function wroteAFile(path: string,): Promise<boolean> {
  try {
    await readFile(
      path,
      'utf8',
    );
    return true;
  }
  catch (error) {
    if (((typeof error) === 'object') && (error !== null) && ('code' in error) && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Stop the child a stopped command started in a session of its own, so the test leaves nothing running.

 @param pidFile - file that child writes its own process ID to
 */
async function stopProcessInFile(pidFile: string,): Promise<void> {
  try {
    /**
     The escaped child's own process ID, which it wrote when it started.
     */
    const text = await readFile(
      pidFile,
      'utf8',
    );
    process.kill(
      Number(text),
      'SIGKILL',
    );
  }
  catch (error) {
    // A child that never started left no file, and one that already ended answers ESRCH: neither is a test failure.
    if (((typeof error) === 'object') && (error !== null) && ('code' in error)
      && ((error.code === 'ENOENT') || (error.code === 'ESRCH'))) {
      return;
    }
    throw error;
  }
}

/**
 A command that starts one child of its own in a session of its own, which holds this command's output while the stop of
 the command's own process group cannot reach it.
 */
const startsAChildInItsOwnSession = String.raw`
const { spawn } = require('node:child_process');
const { writeFileSync } = require('node:fs');
const child = spawn(process.execPath, ['--eval', 'setTimeout(() => {}, 60000)'], { detached: true, stdio: ['ignore', 'inherit', 'inherit'] });
writeFileSync(process.env.ESCAPED_PID_FILE, String(child.pid));
setTimeout(() => {}, 60000);
`;

/**
 A command that starts one child of its own, which writes a record a moment later if it is still running.

 The child holds the pipe this command's output is read from through `stdio: 'inherit'`, so a deadline that stopped only
 the program this test started is caught twice: the wait lasts until the grace ends it, and the child leaves the record.
 */
const startsAChildThatOutlivesTheDeadline = String.raw`
const { spawn } = require('node:child_process');
const { writeFileSync } = require('node:fs');
const record = 'const { writeFileSync } = require("node:fs"); setTimeout(() => writeFileSync(process.env.SURVIVAL_FILE, "survived"), Number(process.env.SURVIVAL_WRITE_MILLISECONDS)); setTimeout(() => {}, 60000);';
spawn(process.execPath, ['--eval', record], { stdio: ['ignore', 'inherit', 'inherit'] });
writeFileSync(process.env.STARTED_FILE, 'started');
setTimeout(() => {}, 60000);
`;

await describe({
  name: runCommand.name,
  children: [
    //region The deadline a command may be given

    it({
      name: 'stops a command that ran past its deadline and names the step and the bound',
      fn: async () => {
        /**
         When the deadline was noticed.
         */
        const started = Date.now();
        /**
         The failure of a command that would have run for three seconds.
         */
        let caught: unknown;
        try {
          await runCommand({
            command: process.execPath,
            args: [
              '--eval',
              'setTimeout(() => {}, 3000)',
            ],
            capture: true,
            deadlineMilliseconds: shortDeadlineMilliseconds,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('--eval',);
        expect((caught as Error).message,).toContain('was still running after 300 ms',);
        expect(Date.now() - started,).toBeLessThan(2_500,);
      },
      timeout: 10_000,
    },),
    it({
      name: 'stops the child the command started, so nothing records that it outlived the deadline',
      fn: async () => {
        /**
         Tree holding the record only the command's own child could write.
         */
        await using tree = await scratchTree({ name: 'stopped-command-test-', },);
        /**
         File the command writes once it has started its own child, so a command that was stopped before it started a
         child cannot pass this control.
         */
        const startedFile = join(
          tree.path,
          'started.txt',
        );
        /**
         File the command's own child writes, a moment after it starts, when it is still running.
         */
        const survivalFile = join(
          tree.path,
          'survival.txt',
        );
        /**
         The failure of the command the deadline stopped.
         */
        let caught: unknown;
        try {
          await runCommand({
            command: process.execPath,
            args: [
              '--eval',
              startsAChildThatOutlivesTheDeadline,
            ],
            capture: true,
            env: {
              ...process.env,
              STARTED_FILE: startedFile,
              SURVIVAL_FILE: survivalFile,
              SURVIVAL_WRITE_MILLISECONDS: String(survivalWriteMilliseconds),
            },
            deadlineMilliseconds: groupStopDeadlineMilliseconds,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('was still running after 1500 ms',);
        await wait(survivalWaitMilliseconds,);
        expect(await wroteAFile(startedFile,),).toBe(true,);
        expect(await wroteAFile(survivalFile,),).toBe(false,);
      },
      timeout: 20_000,
    },),
    it({
      name: 'releases the caller at the grace when a child the stop cannot reach still holds the output',
      fn: async () => {
        await using tree = await scratchTree({ name: 'escaped-child-test-', },);
        /**
         File the command's own child writes its process ID to, so the test can stop it.
         */
        const escapedPidFile = join(
          tree.path,
          'escaped.pid',
        );
        /**
         When the deadline was noticed.
         */
        const started = Date.now();
        /**
         The failure of a command whose output a child the stop cannot reach keeps open.
         */
        let caught: unknown;
        try {
          await runCommand({
            command: process.execPath,
            args: [
              '--eval',
              startsAChildInItsOwnSession,
            ],
            capture: true,
            env: {
              ...process.env,
              ESCAPED_PID_FILE: escapedPidFile,
            },
            deadlineMilliseconds: groupStopDeadlineMilliseconds,
          },);
        }
        catch (error) {
          caught = error;
        }
        /**
         How long the wait lasted, which only the grace can end.
         */
        const elapsed = Date.now() - started;
        /**
         Whether the command started the child that holds its output open, which this control needs.
         */
        const ranTheChild = await wroteAFile(escapedPidFile,);
        await stopProcessInFile(escapedPidFile,);
        expect(ranTheChild,).toBe(true,);
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('was still running after 1500 ms',);
        expect(elapsed,).toBeLessThan(escapedChildGivesUpMilliseconds,);
      },
      timeout: 40_000,
    },),
    it({
      name: 'leaves a command that ends inside its deadline alone, so a bound never cuts off good work',
      fn: async () => {
        /**
         What the command answered.
         */
        const outcome = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            'process.stdout.write("answered")',
          ],
          capture: true,
          deadlineMilliseconds: testDeadlineMilliseconds,
        },);
        expect(outcome.status,).toBe(0,);
        expect(outcome.stdout,).toBe('answered',);
      },
      timeout: 10_000,
    },),
    it({
      name: 'reads the status and output of a command that ends inside its deadline, which a caller judges',
      fn: async () => {
        /**
         What the command answered, which a caller reading a status keeps.
         */
        const outcome = await runCommand({
          command: process.execPath,
          args: [
            '--eval',
            String.raw`process.stderr.write("refused\n"); process.exitCode = 2`,
          ],
          capture: true,
          allowFailure: true,
          deadlineMilliseconds: testDeadlineMilliseconds,
        },);
        expect(outcome.status,).toBe(2,);
        expect(outcome.stderr,).toBe('refused\n',);
      },
      timeout: 10_000,
    },),

    //endregion The deadline a command may be given
  ],
},);
