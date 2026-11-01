import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { reconcileCap, } from './cap.ts';
import {
  ABSENT,
  BURST_MS,
  DEFAULT_IO_WEIGHT,
  INTERVAL_MS,
  MILLI_PER_UNIT,
  MIN_THREADS,
  NOT_MEASURED,
  SAMPLE_EVERY_ITERATIONS,
  TONE_DOWN_STEP,
} from './constants.ts';
import { reconcileIoWeight, } from './io-weight.ts';
import { samplePower, } from './power.ts';
import { readPressure, } from './pressure.ts';
import { writeSample, } from './samples.ts';
import { advanceStreak, } from './streak.ts';
import type {
  Limits,
  Metric,
  Paths,
  PowerSample,
  Sensors,
  WatchState,
} from './types.ts';

/**
 Everything a pass needs that does not change between passes.
 */
export type PassInputs = Readonly<{
  /**
   Paths the watcher reads and writes.
   */
  paths: Paths;
  /**
   Discovered sensor locations.
   */
  sensors: Sensors;
  /**
   Alert limits.
   */
  limits: Limits;
  /**
   Tctl in degrees Celsius at which the chip regulates clocks.
   */
  thermalTargetCelsius: number;
  /**
   Clock in MHz with boost off, the reference for the throttle shortfall.
   */
  boostOffMhz: number;
  /**
   Package watts step between passes that starts burst mode.
   */
  burstStepWatts: number;
  /**
   Hardware thread count.
   */
  totalThreads: number;
  /**
   Parent logger.
   */
  l: Logger;
}>;

/**
 Builds the hardware part of an alert line from a power sample.

 @param power - this pass's power sample

 @returns text such as `package 74 W, average clock 4446 MHz`, empty when nothing was measured

 @example
 ```ts
 describeHardware(power,); // 'package 74 W, average clock 4446 MHz'
 ```
 */
export function describeHardware(power: PowerSample,): string {
  /**
   Text pieces of the hardware description.
   */
  const parts: string[] = [];
  if (!Number.isNaN(power.watts,)) parts.push(`package ${power.watts
    .toFixed(0,)} W`,);
  if (!Number.isNaN(power.mhz,)) parts.push(`average clock ${power.mhz} MHz`,);
  return parts.join(', ',);
}

/**
 Records one sample of a metric and logs the alert or clear it causes.

 
 @param state - state before this sample
 
 @param metric - which metric
 
 @param value - the sample value
 
 @param cap - thread cap in force
 
 @param detail - hardware text for the alert line
 
 @param inputs - constant pass inputs

 @returns the state with the metric's streak advanced

 @example
 ```ts
 recordMetric({ state, metric: 'scheduler-lag-ms', value: 12, cap: 28, detail: '', inputs, },);
 ```
 */
export function recordMetric({
  state,
  metric,
  value,
  cap,
  detail,
  inputs,
}: Readonly<{
  state: WatchState;
  metric: Metric;
  value: number;
  cap: number;
  detail: string;
  inputs: PassInputs;
}>,): WatchState {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: recordMetric.name,
    l: inputs.l,
  },);
  /**
   Limit of this metric.
   */
  const limit = inputs.limits[metric];
  /**
   The advanced streak and the event it caused.
   */
  const {
    streak,
    event,
  } = advanceStreak({
    streak: state.streaks[metric],
    isBad: value >= limit,
  },);
  if (event === 'alert') {
    rl.warn(
      `ALERT ${metric}=${value.toFixed(1,)} (limit ${limit}), cap ${cap} threads; ${detail}; `
        + `to tone down write ${Math.max(
          MIN_THREADS,
          cap - TONE_DOWN_STEP,
        )} to ${inputs.paths
          .capFile}`,
    );
  } else if (event === 'clear') {
    rl.info(`CLEAR ${metric}=${value.toFixed(1,)} (limit ${limit}), cap ${cap} threads`,);
  }
  return {
    ...state,
    streaks: {
      ...state.streaks,
      [metric]: streak,
    },
  };
}

/**
 Computes the throttle shortfall: how far the average clock sits below the boost-off clock while Tctl is at the target.
 Counting only at the target keeps idle cores' low clocks from raising the alert.

 
 @param celsius - Tctl in degrees Celsius
 
 @param mhz - average clock in MHz
 
 @param inputs - constant pass inputs

 @returns the shortfall in MHz, 0 below the target

 @example
 ```ts
 throttleShortfall({ celsius: 95, mhz: 4000, inputs, },); // 490 with the default boost-off clock
 ```
 */
export function throttleShortfall({
  celsius,
  mhz,
  inputs,
}: Readonly<{
  celsius: number;
  mhz: number;
  inputs: PassInputs;
}>,): number {
  return celsius >= inputs.thermalTargetCelsius ? Math.max(
    0,
    inputs.boostOffMhz - mhz,
  ) : 0;
}

/**
 Records the thermal throttle metric when both Tctl and the average clock are known.

 @param state - state after the pressure metrics were recorded
 
 @param power - this pass's power sample
 
 @param cap - thread cap in force
 
 @param detail - hardware text for the alert line
 
 @param inputs - constant pass inputs

 @returns the state with the throttle streak advanced, unchanged when Tctl or the clock is unavailable

 @example
 ```ts
 await recordThermal({ state, power, cap: 28, detail: 'package 74 W', inputs, },);
 ```
 */
export async function recordThermal({
  state,
  power,
  cap,
  detail,
  inputs,
}: Readonly<{
  state: WatchState;
  power: PowerSample;
  cap: number;
  detail: string;
  inputs: PassInputs;
}>,): Promise<WatchState> {
  if ((inputs.sensors
    .temperatureFile
    === ABSENT) || Number.isNaN(power.mhz,)) return state;
  /**
   Tctl in degrees Celsius.
   */
  const celsius = Number(await readFile(
    inputs.sensors
      .temperatureFile,
    'utf8',
  ),) / MILLI_PER_UNIT;
  return recordMetric({
    state,
    metric: 'cpu-thermal-throttle-mhz',
    value: throttleShortfall({
      celsius,
      mhz: power.mhz,
      inputs,
    },),
    cap,
    detail: `Tctl ${celsius.toFixed(1,)}, ${detail}`,
    inputs,
  },);
}

/**
 Runs one watcher pass: reconciles the cap, samples power and pressure, advances every streak,
 tracks burst mode and writes a sample row when due.
 IO pressure is deliberately not sampled; instead the cgroup's `io.weight` is lowered every pass so tool commands
 yield the NVMe to the desktop under BFQ (2026-10-29, `doc/planning/agent-search-io-throttle.md`).

 @param state - state from the previous pass
 
 @param inputs - constant pass inputs
 
 @param now - `performance.now()` of this tick

 @returns the state for the next pass

 @example
 ```ts
 state = await runPass({ state, inputs, now: performance.now(), },);
 ```
 */
export async function runPass({
  state,
  inputs,
  now,
}: Readonly<{
  state: WatchState;
  inputs: PassInputs;
  now: number;
}>,): Promise<WatchState> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: runPass.name,
    l: inputs.l,
  },);
  /**
   Thread cap in force after reconciling the cgroup.
   */
  const { threads: cap, } = await reconcileCap({
    capFile: inputs.paths
      .capFile,
    cgroupFile: inputs.paths
      .cgroupFile,
    totalThreads: inputs.totalThreads,
    l: rl,
  },);
  await reconcileIoWeight({
    ioWeightFile: inputs.paths
      .ioWeightFile,
    weight: DEFAULT_IO_WEIGHT,
    l: rl,
  },);
  /**
   Power and clock measurements of this pass.
   */
  const power = await samplePower({
    sensors: inputs.sensors,
    procRoot: inputs.paths
      .procRoot,
    previous: state.reading,
    at: now,
  },);
  /**
   Hardware text shared by the alert lines of this pass.
   */
  const detail = describeHardware(power,);
  /**
   State with this pass's tick bookkeeping applied.
   */
  const ticked: WatchState = {
    ...state,
    reading: power.reading,
    previousTickAt: now,
    iteration: state.iteration + 1,
  };
  /**
   State after the scheduler lag sample.
   */
  const afterLag = recordMetric({
    state: ticked,
    metric: 'scheduler-lag-ms',
    value: Math.max(
      0,
      now - state.previousTickAt
        - INTERVAL_MS,
    ),
    cap,
    detail,
    inputs,
  },);
  /**
   State after the CPU pressure sample.
   */
  const afterCpu = recordMetric({
    state: afterLag,
    metric: 'cpu-pressure-some-avg10',
    value: await readPressure({
      file: join(
        inputs.paths
          .procRoot,
        'pressure',
        'cpu',
      ),
      kind: 'some',
    },),
    cap,
    detail,
    inputs,
  },);
  /**
   State after the memory pressure sample.
   */
  const afterMemory = recordMetric({
    state: afterCpu,
    metric: 'memory-pressure-some-avg10',
    value: await readPressure({
      file: join(
        inputs.paths
          .procRoot,
        'pressure',
        'memory',
      ),
      kind: 'some',
    },),
    cap,
    detail,
    inputs,
  },);
  return writeDue({
    state: await recordThermal({
      state: afterMemory,
      power,
      cap,
      detail,
      inputs,
    },),
    inputs,
    power,
    cap,
    now,
  },);
}

/**
 Updates burst mode from the package power step and writes a sample row when one is due.

 
 @param state - state after the metrics were recorded
 
 @param inputs - constant pass inputs
 
 @param power - this pass's power sample
 
 @param cap - thread cap in force
 
 @param now - `performance.now()` of this tick

 @returns the state with burst and sample bookkeeping updated

 @example
 ```ts
 state = await writeDue({ state, inputs, power, cap: 28, now, },);
 ```
 */
export async function writeDue({
  state,
  inputs,
  power,
  cap,
  now,
}: Readonly<{
  state: WatchState;
  inputs: PassInputs;
  power: PowerSample;
  cap: number;
  now: number;
}>,): Promise<WatchState> {
  /**
   Whether package power moved by more than the burst step since the previous pass.
   */
  const stepped = Math.abs(power.watts - state.previousPassWatts,) > inputs.burstStepWatts;
  /**
   Time until which every pass is sampled.
   */
  const burstUntil = stepped ? now + BURST_MS : state.burstUntil;
  /**
   Package watts to compare the next pass against.
   */
  const previousPassWatts = Number.isNaN(power.watts,) ? state.previousPassWatts : power.watts;
  /**
   Whether burst mode is on.
   */
  const burst = now < burstUntil;
  if (((state.iteration % SAMPLE_EVERY_ITERATIONS) !== 0) && (!burst)) {
    return {
      ...state,
      burstUntil,
      previousPassWatts,
    };
  }
  /**
   Local date of the sample file written.
   */
  const sampleDay = await writeSample({
    paths: inputs.paths,
    sensors: inputs.sensors,
    power,
    cap,
    burst,
    sampleDay: state.sampleDay,
  },);
  return {
    ...state,
    burstUntil,
    previousPassWatts,
    sampleDay,
  };
}

/**
 State before the first pass: every metric quiet, no energy reading, no burst.
 */
export const INITIAL_STATE_BASE: Omit<WatchState, 'streaks' | 'reading' | 'previousTickAt'> = {
  iteration: 0,
  burstUntil: 0,
  previousPassWatts: NOT_MEASURED,
  sampleDay: '',
};
