import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { BoundViolation } from './native-perf-judge.ts';
import type { SampleFailure } from './native-perf-measure.ts';
import {
  assertPerfVerdict,
  PerfBoundsExceededError,
  PerfSamplesFailedError,
} from './native-perf-verdict.ts';

//region Verdict fixtures

/**
 A sample that failed because its Git command exited nonzero.
 */
const failedSample: SampleFailure = {
  scenario: 'forward',
  wrapped: true,
  sample: 3,
  status: 128,
  stderr: 'fatal: not a git repository',
};

/**
 A requirement a scenario broke: its added p95 against the bound that statistic must stay under.
 */
const boundViolation: BoundViolation = {
  scenario: 'forward',
  statistic: 'added p95',
  value: 2_500,
  limit: 2_000,
};

//endregion Verdict fixtures

await describe({
  name: 'native perf verdict',
  children: [
    it({
      name: 'accepts a run whose every sample was measured and no requirement broke',
      fn: async function acceptsCleanRun(): Promise<void> {
        // The control: without it a verdict that always throws would pass the three cases below.
        expect(function passesCleanRun(): void {
          assertPerfVerdict({
            failures: [],
            violations: [],
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'throws over a sample the run could not measure',
      fn: async function throwsOverUnmeasuredSample(): Promise<void> {
        /**
         The verdict of a run holding one sample that could not be measured.
         */
        function oneUnmeasuredSample(): void {
          assertPerfVerdict({
            failures: [failedSample,],
            violations: [],
          },);
        }
        expect(oneUnmeasuredSample,).toThrow(PerfSamplesFailedError,);
        expect(oneUnmeasuredSample,).toThrow('1 sample failed',);
        expect(oneUnmeasuredSample,).toThrow(failedSample.scenario,);
        expect(oneUnmeasuredSample,).toThrow('fatal: not a git repository',);
      },
    },),
    it({
      name: 'throws over a judged statistic that reached its bound',
      fn: async function throwsOverBoundViolation(): Promise<void> {
        /**
         The verdict of a run whose one judged statistic reached its bound.
         */
        function oneBrokenRequirement(): void {
          assertPerfVerdict({
            failures: [],
            violations: [boundViolation,],
          },);
        }
        expect(oneBrokenRequirement,).toThrow(PerfBoundsExceededError,);
        expect(oneBrokenRequirement,).toThrow(boundViolation.scenario,);
        expect(oneBrokenRequirement,).toThrow(boundViolation.statistic,);
        expect(oneBrokenRequirement,).toThrow(String(boundViolation.limit,),);
      },
    },),
    it({
      name: 'reports an unmeasurable run before judging what it measured',
      fn: async function reportsUnmeasuredRunFirst(): Promise<void> {
        // A sample the run could not measure makes every figure beside it unreliable, so that is the failure to name.
        expect(function crossesBoundWithUnmeasuredSample(): void {
          assertPerfVerdict({
            failures: [failedSample,],
            violations: [boundViolation,],
          },);
        },).toThrow(PerfSamplesFailedError,);
      },
    },),
  ],
},);
