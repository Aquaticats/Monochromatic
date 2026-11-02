import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseTimeoutAndExamine } from './campaign-options.ts';
import {
  buildTimeoutFlag,
  cpusFlag,
  examineFlag,
  memoryFlag,
  passesFlag,
  timeoutFlag,
} from './constants.ts';
import { buildTaskArguments } from './task-arguments.ts';

/**
 Mutant-name patterns of the cases: one with the alternation and spaces a mutant name needs, one with a wildcard.
 */
const patterns = ['replace foo|bar::baz', 'read_.*file'];

/**
 Position of `item` in `list`, which every case requires to exist.

 @param list - arguments under test
 @param item - argument to find
 @returns its index
 */
function positionOf(list: readonly string[], item: string): number {
  const position = list.indexOf(item);
  expect(position).toBeGreaterThanOrEqual(0);
  return position;
}

await describe({
  name: '',
  children: [
    describe({
      name: buildTaskArguments.name,
      children: [
        it({
          name: 'forwards neither container limit when neither is given',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--file',
              scope: 'src/native/a.rs',
              timeoutSeconds: 900,
            })).toEqual(['--timeout', '900', '--file', 'src/native/a.rs']);
          },
        }),
        it({
          name: 'forwards only the memory limit when only it is given',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 900,
              memory: '4g',
            })).toEqual(['--timeout', '900', '--file', 's', '--memory', '4g']);
          },
        }),
        it({
          name: 'forwards only the CPU count when only it is given',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 900,
              cpus: '3',
            })).toEqual(['--timeout', '900', '--file', 's', '--cpus', '3']);
          },
        }),
        it({
          name: 'forwards both limits, memory first, when both are given',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 900,
              memory: '4g',
              cpus: '2.5',
            })).toEqual(['--timeout', '900', '--file', 's', '--memory', '4g', '--cpus', '2.5']);
          },
        }),
        it({
          name: 'forwards values that need quoting unchanged, as single arguments',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--file',
              scope: 'src/native/a b.rs',
              timeoutSeconds: 900,
              memory: `1 g; rm -rf 'x'`,
              cpus: '$(id)\n2',
            })).toEqual([
              '--timeout',
              '900',
              '--file',
              'src/native/a b.rs',
              '--memory',
              `1 g; rm -rf 'x'`,
              '--cpus',
              '$(id)\n2',
            ]);
          },
        }),
        it({
          name: 'forwards no build timeout and no pattern when none is given',
          fn: async () => {
            const forwarded = buildTaskArguments({ scopeFlag: '--file', scope: 's', timeoutSeconds: 7 });
            expect(forwarded).not.toContain(buildTimeoutFlag);
            expect(forwarded).not.toContain(examineFlag);
          },
        }),
        it({
          name: 'forwards the build timeout as its own flag and value when it is given',
          fn: async () => {
            const buildTimeoutSeconds = 11;
            const forwarded = buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 7,
              buildTimeoutSeconds,
            });
            expect(forwarded[positionOf(forwarded, buildTimeoutFlag) + 1]).toBe(String(buildTimeoutSeconds));
            expect(forwarded[positionOf(forwarded, timeoutFlag) + 1]).toBe('7');
          },
        }),
        it({
          name: 'forwards every pattern as its own flag and value, in the order given',
          fn: async () => {
            const forwarded = buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 7,
              examine: patterns,
            });
            expect(forwarded.filter((argument) => argument === examineFlag)).toHaveLength(patterns.length);
            expect(forwarded.flatMap((argument, index) => (argument === examineFlag ? [forwarded[index + 1]] : [])))
              .toEqual(patterns);
          },
        }),
        it({
          name: 'forwards a repeated identical pattern once per occurrence',
          fn: async () => {
            const [pattern = ''] = patterns;
            const forwarded = buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 7,
              examine: [pattern, pattern],
            });
            expect(forwarded.filter((argument) => argument === pattern)).toHaveLength(2);
          },
        }),
        it({
          name: 'puts the patterns before the container limits, which the task reads in its own order',
          fn: async () => {
            const forwarded = buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 7,
              examine: patterns,
              memory: '4g',
              cpus: '3',
            });
            expect(positionOf(forwarded, examineFlag)).toBeLessThan(positionOf(forwarded, memoryFlag));
            expect(positionOf(forwarded, memoryFlag)).toBeLessThan(positionOf(forwarded, cpusFlag));
          },
        }),
        it({
          name: 'forwards no passes flag unless the caller names the passes, and then forwards it last as its own flag and value',
          fn: async () => {
            expect(buildTaskArguments({ scopeFlag: '--file', scope: 's', timeoutSeconds: 7 })).not.toContain(passesFlag);
            const forwarded = buildTaskArguments({
              scopeFlag: '--file',
              scope: 's',
              timeoutSeconds: 7,
              memory: '4g',
              cpus: '3',
              passes: 2,
            });
            expect(forwarded.slice(-2)).toEqual([passesFlag, '2']);
          },
        }),
        it({
          name: 'forwards exactly what the scoped task parser reads back, leaving only the scope for the scope parser',
          fn: async () => {
            const timeoutSeconds = 21;
            const buildTimeoutSeconds = 34;
            const scope = 'src/native/a.rs';
            const parsed = parseTimeoutAndExamine({
              options: buildTaskArguments({
                scopeFlag: '--file',
                scope,
                timeoutSeconds,
                buildTimeoutSeconds,
                examine: patterns,
              }),
              rules: {
                defaultTimeoutSeconds: timeoutSeconds + 1,
                maximumTimeoutSeconds: timeoutSeconds,
                defaultBuildTimeoutSeconds: buildTimeoutSeconds + 1,
                maximumBuildTimeoutSeconds: buildTimeoutSeconds,
              },
            });
            expect(parsed).toEqual({
              timeoutSeconds,
              buildTimeoutSeconds,
              examine: patterns,
              scopeOptions: ['--file', scope],
            });
          },
        }),
        it({
          name: 'uses the scope flag the caller names',
          fn: async () => {
            expect(buildTaskArguments({
              scopeFlag: '--only',
              scope: 'x',
              timeoutSeconds: 5,
            })).toEqual(['--timeout', '5', '--only', 'x']);
          },
        }),
      ],
    }),
  ],
});
