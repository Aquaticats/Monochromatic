/**
 Guards for one step command a guest driver runs: the whole of its output in one named log, its own bound, and the stop a
 signal that ends the driver makes, so that no step of an interrupted run is left working with nobody reading it.

 Every case that sends a signal runs the driver as a program of its own. A signal's disposition belongs to the whole
 process, and this file's process is the test run's, which no case may change: the driver is started as the leader of a
 process group, which is what a foreground job is, and the signal goes to that group, which is what a terminal sends.
 */
import { spawn, } from 'node:child_process';
import {
  closeSync,
  existsSync,
  openSync,
  readFileSync,
} from 'node:fs';
import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as sleep, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stopProcessGroup, } from './process-group-stop.ts';
import { createGateScratchDirectory, } from './stand-in-gate-programs-for-tests.ts';

/**
 Search path every driver program of these cases is given, which holds no fixture program of its own.
 */
const systemSearchPath = '/usr/bin:/bin';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 How often a case looks again for something it waits for, in milliseconds.
 */
const pollMilliseconds = 20;

/**
 Milliseconds a case waits for the driver program to start its step.
 */
const stepStartWaitMilliseconds = 30_000;

/**
 Milliseconds a case waits for a stopped step's process group to be gone.
 */
const stepEndWaitMilliseconds = 30_000;

/**
 Milliseconds a case waits for the driver program to end after the signal.
 */
const driverEndWaitMilliseconds = 30_000;

/**
 The seconds the step's stand-in command sleeps, which is far longer than any bound of these cases: only a stop ends it.
 */
const stepSleepSeconds = '600';

/**
 The bound the step is given, which is the two hours a guest build may take, so no bound of the step is what ends it.
 */
const stepBoundMilliseconds = 7_200_000;

/**
 The first words of the line an interrupted driver says, which is what a person reads after pressing the interrupt key.
 */
const saidPrefix = 'git-policy guest step:';

/**
 The value a status the case never read is written as, the sentinel the package's own gate commands use for the same case.
 */
const NO_STATUS = -1;

/**
 The signals whose arrival has to stop the step a driver is still waiting for: a terminal's interrupt, and the request to
 stop a program that a service manager or a shell sends.
 */
const interruptSignals: readonly NodeJS.Signals[] = [
  'SIGINT',
  'SIGTERM',
];

/**
 The program run as the driver in the interrupt cases: it starts one step command, and waits for that command's answer.

 Everything it reads comes from its environment, so the case can hand it a log folder of its own and a command whose
 duration no bound of this file ends.
 */
const driverProgram = `
const { runStepCommand } = await import(process.env.STEP_MODULE_URL);
await runStepCommand({
  logDirectory: process.env.LOG_DIR,
  program: process.env.STEP_PROGRAM,
  argumentWords: process.env.STEP_WORDS.split(' '),
  log: 'step.log',
  timeoutMilliseconds: Number(process.env.STEP_BOUND_MS),
});
`;

/**
 The name, inside the driver's log folder, of the record a driver writes once its step has ended.

 The record is what the case waits for, because a step that ends by itself is gone before a look at the process table can
 catch it: what the case is about is a driver that has had a step and has none now.
 */
const stepEndedRecordName = 'step-ended';

/**
 The program run as the driver in the case where the step has already ended: it starts one short step command, writes
 that the step ended, and stays alive with nothing of its own running.
 */
const driverAfterItsStepEndedProgram = `
const { runStepCommand } = await import(process.env.STEP_MODULE_URL);
const { writeFileSync } = await import('node:fs');
await runStepCommand({
  logDirectory: process.env.LOG_DIR,
  program: 'true',
  argumentWords: [],
  log: 'step.log',
  timeoutMilliseconds: Number(process.env.STEP_BOUND_MS),
});
writeFileSync(process.env.LOG_DIR + '/' + process.env.STEP_ENDED_RECORD, 'the step ended');
// A timer keeps this program alive with nothing of its own running: a top-level await is not unsettled work, and node
// ends a program whose last top-level await never settles.
setTimeout(() => {}, 600000);
`;

/**
 What one interrupt case read once the signal had ended its driver.
 */
type InterruptedDriver = {
  /**
   Whether the step was still running when the signal was sent, which is what makes the rest of the case mean anything.
   */
  readonly stepWasAliveBeforeTheSignal: boolean;
  /**
   The signal the driver ended by, as the process table reported it, or empty text when it exited by itself.
   */
  readonly endingSignal: string;
  /**
   Whether the step's process group was gone once the driver had ended.
   */
  readonly stepIsGone: boolean;
  /**
   The step's own process number, which is also the number of the group it leads.
   */
  readonly stepPid: number;
  /**
   Everything the driver wrote to its standard error.
   */
  readonly said: string;
};

/**
 Whether a caught value is the kernel's answer for a process or a group that is no longer there.

 @param value - the caught value

 @returns true for a system error whose code is `ESRCH`

 @example
 ```ts
 isGoneFromTheProcessTable(Object.assign(new Error('no such process'), { code: 'ESRCH' }));
 ```
 */
function isGoneFromTheProcessTable(value: unknown,): boolean {
  return ((typeof value) === 'object') && (value !== null) && ('code' in value) && (value.code === 'ESRCH');
}

/**
 Whether one process number still names a live process.

 A process the kernel holds in a state no signal ends still answers here, which is why a case reads the group's absence
 rather than the command's exit status.

 @param pid - the process number to ask about

 @returns true when the kernel still has that process

 @example
 ```ts
 alive({ pid: 1234 });
 ```
 */
function alive({ pid, }: { readonly pid: number; },): boolean {
  try {
    process.kill(pid, 0,);
    return true;
  }
  catch (error) {
    if (!isGoneFromTheProcessTable(error,))
      throw error;
    return false;
  }
}

/**
 End one process group this file knows only the number of, and say nothing about a group that has already gone.

 @param pid - the process number, which is the number of the group that process leads

 @example
 ```ts
 endTheGroupOf({ pid: 1234 });
 ```
 */
function endTheGroupOf({ pid, }: { readonly pid: number; },): void {
  try {
    process.kill(-pid, 'SIGKILL',);
  }
  catch (error) {
    if (!isGoneFromTheProcessTable(error,))
      throw error;
  }
}

/**
 The process numbers of the children the given process's main thread holds.

 A driver program starts its step from that thread, and a step of `runStepCommand`'s shape leads a process group of its
 own, so the number read here is both the command's and its group's.

 @param pid - the process whose children are read

 @returns the children's process numbers, in the order the kernel lists them

 @example
 ```ts
 childrenOf({ pid: 1234 });
 ```
 */
function childrenOf({ pid, }: { readonly pid: number; },): readonly number[] {
  /**
   The kernel's own list for that thread, which is one line of space-separated numbers.
   */
  const listed = readFileSync(`/proc/${String(pid,)}/task/${String(pid,)}/children`, 'utf8',);
  return listed
    .trim()
    .split(' ',)
    .filter(function isANumber(word: string,): boolean {
      return word.length > 0;
    },)
    .map(Number,);
}

/**
 Look again, with a bound, until what the case waits for is there.

 Each look waits for the one before it, so the next look is asked for here rather than by a loop around both: a loop
 holding an `await` would read as work that could have run beside itself, and this one cannot.

 @param what - what the case waits for, named in the failure

 @param boundMilliseconds - how long the wait may last

 @param found - one look, answering whether the thing waited for is there

 @param began - when the wait began, in milliseconds, which only the wait's own next look passes

 @throws Error naming what the wait was for and how long it lasted

 @example
 ```ts
 await waitUntil({ what: 'the step to start', boundMilliseconds: 30_000, found: () => false });
 ```
 */
async function waitUntil({
  what,
  boundMilliseconds,
  found,
  began = Date.now(),
}: {
  readonly what: string;
  readonly boundMilliseconds: number;
  readonly found: () => boolean;
  readonly began?: number;
},): Promise<void> {
  if (found())
    return;
  if ((Date.now() - began) >= boundMilliseconds)
    throw new Error(`this case waited ${String(boundMilliseconds,)} ms for ${what}, and it never came`,);
  await sleep(pollMilliseconds,);
  await waitUntil({
    what,
    boundMilliseconds,
    found,
    began,
  },);
}

/**
 Whether the step's process group was gone once the driver had ended, and the end of that group when it was not.

 A case that read a step still alive has to leave nothing of it behind, so a group that outlasted its bound is ended here
 and the case goes on to fail on what it read.

 @param stepPid - the step's own process number, which is the number of the group it leads

 @returns true when the group was gone inside the bound, false when this had to end it

 @example
 ```ts
 await theStepGroupEnded({ stepPid: 1234 });
 ```
 */
async function theStepGroupEnded({ stepPid, }: { readonly stepPid: number; },): Promise<boolean> {
  try {
    await waitUntil({
      what: "the step's process group to end",
      boundMilliseconds: stepEndWaitMilliseconds,
      found: function theStepIsGone() {
        return !alive({ pid: stepPid, },);
      },
    },);
    return true;
  }
  catch (error) {
    endTheGroupOf({ pid: stepPid, },);
    if (!Error.isError(error,))
      throw error;
    return false;
  }
}

/**
 How one driver program ended.

 @param child - the started driver

 @param boundMilliseconds - how long the case waits for it to end

 @returns its exit status, and the signal that ended it as empty text when it exited by itself

 @example
 ```ts
 await endedWithin({ child, boundMilliseconds: 30_000 });
 ```
 */
async function endedWithin({
  child,
  boundMilliseconds,
}: {
  readonly child: ReturnType<typeof spawn>;
  readonly boundMilliseconds: number;
},): Promise<{ readonly code: number; readonly signal: string }> {
  return await Promise.race([
    new Promise<{ readonly code: number; readonly signal: string }>(function waitForTheEnd(resolve) {
      child.once(
        'exit',
        function exited(code, signal) {
          resolve({
            code: code ?? NO_STATUS,
            signal: signal ?? '',
          },);
        },
      );
    },),
    sleep(boundMilliseconds,)
      .then(function gaveUp(): never {
        throw new Error(`this case waited ${String(boundMilliseconds,)} ms for the driver program to end`,);
      },),
  ],);
}

/**
 Run one driver program as the leader of a process group, and hand the case a way to end it whatever happens.

 @param program - the program the driver runs

 @param scratch - the folder this case owns, which holds the driver's log folder and its standard error

 @returns the started driver, its own process number, and the way to end it

 @example
 ```ts
 await using driver = await startDriver({ program: driverProgram, scratch: '/scratch' });
 ```
 */
async function startDriver({
  program,
  scratch,
}: {
  readonly program: string;
  readonly scratch: string;
},): Promise<{
  readonly child: ReturnType<typeof spawn>;
  readonly pid: number;
  readonly stderrPath: string;
  readonly logDirectory: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   The step's own log folder, which the driver is told to use.
   */
  const logDirectory = join(
    scratch,
    'logs',
  );
  await mkdir(
    logDirectory,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   The file the driver's standard error goes to, so a step that outlives it cannot hold this case's output open.
   */
  const stderrPath = join(
    scratch,
    'driver.stderr',
  );
  /**
   The descriptor that file is opened at, closed once the driver holds it.
   */
  const stderrFile = openSync(
    stderrPath,
    'w',
  );
  /**
   The started driver, leading a process group of its own so a signal sent to that group is a terminal's interrupt.
   */
  const child = spawn(
    process.execPath,
    [
      '--input-type=module-typescript',
      '--eval',
      program,
    ],
    {
      cwd: scratch,
      detached: true,
      stdio: [
        'ignore',
        'ignore',
        stderrFile,
      ],
      env: {
        PATH: systemSearchPath,
        HOME: scratch,
        LANG: 'C.UTF-8',
        STEP_MODULE_URL: `file://${join(import.meta.dirname, 'mvm-guest-step-command.ts',)}`,
        LOG_DIR: logDirectory,
        STEP_PROGRAM: 'sleep',
        STEP_WORDS: stepSleepSeconds,
        STEP_BOUND_MS: String(stepBoundMilliseconds,),
        STEP_ENDED_RECORD: stepEndedRecordName,
      },
    },
  );
  closeSync(stderrFile,);
  /**
   The driver's own process number, which is the number of the group it leads.
   */
  const { pid, } = child;
  if (pid === undefined)
    throw new Error('the driver program was started without a process number',);
  return {
    child,
    pid,
    stderrPath,
    logDirectory,
    async [Symbol.asyncDispose](): Promise<void> {
      // Whatever this case left, on every path: the group the driver leads, with the driver itself in it.
      stopProcessGroup({ child, },);
    },
  };
}

/**
 End one driver program with one signal, as a terminal ends the foreground job, and read what that did to its step.

 @param signal - the signal the driver's process group is sent

 @param scratch - the folder this case owns

 @returns what the case read once the driver had ended

 @example
 ```ts
 await interruptAStartedStep({ signal: 'SIGINT', scratch: '/scratch' });
 ```
 */
async function interruptAStartedStep({
  signal,
  scratch,
}: {
  readonly signal: NodeJS.Signals;
  readonly scratch: string;
},): Promise<InterruptedDriver> {
  await using driver = await startDriver({
    program: driverProgram,
    scratch,
  },);
  await waitUntil({
    what: 'the driver program to start its step',
    boundMilliseconds: stepStartWaitMilliseconds,
    found: function theStepIsListed() {
      return childrenOf({ pid: driver.pid, },).length > 0;
    },
  },);
  /**
   The step's own process number, once the driver has started one.
   */
  const [stepPid,] = childrenOf({ pid: driver.pid, },);
  if (stepPid === undefined)
    throw new Error('the driver program listed no step of its own',);
  /**
   Whether the step was still running when the signal was sent, which a case reads before the signal because only a live
   step can show whether a signal reached it.
   */
  const stepWasAliveBeforeTheSignal = alive({ pid: stepPid, },);
  /**
   How the driver ended, once the signal had reached its group.
   */
  const ending = endedWithin({
    child: driver.child,
    boundMilliseconds: driverEndWaitMilliseconds,
  },);
  process.kill(-driver.pid, signal,);
  /**
   What the driver's own end reported.
   */
  const ended = await ending;
  return {
    stepWasAliveBeforeTheSignal,
    endingSignal: ended.signal,
    stepIsGone: await theStepGroupEnded({ stepPid, },),
    stepPid,
    said: readFileSync(driver.stderrPath, 'utf8',),
  };
}

await describe({
  name: 'mvm-guest-step-command',
  children: [
    describe({
      name: 'a signal that ends a driver',
      children: interruptSignals.map(function aSignalCase(signal: NodeJS.Signals,) {
        return it({
          name: `stops the step it started when ${signal} ends the driver, and names what it stopped`,
          fn: async () => {
            await using scratch = await createGateScratchDirectory({ prefix: `mvm-guest-step-${signal.toLowerCase()}-`, },);
            /**
             What the case read once the driver had ended.
             */
            const interrupted = await interruptAStartedStep({
              signal,
              scratch: scratch.path,
            },);
            expect(interrupted.stepWasAliveBeforeTheSignal,).toBe(true,);
            expect(interrupted.endingSignal,).toBe(signal,);
            expect(interrupted.stepIsGone,).toBe(true,);
            expect(interrupted.said,).toContain(`${saidPrefix} ${signal}`,);
            expect(interrupted.said,).toContain(`sleep ${stepSleepSeconds} (process group ${String(interrupted.stepPid,)})`,);
          },
        },);
      },),
    },),
    describe({
      name: 'a step that has already ended',
      children: [
        it({
          name: 'stops and names no step that had already ended',
          fn: async () => {
            await using scratch = await createGateScratchDirectory({ prefix: 'mvm-guest-step-ended-', },);
            await using driver = await startDriver({
              program: driverAfterItsStepEndedProgram,
              scratch: scratch.path,
            },);
            // The driver's own record of its step's end is what this waits for, so the signal finds a driver that has had
            // a step and has none now.
            await waitUntil({
              what: "the driver program's record that its step ended",
              boundMilliseconds: stepStartWaitMilliseconds,
              found: function theStepEnded() {
                return existsSync(join(driver.logDirectory, stepEndedRecordName,),);
              },
            },);
            /**
             How the driver ended, once the signal had reached its group.
             */
            const ending = endedWithin({
              child: driver.child,
              boundMilliseconds: driverEndWaitMilliseconds,
            },);
            process.kill(-driver.pid, 'SIGINT',);
            /**
             What the driver's own end reported.
             */
            const ended = await ending;
            expect(ended.signal,).toBe('SIGINT',);
            // A driver with nothing of its own running ends as it always did, silently: the entry a step leaves is
            // forgotten when its wait answers, so a signal neither stops it again nor names a command that has ended.
            expect(readFileSync(driver.stderrPath, 'utf8',),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
