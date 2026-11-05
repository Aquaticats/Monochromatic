/**
 The commit service of one scenario.

 Every commit the scenarios make through the wrapper lands through the resident service, so each scenario starts its own,
 as the operating system's socket activation would start one for a person, and names its runtime directory in every
 wrapper environment. A scenario owns its service: it can stall it (`SIGSTOP`) to hold prepared commits in the queue,
 kill it, and restart it the way the service manager restarts a crashed unit, without touching any other scenario.

 @module
 */

import type { ChildProcess, } from 'node:child_process';
import { once, } from 'node:events';
import { mkdir, } from 'node:fs/promises';
import { createHash, } from 'node:crypto';
import {
  dirname,
  join,
} from 'node:path';
import { setInterval as ticks, } from 'node:timers/promises';

import { listening, } from './service-connection-fixture.ts';
import {
  instanceOf,
  spawnService,
} from './service-launch-fixture.ts';
import type { ScenarioService, } from './service-model-fixture.ts';

//region Constants

/**
 Hexadecimal digits of the digest that names a scenario's runtime directory, enough that two scenarios never share one.
 */
const RUNTIME_NAME_HEXADECIMAL_DIGITS = 8;

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

//endregion Constants

//region Service

/**
 The runtime directory of a scenario's service, beside the scenario roots and named by a digest of the root.
 Its name is short on purpose: a socket path holds about a hundred bytes, and the runtime directory, the socket directory
 (the program's name) and the 32-digit instance name must all fit in it, which the longer name of a scenario root and the
 program's own name no longer allow when the runtime directory sits inside the root.

 @param root - scenario root

 @returns the directory the service's `monochromatic-git-policy-cli/<instance>.sock` sits under

 @example
 ```ts
 scenarioRuntimeDirectory('/work/2.55.0/baseline');
 ```
 */
export function scenarioRuntimeDirectory(root: string,): string {
  /**
   Digest of the scenario root, which names its runtime directory.
   */
  const hash = createHash('sha256',);
  hash.update(root,);
  /**
   Hexadecimal text of the digest.
   */
  const digest = hash.digest('hex',);
  return join(
    dirname(root,),
    'r',
    digest.slice(
      0,
      RUNTIME_NAME_HEXADECIMAL_DIGITS,
    ),
  );
}

/**
 Starts the commit service of one scenario and waits until it listens.

 @param root - scenario root; the service keeps its runtime directory and journal under `service/`.
 The service serves one worktree, as in production, and starts when `bind` names it.

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
export function startScenarioService({
  root,
  fault,
}: Readonly<{
  root: string;
  fault?: string;
}>,): Promise<ScenarioService & AsyncDisposable> {
  /**
   Runtime directory of the scenario.
   */
  const runtime = scenarioRuntimeDirectory(root,);
  /**
   Directory the socket sits in.
   */
  const socketDirectory = join(
    runtime,
    'monochromatic-git-policy-cli',
  );
  /**
   The worktree the service serves, its instance name, and the socket it listens on, known once `bind` names the worktree.
   */
  const served: {
    instance: string;
    socket: string;
    fault?: string;
  } = {
    instance: '',
    socket: '',
  };
  /**
   Journal directory, kept across restarts so a restart recovers what a killed service left open.
   */
  const journal = join(
    root,
    'service',
    'journal',
  );
  /**
   Mutable state of the scenario's service processes: the current process, the standard error of every process so far,
   when the current process ends, and whether the scenario ended, after which nothing may start a service.
   */
  const current: {
    child?: ChildProcess;
    log: string;
    ended: Promise<void>;
    disposed: boolean;
  } = {
    log: '',
    ended: Promise.resolve(),
    disposed: false,
  };
  /**
   Whether the current process has ended.

   @returns true when it exited or was killed by a signal, or none was started
   */
  function hasExited(): boolean {
    /**
     The current process.
     */
    const running = current.child;
    if (running === undefined)
      return true;
    /**
     How the process ended, if it did.
     */
    const {
      exitCode,
      signalCode,
    } = running;
    return (exitCode !== null) || (signalCode !== null);
  }
  /**
   Process ID of the current process.

   @returns the process ID

   @throws Error when no service was started
   */
  function pid(): number {
    /**
     The current process.
     */
    const running = current.child;
    if (running === undefined)
      throw new Error('the scenario has no running commit service',);
    /**
     The process's ID, absent when it did not start.
     */
    const { pid: processId, } = running;
    if (processId === undefined)
      throw new Error('the commit service process has no process ID',);
    return processId;
  }
  /**
   Starts one service process and waits until it listens.

   @param armed - fault point to arm, none for a clean service

   @throws Error when the service does not listen within the deadline
   */
  async function launch(armed?: string,): Promise<void> {
    await mkdir(
      socketDirectory,
      { recursive: true, },
    );
    /**
     Socket the service listens on. A restart leaves the dead service's socket file where it was: the service takes it over.
     */
    const { socket, } = served;
    /**
     The new service process.
     */
    const started = spawnService({
      socket,
      journal,
      instance: served.instance,
      ...armed === undefined ? {} : { fault: armed, },
    },);
    current.child = started;
    current.log += `--- service process ${String(started.pid,)} ---\n`;
    started.stderr
      ?.setEncoding('utf8',);
    started.stderr
      ?.on(
      'data',
      function remember(text: string,) {
        current.log += text;
      },
      );
    current.ended = (async function waitForExit(): Promise<void> {
      await once(
        started,
        'exit',
      );
    })();
    /**
     When the wait for the service ends.
     */
    const deadline = Date.now() + START_DEADLINE_MILLISECONDS;
    for await (const _tick of ticks(POLL_MILLISECONDS,)) {
      if (await listening(socket,))
        return;
      if ((Date.now() >= deadline) || hasExited())
        throw new Error(`the commit service did not listen at ${socket}: ${current.log}`,);
    }
  }
  /**
   Ends the current process: `SIGCONT` so a frozen one can hear `SIGTERM`, then `SIGKILL` after the deadline.
   */
  async function stop(): Promise<void> {
    if (hasExited())
      return;
    /**
     The process being stopped.
     */
    const stopping = current.child;
    stopping?.kill('SIGCONT',);
    stopping?.kill('SIGTERM',);
    /**
     Forced end after the deadline.
     */
    const forced = setTimeout(
      function force() {
        stopping?.kill('SIGKILL',);
      },
      STOP_DEADLINE_MILLISECONDS,
    );
    await current.ended;
    clearTimeout(forced,);
  }
  return Promise.resolve({
    runtime,
    async bind({ gitDir, }: Readonly<{ gitDir: string; }>,) {
      served.instance = await instanceOf(gitDir,);
      served.socket = join(
        socketDirectory,
        `${served.instance}.sock`,
      );
      await launch(fault,);
    },
    pid,
    hasExited,
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
      /**
       The process being killed.
       */
      const killed = current.child;
      killed?.kill('SIGKILL',);
      await current.ended;
    },
    async restart(armed,) {
      await stop();
      await launch(armed,);
    },
    async restartAfterExit() {
      await current.ended;
      // A scenario that ends while it waits must not leave a service running behind it.
      if (!current.disposed)
        await launch();
    },
    async ensureRunning() {
      if (hasExited())
        await launch();
    },
    async [Symbol.asyncDispose]() {
      current.disposed = true;
      await stop();
    },
  });
}

//endregion Service
