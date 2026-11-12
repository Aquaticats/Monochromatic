/**
 The launchers that outlived their units: which processes are launchers, which of them have outlived a unit that ended, the line
 each prints, and the order they are ended in. The command is run as a program too, against stand-in `ps` and `systemctl` programs
 on a `PATH` of the test's own.

 Nothing here signals a process of the machine the test runs on: the stopping tests hand over a function that records what it would
 have signalled, and the program runs never pass `--stop`, because the process numbers a stand-in `ps` prints name no process this
 test started.
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { delimiter, dirname, join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  LAUNCHER_ENTRY_NAMES,
  LAUNCHER_MINIMUM_AGE_SECONDS,
  type LauncherProcess,
  STOP_SIGNAL,
  type ProcessUnderARun,
  childrenOf,
  launcherLineOf,
  launchersIn,
  launchersPastTheirUnits,
  optionWordOf,
  signalFailureLineOf,
  stopLaunchers,
  stopTargetsOf,
  unitStateIsRunning,
  wordsInCommand,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 One process, as `ps` prints it.

 @param pid - the process number

 @param command - its command line

 @param parentPid - the process that started it

 @param elapsedSeconds - how long it has been running

 @returns the process

 @example
 ```ts
 const process = aProcess({ pid: 12, command: 'node /runs/launcher.ts --name=a' });
 ```
 */
function aProcess({
  pid,
  command,
  parentPid = 1,
  elapsedSeconds = LAUNCHER_MINIMUM_AGE_SECONDS * 2,
}: {
  readonly pid: number;
  readonly command: string;
  readonly parentPid?: number;
  readonly elapsedSeconds?: number;
},): ProcessUnderARun {
  return {
    pid,
    parentPid,
    elapsedSeconds,
    state: 'S',
    waitingIn: 'do_wait',
    command,
  };
}

/**
 One launcher, as the process table describes it.

 @param name - the run's name

 @param pid - the launcher's process number

 @returns the launcher

 @example
 ```ts
 const launcher = aLauncher({ name: 'copy-route', pid: 12 });
 ```
 */
function aLauncher({
  name,
  pid,
}: {
  readonly name: string;
  readonly pid: number;
},): LauncherProcess {
  return {
    name,
    pid,
    parentPid: 1,
    elapsedSeconds: LAUNCHER_MINIMUM_AGE_SECONDS * 2,
    command: `node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --name=${name}`,
  };
}

/**
 A folder a test throws away.

 @returns the folder and where its stand-in programs live

 @example
 ```ts
 await using folder = await makeFolderWithPrograms();
 ```
 */
async function makeFolderWithPrograms(): Promise<{
  readonly path: string;
  readonly bins: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-launchers-',),);
  /**
   Where the stand-in programs live.
   */
  const bins = join(path, 'bin',);
  await mkdir(bins,);
  return {
    path,
    bins,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one stand-in program.

 @param folder - the folder the programs live in

 @param name - the program's name

 @param body - what it does, as a bash body

 @example
 ```ts
 await writeProgram({ folder: bins, name: 'ps', body: 'exit 0', });
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
}): Promise<void> {
  /**
   The program's file.
   */
  const file = join(folder, name,);
  await writeFile(file, `#!/usr/bin/env bash\n${body}\n`,);
  await chmod(file, 0o700,);
}

/**
 Runs the command as a program, with the test's own programs ahead of node.

 @param request - the folder whose programs it uses, its words, and its environment

 @returns the status and both streams

 @example
 ```ts
 const ran = await runTheCommand({ folder, words: [], environment: {} });
 ```
 */
async function runTheCommand({
  folder,
  words,
  environment,
}: {
  readonly folder: {
    readonly path: string;
    readonly bins: string;
  };
  readonly words: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
}): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The programs the command may run, then the node that runs it.
   */
  const path = [
    folder.bins,
    dirname(process.execPath,),
    '/usr/bin',
  ].join(delimiter,);
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(process.execPath, [
      new URL('list-launchers-whose-unit-has-ended.ts', import.meta.url,).pathname,
      ...words,
    ], {
      env: {
        PATH: path,
        HOME: folder.path,
        ...environment,
      },
      encoding: 'utf8',
      timeout: 60_000,
    },);
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    /**
     What the failure carries.
     */
    const failure = caught as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : 1,
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: wordsInCommand.name,
      children: [
        it({
          name: 'reads the words of a command line joined by single spaces, and nothing for a blank one',
          fn: async () => {
            expect(wordsInCommand({ command: 'node /runs/a.ts --name=b', },),).toEqual([
              'node',
              '/runs/a.ts',
              '--name=b',
            ],);
            expect(wordsInCommand({ command: '   ', },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: optionWordOf.name,
      children: [
        it({
          name: 'reads the value of one option out of the words, and nothing when the words do not carry it',
          fn: async () => {
            expect(optionWordOf({
              words: ['node', 'launcher.ts', '--name=copy-route',],
              option: '--name=',
            },),).toBe('copy-route',);
            expect(optionWordOf({
              words: ['node', 'launcher.ts',],
              option: '--name=',
            },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: launchersIn.name,
      children: [
        it({
          name: 'finds every process that runs a launcher entry and names a run, and leaves a process that runs another program or names no run out',
          fn: async () => {
            /**
             The processes a machine might report.
             */
            const table = [
              aProcess({
                pid: 12,
                command: `node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --name=copy-route`,
                parentPid: 3,
              },),
              aProcess({
                pid: 13,
                command: 'node /runs/another-command.ts --name=not-a-run',
              },),
              aProcess({
                pid: 14,
                command: `node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --branch=feat/a`,
              },),
            ];
            expect(launchersIn({
              table,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
            },),).toEqual([{
              name: 'copy-route',
              pid: 12,
              parentPid: 3,
              elapsedSeconds: LAUNCHER_MINIMUM_AGE_SECONDS * 2,
              command: `node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --name=copy-route`,
            },],);
          },
        },),
      ],
    },),
    describe({
      name: childrenOf.name,
      children: [
        it({
          name: 'names the processes one launcher started, and no other',
          fn: async () => {
            /**
             A launcher, its waiter and an unrelated process.
             */
            const table = [
              aProcess({
                pid: 20,
                command: 'systemd-run --user --wait',
                parentPid: 12,
              },),
              aProcess({
                pid: 21,
                command: 'node /runs/other.ts',
              },),
            ];
            expect(childrenOf({
              table,
              parentPid: 12,
            },).map(function pidOf(child,) {
              return child.pid;
            },),).toEqual([20,],);
          },
        },),
      ],
    },),
    describe({
      name: launchersPastTheirUnits.name,
      children: [
        it({
          name: 'lists a launcher whose unit has ended, skips one that is too young to have made its unit, one whose unit still runs, and one whose state could not be read',
          fn: async () => {
            /**
             The launchers a machine might report.
             */
            const launchers = [
              aLauncher({
                name: 'ended',
                pid: 12,
              },),
              {
                ...aLauncher({
                  name: 'young',
                  pid: 13,
                },),
                elapsedSeconds: 1,
              },
              aLauncher({
                name: 'running',
                pid: 14,
              },),
              aLauncher({
                name: 'unreadable',
                pid: 15,
              },),
            ];
            /**
             What systemd answered for each unit.
             */
            const states = new Map([
              ['pi-ended.service', 'inactive',],
              ['pi-young.service', 'inactive',],
              ['pi-running.service', 'active',],
              ['pi-unreadable.service', '',],
            ],);
            /**
             The launchers the command may end.
             */
            const past = await launchersPastTheirUnits({
              launchers,
              minimumAgeSeconds: LAUNCHER_MINIMUM_AGE_SECONDS,
              unitStateOf: async function stateOf(unitName,) {
                return states.get(unitName,) ?? '';
              },
              stateIsRunning: function isRunning(state,) {
                return unitStateIsRunning({ state, },);
              },
              childrenOfParent: function noChildren() {
                return [];
              },
            },);
            expect(past.map(function nameOf(entry,) {
              return entry.launcher.name;
            },),).toEqual(['ended',],);
            expect(past[0]?.unitState,).toBe('inactive',);
          },
        },),
      ],
    },),
    describe({
      name: launcherLineOf.name,
      children: [
        it({
          name: 'says the run, the unit state, how long the launcher has been up and what it started, and says none when it started nothing',
          fn: async () => {
            expect(launcherLineOf({
              launcher: aLauncher({
                name: 'copy-route',
                pid: 12,
              },),
              unitState: 'inactive',
              children: [aProcess({
                pid: 20,
                command: '/usr/bin/systemd-run --user --wait',
                parentPid: 12,
              },),],
            },),).toBe(`copy-route: unit inactive, launcher 12 started ${String(Math.round((LAUNCHER_MINIMUM_AGE_SECONDS * 2) / 60,),)} min ago, children 20:systemd-run`,);
            expect(launcherLineOf({
              launcher: aLauncher({
                name: 'copy-route',
                pid: 12,
              },),
              unitState: 'failed',
              children: [],
            },).endsWith('children none',),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: stopTargetsOf.name,
      children: [
        it({
          name: 'names each child before the launcher, so the waiter goes first',
          fn: async () => {
            expect(stopTargetsOf({
              launcher: aLauncher({
                name: 'a',
                pid: 12,
              },),
              unitState: 'inactive',
              children: [
                aProcess({
                  pid: 20,
                  command: 'systemd-run --user --wait',
                  parentPid: 12,
                },),
                aProcess({
                  pid: 21,
                  command: 'node a.ts',
                  parentPid: 20,
                },),
              ],
            },),).toEqual([
              20,
              21,
              12,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: stopLaunchers.name,
      children: [
        it({
          name: 'signals every launcher and its children with the polite signal, keeps going when one cannot be signalled, and says which',
          fn: async () => {
            /**
             What was signalled, in order.
             */
            const signalled: string[] = [];
            /**
             What could not be signalled.
             */
            const failures = stopLaunchers({
              past: [
                {
                  launcher: aLauncher({
                    name: 'a',
                    pid: 12,
                  },),
                  unitState: 'inactive',
                  children: [aProcess({
                    pid: 20,
                    command: 'systemd-run --user --wait',
                    parentPid: 12,
                  },),],
                },
                {
                  launcher: aLauncher({
                    name: 'b',
                    pid: 13,
                  },),
                  unitState: 'inactive',
                  children: [],
                },
              ],
              signal: function signal({ pid, signal: name, },) {
                if (pid === 20) {
                  throw new Error('no such process',);
                }
                signalled.push(`${String(pid,)}:${name}`,);
              },
            },);
            expect(signalled,).toEqual([
              `12:${STOP_SIGNAL}`,
              `13:${STOP_SIGNAL}`,
            ],);
            expect(failures,).toEqual([signalFailureLineOf({
              pid: 20,
              caught: new Error('no such process',),
            },),],);
          },
        },),
      ],
    },),
    describe({
      name: 'list-launchers-whose-unit-has-ended',
      children: [
        it({
          name: 'names the launcher whose unit has ended, with the state, its age and the waiter it started',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'ps',
              body: `printf '%s\\n' ' 12 1 7200 S - node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --name=copy-route' ' 20 12 7100 S - /usr/bin/systemd-run --user --wait --unit=pi-copy-route' ' 21 1 60 S - node /runs/other.ts'`,
            },);
            await writeProgram({
              folder: folder.bins,
              name: 'systemctl',
              body: `printf '%s\\n' 'inactive'`,
            },);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              folder,
              words: [],
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe(`copy-route: unit inactive, launcher 12 started 120 min ago, children 20:systemd-run`,);
          },
        },),
        it({
          name: 'says no launcher outlived its unit when every unit still runs, and refuses an age it cannot read',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'ps',
              body: `printf '%s\\n' ' 12 1 7200 S - node /runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''} --name=copy-route'`,
            },);
            await writeProgram({
              folder: folder.bins,
              name: 'systemctl',
              body: `printf '%s\\n' 'active'`,
            },);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              folder,
              words: [],
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toContain('no launcher outlived its unit',);
            /**
             What it answers for an age that is no number.
             */
            const refused = await runTheCommand({
              folder,
              words: ['--minimum-age-seconds=an-hour',],
              environment: {},
            },);
            expect(refused.status,).not.toBe(0,);
            expect(refused.stderr,).toContain('--minimum-age-seconds must be a number above zero',);
          },
        },),
      ],
    },),
  ],
},);
