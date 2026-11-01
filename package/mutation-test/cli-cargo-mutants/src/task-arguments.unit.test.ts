import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildTaskArguments } from './task-arguments.ts';

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
