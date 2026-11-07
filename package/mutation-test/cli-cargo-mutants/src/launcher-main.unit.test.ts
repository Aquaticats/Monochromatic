import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import { runLauncherMain } from './launcher-main.ts';

await describe({
  name: runLauncherMain.name,
  children: [
    it({
      name: 'returns the exit code the launch gave',
      fn: async () => {
        expect(await runLauncherMain({
          main: async () => 3,
          report: () => undefined,
        })).toBe(3);
      },
    }),
    it({
      name: 'prints only the message of a tooling error and returns 1',
      fn: async () => {
        const reported: string[] = [];
        const code = await runLauncherMain({
          main: async () => {
            throw new CargoMutantsToolingError('plain words');
          },
          report: (text) => {
            reported.push(text);
          },
        });
        expect(code).toBe(1);
        expect(reported).toEqual(['plain words']);
      },
    }),
    it({
      name: 'lets an unexpected error through with its stack, and prints nothing itself',
      fn: async () => {
        const reported: string[] = [];
        const failure = new RangeError('unexpected');
        let thrown: unknown;
        try {
          await runLauncherMain({
            main: async () => {
              throw failure;
            },
            report: (text) => {
              reported.push(text);
            },
          });
        }
        catch (error) {
          thrown = error;
        }
        expect(thrown).toBe(failure);
        expect(reported).toEqual([]);
      },
    }),
  ],
});
