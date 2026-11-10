/**
 Tests for TCP localhost IPC
 ({@link createOneShotTcpServer}, {@link readFromTcpSocket}).
 
 @module
 */

import { execFile, } from 'node:child_process';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';

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

/**
 Payload the child-process probe reads before it is expected to exit.
 */
const PROBE_TEXT = 'settled-read';

/**
 Milliseconds the child-process probe may live, far below the 10-second read
 timeout and far above the settle time of a successful read.
 */
const CHILD_EXIT_BUDGET_MS = 3_000;

/**
 Run a script that imports this module in a fresh process and return its output.

 The probe proves process-exit behaviour, which the reader's own timeout can
 only express through the event loop: a scheduled timer keeps the child alive.

 @param body - Script body appended after the module import

 @returns Trimmed child stdout

 @throws when the child exits non-zero or is killed at the exit budget
 */
async function runProbeChild(body: string,): Promise<string> {
  /**
   Module URL the probe script imports the reader under test from.
   */
  const moduleUrl = new URL(
    './ipc-socket-tcp.ts',
    import.meta.url,
  ).href;
  /**
   Complete probe script: import the module, then run the caller's steps.
   */
  const script = `import * as moduleUnderTest from ${JSON.stringify(moduleUrl,)};\n`
    + body;
  const { stdout, } = await promisify(execFile,)(
    process.execPath,
    [
      '--input-type=module-typescript',
      '--eval',
      script,
    ],
    {
      cwd: fileURLToPath(new URL('.', import.meta.url,),),
      timeout: CHILD_EXIT_BUDGET_MS,
    },
  );
  return stdout.trim();
}

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
        it({
          name: 'lets the reading process exit once the read settles',
          timeout: 15_000,
          fn: async () => {
            const childOutput = await runProbeChild(`
const { address } = await moduleUnderTest.createOneShotTcpServer(${JSON.stringify(PROBE_TEXT,)});
const text = await moduleUnderTest.readFromTcpSocket(address);
console.log(text);
`,);
            expect(childOutput,).toBe(PROBE_TEXT,);
          },
        },),
      ],
    },),
  ],
},);
