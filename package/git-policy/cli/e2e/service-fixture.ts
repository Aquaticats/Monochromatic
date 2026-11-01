/**
 The commit service of one scenario.

 Every commit the scenarios make through the wrapper lands through the resident service, so each scenario starts its own,
 as the operating system's socket activation would start one for a person, and names its runtime directory in every
 wrapper environment. A scenario owns its service: it can stall it (`SIGSTOP`) to hold prepared commits in the queue,
 kill it, and restart it the way the service manager restarts a crashed unit, without touching any other scenario.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  readdir,
  rm,
} from 'node:fs/promises';
import { connect, } from 'node:net';
import { join, } from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

//region Constants

/**
 The service program, installed beside the wrapper by the image build.
 */
const SERVICE_PROGRAM = '/opt/native/cli-git-service';

/**
 The Git the service runs, which the image keeps off the default path.
 */
const SERVICE_GIT = '/opt/git/2.55.0/bin/git';

/**
 Environment variable that arms one fault point of the service
 (`src/native/service_fault.rs`, compiled into the release executable).
 */
const FAULT_VARIABLE = 'CLI_GIT_TEST_ONLY_SERVICE_FAULT';

/**
 How long the suite waits for the service to listen, in milliseconds.
 */
const START_DEADLINE_MILLISECONDS = 30_000;

/**
 How long a stopping service gets to end after `SIGTERM`, in milliseconds.
 */
const STOP_DEADLINE_MILLISECONDS = 5_000;

/**
 Pause between two polls, in milliseconds.
 */
const POLL_MILLISECONDS = 10;

/**
 How long the observation waits for the service to finish a repository's after-commit work, in milliseconds.
 */
const SETTLE_DEADLINE_MILLISECONDS = 60_000;

/**
 Error codes that say a path holds nothing: no socket file, or a socket file nothing listens on.
 */
const ABSENT_CODES: ReadonlySet<unknown> = new Set([
  'ENOENT',
  'ECONNREFUSED',
]);

//endregion Constants

//region Types

/**
 One scenario's commit service and what a scenario may do to it.
 */
export type ScenarioService = Readonly<{
  /**
   Runtime directory the wrappers of the scenario find the service in (`XDG_RUNTIME_DIR`).
   */
  runtime: string;
  /**
   Process ID of the running service.
   */
  pid: () => number;
  /**
   Whether the service process has ended.
   */
  hasExited: () => boolean;
  /**
   Settles when the current service process ends, however it ended.
   */
  exited: () => Promise<void>;
  /**
   Everything the current service process wrote to its standard error.
   */
  log: () => string;
  /**
   Freezes the service with `SIGSTOP`.
   A client that connects meanwhile is accepted by the kernel but receives nothing, so its prepared commit waits.
   */
  stall: () => void;
  /**
   Lets a frozen service run again.
   */
  resume: () => void;
  /**
   Kills the service with `SIGKILL` and waits until it is gone.
   */
  kill: () => Promise<void>;
  /**
   Starts a clean service after the previous one ended, the way the service manager restarts a crashed unit:
   the stale socket file is removed first, because the service itself does not replace one.
   */
  restart: () => Promise<void>;
  /**
   Waits until the current service ends, then restarts it after a short outage.
   */
  restartAfterExit: () => Promise<void>;
  /**
   Restarts the service when it has ended, so the observation finds one to finish the after-commit work.
   */
  ensureRunning: () => Promise<void>;
}>;

//endregion Types

//region Connection

/**
 Whether a failure says only that the path holds nothing.

 @param error - value a connection or a directory read threw

 @returns true for a missing path or a refused connection
 */
function isAbsent(error: unknown,): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && ABSENT_CODES.has(error.code,);
}

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made

 @throws the connection's failure when it is not a refusal
 */
async function listening(socket: string,): Promise<boolean> {
  /**
   The connection attempt.
   */
  const attempt = connect(socket,);
  try {
    await once(
      attempt,
      'connect',
    );
  }
  catch (error) {
    attempt.destroy();
    if (isAbsent(error,))
      return false;
    throw error;
  }
  attempt.destroy();
  return true;
}

//endregion Connection

//region Service

/**
 Starts the commit service of one scenario and waits until it listens.

 @param root - scenario root; the service keeps its runtime directory and journal under `service/`

 @param fault - fault point the first service process is armed with,
 `after-objects`,
 `after-intent`,
 `after-ref`,
 `after-index`,
 or `before-ack`

 @returns the service, stopped when disposed

 @throws Error when the service does not listen within the deadline

 @example
 ```ts
 await using service = await startScenarioService({ root: '/work/2.55.0/baseline' });
 ```
 */
export async function startScenarioService({
  root,
  fault,
}: Readonly<{
  root: string;
  fault?: string;
}>,): Promise<ScenarioService & AsyncDisposable> {
  /**
   Runtime directory of the scenario.
   */
  const runtime = join(
    root,
    'service',
    'runtime',
  );
  /**
   Directory the socket sits in.
   */
  const socketDirectory = join(
    runtime,
    'cli-git',
  );
  /**
   Socket the service listens on.
   */
  const socket = join(
    socketDirectory,
    'service.sock',
  );
  /**
   Journal directory, kept across restarts so a restart recovers what a killed service left open.
   */
  const journal = join(
    root,
    'service',
    'journal',
  );
  /**
   Mutable holder of the current process and its log.
   */
  const current: {
    child: ChildProcess | undefined;
    log: string;
    ended: Promise<void>;
  } = {
    child: undefined,
    log: '',
    ended: Promise.resolve(),
  };
  /**
   Starts one service process and waits until it listens.

   @param armed - fault point to arm, none for a clean service
   */
  async function launch(armed: string | undefined,): Promise<void> {
    await mkdir(
      socketDirectory,
      { recursive: true, },
    );
    await rm(
      socket,
      { force: true, },
    );
    /**
     The new service process.
     */
    const child = spawn(
      SERVICE_PROGRAM,
      [
        '--journal',
        journal,
        '--git',
        SERVICE_GIT,
        '--path',
        socket,
      ],
      {
        stdio: [
          'ignore',
          'ignore',
          'pipe',
        ],
        env: armed === undefined ? process.env : {
          ...process.env,
          [FAULT_VARIABLE]: armed,
        },
      },
    );
    current.child = child;
    current.log = '';
    child.stderr
      .setEncoding('utf8',);
    child.stderr
      .on(
        'data',
        function remember(text: string,) {
          current.log += text;
        },
      );
    current.ended = once(
      child,
      'exit',
    )
      .then(function ignoreExit() {
        return undefined;
      },);
    /**
     When the wait for the service ends.
     */
    const deadline = Date.now() + START_DEADLINE_MILLISECONDS;
    for await (const _tick of ticks(POLL_MILLISECONDS,)) {
      if (await listening(socket,))
        return;
      if ((Date.now() >= deadline) || hasExited()) {
        throw new Error(`the commit service did not listen at ${socket}: ${current.log}`,);
      }
    }
  }
  /**
   Whether the current process has ended.

   @returns true when it exited or was killed by a signal
   */
  function hasExited(): boolean {
    return (current.child === undefined) || (current.child.exitCode !== null) || (current.child.signalCode !== null);
  }
  /**
   Process ID of the current process.

   @returns the process ID

   @throws Error when no service was started
   */
  function pid(): number {
    if (current.child?.pid === undefined)
      throw new Error('the scenario has no running commit service',);
    return current.child.pid;
  }
  /**
   Ends the current process: `SIGCONT` so a frozen one can hear `SIGTERM`, then `SIGKILL` after the deadline.
   */
  async function stop(): Promise<void> {
    if (hasExited())
      return;
    /**
     Process being stopped.
     */
    const child = current.child;
    child?.kill('SIGCONT',);
    child?.kill('SIGTERM',);
    /**
     Forced end after the deadline.
     */
    const forced = setTimeout(
      function force() {
        child?.kill('SIGKILL',);
      },
      STOP_DEADLINE_MILLISECONDS,
    );
    await current.ended;
    clearTimeout(forced,);
  }
  await launch(fault,);
  return {
    runtime,
    pid,
    hasExited,
    exited() {
      return current.ended;
    },
    log() {
      return current.log;
    },
    stall() {
      process.kill(
        pid(),
        'SIGSTOP',
      );
    },
    resume() {
      process.kill(
        pid(),
        'SIGCONT',
      );
    },
    async kill() {
      if (hasExited())
        return;
      current.child?.kill('SIGKILL',);
      await current.ended;
    },
    async restart() {
      await stop();
      await launch(undefined,);
    },
    async restartAfterExit() {
      await current.ended;
      await launch(undefined,);
    },
    async ensureRunning() {
      if (hasExited())
        await launch(undefined,);
    },
    [Symbol.asyncDispose]: stop,
  };
}

//endregion Service

//region Settlement

/**
 Names in a directory, none when the directory does not exist.

 @param directory - directory to list

 @returns the entry names

 @throws the read's failure when it is not a missing directory
 */
async function namesIn(directory: string,): Promise<readonly string[]> {
  try {
    return await readdir(directory,);
  }
  catch (error) {
    // A repository the service never saw has no work directory; any other failure is real.
    if (isAbsent(error,))
      return [];
    throw error;
  }
}

/**
 Waits until the service has finished the after-commit work of one repository.

 A commit the service lands returns before its push: the wrapper writes a push record and a maintenance record, and the
 service runs them on its own thread. The remote therefore reaches the landed commit some time after the exit, so the
 observation that compares the remote with the local branch waits here first. A record that failed counts as finished,
 so the remote check reports the missing push instead of this wait hiding it.

 @param commonDir - the repository's common Git directory

 @throws Error when records are still pending or claimed after the deadline

 @example
 ```ts
 await settleServiceWork(repository.commonDir);
 ```
 */
export async function settleServiceWork(commonDir: string,): Promise<void> {
  /**
   Directory of the repository's after-commit records.
   */
  const workDirectory = join(
    commonDir,
    'cli-git',
    'work',
  );
  /**
   When the wait for the records ends.
   */
  const deadline = Date.now() + SETTLE_DEADLINE_MILLISECONDS;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    /**
     Records waiting for the service and records it is working on.
     */
    const [
      pending,
      claimed,
    ] = await Promise.all([
      namesIn(join(
        workDirectory,
        'pending',
      ),),
      namesIn(join(
        workDirectory,
        'claimed',
      ),),
    ],);
    if ((pending.length + claimed.length) === 0)
      return;
    if (Date.now() >= deadline)
      throw new Error(`the service left ${pending.length} pending and ${claimed.length} claimed records in ${workDirectory}: ${[...pending, ...claimed,].join(', ',)}`,);
  }
}

//endregion Settlement
