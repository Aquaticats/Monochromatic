/**
 The worktree launcher's own preflight: a run whose session would find a git that is not this repository's policy wrapper is
 refused before the launcher asks git anything else, so a reader is told the wrong git rather than what that git answered, and
 before it links a dependency or starts a unit.

 Two orders of failure are covered: a session whose git is not the wrapper while the base is also unreadable, where the wrapper
 is the reason, and a session whose git is the wrapper while the base names no commit, where the base is.

 The launcher is started as the command it is, with stand-ins on a `PATH` of its own, so what this reaches is the wiring a
 coordinator's launch goes through rather than a function beside it.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { existsSync, } from 'node:fs';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import { execFile, } from 'node:child_process';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  CLOSED_POOL_EXIT_STATUS,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_STANDARD,
  OPENCODE_GO,
  POOL_LIMITS_VARIABLE,
  QWEN_PLAN_DEEPSEEK,
  SUMMARY_FILE_NAME,
  WRAPPER_COMMAND,
  entryLineOf,
  reasonInFailedStart,
} from '../dist/final/node/index.mjs';

/**
 The launcher's own entry, run as the command a coordinator runs.
 */
const LAUNCHER = new URL('start-pi-run-in-a-worktree.ts', import.meta.url,).pathname;

/**
 The line the policy wrapper's help starts with, which is the one identity signal the preflight reads. It is built from the
 package's own word for that command, so a rename breaks this fixture rather than leaving it standing for a program nothing
 answers.
 */
const WRAPPER_HELP_LINE = `Usage: git ${WRAPPER_COMMAND} <command> [options]`;

/**
 The error Pi printed when the settled-notification extension could not be loaded, which is what a Pi that leaves before its first
 turn leaves in `stderr.log`.
 */
const EXTENSION_ERROR = 'Error: Failed to load extension '
  + '"/repo/package/pi-plugin/agent-settled-notification/dist/final/node/index.mjs": '
  + "Failed to load extension: Cannot find module '@monochromatic-dev/async-time-core/ts'";

/**
 The advice Pi prints after an extension it could not load, which is not the reason the run did not start.
 */
const PI_ADVICE = 'Hint: Start without extensions using "pi -ne".';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Seconds the launcher may take before the test calls it a launch that never ended. A refusal is a few process starts; this only
 keeps one that hangs from holding the test.
 */
const PATIENCE_SECONDS = 60;

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-launcher-preflight-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Writes an executable stand-in program.

 @param request - where it goes, its name, and what it does

 @returns the path of the program written

 @example
 ```ts
 const git = await writeProgram({ folder: bin, name: 'git', body: 'exit 0\n', },);
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
}): Promise<string> {
  /**
   The program's file.
   */
  const file = join(
    folder,
    name,
  );
  await mkdir(
    folder,
    { recursive: true, },
  );
  await writeFile(file, `#!/bin/sh\n${body}`,);
  await chmod(file, 0o700,);
  return file;
}

/**
 Reads what a stand-in logged, which is empty when it was never asked anything.

 @param file - the stand-in's log

 @returns the log's text, or empty text when the stand-in wrote none

 @throws Error when the log is there and cannot be read, which would hide what the stand-in was asked

 @example
 ```ts
 const asked = await logIn(join(folder.path, 'mise.log'));
 ```
 */
async function logIn(file: string,): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    // A stand-in that was asked nothing writes no log at all, which is the empty answer; any other failure would hide what it
    // was asked, so it is carried out.
    if ((error as NodeJS.ErrnoException).code !== MISSING_PATH_CODE) {
      throw error;
    }
    return '';
  }
}

/**
 What one run of the launcher ended with, whether it refused or answered.

 @param request - the stand-in folders the launcher's own search finds, the folder one run's files go in, the branch the run works on, the provider and model it names, the environment the launcher runs with, and the run folder number to ask for, when the run asks for one

 @returns the status it exited with, and both streams

 @example
 ```ts
 const ended = await launch({ folder, launcherBin, runs, checkout, prompt });
 ```
 */
async function launch({
  folder,
  launcherBin,
  runs,
  checkout,
  prompt,
  branch = 'none',
  provider,
  model,
  environment = {},
  runNumber,
}: {
  readonly folder: string;
  readonly launcherBin: string;
  readonly runs: string;
  readonly checkout: string;
  readonly prompt: string;
  readonly branch?: string;
  readonly provider?: string;
  readonly model?: string;
  readonly environment?: Readonly<Record<string, string>>;
  readonly runNumber?: number;
}): Promise<{
  readonly code: number;
  readonly stdout: string;
  readonly stderr: string
}> {
  try {
    /**
     What a launcher that answered printed.
     */
    const answered = await execFileAsync(process.execPath, [
      LAUNCHER,
      '--name=refused',
      `--branch=${branch}`,
      `--prompt=${prompt}`,
      '--cpu=100',
      '--memory=1G',
      '--tasks=100',
      '--wall=60',
      '--turn-cap=10',
      `--runs-folder=${runs}`,
      `--checkout-folder=${checkout}`,
      ...((provider === undefined) ? [] : [`--provider=${provider}`,]),
      ...((model === undefined) ? [] : [`--model=${model}`,]),
      ...((runNumber === undefined) ? [] : [`--run=${String(runNumber,)}`,]),
    ], {
      env: {
        PATH: [
          launcherBin,
          dirname(process.execPath,),
          '/usr/bin',
        ].join(delimiter,),
        HOME: folder,
        ...environment,
      },
      encoding: 'utf8',
      timeout: PATIENCE_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {
      code: 0,
      stdout: answered.stdout,
      stderr: answered.stderr,
    };
  }
  catch (error) {
    /**
     What the launcher's own failure reported, read field by field because a refusal carries the status it exited with.
     */
    const failure = error as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      code: ((typeof failure.code) === 'number') ? failure.code : (-1),
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

/**
 Writes the stand-ins a launcher walks through to reach its unit: a git that resolves the base, a mise that answers the session's
 `PATH` and the node the unit runs, a manager that answers a free unit name, a unit manager that writes what a driver writes and
 fails, and a session whose git is the wrapper and whose Pi prints its help.

 @param request - the folder the launcher's own search path finds, the session's own search path, the checkout folder, the prompt
 file, the marker the unit manager leaves, and the file it writes the words it was asked with to

 @returns nothing, once every program is written

 @example
 ```ts
 await standInsThatReachTheUnit({ launcherBin, sessionBin, checkout, prompt, unitMarker, unitWords });
 ```
 */
async function standInsThatReachTheUnit({
  launcherBin,
  sessionBin,
  checkout,
  prompt,
  unitMarker,
  unitWords,
}: {
  readonly launcherBin: string;
  readonly sessionBin: string;
  readonly checkout: string;
  readonly prompt: string;
  readonly unitMarker: string;
  readonly unitWords: string;
},): Promise<void> {
  await mkdir(
    checkout,
    { recursive: true, },
  );
  await writeFile(prompt, 'do the work',);
  /**
   The git the launcher itself uses, which records nothing and resolves the base and the checkout's root.
   */
  await writeProgram({
    folder: launcherBin,
    name: 'git',
    body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
  },);
  /**
   The mise the launcher itself uses, which answers the session's `PATH` and the node the unit runs.
   */
  await writeProgram({
    folder: launcherBin,
    name: 'mise',
    body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\n  *'command -v node'*)\n    printf '%s\\n' '${process.execPath}'\n    exit 0 ;;\nesac\nexit 1\n`,
  },);
  /**
   The manager this test reads a unit's name from, which is free because no unit holds it. It answers slowly enough that the
   unit's waiter has ended by the time the launcher looks again, so the wait reads the waiter gone rather than spending its
   one-minute look interval.
   */
  await writeProgram({
    folder: launcherBin,
    name: 'systemctl',
    body: `sleep 0.2\nprintf '%s\\n' 'not-found'\nexit 0\n`,
  },);
  /**
   The unit manager, which writes the words it was asked with, leaves the marker, writes what a driver writes for a run whose Pi
   left, prints the block systemd-run prints for a failed unit, and ends with a failing status.
   */
  await writeProgram({
    folder: launcherBin,
    name: 'systemd-run',
    body: `for word in "$@"; do\n  case "$word" in\n    --run-folder=*)\n      runFolder="\${word#--run-folder=}"\n      ;;\n  esac\ndone\nprintf '%s\\n' "$@" > '${unitWords}'\nprintf '%s\\n' 'the unit started' > '${unitMarker}'\nprintf '%s\\n' '{"endReason":"pi-exited","piExit":{"code":1}}' > "$runFolder/summary.json"\nprintf '%s\\n' 'Pi left before it took the prompt' > "$runFolder/stderr.log"\nexit 1\n`,
  },);
  /**
   The git the session would find, which answers the policy namespace help as the wrapper does.
   */
  await writeProgram({
    folder: sessionBin,
    name: 'git',
    body: `printf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
  },);
  /**
   The Pi the session would find, which loads its extensions and prints its help.
   */
  await writeProgram({
    folder: sessionBin,
    name: 'pi',
    body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
  },);
}

/**
 Launches one run through the stand-ins that reach its unit, in a folder of the test's own that goes when the answer is read.

 @param request - the provider and model the run names, when it names them; the environment the launcher runs with; and the
 branch the run works on

 @returns what the launcher ended with, whether its unit manager was reached, the words that manager was asked with, and what its
 runs and checkout folders hold

 @example
 ```ts
 const answered = await launchThroughTheStandIns({ provider: OPENCODE_GO.provider, model: OPENCODE_GO.model });
 ```
 */
async function launchThroughTheStandIns({
  provider,
  model,
  environment = {},
  branch = 'none',
}: {
  readonly provider?: string;
  readonly model?: string;
  readonly environment?: Readonly<Record<string, string>>;
  readonly branch?: string;
},): Promise<{
  readonly code: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly unitWasReached: boolean;
  readonly unitWords: readonly string[];
  readonly runsFolderHolds: readonly string[];
  readonly checkoutHolds: readonly string[];
}> {
  await using folder = await makeFolder();
  /**
   Where the stand-ins of the launcher's own search path live.
   */
  const launcherBin = join(
    folder.path,
    'launcher-bin',
  );
  /**
   Where the stand-in of the session's own search path lives.
   */
  const sessionBin = join(
    folder.path,
    'session-bin',
  );
  /**
   The checkout the run would work in.
   */
  const checkout = join(
    folder.path,
    'checkout',
  );
  /**
   The folder the run's folder goes in.
   */
  const runs = join(
    folder.path,
    'runs',
  );
  /**
   The prompt of the run.
   */
  const prompt = join(
    folder.path,
    'prompt.md',
  );
  /**
   The marker the unit manager leaves, which is what says the launcher got past every check.
   */
  const unitMarker = join(
    folder.path,
    'unit-started',
  );
  /**
   The words the unit manager was asked with, one per line, which is where a control reads what the unit was given.
   */
  const unitWords = join(
    folder.path,
    'unit-words',
  );
  await mkdir(
    runs,
    { recursive: true, },
  );
  await standInsThatReachTheUnit({
    launcherBin,
    sessionBin,
    checkout,
    prompt,
    unitMarker,
    unitWords,
  },);
  /**
   What the launcher ended with.
   */
  const ended = await launch({
    folder: folder.path,
    launcherBin,
    runs,
    checkout,
    prompt,
    branch,
    ...((provider === undefined) ? {} : { provider, }),
    ...((model === undefined) ? {} : { model, }),
    environment,
  },);
  return {
    ...ended,
    unitWasReached: existsSync(unitMarker,),
    unitWords: (await logIn(unitWords,)).split('\n',)
      .filter(function isAWord(line,) {
        return line !== '';
      },),
    runsFolderHolds: await readdir(runs,),
    checkoutHolds: await readdir(checkout,),
  };
}

await describe({
  name: 'start-pi-run-in-a-worktree',
  children: [
    it({
      name: 'refuses a run whose session would find a git that is not the policy wrapper before it asks git for the base, links a dependency or starts a unit',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the stand-ins of the launcher's own search path live.
         */
        const launcherBin = join(
          folder.path,
          'launcher-bin',
        );
        /**
         Where the stand-in of the session's own search path lives, which is what the launcher hands the session.
         */
        const sessionBin = join(
          folder.path,
          'session-bin',
        );
        /**
         The checkout the run would work in, which the launcher was told about.
         */
        const checkout = join(
          folder.path,
          'checkout',
        );
        /**
         The folder the run's folder goes in.
         */
        const runs = join(
          folder.path,
          'runs',
        );
        /**
         The prompt of the run.
         */
        const prompt = join(
          folder.path,
          'prompt.md',
        );
        /**
         Where the launcher's own mise stand-in records what it was asked.
         */
        const miseLog = join(
          folder.path,
          'mise.log',
        );
        /**
         Where the launcher's own git stand-in records what it was asked, which is how the test sees that the base was never
         asked for.
         */
        const launcherGitLog = join(
          folder.path,
          'launcher-git.log',
        );
        /**
         The marker a unit would leave, which must never appear.
         */
        const unitMarker = join(
          folder.path,
          'unit-started',
        );
        await mkdir(
          checkout,
          { recursive: true, },
        );
        await mkdir(
          runs,
          { recursive: true, },
        );
        await writeFile(prompt, 'do the work',);
        /**
         The git the launcher itself uses, which records every question and answers only the checkout's root.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `printf '%s\\n' "$*" >> '${launcherGitLog}'\ncase "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The mise the launcher itself uses, which answers the session's PATH and would record a dependency link.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'mise',
          body: `printf '%s\\n' "$*" >> '${miseLog}'\ncase "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The unit manager the launcher must never reach.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemd-run',
          body: `printf '%s\\n' "$*" > '${unitMarker}'\nexit 0\n`,
        },);
        /**
         The manager this test reads a unit's name from, which is free because no unit holds it.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemctl',
          body: `printf '%s\\n' 'not-found'\nexit 0\n`,
        },);
        /**
         The git the session would find, which answers as the system git does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' "git: '${WRAPPER_COMMAND}' is not a git command. See 'git --help'." >&2\nexit 1\n`,
        },);
        /**
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
        },);
        expect(ended.code,).toBe(1,);
        expect(ended.stderr,).toContain('refused: refusing to start',);
        expect(ended.stderr,).toContain(join(
          sessionBin,
          'git',
        ),);
        expect(ended.stderr,).toContain('.cache/monochromatic-git-policy-cli-bin',);
        // The base is read with git, and a git that is the wrong program cannot say whether the base names a commit, so the
        // wrong git is the reason a reader gets and the base is never asked about.
        expect(ended.stderr,).not.toContain('names no commit',);
        expect(await logIn(launcherGitLog,),).not.toContain('rev-parse --verify',);
        expect(ended.stdout,).not.toContain('dependencies:',);
        expect((await logIn(miseLog,)).trim(),).toBe('exec -- printenv PATH',);
        await expect(readFile(unitMarker,),).rejects.toThrow('ENOENT',);
        await expect(readdir(runs,),).resolves.toEqual([],);
      },
    },),
    it({
      name: 'refuses a run whose base names no commit, and says so only after the session git is shown to be the policy wrapper',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the stand-ins of the launcher's own search path live.
         */
        const launcherBin = join(
          folder.path,
          'launcher-bin',
        );
        /**
         Where the stand-in of the session's own search path lives, which is what the launcher hands the session.
         */
        const sessionBin = join(
          folder.path,
          'session-bin',
        );
        /**
         The checkout the run would work in, which the launcher was told about and whose `main` holds no commit.
         */
        const checkout = join(
          folder.path,
          'checkout',
        );
        /**
         The folder the run's folder goes in.
         */
        const runs = join(
          folder.path,
          'runs',
        );
        /**
         The prompt of the run, which can be read.
         */
        const prompt = join(
          folder.path,
          'prompt.md',
        );
        /**
         Where the launcher's own git stand-in records what it was asked.
         */
        const launcherGitLog = join(
          folder.path,
          'launcher-git.log',
        );
        /**
         Where the session's git stand-in records what it was asked.
         */
        const sessionGitLog = join(
          folder.path,
          'session-git.log',
        );
        /**
         The marker a unit would leave, which must never appear.
         */
        const unitMarker = join(
          folder.path,
          'unit-started',
        );
        await mkdir(
          checkout,
          { recursive: true, },
        );
        await mkdir(
          runs,
          { recursive: true, },
        );
        await writeFile(prompt, 'do the work',);
        /**
         The git the launcher itself uses, which records every question and resolves no base.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `printf '%s\\n' "$*" >> '${launcherGitLog}'\ncase "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\nesac\nprintf '%s\\n' 'fatal: Needed a single revision' >&2\nexit 1\n`,
        },);
        /**
         The mise the launcher itself uses, which answers the session's PATH.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'mise',
          body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The unit manager the launcher must never reach.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemd-run',
          body: `printf '%s\\n' "$*" > '${unitMarker}'\nexit 0\n`,
        },);
        /**
         The manager this test reads a unit's name from, which is free because no unit holds it.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemctl',
          body: `printf '%s\\n' 'not-found'\nexit 0\n`,
        },);
        /**
         The git the session would find, which answers the policy namespace help as the wrapper does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' "$*" >> '${sessionGitLog}'\nprintf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
        },);
        /**
         The Pi the session would find, which prints its help and loads the extensions of an empty home.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'pi',
          body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
        },);
        /**
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
        },);
        expect(ended.code,).toBe(1,);
        // The wrapper answered before the base was read, so the line saying so is there: a launcher that asked the base first
        // would have refused before printing it.
        expect(ended.stdout,).toContain('answers as the policy wrapper',);
        expect(ended.stdout,).not.toContain('dependencies:',);
        expect(ended.stderr,).toContain('refused was not started',);
        expect(ended.stderr,).toContain('the base main names no commit',);
        expect(await logIn(sessionGitLog,),).toContain(`${WRAPPER_COMMAND} --help`,);
        expect(await logIn(launcherGitLog,),).toContain('rev-parse --verify --quiet main^{commit}',);
        await expect(readFile(unitMarker,),).rejects.toThrow('ENOENT',);
        await expect(readdir(runs,),).resolves.toEqual([],);
      },
    },),
    it({
      name: 'refuses a run whose folder already holds the summary of a run that finished, and leaves that summary as it was',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the stand-ins of the launcher's own search path live.
         */
        const launcherBin = join(
          folder.path,
          'launcher-bin',
        );
        /**
         Where the stand-in of the session's own search path lives.
         */
        const sessionBin = join(
          folder.path,
          'session-bin',
        );
        /**
         The checkout the run would work in.
         */
        const checkout = join(
          folder.path,
          'checkout',
        );
        /**
         The folder the run folders go in.
         */
        const runs = join(
          folder.path,
          'runs',
        );
        /**
         The prompt of the run, which can be read.
         */
        const prompt = join(
          folder.path,
          'prompt.md',
        );
        /**
         The folder the run was told to write into, which holds the summary of a run that finished.
         */
        const runFolder = join(
          runs,
          'refused-5',
        );
        /**
         The summary that was there before the launcher ran, in the bytes a reader would parse.
         */
        const summary = '{"endReason":"idle","lastAssistantText":"the review is written"}\n';
        /**
         The marker a unit would leave, which must never appear.
         */
        const unitMarker = join(
          folder.path,
          'unit-started',
        );
        await mkdir(
          checkout,
          { recursive: true, },
        );
        await mkdir(
          runFolder,
          { recursive: true, },
        );
        await writeFile(prompt, 'do the work',);
        await writeFile(
          join(
            runFolder,
            SUMMARY_FILE_NAME,
          ),
          summary,
        );
        /**
         The git the launcher itself uses, which resolves the base and the checkout's root.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The mise the launcher itself uses, which answers the session's PATH.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'mise',
          body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The unit manager the launcher must never reach.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemd-run',
          body: `printf '%s\\n' "$*" > '${unitMarker}'\nexit 0\n`,
        },);
        /**
         The manager this test reads a unit's name from, which is free because no unit holds it.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemctl',
          body: `printf '%s\\n' 'not-found'\nexit 0\n`,
        },);
        /**
         The git the session would find, which answers the policy namespace help as the wrapper does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
        },);
        /**
         The Pi the session would find, which prints its help and loads the extensions of an empty home.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'pi',
          body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
        },);
        /**
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
          runNumber: 5,
        },);
        expect(ended.code,).toBe(1,);
        expect(ended.stderr,).toContain('refused was not started',);
        expect(ended.stderr,).toContain(runFolder,);
        expect(ended.stderr,).toContain(SUMMARY_FILE_NAME,);
        expect(ended.stderr,).toContain('--run=<number>',);
        expect(ended.stdout,).not.toContain('dependencies:',);
        await expect(readFile(unitMarker,),).rejects.toThrow('ENOENT',);
        expect(await readdir(runFolder,),).toEqual([SUMMARY_FILE_NAME,],);
        expect(await readFile(
          join(
            runFolder,
            SUMMARY_FILE_NAME,
          ),
          'utf8',
        ),).toBe(summary,);
      },
    },),
    it({
      name: 'refuses a run whose folder holds what an attempt killed before its summary left, and leaves that attempt records as they were',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the stand-ins of the launcher's own search path live.
         */
        const launcherBin = join(
          folder.path,
          'launcher-bin',
        );
        /**
         Where the stand-in of the session's own search path lives.
         */
        const sessionBin = join(
          folder.path,
          'session-bin',
        );
        /**
         The checkout the run would work in.
         */
        const checkout = join(
          folder.path,
          'checkout',
        );
        /**
         The folder the run folders go in.
         */
        const runs = join(
          folder.path,
          'runs',
        );
        /**
         The prompt of the run, which can be read.
         */
        const prompt = join(
          folder.path,
          'prompt.md',
        );
        /**
         The folder the run was told to write into, which holds what an attempt at it left when it was killed before its summary.
         */
        const runFolder = join(
          runs,
          'refused-5',
        );
        /**
         What that attempt printed as it crashed, which no later attempt may truncate away.
         */
        const earlierStandardError = 'ORIGINAL-CRASHED-RUN-STDERR\n';
        /**
         The marker a unit would leave, which must never appear.
         */
        const unitMarker = join(
          folder.path,
          'unit-started',
        );
        await mkdir(
          checkout,
          { recursive: true, },
        );
        await mkdir(
          runFolder,
          { recursive: true, },
        );
        await writeFile(prompt, 'do the work',);
        await writeFile(
          join(
            runFolder,
            'stderr.log',
          ),
          earlierStandardError,
        );
        /**
         The git the launcher itself uses, which resolves the base and the checkout's root.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The mise the launcher itself uses, which answers the session's PATH.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'mise',
          body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The unit manager the launcher must never reach.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemd-run',
          body: `printf '%s\\n' "$*" > '${unitMarker}'\nexit 0\n`,
        },);
        /**
         The manager this test reads a unit's name from, which is free because no unit holds it.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemctl',
          body: `printf '%s\\n' 'not-found'\nexit 0\n`,
        },);
        /**
         The git the session would find, which answers the policy namespace help as the wrapper does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
        },);
        /**
         The Pi the session would find, which prints its help and loads the extensions of an empty home.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'pi',
          body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
        },);
        /**
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
          runNumber: 5,
        },);
        expect(ended.code,).toBe(1,);
        expect(ended.stderr,).toContain('refused was not started',);
        expect(ended.stderr,).toContain(runFolder,);
        expect(ended.stderr,).toContain('stderr.log',);
        expect(ended.stdout,).not.toContain('dependencies:',);
        await expect(readFile(unitMarker,),).rejects.toThrow('ENOENT',);
        expect(await readdir(runFolder,),).toEqual(['stderr.log',],);
        expect(await readFile(
          join(
            runFolder,
            'stderr.log',
          ),
          'utf8',
        ),).toBe(earlierStandardError,);
      },
    },),
    it({
      name: 'says what Pi printed when Pi left before the run started, so the batch names the cause and not the unit memory line',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the stand-ins of the launcher's own search path live.
         */
        const launcherBin = join(
          folder.path,
          'launcher-bin',
        );
        /**
         Where the stand-in of the session's own search path lives.
         */
        const sessionBin = join(
          folder.path,
          'session-bin',
        );
        /**
         The checkout the run would work in.
         */
        const checkout = join(
          folder.path,
          'checkout',
        );
        /**
         The folder the run's folder goes in.
         */
        const runs = join(
          folder.path,
          'runs',
        );
        /**
         The prompt of the run, which can be read.
         */
        const prompt = join(
          folder.path,
          'prompt.md',
        );
        await mkdir(
          checkout,
          { recursive: true, },
        );
        await mkdir(
          runs,
          { recursive: true, },
        );
        await writeFile(prompt, 'do the work',);
        /**
         The git the launcher itself uses, which resolves the base and the checkout's root.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The mise the launcher itself uses, which answers the session's PATH and the node the unit runs.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'mise',
          body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\n  *'command -v node')\n    printf '%s\\n' '${process.execPath}'\n    exit 0 ;;\nesac\nexit 1\n`,
        },);
        /**
         The manager this test reads a unit's name from, which is free because no unit holds it. It answers slowly enough that
         the unit's waiter has ended by the time the launcher looks again, so the wait reads the waiter gone rather than
         spending its one-minute look interval.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemctl',
          body: `sleep 0.2\nprintf '%s\\n' 'not-found'\nexit 0\n`,
        },);
        /**
         The unit manager, which writes what a driver writes for a Pi that would not load its extensions, prints the block
         systemd-run prints for a failed unit, and ends with a failing status.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'systemd-run',
          body: `for word in "$@"; do\n  case "$word" in\n    --run-folder=*)\n      runFolder="\${word#--run-folder=}"\n      ;;\n  esac\ndone\nprintf '%s\\n' '{"endReason":"pi-exited","piExit":{"code":1}}' > "$runFolder/summary.json"\nprintf '%s\\n' '${EXTENSION_ERROR}' '    at loadOne (file:///repo/a.ts:12:9)' '${PI_ADVICE}' > "$runFolder/stderr.log"\nprintf '%s\\n' 'Finished with result: exit-code' 'Memory peak: 12.0M'\nexit 1\n`,
        },);
        /**
         The git the session would find, which answers the policy namespace help as the wrapper does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
        },);
        /**
         The Pi the session would find, which loads its extensions and prints its help.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'pi',
          body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
        },);
        /**
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
        },);
        /**
         Everything the launcher printed, which is what a batch keeps and reads its one line from.
         */
        const printed = `${ended.stdout}${ended.stderr}`;
        expect(ended.code,).toBe(1,);
        expect(printed,).not.toContain('Memory peak: 12.0M Pi');
        // The reason comes after the unit's own block, so the line a batch prints carries Pi's words rather than the memory line.
        expect(reasonInFailedStart({ output: printed, },).reason,).toContain('Cannot find module',);
        expect(entryLineOf({
          name: 'refused',
          started: {
            status: ended.code,
            output: printed,
          },
        },),).toContain('Cannot find module',);
        expect(entryLineOf({
          name: 'refused',
          started: {
            status: ended.code,
            output: printed,
          },
        },),).not.toContain('Memory peak',);
      },
    },),
    it({
      name: 'refuses a run that names a closed pool before it makes the worktree, links a dependency or starts a unit, and names the pools to take instead',
      fn: async () => {
        /**
         The owner's reason the pool takes no new run, which the refusal carries.
         */
        const reason = NEURALWATT_STANDARD.closed?.reason ?? '';
        expect(reason.length,).toBeGreaterThan(0,);
        const answered = await launchThroughTheStandIns({
          provider: NEURALWATT_STANDARD.provider,
          model: NEURALWATT_STANDARD.model,
          branch: 'fix/the-work',
        },);
        expect(answered.code,).toBe(CLOSED_POOL_EXIT_STATUS,);
        expect(answered.stderr,).toContain(NEURALWATT_STANDARD.name,);
        expect(answered.stderr,).toContain(reason,);
        expect(answered.stderr,).toContain(`--provider=${OPENCODE_GO.provider} --model=${OPENCODE_GO.model}`,);
        expect(answered.stderr,)
          .toContain(`--provider=${NEURALWATT_DEEPSEEK_FLEX.provider} --model=${NEURALWATT_DEEPSEEK_FLEX.model}`,);
        // Neither a worktree nor a run folder was made, no unit was asked for, and nothing was asked of the session.
        expect(answered.checkoutHolds,).toEqual([],);
        expect(answered.runsFolderHolds,).toEqual([],);
        expect(answered.unitWasReached,).toBe(false,);
        expect(answered.stdout,).not.toContain('answers as the policy wrapper',);
      },
    },),
    it({
      name: 'refuses a run whose command line names no provider, and says the pool gate is how a run that names none starts',
      fn: async () => {
        const answered = await launchThroughTheStandIns({},);
        expect(answered.code,).toBe(CLOSED_POOL_EXIT_STATUS,);
        // The default provider and model name the pool the owner replaced, so the refusal names it and the gate that counts the
        // runs going before choosing one.
        expect(answered.stderr,).toContain(NEURALWATT_STANDARD.name,);
        expect(answered.stderr,).toContain('start-pi-run-when-a-pool-has-room',);
        expect(answered.stderr,).toContain(`--provider=${OPENCODE_GO.provider} --model=${OPENCODE_GO.model}`,);
        expect(answered.runsFolderHolds,).toEqual([],);
        expect(answered.unitWasReached,).toBe(false,);
      },
    },),
    it({
      name: 'takes a run naming the closed pool once the limits variable gives the pool room, and hands that variable to its unit',
      fn: async () => {
        /**
         The limit the caller gives the closed pool, which is another pool's own limit rather than a number written down here.
         */
        const given = `${NEURALWATT_STANDARD.name}=${String(NEURALWATT_DEEPSEEK_FLEX.limit,)}`;
        const answered = await launchThroughTheStandIns({
          provider: NEURALWATT_STANDARD.provider,
          model: NEURALWATT_STANDARD.model,
          environment: { [POOL_LIMITS_VARIABLE]: given, },
        },);
        expect(answered.unitWasReached,).toBe(true,);
        expect(answered.code,).not.toBe(CLOSED_POOL_EXIT_STATUS,);
        expect(answered.stderr,).not.toContain('takes no new run',);
        // The unit is given the same setting the launcher was: a unit inherits nothing from the shell that asked for it, and the
        // driver inside it reads the variable again before it starts Pi.
        expect(answered.unitWords,).toContain(`--setenv=${POOL_LIMITS_VARIABLE}=${given}`,);
        // The run's own folder was made, beside the answer about Pi kept for the starts that follow.
        expect(answered.runsFolderHolds.some(function isTheRunsFolder(entry,) {
          return entry.startsWith('refused-',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'takes a run that names an open pool, and reaches its unit',
      fn: async () => {
        const answered = await launchThroughTheStandIns({
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
        },);
        expect(answered.unitWasReached,).toBe(true,);
        expect(answered.code,).not.toBe(CLOSED_POOL_EXIT_STATUS,);
        expect(answered.stderr,).not.toContain('takes no new run',);
        // A caller who set no limits gets no limits word in the unit, so the unit's environment is never a setting of this
        // command's own making.
        expect(answered.unitWords.filter(function isTheLimitsWord(word,) {
          return word.startsWith(`--setenv=${POOL_LIMITS_VARIABLE}=`,);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'takes a run that names the qwen plan\'s DeepSeek pool, which the owner reopened for the short reviews',
      fn: async () => {
        const answered = await launchThroughTheStandIns({
          provider: QWEN_PLAN_DEEPSEEK.provider,
          model: QWEN_PLAN_DEEPSEEK.model,
        },);
        expect(answered.unitWasReached,).toBe(true,);
        expect(answered.code,).not.toBe(CLOSED_POOL_EXIT_STATUS,);
        expect(answered.stderr,).not.toContain('takes no new run',);
      },
    },),
    it({
      name: 'takes a run whose provider and model belong to no pool, and reaches its unit',
      fn: async () => {
        const answered = await launchThroughTheStandIns({
          provider: 'synthetic',
          model: 'moonshotai/Kimi-K3.1-Pro',
        },);
        expect(answered.unitWasReached,).toBe(true,);
        expect(answered.code,).not.toBe(CLOSED_POOL_EXIT_STATUS,);
        expect(answered.stderr,).not.toContain('takes no new run',);
      },
    },),
    it({
      name: 'refuses a run that names a pool when the pool limits setting cannot be read, before the worktree or a unit, naming the setting and its fault',
      fn: async () => {
        /**
         A setting this command cannot read: a limit that is no whole number.
         */
        const unreadable = `${OPENCODE_GO.name}=twenty`;
        const answered = await launchThroughTheStandIns({
          provider: OPENCODE_GO.provider,
          model: OPENCODE_GO.model,
          environment: { [POOL_LIMITS_VARIABLE]: unreadable, },
        },);
        expect(answered.code,).toBe(1,);
        expect(answered.stderr,).toContain(POOL_LIMITS_VARIABLE,);
        expect(answered.stderr,).toContain(unreadable,);
        expect(answered.stderr,).not.toContain('\n    at ',);
        expect(answered.unitWasReached,).toBe(false,);
        expect(answered.runsFolderHolds,).toEqual([],);
        expect(answered.checkoutHolds,).toEqual([],);
      },
    },),
    it({
      name: 'takes a run that belongs to no pool even when the pool limits setting cannot be read',
      fn: async () => {
        const answered = await launchThroughTheStandIns({
          provider: 'synthetic',
          model: 'moonshotai/Kimi-K3.1-Pro',
          environment: { [POOL_LIMITS_VARIABLE]: 'not-a-pool=3', },
        },);
        expect(answered.unitWasReached,).toBe(true,);
        expect(answered.stderr,).not.toContain(POOL_LIMITS_VARIABLE,);
      },
    },),
  ],
},);
