import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { BoundViolation } from './native-perf-judge.ts';
import {
  assertSamplesSucceededAndBoundsHeld,
  violationLine,
} from './native-perf-lines.ts';
import type { SampleFailure } from './native-perf-measure.ts';

/**
 A sample whose command answered with a failure status, as a measured run reports it.
 */
const failedSample: SampleFailure = {
  scenario: 'status',
  wrapped: true,
  sample: 3,
  status: 128,
  stderr: 'fatal: not a git repository',
};

/**
 The line a run with one failed sample fails on.
 */
const failedSamplesLine = `FAILED samples: 1; first: ${JSON.stringify(failedSample)}`;

/**
 A forwarded command whose added p95 reached the bound it is judged on.
 */
const brokenBound: BoundViolation = {
  scenario: 'status',
  statistic: 'added p95',
  value: 2_500,
  limit: 2_000,
};

await describe({
  name: assertSamplesSucceededAndBoundsHeld.name,
  children: [
    it({
      name: 'a run whose every sample succeeded and every bound held returns',
      fn: async () => {
        expect(() => assertSamplesSucceededAndBoundsHeld({
          failures: [],
          violations: [],
        })).not.toThrow();
      },
    }),
    it({
      name: 'a failed sample fails the run with its count and the first one',
      fn: async () => {
        expect(() => assertSamplesSucceededAndBoundsHeld({
          failures: [failedSample],
          violations: [],
        })).toThrow(failedSamplesLine);
      },
    }),
    it({
      name: 'a broken bound fails the run with the line the report states it on',
      fn: async () => {
        expect(() => assertSamplesSucceededAndBoundsHeld({
          failures: [],
          violations: [brokenBound],
        })).toThrow(violationLine(brokenBound));
      },
    }),
    it({
      name: 'a run with both states the failed samples before the broken bound',
      fn: async () => {
        expect(() => assertSamplesSucceededAndBoundsHeld({
          failures: [failedSample],
          violations: [brokenBound],
        })).toThrow([
          failedSamplesLine,
          violationLine(brokenBound),
        ].join('\n'));
      },
    }),
  ],
});
