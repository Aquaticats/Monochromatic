/**
 Git processes and fixture repositories the latency benchmark in `native-perf.ts` measures against.

 Every direct sample and every wrapper child runs the real Git at `/usr/bin/git`; the wrapped run only differs in
 that its PATH lists the scratch `bin` directory, where `git` is a symlink to the release wrapper, first.
 */
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { runCommand } from './native-verification-process.ts';

/**
 The real Git every direct sample and every wrapper child runs.
 */
const REAL_GIT = '/usr/bin/git';

/**
 Number of tracked files in a fixture repository, so forwarded commands read a realistic index.
 */
const TRACKED_FILE_COUNT = 30;

/**
 Nanoseconds in a millisecond, to turn a `process.hrtime.bigint()` difference into milliseconds.
 */
const NANOSECONDS_PER_MILLISECOND = 1e6;

/**
 The environment of one sample: identical for the direct and the wrapped run apart from the PATH order.

 @param root - scratch root that holds `bin` and `home`

 @param wrapped - whether the wrapper directory leads the PATH

 @returns the complete environment, so no host variable leaks into a sample
 */
function environment({
  root,
  wrapped,
}: {
  readonly root: string;
  readonly wrapped: boolean;
}): Readonly<Record<string, string>> {
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

 @param root - scratch root that holds `bin` and `home`

 @param wrapped - whether the wrapper runs instead of the real Git

 @param directory - repository to run in

 @param arguments_ - Git arguments

 @returns wall-clock milliseconds,
   exit status (absent when a signal ended Git) and captured stderr, which the caller records when the status is not zero

 @example
 ```ts
 const sample = await timed({ root, wrapped: true, directory, arguments_: ['status'] });
 ```
 */
export async function timed({
  root,
  wrapped,
  directory,
  arguments_,
}: {
  readonly root: string;
  readonly wrapped: boolean;
  readonly directory: string;
  readonly arguments_: readonly string[];
}): Promise<{
  readonly elapsed: number;
  readonly status?: number;
  readonly stderr: string;
}> {
  /**
   Executable of the sample: the wrapper symlink for a wrapped run, the real Git otherwise.
   */
  const program = wrapped
    ? join(
      root,
      'bin',
      'git',
    )
    : REAL_GIT;
  /**
   Monotonic start time, read immediately before Git starts.
   */
  const started = process.hrtime
    .bigint();
  /**
   Finished sample, with the exit state the caller checks.
   */
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
  /**
   Wall-clock milliseconds from the start time to now.
   */
  const elapsed = Number(process.hrtime
    .bigint()
    - started) / NANOSECONDS_PER_MILLISECOND;
  return {
    elapsed,
    ...(result.status === undefined ? {} : { status: result.status, }),
    stderr: result.stderr,
  };
}

/**
 Run the real Git for setup and fail loudly.

 @param root - scratch root that holds `bin` and `home`

 @param directory - repository to run in

 @param arguments_ - Git arguments

 @throws Error when Git exits nonzero, naming the arguments, the repository and Git's stderr
 */
async function setup({
  root,
  directory,
  arguments_,
}: {
  readonly root: string;
  readonly directory: string;
  readonly arguments_: readonly string[];
}): Promise<void> {
  /**
   Finished setup command, whose exit status decides whether the fixture is usable.
   */
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

 @param root - scratch root that holds `bin` and `home`

 @param name - directory name inside the scratch root

 @returns the repository directory

 @example
 ```ts
 const directory = await repository({ root, name: 'direct' });
 ```
 */
export async function repository({
  root,
  name,
}: {
  readonly root: string;
  readonly name: string;
}): Promise<string> {
  /**
   Directory of the new repository inside the scratch root.
   */
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
