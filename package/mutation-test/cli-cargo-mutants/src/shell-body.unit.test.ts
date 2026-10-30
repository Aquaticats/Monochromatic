import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialValues } from './adversarial-fixture.ts';
import { defaultRetryMarker } from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  createFakeTools,
  readCallArguments,
  readCallCount,
} from './fake-tools-fixture.ts';
import { buildShellBody } from './shell-body.ts';

/**
 Body parameters shared by every case, with the log placed in the fixture directory.

 @param logBase - log path prefix
 @returns arguments for `buildShellBody`
 */
function baseArguments(logBase: string): Parameters<typeof buildShellBody>[0] {
  return {
    logBase,
    attempts: 3,
    retryMarker: defaultRetryMarker,
    gateSlotPath: '/x/gate-slot.ts',
    gateSlotArguments: [],
    task: '//package/p/cli:native:mutation:scoped',
    taskArguments: ['--timeout', '900', '--file', 'src/native/a.rs'],
  };
}

/**
 Run a body under a real `sh` with stand-in `node` and `rg` programs.

 @param failUntil - number of leading attempts whose log shows the retry marker
 @param taskArguments - task arguments to interpolate
 @param attempts - attempt count
 @returns the stand-in's call count and the arguments of its first call
 */
async function runBody({
  failUntil,
  taskArguments,
  attempts,
}: {
  readonly failUntil: number;
  readonly taskArguments: readonly string[];
  readonly attempts: number;
}): Promise<{
  readonly count: number;
  readonly first: readonly string[];
}> {
  /**
   Stand-in programs of this case, removed when the function ends.
   */
  await using tools = await createFakeTools(failUntil);
  /**
   Shell program under test.
   */
  const body = buildShellBody({
    ...baseArguments(join(tools.directory, 'campaign.log')),
    attempts,
    taskArguments,
  });
  spawnSync('sh', ['-c', body], {
    env: tools.environment,
    encoding: 'utf8',
  });
  return {
    count: await readCallCount(tools),
    first: await readCallArguments({ tools, call: 1 }),
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: buildShellBody.name,
      children: [
        //region Shape

        it({
          name: 'is a real sh for loop over the attempt numbers',
          fn: async () => {
            expect(
              buildShellBody(baseArguments('/l/c.log')),
            ).toContain('for attempt in 1 2 3; do ');
          },
        }),
        it({
          name: 'writes the loop for another attempt count',
          fn: async () => {
            expect(buildShellBody({ ...baseArguments('/l/c.log'), attempts: 5 })).toContain('for attempt in 1 2 3 4 5; do ');
          },
        }),
        it({
          name: 'breaks after a log without the retry marker',
          fn: async () => {
            expect(
              buildShellBody(baseArguments('/l/c.log')),
            ).toContain(
              `if ! rg --quiet --fixed-strings -- 'FAILED   Unmutated baseline' "$log.$attempt"; then break; fi; done`,
            );
          },
        }),
        it({
          name: 'runs the task through the limiter with its arguments after the separator',
          fn: async () => {
            expect(
              buildShellBody(baseArguments('/l/c.log')),
            ).toContain(
              `node '/x/gate-slot.ts' mise run '//package/p/cli:native:mutation:scoped' -- '--timeout' '900' '--file' 'src/native/a.rs' >> "$log.$attempt" 2>&1`,
            );
          },
        }),
        it({
          name: 'puts limiter options between the limiter and the task runner',
          fn: async () => {
            expect(buildShellBody({
              ...baseArguments('/l/c.log'),
              gateSlotArguments: ['--slot-count', '4'],
            })).toContain(`node '/x/gate-slot.ts' '--slot-count' '4' mise run `);
          },
        }),
        it({
          name: 'rejects a zero attempt count',
          fn: async () => {
            expect(() => buildShellBody({ ...baseArguments('/l/c.log'), attempts: 0 })).toThrow(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'rejects a NUL character in the file argument',
          fn: async () => {
            expect(() => buildShellBody({
              ...baseArguments('/l/c.log'),
              taskArguments: ['--file', 'a\0b'],
            })).toThrow(CargoMutantsToolingError);
          },
        }),

        //endregion Shape

        //region Execution under a real shell

        it({
          name: 'runs once and stops when the log lacks the marker',
          fn: async () => {
            const outcome = await runBody({ failUntil: 0, taskArguments: ['--file', 'x'], attempts: 3 });
            expect(outcome.count).toBe(1);
          },
        }),
        it({
          name: 'reruns while the log shows the marker, then stops at the first clean log',
          fn: async () => {
            const outcome = await runBody({ failUntil: 2, taskArguments: ['--file', 'x'], attempts: 3 });
            expect(outcome.count).toBe(3);
          },
        }),
        it({
          name: 'gives up after the last attempt when every log shows the marker',
          fn: async () => {
            const outcome = await runBody({ failUntil: 99, taskArguments: ['--file', 'x'], attempts: 3 });
            expect(outcome.count).toBe(3);
          },
        }),
        it({
          name: 'honors a different attempt count',
          fn: async () => {
            const outcome = await runBody({ failUntil: 99, taskArguments: ['--file', 'x'], attempts: 2 });
            expect(outcome.count).toBe(2);
          },
        }),
        ...adversarialValues.map(function adversarialFile(value) {
          return it({
            name: `delivers ${JSON.stringify(value)} to the task as one literal argument`,
            fn: async () => {
              const outcome = await runBody({
                failUntil: 0,
                taskArguments: ['--file', `src/native/${value}`, '--memory', value],
                attempts: 3,
              });
              expect(outcome.first.slice(-4)).toEqual(['--file', `src/native/${value}`, '--memory', value]);
              expect(outcome.count).toBe(1);
            },
          });
        }),

        //endregion Execution under a real shell
      ],
    }),
  ],
});
