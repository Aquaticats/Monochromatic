import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  runProgram,
  signalExitStatus,
} from './run-program.ts';

await describe({
  name: '',
  children: [
    describe({
      name: runProgram.name,
      children: [
        it({
          name: 'reports a zero status and empty standard error for a program that succeeds',
          fn: async () => {
            expect(await runProgram({
              program: 'sh',
              args: ['-c', 'exit 0'],
              inheritStdio: false,
            })).toEqual({
              status: 0,
              stderr: '',
            });
          },
        }),
        it({
          name: 'reports the exit status of a failing program',
          fn: async () => {
            const result = await runProgram({
              program: 'sh',
              args: ['-c', 'exit 7'],
              inheritStdio: false,
            });
            expect(result.status).toBe(7);
          },
        }),
        it({
          name: 'captures standard error exactly, including quotes and newlines',
          fn: async () => {
            const result = await runProgram({
              program: 'sh',
              args: ['-c', String.raw`printf "it's\nbroken" >&2`],
              inheritStdio: false,
            });
            expect(result.stderr).toBe(`it's\nbroken`);
          },
        }),
        it({
          name: 'passes each argument as one argument, without a shell',
          fn: async () => {
            const result = await runProgram({
              program: 'sh',
              args: ['-c', 'printf "%s" "$1" >&2', 'sh', '$(echo injected); a b'],
              inheritStdio: false,
            });
            expect(result.stderr).toBe('$(echo injected); a b');
          },
        }),
        it({
          name: 'leaves captured standard error empty when the streams are inherited',
          fn: async () => {
            const result = await runProgram({
              program: 'sh',
              args: ['-c', 'exit 0'],
              inheritStdio: true,
            });
            expect(result.stderr).toBe('');
          },
        }),
        it({
          name: 'reports the signal status when a signal ends the program',
          fn: async () => {
            const result = await runProgram({
              program: 'sh',
              args: ['-c', 'kill -9 $$'],
              inheritStdio: false,
            });
            expect(result.status).toBe(signalExitStatus);
          },
        }),
        it({
          name: 'rejects when the program is not installed',
          fn: async () => {
            let caught: unknown;
            try {
              await runProgram({
                program: 'definitely-not-an-installed-program',
                args: [],
                inheritStdio: false,
              });
            } catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(Error);
          },
        }),
      ],
    }),
  ],
});
