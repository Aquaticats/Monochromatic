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
import { WRAPPER_COMMAND, } from '../dist/final/node/index.mjs';

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

 @param request - the stand-in folders the launcher's own search finds, and the folder one run's files go in

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
}: {
  readonly folder: string;
  readonly launcherBin: string;
  readonly runs: string;
  readonly checkout: string;
  readonly prompt: string;
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
      '--branch=none',
      `--prompt=${prompt}`,
      '--cpu=100',
      '--memory=1G',
      '--tasks=100',
      '--wall=60',
      '--turn-cap=10',
      `--runs-folder=${runs}`,
      `--checkout-folder=${checkout}`,
    ], {
      env: {
        PATH: [
          launcherBin,
          dirname(process.execPath,),
          '/usr/bin',
        ].join(delimiter,),
        HOME: folder,
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
         What the launcher ended with.
         */
        const ended = await launch({
          folder: folder.path,
          launcherBin,
          runs,
          checkout,
          prompt,
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
  ],
},);
