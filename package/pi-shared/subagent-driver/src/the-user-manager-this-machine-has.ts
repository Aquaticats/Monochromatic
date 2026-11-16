/**
 What this machine's user manager is, as far as a start can tell: whether it answers the question the start's own unit depends on,
 and which folder its socket lives in.

 Why one question decides: a run's driver only ever runs as a unit of the user manager, so a machine whose manager does not answer
 cannot start a run at all, and a start that made a worktree and ran the branch's install scripts before finding that out would have
 spent the launcher's access on work that cannot be used. The same question is what tells a confined install from an unconfined one,
 and asking it this way means the answer is about the same client the unit start uses, not about a folder that happens to be there.

 A manager that misses one answer is not a missing manager, so the question is asked twice.

 @module
 */

import {
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { RunCommand, } from './run-command.ts';

/**
 How long the manager has to answer whether it is there, in milliseconds.

 A bound rather than a measurement: a manager on this machine answers in milliseconds, and one that has not answered in ten seconds
 is not answering.
 */
export const MANAGER_ANSWER_MILLISECONDS: number = 10 * MILLISECONDS_PER_SECOND;

/**
 How many times the manager is asked whether it is there.

 Two, not one: the question decides whether a run can start at all, and a manager that missed one answer on a loaded machine is not a
 missing manager.
 */
/**
 Where the manager keeps its own socket under the runtime directory of the user.
 */
const MANAGER_SOCKET: string = 'systemd/private';

/**
 What asking the manager whether it is there answered.
 */
export type ManagerAnswer = {
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
   What this answer amounts to: the version it named, or the status and words that came instead.
   */
  const version = answered.stdout
    .trim();
  if ((answered.status === 0) && (version !== '')) {
    return {
      answered: true,
      answer: version,
    };
  }
  return {
    answered: false,
    answer: `status ${String(answered.status,)} ${JSON.stringify(`${answered.stdout}${answered.stderr}${answered.problem ?? ''}`.trim(),)}`,
  };
}

/**
 Asks this machine's user manager whether it is there, twice.

 @param run - how one command is run

 @param folder - the folder the question is asked in, which exists whether a manager does or not

 @returns whether it answered, and what it answered or said

 @example
 ```ts
 const answers = await managerAnswerOf({ run: runCommandWithChildProcess, folder: '/repo', });
 ```
 */
export async function managerAnswerOf({
  run,
  folder,
}: {
  /**
   How one command is run.
   */
  readonly run: RunCommand;

  /**
   The folder the question is asked in.
   */
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
    answer: `asked 1: ${first.answer}; asked 2: ${second.answer}`,
  };
}

/**
 Says what a start has to do about the manager, in the words a refusal prints, or nothing when the manager answered.

 @param run - how one command is run

 @param folder - the folder the question is asked in

 @returns the words of the refusal, or nothing when the manager answered

 @example
 ```ts
 const problem = await managerProblemIn({ run: runCommandWithChildProcess, folder: '/repo', });
 ```
 */
export async function managerProblemIn({
  run,
  folder,
}: {
  /**
   How one command is run.
   */
  readonly run: RunCommand;

  /**
   The folder the question is asked in.
   */
  readonly folder: string;
},): Promise<{ readonly problem?: string }> {
  /**
   What the manager answered.
   */
  const manager = await managerAnswerOf({
    run,
    folder,
  },);
  if (manager.answered) {
    return {};
  }
  return { problem: `no user manager answered \`systemctl --user show --property=Version\` (${manager.answer}), and a run's driver only ever runs as a unit of this machine's user manager, so no run can be started here. Nothing was made and no run happened: install a user manager, or start the run on a machine that has one, and start it again.` };
}

/**
 The runtime directory the manager's own socket lives in, or nothing when this launcher cannot see one.

 Why a start passes this at all: every brief on this machine tells a run to put its commands into a `systemd-run --user --scope` of
 their own, and a scope reaches the manager through the socket under this folder. A launcher reaches the manager through its own
 environment, and a launcher whose `XDG_RUNTIME_DIR` is not the manager's (a disposable one, an ssh session, a login session's
 scope) would hand the run a folder that has no socket in it, and every scope the run starts would end with the bus client's refusal.

 What the answer is used for: only a folder whose `systemd/private` is a socket is named in the unit; anything else is left out, so
 the unit inherits the manager's own environment, which is what a start without this question used to give it.

 @param runtimeDirectory - the runtime directory the launcher itself was given

 @returns that folder when the manager's socket is in it, and nothing otherwise

 @example
 ```ts
 const folder = await runtimeDirectoryTheManagerKeepsItsSocketIn({ runtimeDirectory: '/run/user/1000', });
 ```
 */
export async function runtimeDirectoryTheManagerKeepsItsSocketIn({
  runtimeDirectory,
}: {
  /**
   The runtime directory the launcher itself was given, which may name no manager's socket at all.
   */
  readonly runtimeDirectory?: string;
},): Promise<{ readonly runtimeDirectory?: string }> {
  if (runtimeDirectory === undefined) {
    return {};
  }
  /**
   The manager's own socket, which every connection to a user manager goes through.
   */
  const socket = join(
    runtimeDirectory,
    MANAGER_SOCKET,
  );
  try {
    return (await stat(socket,)).isSocket() ? { runtimeDirectory, } : {};
  }
  catch (error) {
    // A socket that is not there is the answer this asks about: a unit gets no runtime directory, and the manager's own environment
    // decides. Any other failure would hide what the launcher meant to check.
    if (errorCodeOf(error,) === 'ENOENT') {
      return {};
    }
    throw error;
  }
}
