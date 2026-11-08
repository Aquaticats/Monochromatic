/**
 sops decrypt capture: the decrypt seam returns the child's whole stdout,
 and a child that ends non-zero still classifies as a decrypt failure.

 The capture regression this guards: joining captured chunks at the child's
 `exit` event truncates the tail, because the stdout pipe can still hold
 unread data after `exit` fires. A truncated join surfaces upstream as
 "decrypted env store ... is not valid JSON" whenever the document spans more
 than one pipe read.

 @module
 */

import { once, } from 'node:events';
import { chmod, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { PassThrough, } from 'node:stream';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempRoot,
  readerEnvironment,
} from './fixture.ts';
import {
  collectChildStdout,
  EnvLocalJsonDecryptError,
  runSopsDecrypt,
} from '../dist/final/node/testing.mjs';

//region Constants

/**
 Payload text length: three quarters of a mebibyte in text, far beyond one
 pipe buffer so the capture spans many reads, and under
 `MAX_SOPS_OUTPUT_BYTES` (1 MiB) so no overflow path can fire.
 */
const PAYLOAD_TEXT_BYTES = 3 * (2 ** 18);

/**
 Concurrent decrypt count: the live failure surfaced while a pi startup ran
 several store reads at once, so the guard reproduces that shape.
 */
const CONCURRENT_DECRYPTS = 8;

/**
 Payload text length of each concurrent decrypt: large enough to span pipe
 reads while the parent juggles every child at once.
 */
const CONCURRENT_PAYLOAD_TEXT_BYTES = 2 ** 18;

/**
 Output bound handed to the synthetic capture seam: far above the tiny
 synthetic document so no overflow path can fire.
 */
const SYNTHETIC_CAPTURE_BOUND = 2 ** 16;

//endregion Constants

//region Helpers

/**
 Writes a fake `sops` executable that prints one fixed document in two large
 chunks and then ends, the shape that left the tail unflushed at `exit`.

 @param root - disposable directory that holds the fake executable

 @param payload - document the fake prints to stdout

 @param exitCode - exit status the fake reports

 @returns path of the fake executable

 @example
 ```ts
 const fake = await writeFakeSops({ root, payload, exitCode: 0 });
 ```
 */
async function writeFakeSops({
  root,
  payload,
  exitCode,
}: {
  readonly root: string;
  readonly payload: string;
  readonly exitCode: number;
},): Promise<string> {
  /**
   Path of the fake executable.
   */
  const path = join(
    root,
    'fake-sops',
  );
  /**
   Script that rebuilds the payload from its length and prints it in halves.
   */
  const script = [
    '#!/usr/bin/env node',
    `const payload = JSON.stringify('x'.repeat(${String(payload.length - 2,)}));`,
    'process.stdout.write(payload.slice(0, Math.floor(payload.length / 2)));',
    'process.stdout.write(payload.slice(Math.floor(payload.length / 2)));',
    `process.exitCode = ${String(exitCode,)};`,
    '',
  ]
    .join('\n',);
  await writeFile(
    path,
    script,
    {
      mode: 0o700,
    },
  );
  await chmod(
    path,
    0o700,
  );
  return path;
}

/**
 Captures the rejection of a decrypt run.

 @param options - decrypt options for the expected-failure run

 @returns rejection value

 @throws Error when the decrypt unexpectedly resolves

 @example
 ```ts
 const error = await captureDecryptError({ sopsPath, storePath, childEnv });
 ```
 */
async function captureDecryptError(
  options: Readonly<{
    readonly sopsPath: string;
    readonly storePath: string;
    readonly childEnv: Readonly<Record<string, string>>;
  }>,
): Promise<unknown> {
  try {
    await runSopsDecrypt(options,);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the sops decrypt to fail',);
}

//endregion Helpers

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Document the synthetic child prints across its completion events.
         */
        const whole = '{"values":{"a":"b"}}';
        /**
         Stream carrying the synthetic child's stdout.
         */
        const stdout = new PassThrough();
        /**
         Synthetic child whose completion events the case dispatches itself.
         */
        const child = new EventTarget();
        /**
         Whether the overflow callback ran; the tiny document must not trip it.
         */
        let overflowed = false;
        const capture = collectChildStdout({
          completionSource: child,
          stdoutSource: { stdout, },
          onOverflow: function recordOverflow() {
            overflowed = true;
          },
          maxBytes: SYNTHETIC_CAPTURE_BOUND,
        },);
        stdout.write(whole.slice(0, 10),);
        child.dispatchEvent(new Event('exit',),);
        /**
         Macrotask boundary: the seam's completion handling runs here, so the
         tail below reaches stdout strictly after it.
         */
        await delay(0,);
        stdout.end(whole.slice(10),);
        await once(
          stdout,
          'end',
        );
        child.dispatchEvent(new Event('close',),);
        const captured = await capture;
        expect(captured.output)
          .toBe(whole,);
        expect(overflowed)
          .toBe(false,);
      },
      name: 'returns the tail that reaches stdout after the exit event once the child closes',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot({ prefix: 'env-local-json-sops-', },);
        /**
         Document the fake prints and the seam must return whole.
         */
        const payload = JSON.stringify(
          'x'
            .repeat(PAYLOAD_TEXT_BYTES,),
        );
        const sopsPath = await writeFakeSops({
          root: temp.root,
          payload,
          exitCode: 0,
        },);
        const decrypted = await runSopsDecrypt({
          sopsPath,
          storePath: join(
            temp.root,
            'unused-store.json',
          ),
          childEnv: readerEnvironment({ home: temp.root, }),
        },);
        expect(decrypted)
          .toBe(payload,);
      },
      name: 'returns the child\'s whole stdout when the child writes two large chunks and ends immediately',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot({ prefix: 'env-local-json-sops-', },);
        /**
         Document every concurrent fake prints.
         */
        const payload = JSON.stringify(
          'x'
            .repeat(CONCURRENT_PAYLOAD_TEXT_BYTES,),
        );
        const sopsPath = await writeFakeSops({
          root: temp.root,
          payload,
          exitCode: 0,
        },);
        /**
         Every decrypt result of the concurrent burst.
         */
        const results = await Promise.all(
          Array.from(
            {
              length: CONCURRENT_DECRYPTS,
            },
            function oneDecrypt() {
              return runSopsDecrypt({
                sopsPath,
                storePath: join(
                  temp.root,
                  'unused-store.json',
                ),
                childEnv: readerEnvironment({ home: temp.root, }),
              },);
            },
          ),
        );
        for (const decrypted of results) {
          expect(decrypted)
            .toBe(payload,);
        }
      },
      name: 'returns every whole document while concurrent decrypts jostle for the parent event loop',
    },),
    it({
      fn: async () => {
        await using temp = await createTempRoot({ prefix: 'env-local-json-sops-', },);
        /**
         Document the fake prints before reporting failure.
         */
        const payload = JSON.stringify(
          'x'
            .repeat(PAYLOAD_TEXT_BYTES,),
        );
        const sopsPath = await writeFakeSops({
          root: temp.root,
          payload,
          exitCode: 3,
        },);
        const error = await captureDecryptError({
          sopsPath,
          storePath: join(
            temp.root,
            'unused-store.json',
          ),
          childEnv: readerEnvironment({ home: temp.root, }),
        },);
        expect(error instanceof EnvLocalJsonDecryptError,)
          .toBe(true,);
        if (!(error instanceof EnvLocalJsonDecryptError))
          return;
        expect(error.kind)
          .toBe('exit',);
        expect(error.exitCode)
          .toBe(3,);
      },
      name: 'classifies a non-zero child end as a decrypt failure carrying its exit code',
    },),
  ],
  name: 'sops decrypt stdout capture',
},);
