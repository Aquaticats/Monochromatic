import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  doneStatusOf,
  detachedScriptFiles,
  guestCommandTimeoutSeconds,
  guestTimeoutMarginSeconds,
  holdsLineStartingWith,
  requireMarkers,
  runArguments,
  shellQuote,
} from './guest-host-rules-fixture.ts';

/**
 One guest a test drives.

 @returns the guest's name and the folder its logs go in
 */
function chosen(): { readonly name: string; readonly logDir: string } {
  return {
    name: 'guest-one',
    logDir: 'target/logs',
  };
}

/**
 The markers a test log is asked for.

 @returns two lines a passing log carries
 */
function markers(): readonly string[] {
  return [
    'ARM started',
    'ARM done',
  ];
}

await describe({
  name: '',
  children: [
    //region The command line
    describe({
      name: runArguments.name,
      children: [
        it({
          name: 'takes the first word as the step',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--name',
              'guest-one',
            ], },).step,).toBe('run');
          },
        },),
        it({
          name: 'gives an empty step for a command line with no words',
          fn: async () => {
            expect(runArguments({ argv: [], },).step,).toBe('');
          },
        },),
        it({
          name: 'reads the word after a flag as that flag value',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--log-dir',
              'target/logs',
            ], },).values.get('--log-dir',),).toBe('target/logs');
          },
        },),
        it({
          name: 'keeps the first occurrence of a flag, which is the one the drivers always read',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--name',
              'first',
              '--name',
              'second',
            ], },).values.get('--name',),).toBe('first');
          },
        },),
        it({
          name: 'reads no value for a flag that is the last word',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--keep-guest',
            ], },).values.has('--keep-guest',),).toBe(false);
          },
        },),
        it({
          name: 'reads a flag-shaped word as the value of the flag before it, as the drivers always did',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--name',
              '--keep-guest',
            ], },).values.get('--name',),).toBe('--keep-guest');
          },
        },),
        it({
          name: 'holds every word after the step as a flag, so a bare flag is one lookup',
          fn: async () => {
            expect(runArguments({ argv: [
              'run',
              '--no-build',
              '--keep-guest',
            ], },).flags,).toEqual(new Set([
              '--no-build',
              '--keep-guest',
            ],),);
          },
        },),
        it({
          name: 'leaves the step out of the bare flags',
          fn: async () => {
            expect(runArguments({ argv: ['run',], },).flags.has('run',),).toBe(false);
          },
        },),
      ],
    },),
    //endregion The command line

    //region The quoting
    describe({
      name: shellQuote.name,
      children: [
        it({
          name: 'wraps a plain script in single quotes',
          fn: async () => {
            expect(shellQuote('echo one',),).toBe('\'echo one\'');
          },
        },),
        it({
          name: 'ends the quoted run at a single quote and escapes it, which is the shape sh documents',
          fn: async () => {
            expect(shellQuote('echo it\'s here',),).toBe(String.raw`'echo it'\''s here'`);
          },
        },),
        it({
          name: 'leaves double quotes, dollars and backticks alone',
          fn: async () => {
            expect(shellQuote('echo "$1" `date` $HOME',),).toBe('\'echo "$1" `date` $HOME\'');
          },
        },),
        it({
          name: 'keeps a newline inside the quoted word',
          fn: async () => {
            expect(shellQuote('one\ntwo',),).toBe('\'one\ntwo\'');
          },
        },),
        it({
          name: 'quotes a trailing single quote without leaving the run open',
          fn: async () => {
            expect(shellQuote('test \'',),).toBe(String.raw`'test '\'''`);
          },
        },),
      ],
    },),
    //endregion The quoting

    //region What a pulled log has to prove
    describe({
      name: requireMarkers.name,
      children: [
        it({
          name: 'passes a log that carries every marker',
          fn: async () => {
            expect(function checks(): void {
              requireMarkers({
                pulled: 'ARM started\nARM done\n',
                markers: markers(),
                subject: 'the arm log',
                because: 'the arm did not run',
              },);
            },).not.toThrow();
          },
        },),
        it({
          name: 'names the first marker the log lacks and why a missing marker fails',
          fn: async () => {
            expect(function checks(): void {
              requireMarkers({
                pulled: 'ARM started\n',
                markers: markers(),
                subject: 'the arm log',
                because: 'the arm did not run',
              },);
            },).toThrow('the arm log lacks ARM done; the arm did not run');
          },
        },),
      ],
    },),
    describe({
      name: holdsLineStartingWith.name,
      children: [
        it({
          name: 'finds a line that starts with the word',
          fn: async () => {
            expect(holdsLineStartingWith({
              pulled: 'SKIPPED the adoption arm\n',
              word: 'SKIPPED',
            },),).toBe(true);
          },
        },),
        it({
          name: 'reads only a line start, not a word inside a line',
          fn: async () => {
            expect(holdsLineStartingWith({
              pulled: 'the arm answered SKIPPED\n',
              word: 'SKIPPED',
            },),).toBe(false);
          },
        },),
        it({
          name: 'finds a word that starts the first line, which an anchored pattern once missed',
          fn: async () => {
            expect(holdsLineStartingWith({
              pulled: 'SKIPPED\nARM done\n',
              word: 'SKIP',
            },),).toBe(true);
          },
        },),
      ],
    },),
    describe({
      name: doneStatusOf.name,
      children: [
        it({
          name: 'reads the status a finished script appended',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'arm output\nDONE=0\n', },),).toEqual({
              found: true,
              status: '0',
            },);
          },
        },),
        it({
          name: 'reads a status that is more than one digit',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'DONE=17\n', },),).toEqual({
              found: true,
              status: '17',
            },);
          },
        },),
        it({
          name: 'answers that a log with no done line names no status',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'arm output\n', },),).toEqual({ found: false, },);
          },
        },),
        it({
          name: 'answers that an empty log names no status',
          fn: async () => {
            expect(doneStatusOf({ pulled: '', },),).toEqual({ found: false, },);
          },
        },),
        it({
          name: 'refuses a done line that holds anything but digits',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'DONE=0x1\nDONE=2\n', },),).toEqual({
              found: true,
              status: '2',
            },);
          },
        },),
        it({
          name: 'reads the first status the log names',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'DONE=1\nDONE=2\n', },),).toEqual({
              found: true,
              status: '1',
            },);
          },
        },),
        it({
          name: 'does not read a status from a line that only mentions one',
          fn: async () => {
            expect(doneStatusOf({ pulled: 'the script wrote DONE=0\n', },),).toEqual({ found: false, },);
          },
        },),
      ],
    },),
    //endregion What a pulled log has to prove

    //region The bound and the detached script
    describe({
      name: guestCommandTimeoutSeconds.name,
      children: [
        it({
          name: 'subtracts the margin from the whole seconds of the bound',
          fn: async () => {
            expect(guestCommandTimeoutSeconds({ timeoutMilliseconds: 3_600_000, },),).toBe(3_600 - guestTimeoutMarginSeconds);
          },
        },),
        it({
          name: 'rounds a part second down, so the guest never outwaits the host',
          fn: async () => {
            expect(guestCommandTimeoutSeconds({ timeoutMilliseconds: 120_900, },),).toBe(120 - guestTimeoutMarginSeconds);
          },
        },),
        it({
          name: 'gives a bound no longer than the margin the guest no time to run',
          fn: async () => {
            expect(guestCommandTimeoutSeconds({ timeoutMilliseconds: 60_000, },),).toBe(0);
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
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: true,
            },);
            expect(files.launch.startsWith('test "$(hostname)" = guest-one || exit 1\n',),).toBe(true);
            expect(files.step.includes('hostname',),).toBe(false);
          },
        },),
        it({
          name: 'puts the naming check in the step when the launcher does not carry it',
          fn: async () => {
            const files = detachedScriptFiles({
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: false,
            },);
            expect(files.step.startsWith('test "$(hostname)" = guest-one\n',),).toBe(true);
            expect(files.launch.includes('hostname',),).toBe(false);
          },
        },),
        it({
          name: 'runs the bounded script detached and keeps its whole output in the shared log',
          fn: async () => {
            const files = detachedScriptFiles({
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: true,
            },);
            expect(files.step.includes('\'echo one\' > /mnt/shared/arm.log 2>&1',),).toBe(true);
          },
        },),
        it({
          name: 'ends the step by writing the status the host waits for into that log',
          fn: async () => {
            const files = detachedScriptFiles({
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: true,
            },);
            expect(files.step.endsWith('status=$?\necho "DONE=$status" >> /mnt/shared/arm.log\n',),).toBe(true);
          },
        },),
        it({
          name: 'starts the step under setsid and nohup, so the host returns as soon as the script is detached',
          fn: async () => {
            const files = detachedScriptFiles({
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: true,
            },);
            expect(files.launch.includes('setsid nohup sh /mnt/shared/arm.log.step.sh < /dev/null > /dev/null 2>&1 &',),).toBe(true);
            expect(files.launch.endsWith('exit 0\n',),).toBe(true);
          },
        },),
        it({
          name: 'gives the guest the bound less the margin its own timeout leaves before a kill',
          fn: async () => {
            const files = detachedScriptFiles({
              chosen: chosen(),
              script: 'echo one',
              log: 'arm.log',
              timeout: 900_000,
              hostnameCheckInLaunch: true,
            },);
            const expected = guestCommandTimeoutSeconds({ timeoutMilliseconds: 900_000, },);
            expect(files.step.includes(`timeout ${String(expected,)} sh -ec `,),).toBe(true);
          },
        },),
      ],
    },),
    //endregion The bound and the detached script
  ],
},);
