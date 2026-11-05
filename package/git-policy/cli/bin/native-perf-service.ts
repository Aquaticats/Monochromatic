/**
 The commit service and the signing key the latency benchmark in `native-perf.ts` measures the service route with.

 The services are started by the benchmark itself, one per wrapped repository as the wrapper requires (an instance named
 from the repository's Git directory, listening at the socket the wrapper derives from it), in a runtime directory of its
 own under the scratch parent, and the wrapped samples are pointed at that directory through `XDG_RUNTIME_DIR`. The
 wrapper finds a socket there and uses it as it is, because the directory holds no session bus to provision through. No
 other directory is used: the benchmark never reads the person's runtime directory, never connects to their session bus
 and never writes a unit file, so a measurement can neither provision nor reuse the service a person's commits run
 through. The signing key is a throwaway made in the scratch root and never the person's.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  open,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { once } from 'node:events';
import { connect } from 'node:net';
import { join } from 'node:path';
import { setTimeout as pause } from 'node:timers/promises';

import {
  instanceOf,
  SERVICE_MODE_WORDS,
} from '../e2e/service-launch-fixture.ts';
import {
  straceArguments,
  timed,
} from './native-perf-git.ts';
import { SERVICE_LOG_NAME } from './native-perf-evidence.ts';
import { runCommand } from './native-verification-process.ts';

/**
 How long the benchmark waits for the service to listen before it fails, in milliseconds.
 */
const START_DEADLINE_MILLISECONDS = 30_000;

/**
 Pause between two tries to connect while the service starts, in milliseconds.
 */
const START_POLL_MILLISECONDS = 10;

/**
 Directory of the service sockets under a runtime directory; each worktree's socket in it is named by its instance.
 */
const SOCKET_DIRECTORY = 'monochromatic-git-policy-cli';

/**
 The longest socket path the operating system accepts, in bytes: `sun_path` holds 108 bytes with its terminator on Linux.
 */
const SOCKET_PATH_LIMIT = 107;

/**
 The wrapper's diagnostic that succeeds only when the service of the worktree it runs in answers.
 */
const STATUS_ARGUMENTS = [
  'monochromatic-git-policy-cli',
  'service',
  'status',
] as const;

/**
 The commit services a benchmark run started, one per worktree, and the runtime directory the wrapped samples are pointed at.
 */
export type PerfService = {
  readonly runtime: string;
  readonly serve: (gitDir: string) => Promise<void>;
  readonly stop: () => Promise<void>;
};

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made
 */
async function listening(socket: string): Promise<boolean> {
  /**
   The connection attempt, which `once` fails when the socket refuses it.
   */
  const attempt = connect(socket);
  try {
    await once(
      attempt,
      'connect',
    );
    attempt.destroy();
    return true;
  }
  catch (error) {
    // A service that has not bound its socket yet refuses, or has no socket file; anything else is worth seeing.
    if (!(Error.isError(error) && ('code' in error)
      && [
        'ECONNREFUSED',
        'ENOENT'
      ].includes(String(error.code))))
      console.error(`Connecting to ${socket} failed: ${String(error)}`);
    return false;
  }
}

/**
 Wait until something listens on a socket, giving up when the deadline passes or the service ended.

 @param socket - path of the unix socket

 @param deadline - time after which the wait ends, in milliseconds since the epoch

 @param ended - whether the service process has ended

 @returns true once the socket accepts a connection, false when the wait gave up

 @example
 ```ts
 await waitUntilListening({ socket, deadline: Date.now() + 1000, ended: () => false });
 ```
 */
async function waitUntilListening({
  socket,
  deadline,
  ended,
}: {
  readonly socket: string;
  readonly deadline: number;
  readonly ended: () => boolean;
}): Promise<boolean> {
  if (await listening(socket))
    return true;
  if ((Date.now() >= deadline) || ended())
    return false;
  await pause(START_POLL_MILLISECONDS);
  return await waitUntilListening({
    socket,
    deadline,
    ended,
  });
}

/**
 Make the runtime directory of a benchmark run and the means to start the commit service of each wrapped repository in it.

 No process starts here. In production the user's service manager starts one instance per worktree on its first
 connection; the benchmark has no manager, so `serve` starts the instance of one worktree and waits until it listens at
 the socket the wrapper derives from that worktree's Git directory, as the fixtures of the binary tests do.

 @param program - path of the measured wrapper's own executable, which runs the service when it is given the service mode words

 @param root - scratch root of the run, which holds the services' journal and their `home`

 @param scratchParent - short private directory the runtime directory is made in, because a socket path holds only about a
   hundred bytes

 @param traceDirectory - directory that receives one `strace` record per service, named after its instance; absent starts the
   services untraced

 @returns the runtime directory for `XDG_RUNTIME_DIR`, a function that starts the service of one worktree, and a function
   that stops every service and removes the directory

 @example
 ```ts
 const service = await makeService({ program, root, scratchParent });
 await service.serve(join(repository, '.git'));
 ```
 */
export async function makeService({
  program,
  root,
  scratchParent,
  traceDirectory,
}: {
  readonly program: string;
  readonly root: string;
  readonly scratchParent: string;
  readonly traceDirectory?: string;
}): Promise<PerfService> {
  /**
   Private runtime directory of the run.
   */
  const runtime = await mkdtemp(join(
    scratchParent,
    'rt-',
  ));
  await mkdir(
    join(
      runtime,
      SOCKET_DIRECTORY,
    ),
    { mode: 0o700, },
  );
  /**
   The service processes this run started, one per worktree.
   */
  const children: ChildProcess[] = [];
  /**
   Stops every service and removes the runtime directory; safe to call when a service already ended.
   */
  async function stop(): Promise<void> {
    await Promise.all(children.map(async function stopChild(child) {
      if ((child.exitCode === null) && (child.signalCode === null)) {
        // A traced service is the child of `strace`, which detaches and leaves its tracee running when it is terminated
        // itself, so the tracee is told to stop and `strace` ends with it.
        if (traceDirectory === undefined) {
          child.kill('SIGTERM');
        }
        else {
          await runCommand({
            command: 'pkill',
            args: [
              '--parent',
              String(child.pid),
              '--signal',
              'TERM',
            ],
            capture: true,
            allowFailure: true,
          });
        }
        await once(
          child,
          'exit',
        );
      }
    }));
    await rm(
      runtime,
      {
        recursive: true,
        force: true,
      },
    );
  }
  /**
   Starts the service of the worktree whose own Git directory is `gitDir` and waits until it listens.

   @param gitDir - the worktree's own Git directory, `.git` of a repository the wrapped samples commit in

   @throws Error when the socket path is too long for the operating system, or the service does not listen within the deadline
   */
  async function serve(gitDir: string): Promise<void> {
    /**
     Name of the worktree's instance, which the service answers to and the wrapper derives the socket from.
     */
    const instance = await instanceOf(gitDir);
    /**
     Where the wrapper looks for this worktree's service.
     */
    const socket = join(
      runtime,
      SOCKET_DIRECTORY,
      `${instance}.sock`,
    );
    if (Buffer.byteLength(socket) > SOCKET_PATH_LIMIT) {
      throw new Error(`the socket path is longer than ${String(SOCKET_PATH_LIMIT)} bytes, which a socket cannot bind: ${socket}`);
    }
    /**
     The file the service's standard error goes to: its landing lines, which it writes plainly because it is no journal stream
     here, and which the run reads back.
     */
    const log = await open(
      join(
        root,
        SERVICE_LOG_NAME,
      ),
      'a',
    );
    /**
     The service process.
     */
    const child = spawn(
      traceDirectory === undefined ? program : 'strace',
      [
        ...(traceDirectory === undefined
          ? []
          : [
            ...straceArguments(join(
              traceDirectory,
              `service-${instance}.strace`,
            )),
            program,
          ]),
        ...SERVICE_MODE_WORDS,
        '--journal',
        join(
          root,
          'journal',
        ),
        '--path',
        socket,
        '--instance',
        instance,
      ],
      {
        stdio: [
          'ignore',
          'ignore',
          log.fd,
        ],
        env: {
          PATH: `/usr/bin:/bin:${join(
            root,
            'tools',
          )}`,
          HOME: join(
            root,
            'home',
          ),
          GIT_CONFIG_NOSYSTEM: '1',
          GIT_CONFIG_GLOBAL: '/nonexistent-global-config',
        },
      },
    );
    // The child holds its own copy of the descriptor.
    await log.close();
    children.push(child);
    if (!await waitUntilListening({
      socket,
      deadline: Date.now() + START_DEADLINE_MILLISECONDS,
      ended: function hasEnded() {
        return child.exitCode !== null;
      },
    })) {
      throw new Error(`the commit service of ${gitDir} did not listen at ${socket}`);
    }
  }
  return {
    runtime,
    serve,
    stop,
  };
}

/**
 Fail before any measurement when a wrapped command in a repository cannot reach that repository's commit service.

 The wrapper answers `git monochromatic-git-policy-cli service status` with success only when the service of the worktree it is run in answers
 at the socket it derives from the worktree's Git directory, so a service started under another name, or in another
 runtime directory, fails here once instead of failing every commit sample.

 @param root - scratch root that holds `bin` and `home`

 @param runtime - runtime directory the service was made in

 @param directory - repository whose service must answer

 @throws Error naming the repository and the wrapper's words when the service does not answer

 @example
 ```ts
 await assertServiceAnswers({ root, runtime: service.runtime, directory });
 ```
 */
export async function assertServiceAnswers({
  root,
  runtime,
  directory,
}: {
  readonly root: string;
  readonly runtime: string;
  readonly directory: string;
}): Promise<void> {
  /**
   The wrapper's answer to the status question, asked the way the commit samples ask.
   */
  const answer = await timed({
    root,
    wrapped: true,
    runtime,
    directory,
    arguments_: STATUS_ARGUMENTS,
  });
  if (answer.status !== 0) {
    /**
     The command the wrapper was asked, for the message.
     */
    const asked = `git ${STATUS_ARGUMENTS.join(' ')}`;
    /**
     The wrapper's exit status and words.
     */
    const ended = `status ${String(answer.status)}: ${answer.stderr}`;
    throw new Error(`a wrapped commit in ${directory} cannot reach its commit service; \`${asked}\` ended with ${ended}`);
  }
}

/**
 Make a throwaway SSH signing key and its allowed-signers file in the scratch root.

 @param root - scratch root of the run

 @returns paths of the private key and of the allowed-signers file, to be set in a repository with `gpg.format ssh`

 @throws Error when `ssh-keygen` fails

 @example
 ```ts
 const { key, allowedSigners } = await makeSigningKey({ root });
 ```
 */
export async function makeSigningKey({ root, }: { readonly root: string; }): Promise<{
  readonly key: string;
  readonly allowedSigners: string;
}> {
  /**
   Path of the private key.
   */
  const key = join(
    root,
    'signing-key',
  );
  await runCommand({
    command: 'ssh-keygen',
    args: [
      '-q',
      '-t',
      'ed25519',
      '-N',
      '',
      '-C',
      'perf',
      '-f',
      key,
    ],
    capture: true,
  });
  /**
   Text of the public key.
   */
  const publicKey = await readFile(
    `${key}.pub`,
    'utf8',
  );
  /**
   Path of the allowed-signers file.
   */
  const allowedSigners = join(
    root,
    'allowed-signers',
  );
  await writeFile(
    allowedSigners,
    `perf@example.invalid ${publicKey.trim()}\n`,
  );
  return {
    key,
    allowedSigners,
  };
}
