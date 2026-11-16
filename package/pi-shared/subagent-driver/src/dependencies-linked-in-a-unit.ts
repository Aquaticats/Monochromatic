/**
 The step that links a run's dependencies, run inside a capped unit of the run's own.

 Why a unit and not the launcher: `pnpm install` in a fresh worktree runs the branch's own lifecycle scripts (`postinstall` and the
 rest), and `--offline` and `--frozen-lockfile` do not switch them off. Run with the launcher's access, such a script writes
 anywhere the launcher may, the main checkout included, before the read-only unit that confines the run exists. Run inside a unit
 with the same read-only checkout, the script writes only where the run may write, and a write anywhere else ends with the
 operating system's own refusal.

 The unit is named `<run name>-dependencies`, never `pi-*`: the slot count reads every `pi-*` unit as a run. It ends by itself, and
 the manager collects it.

 Where no service manager answers, the install still happens: it runs with the launcher's own access, and the answer says so, so
 the launcher tells its reader what protection it lacked. No step here depends on a manager being there.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import type { RunCommand, } from './run-command.ts';
import {
  type FileSystemOfARun,
  fileSystemPropertiesOf,
} from './file-system-of-a-run.ts';
import { StartRefusedError, } from './checks-before-a-start.ts';
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
 How long the client that waits for the dependency unit may take, in seconds: the unit's own bound plus its grace, plus a minute
 for the manager to collect it and the client to read the status.

 Why it must be this and not a default: a wait's bound is the one its caller chose, and the client must outlive the unit it waits
 for. The default a command gets is half a minute, shorter than any install worth waiting for; a client killed before its unit
 ends leaves the install running while the run starts.
 */
const DEPENDENCIES_CLIENT_SLACK_SECONDS: number = 60;

/**
 How much longer than its own unit the client that waits for it may take, in seconds: the unit's own bound plus its grace, plus
 room for the manager to collect the unit and for the client to read its status after the unit has ended.
 */
export const DEPENDENCIES_CLIENT_SECONDS: number = DEPENDENCIES_UNIT_SECONDS + UNIT_GRACE_SECONDS
  + DEPENDENCIES_CLIENT_SLACK_SECONDS;

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

  /**
   Whether a unit confined the install. When this machine's manager does not answer, the install runs with the launcher's access
   and this is false, so a caller can say what protection the run lacked.
   */
  readonly confined: boolean;
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
   The install, as this machine's mise runs it in the worktree, which is what the unit runs too.
   */
  const install = {
    command: mise,
    words: [
      'exec',
      '--',
      'pnpm',
      'install',
      '--offline',
      '--frozen-lockfile',
    ],
    folder: request.worktree,
  };
  if (!(await managerAnswers({
    run: request.run,
    folder: request.worktree,
  },))) {
    // No manager: the install still happens, and this is the step that says so rather than refusing the run.
    /**
     What the unconfined install answered.
     */
    const unconfined = await request.run(install,);
    return {
      status: unconfined.status,
      output: `${unconfined.stdout}${unconfined.stderr}${unconfined.problem ?? ''}`.trim(),
      confined: false,
    };
  }
  /**
   The command that starts the unit.
   */
  const command = dependenciesUnitCommandOf({
    ...request,
    mise,
  },);
  /**
   What the unit answered, waited for as long as the unit itself may run.
   */
  const answered = await request.run({
    command: command.command,
    words: command.words,
    folder: request.worktree,
    timeoutMilliseconds: DEPENDENCIES_CLIENT_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  return {
    status: answered.status,
    output: `${answered.stdout}${answered.stderr}${answered.problem ?? ''}`.trim(),
    confined: true,
  };
}

/**
 Says whether this machine's user manager answers, which decides whether the install can be confined.

 @param run - how one command is run

 @param folder - the folder the question is asked in, which exists whether a manager does or not

 @returns whether a manager answered with its version

 @example
 ```ts
 const answers = await managerAnswers({ run: runCommandWithChildProcess, folder: '/repo', });
 ```
 */
async function managerAnswers({
  run,
  folder,
}: {
  readonly run: RunCommand;
  readonly folder: string;
},): Promise<boolean> {
  /**
   What the manager said about itself.
   */
  const answered = await run({
    command: 'systemctl',
    words: [
      '--user',
      'show',
      '--property=Version',
      '--value',
    ],
    folder,
    timeoutMilliseconds: MANAGER_ANSWER_MILLISECONDS,
  },);
  return (answered.status === 0) && (answered.stdout
    .trim()
    !== '');
}

/**
 How long this machine's manager has to answer whether it is there, in milliseconds.

 A bound rather than a measurement: a manager on this machine answers in milliseconds, and one that has not answered in ten
 seconds is not answering.
 */
const MANAGER_ANSWER_MILLISECONDS: number = 10 * MILLISECONDS_PER_SECOND;

/**
 What linking one run's dependencies answered for the launcher to print.

 @example
 ```ts
 const outcome = await linkDependenciesOrRefuse(request,);
 ```
 */
export type DependenciesOutcome = {
  /**
   The status the install ended with, which is zero whenever this answers at all.
   */
  readonly status: number;

  /**
   What the run lacked where no manager answered, which the launcher prints after the status, and nothing when a unit confined
   the install.
   */
  readonly note?: string;
};

/**
 Links a run's dependencies, says what protection the run lacks where no manager answered, and refuses the start when they were
 not linked.

 A failed install is acted on, never printed and passed over: a run whose dependencies were not linked would build, test and
 commit against a half-installed tree, and spend a provider slot doing it.

 @param request - the run, its folders, the caps and the environment

 @returns the status the install ended with, and the notes the launcher prints

 @throws StartRefusedError when the install failed, naming its status and what it printed

 @example
 ```ts
 const outcome = await linkDependenciesOrRefuse({ runName: 'a', checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', fileSystem, cpuPercent: 100, memory: '1G', taskCount: 100, pathOfMiseEnvironment: '/bin', home: '/home/a', run: runCommandWithChildProcess, });
 ```
 */
export async function linkDependenciesOrRefuse(request: DependenciesUnitRequest,): Promise<DependenciesOutcome> {
  /**
   What linking the dependencies answered.
   */
  const linked = await linkDependenciesInAUnit(request,);
  if (linked.status !== 0) {
    throw new StartRefusedError(
      `${request.runName} was not started: its dependencies were not linked (status ${String(linked.status,)}), so the worktree ${request.worktree} holds a half-installed tree and is kept: ${linked.output}`,
    );
  }
  return {
    status: linked.status,
    ...(linked.confined
      ? {}
      : { note: `linked with this launcher's access, because no user manager answered, so an install script could write ${request.checkoutFolder}`, }),
  };
}
