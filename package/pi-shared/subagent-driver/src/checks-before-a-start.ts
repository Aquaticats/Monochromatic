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
import type { GitRunner, } from './worktree-of-a-run.ts';
import type { RunCommand, } from './run-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path whose permission bits refuse the caller.
 */
const PERMISSION_CODE = 'EACCES';

/**
 The state systemd answers for a unit it does not hold at all.
 */
const UNIT_NOT_FOUND_STATE = 'not-found';

/**
 What Pi prints on its standard error when an extension of the session cannot be loaded, which is the failure this check exists
 for: on 2026-11-11 a rebuilt extension imported a workspace package the checkout had not linked, and every Pi started in this
 session ended at once with `Failed to load extension ...: Cannot find module '@monochromatic-dev/async-time-core/ts'`.

 Pi reports that on `--help` as well as on a real run, because `--help` loads the session's extensions before it prints and exits
 zero whatever it found. So the check reads what Pi printed, not only how it ended.
 */
export const PI_LOAD_FAILURE_TEXT = 'Failed to load extension';

/**
 The word Pi is asked with, which loads the session's extensions and prints the help without calling a model.
 */
const PI_START_CHECK_WORD = '--help';

/**
 Seconds the question may take. `pi --help` in this checkout took 1.55 to 2.90 seconds on 2026-11-11 with a 7.89 second
 outlier under load, because loading extensions is most of Pi's start-up; a minute is a bound rather than a measurement, and it
 is what keeps a Pi that never answers from holding a start.
 */
const PI_START_CHECK_SECONDS = 60;

/**
 A start this package refused, with every reason it found.
 */
export class StartRefusedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'StartRefusedError';
}

/**
 The mode a prompt file has to grant: a file nobody may read cannot be a run's first prompt.
 */
const PROMPT_READABLE_MODE: number = constants.R_OK;

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
 Says what stops Pi from starting in this session, if anything does.

 The question is `pi --help`: it loads the extensions this session's settings name and exits without calling a model, which is
 the cheapest invocation measured that does. A cost of about two seconds per start is paid rather than remembered, because a
 remembered answer outlives the state it describes, which is the state a rebuild changes.

 @param run - how a command is run

 @param sessionPath - the `PATH` the session's own processes get

 @param home - the `HOME` Pi would run with, whose settings name the extensions to load

 @param folder - the folder Pi would start in, whose settings and trust Pi reads

 @returns the problem in words, or nothing when Pi starts and loads what its settings name

 @example
 ```ts
 const problem = await piProblemIn({ run, sessionPath, home: '/home/user', folder: '/repo', },);
 ```
 */
export async function piProblemIn({
  run,
  sessionPath,
  home,
  folder,
}: {
  readonly run: RunCommand;
  readonly sessionPath: string;
  readonly home: string;
  readonly folder: string;
}): Promise<{ readonly problem?: string }> {
  /**
   What Pi answered when asked for its help.
   */
  const answered = await run({
    command: 'pi',
    words: [PI_START_CHECK_WORD,],
    folder,
    environment: {
      PATH: sessionPath,
      HOME: home,
    },
    timeoutMilliseconds: PI_START_CHECK_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  /**
   The load failure Pi printed, when it printed one.
   */
  /**
   Every line Pi printed, on either stream, which a load failure can reach.
   */
  const printedLines = [
    ...answered.stdout
      .split('\n',),
    ...answered.stderr
      .split('\n',),
  ];
  /**
   The load failure Pi printed, when it printed one.
   */
  const loadFailure = printedLines
    .map(function withoutTrailingSpace(line,) {
      return line.trim();
    },)
    .find(function namesALoadFailure(line,) {
      return line.includes(PI_LOAD_FAILURE_TEXT,);
    },);
  if ((answered.status === 0) && (loadFailure === undefined)) {
    return {};
  }
  if (loadFailure === undefined) {
    /**
     What Pi complained about, which a failed start of its own is.
     */
    const complaint = answered.stderr
      .trim();
    /**
     What Pi answered, which is a failure of its own start-up rather than a load failure.
     */
    const answeredWith = (answered.status === (-1))
      ? (answered.problem ?? 'did not answer')
      : `answered ${String(answered.status,)}: ${complaint}`;
    return {
      problem: `Pi would not start in this session: \`pi ${PI_START_CHECK_WORD}\` ${answeredWith}. A run started in this session would end before its first turn.`,
    };
  }
  return { problem: `Pi would not start in this session, and said: ${loadFailure}. A run started in this session would end before its first turn.` };
}

/**
 Every reason a start is refused, in the order a reader needs them.

 @param promptFile - the run's first prompt

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree would be made in

 @param base - the commit or branch the run's new branch starts at

 @param run - how a command is run

 @param runName - the run's name

 @returns one line per reason, empty when the start may go ahead

 @example
 ```ts
 const refusals = await startRefusalsIn({ promptFile: '/runs/a-prompt.md', git, checkoutFolder: '/repo', base: 'main', run, runName: 'a', },);
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
},): Promise<readonly string[]> {
  /**
   What each check found, in the order the checks run.
   */
  const problems = await Promise.all([
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
    piProblemIn({
      run,
      sessionPath,
      home,
      folder: checkoutFolder,
    },),
  ],);
  return problems.flatMap(function problemOf(answer,) {
    return (answer.problem === undefined) ? [] : [answer.problem,];
  },);
}
