import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runCli } from './cli-main.ts';
import { usageExitCode } from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Run the shared handling against a scripted command.

 @param argv - arguments after the script path
 @param execute - the command
 @returns the exit status and the printed text
 */
async function record({
  argv,
  execute,
}: {
  readonly argv: readonly string[];
  readonly execute: (argv: readonly string[]) => Promise<number>;
}) {
  const out: string[] = [];
  const err: string[] = [];
  const status = await runCli({
    argv,
    usage: 'USAGE TEXT',
    execute,
    print: function print(text) {
      out.push(text);
    },
    printError: function printError(text) {
      err.push(text);
    },
  });
  return { status, out, err };
}

await describe({
  name: '',
  children: [
    describe({
      name: runCli.name,
      children: [
        it({
          name: 'prints the usage on standard output and exits 0 for --help without running the command',
          fn: async () => {
            let ran = false;
            const result = await record({
              argv: ['--help'],
              execute: async function execute() {
                ran = true;
                return 0;
              },
            });
            expect(result).toEqual({ status: 0, out: ['USAGE TEXT'], err: [] });
            expect(ran).toBe(false);
          },
        }),
        it({
          name: 'reads a --help after a double hyphen as an argument, not as a request for help',
          fn: async () => {
            const result = await record({ argv: ['--', '--help'], execute: async () => 0 });
            expect(result.out).toEqual([]);
            expect(result.status).toBe(0);
          },
        }),
        it({
          name: 'prints the usage on standard error and exits with the usage status when there are no arguments',
          fn: async () => {
            const result = await record({ argv: [], execute: async () => 0 });
            expect(result).toEqual({ status: usageExitCode, out: [], err: ['USAGE TEXT'] });
          },
        }),
        it({
          name: 'returns the exit status of the command',
          fn: async () => {
            const result = await record({ argv: ['x'], execute: async () => 1 });
            expect(result.status).toBe(1);
          },
        }),
        it({
          name: 'turns a refusal into its message on standard error and the usage status',
          fn: async () => {
            const result = await record({
              argv: ['x'],
              execute: async function execute() {
                throw new CrateReleaseError('--scope is required.');
              },
            });
            expect(result).toEqual({ status: usageExitCode, out: [], err: ['--scope is required.'] });
          },
        }),
        it({
          name: 'rethrows any other error, because it is a defect and not bad input',
          fn: async () => {
            let caught: unknown;
            try {
              await record({
                argv: ['x'],
                execute: async function execute() {
                  throw new TypeError('defect');
                },
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(TypeError);
          },
        }),
      ],
    }),
  ],
});
