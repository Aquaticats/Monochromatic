import { realpathSync } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnProgram } from './run-program.ts';

/**
 Directory of this test, used as a working directory the test can recognise.
 */
const workingDirectory = realpathSync(import.meta.dirname);

await describe({
  name: '',
  children: [
    describe({
      name: spawnProgram.name,
      children: [
        it({
          name: 'captures standard output, standard error and a zero status',
          fn: async () => {
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', `process.stdout.write('out'); process.stderr.write('err');`],
              cwd: workingDirectory,
              capture: true,
            });
            expect(result).toEqual({ status: 0, stdout: 'out', stderr: 'err' });
          },
        }),
        it({
          name: 'reports the exit status of a failing program without throwing',
          fn: async () => {
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', 'process.exitCode = 7'],
              cwd: workingDirectory,
              capture: true,
            });
            expect(result.status).toBe(7);
          },
        }),
        it({
          name: 'starts the program in the given working directory',
          fn: async () => {
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', 'process.stdout.write(process.cwd())'],
              cwd: workingDirectory,
              capture: true,
            });
            expect(result.stdout).toBe(workingDirectory);
          },
        }),
        it({
          name: 'passes shell syntax in an argument through literally, because no shell reads it',
          fn: async () => {
            const hostile = `$HOME; echo pwned \`id\` $(id) 'q' "d"`;
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', 'process.stdout.write(process.argv[1])', hostile],
              cwd: workingDirectory,
              capture: true,
            });
            expect(result.stdout).toBe(hostile);
          },
        }),
        it({
          name: 'reports a failure status for a program that a signal ended',
          fn: async () => {
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', `process.kill(process.pid, 'SIGKILL')`],
              cwd: workingDirectory,
              capture: true,
            });
            expect(result.status).toBe(1);
          },
        }),
        it({
          name: 'rejects when the program is not installed, naming it',
          fn: async () => {
            let caught: unknown;
            try {
              await spawnProgram({
                program: 'crate-release-no-such-program',
                args: [],
                cwd: workingDirectory,
                capture: true,
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(Error);
            expect((caught as Error).message).toContain('crate-release-no-such-program');
          },
        }),
        it({
          name: 'lets an inheriting call finish with its status and no captured text',
          fn: async () => {
            const result = await spawnProgram({
              program: 'node',
              args: ['--eval', ''],
              cwd: workingDirectory,
              capture: false,
            });
            expect(result).toEqual({ status: 0, stdout: '', stderr: '' });
          },
        }),
      ],
    }),
  ],
});
