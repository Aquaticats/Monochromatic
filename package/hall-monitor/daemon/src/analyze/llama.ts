import {
  type ChildProcess,
  spawn as cpSpawn,
} from 'node:child_process';
import { setTimeout, } from 'node:timers/promises';

import spawn from 'nano-spawn';

import { log, } from '../infra/syslog.ts';

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
 Module-singleton mutable state for the running llama-server subprocess handle; wrapped so it satisfies no-module-root-let.
 */
const state: { server?: ChildProcess; } = {};

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
  if (state.server)
    return;

  log.debug('[llama] Starting llama-server via distrobox...',);
  state.server = cpSpawn(
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

  await waitForHealth();
  log.debug('[llama] Server ready.',);
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
  log.debug('[llama] Stopping llama-server...',);

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
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // process may already be gone, or pkill exits non-zero if no match
  }

  /**
   Milliseconds to wait after server exit for the port to be freed.
   */
  const PORT_FREE_DELAY_MS = 500;

  /**
   Pinned reference to the current server handle so the exit listener still works after `state.server` is nulled below.
   */
  const currentServer = state.server;
  currentServer.kill();
  // oxlint-disable-next-line promise/avoid-new -- wrapping Node.js event-based ChildProcess API
  await new Promise<void>(
    /**
     Registers promise settlement with the child exit capability.
     
     @param resolve - Promise settlement callback.
     
     @mutates resolve through currentServer.on listener retention and invocation
     */
    function awaitExit(resolve,): void {
      currentServer.on(
        'exit',
        resolve,
      );
    },
  );
  delete state.server;

  // Wait briefly for port to free up
  await setTimeout(PORT_FREE_DELAY_MS,);
  log.debug('[llama] Server stopped.',);
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
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // process may already be gone, or pkill exits non-zero if no match
  }
  delete state.server;
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
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      // server not up yet
    }
    // oxlint-disable-next-line no-await-in-loop -- sequential poll delay
    await setTimeout(HEALTH_POLL_MS,);
  }
  throw new Error(
    `llama-server failed to become healthy within ${MAX_HEALTH_POLLS} polls`,
  );
}
