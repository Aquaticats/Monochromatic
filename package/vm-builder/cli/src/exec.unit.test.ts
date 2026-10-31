import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { exec, } from '../dist/final/node/exec.mjs';

/**
 Awaits a promise and returns its rejection reason so assertions can read error properties;
 resolves to `undefined` when the promise fulfils.

 @param promise - Operation expected to reject

 @returns The rejection reason, or `undefined` when the operation succeeded
 */
async function rejectionOf(promise: Promise<unknown>,): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Error fields `nano-spawn` attaches that these tests read.
 */
type SubprocessFields = {
  readonly name: string;
  readonly message: string;
  readonly exitCode?: number;
  readonly signalName?: string;
  readonly stderr: string;
  readonly cause?: { readonly code?: string; };
};

await describe({
  name: '',
  children: [
    describe({
      name: exec.name,
      children: [
        //region Captured output

        it({
          name: 'resolves with stdout of a successful command',
          fn: async () => {
            const result = await exec({
              cmd: 'echo',
              args: ['hello', 'world',],
            },);
            expect(result,).toBe('hello world',);
          },
        },),
        it({
          name: 'resolves with an empty string when the command prints nothing',
          fn: async () => {
            const result = await exec({ cmd: 'true', },);
            expect(result,).toBe('',);
          },
        },),
        it({
          name: 'keeps interior newlines and strips one trailing newline',
          fn: async () => {
            const result = await exec({
              cmd: 'printf',
              args: [String.raw`line1\nline2\nline3\n`,],
            },);
            expect(result,).toBe('line1\nline2\nline3',);
          },
        },),
        it({
          name: 'passes arguments without shell expansion',
          fn: async () => {
            const result = await exec({
              cmd: 'printf',
              args: ['%s', '$HOME; echo injected',],
            },);
            expect(result,).toBe('$HOME; echo injected',);
          },
        },),
        it({
          name: 'captures large stdout without truncation',
          fn: async () => {
            const lineCount = 5_000;
            const result = await exec({
              cmd: 'seq',
              args: ['1', String(lineCount,),],
            },);
            expect(result.split('\n',).length,).toBe(lineCount,);
          },
        },),
        it({
          name: 'does not return stderr',
          fn: async () => {
            const result = await exec({
              cmd: 'sh',
              args: ['-c', 'echo out; echo err >&2',],
            },);
            expect(result,).toBe('out',);
          },
        },),
        it({
          name: 'accepts a timeout that the command beats',
          fn: async () => {
            const generousTimeoutMs = 10_000;
            const result = await exec({
              cmd: 'echo',
              args: ['fast',],
              timeoutMs: generousTimeoutMs,
            },);
            expect(result,).toBe('fast',);
          },
        },),

        //endregion Captured output

        //region Non-zero exit

        it({
          name: 'rejects with the exit code in the message and on the error',
          fn: async () => {
            const error = await rejectionOf(exec({
              cmd: 'sh',
              args: ['-c', 'exit 3',],
            },),) as SubprocessFields;
            expect(error.name,).toBe('SubprocessError',);
            expect(error.message,).toBe("Command failed with exit code 3: sh -c 'exit 3'",);
            expect(error.exitCode,).toBe(3,);
          },
        },),
        it({
          name: 'carries stderr on the error but not in the message',
          fn: async () => {
            const error = await rejectionOf(exec({
              cmd: 'sh',
              args: ['-c', 'printf "%s-%s" diag text >&2; exit 1',],
            },),) as SubprocessFields;
            expect(error.stderr,).toBe('diag-text',);
            expect(error.message,).not.toContain('diag-text',);
          },
        },),

        //endregion Non-zero exit

        //region Spawn failure

        it({
          name: 'rejects when the executable does not exist',
          fn: async () => {
            const error = await rejectionOf(exec({
              cmd: 'vm-builder-no-such-executable',
              args: ['--flag',],
            },),) as SubprocessFields;
            expect(error.name,).toBe('SubprocessError',);
            expect(error.message,).toBe('Command failed: vm-builder-no-such-executable --flag',);
            expect(error.exitCode,).toBeUndefined();
            expect(error.cause?.code,).toBe('ENOENT',);
          },
        },),

        //endregion Spawn failure

        //region Timeout

        it({
          name: 'terminates a command that outlives timeoutMs',
          fn: async () => {
            const timeoutMs = 100;
            const startedAt = performance.now();
            const error = await rejectionOf(exec({
              cmd: 'sleep',
              args: ['30',],
              timeoutMs,
            },),) as SubprocessFields;
            const elapsedMs = performance.now() - startedAt;
            expect(error.name,).toBe('SubprocessError',);
            expect(error.message,).toBe('Command was terminated with SIGTERM: sleep 30',);
            expect(error.signalName,).toBe('SIGTERM',);
            expect(error.exitCode,).toBeUndefined();
            expect(elapsedMs,).toBeLessThan(10_000,);
          },
          timeout: 20_000,
        },),

        //endregion Timeout
      ],
    },),
  ],
},);
