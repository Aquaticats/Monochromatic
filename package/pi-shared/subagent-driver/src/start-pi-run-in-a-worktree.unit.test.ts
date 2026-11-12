/**
 The worktree launcher's own preflight: a run whose session would find a git that is not this repository's policy wrapper is
 refused before it links a dependency or starts a unit, and the refusal names the git it found and how to fix the PATH.

 The launcher is started as the command it is, with stand-ins on a `PATH` of its own, so what this reaches is the wiring a
 coordinator's launch goes through rather than a function beside it.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
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

/**
 The launcher's own entry, run as the command a coordinator runs.
 */
const LAUNCHER = new URL('start-pi-run-in-a-worktree.ts', import.meta.url,).pathname;

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 Seconds the launcher may take before the test calls it a launch that never ended. A refusal is a few process starts; this only
 keeps one that hangs from holding the test.
 */
const PATIENCE_SECONDS = 60;

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it
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

await describe({
  name: 'start-pi-run-in-a-worktree',
  children: [
    it({
      name: 'refuses a run whose session would find a git that is not the policy wrapper, before it links a dependency or starts a unit',
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
         The checkout the run would work in, which the launcher asks its own git for.
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
         The git the launcher itself uses, which answers only the question this command opens with.
         */
        await writeProgram({
          folder: launcherBin,
          name: 'git',
          body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${checkout}'\n    exit 0 ;;\nesac\nexit 1\n`,
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
         The git the session would find, which answers as the system git does.
         */
        await writeProgram({
          folder: sessionBin,
          name: 'git',
          body: `printf '%s\\n' "git: 'monochromatic-git-policy-cli' is not a git command. See 'git --help'." >&2\nexit 1\n`,
        },);
        /**
         What the launcher ended with, whether it refused or answered.
         */
        let ended: {
          readonly code: number;
          readonly stdout: string;
          readonly stderr: string
        };
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
          ], {
            env: {
              PATH: [
                launcherBin,
                dirname(process.execPath,),
                '/usr/bin',
              ].join(delimiter,),
              HOME: folder.path,
            },
            encoding: 'utf8',
            timeout: PATIENCE_SECONDS * 1_000,
            killSignal: 'SIGKILL',
          },);
          ended = {
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
          ended = {
            code: ((typeof failure.code) === 'number') ? failure.code : (-1),
            stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
            stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
          };
        }
        expect(ended.code,).toBe(1,);
        expect(ended.stderr,).toContain('refused: refusing to start',);
        expect(ended.stderr,).toContain(join(
          sessionBin,
          'git',
        ),);
        expect(ended.stderr,).toContain('.cache/monochromatic-git-policy-cli-bin',);
        expect(ended.stdout,).not.toContain('dependencies:',);
        expect((await readFile(miseLog, 'utf8',)).trim(),).toBe('exec -- printenv PATH',);
        await expect(readFile(unitMarker,),).rejects.toThrow('ENOENT',);
      },
    },),
  ],
},);
