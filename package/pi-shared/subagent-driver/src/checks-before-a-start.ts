/**
 What a start checks before it makes anything: that the prompt file can be read, that the base names a commit, that the run's
 unit name is free, and that Pi starts in this session and loads its extensions.

 A start that fails one of these made nothing, so nothing has to be undone, and no provider slot is spent on a run that could not
 have worked. The checkout and the runs folder are checked too: a unit's path properties take absolute paths, and a relative one
 refuses the unit before any run starts. A start that passes every check then makes the folder a run's tools install themselves
 into, since systemd leaves a writable path that is not there out of the unit's namespace, and a tool that made that folder under
 the run's read-only checkout would be refused. Before this, a launcher took a slot, made a worktree and linked its dependencies,
 and only then did the driver find that its prompt file was not there.

 @module
 */

import {
  access,
  stat,
} from 'node:fs/promises';
import { constants, } from 'node:fs';
import {
  isAbsolute,
  join,
} from 'node:path';
import type { GitRunner, } from './worktree-of-a-run.ts';
import type { RunCommand, } from './run-command.ts';
import { CHECKOUT_MARKER, } from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  FolderAToolMakesCannotBeMadeError,
  makeTheFoldersAToolMakesIn,
} from './file-system-of-a-run.ts';
import {
  type PiStartAnswer,
  piProblemIn,
  rememberPiStartCheck,
} from './pi-start-check.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';
import { managerProblemIn, } from './the-user-manager-this-machine-has.ts';

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
 Says what stops a checkout's Git marker from being read, if anything does.

 Why: the marker is read to name the Git directory a run's commits write, and a marker that is a named pipe or a device would make
 that read wait for a writer that may never come, before any unit exists whose own time limit could end the wait. A folder is what
 a repository holds and an ordinary file is what a linked worktree holds; anything else is refused here, in words.

 @param checkoutFolder - the checkout whose marker is read

 @returns the problem in words, or nothing when the marker is a folder, an ordinary file, or not there at all

 @throws Error when the marker's own state cannot be read

 @example
 ```ts
 const problem = await markerProblemIn({ checkoutFolder: '/repo', });
 ```
 */
export async function markerProblemIn({
  checkoutFolder,
}: {
  /**
   The checkout whose marker is read.
   */
  readonly checkoutFolder: string;
},): Promise<{ readonly problem?: string }> {
  /**
   The marker itself, which is a folder in a repository and a file in a linked worktree.
   */
  const marker = join(
    checkoutFolder,
    CHECKOUT_MARKER,
  );
  try {
    /**
     What the marker is.
     */
    const entry = await stat(marker,);
    if (entry.isDirectory() || entry.isFile()) {
      return {};
    }
  }
  catch (error) {
    // A checkout with no marker is a start this launcher already refuses elsewhere, or one whose Git folders are named with the
    // missing-path prefix; nothing here is written or read for it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    throw error;
  }
  return { problem: `the marker ${marker} is neither a folder nor an ordinary file, so the Git directory a run's commits write cannot be read without waiting for a writer that may never come. Put back the ${CHECKOUT_MARKER} Git writes, or name another checkout with --checkout-folder.`, };
}

/**
 Says what stops a folder from being named in a unit's path properties, if anything does.

 Why: systemd reads `ReadOnlyPaths=` and `ReadWritePaths=` as lists of paths under the host's root, so a relative folder refuses
 the whole unit (`Invalid ReadOnlyPaths`) and the run never starts. A start says so before it makes anything, naming the option's
 folder and what to write instead.

 @param folder - the folder a start would name

 @param what - what the folder is, for the message

 @returns the problem in words, or nothing when the folder is absolute

 @example
 ```ts
 const problem = absoluteFolderProblemIn({ folder: 'sub/repo', what: 'checkout folder', });
 ```
 */
export function absoluteFolderProblemIn({
  folder,
  what,
}: {
  /**
   The folder a start would name in its unit's path properties.
   */
  readonly folder: string;

  /**
   What the folder is, which the message names.
   */
  readonly what: string;
},): { readonly problem?: string } {
  if (isAbsolute(folder,)) {
    return {};
  }
  return { problem: `the ${what} '${folder}' is not an absolute path, so the run's unit cannot name it: systemd's ReadOnlyPaths= and ReadWritePaths= are lists of paths under the host's root, and a relative one refuses the whole unit with "Invalid ReadOnlyPaths". Name it as an absolute path, starting with /.`, };
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
  const [promptAnswer, baseAnswer, nameAnswer, markerAnswer, managerAnswer] = await Promise.all([
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
    markerProblemIn({ checkoutFolder, },),
    managerProblemIn({
      run,
      folder: checkoutFolder,
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
    markerAnswer,
    managerAnswer,
    absoluteFolderProblemIn({
      folder: checkoutFolder,
      what: 'checkout folder',
    },),
    absoluteFolderProblemIn({
      folder: runsFolder,
      what: 'runs folder',
    },),
    pi,
  ],);
  // A refused start makes nothing, not even a memory: the answer about Pi is kept only once every check has passed.
  /**
   The reasons this start is refused, and the folder a run's tools install themselves into when every other check passed.
   */
  const refusals = [...checks.refusals,];
  if (refusals.length === 0) {
    try {
      await makeTheFoldersAToolMakesIn({ checkoutFolder, },);
    }
    catch (error) {
      if (!(error instanceof FolderAToolMakesCannotBeMadeError)) {
        throw error;
      }
      refusals.push(error.message,);
    }
  }
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
  return {
    refusals,
    warnings: checks.warnings,
  };
}
