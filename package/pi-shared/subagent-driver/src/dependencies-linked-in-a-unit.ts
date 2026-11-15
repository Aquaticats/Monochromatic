/**
 The step that links a run's dependencies, run inside a capped unit of the run's own.

 Why a unit and not the launcher: `pnpm install` in a fresh worktree runs the branch's own lifecycle scripts (`postinstall` and the
 rest), and `--offline` and `--frozen-lockfile` do not switch them off. Run with the launcher's access, such a script writes
 anywhere the launcher may, the main checkout included, before the read-only unit that confines the run exists. Run inside a unit
 with the same read-only checkout, the script writes only where the run may write, and a write anywhere else ends with the
 operating system's own refusal.

 The unit is named `<run name>-dependencies`, never `pi-*`: the slot count reads every `pi-*` unit as a run. It ends by itself, and
 the manager collects it.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import {
  type FileSystemOfARun,
  fileSystemPropertiesOf,
} from './file-system-of-a-run.ts';
import { requiredOutputOf, } from './required-output-of-a-command.ts';
import {
  UNIT_IO_WEIGHT,
  UNIT_GRACE_SECONDS,
} from './unit-command-of-a-run.ts';

/**
 How long linking one worktree's dependencies may take, in seconds.

 A bound rather than a measurement: the step is `pnpm install --offline --frozen-lockfile`, which answers from this machine's
 store, and a bounded run of the fixtures here answered in about a second. Fifteen minutes matches the bound a `git worktree add`
 gets, which copies the same ignored state.
 */
export const DEPENDENCIES_UNIT_SECONDS: number = 900;

/**
 What starting one run's dependency linking needs.
 */
export type DependenciesUnitRequest = {
  /**
   The run's name, which the unit's name carries.
   */
  readonly runName: string;

  /**
   The checkout the run's worktree belongs to, which the unit cannot write.
   */
  readonly checkoutFolder: string;

  /**
   The worktree the dependencies are linked in, which is the unit's working folder.
   */
  readonly worktree: string;

  /**
   The run's own folder.
   */
  readonly runFolder: string;

  /**
   What the run's unit may and may not write, which this unit gets too.
   */
  readonly fileSystem: FileSystemOfARun;

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
   The `PATH` the session's own processes get.
   */
  readonly pathOfMiseEnvironment: string;

  /**
   The home folder the unit runs with.
   */
  readonly home: string;

  /**
   How one command is run.
   */
  readonly run: RunCommand;
};

/**
 Reads the name of the unit one run's dependencies are linked in.

 @param runName - the run's name

 @returns the unit's name, without its `.service` suffix

 @example
 ```ts
 const unit = dependenciesUnitNameOf({ runName: 'copy-route', });
 ```
 */
export function dependenciesUnitNameOf({ runName, }: { readonly runName: string },): string {
  return `${runName}-dependencies`;
}

/**
 Builds the command that links one run's dependencies inside a capped unit.

 @param request - the run, its folders, the caps, the environment, and the mise the unit runs

 @returns the program and its arguments, each kept whole

 @example
 ```ts
 const command = dependenciesUnitCommandOf({ runName: 'a', checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', fileSystem, cpuPercent: 100, memory: '1G', taskCount: 100, pathOfMiseEnvironment: '/bin', home: '/home/a', mise: '/bin/mise', run: runCommandWithChildProcess, });
 ```
 */
export function dependenciesUnitCommandOf(request: DependenciesUnitRequest & {
  /**
   The mise the unit runs, by absolute path, because a unit does not inherit the launcher's search path.
   */
  readonly mise: string;
},): {
  readonly command: string;
  readonly words: readonly string[];
} {
  return {
    command: 'systemd-run',
    words: [
      '--user',
      '--wait',
      '--collect',
      '--quiet',
      '--pipe',
      `--unit=${dependenciesUnitNameOf({ runName: request.runName, },)}`,
      `--property=RuntimeMaxSec=${String(DEPENDENCIES_UNIT_SECONDS + UNIT_GRACE_SECONDS,)}`,
      `--property=MemoryMax=${request.memory}`,
      `--property=CPUQuota=${String(request.cpuPercent,)}%`,
      `--property=TasksMax=${String(request.taskCount,)}`,
      `--property=IOWeight=${String(UNIT_IO_WEIGHT,)}`,
      `--working-directory=${request.worktree}`,
      ...fileSystemPropertiesOf(request.fileSystem,),
      `--setenv=PATH=${request.pathOfMiseEnvironment}`,
      `--setenv=HOME=${request.home}`,
      request.mise,
      'exec',
      '--',
      'pnpm',
      'install',
      '--offline',
      '--frozen-lockfile',
    ],
  };
}

/**
 Links one run's dependencies inside a capped unit, which confines whatever the branch's lifecycle scripts do.

 @param request - the run, its folders, the caps and the environment

 @returns the status the install ended with, and what it printed

 @throws CommandAnswerMissingError when the mise the unit would run cannot be found

 @example
 ```ts
 const linked = await linkDependenciesInAUnit({ runName: 'a', checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', fileSystem, cpuPercent: 100, memory: '1G', taskCount: 100, pathOfMiseEnvironment: '/bin', home: '/home/a', run: runCommandWithChildProcess, });
 ```
 */
export async function linkDependenciesInAUnit(request: DependenciesUnitRequest,): Promise<{
  readonly status: number;
  readonly output: string;
}> {
  /**
   The mise the unit runs, found in the launcher's own search path because a unit does not inherit it.
   */
  const mise = await requiredOutputOf({
    run: request.run,
    command: 'bash',
    words: [
      '--noprofile',
      '--norc',
      '-c',
      'command -v mise',
    ],
    folder: request.checkoutFolder,
  },);
  /**
   The command that starts the unit.
   */
  const command = dependenciesUnitCommandOf({
    ...request,
    mise,
  },);
  /**
   What the unit answered.
   */
  const answered = await request.run({
    command: command.command,
    words: command.words,
    folder: request.worktree,
  },);
  return {
    status: answered.status,
    output: `${answered.stdout}${answered.stderr}${answered.problem ?? ''}`.trim(),
  };
}
