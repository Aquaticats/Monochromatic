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
 The `strace` arguments that record a command and every process it starts, ending at the command.

 The record carries a precise absolute time on every call, the time each call took, and the path behind every file
 descriptor, which is what a reader sums per call and per path.

 @param output - file that receives the record

 @returns the arguments to put before the traced command

 @example
 ```ts
 spawn('strace', [...straceArguments('commit.strace'), 'git', 'status']);
 ```
 */
export function straceArguments(output: string): readonly string[] {
  return [
    // A narrow set (for example `execve,fsync,rename`) keeps `strace` from slowing the traced commands much, so the times of
    // the calls it does record stay close to untraced times; the full record is for counts and per-call sums only.
    ...(process.env
      .NATIVE_PERF_TRACE_CALLS
      === undefined ? [] : [
        `--trace=${process.env
          .NATIVE_PERF_TRACE_CALLS}`,
        '--seccomp-bpf'
      ]),
    '--follow-forks',
    '--absolute-timestamps=unix,precision:us',
    '--syscall-times',
    '--decode-fds=path',
    '--string-limit=160',
    '--output',
    output,
    '--',
  ];
}

/**
 The environment of one sample: identical for the direct and the wrapped run apart from the PATH order.

 @param root - scratch root that holds `bin` and `home`

 @param wrapped - whether the wrapper directory leads the PATH

 @param runtime - runtime directory of the commit service the wrapped run is pointed at; a commit needs it, and without it the
   wrapper finds no service, so only forwarded commands run that way, never the person's own runtime directory

 @returns the complete environment, so no host variable leaks into a sample

 @example
 ```ts
 const env = environment({ root, wrapped: true });
 ```
 */
export function environment({
  root,
  wrapped,
  runtime,
}: {
  readonly root: string;
  readonly wrapped: boolean;
  readonly runtime?: string;
}): Readonly<Record<string, string>> {
  return {
    ...(wrapped && (runtime !== undefined) ? { XDG_RUNTIME_DIR: runtime, } : {}),
    PATH: wrapped
      ? `${join(
        root,
        'bin',
      )}:/usr/bin:/bin:${join(
        root,
        'tools',
      )}`
      : `/usr/bin:/bin:${join(
        root,
        'tools',
      )}`,
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

 @param runtime - runtime directory of the commit service for a wrapped run

 @param directory - repository to run in

 @param arguments_ - Git arguments

 @param trace - file that receives the `strace` record of the run and of every process it starts; absent runs untraced, and a
   traced run is slower, so only its counts and per-call sums are evidence, never its elapsed time

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
  runtime,
  directory,
  arguments_,
  trace,
}: {
  readonly root: string;
  readonly wrapped: boolean;
  readonly runtime?: string;
  readonly directory: string;
  readonly arguments_: readonly string[];
  readonly trace?: string;
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
    ...(trace === undefined
      ? {
        command: program,
        args: arguments_,
      }
      : {
        command: 'strace',
        args: [
          ...straceArguments(trace),
          program,
          ...arguments_,
        ],
      }),
    capture: true,
    allowFailure: true,
    cwd: directory,
    env: environment({
      root,
      wrapped,
      ...(runtime === undefined ? {} : { runtime, }),
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

 @example
 ```ts
 await setup({ root, directory, arguments_: ['status'] });
 ```
 */
export async function setup({
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

/**
 Make every commit of a fixture repository SSH-signed with a throwaway key, as the person's commits are.

 @param root - scratch root that holds `bin` and `home`

 @param directory - repository to configure

 @param key - path of the private key

 @param allowedSigners - path of the allowed-signers file

 @example
 ```ts
 await enableSshSigning({ root, directory, key, allowedSigners });
 ```
 */
export async function enableSshSigning({
  root,
  directory,
  key,
  allowedSigners,
}: {
  readonly root: string;
  readonly directory: string;
  readonly key: string;
  readonly allowedSigners: string;
}): Promise<void> {
  /**
   Git settings that turn on SSH signing, in the order they are applied.
   */
  const settings = [
    [
      'gpg.format',
      'ssh',
    ],
    [
      'user.signingkey',
      key,
    ],
    [
      'gpg.ssh.allowedsignersfile',
      allowedSigners,
    ],
    [
      'commit.gpgsign',
      'true',
    ],
  ] as const;
  // Array.fromAsync awaits each mapped call before it starts the next, which Git needs: two `git config` runs in one
  // repository contend for `config.lock`.
  await Array.fromAsync(
    settings,
    async function configure([
      name,
      value,
    ],) {
      await setup({
        root,
        directory,
        arguments_: [
          'config',
          name,
          value,
        ],
      });
    },
  );
}
