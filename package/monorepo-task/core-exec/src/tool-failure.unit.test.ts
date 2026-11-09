import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isCapturedSubprocessFailure,
  ToolKilledBySignalError,
  ToolNotStartedError,
} from '../dist/final/node/testing.mjs';

/** Status a wrapped command reports when it ran and failed. */
const failingExitCode = 2;
/** Signal name a wrapped command reports when a signal ended it. */
const endingSignal = 'SIGTERM';

await describe({
  name: '',
  children: [
    describe({
      name: isCapturedSubprocessFailure.name,
      children: [
        it({
          name: 'refuses a value that is not an object',
          fn: async () => {
            expect(isCapturedSubprocessFailure(null,),).toBe(false,);
            expect(isCapturedSubprocessFailure('Command failed',),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a plain error carrying neither a status nor a signal',
          fn: async () => {
            const plainError = new Error('boom',);
            expect(isCapturedSubprocessFailure(plainError,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses the spawn-failure shape, whose exitCode key holds undefined',
          fn: async () => {
            expect(isCapturedSubprocessFailure({ exitCode: undefined, },),).toBe(false,);
            expect(isCapturedSubprocessFailure({ exitCode: undefined, stdout: '', stderr: '', },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'accepts a command that ran and reported an exit status',
          fn: async () => {
            expect(isCapturedSubprocessFailure({ exitCode: failingExitCode, },),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a command ended by a signal, which reports no exit status',
          fn: async () => {
            expect(isCapturedSubprocessFailure({ signalName: endingSignal, },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: ToolNotStartedError.name,
      children: [
        it({
          name: 'names the task, the tool, and what happened',
          fn: async () => {
            const error = new ToolNotStartedError({
              task: 'task-tsc',
              tool: 'tsc',
              detail: 'Command failed: tsc --build',
            },);
            expect(error.name,).toBe(ToolNotStartedError.name,);
            expect(error.message,).toBe('[task-tsc] failed to execute tsc: Command failed: tsc --build',);
          },
        },),
      ],
    },),
    describe({
      name: ToolKilledBySignalError.name,
      children: [
        it({
          name: 'names the task, the tool, and the signal that ended it',
          fn: async () => {
            const error = new ToolKilledBySignalError({
              task: 'task-oxlint',
              tool: 'oxlint',
              detail: endingSignal,
            },);
            expect(error.name,).toBe(ToolKilledBySignalError.name,);
            expect(error.message,).toBe(`[task-oxlint] oxlint terminated by signal: ${endingSignal}`,);
          },
        },),
      ],
    },),
  ],
},);
