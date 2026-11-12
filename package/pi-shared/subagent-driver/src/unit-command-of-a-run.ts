/**
 The unit one run is started as: the driver's own options, the caps the unit holds, and the environment that gives Pi its `PATH`.

 The unit is what bounds the driver itself. `--wall-seconds` bounds the run, and the unit's own `RuntimeMaxSec` bounds the driver
 with it, so a file system call or a named pipe that cannot be cancelled is ended by systemd rather than left hanging.

 @module
 */

import {
  MILLISECONDS_PER_SECOND,
  TEARDOWN_MILLISECONDS,
} from './driver-constants.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';

/**
 How long the unit may outlive the run's own wall clock, in seconds: the driver's own teardown after the wall clock, which
 `TEARDOWN_MILLISECONDS` bounds at 35 seconds, plus the same 35 seconds again for a loaded host.

 This is what ends a driver that cannot get past a file system call or a named pipe after it has written its summary, so it is
 the one bound nothing inside the driver reaches. The scratch launcher wrote 300 seconds here without saying where it came from;
 every stage the driver itself waits for is bounded, so its teardown cannot run past what it was given.
 */
export const UNIT_GRACE_SECONDS: number = 2 * Math.ceil(TEARDOWN_MILLISECONDS / MILLISECONDS_PER_SECOND);

/**
 The weight the unit's input and output get against other units', which is what the scratch launcher gave every run it started.
 */
export const UNIT_IO_WEIGHT = 10;

/**
 What starting one run's unit needs.
 */
export type UnitCommandRequest = {
  /**
   The run's name, which names the unit.
   */
  readonly runName: string;

  /**
   The processor share the unit gets, in percent.
   */
  readonly cpuPercent: number;

  /**
   The memory the unit may hold, as systemd writes a size.
   */
  readonly memory: string;

  /**
   The number of tasks the unit may hold.
   */
  readonly taskCount: number;

  /**
   How long the run may last, in seconds.
   */
  readonly wallSeconds: number;

  /**
   The `PATH` Pi's environment holds, which a unit does not inherit from the shell that started it.
   */
  readonly pathOfMiseEnvironment: string;

  /**
   The home folder Pi's environment names.
   */
  readonly home: string;

  /**
   The node that runs the driver.
   */
  readonly node: string;

  /**
   The driver's entry.
   */
  readonly driverEntry: string;

  /**
   Where the driver writes this run.
   */
  readonly runFolder: string;

  /**
   Where Pi works.
   */
  readonly workingFolder: string;

  /**
   The first prompt.
   */
  readonly promptFile: string;

  /**
   The provider the run asks for.
   */
  readonly provider: string;

  /**
   The model the run asks for.
   */
  readonly model: string;

  /**
   Seconds the driver stays up for a steering message after Pi comes to rest.
   */
  readonly idleSeconds: number;

  /**
   How many model turns the run may start.
   */
  readonly turnCap: number;

  /**
   The session file the run resumes, when it resumes one.
   */
  readonly session?: string;
};

/**
 Says how long a run's unit may last: its own wall clock, plus what the driver needs to put the run away after it.

 @param wallSeconds - how long the run may last, in seconds

 @returns how long the unit may last, in seconds

 @example
 ```ts
 const runtimeMax = runtimeMaxSecondsOf({ wallSeconds: 7_200, },);
 ```
 */
export function runtimeMaxSecondsOf({ wallSeconds, }: { readonly wallSeconds: number },): number {
  return wallSeconds + UNIT_GRACE_SECONDS;
}

/**
 Builds the command that starts one run's unit.

 @param request - what the unit holds and runs

 @returns the program and its arguments, each kept whole

 @example
 ```ts
 const command = unitCommandOf({ runName: 'copy-route', cpuPercent: 800, memory: '12G', taskCount: 3_000, wallSeconds: 7_200, pathOfMiseEnvironment: '/bin', home: '/home/a', node: '/bin/node', driverEntry: '/pkg/src/drive-pi-subagent.ts', runFolder: '/runs/copy-route-4', workingFolder: '/repo/.claude/worktrees/pi-copy-route', promptFile: '/runs/copy-route-prompt.md', provider: 'neuralwatt', model: 'deepseek-v4.1-flash', idleSeconds: 120, turnCap: 400, },);
 ```
 */
export function unitCommandOf(request: UnitCommandRequest,): {
  readonly command: string;
  readonly words: readonly string[];
} {
  return {
    command: 'systemd-run',
    words: [
      '--user',
      '--wait',
      `--unit=${unitNameOfGiven({ name: request.runName, },)}`,
      '--collect',
      `--property=RuntimeMaxSec=${String(runtimeMaxSecondsOf({ wallSeconds: request.wallSeconds, },),)}`,
      `--property=MemoryMax=${request.memory}`,
      `--property=CPUQuota=${String(request.cpuPercent,)}%`,
      `--property=TasksMax=${String(request.taskCount,)}`,
      `--property=IOWeight=${String(UNIT_IO_WEIGHT,)}`,
      `--setenv=PATH=${request.pathOfMiseEnvironment}`,
      `--setenv=HOME=${request.home}`,
      request.node,
      request.driverEntry,
      `--run-folder=${request.runFolder}`,
      `--working-folder=${request.workingFolder}`,
      `--prompt-file=${request.promptFile}`,
      `--provider=${request.provider}`,
      `--model=${request.model}`,
      `--wall-seconds=${String(request.wallSeconds,)}`,
      `--idle-seconds=${String(request.idleSeconds,)}`,
      `--turn-cap=${String(request.turnCap,)}`,
      ...((request.session === undefined) ? [] : [`--session=${request.session}`,]),
    ],
  };
}
