/**
 The step that links a run's dependencies, run inside a capped unit of the run's own.

 Why a unit and not the launcher: `pnpm install` in a fresh worktree runs the branch's own lifecycle scripts (`postinstall` and the
 rest), and `--offline` and `--frozen-lockfile` do not switch them off. Run with the launcher's access, such a script writes
 anywhere the launcher may, the main checkout included, before the read-only unit that confines the run exists. Run inside a unit
 with the same read-only checkout, the script writes only where the run may write, and a write anywhere else ends with the
 operating system's own refusal.

 The unit is named `<run name>-dependencies`, never `pi-*`: the slot count reads every `pi-*` unit as a run. It ends by itself, and
 the manager collects it.

 Where this machine has no user manager at all, the install still happens: it runs with the launcher's own access, the answer says
 so, and the run's own folder records it, so a reader of the run knows what protection it lacked and why. No step here depends on
 a manager being there.

 A machine that *has* a manager and did not answer is another case, and it is refused rather than degraded: the socket says the
 manager is there, so an install that could not be confined to the worktree would be a fault silently spent, and the branch's own
 lifecycle scripts would run with the launcher's access. The question is asked twice before that, because a manager that missed one
 answer on a loaded machine is not a missing manager.

 @module
 */

import {
  mkdir,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { readOrdinaryFileWithin, } from './read-ordinary-file.ts';
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

  /**
   The folder the manager's socket lives in, which is this launcher's own runtime directory; a test points it at a fixture.
   */
  readonly managerRuntimeDirectory?: string;

  /**
   The file that names the units this launcher is in, which a test points at a fixture.
   */
  readonly cgroupFile?: string;
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
   Whether a unit confined the install. On a machine with no user manager at all the install runs with the launcher's access and
   this is false, so a caller can say what protection the run lacked.
   */
  readonly confined: boolean;

  /**
   What the run lacked, when no unit confined the install, as the run's own folder records it.
   */
  readonly why?: string;
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
  /**
   What this machine has, which decides whether an install that cannot be confined is degraded or refused.
   */
  const presence = await managerPresenceIn({
    ...((request.managerRuntimeDirectory === undefined)
      ? {}
      : { runtimeDirectory: request.managerRuntimeDirectory, }),
    ...((request.cgroupFile === undefined) ? {} : { cgroupFile: request.cgroupFile, }),
  },);
  if (presence.there) {
    /**
     Whether that manager answered, asked twice before this start gives up on it.
     */
    const manager = await managerAnswerOf({
      run: request.run,
      folder: request.worktree,
    },);
    if (!manager.answered) {
      throw new StartRefusedError(
        `${request.runName} was not started: ${presence.evidence}, so this machine has a user manager, and \`systemctl --user show --property=Version\` did not answer it twice (${manager.answer}). The install was not run, because it could only have run with this launcher's access, and the branch's own lifecycle scripts would have written wherever the launcher may. A manager that answers again can be started then, and a machine that has no user manager at all takes the unconfined path and records it.`,
      );
    }
  }
  if (!presence.there) {
    // No manager on this machine: the install still happens, this step says so, and the run's own folder records it rather than
    // refusing a run this machine can confine no other way.
    /**
     What the unconfined install answered.
     */
    const unconfined = await request.run(install,);
    /**
     What this step has to say about the run's dependencies, which the run's own folder keeps.
     */
    const record = {
      status: unconfined.status,
      confined: false,
      why: `this machine has no user manager (${presence.evidence}), so the install ran with the launcher's access and an install script could write ${request.checkoutFolder}`,
    };
    await writeTheRecord({
      runFolder: request.runFolder,
      record,
    },);
    return {
      status: unconfined.status,
      output: `${unconfined.stdout}${unconfined.stderr}${unconfined.problem ?? ''}`.trim(),
      confined: false,
      why: record.why,
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
  await writeTheRecord({
    runFolder: request.runFolder,
    record: {
      status: answered.status,
      confined: true,
    },
  },);
  return {
    status: answered.status,
    output: `${answered.stdout}${answered.stderr}${answered.problem ?? ''}`.trim(),
    confined: true,
  };
}

/**
 What one run's folder keeps about its dependency linking, so a reader of the run knows what confined the install.

 @param runFolder - the run's own folder

 @param record - what this step answered

 @example
 ```ts
 await writeTheRecord({ runFolder: '/runs/a-1', record: { status: 0, confined: true, }, });
 ```
 */
async function writeTheRecord({
  runFolder,
  record,
}: {
  readonly runFolder: string;
  readonly record: {
    readonly status: number;
    readonly confined: boolean;
    readonly why?: string;
  };
},): Promise<void> {
  await mkdir(
    runFolder,
    { recursive: true, },
  );
  await writeFile(
    join(
      runFolder,
      DEPENDENCIES_RECORD_NAME,
    ),
    `${JSON.stringify(
      record,
      undefined,
      2,
    )}\n`,
    { encoding: 'utf8', },
  );
}

/**
 What the run's own folder calls what this step answered about its dependencies.
 */
export const DEPENDENCIES_RECORD_NAME: string = 'dependencies.json';

/**
 What this machine has, which decides whether an install that cannot be confined is degraded or refused.
 */
export type ManagerPresence = {
  /**
   Whether a user manager is on this machine at all.
   */
  readonly there: boolean;

  /**
   What says so, in the words a refusal or a note prints: the socket that was found, the unit this launcher runs in, or that neither
   was there.
   */
  readonly evidence: string;
};

/**
 Reads whether this machine has a user manager at all: its own socket, or the unit this launcher itself runs in.

 Why this and not one answer to one question: `systemctl` failing once says nothing about whether the machine has a manager, and a
 timeout on a loaded machine says less. The socket and the cgroup are what the machine has, and they are read from the filesystem,
 so a wrong answer to a question cannot turn a manager that is there into one that is not.

 @param runtimeDirectory - the folder the manager's socket lives in, which a test points at a fixture

 @param cgroupFile - the file that names the units this process is in, which a test points at a fixture

 @returns whether the machine has a manager, and what says so

 @example
 ```ts
 const presence = await managerPresenceIn({ runtimeDirectory: '/run/user/1000', });
 ```
 */
export async function managerPresenceIn({
  runtimeDirectory = process.env
    .XDG_RUNTIME_DIR,
  cgroupFile = CGROUP_FILE,
}: {
  /**
   The folder the manager's socket lives in, which is this launcher's own runtime directory when it is not named.
   */
  readonly runtimeDirectory?: string;

  /**
   The file that names the units this process is in, which a test points at a fixture.
   */
  readonly cgroupFile?: string;
},): Promise<ManagerPresence> {
  if (runtimeDirectory !== undefined) {
    /**
     The manager's own socket, which every connection to a user manager goes through.
     */
    const socket = join(
      runtimeDirectory,
      MANAGER_SOCKET,
    );
    try {
      if ((await stat(socket,)).isSocket()) {
        return {
          there: true,
          evidence: `the manager's socket is ${socket}`,
        };
      }
    }
    catch (error) {
      // A socket that is not there is one of the two things a machine without a manager looks like, so the cgroup decides next.
      if (errorCodeOf(error,) !== 'ENOENT') {
        throw error;
      }
    }
  }
  /**
   What the units this process is in say, which a manager writes and nothing else does.
   */
  const cgroup = await cgroupTextOf({ cgroupFile, },);
  if (namesAManagerUnit({ cgroup, },)) {
    return {
      there: true,
      evidence: `this launcher runs in a unit of a manager (${cgroupFile})`,
    };
  }
  return {
    there: false,
    evidence: `neither ${String(runtimeDirectory,)}/systemd/private nor a unit of a manager in ${cgroupFile}`,
  };
}

/**
 Reads what this process's own cgroup file says, and nothing when it cannot be read.

 @param cgroupFile - the file that names the units this process is in

 @returns its text, or nothing when it is not there or cannot be read as an ordinary file

 @example
 ```ts
 const cgroup = await cgroupTextOf({ cgroupFile: '/proc/self/cgroup', });
 ```
 */
async function cgroupTextOf({ cgroupFile }: { readonly cgroupFile: string },): Promise<string> {
  try {
    return await readOrdinaryFileWithin({ path: cgroupFile, },);
  }
  catch (error) {
    // A cgroup file that cannot be read is no answer about this machine, and the manager's socket then decides alone.
    if (errorCodeOf(error,) === 'ENOENT') {
      return '';
    }
    throw error;
  }
}

/**
 Says whether a cgroup file names a unit of this user's own manager, which is `user@1000.service`: a session's own
 `session-1.scope` is not one of them, so a machine whose session started this process has no manager.

 @param cgroup - what the cgroup file says

 @returns whether it names a manager's own unit

 @example
 ```ts
 const named = namesAManagerUnit({ cgroup: '0::/user.slice/user-1000.slice/user@1000.service/app.slice/a.service', });
 ```
 */
function namesAManagerUnit({ cgroup }: { readonly cgroup: string },): boolean {
  return cgroup.includes('/user@',) && cgroup.includes('.service',);
}

/**
 Where the manager's socket lives under the runtime directory of the user.
 */
const MANAGER_SOCKET: string = 'systemd/private';

/**
 Where this process's own units are named.
 */
const CGROUP_FILE: string = '/proc/self/cgroup';

/**
 What asking the manager whether it is there answered.
 */
type ManagerAnswer = {
  /**
   Whether it answered at all.
   */
  readonly answered: boolean;

  /**
   What it answered, or what it said instead, in the words a refusal prints.
   */
  readonly answer: string;
};

/**
 Asks this machine's user manager whether it is there, and asks once more when the first answer does not come.

 Why twice: the question decides whether an install can be confined, and a manager that missed one answer on a loaded machine is not a
 missing manager. Two answers that both fail are a manager that is there and is not answering, which the caller refuses rather than
 degrading.

 @param run - how one command is run

 @param folder - the folder the question is asked in, which exists whether a manager does or not

 @returns whether it answered, and what it answered or said

 @example
 ```ts
 const answers = await managerAnswerOf({ run: runCommandWithChildProcess, folder: '/repo', });
 ```
 */
async function managerAnswerOf({
  run,
  folder,
}: {
  readonly run: RunCommand;
  readonly folder: string;
},): Promise<ManagerAnswer> {
  /**
   What the manager said the first time.
   */
  const first = await oneManagerAnswer({
    run,
    folder,
  },);
  if (first.answered) {
    return first;
  }
  /**
   What it said when it was asked again, which a manager that missed one answer on a loaded machine is worth.
   */
  const second = await oneManagerAnswer({
    run,
    folder,
  },);
  if (second.answered) {
    return second;
  }
  return {
    answered: false,
    answer: `the first said ${first.answer}; asked again, it said ${second.answer}`,
  };
}

/**
 Asks the manager whether it is there, once.

 @param run - how one command is run

 @param folder - the folder the question is asked in

 @returns whether it answered, and what it answered or said

 @example
 ```ts
 const answer = await oneManagerAnswer({ run: runCommandWithChildProcess, folder: '/repo', });
 ```
 */
async function oneManagerAnswer({
  run,
  folder,
}: {
  readonly run: RunCommand;
  readonly folder: string;
},): Promise<ManagerAnswer> {
  /**
   What the manager said about itself this time.
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
  /**
   What this answer amounts to, which is the answer itself or the status and words that came instead.
   */
  const thisAnswer = answered.stdout
    .trim();
  if ((answered.status === 0) && (thisAnswer !== '')) {
    return {
      answered: true,
      answer: thisAnswer,
    };
  }
  return {
    answered: false,
    answer: `status ${String(answered.status,)} ${JSON.stringify(`${answered.stdout}${answered.stderr}${answered.problem ?? ''}`.trim(),)}`,
  };
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
      : {
        note: `${linked.why ?? `linked with this launcher's access, so an install script could write ${request.checkoutFolder}`}; the run's own folder records this as ${DEPENDENCIES_RECORD_NAME}`,
      }),
  };
}
