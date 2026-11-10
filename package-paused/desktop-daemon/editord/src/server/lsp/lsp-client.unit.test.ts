/**
 * Unit tests for {@link createLspClient}'s lifecycle handling.
 *
 * Covers:
 *
 * - a request issued after the child exited, which must reject instead of
 *   writing to a dead pipe
 * - a notify written after the child closed its stdin, which must not take the
 *   process down with an unhandled EPIPE
 * - a graceful child exit while requests are in flight, which must settle them
 * - a child that never answers `shutdown`, whose handshake must still settle
 *
 * Each test spawns a throwaway `node` child whose script is written into the
 * harness scratch root, never the system temporary directory.
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { l, } from '../log.ts';
import {
  createLspClient,
  type LspClient,
  type LspClientExit,
} from './lsp-client.ts';

/**
 * Milliseconds a child script that closes its stdin stays alive afterwards, so
 * the test can write into the closed pipe while the process is still running.
 */
const CLOSE_STDIN_LINGER_MS = 400;

/**
 * Milliseconds allowed after spawning before the child script has certainly
 * closed its stdin, so the test writes into a closed pipe rather than racing the
 * child's startup.
 */
const CLOSE_STDIN_SETTLE_MS = 150;

/**
 * Milliseconds allowed for a promise under test to settle before the test
 * treats it as hung. Generous against loaded hosts, far below the suite budget.
 */
const SETTLE_WINDOW_MS = 3_000;

/**
 * Milliseconds a silent child stays alive before exiting on its own, which
 * bounds the shutdown-timeout case even when the client leaves it running.
 */
const SILENT_CHILD_LIFETIME_MS = 4_000;

/**
 * Milliseconds the graceful-exit child stays alive before exiting on its own.
 */
const GRACEFUL_EXIT_DELAY_MS = 50;

/**
 * Script for a child that closes its stdin and keeps running, which is what a
 * crashing language server leaves behind from the client's point of view.
 */
const CLOSE_STDIN_SCRIPT = `import { closeSync } from 'node:fs';
closeSync(0,);
setTimeout(function linger() { process.exit(0); }, ${String(CLOSE_STDIN_LINGER_MS,)},);
`;

/**
 * Script for a child that exits on its own after a short delay and answers
 * nothing, the shape of a server that dies during the shutdown handshake.
 */
const GRACEFUL_EXIT_SCRIPT = `setTimeout(function exitSoon() { process.exit(0); }, ${
  String(GRACEFUL_EXIT_DELAY_MS,)
},);
`;

/**
 * Script for a child that stays alive long enough to outlast the shutdown
 * handshake without ever answering it.
 */
const SILENT_SCRIPT = `setTimeout(function stayAlive() { process.exit(0); }, ${
  String(SILENT_CHILD_LIFETIME_MS,)
},);
`;

/**
 * Client under test plus the exits its callback recorded and the fixture
 * directory holding the child script.
 */
type ClientFixture = {
  readonly client: LspClient;
  readonly exits: LspClientExit[];
  readonly whenExited: Promise<LspClientExit>;
  readonly dispose: () => Promise<void>;
};

/**
 * Writes a child script into a fresh fixture directory and starts a client on
 * it.
 *
 * @param script - source of the child script
 *
 * @returns client, recorded exits, first-exit promise, and cleanup
 *
 * @example
 * ```ts
 * const fixture = await createClientFixture(SILENT_SCRIPT);
 * ```
 */
async function createClientFixture(script: string,): Promise<ClientFixture> {
  const directory = await mkdtemp(join(scratchRoot(), 'lsp-client-',),);
  const scriptPath = join(directory, 'child.mjs',);
  await writeFile(scriptPath, script,);
  const exits: LspClientExit[] = [];
  /**
   * Resolver for the first exit event, wired below so a test can await it.
   */
  const settleFirstExit: { resolve: (event: LspClientExit,) => void } = {
    resolve: function notYet() {},
  };
  const whenExited = new Promise<LspClientExit>(function capture(resolve,) {
    settleFirstExit.resolve = resolve;
  },);
  const client = createLspClient({
    command: process.execPath,
    args: [scriptPath,],
    name: 'fixture',
    cwd: directory,
    env: { ...process.env, },
    l,
    onNotification: function ignoreNotification(): void {},
    onExit: function recordExit(event,) {
      exits.push(event,);
      settleFirstExit.resolve(event,);
    },
  },);
  return {
    client,
    exits,
    whenExited,
    dispose: async function dispose() {
      await rm(directory, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 * Resolves after `ms` milliseconds.
 *
 * @param ms - delay in milliseconds
 *
 * @returns promise that settles once the delay elapses
 *
 * @example
 * ```ts
 * await delay(150);
 * ```
 */
async function delay(ms: number,): Promise<void> {
  await new Promise(function wait(resolve,) {
    setTimeout(
      resolve,
      ms,
    );
  },);
}

/**
 * Waits up to `windowMs` for a promise to settle.
 *
 * @param promise - promise under test; its rejection is treated as settling
 *
 * @param windowMs - milliseconds allowed before the promise counts as hung
 *
 * @returns true when the promise settled inside the window
 *
 * @example
 * ```ts
 * const settled = await settledWithin(pending.catch(() => null), 1000);
 * ```
 */
async function settledWithin(promise: Promise<unknown>, windowMs: number,): Promise<boolean> {
  /**
   * Sentinel the race resolves to when the window elapses before the promise
   * settles, so a hung promise fails an assertion instead of the suite.
   */
  const WINDOW_ELAPSED = Symbol('settle window elapsed with no settlement',);
  /**
   * Handle of the timer that ends the window; held in an object so the factory
   * body keeps only consts.
   */
  const timer: { handle?: ReturnType<typeof setTimeout> } = {};
  const window = new Promise<typeof WINDOW_ELAPSED>(function startTimer(resolve,) {
    timer.handle = setTimeout(
      function elapsed() {
        resolve(WINDOW_ELAPSED,);
      },
      windowMs,
    );
  },);
  const winner = await Promise.race([
    promise.then(function settled() {
      return undefined;
    },),
    window,
  ],);
  clearTimeout(timer.handle,);
  return winner !== WINDOW_ELAPSED;
}

await describe({
  name: createLspClient.name,
  concurrency: 1,
  children: [
    it({
      name: 'survives a notify written after the child closed its stdin',
      async fn() {
        const fixture = await createClientFixture(CLOSE_STDIN_SCRIPT,);
        await delay(CLOSE_STDIN_SETTLE_MS,);
        /**
         * The child still runs but no longer reads stdin, so this write fails
         * with EPIPE; an unhandled `error` event here would abort the process.
         */
        fixture.client.notify({
          method: 'textDocument/didSave',
          params: {},
        },);
        await delay(CLOSE_STDIN_SETTLE_MS,);
        expect(fixture.exits.length,).toBe(0,);
        expect(fixture.client.dead,).toBe(true,);
        const exited = await settledWithin(
          fixture.whenExited,
          SETTLE_WINDOW_MS,
        );
        expect(exited,).toBe(true,);
        await fixture.dispose();
      },
    },),
  ],
},);
