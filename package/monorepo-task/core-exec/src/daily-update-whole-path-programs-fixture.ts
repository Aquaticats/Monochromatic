/**
 Starting a program for the whole-path fixture and finding the folders its own `PATH` names.

 `daily-update-whole-path-repository-fixture.ts` uses `run` for its `git` steps and `directoryOf` for the one `mise` the
 run needs; the fixture that lays out and drives the update uses both.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  realpath,
  stat,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';

/**
 Upper bound that stops a broken fixture from hanging verification.
 */
const WHOLE_PATH_TIMEOUT_MS = 180_000;

/**
 Runs a program to its end and returns its status and output.

 @param program - program name or path

 @param args - arguments

 @param directory - working directory

 @param environment - process environment

 @returns the exit status and the text of both output streams

 @throws Error when the program runs past the fixture's bound

 @example
 ```ts
 const { exitCode, } = await run({ program: 'git', args: ['status',], directory: '/r', environment: {}, },);
 ```
 */
export async function run({
  program,
  args,
  directory,
  environment,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly directory: string;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<{
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string
}> {
  /**
   The process, stopped when it runs past the bound.
   */
  const child = spawn(
    program,
    [...args,],
    {
      cwd: directory,
      env: environment,
      timeout: WHOLE_PATH_TIMEOUT_MS,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  // Reading both streams to their end while waiting for the exit keeps a full pipe from stalling the child.
  /**
   Both output streams, read to their end.
   */
  const [stdout, stderr,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ),
  ],);
  return {
    exitCode: child.exitCode ?? (-1),
    stdout,
    stderr,
  };
}

/**
 A folder on `PATH` with whether it holds a program.
 */
type PathFolder = {
  /**
   The folder.
   */
  readonly directory: string;
  /**
   Whether the program is a file in it.
   */
  readonly holds: boolean;
};

/**
 Resolves the directory of an executable that is on the caller's `PATH`.

 @param name - executable name

 @returns the real directory that holds it

 @throws Error when it is not on `PATH`

 @example
 ```ts
 const miseDirectory = await directoryOf('mise',);
 ```
 */
export async function directoryOf(name: string,): Promise<string> {
  // Looking in one folder does not depend on looking in another, so the folders are looked in side by side.
  /**
   Each `PATH` folder with whether it holds the program.
   */
  const candidates = await Promise.all((process.env
    .PATH
    ?? '').split(delimiter,)
    .map(async function lookIn(directory,): Promise<PathFolder> {
      /**
       Metadata of the candidate, absent when the folder has no such file.
       */
      const metadata = await stat(
        join(
          directory,
          name,
        ),
        { throwIfNoEntry: false, },
      );
      return {
        directory,
        holds: metadata?.isFile() === true,
      };
    },),);
  /**
   The first folder, in `PATH` order, that holds the program.
   */
  const found = candidates.find(function holdsIt(candidate,) {
    return candidate.holds;
  },);
  if (found === undefined)
    throw new Error(`${name} is required on PATH for the whole-path test`,);
  return dirname(
    await realpath(join(
      found.directory,
      name,
    ),),
  );
}
