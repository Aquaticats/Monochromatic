import {
  type ChildProcess,
  spawn as cpSpawn,
} from 'node:child_process';
import { setTimeout, } from 'node:timers/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';

import {
  log,
  reportExpectedFailure,
} from '../infra/syslog.ts';

/**
 Module logger: the journal identity this package files every record under, then this module's
 subsystem tag.
 */
const l = tagged({
  l: log,
  tag: 'llama',
},);

//region Server configuration
// The weights, the binary, the port, and the health-poll bounds one server run uses.

// LFM2.5-VL-1.6B: smaller/faster than Qwen3-VL-2B while still vision-capable
/**
 Path to the quantized LFM2.5-VL model weights.
 */
const MODEL = '/var/home/user/models/lfm25-vl-1.6b/LFM2.5-VL-1.6B-Q4_0.gguf';

/**
 Path to the multimodal projection weights for LFM2.5-VL.
 */
const MMPROJ = '/var/home/user/models/lfm25-vl-1.6b/mmproj-LFM2.5-VL-1.6b-Q8_0.gguf';

/**
 Path to the llama-server binary inside the distrobox container.
 */
const LLAMA_SERVER = '/var/home/user/llama-cpp-build/build/bin/llama-server';

/**
 Port for the local llama-server HTTP API.
 */
const PORT = 8_787;

/**
 Health endpoint URL for readiness polling.
 */
const HEALTH_URL = `http://127.0.0.1:${PORT}/health`;

/**
 Maximum time to wait for llama-server to become healthy.
 */
const HEALTH_TIMEOUT_MS = 30_000;

/**
 Interval between health check polls.
 */
const HEALTH_POLL_MS = 500;

/**
 OpenAI-compatible chat completions endpoint served by llama-server.
 */
export const API_URL: string = `http://127.0.0.1:${PORT}/v1/chat/completions`;

//endregion Server configuration

//region Child lifecycle
// The stored handle, and the end-of-child bookkeeping every wait depends on.

/**
 Module-singleton mutable state for the running llama-server subprocess handle; wrapped so it satisfies no-module-root-let.
 */
const state: { server?: ChildProcess; } = {};

/**
 Reports whether a child has already ended.

 Node emits `exit` exactly once, so a listener attached after that event never
 runs. Every wait for a child therefore checks the recorded exit state first.

 @param child - child to inspect

 @returns whether the child has exited or been killed

 @example
 ```ts
 if (childHasEnded({ child })) return;
 ```
 */
function childHasEnded({ child, }: Readonly<{ child: ChildProcess; }>,): boolean {
  return (child.exitCode !== null) || (child.signalCode !== null);
}

/**
 Forgets the stored server handle when it is still the one this child owns.

 A handle whose process ended must not be mistaken for a live server by
 {@link start}, and a newer handle must not be discarded by an older child's
 late exit.

 @param child - child whose handle is being forgotten
 */
function forgetServer({ child, }: Readonly<{ child: ChildProcess; }>,): void {
  if (state.server === child)
    delete state.server;
}

/**
 Watches one child for the two ways it can end, so the stored handle is cleared
 and the outcome is filed whether or not any later call is watching.

 The listeners are attached at spawn time, which is the only moment they are
 guaranteed to precede the event: `stop` waits for the process it just killed,
 and a server that crashed on its own would otherwise leave a stale handle that
 makes {@link start} no-op and every later {@link analyze} fail.

 @param child - child to watch

 @example
 ```ts
 watchChildEnd({ child: state.server });
 ```
 */
function watchChildEnd({ child, }: Readonly<{ child: ChildProcess; }>,): void {
  /**
   Lifecycle logger, tagged with the watching function's name.
   */
  const fnLog = tagged({
    l,
    tag: watchChildEnd.name,
  },);

  child.on(
    'exit',
    function forgetExitedServer(): void {
      fnLog.debug(
        `llama-server exited (code ${String(child.exitCode,)}, signal ${String(child.signalCode,)})`,
      );
      forgetServer({ child, },);
    },
  );
  child.on(
    'error',
    function reportSpawnFailure(error: Error,): void {
      fnLog.error(`llama-server failed to start: ${caughtValueText(error,)}`,);
      forgetServer({ child, },);
    },
  );
}

/**
 Resolves once a child has ended, whether it exited, was killed, or never
 started.

 @param child - child to wait for

 @example
 ```ts
 currentServer.kill();
 await awaitChildEnd({ child: currentServer });
 ```
 */
export async function awaitChildEnd({ child, }: Readonly<{ child: ChildProcess; }>,): Promise<void> {
  if (childHasEnded({ child },))
    return;

  // oxlint-disable-next-line promise/avoid-new -- callback bridge over the child process event API
  await new Promise<void>(
    /**
     Registers promise settlement with the child's end events.

     @param resolve - Promise settlement callback.

     @mutates resolve through child listener retention and invocation
     */
    function waitForEnd(resolve,): void {
      child.once(
        'exit',
        function onExit(): void {
          resolve();
        },
      );
      child.once(
        'error',
        function onError(): void {
          resolve();
        },
      );
      // A child that ended between the check above and these listeners emits nothing more, so the
      // recorded state is the only signal left.
      if (childHasEnded({ child },))
        resolve();
    },
  );
}

//endregion Child lifecycle

//region Server run
// Starting, stopping, and force-cleaning one llama-server, and polling its health endpoint.

/**
 Starts llama-server inside a distrobox container with AMD GPU overrides.
 No-ops if the server is already running. Blocks via {@link waitForHealth}
 until the health endpoint reports ready or the timeout expires.
 
 @throws when llama-server fails to become healthy within {@link HEALTH_TIMEOUT_MS}
 
 @example
 ```ts
 await start();
 // server is ready for inference
 ```
 */
export async function start(): Promise<void> {
  /**
   Start logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: start.name,
  },);

  /**
   Stored handle, which may belong to a server that already ended.
   */
  const existingServer = state.server;
  if (existingServer !== undefined) {
    if (!childHasEnded({ child: existingServer },))
      return;
    fnLog.debug('Discarding a llama-server handle whose process already exited.',);
    forgetServer({ child: existingServer, },);
  }

  fnLog.debug('Starting llama-server via distrobox...',);
  /**
   Fresh server child, watched from the moment it exists so its handle cannot
   outlive it.
   */
  const server = cpSpawn(
    'distrobox',
    [
      'enter',
      'llama-build',
      '--',
      'env',
      'HSA_OVERRIDE_GFX_VERSION=11.0.2',
      LLAMA_SERVER,
      '-m',
      MODEL,
      '--mmproj',
      MMPROJ,
      '-ngl',
      '99',
      '-c',
      '8192',
      '-b',
      '4096',
      '-ub',
      '4096',
      '--port',
      String(PORT,),
    ],
    { stdio: [
      'ignore',
      'ignore',
      'ignore',
    ], },
  );
  state.server = server;
  watchChildEnd({ child: server, },);

  await waitForHealth();
  fnLog.debug('Server ready.',);
}

/**
 Gracefully stops llama-server by killing the wrapped process by name
 (distrobox prevents direct PID signalling), then awaits subprocess exit.
 
 @example
 ```ts
 await stop();
 // VRAM is freed
 ```
 */
export async function stop(): Promise<void> {
  if (!state.server)
    return;
  /**
   Stop logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: stop.name,
  },);
  fnLog.debug('Stopping llama-server...',);

  // Kill the actual llama-server process by name since distrobox wraps it
  try {
    await spawn(
      'pkill',
      [
        '-f',
        `llama-server.*--port ${PORT}`,
      ],
    );
  }
  catch (error: unknown) {
    reportExpectedFailure({
      error,
      l: fnLog,
      operation: 'pkill of the running llama-server',
    },);
  }

  /**
   Milliseconds to wait after server exit for the port to be freed.
   */
  const PORT_FREE_DELAY_MS = 500;

  /**
   Pinned reference to the current server handle, so the wait still refers to this child after
   `state.server` is cleared.
   */
  const currentServer = state.server;
  currentServer.kill();
  // The child may have ended while `pkill` ran, and Node emits `exit` once, so this wait settles
  // from the recorded exit state rather than from an event that already fired.
  await awaitChildEnd({ child: currentServer, },);
  forgetServer({ child: currentServer, },);

  // Wait briefly for port to free up
  await setTimeout(PORT_FREE_DELAY_MS,);
  fnLog.debug('Server stopped.',);
}

/**
 Forcefully kills any llama-server process matching the configured port.
 Used during shutdown to ensure no orphaned GPU processes remain.
 
 @example
 ```ts
 process.on("SIGTERM", () => forceCleanup());
 ```
 */
export async function forceCleanup(): Promise<void> {
  /**
   Forced-cleanup logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: forceCleanup.name,
  },);

  try {
    await spawn(
      'pkill',
      [
        '-9',
        '-f',
        `llama-server.*--port ${PORT}`,
      ],
    );
  }
  catch (error: unknown) {
    reportExpectedFailure({
      error,
      l: fnLog,
      operation: 'pkill -9 of any llama-server left running',
    },);
  }
  /**
   Handle of the server the forced cleanup just ended, if any.
   */
  const forcedServer = state.server;
  if (forcedServer !== undefined)
    forgetServer({ child: forcedServer, },);
}

/**
 Maximum number of health polls before giving up.
 */
const MAX_HEALTH_POLLS = Math.ceil(HEALTH_TIMEOUT_MS / HEALTH_POLL_MS,);

/**
 Polls the llama-server health endpoint until it reports ready.
 
 @throws when the server does not become healthy within {@link MAX_HEALTH_POLLS} attempts
 */
async function waitForHealth(): Promise<void> {
  /**
   Health-poll logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: waitForHealth.name,
  },);

  for (let attempt = 0; attempt < MAX_HEALTH_POLLS; attempt++) {
    try {
      /**
       Health-endpoint response; non-OK statuses keep the poll loop waiting.
       */
      // oxlint-disable-next-line no-await-in-loop -- sequential health polling by design
      const res = await fetch(HEALTH_URL,);
      if (res.ok) {
        /**
         Parsed health payload; `status === 'ok'` ends the poll loop.
         */
        // oxlint-disable-next-line no-await-in-loop, typescript/no-unsafe-type-assertion -- sequential poll; JSON response shape is known
        const body = (await res.json()) as { status: string; };
        if (body.status
          === 'ok')
          return;
      }
    }
    catch (error: unknown) {
      reportExpectedFailure({
        error,
        l: fnLog,
        operation: 'llama-server health probe',
      },);
    }
    // oxlint-disable-next-line no-await-in-loop -- sequential poll delay
    await setTimeout(HEALTH_POLL_MS,);
  }
  throw new Error(
    `llama-server failed to become healthy within ${MAX_HEALTH_POLLS} polls`,
  );
}

//endregion Server run
