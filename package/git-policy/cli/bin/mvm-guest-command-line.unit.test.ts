/**
 Guards for the text one guest driver hands a shell: the quoting, the command `mvm exec` is handed, the guest's own
 `timeout` in front of `mvm`'s kill, the two files a detached script runs from and the guest-name check each of them can
 carry, and the status line the host reads back.

 Three sets of guest helpers held these answers before one set did (`bin/guest-route-shell-quote.ts`,
 `bin/guest-route-guest-command.ts`, `bin/mvm-guest-command-line.ts` and `e2e/guest-host/guest-host-rules-fixture.ts`),
 and every case each of them held is stated here against the surviving answer.
 */
import { spawnSync, } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { GUEST_TIMEOUT_MARGIN_SECONDS, } from './mvm-guest-bounds.ts';
import { createGateScratchDirectory, } from './stand-in-gate-programs-for-tests.ts';
import {
  detachedScriptFiles,
  doneStatusOf,
  endedLineHolds,
  guestCommandLine,
  guestSecondsOf,
  secondsOf,
  shellQuote,
} from './mvm-guest-command-line.ts';

/** The guest every case below runs in. */
const GUEST = 'issue-6612-guest';

/** Milliseconds in one second, stated here so the cases read their own arithmetic rather than the module's. */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 The log name the shell cases give the detached script, which also names its step file.
 */
const shellLogName = 'arm.log';

/**
 What one step file run with `sh` left: the shell's own status, and the log the file would have written inside the guest.
 */
type RunStepFile = {
  /**
   The status `sh` exited with, or minus one when a signal ended it, as the package's gate commands report the same case.
   */
  readonly status: number;
  /**
   The log the step file left, or the words saying it left none.
   */
  readonly log: string;
};

/**
 Write the stand-in `hostname` of one case, whose answer is the machine name the case is about.

 The check reads `$(hostname)`, so this is how a command that reached another machine's guest is stood in for, and how the
 guest the file names is stood in for too.

 @param folder - the fixture folder whose `hostname` is written

 @param machineName - the name that `hostname` prints

 @example
 ```ts
 standInHostname({ folder: '/scratch', machineName: 'another-machine' });
 ```
 */
function standInHostname({
  folder,
  machineName,
}: {
  readonly folder: string;
  readonly machineName: string;
},): void {
  writeFileSync(
    join(
      folder,
      'hostname',
    ),
    `#!${process.execPath}\nconsole.log(${JSON.stringify(machineName,)});\n`,
  );
  chmodSync(
    join(
      folder,
      'hostname',
    ),
    ownerOnlyMode,
  );
}

/**
 Run one step-shaped detached script with `sh`, on a machine whose name the case chose.

 The step file is the package's own text with the guest's shared folder replaced by one of the fixture's, so the run
 writes nothing outside the fixture and needs no guest.

 @param folder - the fixture folder the run writes in

 @param machineName - the name that `hostname` prints during the run

 @returns the shell's status and the log the step file left

 @example
 ```ts
 runTheStepFile({ folder: '/scratch', machineName: 'another-machine' });
 ```
 */
function runTheStepFile({
  folder,
  machineName,
}: {
  readonly folder: string;
  readonly machineName: string;
},): RunStepFile {
  /**
   The folder standing in for the guest's `/mnt/shared`.
   */
  const sharedFolder = join(
    folder,
    'shared',
  );
  mkdirSync(sharedFolder, { recursive: true, mode: ownerOnlyMode, },);
  standInHostname({
    folder,
    machineName,
  },);
  /**
   The step file as the module builds it, written where this fixture's shell can run it.
   */
  const stepPath = join(
    sharedFolder,
    `${shellLogName}.step.sh`,
  );
  writeFileSync(
    stepPath,
    detachedScriptFiles({
      name: GUEST,
      script: 'echo the script itself ran',
      log: shellLogName,
      timeoutMilliseconds: 70_000,
      nameCheck: 'step',
    },).step.replaceAll('/mnt/shared/', `${sharedFolder}/`,),
  );
  /**
   The shell's own run of that file.
   */
  const ran = spawnSync('sh', [stepPath,], {
    encoding: 'utf8',
    env: {
      PATH: `${folder}:/usr/bin:/bin`,
      LANG: 'C.UTF-8',
    },
  },);
  /**
   The log the run left, or the words saying it left none.
   */
  const logPath = join(sharedFolder, shellLogName,);
  return {
    status: ran.status ?? (-1),
    log: existsSync(logPath,) ? readFileSync(logPath, 'utf8',) : '(the run left no log)',
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: secondsOf.name,
      children: [
        it({
          name: 'turns a whole number of seconds into itself',
          fn: async () => {
            expect(secondsOf({ milliseconds: 2_400_000, },),).toBe(2_400,);
          },
        },),
        it({
          name: 'rounds a part of a second down, as a shell timeout cannot take it',
          fn: async () => {
            expect(secondsOf({ milliseconds: 1_500, },),).toBe(1,);
            expect(secondsOf({ milliseconds: 999, },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: shellQuote.name,
      children: [
        it({
          name: 'wraps a script in one pair of single quotes',
          fn: async () => {
            expect(shellQuote('zpool list',),).toBe(`'zpool list'`,);
          },
        },),
        it({
          name: 'closes, escapes and reopens each quote the script holds',
          fn: async () => {
            expect(shellQuote(`echo 'hi'`,),).toBe(`'echo '\\''hi'\\'''`,);
          },
        },),
        it({
          name: 'leaves a script with no quote as one word',
          fn: async () => {
            expect(shellQuote('',),).toBe(`''`,);
          },
        },),
        it({
          name: 'leaves dollars, backticks and backslashes for the guest shell to read',
          fn: async () => {
            expect(shellQuote('echo $HOME `id` a\\b',),).toBe(`'echo $HOME \`id\` a\\b'`,);
          },
        },),
        it({
          name: 'keeps a newline inside the one word it hands the guest',
          fn: async () => {
            expect(shellQuote('set -eu\ntrue',),).toBe(`'set -eu\ntrue'`,);
          },
        },),
        //region The word a real shell reads back
        it({
          name: 'hands a shell a script holding a quote, a dollar and a newline, and the shell reads it back whole',
          fn: async () => {
            /**
             A script the shell would read as something else if the quoting around it broke: a quote that would end the
             word early, a dollar and a backtick the shell would expand, and a newline between two commands.
             */
            const script = [
              "printf '%s' \"'\"",
              "printf '%s' '$HOME `id`'",
              "printf '%s' 'two words'",
            ]
              .join('\n',);
            /**
             The line a guest driver's own shell reads: the script quoted whole stands where the guest runs it, so this
             checks the encoding the drivers emit rather than a word handed to `-c` directly.
             */
            const line = `sh -ec ${shellQuote(script,)}`;
            /** The shell's answer to that line. */
            const answered = spawnSync(
              'sh',
              [
                '-c',
                line,
              ],
              { encoding: 'utf8', },
            );
            /**
             What the shell must print: the quote as data, the dollar and the backtick unexpanded, and both words, which
             a word broken anywhere would change.
             */
            const expected = "'$HOME `id`two words";
            expect(answered.status,).toBe(0,);
            expect(answered.stdout,).toBe(expected,);
          },
        },),
        //endregion The word a real shell reads back
      ],
    },),
    describe({
      name: guestSecondsOf.name,
      children: [
        it({
          name: 'subtracts the margin from the whole seconds of the bound',
          fn: async () => {
            expect(guestSecondsOf({ timeoutMilliseconds: 3_600_000, },),).toBe(3_600 - GUEST_TIMEOUT_MARGIN_SECONDS,);
          },
        },),
        it({
          name: 'rounds a part second down, so the guest never outwaits the host',
          fn: async () => {
            expect(guestSecondsOf({ timeoutMilliseconds: 120_900, },),).toBe(120 - GUEST_TIMEOUT_MARGIN_SECONDS,);
          },
        },),
        it({
          name: 'gives a bound no longer than the margin the guest no time to run',
          fn: async () => {
            expect(guestSecondsOf({ timeoutMilliseconds: 60_000, },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: guestCommandLine.name,
      children: [
        it({
          name: 'checks the guest hostname first, then runs the script under the guest timeout',
          fn: async () => {
            expect(guestCommandLine({
              name: GUEST,
              timeoutMilliseconds: 1_800_000,
              script: 'zpool list',
            },),).toBe(`test "$(hostname)" = ${GUEST} && timeout 1740 sh -ec 'zpool list'`,);
          },
        },),
        it({
          name: 'quotes the script so a second shell reads it as one word',
          fn: async () => {
            expect(guestCommandLine({
              name: GUEST,
              timeoutMilliseconds: 120_000,
              script: `echo 'a b'`,
            },),).toBe(`test "$(hostname)" = ${GUEST} && timeout 60 sh -ec 'echo '\\''a b'\\'''`,);
          },
        },),
        it({
          name: 'gives the guest a timeout the margin ahead of the bound mvm is given',
          fn: async () => {
            /** The bound one arm is given, in milliseconds. */
            const bound = 900_000;
            /** The seconds the guest's own timeout allows. */
            const guestSeconds = (bound / MILLISECONDS_PER_SECOND) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              timeoutMilliseconds: bound,
              script: 'true',
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
        it({
          name: 'rounds a bound that is not a whole number of seconds down, so the guest never outlasts mvm',
          fn: async () => {
            /** The bound one arm is given, in milliseconds, which is 900.999 seconds. */
            const bound = 900_999;
            /** The seconds the guest's own timeout allows, rounded down the way the drivers always rounded. */
            const guestSeconds = Math.floor(bound / MILLISECONDS_PER_SECOND,) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              timeoutMilliseconds: bound,
              script: 'true',
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
        it({
          name: 'sizes the guest timeout even when the bound the caller gave is below the margin',
          fn: async () => {
            // The drivers pass a bound of minutes, so this boundary is only about the arithmetic staying the one they
            // always did: a bound below the margin leaves the guest a negative timeout, which `timeout` refuses at once.
            /** The bound one arm is given, in milliseconds, which is below the margin's own seconds. */
            const bound = 30_000;
            /** The seconds the guest's own timeout allows. */
            const guestSeconds = (bound / MILLISECONDS_PER_SECOND) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              timeoutMilliseconds: bound,
              script: 'true',
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
      ],
    },),
    describe({
      name: detachedScriptFiles.name,
      children: [
        it({
          name: 'puts the naming check in the launcher, which refuses a wrong guest before the script starts',
          fn: async () => {
            const files = detachedScriptFiles({
              name: GUEST,
              script: 'echo one',
              log: 'arm.log',
              timeoutMilliseconds: 900_000,
              nameCheck: 'launcher',
            },);
            expect(files.launch.startsWith(`test "$(hostname)" = ${GUEST} || exit 1\n`,),).toBe(true,);
            expect(files.step.includes('hostname',),).toBe(false,);
          },
        },),
        it({
          name: 'puts a check that refuses, and its status line, in the step when the launcher carries none',
          fn: async () => {
            const files = detachedScriptFiles({
              name: GUEST,
              script: 'echo one',
              log: 'arm.log',
              timeoutMilliseconds: 900_000,
              nameCheck: 'step',
            },);
            expect(files.step.startsWith(`test "$(hostname)" = ${GUEST} || { echo "DONE=1" >> /mnt/shared/arm.log; exit 1; }\n`,),).toBe(true,);
            expect(files.launch.includes('hostname',),).toBe(false,);
          },
        },),
        it({
          name: 'bounds the script, sends both streams to the shared log and appends its status',
          fn: async () => {
            expect(detachedScriptFiles({
              name: GUEST,
              script: 'true',
              log: 'build.log',
              timeoutMilliseconds: 70_000,
              nameCheck: 'step',
            },).step,).toBe(
              `test "$(hostname)" = ${GUEST} || { echo "DONE=1" >> /mnt/shared/build.log; exit 1; }\n`
                + `timeout 10 sh -ec 'true' > /mnt/shared/build.log 2>&1\n`
                + 'status=$?\n'
                + 'echo "DONE=$status" >> /mnt/shared/build.log\n',
            );
          },
        },),
        it({
          name: 'starts the step file in its own session holding nothing, and returns at once',
          fn: async () => {
            expect(detachedScriptFiles({
              name: GUEST,
              script: 'true',
              log: 'build.log',
              timeoutMilliseconds: 70_000,
              nameCheck: 'step',
            },).launch,).toBe(
              'setsid nohup sh /mnt/shared/build.log.step.sh < /dev/null > /dev/null 2>&1 &\n'
                + 'exit 0\n',
            );
          },
        },),
        it({
          name: 'gives the guest the bound less the margin its own timeout leaves before a kill',
          fn: async () => {
            const files = detachedScriptFiles({
              name: GUEST,
              script: 'echo one',
              log: 'arm.log',
              timeoutMilliseconds: 900_000,
              nameCheck: 'launcher',
            },);
            const expected = guestSecondsOf({ timeoutMilliseconds: 900_000, },);
            expect(files.step.includes(`timeout ${String(expected,)} sh -ec `,),).toBe(true,);
          },
        },),
        it({
          name: 'carries no name check at all for a driver whose host names the guest it runs in',
          fn: async () => {
            const files = detachedScriptFiles({
              name: GUEST,
              script: 'echo one',
              log: 'arm.log',
              timeoutMilliseconds: 900_000,
              nameCheck: 'none',
            },);
            expect(files.step.includes('hostname',),).toBe(false,);
            expect(files.launch.includes('hostname',),).toBe(false,);
            expect(files.step.startsWith(`timeout `,),).toBe(true,);
          },
        },),
        it({
          name: 'quotes a script holding its own quotes so the guest reads it whole',
          fn: async () => {
            const files = detachedScriptFiles({
              name: GUEST,
              script: `grep --quiet --fixed-strings 'ARM done' /root/arm.log`,
              log: 'arm.log',
              timeoutMilliseconds: 900_000,
              nameCheck: 'launcher',
            },);
            expect(files.step.includes(`sh -ec 'grep --quiet --fixed-strings '\\''ARM done'\\'' /root/arm.log' > /mnt/shared/arm.log 2>&1`,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'a step-shaped check run with sh',
      children: [
        it({
          name: 'refuses at once, and writes the status line the host waits for, on a machine of another name',
          fn: async () => {
            await using scratch = await createGateScratchDirectory({ prefix: 'mvm-guest-command-line-other-machine-', },);
            /**
             What the step file left on a machine whose name is not the guest's.
             */
            const run = runTheStepFile({
              folder: scratch.path,
              machineName: 'another-machine',
            },);
            expect(run.status,).toBe(1,);
            expect(run.log,).toBe('DONE=1\n',);
          },
        },),
        it({
          name: 'runs the script on the guest it names, which is what the refusal is told apart from',
          fn: async () => {
            await using scratch = await createGateScratchDirectory({ prefix: 'mvm-guest-command-line-its-guest-', },);
            /**
             What the step file left on the guest it names.
             */
            const run = runTheStepFile({
              folder: scratch.path,
              machineName: GUEST,
            },);
            expect(run.status,).toBe(0,);
            expect(run.log,).toBe('the script itself ran\nDONE=0\n',);
          },
        },),
      ],
    },),
    describe({
      name: endedLineHolds.name,
      children: [
        it({
          name: 'reads a status line whatever status it carries',
          fn: async () => {
            expect(endedLineHolds({ text: 'DONE=0\n', },),).toBe(true,);
            expect(endedLineHolds({ text: 'DONE=1\n', },),).toBe(true,);
          },
        },),
        it({
          name: 'reads a line that is not a status line as a script still working',
          fn: async () => {
            expect(endedLineHolds({ text: 'Compiling zbus\n', },),).toBe(false,);
            expect(endedLineHolds({ text: '', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: doneStatusOf.name,
      children: [
        it({
          name: 'reads the digits of the first status line',
          fn: async () => {
            expect(doneStatusOf({ text: 'building\nDONE=1\nmore\n', },),).toEqual({
              found: true,
              status: '1',
            },);
            expect(doneStatusOf({ text: 'DONE=0\nDONE=1\n', },),).toEqual({
              found: true,
              status: '0',
            },);
          },
        },),
        it({
          name: 'reads a status that is more than one digit',
          fn: async () => {
            expect(doneStatusOf({ text: 'DONE=17\n', },),).toEqual({
              found: true,
              status: '17',
            },);
          },
        },),
        it({
          name: 'answers that a log with no status line names no status',
          fn: async () => {
            expect(doneStatusOf({ text: 'building\n', },),).toEqual({ found: false, },);
            expect(doneStatusOf({ text: '', },),).toEqual({ found: false, },);
          },
        },),
        it({
          name: 'refuses a status word that is not only decimal digits',
          fn: async () => {
            expect(doneStatusOf({ text: 'DONE=x\n', },),).toEqual({ found: false, },);
            expect(doneStatusOf({ text: 'DONE=1x\n', },),).toEqual({ found: false, },);
            expect(doneStatusOf({ text: 'xDONE=1\n', },),).toEqual({ found: false, },);
            expect(doneStatusOf({ text: 'DONE=\n', },),).toEqual({ found: false, },);
          },
        },),
        it({
          name: 'reads the first status line that names a whole number',
          fn: async () => {
            expect(doneStatusOf({ text: 'DONE=0x1\nDONE=2\n', },),).toEqual({
              found: true,
              status: '2',
            },);
          },
        },),
        it({
          name: 'does not read a status from a line that only mentions one',
          fn: async () => {
            expect(doneStatusOf({ text: 'the script wrote DONE=0\n', },),).toEqual({ found: false, },);
          },
        },),
      ],
    },),
  ],
},);
