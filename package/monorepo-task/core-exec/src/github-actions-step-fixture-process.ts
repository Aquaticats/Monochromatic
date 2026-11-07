/**
 Running processes and writing files for the disposable step runner: stand-in scripts, runner files that may be absent, and
 collecting a command's output.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  readFile,
  writeFile,
} from 'node:fs/promises';

/**
 Permission bits of a stand-in script: the owner reads, writes and runs it, everyone else reads and runs it.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Interpreter line of a stand-in, naming the Node that runs the tests so no `PATH` lookup is needed.

 @returns The first line of a script

 @example
 ```ts
 const line = interpreterLine();
 ```
 */
export function interpreterLine(): string {
  return `#!${process.execPath}\n`;
}

/**
 Writes an executable script.

 @param path - Where to write it

 @param source - Node script source

 @example
 ```ts
 await writeScript({ path, source: 'console.log(1)', },);
 ```
 */
export async function writeScript(
  {
    path,
    source,
  }: {
    readonly path: string;
    readonly source: string;
  },
): Promise<void> {
  await writeFile(
    path,
    `${interpreterLine()}${source}`,
  );
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

/**
 Reads a file that may not exist.

 @param path - File to read

 @returns Its text, empty when the file does not exist

 @example
 ```ts
 const text = await readIfPresent(path,);
 ```
 */
export async function readIfPresent(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    // A step that wrote nothing to a runner file leaves it absent; any other failure is real.
    if (((typeof error) === 'object') && (error !== null)
      && ('code' in error)
      && (error.code === 'ENOENT'))
      return '';
    throw error;
  }
}

/**
 Runs a command and collects its output.

 @param command - Program and arguments

 @param env - The command's whole environment

 @param cwd - Folder to run in

 @returns Status, standard output and standard error

 @example
 ```ts
 const { status, } = await collect({ command: ['true',], env: {}, cwd: '/', },);
 ```
 */
export async function collect(
  {
    command,
    env,
    cwd,
  }: {
    readonly command: readonly string[];
    readonly env: Readonly<Record<string, string>>;
    readonly cwd: string;
  },
): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string
}> {
  /**
   The command, with no input.
   */
  const child = spawn(
    command[0] ?? '',
    command.slice(1,),
    {
      cwd,
      env,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Pieces of standard output.
   */
  const out: string[] = [];
  /**
   Pieces of standard error.
   */
  const err: string[] = [];
  child.stdout
    .setEncoding('utf8',)
    .on(
    'data',
    function collectOut(chunk: string,) {
      out.push(chunk,);
    },
    );
  child.stderr
    .setEncoding('utf8',)
    .on(
    'data',
    function collectErr(chunk: string,) {
      err.push(chunk,);
    },
    );
  await once(
    child,
    'close',
  );
  return {
    status: child.exitCode ?? (-1),
    stdout: out.join('',),
    stderr: err.join('',),
  };
}
