import { once, } from 'node:events';
import { createConnection, } from 'node:net';
import { setTimeout as wait, } from 'node:timers/promises';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

import { createAnswerChannel, } from '../dist/final/node/index.mjs';

//region Startup cancellation

await describe({
  name: 'answer channel cancellation',
  children: [
    it({
      name: 'preserves caller cancellation while no helper has started',
      fn: async () => {
        await using channel = await createAnswerChannel();
        const controller = new AbortController();
        controller.abort();
        let caught: unknown;
        try {
          await channel.wait({ signal: controller.signal, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(caught,).toHaveProperty('cause', controller.signal.reason,);
      },
    },),
    ...['idle', 'reset',].map(kind => it({
      name: `a ${kind} authentication candidate cannot expire the question`,
      timeout: 9_000,
      fn: async () => {
        await using channel = await createAnswerChannel();
        const controller = new AbortController();
        const state: { error?: unknown; } = {};
        const completion = (async function observe() {
          try {
            return await channel.wait({ signal: controller.signal, },);
          }
          catch (error: unknown) {
            state.error = error;
            throw error;
          }
        })();
        const settled = Promise.allSettled([completion,],);
        await using cleanup = { async [Symbol.asyncDispose](): Promise<void> {
          controller.abort();
          await settled;
        }, };
        const idle = createConnection({ host: channel.host, port: channel.port, },);
        using idleCleanup = { [Symbol.dispose](): void {
          idle.destroy();
        }, };
        idle.resume();
        await once(idle, 'connect',);
        const closed = once(idle, 'close',);
        if (kind === 'reset')
          idle.resetAndDestroy();
        await closed;
        await wait(0,);
        expect(state.error,).toBeUndefined();
        const helper = createConnection({ host: channel.host, port: channel.port, },);
        using helperCleanup = { [Symbol.dispose](): void {
          helper.destroy();
        }, };
        helper.resume();
        await once(helper, 'connect',);
        helper.end(`${channel.token}\n{"status":"cancelled"}`,);
        expect(await completion,).toEqual({ status: 'cancelled', },);
      },
    },),),
    it({
      name: 'disposes unauthenticated sockets without hanging shutdown',
      fn: async () => {
        await using channel = await createAnswerChannel();
        const socket = createConnection({ host: channel.host, port: channel.port, },);
        using cleanup = { [Symbol.dispose](): void {
          socket.destroy();
        }, };
        await once(socket, 'connect',);
        const closed = once(socket, 'close',);
        socket.resume();
        await channel[Symbol.asyncDispose]();
        await closed;
        expect(socket.destroyed,).toBe(true,);
      },
    },),
  ],
},);

//endregion Startup cancellation
