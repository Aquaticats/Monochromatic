/**
 Tests for Unix domain socket IPC
 ({@link createOneShotSocketServer}, {@link readFromUnixSocket}).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createOneShotSocketServer,
  readFromUnixSocket,
} from './ipc-socket-unix.ts';

/** Wrap a socket server result so `using` calls cleanup automatically. */
function usingSocketServer(
  result: ReturnType<typeof createOneShotSocketServer>,
): Disposable & {
  socketPath: string;
} {
  return {
    socketPath: result.socketPath,
    [Symbol.dispose]: result.cleanup,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: createOneShotSocketServer.name,
      children: [
        it({
          name: 'serves text to the first client connection',
          timeout: 15_000,
          fn: async () => {
            const text = 'compressed context via socket';
            using server = usingSocketServer(createOneShotSocketServer(text,),);

            const received = await readFromUnixSocket(server.socketPath,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'handles large text (100KB+)',
          timeout: 15_000,
          fn: async () => {
            const text = 'x'.repeat(150_000,);
            using server = usingSocketServer(createOneShotSocketServer(text,),);

            const received = await readFromUnixSocket(server.socketPath,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'handles text with special characters',
          timeout: 15_000,
          fn: async () => {
            const text = 'hello\nworld\t"quotes" \'single\' $var `backtick` \\slash\\';
            using server = usingSocketServer(createOneShotSocketServer(text,),);

            const received = await readFromUnixSocket(server.socketPath,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'creates unique socket paths for each call',
          fn: async () => {
            const first = createOneShotSocketServer('first',);
            const second = createOneShotSocketServer('second',);

            expect(first.socketPath,).not.toBe(second.socketPath,);

            first.cleanup();
            second.cleanup();
          },
        },),
        it({
          name: 'closes server after serving one connection',
          timeout: 15_000,
          fn: async () => {
            const text = 'one-shot data';
            using server = usingSocketServer(createOneShotSocketServer(text,),);

            // First read succeeds
            const received = await readFromUnixSocket(server.socketPath,);
            expect(received,).toBe(text,);

            // Wait a moment for server to close
            await new Promise(
              function delay(resolve,): void {
                setTimeout(resolve, 200,);
              },
            );

            // Second connection should fail: server is closed
            let secondReadErrored = false;
            try {
              await readFromUnixSocket(server.socketPath,);
            }
            catch (error: unknown) {
              secondReadErrored = true;
              expect(error,).toBeDefined();
            }
            expect(secondReadErrored,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: readFromUnixSocket.name,
      children: [
        it({
          name: 'round-trips data through socket server',
          timeout: 15_000,
          fn: async () => {
            const text = 'round-trip via Unix socket';
            using server = usingSocketServer(createOneShotSocketServer(text,),);

            const received = await readFromUnixSocket(server.socketPath,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'throws on connection to nonexistent socket',
          fn: async () => {
            let caught = false;
            try {
              await readFromUnixSocket('/tmp/nonexistent-morph-compact-test.sock',);
            }
            catch (error: unknown) {
              caught = true;
              expect(error,).toBeDefined();
            }
            expect(caught,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
