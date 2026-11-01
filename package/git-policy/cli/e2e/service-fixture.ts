/**
 The commit service of the end-to-end suite.

 Every commit the scenarios make through the wrapper lands through the resident service, so the suite starts one before the
 first scenario, as the operating system's socket activation would start it for a person, and every scenario's wrapper
 environment names its runtime directory. One service serves all scenarios, as one service serves all of a person's
 repositories; a scenario that kills a wrapper never kills it.

 @module
 */

import { spawn, } from 'node:child_process';
import { mkdir, } from 'node:fs/promises';
import { once, } from 'node:events';
import { connect, } from 'node:net';
import { join, } from 'node:path';
import { setTimeout as pause, } from 'node:timers/promises';

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
 Pause between two tries to connect, in milliseconds.
 */
const START_POLL_MILLISECONDS = 10;

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made
 */
async function listening(socket: string,): Promise<boolean> {
  return new Promise(function bridge(resolve,) {
    const attempt = connect(socket,);
    attempt.once('connect', function connected() {
      attempt.destroy();
      resolve(true,);
    },);
    attempt.once('error', function refused() {
      resolve(false,);
    },);
  },);
}

/**
 Starts the suite's commit service and waits until it listens.

 @returns a function that stops the service

 @throws Error when the service does not listen within the deadline

 @example
 ```ts
 const stop = await startSuiteService();
 ```
 */
export async function startSuiteService(): Promise<() => Promise<void>> {
  const socket = join(SERVICE_RUNTIME, 'cli-git', 'service.sock',);
  await mkdir(join(SERVICE_RUNTIME, 'cli-git',), { recursive: true, },);
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
    if (child.exitCode === null && child.signalCode === null) {
      child.kill('SIGTERM',);
      await once(child, 'exit',);
    }
  }
  const deadline = Date.now() + START_DEADLINE_MILLISECONDS;
  while (!await listening(socket,)) {
    if (Date.now() >= deadline || child.exitCode !== null) {
      await stop();
      throw new Error(`the commit service did not listen at ${socket}`,);
    }
    await pause(START_POLL_MILLISECONDS,);
  }
  return stop;
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
  const stop = await startSuiteService();
  try {
    return await work();
  }
  finally {
    await stop();
  }
}
