/**
 Tests for extension wiring in the built bash-poke artifact.

 @module
 */

import { mkdtemp, rm, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  DEFAULT_SETTINGS,
  pendingBashResult,
  registerBashPoke,
  resolveShell,
  type BashPokeSettings,
  type ShellInvocation,
} from '../dist/final/node/index.mjs';

import {
  createFakeContext,
  createFakePi,
  type EventHandler,
} from './test-fixtures.ts';

//region Fixtures

/**
 Milliseconds between polls while waiting for a poke.
 */
const POLL_INTERVAL_MS = 20;

/**
 Milliseconds a poke is given to arrive before a case fails.
 */
const POKE_DEADLINE_MS = 5_000;

/**
 Milliseconds given to prove a cancelled job never pokes.
 */
const SILENCE_WINDOW_MS = 400;

/**
 Resolved shell used by every case.
 */
const shell: ShellInvocation = await resolveShell({
  pathValue: process.env.PATH ?? '',
}, );

/**
 Settings with small budgets so cases stay fast.
 */
const settings: BashPokeSettings = {
  ...DEFAULT_SETTINGS,
  pokeHeadChars: 200,
  pokeTailChars: 200,
  killGraceMs: 100,
  progressRefreshMs: 10,
};

/**
 Polls until a condition holds or the deadline passes.
 
 @param condition - predicate checked each poll
 
 @returns whether the condition held before the deadline
 
 @example
 ```ts
 await pollUntil(function sent() { return messages.length > 0; },);
 ```
 */
async function pollUntil(condition: () => boolean, ): Promise<boolean> {
  /**
   Deadline after which waiting is pointless.
   */
  const deadline = Date.now() + POKE_DEADLINE_MS;
  while (Date.now() < deadline) {
    if (condition())
      return true;
    // oxlint-disable-next-line no-await-in-loop -- polling a condition is inherently sequential: the next check must see whatever the previous interval produced.
    await wait(POLL_INTERVAL_MS, );
  }
  return condition();
}

/**
 Retrieves the sole handler registered for one event.
 
 @param handlers - registrations recorded by the fake Pi API
 
 @param event - event name to retrieve
 
 @returns registered handler
 
 @example
 ```ts
 onlyHandler(fake.handlers, 'user_bash');
 ```
 */
function onlyHandler(
  handlers: ReadonlyMap<string, readonly EventHandler[]>,
  event: string,
): EventHandler {
  /**
   Handlers registered for that event.
   */
  const registered = handlers.get(event, );
  if ((registered === undefined) || (registered.length !== 1))
    throw new Error(`expected one ${event} handler`);

  /**
   Sole handler for the event.
   */
  const [handler] = registered;
  if (handler === undefined)
    throw new Error(`expected one ${event} handler`);
  return handler;
}

/**
 Registers the extension against fresh fakes and a disposable temp root.
 
 @param tmp - temp root spool files are written below
 
 @returns fakes needed to drive one case
 
 @example
 ```ts
 const wired = wireExtension(tmp);
 ```
 */
function wireExtension(tmp: string, ) {
  /**
   Fake Pi API recording registrations and messages.
   */
  const fake = createFakePi();

  /**
   Fake interactive context recording widget and input wiring.
   */
  const context = createFakeContext({ mode: 'tui', hasUI: true, }, );
  registerBashPoke({ pi: fake.api, settings, shell, tmp, }, );
  return { fake, context, };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region pendingBashResult

    describe({
      name: pendingBashResult.name,
      children: [
        it({
          name: 'records the placeholder Pi shows while a job runs',
          fn: async () => {
            expect(pendingBashResult(), ).toEqual({
              output: constants.PENDING_NOTE,
              exitCode: undefined,
              cancelled: false,
              truncated: false,
            }, );
          },
        }, ),
      ],
    }, ),

    //endregion pendingBashResult

    //region registerBashPoke

    describe({
      name: registerBashPoke.name,
      children: [
        it({
          name: 'registers the renderer and exactly two event handlers',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, } = wireExtension(tmp, );
            expect(fake.renderers.has(constants.POKE_CUSTOM_TYPE, ), ).toBe(true);
            expect([...fake.handlers.keys(), ].toSorted(), )
              .toEqual(['session_start', 'user_bash', ]);
          },
        }, ),
        it({
          name: 'leaves a hidden command to Pi',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );

            /**
             Result of handing Pi a hidden command.
             */
            const result = await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'printf hidden',
              excludeFromContext: true,
              cwd: tmp,
            }, context.ctx, );
            expect(result, ).toBeUndefined();
            expect(fake.sentMessages, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'returns the placeholder immediately and pokes when the job exits',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );

            /**
             Result handed back to Pi while the job runs.
             */
            const result = await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'printf poked',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, ) as { result: { output: string; }; };
            expect(result.result.output, ).toBe(constants.PENDING_NOTE);

            expect(await pollUntil(function pokeArrived(): boolean {
              return fake.sentMessages.length > 0;
            }, ), ).toBe(true);

            /**
             Sole poke produced by the finished job.
             */
            const [sent] = fake.sentMessages;
            if (sent === undefined)
              throw new Error('expected one poke');
            expect(String(sent.message.content, ), )
              .toContain('[bash finished] $ printf poked (exit 0)');
            expect(String(sent.message.content, ), ).toContain('poked');
            expect(sent.options, ).toEqual({ triggerTurn: true, deliverAs: 'followUp', });
          },
        }, ),
        it({
          name: 'draws and then clears the progress widget around a job',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );
            await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'printf drawn',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, );
            expect(await pollUntil(function pokeArrived(): boolean {
              return fake.sentMessages.length > 0;
            }, ), ).toBe(true);
            expect(context.widgets.length > 1, ).toBe(true);
            expect(context.widgets.at(-1)?.kind, ).toBe('clear');
          },
        }, ),
        it({
          name: 'cancels a running job on a lone escape and never pokes for it',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );
            await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'sleep 30',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, );

            /**
             Terminal input listener the session binding registered.
             */
            const [listener] = context.terminalListeners;
            if (listener === undefined)
              throw new Error('expected a terminal input listener');
            expect(listener(constants.BARE_ESCAPE, ), ).toBeUndefined();
            expect(context.notifications, ).toEqual(['Cancelled 1 background command(s)', ]);

            await wait(SILENCE_WINDOW_MS, );
            expect(fake.sentMessages, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'leaves escape alone while Pi is in bash mode',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );
            await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'sleep 30',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, );
            context.editor.text = '! partial';

            /**
             Terminal input listener the session binding registered.
             */
            const [listener] = context.terminalListeners;
            if (listener === undefined)
              throw new Error('expected a terminal input listener');
            listener(constants.BARE_ESCAPE, );
            expect(context.notifications, ).toEqual([]);

            // The job is still running, so the case must cancel it explicitly
            // instead of leaving a sleep behind for the rest of the suite.
            context.editor.text = '';
            listener(constants.BARE_ESCAPE, );
            expect(context.notifications, ).toEqual(['Cancelled 1 background command(s)', ]);
            await wait(SILENCE_WINDOW_MS, );
            expect(fake.sentMessages, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'leaves escape alone while the agent is streaming',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );
            await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'sleep 30',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, );
            context.idle.value = false;

            /**
             Terminal input listener the session binding registered.
             */
            const [listener] = context.terminalListeners;
            if (listener === undefined)
              throw new Error('expected a terminal input listener');
            listener(constants.BARE_ESCAPE, );
            expect(context.notifications, ).toEqual([]);

            context.idle.value = true;
            listener(constants.BARE_ESCAPE, );
            expect(context.notifications, ).toEqual(['Cancelled 1 background command(s)', ]);
            await wait(SILENCE_WINDOW_MS, );
          },
        }, ),
        it({
          name: 'registers no terminal listener outside the interactive TUI',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const fake = createFakePi();
            const context = createFakeContext({ mode: 'rpc', hasUI: true, }, );
            registerBashPoke({ pi: fake.api, settings, shell, tmp, }, );
            onlyHandler(fake.handlers, 'session_start', )(
              { type: 'session_start', reason: 'new', },
              context.ctx,
            );
            expect(context.terminalListeners, ).toHaveLength(0);
          },
        }, ),
        it({
          name: 'rebuilds the binding when Pi replaces the session',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const fake = createFakePi();
            const first = createFakeContext({ mode: 'tui', hasUI: true, }, );
            const second = createFakeContext({ mode: 'tui', hasUI: true, }, );
            registerBashPoke({ pi: fake.api, settings, shell, tmp, }, );

            /**
             Session start handler under test.
             */
            const handler = onlyHandler(fake.handlers, 'session_start', );
            handler({ type: 'session_start', reason: 'new', }, first.ctx, );
            expect(first.terminalListeners, ).toHaveLength(1);
            handler({ type: 'session_start', reason: 'resume', }, second.ctx, );
            // Disposal unsubscribes from the replaced context and the new one
            // registers its own listener.
            expect(first.terminalListeners, ).toHaveLength(0);
            expect(second.terminalListeners, ).toHaveLength(1);
          },
        }, ),
        it({
          name: 'builds a binding on first use when no session start arrived',
          fn: async () => {
            const tmp = await mkdtemp(join(scratchRoot(), 'bash-poke-register-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(tmp, { recursive: true, force: true, }, );
              },
            };
            const { fake, context, } = wireExtension(tmp, );
            await onlyHandler(fake.handlers, 'user_bash', )({
              type: 'user_bash',
              command: 'printf late',
              excludeFromContext: false,
              cwd: tmp,
            }, context.ctx, );
            expect(context.terminalListeners, ).toHaveLength(1);
            expect(await pollUntil(function pokeArrived(): boolean {
              return fake.sentMessages.length > 0;
            }, ), ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion registerBashPoke
  ],
}, );
