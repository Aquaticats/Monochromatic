/**
 What a start checks before it makes anything: that the prompt file can be read, that the base names a commit, that the run's
 unit name is free, and that Pi starts in this session and loads its extensions.

 A start that fails one of these made nothing, so nothing has to be undone, and no provider slot is spent on a run that could not
 have worked. Before this, a launcher took a slot, made a worktree and linked its dependencies, and only then did the driver find
 that its prompt file was not there.

 @module
 */

import {
  access,
  stat,
} from 'node:fs/promises';
import { constants, } from 'node:fs';
import { join, } from 'node:path';
import type { GitRunner, } from './worktree-of-a-run.ts';
import type { RunCommand, } from './run-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type PiStartAnswer,
  piProblemIn,
  rememberPiStartCheck,
} from './pi-start-check.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';

export {
  PI_LOAD_FAILURE_TEXT,
  PI_START_CHECK_MEMORY_SECONDS,
  type PiStartCheckOutcome,
  type PiStartAnswer,
  type PiStartCheckRecord,
  StartRefusedError,
  piProblemIn,
  piStartCheckFileIn,
  rememberPiStartCheck,
} from './pi-start-check.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The mode a prompt file has to grant: a file nobody may read cannot be a run's first prompt.
 */
const PROMPT_READABLE_MODE: number = constants.R_OK;

/**
 The code the operating system gives a path whose permission bits refuse the caller.
 */
const PERMISSION_CODE = 'EACCES';

/**
 The state systemd answers for a unit it does not hold at all.
 */
const UNIT_NOT_FOUND_STATE = 'not-found';


/**
 Says what stops a prompt file from being read, if anything does.

 @param path - the prompt file the run names

 @returns the problem in words, or nothing when the file can be read

 @example
 ```ts
 const problem = await promptProblemIn({ path: '/runs/a-prompt.md', },);
 ```
 */
export async function promptProblemIn({ path }: { readonly path: string },): Promise<{ readonly problem?: string }> {
  try {
    /**
     What is at the path a start was given.
     */
    const entry = await stat(path,);
    if (!entry.isFile()) {
      return { problem: `${path} is not a regular file, so it cannot be a run's first prompt`, };
    }
    await access(
      path,
      PROMPT_READABLE_MODE,
    );
    return {};
  }
  catch (error) {
    /**
     The code the operating system gave the path.
     */
    const code = errorCodeOf(error,);
    if (code === MISSING_PATH_CODE) {
      return { problem: `${path} is not there, so the run has no first prompt`, };
    }
    if (code === PERMISSION_CODE) {
      return { problem: `${path} cannot be read by this user, so the run has no first prompt`, };
    }
    return { problem: `${path} could not be read for a reason the start cannot act on: ${String(error,)}`, };
  }
}

/**
 Says what stops a base from naming a commit, if anything does.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the base is read in

 @param base - the commit or branch the run's new branch starts at

 @returns the problem in words, or nothing when the base names a commit

 @example
 ```ts
 const problem = await baseProblemIn({ git, checkoutFolder: '/repo', base: 'main', },);
 ```
 */
export async function baseProblemIn({
  git,
  checkoutFolder,
  base,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly base: string;
},): Promise<{ readonly problem?: string }> {
  /**
   What git answered about the base.
   */
  const answered = await git({
    folder: checkoutFolder,
    words: [
      'rev-parse',
      '--verify',
      '--quiet',
      `${base}^{commit}`,
    ],
  },);
  if (answered.status === 0) {
    return {};
  }
  /**
   What git complained about, which names the reason when it gave one.
   */
  const said = (answered.problem ?? answered.stderr).trim();
  return { problem: `the base ${base} names no commit in ${checkoutFolder}${(said === '') ? '' : `: ${said}`}`, };
}

/**
 Says what stops a run's own unit name from being free, if anything does.

 @param run - how a command is run

 @param runName - the run's name, which names its unit

 @returns the problem in words, or nothing when no unit holds the name

 @example
 ```ts
 const problem = await nameProblemIn({ run, runName: 'copy-route', },);
 ```
 */
export async function nameProblemIn({
  run,
  runName,
}: {
  readonly run: RunCommand;
  readonly runName: string;
},): Promise<{ readonly problem?: string }> {
  /**
   The unit the run would be started as.
   */
  const unitName = unitNameOfGiven({ name: runName, },);
  /**
   What the manager answered about that unit's state.
   */
  const answered = await run({
    command: 'systemctl',
    words: [
      '--user',
      'show',
      unitName,
      '--property=LoadState',
      '--value',
    ],
  },);
  /**
   The state the manager named, empty when it answered nothing.
   */
  const state = (answered.status === 0) ? answered.stdout
    .trim() : '';
  if ((state === '') || (state === UNIT_NOT_FOUND_STATE)) {
    return {};
  }
  return { problem: `${unitName} is already there (LoadState ${state}), so a run named ${runName} cannot take that name`, };
}

/**
 What a start settled before it made anything: the reasons it is refused, and the warnings a start that goes ahead prints.
 */
export type StartChecks = {
  /**
   One line per reason the start is refused, empty when it may go ahead.
   */
  readonly refusals: readonly string[];

  /**
   One line per question the start could not get an answer to, which is no reason to refuse a run.
   */
  readonly warnings: readonly string[];
};

/**
 Reads the refusals and the warnings a start's checks answered.

 @param answers - what each check answered, in the order the checks ran

 @returns the reasons and the warnings, each as its own list

 @example
 ```ts
 const { refusals, warnings } = answersIn([{ problem: 'no prompt', }, {},],);
 ```
 */
export function answersIn(answers: readonly PiStartAnswer[],): StartChecks {
  return {
    refusals: answers.flatMap(function problemOf(answer,) {
      return (answer.problem === undefined) ? [] : [answer.problem,];
    },),
    warnings: answers.flatMap(function warningOf(answer,) {
      return (answer.warning === undefined) ? [] : [answer.warning,];
    },),
  };
}

/**
 Every reason a start is refused, in the order a reader needs them, with the warnings a start that goes ahead prints.

 @param promptFile - the run's first prompt

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree would be made in

 @param base - the commit or branch the run's new branch starts at

 @param run - how a command is run

 @param runName - the run's name

 @param sessionPath - the `PATH` the session's own processes get, which the question about Pi is asked with

 @param home - the `HOME` the session runs with, whose settings name the extensions Pi loads

 @param runsFolder - folder the runs live in, where an answer about Pi is kept for the starts that follow

 @returns the refusals and the warnings

 @example
 ```ts
 const { refusals, warnings } = await startRefusalsIn({ promptFile: '/runs/a-prompt.md', git, checkoutFolder: '/repo', base: 'main', run, runName: 'a', sessionPath: '/bin', home: '/home/a', runsFolder: '/runs', },);
 ```
 */
export async function startRefusalsIn({
  promptFile,
  git,
  checkoutFolder,
  base,
  run,
  runName,
  sessionPath,
  home,
  runsFolder,
}: {
  readonly promptFile: string;
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly base: string;
  readonly run: RunCommand;
  readonly runName: string;

  /**
   The `PATH` the session's own processes get, which the question about Pi is asked with.
   */
  readonly sessionPath: string;

  /**
   The `HOME` the session runs with, whose settings name the extensions Pi loads.
   */
  readonly home: string;

  /**
   Folder the runs live in, where an answer about Pi is kept for the starts that follow.
   */
  readonly runsFolder: string;
},): Promise<StartChecks> {
  /**
   What the three checks about the run itself found, in the order a reader needs them.
   */
  const [promptAnswer, baseAnswer, nameAnswer] = await Promise.all([
    promptProblemIn({ path: promptFile, },),
    baseProblemIn({
      git,
      checkoutFolder,
      base,
    },),
    nameProblemIn({
      run,
      runName,
    },),
  ],);
  /**
   What the question about Pi answered, which is asked even when another check refuses, so a reader fixes them all at once.
   */
  const pi = await piProblemIn({
    run,
    sessionPath,
    home,
    folder: checkoutFolder,
    runsFolder,
    keepTheAnswer: false,
  },);
  /**
   What the checks answered.
   */
  const checks = answersIn([
    promptAnswer,
    baseAnswer,
    nameAnswer,
    pi,
  ],);
  // A refused start makes nothing, not even a memory: the answer about Pi is kept only once every check has passed.
  /**
   The reasons this start is refused.
   */
  const { refusals, } = checks;
  if (refusals.length === 0) {
    /**
     What the question about Pi found worth keeping, when it found anything.
     */
    const { keep, } = pi;
    if (keep !== undefined) {
      await rememberPiStartCheck({
        runsFolder,
        record: {
          askedAt: new Date().toISOString(),
          ...keep,
          sessionPath,
          home,
          folder: checkoutFolder,
        },
      },);
    }
  }
  return checks;
}
