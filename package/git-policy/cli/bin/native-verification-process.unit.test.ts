import type { ChildProcess, } from 'node:child_process';
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

import { stopProcessGroup, } from './process-group-stop.ts';
import {
  NativeVerificationError,
  runCommand,
  runStartedCommand,
  startCommand,
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
 Milliseconds a test's own deadline allows, which is far more than a `node --eval` needs: a command that should end by
 itself, and the wait for the record a stand-in's child writes before a control starts the clock it asserts.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Milliseconds the command of the group-stop control is given once its child has started, which is the bound that control
 asserts.

 It is short enough that a child the stop missed records its survival after it, so the absence of that record is what
 shows the stop reached the child, and that the control ends in seconds.
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
 Milliseconds between two looks for the record a stand-in's child writes, short enough that the control starts the clock
 it asserts within a moment of the child's own start.
 */
const recordedFilePollMilliseconds = 20;

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
 Wait until the child a stand-in started has written the record saying it is running.

 The wait is bounded by {@link testDeadlineMilliseconds}, which is a deadline of this control's own and never the bound
 the code under test keeps: a host that starts a process slowly spends this window, and the bound the control asserts
 begins only once the record is there.

 @param record - file the stand-in's own child writes once it is running

 @param missing - what the stand-in was asked to start, said in the words of the control that waits for it

 @throws NativeVerificationError when the record never appears, which names the bound the wait was given
 */
async function waitsForTheChildToStart({ record, missing, }: {
  readonly record: string;
  readonly missing: string;
},): Promise<void> {
  /**
   When this wait gives up, which is the control's own bound and not the bound under test.
   */
  const deadline = performance.now() + testDeadlineMilliseconds;
  for (;;) {
    // oxlint-disable-next-line no-await-in-loop -- one look for the record, then one wait, is the wait itself
    if (await wroteAFile(record,))
      return;
    if (performance.now() >= deadline) {
      throw new NativeVerificationError(
        `${missing} never wrote ${record} within ${String(testDeadlineMilliseconds)} ms, so this control cannot tell whether the stand-in was slow to start or the bound under test stopped it: nothing here judges that bound.`,
      );
    }
    // oxlint-disable-next-line no-await-in-loop -- one look each interval is what this wait is
    await wait(recordedFilePollMilliseconds,);
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
 A stand-in command one control started, which the control ends when its scope ends.

 Its disposal stops the command and everything the command started, so a wait that gave up and a failing assertion
 leave nothing running.
 */
type OwnedStandIn = AsyncDisposable & {
  /**
   The started command.
   */
  readonly child: ChildProcess;
  /**
   Command and first argument, which the refusal of a bound names.
   */
  readonly step: string;
};

/**
 Start a stand-in that one control waits on and ends.

 @param command - executable to start

 @param args - its arguments, each passed whole

 @param env - the whole environment, which names the files of the control's fixture the stand-in writes

 @returns the started command, ended when the caller's scope ends
 */
function ownStandIn({ command, args, env, }: {
  readonly command: string;
  readonly args: readonly string[];
  readonly env: NodeJS.ProcessEnv;
},): OwnedStandIn {
  /**
   The command, leading a process group of its own so a bound's stop reaches the child it starts.
   */
  const started = startCommand({
    command,
    args,
    capture: true,
    env,
    leadOwnProcessGroup: true,
  },);
  return {
    child: started.child,
    step: started.step,
    async [Symbol.asyncDispose](): Promise<void> {
      stopProcessGroup({ child: started.child, },);
    },
  };
}

/**
 Own the child a stand-in started outside its process group, so the caller's scope ends it however the scope ends.

 @param pidFile - file the child writes its own process number to

 @returns the ownership, whose disposal stops the process the file names
 */
function ownEscapedChild({ pidFile, }: {
  readonly pidFile: string;
},): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await stopProcessInFile(pidFile,);
    },
  };
}

/**
 The process number of a stand-in and of the process group it is in, as the stand-in wrote them.

 @param text - what the stand-in wrote: its own number, then the number of its group, separated by one space

 @returns both numbers

 @throws NativeVerificationError when the text is not two whole numbers
 */
function processNumbersOf(text: string,): { readonly own: number; readonly group: number; } {
  /**
   The two words the stand-in wrote, read apart so a shorter answer is refused rather than compared as a NaN.
   */
  const [own, group,] = text.trim()
    .split(' ',);
  const ownNumber = Number(own,);
  const groupNumber = Number(group,);
  if ((own === undefined) || (group === undefined) || (!Number.isInteger(ownNumber,)) || (!Number.isInteger(groupNumber,)))
    throw new NativeVerificationError(`the stand-in wrote "${text}" where this control reads two process numbers`);
  return {
    own: ownNumber,
    group: groupNumber,
  };
}

/**
 A command that starts one child of its own in a session of its own, which writes down its own process number and holds
 this command's output while the stop of the command's own process group cannot reach it.
 */
const startsAChildInItsOwnSession = String.raw`
const { spawn } = require('node:child_process');
const record = 'const { writeFileSync } = require("node:fs"); writeFileSync(process.env.ESCAPED_PID_FILE, String(process.pid)); setTimeout(() => {}, 60000);';
spawn(process.execPath, ['--eval', record], { detached: true, stdio: ['ignore', 'inherit', 'inherit'] });
setTimeout(() => {}, 60000);
`;

/**
 A command that starts one child of its own, which writes the record saying it started and, a moment later, the record
 saying it outlived the deadline.

 The child holds the pipe this command's output is read from through `stdio: 'inherit'`, so a deadline that stopped only
 the program this test started is caught twice: the wait lasts until the grace ends it, and the child leaves the record.
 */
const startsAChildThatOutlivesTheDeadline = String.raw`
const { spawn } = require('node:child_process');
const record = 'const { writeFileSync } = require("node:fs"); writeFileSync(process.env.STARTED_FILE, "started"); setTimeout(() => writeFileSync(process.env.SURVIVAL_FILE, "survived"), Number(process.env.SURVIVAL_WRITE_MILLISECONDS)); setTimeout(() => {}, 60000);';
spawn(process.execPath, ['--eval', record], { stdio: ['ignore', 'inherit', 'inherit'] });
setTimeout(() => {}, 60000);
`;

/**
 A command that writes its own process number and the number of the process group it is in, as the kernel reports them.

 The two are equal for a command that leads a process group of its own, and different for one that stays in the group of
 whatever started it.
 */
const reportsItsProcessGroup = String.raw`
const { readFileSync, writeFileSync } = require('node:fs');
const reported = readFileSync('/proc/self/stat', 'utf8');
// After the command's name in parentheses the fields are state, parent and process group, so the group is the third.
const afterTheName = reported.slice(reported.lastIndexOf(')') + 2).split(' ');
writeFileSync(process.env.PROCESS_GROUP_FILE, String(process.pid) + ' ' + afterTheName[2]);
`;

await describe({
  name: '',
  children: [
    describe({
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
          name: 'leads a process group of its own under a deadline, and shares this process\'s group without one',
          fn: async () => {
            /**
             Tree holding the two records of what a stand-in's process group is.
             */
            await using tree = await scratchTree({ name: 'process-group-test-', },);
            /**
             File the bounded command writes its own number and its group's to.
             */
            const boundedGroupFile = join(
              tree.path,
              'bounded-group.txt',
            );
            /**
             File the unbounded command writes the same two numbers to.
             */
            const unboundedGroupFile = join(
              tree.path,
              'unbounded-group.txt',
            );
            await runCommand({
              command: process.execPath,
              args: [
                '--eval',
                reportsItsProcessGroup,
              ],
              capture: true,
              env: {
                ...process.env,
                PROCESS_GROUP_FILE: boundedGroupFile,
              },
              deadlineMilliseconds: testDeadlineMilliseconds,
            },);
            /**
             What the bounded command reported of itself and of its group, which must be one and the same.
             */
            const bounded = processNumbersOf(await readFile(
              boundedGroupFile,
              'utf8',
            ),);
            expect(bounded.group,).toBe(bounded.own,);
            await runCommand({
              command: process.execPath,
              args: [
                '--eval',
                reportsItsProcessGroup,
              ],
              capture: true,
              env: {
                ...process.env,
                PROCESS_GROUP_FILE: unboundedGroupFile,
              },
            },);
            /**
             What a command with no bound reported, whose group is this test's: the control that the equality above can
             fail at all.
             */
            const unbounded = processNumbersOf(await readFile(
              unboundedGroupFile,
              'utf8',
            ),);
            expect(unbounded.group,).not.toBe(unbounded.own,);
          },
          timeout: 20_000,
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
    },),
    describe({
      name: runStartedCommand.name,
      children: [
        //region A bound armed once the command's own child has started

        it({
          name: 'stops the child the command started, so nothing records that it outlived the deadline',
          fn: async () => {
            /**
             Tree holding the record only the command's own child could write.
             */
            await using tree = await scratchTree({ name: 'stopped-command-test-', },);
            /**
             File the command's own child writes once it is running, which this control waits for before it starts the
             clock of the bound it asserts.
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
             The stand-in command, which this control owns and waits on before it starts the clock it asserts.
             */
            await using standIn = ownStandIn({
              command: process.execPath,
              args: [
                '--eval',
                startsAChildThatOutlivesTheDeadline,
              ],
              env: {
                ...process.env,
                STARTED_FILE: startedFile,
                SURVIVAL_FILE: survivalFile,
                SURVIVAL_WRITE_MILLISECONDS: String(survivalWriteMilliseconds,),
              },
            },);
            await waitsForTheChildToStart({
              record: startedFile,
              missing: 'the child of the stand-in command',
            },);
            /**
             The failure of the command the deadline stopped.
             */
            let caught: unknown;
            try {
              await runStartedCommand({
                child: standIn.child,
                step: standIn.step,
                deadlineMilliseconds: groupStopDeadlineMilliseconds,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(NativeVerificationError,);
            expect((caught as Error).message,).toContain(`was still running after ${String(groupStopDeadlineMilliseconds,)} ms`,);
            await wait(survivalWaitMilliseconds,);
            expect(await wroteAFile(survivalFile,),).toBe(false,);
          },
          timeout: 90_000,
        },),
        it({
          name: 'releases the caller at the grace when a child the stop cannot reach still holds the output',
          fn: async () => {
            await using tree = await scratchTree({ name: 'escaped-child-test-', },);
            /**
             File the command's own child writes its process number to, which this control waits for before it starts the
             clock it asserts and reads again to stop the child the stop cannot reach.
             */
            const escapedPidFile = join(
              tree.path,
              'escaped.pid',
            );
            /**
             The stand-in command, which this control owns, and the child it starts outside its process group, which
             this control owns by the number the child writes.
             */
            await using standIn = ownStandIn({
              command: process.execPath,
              args: [
                '--eval',
                startsAChildInItsOwnSession,
              ],
              env: {
                ...process.env,
                ESCAPED_PID_FILE: escapedPidFile,
              },
            },);
            await using escaped = ownEscapedChild({ pidFile: escapedPidFile, },);
            await waitsForTheChildToStart({
              record: escapedPidFile,
              missing: 'the child of the stand-in command that leaves its process group',
            },);
            /**
             When the child that holds the output open had started, which is where the clock of this control begins.
             */
            const startedAtTheChild = Date.now();
            /**
             The failure of a command whose output a child the stop cannot reach keeps open.
             */
            let caught: unknown;
            try {
              await runStartedCommand({
                child: standIn.child,
                step: standIn.step,
                deadlineMilliseconds: groupStopDeadlineMilliseconds,
              },);
            }
            catch (error) {
              caught = error;
            }
            /**
             How long the wait lasted, which only the grace can end.
             */
            const elapsed = Date.now() - startedAtTheChild;
            expect(caught,).toBeInstanceOf(NativeVerificationError,);
            expect((caught as Error).message,).toContain(`was still running after ${String(groupStopDeadlineMilliseconds,)} ms`,);
            expect(elapsed,).toBeLessThan(escapedChildGivesUpMilliseconds,);
          },
          timeout: 90_000,
        },),

        //endregion A bound armed once the command's own child has started
      ],
    },),
  ],
},);
