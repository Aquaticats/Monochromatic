import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { systemdRunProgram } from './constants.ts';
import { startUnit } from './start-units.ts';

await describe({
  name: startUnit.name,
  children: [
    it({
      name: 'runs systemd-run with the arguments, captures its error text, and reports a started unit',
      fn: async () => {
        const calls: unknown[] = [];
        const outcome = await startUnit({
          unit: 'u',
          systemdRunArguments: ['--user', '--unit=u'],
          run: async function accept(call) {
            calls.push(call);
            return { status: 0, stderr: '' };
          },
        });
        expect(calls).toEqual([{ program: systemdRunProgram, args: ['--user', '--unit=u'], inheritStdio: false }]);
        expect(outcome).toEqual({ unit: 'u', started: true, line: 'u: started' });
      },
    }),
    it({
      name: 'reports a refused unit with the error text and started false',
      fn: async () => {
        const outcome = await startUnit({
          unit: 'u',
          systemdRunArguments: [],
          run: async function refuse() {
            return { status: 1, stderr: 'Unit u already exists.\n' };
          },
        });
        expect(outcome).toEqual({ unit: 'u', started: false, line: 'u: Unit u already exists.' });
      },
    }),
  ],
});
