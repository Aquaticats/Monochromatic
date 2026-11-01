/**
 The commit service of the end-to-end suite.

 Every commit the scenarios make through the wrapper lands through the resident service, so the suite starts one before the
 first scenario, as the operating system's socket activation would start it for a person, and every scenario's wrapper
 environment names its runtime directory. One service serves all scenarios, as one service serves all of a person's
 repositories; a scenario that kills a wrapper never kills it.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { connect, } from 'node:net';
import { join, } from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

/**
 Runtime directory the wrappers of the suite find the service in (`XDG_RUNTIME_DIR`).
 */
export const SERVICE_RUNTIME = '/work/runtime';

/**
 The service program, installed beside the wrapper by the image build.
 */
const SERVICE_PROGRAM = '/opt/native/cli-git-service';

/**
 The Git the service runs, which the image keeps off the default path.
 */
const SERVICE_GIT = '/opt/git/2.55.0/bin/git';

/**
 Where the service keeps its journal.
 */
const SERVICE_JOURNAL = '/work/service-journal';

/**
 How long the suite waits for the service to listen, in milliseconds.
 */
const START_DEADLINE_MILLISECONDS = 30_000;

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

/**
 Starts the suite's commit service and waits until it listens.

 @returns a handle that stops the service when disposed

 @throws Error when the service does not listen within the deadline

 @example
 ```ts
 await using service = await startSuiteService();
 ```
 */
export async function startSuiteService(): Promise<AsyncDisposable> {
  /**
   Socket the service listens on.
   */
  const socket = join(
    SERVICE_RUNTIME,
    'cli-git',
    'service.sock',
  );
  await mkdir(
    join(
      SERVICE_RUNTIME,
      'cli-git',
    ),
    { recursive: true, },
  );
  /**
   The service process, whose output goes to the suite's.
   */
  const child = spawn(
    SERVICE_PROGRAM,
    [
      '--journal',
      SERVICE_JOURNAL,
      '--git',
      SERVICE_GIT,
      '--path',
      socket,
    ],
    { stdio: 'inherit', },
  );
  /**
   Stops the service; safe to call when it already ended.
   */
  async function stop(): Promise<void> {
    if ((child.exitCode === null) && (child.signalCode === null)) {
      child.kill('SIGTERM',);
      await once(
        child,
        'exit',
      );
    }
  }
  /**
   When the wait for the service ends.
   */
  const deadline = Date.now() + START_DEADLINE_MILLISECONDS;
  for await (const _tick of ticks(POLL_MILLISECONDS,)) {
    if (await listening(socket,))
      break;
    if ((Date.now() >= deadline) || (child.exitCode !== null)) {
      await stop();
      throw new Error(`the commit service did not listen at ${socket}`,);
    }
  }
  return { [Symbol.asyncDispose]: stop, };
}

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
      throw new Error(`the service left ${pending.length} pending and ${claimed.length} claimed records in ${workDirectory}`,);
  }
}

/**
 Runs work while the suite's commit service runs, and stops the service afterwards however the work ends.

 @param work - the scenarios

 @returns what the work returned

 @example
 ```ts
 const results = await whileServing(async () => runAll());
 ```
 */
export async function whileServing<Result,>(work: () => Promise<Result>,): Promise<Result> {
  /**
   The running service, stopped when this function ends.
   */
  await using _service = await startSuiteService();
  return await work();
}
