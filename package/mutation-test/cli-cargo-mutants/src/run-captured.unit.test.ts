import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runCaptured } from './run-captured.ts';

/**
 Print one environment variable with the current Node, which is always on the path of a test run.
 */
const printVariable = ['--eval', 'process.stdout.write(process.env.RUN_CAPTURED_PROBE ?? "unset")'];

await describe({
  name: runCaptured.name,
  children: [
    it({
      name: 'runs the program with the given variables over the caller\'s own, and with the caller\'s alone when none are given',
      fn: async () => {
        const without = await runCaptured({ program: process.execPath, args: printVariable, capture: true });
        const withVariable = await runCaptured({
          program: process.execPath,
          args: printVariable,
          capture: true,
          environment: { RUN_CAPTURED_PROBE: 'given' },
        });
        expect(without.stdout).toBe('unset');
        expect(withVariable.stdout).toBe('given');
      },
    }),
  ],
});
