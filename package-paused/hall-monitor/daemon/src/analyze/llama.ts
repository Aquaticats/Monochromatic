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

/**
 Signal the server's processes are asked to take, for the two stop paths.
 */
type StopSignal = 'KILL' | 'TERM';

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

/**
 Regular expression matched against every process's command line to find this
 daemon's server.

 `distrobox` owns the pid the server actually runs under, so a signal sent to the
 stored handle ends the wrapper and can leave the server behind; `pkill` matches
 the server's own command line instead, where the port identifies it.
 */
const SERVER_MATCH_PATTERN = `llama-server.*--port ${PORT}`;

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
 Starts the server inside a distrobox container with the AMD GPU overrides the
 model needs.

 @returns the wrapper's child handle, whose stdio is ignored because the server
   reports its state through the health endpoint

 @example
 ```ts
 watchChildEnd({ child: spawnWrappedServer() });
 ```
 */
function spawnWrappedServer(): ChildProcess {
  return cpSpawn(
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
}

/**
 Asks the operating system to end every process this daemon's server runs as.

 @param signal - TERM for a graceful stop, KILL for the shutdown path

 @throws when `pkill` is missing or refuses the call

 @example
 ```ts
 await signalServerByPattern({ signal: 'TERM' });
 ```
 */
async function signalServerByPattern({ signal, }: Readonly<{ signal: StopSignal; }>,): Promise<void> {
  await spawn(
    'pkill',
    signal === 'KILL'
      ? [
        '-9',
        '-f',
        SERVER_MATCH_PATTERN,
      ]
      : [
        '-f',
        SERVER_MATCH_PATTERN,
      ],
  );
}

/**
 Ends the server's processes without killing the wrapper abruptly, for
 {@link stop}.
 */
function signalServerGracefully(): Promise<void> {
  return signalServerByPattern({ signal: 'TERM', },);
}

/**
 Starts the server inside a distrobox container with AMD GPU overrides.
 No-ops if the server is already running. Blocks until the health endpoint
 reports ready or the timeout expires.

 @param spawnServer - starts the server process; the distrobox-wrapped one by
   default, and another one only so a test drives this module without a GPU

 @param waitForReady - resolves once the started server answers its health
   endpoint; the real poll by default, and a no-op only in a test whose child is
   not a server
 
 @throws when llama-server fails to become healthy within {@link HEALTH_TIMEOUT_MS}
 
 @example
 ```ts
 await start();
 // server is ready for inference
 ```
 */
export async function start({
  spawnServer = spawnWrappedServer,
  waitForReady = waitForHealth,
}: Readonly<{
  spawnServer?: () => ChildProcess;
  waitForReady?: () => Promise<void>;
}> = {},): Promise<void> {
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
  const server = spawnServer();
  state.server = server;
  watchChildEnd({ child: server, },);

  await waitForReady();
  fnLog.debug('Server ready.',);
}

/**
 Gracefully stops llama-server by killing the wrapped process by name
 (distrobox prevents direct PID signalling), then awaits subprocess exit.

 @param signalServer - asks the operating system to end the server's processes;
   `pkill` by default, and another one only so a test ends its own child at a
   known moment
 
 @example
 ```ts
 await stop();
 // VRAM is freed
 ```
 */
export async function stop({
  signalServer = signalServerGracefully,
}: Readonly<{ signalServer?: () => Promise<void>; }> = {},): Promise<void> {
  /**
   Handle taken before anything awaits, which is the only moment it is certainly
   held: the process can end while the call below is delivering its signal, and
   the `exit` listener attached at spawn time clears the stored handle when it
   does.
   */
  const currentServer = state.server;
  if (currentServer === undefined)
    return;
  /**
   Stop logger, tagged with this function's name under the module's subsystem tag.
   */
  const fnLog = tagged({
    l,
    tag: stop.name,
  },);
  fnLog.debug('Stopping llama-server...',);

  try {
    await signalServer();
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
    await signalServerByPattern({ signal: 'KILL', },);
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
