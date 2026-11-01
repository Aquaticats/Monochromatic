import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import type { DispatchOptions } from './dispatch-options.ts';
import { dispatchRelease } from './dispatch-run.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';

/**
 Options of the fixture dispatch.
 */
const options: DispatchOptions = {
  root: '/repo',
  crate: 'forbidden-strings',
  from: '0.6.0',
  to: '0.6.1',
  note: 'Skip the generated rule sources.',
  dryRun: false,
};

/**
 Run a dispatch against a canned `gh`.

 @param dispatchOptions - options of the run
 @param result - what `gh` prints and exits with
 @returns the exit status, the calls `gh` got and the printed lines
 */
async function record({
  dispatchOptions = options,
  result = { status: 0, stdout: '', stderr: '' },
}: {
  readonly dispatchOptions?: DispatchOptions;
  readonly result?: ProgramResult;
} = {}) {
  const calls: ProgramCall[] = [];
  const printed: string[] = [];
  const status = await dispatchRelease({
    options: dispatchOptions,
    run: async function run(call) {
      calls.push(call);
      return result;
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  return { status, calls, printed };
}

await describe({
  name: '',
  children: [
    describe({
      name: dispatchRelease.name,
      children: [
        it({
          name: 'starts the workflow with gh in the repository root and prints the run address',
          fn: async () => {
            const result = await record({
              result: {
                status: 0,
                stdout: '✓ Created workflow_dispatch event for crate-release.yml at main\nhttps://github.com/o/r/actions/runs/42\n',
                stderr: '',
              },
            });
            expect(result.status).toBe(0);
            expect(result.calls).toEqual([
              {
                program: 'gh',
                args: [
                  'workflow',
                  'run',
                  'crate-release.yml',
                  '--raw-field=crate=forbidden-strings',
                  '--raw-field=from=0.6.0',
                  '--raw-field=to=0.6.1',
                  '--raw-field=note=Skip the generated rule sources.',
                ],
                cwd: '/repo',
              },
            ]);
            expect(result.printed).toContain('https://github.com/o/r/actions/runs/42');
            expect(result.printed.at(-1)).toBe('RELEASE STARTED https://github.com/o/r/actions/runs/42');
          },
        }),
        it({
          name: 'prints a copyable command line before it starts the workflow',
          fn: async () => {
            const result = await record();
            expect(result.printed[0]).toBe(
              "$ gh workflow run crate-release.yml --raw-field=crate=forbidden-strings --raw-field=from=0.6.0 --raw-field=to=0.6.1 '--raw-field=note=Skip the generated rule sources.'",
            );
          },
        }),
        it({
          name: 'names a dry run in the closing line and passes the input',
          fn: async () => {
            const result = await record({
              dispatchOptions: { ...options, dryRun: true },
              result: { status: 0, stdout: 'https://github.com/o/r/actions/runs/7\n', stderr: '' },
            });
            expect(result.calls[0]?.args).toContain('--raw-field=dry-run=true');
            expect(result.printed.at(-1)).toBe('RELEASE DRY RUN STARTED https://github.com/o/r/actions/runs/7');
          },
        }),
        it({
          name: 'passes the repository to gh when one is named',
          fn: async () => {
            const result = await record({ dispatchOptions: { ...options, repo: 'o/r' } });
            expect(result.calls[0]?.args).toContain('--repo=o/r');
          },
        }),
        it({
          name: 'says so and shows how to list the run when gh printed no address',
          fn: async () => {
            const result = await record({
              result: { status: 0, stdout: '✓ Created workflow_dispatch event for crate-release.yml at main\n', stderr: '' },
            });
            expect(result.status).toBe(0);
            expect(result.printed.at(-1)).toContain('RELEASE STARTED, but gh did not print the address of the created run.');
            expect(result.printed.at(-1)).toContain('gh run list --workflow=crate-release.yml --limit=1');
          },
        }),
        it({
          name: 'shows what gh said, names the workflow and returns the failure status when gh refuses',
          fn: async () => {
            const result = await record({
              result: { status: 1, stdout: '', stderr: "could not find any workflows named crate-release.yml\n" },
            });
            expect(result.status).toBe(1);
            expect(result.printed).toContain('could not find any workflows named crate-release.yml');
            expect(result.printed.at(-1)).toBe('FAILED crate-release.yml was not started (gh exited 1)');
            expect(result.printed.some(function isStarted(line) {
              return line.startsWith('RELEASE STARTED');
            })).toBe(false);
          },
        }),
        ...adversarialLines.map(function noteCase(note) {
          return it({
            name: `passes the note ${JSON.stringify(note)} to gh as one argument`,
            fn: async () => {
              const result = await record({ dispatchOptions: { ...options, note } });
              expect(result.calls[0]?.args.at(6)).toBe(`--raw-field=note=${note}`);
              expect(result.calls[0]?.args).toHaveLength(7);
            },
          });
        }),
      ],
    }),
  ],
});
