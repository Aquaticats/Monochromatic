import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  maximumUnitIoWeight,
  unitIoWeight,
} from './constants.ts';
import { buildSystemdRunArguments } from './systemd-run-arguments.ts';

await describe({
  name: '',
  children: [
    describe({
      name: buildSystemdRunArguments.name,
      children: [
        it({
          name: 'builds a capped, collected user unit that runs the body under sh -c',
          fn: async () => {
            expect(buildSystemdRunArguments({
              unit: 'u',
              memoryMax: '12G',
              cpuQuota: '500%',
              workingDirectory: '/w',
              environment: ['A=1', 'B=2'],
              body: 'echo hi',
            })).toEqual([
              '--user',
              '--unit=u',
              '--collect',
              '--property=MemoryMax=12G',
              '--property=CPUQuota=500%',
              `--property=IOWeight=${String(unitIoWeight)}`,
              '--working-directory=/w',
              '--setenv=A=1',
              '--setenv=B=2',
              'sh',
              '-c',
              'echo hi',
            ]);
          },
        }),
        it({
          name: 'sets a disk weight within the owner limit and a task limit',
          fn: async () => {
            const built = buildSystemdRunArguments({
              unit: 'u',
              memoryMax: '12G',
              cpuQuota: '500%',
              workingDirectory: '/w',
              environment: [],
              body: 'true',
            });
            const weights = built.flatMap((argument) => /^--property=IOWeight=(\d+)$/u.exec(argument)?.[1] ?? []);
            expect(weights).toHaveLength(1);
            expect(Number(weights[0])).toBeLessThanOrEqual(maximumUnitIoWeight);
            expect(built.filter((argument) => /^--property=TasksMax=[1-9]\d*$/u.test(argument))).toHaveLength(1);
          },
        }),
        it({
          name: 'passes the body as one untouched argument',
          fn: async () => {
            const body = `for a in 1; do echo "$a" 'b c'; done\nnext`;
            const built = buildSystemdRunArguments({
              unit: 'u',
              memoryMax: '1G',
              cpuQuota: '100%',
              workingDirectory: '/w',
              environment: [],
              body,
            });
            expect(built.at(-1)).toBe(body);
          },
        }),
      ],
    }),
  ],
});
