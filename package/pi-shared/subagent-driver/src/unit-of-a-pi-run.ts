/**
 The systemd unit one run is started as: its name, the ceiling it runs under, and the driver command it runs.

 The shape is the one `.agents/skills/pi-as-subagent` writes by hand, so a run started here reads the same as one a person
 started: the unit is `pi-<name>.service`, the driver writes into the run folder it is given, and `--wait` makes the start end
 when the unit does and answer with its status.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import type { StartPiRunOptions, } from './start-pi-run-options.ts';

/**
 Seconds past the run's wall clock before systemd ends the unit, which bounds the driver itself when a file system call or a
 named pipe keeps it alive past its summary.
 */
const UNIT_END_GRACE_SECONDS = 300;

/**
 How much of the disk's read and write attention the unit may claim. 100 is systemd's default, and the coordinator's starts give
 a unit a tenth of it, so dozens of agents on one btrfs volume share the disk instead of racing for it.
 */
const UNIT_IO_WEIGHT = 10;

/**
 The environment this machine's `mise` puts on `PATH` could not be read, so a unit started without it would find no `pi`.
 */
export class UnitEnvironmentUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'UnitEnvironmentUnreadableError';
}

/**
 What the unit is handed, once the machine's own environment has been read.
 */
export type UnitEnvironment = {
  /**
   The `PATH` a unit needs: the shell's is not inherited, and a start without it failed in seconds with `spawn pi ENOENT`.
   */
  readonly path: string;

  /**
   The interpreter the driver entry runs under.
   */
  readonly node: string;
};

/**
 Asks the machine one short question and insists on an answer.

 @param run - how a command is run

 @param command - the program to ask

 @param words - its arguments

 @returns what it printed, trimmed

 @throws UnitEnvironmentUnreadableError when it exited with anything but zero

 @example
 ```ts
 const node = await answered({ run: runCommandWithChildProcess, command: 'mise', words: ['exec', '--', 'node', '--version',], });
 ```
 */
async function answered({
  run,
  command,
  words,
}: {
  readonly run: RunCommand;
  readonly command: string;
  readonly words: readonly string[];
},): Promise<string> {
  /**
   What the program printed and how it ended.
   */
  const shown = await run({
    command,
    words,
  },);
  if (shown.status !== 0) {
    throw new UnitEnvironmentUnreadableError(
      `${command} ${words.join(' ',)} answered ${String(shown.status,)}: ${shown.problem
        ?? shown.stderr
          .trim()}`,
    );
  }
  return shown.stdout
    .trim();
}

/**
 Reads the environment a unit needs from this machine's `mise` environment.

 @param run - how a command is run

 @returns the `PATH` and the interpreter a unit starts the driver with

 @throws UnitEnvironmentUnreadableError when `mise` could not answer

 @example
 ```ts
 const environment = await unitEnvironmentOf({ run: runCommandWithChildProcess, });
 ```
 */
export async function unitEnvironmentOf({
  run,
}: {
  readonly run: RunCommand;
},): Promise<UnitEnvironment> {
  return {
    path: await answered({
      run,
      command: 'mise',
      words: [
        'exec',
        '--',
        'printenv',
        'PATH',
      ],
    },),
    node: await answered({
      run,
      command: 'mise',
      words: [
        'exec',
        '--',
        'bash',
        '--noprofile',
        '--norc',
        '-c',
        'command -v node',
      ],
    },),
  };
}

/**
 Builds the arguments of the `systemd-run` that starts one run.

 @param options - what the run was asked for

 @param runFolder - the folder the driver writes this attempt into

 @param workingFolder - the folder Pi starts in

 @param wallSeconds - the wall clock the run is given, after its pool multiplied it

 @param turnCap - how many turns the run may start

 @param session - a session file to resume, when the run resumes one

 @param home - the home directory the unit is given

 @param environment - the `PATH` and interpreter read from this machine

 @returns the arguments, each kept whole

 @example
 ```ts
 const words = unitWordsOf({ options, runFolder: '/runs/a-1', workingFolder: '/repo', wallSeconds: 60, turnCap: 8, home: '/home/a', environment, });
 ```
 */
export function unitWordsOf({
  options,
  runFolder,
  workingFolder,
  wallSeconds,
  turnCap,
  session,
  home,
  environment,
}: {
  readonly options: StartPiRunOptions;
  readonly runFolder: string;
  readonly workingFolder: string;
  readonly wallSeconds: number;
  readonly turnCap: number;
  readonly home: string;
  readonly environment: UnitEnvironment;
  readonly session?: string;
},): readonly string[] {
  return [
    '--user',
    '--wait',
    `--unit=pi-${options.name}`,
    '--collect',
    `--property=RuntimeMaxSec=${String(wallSeconds + UNIT_END_GRACE_SECONDS,)}`,
    `--property=MemoryMax=${options.memory}`,
    `--property=CPUQuota=${String(options.cpuPercent,)}%`,
    `--property=TasksMax=${String(options.tasks,)}`,
    `--property=IOWeight=${String(UNIT_IO_WEIGHT,)}`,
    `--setenv=PATH=${environment.path}`,
    `--setenv=HOME=${home}`,
    environment.node,
    options.driverEntry,
    `--run-folder=${runFolder}`,
    `--working-folder=${workingFolder}`,
    `--prompt-file=${options.promptFile}`,
    `--provider=${options.provider}`,
    `--model=${options.model}`,
    `--wall-seconds=${String(wallSeconds,)}`,
    `--idle-seconds=${String(options.idleSeconds,)}`,
    `--turn-cap=${String(turnCap,)}`,
    ...((session === undefined) ? [] : [`--session=${session}`,]),
  ];
}
