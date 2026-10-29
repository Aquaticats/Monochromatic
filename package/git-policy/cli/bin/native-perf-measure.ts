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
  readonly scenario: string;
  readonly wrapped: boolean;
  readonly sample: number;
  readonly status?: number;
  readonly stderr: string;
};

/**
 One scenario's statistics and raw samples.
 */
export type ScenarioReport = {
  readonly scenario: string;
  readonly enforced: boolean;
  readonly direct: ReturnType<typeof summary>;
  readonly wrapped: ReturnType<typeof summary>;
  readonly addedMedianMilliseconds: number;
  readonly addedP95Milliseconds: number;
  readonly rawDirect: readonly number[];
  readonly rawWrapped: readonly number[];
};

/**
 What one scenario measured: every failed sample, and the report unless either side has no successful sample.
 */
export type ScenarioMeasurement = {
  readonly failures: readonly SampleFailure[];
  readonly entry?: ScenarioReport;
};

/**
 Everything one scenario needs to run its samples.
 */
export type ScenarioSetting = {
  readonly name: string;
  readonly prepare?: (request: {
    readonly directory: string;
    readonly sample: number;
  }) => Promise<void>;
  readonly command: (request: { readonly sample: number; }) => readonly string[];
  readonly enforced: boolean;
  readonly samples: number;
  readonly root: string;
  readonly directRepository: string;
  readonly wrappedRepository: string;
};

/**
 One timed sample and which side and round it belongs to.
 */
export type TimedSample = {
  readonly wrapped: boolean;
  readonly sample: number;
  readonly result: Awaited<ReturnType<typeof timed>>;
};

/**
 Prepare and time one sample; the preparation runs before the timer starts, so it never counts as latency.

 @param prepare - the scenario's optional per-sample setup

 @param command - the scenario's Git arguments for a sample

 @param root - scratch root that holds `bin` and `home`

 @param directRepository - repository of the direct run

 @param wrappedRepository - repository of the wrapped run

 @param wrapped - which side runs

 @param sample - the sample number

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
  readonly wrapped: boolean;
  readonly sample: number;
}): Promise<TimedSample> {
  /**
   Repository of the side that runs, so the two sides never share index state.
   */
  const directory = wrapped ? wrappedRepository : directRepository;
  await prepare?.({
    directory,
    sample,
  });
  /**
   Timing and exit state of the sample, measured after the preparation finished.
   */
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
async function* orderedSamples(setting: Readonly<ScenarioSetting>): AsyncGenerator<TimedSample> {
  for (let sample = 0; sample < setting.samples; sample += 1) {
    /**
     Which side runs first in this round, flipped every round so cache order cancels.
     */
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

 @example
 ```ts
 const { failures, entry } = await measure(setting);
 ```
 */
export async function measure(setting: Readonly<ScenarioSetting>): Promise<ScenarioMeasurement> {
  /**
   Milliseconds of the successful direct samples.
   */
  const direct: number[] = [];
  /**
   Milliseconds of the successful wrapped samples.
   */
  const wrappedSamples: number[] = [];
  /**
   Samples whose Git command exited nonzero.
   */
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
        ...(result.status === undefined ? {} : { status: result.status, }),
        stderr: result.stderr,
      });
      continue;
    }
    (wrapped ? wrappedSamples : direct).push(result.elapsed);
  }
  if ((direct.length === 0) || (wrappedSamples.length === 0)) {
    return { failures, };
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
