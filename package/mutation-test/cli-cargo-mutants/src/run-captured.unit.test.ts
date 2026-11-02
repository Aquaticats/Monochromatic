import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runCaptured } from './run-captured.ts';

/**
 Print two environment variables with the current Node, which is always on the path of a test run.
 */
const printVariables = ['--eval', 'process.stdout.write((process.env.RUN_CAPTURED_PROBE ?? "unset") + " " + (process.env.RUN_CAPTURED_INHERITED ?? "unset"))'];

await describe({
  name: runCaptured.name,
  children: [
    it({
      name: 'runs the program with exactly the given environment, nothing inherited, and with the caller\'s own when none is given',
      fn: async () => {
        process.env.RUN_CAPTURED_INHERITED = 'inherited';
        const without = await runCaptured({ program: process.execPath, args: printVariables, capture: true });
        const withVariable = await runCaptured({
          program: process.execPath,
          args: printVariables,
          capture: true,
          environment: { RUN_CAPTURED_PROBE: 'given' },
        });
        delete process.env.RUN_CAPTURED_INHERITED;
        expect(without.stdout).toBe('unset inherited');
        expect(withVariable.stdout).toBe('given unset');
      },
    }),
  ],
});
