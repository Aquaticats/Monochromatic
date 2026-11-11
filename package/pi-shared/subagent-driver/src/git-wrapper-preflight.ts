/**
 The preflight a launcher runs before it starts a session: which `git` the session's own `PATH` resolves to, and whether that
 program is this repository's Git policy wrapper.

 A session whose `git` resolves to the system binary never runs the wrapper, so it loses every commit policy, the guards against
 discarding work, and the automatic push after each commit, and no wrapper can notice its own absence: `git --version` prints the
 same text through both programs, so it is no identity signal. `git monochromatic-git-policy-cli --help` prints the policy
 namespace help only when the wrapper answers it, measured 2026-11-11 in this checkout at 72 milliseconds, exit 0, its first line
 `Usage: git monochromatic-git-policy-cli <command> [options]`.

 Issue 1768 and `doc/troubleshooting/git-policy-wrapper-launcher-path.md` record why the check belongs to the launcher that builds
 the child environment, what it must report, and the owner's rule that nothing waits forever.

 @module
 */

import {
  stat,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { GIT_ENVIRONMENT, } from './git-command.ts';
import { programOnAPath, } from './program-on-a-path.ts';
import type {
  CommandOutput,
  RunCommand,
} from './run-command.ts';

//region Constants

/**
 The command word the wrapper answers with the policy namespace help.
 */
export const WRAPPER_COMMAND = 'monochromatic-git-policy-cli';

/**
 The line the wrapper's help starts with, which is the whole identity signal this preflight reads.

 Measured on 2026-11-11 in this checkout. The command word alone is not enough: a repository's own git may alias that word, and
 the system git's help for an alias prints something else. A release that rewords this line refuses every launch, which is loud and
 one constant to update, while a weaker signal would start sessions without the wrapper in silence.
 */
const WRAPPER_HELP_LINE = `Usage: git ${WRAPPER_COMMAND} <command> [options]`;

/**
 Milliseconds the wrapper may take to print that help before the session is refused.

 Measured at 72 milliseconds on 2026-11-11 in this checkout, so a program still silent after ten seconds is one that will not
 answer, and the owner's rule is that nothing waits forever. It is a bound rather than a measurement: a loaded host, a cold page
 cache and a wrapper that has to start its own helper all fit under it.
 */
export const GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS = 10_000;

/**
 What the wrapper is asked, as one question: the policy namespace, and the flag that makes it print its help instead of running a
 policy.
 */
const WRAPPER_QUESTION: readonly string[] = [
  WRAPPER_COMMAND,
  '--help',
];

/**
 Variables the question is asked under. A session starts unattended, so the help must reach this process rather than wait on a
 terminal or hand itself to a pager.
 */
const PREFLIGHT_ENVIRONMENT: Readonly<Record<string, string>> = {
  ...GIT_ENVIRONMENT,
  GIT_PAGER: 'cat',
  PAGER: 'cat',
};

/**
 What a reader does about a refusal, named once because every refusal ends with it: the folder holding the wrapper's `git` link in
 a checkout, the task that makes that link, and where it has to sit.
 */
const WRAPPER_FIX = 'The wrapper is the `git` link in `.cache/monochromatic-git-policy-cli-bin` under the checkout, which '
  + '`mise run toolchain:install` makes; put that folder before the system one on the PATH the session gets.';

/**
 What a session loses when its `git` is not the wrapper, in one sentence every refusal carries.
 */
const WHAT_A_SESSION_LOSES = 'a session started with that git commits past every repository policy, past the guards against '
  + 'discarding work, and without the automatic push after each commit';

//endregion Constants

//region Refusals

/**
 A session was not started because the `git` it would run is not this repository's Git policy wrapper.
 */
export class GitWrapperPreflightError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'GitWrapperPreflightError';
}

/**
 Says what a refused question answered, as far as a reader needs it: the status, the bound it was given, and whatever the program
 printed of its own complaint.

 @param answer - what the question answered

 @param timeoutMilliseconds - how long it was allowed to take

 @returns what the refusal says the program did

 @example
 ```ts
 const shown = answerOf({ answer: { status: 1, stdout: '', stderr: 'not a git command', }, timeoutMilliseconds: 10_000, },);
 ```
 */
function answerOf({
  answer,
  timeoutMilliseconds,
}: {
  readonly answer: CommandOutput;
  readonly timeoutMilliseconds: number;
},): string {
  /**
   What the program printed of its own complaint, which a status alone does not say: its failure when it never reached an exit,
   its standard error when it wrote one, and otherwise what it printed.
   */
  const printed = (answer.problem
    ?? answer.stderr
      .trim())
    || answer.stdout
      .trim();
  /**
   The program's own words, when it printed any.
   */
  const detail = (printed === '') ? '' : `: ${printed}`;
  // A command that never reached an exit was killed at its bound or never started, and both are answered by the bound it was
  // given rather than by a status nobody saw.
  if (answer.status === (-1)) {
    return `did not answer within ${String(timeoutMilliseconds,)} milliseconds${detail}`;
  }
  return `answered ${String(answer.status,)}${detail}`;
}

/**
 Refuses to go on when the `git` a session would run is not the wrapper, and answers with the program when it is.

 @param run - how one command is run, which a test replaces

 @param sessionPath - the `PATH` the session would get, searched as the session's own processes search it, or nothing when the
 session would get no `PATH` at all

 @param workingFolder - the folder the session would work in, which a relative or empty entry of that `PATH` is taken against

 @param timeoutMilliseconds - how long the wrapper may take to answer, which a test shortens

 @returns the path of the `git` the session would run, which is the wrapper

 @throws GitWrapperPreflightError when that `git` is not the wrapper, naming it, what it answered, and how to fix the environment

 @example
 ```ts
 const git = await refuseUnlessTheWrapperAnswersGit({ run: runCommandWithChildProcess, sessionPath, workingFolder, },);
 ```
 */
export async function refuseUnlessTheWrapperAnswersGit({
  run,
  sessionPath,
  workingFolder,
  timeoutMilliseconds = GIT_WRAPPER_PREFLIGHT_TIMEOUT_MILLISECONDS,
}: {
  readonly run: RunCommand;
  readonly sessionPath?: string;
  readonly workingFolder: string;
  readonly timeoutMilliseconds?: number;
},): Promise<string> {
  if (sessionPath === undefined) {
    throw new GitWrapperPreflightError(
      `The session was not started: it would get no PATH at all, so its git would come from whichever folders the system falls back to, and those hold no wrapper of this repository's. ${WRAPPER_FIX}`,
    );
  }
  /**
   Whether the folder the session would work in is there, which is the premise of every question about it.
   */
  let folderIsThere = true;
  try {
    folderIsThere = (await stat(workingFolder,)).isDirectory();
  }
  catch (error) {
    /**
     Why the folder could not be looked at.
     */
    const code = errorCodeOf(error,);
    // A folder that is not there is the refusal itself; anything else about it is a failure this function does not own and lets
    // through.
    if (code !== 'ENOENT') {
      throw error;
    }
    folderIsThere = false;
  }
  if (!folderIsThere) {
    throw new GitWrapperPreflightError(
      `The session was not started: ${workingFolder} is the folder it would work in, and that is not a folder that is there. ${WRAPPER_FIX}`,
    );
  }
  /**
   The program the session's own search would find under the name `git`.
   */
  const resolvedPath = await programOnAPath({
    name: 'git',
    path: sessionPath,
    folder: workingFolder,
  },);
  if ((typeof resolvedPath) === 'symbol') {
    /**
     The PATH a reader should look at, which an empty one searches only the folder the session works in.
     */
    const searched = (sessionPath === '')
      ? 'empty, so it searches only the folder the session works in'
      : sessionPath;
    throw new GitWrapperPreflightError(
      `The session was not started: no folder of the PATH it would get holds a git this user may run. That PATH was ${searched}. Only this repository's Git policy wrapper answers for git here: ${WHAT_A_SESSION_LOSES}. ${WRAPPER_FIX}`,
    );
  }
  /**
   What the program the session would run answered when asked for the policy namespace help.
   */
  const answer = await run({
    command: resolvedPath,
    words: [...WRAPPER_QUESTION,],
    folder: workingFolder,
    environment: {
      ...PREFLIGHT_ENVIRONMENT,
      PATH: sessionPath,
    },
    timeoutMilliseconds,
  },);
  /**
   Whether that answer is the policy namespace help, which only the wrapper prints.
   */
  const answeredTheWrapper = (answer.status === 0)
    && answer.stdout
      .includes(WRAPPER_HELP_LINE,);
  if (!answeredTheWrapper) {
    /**
     What that question answered, which the refusal names so a reader can tell a silent program from a wrong one.
     */
    const answered = answerOf({
      answer,
      timeoutMilliseconds,
    },);
    throw new GitWrapperPreflightError(
      `The session was not started: ${resolvedPath} is the git it would run in ${workingFolder}, and \`${WRAPPER_COMMAND} --help\` ${answered}. Only this repository's Git policy wrapper answers that question with the policy namespace help, and ${WHAT_A_SESSION_LOSES}. ${WRAPPER_FIX} The PATH searched was: ${sessionPath}`,
    );
  }
  return resolvedPath;
}

//endregion Refusals
