/**
 Guards for the two steps that start one guest script detached and read its end later: what the start does with the
 launcher's own status, and what the wait reads out of the guest's log.

 The launcher's status is the only news this host has that the script was started at all: the launcher's own text returns
 as soon as it has detached the step file, so a launcher that was refused (a command that reached another machine's
 guest, or a guest that could not be asked) exits nonzero and nothing in the guest will ever write the log. The cases
 below run a driver program of their own, whose guest is a stand-in `mvm` on the fixture's own search path, so no guest
 is started and the launcher's status is the case's to choose.
 */
import { spawn, } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as sleep, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createGateScratchDirectory, } from './stand-in-gate-programs-for-tests.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Milliseconds a case waits for the driver program to end.
 */
const driverEndWaitMilliseconds = 60_000;

/**
 The guest every case names, whose name the refusal has to carry: a launcher refused for another machine's guest is
 exactly the failure a person has to be able to read the guest's own name out of.
 */
const guestName = 'guest-a';

/**
 The log name every case gives the detached script, which also names its two files.
 */
const logName = 'build.log';

/**
 The words a refusal about the launcher has to carry, which say what was refused rather than what the run then waited
 for.
 */
const launcherRefusedWords = 'the launcher was refused';

/**
 The words a wait that reached its bound says, which a refusal at the launcher must never be reported as.
 */
const waitRefusedWords = 'the guest script did not finish inside';

/**
 The status a case answers the launcher's `exec` with when it is about a launcher that was refused.
 */
const REFUSED_STATUS = 1;

/**
 The program run as the driver: it starts one detached guest script, as an adoption arm does, and then, when the start
 returned, waits for the script's status line with the shortest bound a wait can be given.

 The two refusals are printed differently, so a case tells one from the other by what was printed rather than by a
 duration.
 */
const driverProgram = `
const { startDetachedGuestScript, awaitDetachedGuestScript } = await import(process.env.DETACHED_SCRIPT_MODULE_URL);
const chosen = { name: process.env.GUEST_NAME, logDir: process.env.LOG_DIR };
try {
  await startDetachedGuestScript({
    chosen,
    script: 'true',
    log: process.env.LOG_NAME,
    timeoutMilliseconds: 900000,
    nameCheck: process.env.NAME_CHECK,
  });
} catch (refusal) {
  console.log('the start refused: ' + refusal.message);
  process.exit(0);
}
console.log('the start returned');
try {
  await awaitDetachedGuestScript({
    chosen,
    log: process.env.LOG_NAME,
    finishTimeoutMilliseconds: 0,
    lastLine: async () => '',
    pull: async () => '',
  });
  console.log('the wait read a log');
} catch (refusal) {
  console.log('the wait refused: ' + refusal.message);
}
`;

/**
 What one driver program printed, and the calls its stand-in `mvm` was given.
 */
type DrivenStart = {
  /**
   The driver program's whole output, both streams.
   */
  readonly printed: string;
  /**
   The words of each `mvm` call the stand-in was given, in the order it was given them.
   */
  readonly calls: readonly (readonly string[])[];
};

/**
 Write the stand-in `mvm` of one case, which records every call and answers the launcher's `exec` with the status the
 case chose.

 The stand-in is written for these cases and says nothing about a real guest: what a real `mvm exec` answers is the
 guest's own exit status (`package/mvm/cli/src/cli.ts` hands `result.exitCode` to `process.exitCode`), which is what a
 launcher's `test "$(hostname)" = NAME || exit 1` is on another machine.

 @param folder - the fixture folder whose `bin` holds the stand-in

 @param execStatus - the status the stand-in exits with for an `exec`, and zero for everything else

 @returns where the stand-in records the calls it was given

 @example
 ```ts
 writeStandInMvm({ folder: '/scratch', execStatus: 1 });
 ```
 */
function writeStandInMvm({
  folder,
  execStatus,
}: {
  readonly folder: string;
  readonly execStatus: number;
},): string {
  /**
   The file the stand-in appends one JSON list of words to, per call.
   */
  const callsPath = join(
    folder,
    'mvm-calls.jsonl',
  );
  writeFileSync(callsPath, '',);
  writeFileSync(
    join(
      folder,
      'bin',
      'mvm',
    ),
    `#!${process.execPath}
import { appendFileSync } from 'node:fs';
appendFileSync(process.env.MVM_CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
process.exit(process.argv.slice(2).includes('exec') ? ${String(execStatus,)} : 0);
`,
  );
  chmodSync(
    join(
      folder,
      'bin',
      'mvm',
    ),
    ownerOnlyMode,
  );
  return callsPath;
}

/**
 Run the driver program once against a stand-in `mvm` whose launcher status the case chose.

 @param scratch - the folder this case owns

 @param execStatus - the status the stand-in answers the launcher's `exec` with

 @returns what the driver program printed, and the calls the stand-in was given

 @example
 ```ts
 await driveTheStart({ scratch: '/scratch', execStatus: 1 });
 ```
 */
async function driveTheStart({
  scratch,
  execStatus,
}: {
  readonly scratch: string;
  readonly execStatus: number;
},): Promise<DrivenStart> {
  /**
   The folder the fixture's own `bin` and the run's logs live in.
   */
  const folder = join(
    scratch,
    `launcher-${String(execStatus,)}`,
  );
  /**
   The folder the driver program writes the detached script's two files into.
   */
  const logDirectory = join(
    folder,
    'logs',
  );
  await mkdir(
    join(
      folder,
      'bin',
    ),
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  await mkdir(
    logDirectory,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Where the stand-in `mvm` records its calls.
   */
  const callsPath = writeStandInMvm({
    folder,
    execStatus,
  },);
  /**
   The driver program, started as a program of its own because its search path names the fixture's stand-in.
   */
  const child = spawn(
    process.execPath,
    [
      '--input-type=module-typescript',
      '--eval',
      driverProgram,
    ],
    {
      cwd: folder,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      env: {
        PATH: `${join(folder, 'bin',)}:/usr/bin:/bin`,
        HOME: folder,
        LANG: 'C.UTF-8',
        DETACHED_SCRIPT_MODULE_URL: `file://${join(import.meta.dirname, 'mvm-guest-detached-script.ts',)}`,
        GUEST_NAME: guestName,
        LOG_DIR: logDirectory,
        LOG_NAME: logName,
        NAME_CHECK: 'launcher',
        MVM_CALLS: callsPath,
      },
    },
  );
  /**
   Everything the driver program printed, both streams together.
   */
  const output: string[] = [];
  child.stdout?.on('data', function onOut(chunk: Buffer) {
    output.push(chunk.toString(),);
  },);
  child.stderr?.on('data', function onErr(chunk: Buffer) {
    output.push(chunk.toString(),);
  },);
  /**
   The moment the driver program's streams have ended, taken before the wait so no event is missed.
   */
  const streamsEnded = new Promise<void>(function waitForTheStreams(resolve) {
    child.once('close', function closed() {
      resolve();
    },);
  },);
  /**
   The driver program's own end, which these cases wait for so the stand-in has been asked everything it will be.
   */
  const ended = await Promise.race([
    new Promise<{ readonly code: number }>(function waitForTheEnd(resolve) {
      child.once(
        'exit',
        function exited(code) {
          resolve({ code: code ?? (-1), },);
        },
      );
    },),
    sleep(driverEndWaitMilliseconds,)
      .then(function gaveUp(): never {
        child.kill('SIGKILL',);
        throw new Error(`this case waited ${String(driverEndWaitMilliseconds,)} ms for the driver program to end`,);
      },),
  ],);
  await streamsEnded;
  /**
   Everything the driver program printed, joined.
   */
  const printed = output.join('',);
  if (ended.code !== 0)
    throw new Error(`the driver program exited ${String(ended.code,)} and printed: ${printed}`,);
  return {
    printed,
    calls: readFileSync(callsPath, 'utf8',)
      .split('\n',)
      .filter(function isACall(line: string,): boolean {
        return line.length > 0;
      },)
      .map(function wordsOf(line: string,): readonly string[] {
        return JSON.parse(line,) as readonly string[];
      },),
  };
}

await describe({
  name: 'mvm-guest-detached-script',
  children: [
    it({
      name: 'refuses at once when the launcher exited nonzero, naming the guest and the status',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'mvm-guest-detached-refused-', },);
        /**
         The folder this case's own run wrote in, which its log folder sits in.
         */
        const folder = join(scratch.path, `launcher-${String(REFUSED_STATUS,)}`,);
        /**
         What the driver program printed, and what its stand-in `mvm` was asked.
         */
        const driven = await driveTheStart({
          scratch: scratch.path,
          execStatus: REFUSED_STATUS,
        },);
        expect(driven.printed,).toContain('the start refused:',);
        expect(driven.printed,).toContain(launcherRefusedWords,);
        expect(driven.printed,).toContain(`exited ${String(REFUSED_STATUS,)}`,);
        expect(driven.printed,).toContain(guestName,);
        expect(driven.printed,).not.toContain('the start returned',);
        expect(driven.printed,).not.toContain(waitRefusedWords,);
        // The two pushes and the launcher, and nothing after it: a start that refused asks the guest for nothing more.
        expect(driven.calls.map(function wordsOfCall(callWords: readonly string[],): string {
          return callWords.join(' ',);
        },),).toEqual([
          `--backend libvirt push ${guestName} ${join(folder, 'logs', `${logName}.step.sh`,)} /shared/${logName}.step.sh`,
          `--backend libvirt push ${guestName} ${join(folder, 'logs', `${logName}.launch.sh`,)} /shared/${logName}.launch.sh`,
          `--backend libvirt exec ${guestName} -- sh /mnt/shared/${logName}.launch.sh`,
        ],);
        // The two files the launcher runs were written before it was started, so the refusal is the guest's and not this
        // host's.
        /**
         The step file the run wrote before it started the launcher.
         */
        const stepFile = join(folder, 'logs', `${logName}.step.sh`,);
        /**
         The launcher file the run wrote before it started the launcher.
         */
        const launchFile = join(folder, 'logs', `${logName}.launch.sh`,);
        expect(existsSync(stepFile,),).toBe(true,);
        expect(existsSync(launchFile,),).toBe(true,);
      },
    },),
    it({
      name: 'waits for the script when the launcher exited zero, which is what the refusal case is told apart from',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'mvm-guest-detached-started-', },);
        /**
         What the driver program printed, and what its stand-in `mvm` was asked.
         */
        const driven = await driveTheStart({
          scratch: scratch.path,
          execStatus: 0,
        },);
        expect(driven.printed,).toContain('the start returned',);
        expect(driven.printed,).not.toContain(launcherRefusedWords,);
        expect(driven.printed,).toContain(waitRefusedWords,);
      },
    },),
  ],
},);
