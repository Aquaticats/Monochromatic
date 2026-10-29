/**
 Tests for the command runner: results for success, a non-zero exit, a missing program and a timeout.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NO_EXIT_STATUS,
  runCommand,
} from './command.ts';

await describe({
  name: runCommand.name,
  children: [
    it({
      name: 'reports status zero and the printed text of a program that succeeds',
      fn: async ({ expect, }) => {
        const result = await runCommand({ command: 'node', args: ['-e', 'console.log("out"); console.error("err")',], },);
        expect(result,).toEqual({ status: 0, stdout: 'out', stderr: 'err', },);
      },
    },),
    it({
      name: 'reports a non-zero exit status with its streams instead of throwing',
      fn: async ({ expect, }) => {
        const result = await runCommand({ command: 'node', args: ['-e', 'console.log("o"); console.error("e"); process.exit(3)',], },);
        expect(result,).toEqual({ status: 3, stdout: 'o', stderr: 'e', },);
      },
    },),
    it({
      name: 'reports no exit status for a program that does not exist',
      fn: async ({ expect, }) => {
        const result = await runCommand({ command: 'e2e-no-such-program', args: [], },);
        expect(result.status,).toBe(NO_EXIT_STATUS,);
        expect(result.stderr,).not.toBe('',);
      },
    },),
    it({
      name: 'passes extra environment variables on top of the current environment',
      fn: async ({ expect, }) => {
        const result = await runCommand({
          command: 'node',
          args: ['-e', 'console.log(process.env.E2E_PROBE + "," + (process.env.PATH !== undefined))',],
          env: { E2E_PROBE: 'yes', },
        },);
        expect(result.stdout,).toBe('yes,true',);
      },
    },),
    it({
      name: 'runs the program in the directory it is given',
      fn: async ({ expect, }) => {
        const result = await runCommand({ command: 'node', args: ['-e', 'console.log(process.cwd())',], cwd: '/', },);
        expect(result.stdout,).toBe('/',);
      },
    },),
    it({
      name: 'kills a program that runs past the timeout and reports no exit status',
      fn: async ({ expect, }) => {
        const result = await runCommand({ command: 'node', args: ['-e', 'setTimeout(() => {}, 30000)',], timeoutMs: 100, },);
        expect(result.status,).toBe(NO_EXIT_STATUS,);
      },
      timeout: 15_000,
    },),
  ],
},);
