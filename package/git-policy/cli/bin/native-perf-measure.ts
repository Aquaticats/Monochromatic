/**
 Paired direct-versus-wrapped sampling for one scenario of the latency benchmark in `native-perf.ts`.
 */
import { timed } from './native-perf-git.ts';
import {
  median,
  percentile95,
  summary,
} from './native-perf-statistics.ts';

/**
 One sample whose Git command exited nonzero.
 */
export type SampleFailure = {
  scenario: string;
  wrapped: boolean;
  sample: number;
  status: number | null;
  stderr: string
};

/**
 One scenario's statistics and raw samples.
 */
export type ScenarioReport = {
  scenario: string;
  enforced: boolean;
  direct: ReturnType<typeof summary>;
  wrapped: ReturnType<typeof summary>;
  addedMedianMilliseconds: number;
  addedP95Milliseconds: number;
  rawDirect: readonly number[];
  rawWrapped: readonly number[]
};

/**
 What one scenario measured: every failed sample, and the report unless either side has no successful sample.
 */
export type ScenarioMeasurement = {
  failures: readonly SampleFailure[];
  entry: ScenarioReport | undefined
};

/**
 Everything one scenario needs to run its samples.
 */
export type ScenarioSetting = {
  name: string;
  prepare?: (request: {
    directory: string;
    sample: number
  }) => Promise<void>;
  command: (request: { sample: number }) => readonly string[];
  enforced: boolean;
  samples: number;
  root: string;
  directRepository: string;
  wrappedRepository: string
};

/**
 One timed sample and which side and round it belongs to.
 */
export type TimedSample = {
  wrapped: boolean;
  sample: number;
  result: Awaited<ReturnType<typeof timed>>
};

/**
 Prepare and time one sample; the preparation runs before the timer starts, so it never counts as latency.

 @param request -
   the scenario's hooks and repositories, which side runs, and the sample number
 
 @returns the timing and exit status of the sample
 */
async function runSample({
  prepare,
  command,
  root,
  directRepository,
  wrappedRepository,
  wrapped,
  sample,
}: Pick<ScenarioSetting, 'prepare' | 'command' | 'root' | 'directRepository' | 'wrappedRepository'> & {
  wrapped: boolean;
  sample: number
}): Promise<TimedSample> {
  const directory = wrapped ? wrappedRepository : directRepository;
  await prepare?.({
    directory,
    sample,
  });
  const result = await timed({
    root,
    wrapped,
    directory,
    arguments_: command({ sample }),
  });
  return {
    wrapped,
    sample,
    result,
  };
}

/**
 Run every sample of a scenario one at a time, alternating which side goes first so cache order cancels.
 Each yielded sample settles before the next one starts, because overlapping Git processes would distort the timings.

 @param setting - the scenario to sample
 
 @returns the samples in the order they ran
 */
async function* orderedSamples(setting: ScenarioSetting): AsyncGenerator<TimedSample> {
  for (let sample = 0; sample < setting.samples; sample += 1) {
    const order = (sample % 2) === 0
      ? [
        false,
        true,
      ]
      : [
        true,
        false,
      ];
    for (const wrapped of order) {
      yield runSample({
        ...setting,
        wrapped,
        sample,
      });
    }
  }
}

/**
 Pair a direct and a wrapped sample, alternating which goes first so cache order cancels.

 @param setting -
   scenario name, the optional per-sample setup that runs before the timer starts, the Git arguments of a sample,
   whether the forwarded-command budget applies, the sample count, the scratch root, and both repositories
 
 @returns failed samples and the scenario report
 */
export async function measure(setting: ScenarioSetting): Promise<ScenarioMeasurement> {
  const direct: number[] = [];
  const wrappedSamples: number[] = [];
  const failures: SampleFailure[] = [];
  for await (const {
    wrapped,
    sample,
    result,
  } of orderedSamples(setting)) {
    if (result.status !== 0) {
      failures.push({
        scenario: setting.name,
        wrapped,
        sample,
        status: result.status,
        stderr: result.stderr,
      });
      continue;
    }
    (wrapped ? wrappedSamples : direct).push(result.elapsed);
  }
  if ((direct.length === 0) || (wrappedSamples.length === 0)) {
    return {
      failures,
      entry: undefined,
    };
  }
  return {
    failures,
    entry: {
      scenario: setting.name,
      enforced: setting.enforced,
      direct: summary(direct),
      wrapped: summary(wrappedSamples),
      addedMedianMilliseconds: median(wrappedSamples) - median(direct),
      addedP95Milliseconds: percentile95(wrappedSamples) - percentile95(direct),
      rawDirect: direct,
      rawWrapped: wrappedSamples,
    },
  };
}
