/**
 What the whole-path run lays out and runs: stand-in programs, the files of the disposable repository, and the
 helpers that write files and run programs to their end.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  mkdir,
  realpath,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';

import {
  CARGO_LOCK,
  CLIPPY_SCRIPT,
  MOVE_SCRIPT,
  ROOT_MISE,
} from './daily-update-whole-path-scripts-fixture.ts';

/**
 Upper bound that stops a broken fixture from hanging verification.
 */
const WHOLE_PATH_TIMEOUT_MS = 180_000;

/**
 Writes a file, making its folder first.

 @param path - absolute path

 @param content - text

 @example
 ```ts
 await put({ path: '/r/a/b', content: 'x', },);
 ```
 */
export async function put({
  path,
  content,
}: {
  readonly path: string;
  readonly content: string;
},): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    content,
  );
}

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
 Permission bits of a stand-in program: the owner may read, write and run it, others may read and run it.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Writes an executable with the given content into a folder.

 @param directory - folder

 @param name - file name

 @param content - script text starting with a shebang

 @example
 ```ts
 await putExecutable({ directory: '/r/bin', name: 'gh', content: '#!/usr/bin/env node\n', },);
 ```
 */
export async function putExecutable({
  directory,
  name,
  content,
}: {
  readonly directory: string;
  readonly name: string;
  readonly content: string;
},): Promise<void> {
  /**
   Absolute path of the executable.
   */
  const path = join(
    directory,
    name,
  );
  await put({
    path,
    content,
  },);
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

/**
 Runs one `git` step in the repository.

 @param args - arguments after `git`

 @param root - absolute path of the repository

 @param environment - process environment for `git`

 @throws Error when `git` exits with a nonzero status

 @example
 ```ts
 await runGitStep({ args: ['init',], root: '/r', environment, },);
 ```
 */
export async function runGitStep({
  args,
  root,
  environment,
}: {
  readonly args: readonly string[];
  readonly root: string;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   How the `git` step ended.
   */
  const result = await run({
    program: 'git',
    args,
    directory: root,
    environment,
  },);
  if (result.exitCode !== 0)
    throw new Error(`git ${args.join(' ',)} exited with status ${result.exitCode}: ${result.stderr}`,);
}

/**
 Lays out the repository: update tasks, two crates, one TypeScript package, and a first commit.

 @param root - absolute path of the repository

 @param environment - process environment for `git`

 @example
 ```ts
 await layOutRepository({ root: '/r', environment: process.env, },);
 ```
 */
export async function layOutRepository({
  root,
  environment,
}: {
  readonly root: string;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<void> {
  await Promise.all([
    put({
      path: join(
        root,
        'mise.toml',
      ),
      content: ROOT_MISE,
    },),
    put({
      path: join(
        root,
        '.gitignore',
      ),
      content: '.cache/\n',
    },),
    put({
      path: join(
        root,
        'toolchain.lock.json',
      ),
      content: '{"tools":{"rust":{"version":"nightly-1"}}}\n',
    },),
    put({
      path: join(
        root,
        'steps',
        'move.ts',
      ),
      content: MOVE_SCRIPT,
    },),
    put({
      path: join(
        root,
        'steps',
        'clippy.ts',
      ),
      content: CLIPPY_SCRIPT,
    },),
    put({
      path: join(
        root,
        'package',
        'a',
        'broken-crate',
        'mise.toml',
      ),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0433"\n',
    },),
    put({
      path: join(
        root,
        'package',
        'a',
        'broken-crate',
        'Cargo.lock',
      ),
      content: CARGO_LOCK,
    },),
    put({
      path: join(
        root,
        'package',
        'c',
        'red-crate',
        'mise.toml',
      ),
      content: '[tasks."lint:clippy"]\nrun = "node ../../../steps/clippy.ts E0277 always"\n',
    },),
    put({
      path: join(
        root,
        'package',
        'c',
        'red-crate',
        'Cargo.lock',
      ),
      content: CARGO_LOCK,
    },),
    put({
      path: join(
        root,
        'package',
        'e',
        'ts-package',
        'mise.toml',
      ),
      content: '[tasks."lint:types"]\nrun = "node -e 0"\n',
    },),
  ],);
  /**
   Everything the first commit holds, named one by one because the repository's Git wrapper rejects bulk staging.
   */
  const committed = [
    'mise.toml',
    '.gitignore',
    'toolchain.lock.json',
    'steps',
    'package',
  ];
  // Each `git` step needs the one before it, so they run one after another and the first failure ends the layout.
  await runGitStep({
    args: [
      'init',
      '--quiet',
    ],
    root,
    environment,
  },);
  await runGitStep({
    args: [
      'add',
      '--',
      ...committed,
    ],
    root,
    environment,
  },);
  await runGitStep({
    args: [
      'commit',
      '--quiet',
      '--message',
      'fixture: first commit',
      '--',
      ...committed,
    ],
    root,
    environment,
  },);
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
