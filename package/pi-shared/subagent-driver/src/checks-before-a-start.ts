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
  mkdir,
  readFile,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { constants, } from 'node:fs';
import { join, } from 'node:path';
import type { GitRunner, } from './worktree-of-a-run.ts';
import type {
  CommandOutput,
  RunCommand,
} from './run-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { isRecord, } from './read-pi-fields.ts';
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
 outlier under load, and 95 to 145 seconds later the same evening while the machine was loaded by other agents: the same
 command, the same session, no change to Pi. Two minutes is a bound for a question that is meant to cost seconds, and it is a
 bound rather than a measurement; a Pi it kills is no longer a refused start, because a slow machine is not a broken Pi.
 */
const PI_START_CHECK_SECONDS = 120;

/**
 How long an answer about Pi is kept for, in seconds. Ten minutes covers a batch whose entries start twenty seconds apart, so a
 batch of thirty entries asks once, and it expires long before a session's extensions are rebuilt by hand. What a pass buys is a
 refused start that made no worktree; what it risks is a broken extension found by the first run instead, whose own end names what
 Pi printed. A start that is merely asking the same question again for the tenth time in a minute is the cost it avoids.
 */
export const PI_START_CHECK_MEMORY_SECONDS = 600;

/**
 The file one answer about Pi is kept in, inside the folder the runs live in.
 */
const PI_START_CHECK_FILE_NAME = 'pi-start-check.json';

/**
 How a question about Pi ended, as far as the next start needs to know.
 */
export type PiStartCheckOutcome =
  /**
   Pi loaded what its settings name and printed its help.
   */
  | 'starts'
  /**
   This check's own bound killed Pi while it printed its help.
   */
  | 'killed-by-its-own-bound'
  /**
   A signal from outside this check killed Pi while it printed its help.
   */
  | 'killed-from-outside';

/**
 One answer about Pi, kept beside the runs so the next start in the same session and checkout does not ask again.
 */
export type PiStartCheckRecord = {
  /**
   When the question was asked, as an ISO 8601 instant.
   */
  readonly askedAt: string;

  /**
   How the question ended.
   */
  readonly outcome: PiStartCheckOutcome;

  /**
   The signal that ended Pi, when a signal from outside did.
   */
  readonly signal?: string;

  /**
   The `PATH` Pi was asked with.
   */
  readonly sessionPath: string;

  /**
   The `HOME` Pi was asked with, whose settings name the extensions it loads.
   */
  readonly home: string;

  /**
   The folder Pi was asked in, whose settings and trust it reads.
   */
  readonly folder: string;
};

/**
 Reads the file one answer about Pi is kept in.

 @param runsFolder - folder the runs live in

 @returns the file's path

 @example
 ```ts
 const file = piStartCheckFileIn({ runsFolder: '/runs', },);
 ```
 */
export function piStartCheckFileIn({ runsFolder, }: { readonly runsFolder: string },): string {
  return join(
    runsFolder,
    PI_START_CHECK_FILE_NAME,
  );
}

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
 the cheapest invocation measured that does. Its answer is kept for {@link PI_START_CHECK_MEMORY_SECONDS} beside the runs, so a
 batch asks it once and a start that follows within that window asks nothing; a session, a `HOME` or a checkout other than the
 one the answer was asked about asks again. A Pi this check's own bound kills is no reason to refuse a run: the start goes ahead
 with a warning that says what killed it, because a loaded machine is not a broken Pi.

 @param run - how a command is run

 @param sessionPath - the `PATH` the session's own processes get

 @param home - the `HOME` Pi would run with, whose settings name the extensions to load

 @param folder - the folder Pi would start in, whose settings and trust Pi reads

 @returns the problem in words, the warning to print when the question could not be answered, or neither when Pi starts and
 loads what its settings name

 @example
 ```ts
 const problem = await piProblemIn({ run, sessionPath, home: '/home/user', folder: '/repo', runsFolder: '/runs', },);
 ```
 */
export async function piProblemIn({
  run,
  sessionPath,
  home,
  folder,
  runsFolder,
}: {
  readonly run: RunCommand;
  readonly sessionPath: string;
  readonly home: string;
  readonly folder: string;

  /**
   Folder the runs live in, where an answer is kept for the starts that follow; no answer is remembered without it.
   */
  readonly runsFolder?: string;
},): Promise<PiStartAnswer> {
  /**
   The answer an earlier start left, when it is this same question asked about the same session.
   */
  const remembered = (runsFolder === undefined)
    ? undefined
    : await rememberedPiStartCheck({
      runsFolder,
      sessionPath,
      home,
      folder,
    },);
  if (remembered !== undefined) {
    return reusedPiStartCheck({ record: remembered, },);
  }
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
   How the question ended, which decides what a reader is told.
   */
  const outcome = outcomeOfPiStartAnswer({ answered, },);
  if (outcome.record !== undefined) {
    if (runsFolder !== undefined) {
      await rememberPiStartCheck({
        runsFolder,
        record: {
          askedAt: new Date().toISOString(),
          ...outcome.record,
          sessionPath,
          home,
          folder,
        },
      },);
    }
  }
  return {
    ...(outcome.problem === undefined ? {} : { problem: outcome.problem, }),
    ...(outcome.warning === undefined ? {} : { warning: outcome.warning, }),
  };
}

/**
 Says what one answer about Pi means: a refusal, a warning that the question could not be answered, or the answer a later start
 may reuse.

 @param answered - what `pi --help` answered

 @returns the line to refuse or warn with, and the answer to keep when the start goes ahead

 @example
 ```ts
 const outcome = outcomeOfPiStartAnswer({ answered: await run(request,), },);
 ```
 */
function outcomeOfPiStartAnswer({ answered, }: { readonly answered: CommandOutput },): {
  readonly problem?: string;
  readonly warning?: string;
  readonly record?: Omit<PiStartCheckRecord, 'askedAt' | 'sessionPath' | 'home' | 'folder'>;
} {
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
  if (loadFailure !== undefined) {
    return { problem: `Pi would not start in this session, and said: ${loadFailure}. A run started in this session would end before its first turn.`, };
  }
  if (answered.status === 0) {
    return { record: { outcome: 'starts', }, };
  }
  // A signal is not a verdict on Pi: this check's own bound is what ended a Pi that was merely slow on a loaded machine, and a
  // signal from anywhere else is someone else's doing. Either way the start goes ahead without the check, and the line says
  // which of the two happened, because `Command failed: pi --help` named neither.
  if (answered.killedByItsOwnBound === true) {
    return {
      warning: `Pi was killed by this check's own bound after ${String(PI_START_CHECK_SECONDS,)} seconds while it printed its help, so its extensions were not checked: a slow machine is not a broken Pi. This start goes ahead.`,
      record: { outcome: 'killed-by-its-own-bound', },
    };
  }
  if (answered.signal !== undefined) {
    return {
      warning: `Pi was killed by ${answered.signal} from outside this check while it printed its help, so its extensions were not checked. This start goes ahead.`,
      record: {
        outcome: 'killed-from-outside',
        signal: answered.signal,
      },
    };
  }
  if (answered.status === (-1)) {
    return { problem: `Pi could not be started at all: ${answered.problem ?? 'it did not answer, and said nothing about why'}. A run started in this session would end before its first turn.`, };
  }
  /**
   What Pi complained about, which a failed start of its own is.
   */
  const complaint = answered.stderr
    .trim();
  return {
    problem: `Pi failed and printed this: ${(complaint === '') ? 'nothing' : complaint} (\`pi ${PI_START_CHECK_WORD}\` answered ${String(answered.status,)}). A run started in this session would end before its first turn.`,
  };
}

/**
 Says what a remembered answer means for the start that reads it, naming how long ago it was asked.

 @param record - the answer an earlier start kept

 @returns the warning to print, or neither when that answer was a pass

 @example
 ```ts
 const answer = reusedPiStartCheck({ record, },);
 ```
 */
function reusedPiStartCheck({ record, }: { readonly record: PiStartCheckRecord },): {
  readonly problem?: string;
  readonly warning?: string;
} {
  if (record.outcome === 'starts') {
    return {};
  }
  /**
   How long ago the answer was asked, rounded to whole seconds for the line.
   */
  const secondsAgo = Math.max(
    0,
    Math.round((Date.now() - Date.parse(record.askedAt,)) / MILLISECONDS_PER_SECOND),
  );
  /**
   How the killed Pi was described, which says which of the two kills it was.
   */
  const killedBy = (record.outcome === 'killed-by-its-own-bound')
    ? `this check's own bound after ${String(PI_START_CHECK_SECONDS,)} seconds`
    : `${record.signal ?? 'a signal'} from outside this check`;
  return {
    warning: `Pi was killed by ${killedBy} when it was asked ${String(secondsAgo,)} seconds ago, so its extensions are still unchecked and this start goes ahead.`,
  };
}

/**
 Reads the answer an earlier start kept, when it is about this same question.

 A record that is missing, unreadable, malformed, about another session or checkout, or older than the memory is no answer at
 all, so the question is asked again; none of those is a failure of this start.

 @param runsFolder - folder the runs live in

 @param sessionPath - the `PATH` this start would ask Pi with

 @param home - the `HOME` this start would ask Pi with

 @param folder - the folder this start would ask Pi in

 @returns the answer, or nothing when this start must ask

 @example
 ```ts
 const remembered = await rememberedPiStartCheck({ runsFolder: '/runs', sessionPath: '/bin', home: '/home/a', folder: '/repo', },);
 ```
 */
async function rememberedPiStartCheck({
  runsFolder,
  sessionPath,
  home,
  folder,
}: {
  readonly runsFolder: string;
  readonly sessionPath: string;
  readonly home: string;
  readonly folder: string;
}): Promise<PiStartCheckRecord | undefined> {
  /**
   The file the last answer was kept in.
   */
  const file = piStartCheckFileIn({ runsFolder, },);
  try {
    /**
     What the file holds, of any shape.
     */
    const kept: unknown = JSON.parse(await readFile(
      file,
      'utf8',
    ),);
    if ((!isRecord(kept,)) || ((typeof kept.askedAt) !== 'string') || ((typeof kept.outcome) !== 'string')) {
      return undefined;
    }
    /**
     The outcome the file names, when it is one this check knows.
     */
    const outcome = kept.outcome;
    if ((outcome !== 'starts') && (outcome !== 'killed-by-its-own-bound') && (outcome !== 'killed-from-outside')) {
      return undefined;
    }
    if ((kept.sessionPath !== sessionPath) || (kept.home !== home) || (kept.folder !== folder)) {
      return undefined;
    }
    /**
     How old the answer is, in milliseconds, which is nothing to the answer when its instant cannot be read.
     */
    const age = Date.now() - Date.parse(kept.askedAt,);
    if ((!Number.isFinite(age,)) || (age >= (PI_START_CHECK_MEMORY_SECONDS * MILLISECONDS_PER_SECOND))) {
      return undefined;
    }
    return {
      askedAt: kept.askedAt,
      outcome,
      ...((typeof kept.signal) === 'string' ? { signal: kept.signal, } : {}),
      sessionPath,
      home,
      folder,
    };
  }
  catch (error) {
    // A record nobody can read is a question nobody has asked yet, and asking it costs one command; nothing here can make a
    // start fail.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (error instanceof SyntaxError)) {
      return undefined;
    }
    return undefined;
  }
}

/**
 Keeps one answer about Pi for the starts that follow, written in one step so a reader never sees half a file.

 An answer that cannot be kept costs the next start one question and nothing else, so a runs folder that refuses the write is
 not a failure of this one.

 @param runsFolder - folder the runs live in

 @param record - the answer to keep

 @example
 ```ts
 await rememberPiStartCheck({ runsFolder: '/runs', record, },);
 ```
 */
async function rememberPiStartCheck({
  runsFolder,
  record,
}: {
  readonly runsFolder: string;
  readonly record: PiStartCheckRecord;
}): Promise<void> {
  /**
   The file the answer is kept in.
   */
  const file = piStartCheckFileIn({ runsFolder, },);
  /**
   The name this write goes to before it is renamed over the last answer, so no reader reads half of it.
   */
  const written = `${file}.${String(process.pid,)}.tmp`;
  try {
    await mkdir(runsFolder, { recursive: true, },);
    await writeFile(
      written,
      `${JSON.stringify(record, undefined, 2)}\n`,
      'utf8',
    );
    await rename(written, file,);
  }
  catch {
    // The memory is what this start saves the next one, never what this start needs: the next start asks Pi itself.
  }
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
 What one check answered about a start: the reason it is refused, the warning a start that goes ahead prints, or neither.
 */
export type PiStartAnswer = {
  /**
   Why the start is refused, when it is.
   */
  readonly problem?: string;

  /**
   What the start could not settle, which is no reason to refuse it.
   */
  readonly warning?: string;
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
   What each check found, in the order the checks run.
   */
  const problems = await Promise.all<PiStartAnswer>([
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
      runsFolder,
    },),
  ],);
  return answersIn(problems,);
}
