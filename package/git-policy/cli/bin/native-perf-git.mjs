/**
 Git processes and fixture repositories the latency benchmark in `native-perf.mjs` measures against.

 Every direct sample and every wrapper child runs the real Git at `/usr/bin/git`; the wrapped run only differs in
 that its PATH lists the scratch `bin` directory, where `git` is a symlink to the release wrapper, first.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { runCommand } from './native-verification-process.mjs';

/** The real Git every direct sample and every wrapper child runs. */
const REAL_GIT = '/usr/bin/git';

/** Number of tracked files in a fixture repository, so forwarded commands read a realistic index. */
const TRACKED_FILE_COUNT = 30;

/** Nanoseconds in a millisecond, to turn a `process.hrtime.bigint()` difference into milliseconds. */
const NANOSECONDS_PER_MILLISECOND = 1e6;

/**
 The environment of one sample: identical for the direct and the wrapped run apart from the PATH order.

 @param {{ root: string, wrapped: boolean }} request - the scratch root that holds `bin` and `home`,
   and whether the wrapper directory leads the PATH
 @returns {Record<string, string>} the complete environment, so no host variable leaks into a sample
 */
function environment({
  root,
  wrapped,
}) {
  return {
    PATH: wrapped
      ? `${join(
        root,
        'bin',
      )}:/usr/bin:/bin`
      : '/usr/bin:/bin',
    HOME: join(
      root,
      'home',
    ),
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/nonexistent-global-config',
    GIT_AUTHOR_NAME: 'Perf',
    GIT_AUTHOR_EMAIL: 'perf@example.invalid',
    GIT_COMMITTER_NAME: 'Perf',
    GIT_COMMITTER_EMAIL: 'perf@example.invalid',
  };
}

/**
 Run Git once and return the elapsed milliseconds and the exit status.

 @param {{ root: string, wrapped: boolean, directory: string, arguments_: readonly string[] }} request -
   scratch root, whether the wrapper runs, the repository to run in, and the Git arguments
 @returns {Promise<{ elapsed: number, status: number | null, stderr: string }>} wall-clock milliseconds,
   exit status and captured stderr, which the caller records when the status is not zero
 */
export async function timed({
  root,
  wrapped,
  directory,
  arguments_,
}) {
  const program = wrapped
    ? join(
      root,
      'bin',
      'git',
    )
    : REAL_GIT;
  const started = process.hrtime
    .bigint();
  const result = await runCommand({
    command: program,
    args: arguments_,
    capture: true,
    allowFailure: true,
    cwd: directory,
    env: environment({
      root,
      wrapped,
    }),
  });
  const elapsed = Number(process.hrtime
    .bigint()
    - started) / NANOSECONDS_PER_MILLISECOND;
  return {
    elapsed,
    status: result.status,
    stderr: result.stderr,
  };
}

/**
 Run the real Git for setup and fail loudly.

 @param {{ root: string, directory: string, arguments_: readonly string[] }} request -
   scratch root, the repository to run in, and the Git arguments
 @returns {Promise<void>} resolves once Git exited zero
 @throws {Error} when Git exits nonzero, naming the arguments, the repository and Git's stderr
 */
async function setup({
  root,
  directory,
  arguments_,
}) {
  const result = await runCommand({
    command: REAL_GIT,
    args: arguments_,
    capture: true,
    allowFailure: true,
    cwd: directory,
    env: environment({
      root,
      wrapped: false,
    }),
  });
  if (result.status !== 0) {
    throw new Error(`git ${arguments_.join(' ')} failed in ${directory}: ${result.stderr}`);
  }
}

/**
 One repository with thirty tracked files, so the forwarded commands read a realistic index.

 @param {{ root: string, name: string }} request - scratch root and the directory name inside it
 @returns {Promise<string>} the repository directory
 */
export async function repository({
  root,
  name,
}) {
  const directory = join(
    root,
    name,
  );
  await mkdir(directory);
  await setup({
    root,
    directory,
    arguments_: [
      'init',
      '--quiet',
      '--initial-branch=main',
    ],
  });
  await Promise.all(Array.from(
    { length: TRACKED_FILE_COUNT },
    function trackedFile(
      _unused,
      index,
    ) {
      return writeFile(
        join(
          directory,
          `file-${String(index)}.txt`,
        ),
        `content ${String(index)}\n`,
      );
    },
  ));
  await setup({
    root,
    directory,
    arguments_: [
      'add',
      '--',
      '.',
    ],
  });
  await setup({
    root,
    directory,
    arguments_: [
      'commit',
      '--quiet',
      '--message=base',
    ],
  });
  return directory;
}
