/**
 Tests for TCP localhost IPC
 ({@link createOneShotTcpServer}, {@link readFromTcpSocket}).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createOneShotTcpServer,
  type OneShotTcpServerResult,
  readFromTcpSocket,
} from './ipc-socket-tcp.ts';

/** Wrap a TCP server result so `using` calls cleanup automatically. */
function usingTcpServer(
  result: OneShotTcpServerResult,
): Disposable & {
  address: string;
} {
  return {
    address: result.address,
    [Symbol.dispose]: result.cleanup,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: createOneShotTcpServer.name,
      children: [
        it({
          name: 'serves text to the first client connection',
          timeout: 15_000,
          fn: async () => {
            const text = 'compressed context via TCP';
            using server = usingTcpServer(await createOneShotTcpServer(text,),);

            const received = await readFromTcpSocket(server.address,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'handles large text (100KB+)',
          timeout: 15_000,
          fn: async () => {
            const text = 'x'.repeat(150_000,);
            using server = usingTcpServer(await createOneShotTcpServer(text,),);

            const received = await readFromTcpSocket(server.address,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'handles text with special characters',
          timeout: 15_000,
          fn: async () => {
            const text = 'hello\nworld\t"quotes" \'single\' $var \\slash\\';
            using server = usingTcpServer(await createOneShotTcpServer(text,),);

            const received = await readFromTcpSocket(server.address,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'returns address in host:port format',
          fn: async () => {
            using server = usingTcpServer(await createOneShotTcpServer('test',),);

            // oxlint-disable-next-line no-restricted-syntax/no-regex -- format check on `host:port`; the test asserts exact loopback IP shape with a numeric port suffix. Input length is bounded by IP+port format, no nested quantifiers.
            expect(server.address,).toMatch(/^127\.0\.0\.1:\d+$/u,);
          },
        },),
        it({
          name: 'closes server after serving one connection',
          timeout: 15_000,
          fn: async () => {
            const text = 'one-shot TCP data';
            using server = usingTcpServer(await createOneShotTcpServer(text,),);

            // First read succeeds
            const received = await readFromTcpSocket(server.address,);
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
              await readFromTcpSocket(server.address,);
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
      name: readFromTcpSocket.name,
      children: [
        it({
          name: 'round-trips data through TCP server',
          timeout: 15_000,
          fn: async () => {
            const text = 'round-trip via TCP';
            using server = usingTcpServer(await createOneShotTcpServer(text,),);

            const received = await readFromTcpSocket(server.address,);
            expect(received,).toBe(text,);
          },
        },),
        it({
          name: 'throws on connection to unavailable port',
          fn: async () => {
            let caught = false;
            try {
              await readFromTcpSocket('127.0.0.1:1',);
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
