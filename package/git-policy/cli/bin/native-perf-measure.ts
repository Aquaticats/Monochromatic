/**
 Paired direct-versus-wrapped sampling for one scenario of the latency benchmark in `native-perf.ts`.
 */
import { join } from 'node:path';
import { setTimeout as pause } from 'node:timers/promises';

import { timed } from './native-perf-git.ts';
import { evictRepositoryCache } from './native-perf-shape.ts';
import {
  median,
  pairedDeltas,
  pairedDeltaSummary,
  percentile95,
  type RoundSample,
  summary,
} from './native-perf-statistics.ts';

/**
 Which latency requirement a scenario is held to: a forwarded command by the wrapper-added p95, a commit by the maximum
 of its paired deltas against native Git. See `native-perf-judge.ts`.
 */
export type ScenarioKind = 'forwarded' | 'commit';

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
  readonly kind: ScenarioKind;
  readonly direct: ReturnType<typeof summary>;
  readonly wrapped: ReturnType<typeof summary>;
  readonly addedMedianMilliseconds: number;
  readonly addedP95Milliseconds: number;
  /**
   Median, p95 and maximum of the wrapped sample minus the direct sample of the same round, over the rounds in which both
   sides succeeded; the commit requirement is judged on its maximum.
   */
  readonly pairedDelta: ReturnType<typeof pairedDeltaSummary>;
  readonly rawDirect: readonly number[];
  readonly rawWrapped: readonly number[];
  readonly rawPairedDeltas: readonly number[];
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
  readonly kind: ScenarioKind;
  readonly samples: number;
  readonly root: string;
  readonly directRepository: string;
  readonly wrappedRepository: string;
  /**
   Runtime directory of the commit service the wrapped side is pointed at; absent for a forwarded command, which needs none.
   */
  readonly runtime?: string;
  /**
   Directory that receives one `strace` record per sample, named after the scenario, the round and the side; absent runs
   every sample untraced. A traced sample is slower, so its counts and per-call sums are evidence, never its elapsed time.
   */
  readonly traceDirectory?: string;
  /**
   Whether the pack files, pack indexes and index of each repository are dropped from the page cache before each sample, on
   both sides alike.
   */
  readonly cold?: boolean;
  /**
   Milliseconds to wait after both sides of a round, so the work a commit leaves running after its answer (the service's queued
   syncs) is over before the next round starts; absent starts the next round at once.
   */
  readonly pauseMilliseconds?: number;
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

 @param prepare - scenario's optional per-sample setup

 @param command - scenario's Git arguments for a sample

 @param root - scratch root that holds `bin` and `home`

 @param directRepository - repository of the direct run

 @param wrappedRepository - repository of the wrapped run

 @param wrapped - which side runs

 @param sample - sample number

 @returns the timing and exit status of the sample
 */
async function runSample({
  prepare,
  command,
  root,
  directRepository,
  wrappedRepository,
  runtime,
  traceDirectory,
  cold,
  pauseMilliseconds,
  name,
  wrapped,
  sample,
}: Pick<
  ScenarioSetting,
  'prepare' | 'command' | 'root' | 'directRepository' | 'wrappedRepository' | 'runtime' | 'traceDirectory' | 'cold' | 'pauseMilliseconds' | 'name'
  > & {
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
  if (cold === true) {
    await evictRepositoryCache(directory);
  }
  /**
   Timing and exit state of the sample, measured after the preparation finished.
   */
  const result = await timed({
    root,
    wrapped,
    ...(runtime === undefined ? {} : { runtime, }),
    directory,
    arguments_: command({ sample }),
    ...(traceDirectory === undefined
      ? {}
      : {
        trace: join(
          traceDirectory,
          `${name}-${String(sample)}-${wrapped ? 'wrapped' : 'direct'}.strace`,
        ),
      }),
  });
  // The work a commit leaves running after its answer is over before the next sample starts, when the run asks for the wait.
  if (pauseMilliseconds !== undefined) {
    await pause(pauseMilliseconds);
  }
  return {
    wrapped,
    sample,
    result,
  };
}

/**
 Run every sample of a scenario one at a time, alternating which side goes first so cache order cancels.
 Each yielded sample settles before the next one starts, because overlapping Git processes would distort the timings.

 @param setting - scenario to sample

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
 Elapsed milliseconds of a sample.

 @param entry - a successful sample

 @returns its elapsed milliseconds
 */
function elapsedOf(entry: RoundSample): number {
  return entry.elapsed;
}

/**
 Statistics and raw samples of one scenario from its successful samples.

 @param scenario - scenario name

 @param kind - which latency requirement applies

 @param direct - successful direct samples with their rounds

 @param wrapped - successful wrapped samples with their rounds

 @returns the report

 @throws SampleSetError when no round has a successful sample on both sides

 @example
 ```ts
 const entry = scenarioReport({ scenario: 'status', kind: 'forwarded', direct, wrapped });
 ```
 */
export function scenarioReport({
  scenario,
  kind,
  direct,
  wrapped,
}: {
  readonly scenario: string;
  readonly kind: ScenarioKind;
  readonly direct: readonly RoundSample[];
  readonly wrapped: readonly RoundSample[];
}): ScenarioReport {
  /**
   Milliseconds of the successful direct samples.
   */
  const directMilliseconds = direct.map(elapsedOf);
  /**
   Milliseconds of the successful wrapped samples.
   */
  const wrappedMilliseconds = wrapped.map(elapsedOf);
  /**
   Wrapped minus direct for every round in which both sides succeeded.
   */
  const deltas = pairedDeltas({
    direct,
    wrapped,
  });
  return {
    scenario,
    kind,
    direct: summary(directMilliseconds),
    wrapped: summary(wrappedMilliseconds),
    addedMedianMilliseconds: median(wrappedMilliseconds) - median(directMilliseconds),
    addedP95Milliseconds: percentile95(wrappedMilliseconds) - percentile95(directMilliseconds),
    pairedDelta: pairedDeltaSummary(deltas),
    rawDirect: directMilliseconds,
    rawWrapped: wrappedMilliseconds,
    rawPairedDeltas: deltas,
  };
}

/**
 Pair a direct and a wrapped sample, alternating which goes first so cache order cancels.

 @param setting -
   scenario name, the optional per-sample setup that runs before the timer starts, the Git arguments of a sample,
   which latency requirement applies, the sample count, the scratch root, and both repositories

 @returns failed samples and the scenario report

 @example
 ```ts
 const { failures, entry } = await measure(setting);
 ```
 */
export async function measure(setting: Readonly<ScenarioSetting>): Promise<ScenarioMeasurement> {
  /**
   Successful direct samples with the round of each.
   */
  const direct: RoundSample[] = [];
  /**
   Successful wrapped samples with the round of each.
   */
  const wrappedSamples: RoundSample[] = [];
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
    (wrapped ? wrappedSamples : direct).push({
      sample,
      elapsed: result.elapsed,
    });
  }
  // A scenario with no round that has both sides has nothing to judge; its failures are already reported.
  /**
   Wrapped minus direct for every round in which both sides succeeded.
   */
  const pairedRounds = pairedDeltas({
    direct,
    wrapped: wrappedSamples,
  });
  if (pairedRounds.length === 0) {
    return { failures, };
  }
  return {
    failures,
    entry: scenarioReport({
      scenario: setting.name,
      kind: setting.kind,
      direct,
      wrapped: wrappedSamples,
    }),
  };
}
