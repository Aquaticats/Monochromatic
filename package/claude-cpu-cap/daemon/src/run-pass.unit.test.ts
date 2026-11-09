import { mkdir, readdir, readFile, rm, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { DEFAULT_BOOST_OFF_MHZ, DEFAULT_IO_WEIGHT, DEFAULT_THERMAL_TARGET_CELSIUS, INTERVAL_MS, PERIOD_USEC, } from './constants.ts';
import { discoverSensors, } from './hwmon.ts';
import { resolveLimits, resolveThermalTuning, } from './paths.ts';
import { NO_READING, } from './power.ts';
import {
  describeHardware,
  INITIAL_STATE_BASE,
  type PassInputs,
  runPass,
  throttleShortfall,
} from './run-pass.ts';
import { INITIAL_STREAKS, } from './streak.ts';
import {
  makeHost,
  recordingLogger,
  type RecordingLogger,
} from './test-fixture.ts';
import type { WatchState, } from './types.ts';

/**
 Builds pass inputs for a fresh disposable host, which removes itself when the
 caller's scope ends.

 @param env - `CPU_WATCH_LIMIT_*` overrides
 @param cpuPressure - CPU pressure written into the fixture

 @returns the inputs, the logger recording their output, the host root, and an
   async disposer that removes the host
 */
async function makeInputs({ env, cpuPressure, }: Readonly<{
  env: Readonly<Record<string, string>>;
  cpuPressure: number;
}>,): Promise<Readonly<{
  inputs: PassInputs;
  l: RecordingLogger;
  root: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}>> {
  /**
   Disposable host.
   */
  const host = await makeHost({ threads: '28', cpuPressure, },);
  const { root, paths, } = host;
  /**
   Recorder for the logged lines.
   */
  const l = recordingLogger();
  return {
    root,
    l,
    inputs: {
      paths,
      sensors: await discoverSensors(paths.hwmonRoot,),
      limits: resolveLimits(env,),
      ...resolveThermalTuning(env,),
      totalThreads: 32,
      l,
    },
    [Symbol.asyncDispose]: async function removeHostFixture(): Promise<void> {
      await host[Symbol.asyncDispose]();
    },
  };
}

/**
 State before the first pass.
 */
const FRESH: WatchState = { ...INITIAL_STATE_BASE, streaks: INITIAL_STREAKS, reading: NO_READING, previousTickAt: 0, };

/**
 Runs several passes one interval apart.

 @param inputs - pass inputs
 @param passes - number of passes
 @param start - state to start from

 @returns the state after the last pass
 */
async function runPasses({ inputs, passes, start, }: Readonly<{
  inputs: PassInputs;
  passes: number;
  start: WatchState;
}>,): Promise<WatchState> {
  return Array.from({ length: passes, }, function passNumber(_unused, index,): number {
    return index + 1;
  },).reduce(async function next(previous: Promise<WatchState>, pass,): Promise<WatchState> {
    return runPass({ state: await previous, inputs, now: pass * INTERVAL_MS, },);
  }, Promise.resolve(start,),);
}

await describe({
  name: 'run-pass',
  children: [
    it({
      name: 'describeHardware lists what was measured',
      fn: async function describes() {
        /**
         A power sample with watts and clock only.
         */
        const base = { reading: NO_READING, watts: 74.8, chiplet1Watts: Number.NaN, chiplet2Watts: Number.NaN, mhz: 4_446, };
        expect(describeHardware(base,),).toBe('package 75 W, average clock 4446 MHz',);
        expect(describeHardware({ ...base, watts: Number.NaN, mhz: Number.NaN, },),).toBe('',);
      },
    },),
    it({
      name: 'throttleShortfall counts the clock deficit only at the thermal target',
      fn: async function shortfall() {
        /**
         Inputs with the default tuning.
         */
        await using fixture = await makeInputs({ env: {}, cpuPressure: 0, },);
        const { inputs, } = fixture;
        expect(throttleShortfall({ celsius: DEFAULT_THERMAL_TARGET_CELSIUS, mhz: DEFAULT_BOOST_OFF_MHZ - 490, inputs, },),).toBe(490,);
        expect(throttleShortfall({ celsius: 94.9, mhz: 1_000, inputs, },),).toBe(0,);
        expect(throttleShortfall({ celsius: DEFAULT_THERMAL_TARGET_CELSIUS, mhz: 4_600, inputs, },),).toBe(0,);
      },
    },),
    it({
      name: 'quiet passes apply the cap, write one sample on the fifth pass and log no alert',
      fn: async function quietPasses() {
        /**
         Inputs for a healthy host.
         */
        await using fixture = await makeInputs({ env: {}, cpuPressure: 0, },);
        const { inputs, l, root, } = fixture;
        /**
         State after five passes.
         */
        const state = await runPasses({ inputs, passes: 5, start: FRESH, },);
        expect(state.iteration,).toBe(5,);
        expect(await readFile(join(root, 'cgroup', 'cpu.max',), 'utf8',),).toBe(`${28 * PERIOD_USEC} ${PERIOD_USEC}`,);
        expect(await readFile(join(root, 'cgroup', 'io.weight',), 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
        expect(await readFile(join(root, 'cgroup', 'io.bfq.weight',), 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
        expect(l.messages.warn,).toEqual([],);
        expect(l.messages.error,).toEqual([],);
        /**
         Daily sample files written under the fixture.
         */
        const files = await readdir(inputs.paths.sampleDir,);
        expect(files,).toHaveLength(1,);
        expect((await readFile(join(inputs.paths.sampleDir, files[0] ?? '',), 'utf8',)).trimEnd().split('\n',),).toHaveLength(2,);
      },
    },),
    it({
      name: 'a cgroup without any weight file logs IO-WEIGHT-UNAVAILABLE but the pass still applies the cap and advances',
      fn: async function noWeightFiles() {
        /**
         Inputs for a host whose cgroup directory has no weight file.
         */
        await using fixture = await makeInputs({ env: {}, cpuPressure: 0, },);
        const { inputs, l, root, } = fixture;
        await rm(join(root, 'cgroup', 'io.weight',),);
        await rm(join(root, 'cgroup', 'io.bfq.weight',),);
        /**
         State after one pass.
         */
        const state = await runPass({ state: FRESH, inputs, now: INTERVAL_MS, },);
        expect(state.iteration,).toBe(1,);
        expect(await readFile(join(root, 'cgroup', 'cpu.max',), 'utf8',),).toBe(`${28 * PERIOD_USEC} ${PERIOD_USEC}`,);
        expect(l.messages.warn,).toHaveLength(1,);
        expect(l.messages.warn[0],).toContain('IO-WEIGHT-UNAVAILABLE',);
        expect(l.messages.error,).toEqual([],);
      },
    },),
    it({
      name: 'a weight file that fails logs IO-WEIGHT-FAILED but the pass still applies the cap and the other weight',
      fn: async function failingWeightFile() {
        /**
         Inputs for a host whose io.weight cannot be read as a file.
         */
        await using fixture = await makeInputs({ env: {}, cpuPressure: 0, },);
        const { inputs, l, root, } = fixture;
        await rm(join(root, 'cgroup', 'io.weight',),);
        await mkdir(join(root, 'cgroup', 'io.weight',),);
        /**
         State after one pass.
         */
        const state = await runPass({ state: FRESH, inputs, now: INTERVAL_MS, },);
        expect(state.iteration,).toBe(1,);
        expect(await readFile(join(root, 'cgroup', 'cpu.max',), 'utf8',),).toBe(`${28 * PERIOD_USEC} ${PERIOD_USEC}`,);
        expect(await readFile(join(root, 'cgroup', 'io.bfq.weight',), 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
        expect(l.messages.error,).toHaveLength(1,);
        expect(l.messages.error[0],).toContain('IO-WEIGHT-FAILED',);
      },
    },),
    it({
      name: 'positive control: a CPU pressure limit of 0 raises ALERT on the third pass, and CLEAR after recovery',
      fn: async function alertsAndClears() {
        /**
         Inputs whose CPU pressure limit is 0, so any pressure is bad.
         */
        await using fixture = await makeInputs({ env: { CPU_WATCH_LIMIT_CPU_SOME: '0', }, cpuPressure: 1, },);
        const { inputs, l, root, } = fixture;
        /**
         State after three passes.
         */
        const alerting = await runPasses({ inputs, passes: 3, start: FRESH, },);
        expect(l.messages.warn,).toHaveLength(1,);
        expect(l.messages.warn[0],).toContain('ALERT cpu-pressure-some-avg10=1.0 (limit 0)',);
        expect(l.messages.warn[0],).toContain(`to tone down write 26 to ${inputs.paths.capFile}`,);
        expect(alerting.streaks['cpu-pressure-some-avg10'].alerting,).toBe(true,);
        // A limit above the pressure is the negative control: the same pressure no longer counts as bad.
        await writeFile(join(root, 'proc', 'pressure', 'cpu',), 'some avg10=0.00 avg60=0 avg300=0 total=0\n',);
        const quietLimits = { ...inputs, limits: resolveLimits({},), };
        await runPasses({ inputs: quietLimits, passes: 5, start: alerting, },);
        expect(l.messages.info.some(function isClear(line,): boolean {
          return line.includes('CLEAR cpu-pressure-some-avg10',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'a thermal throttle alert needs Tctl at the target and a clock far below the boost-off clock',
      fn: async function throttleAlert() {
        /**
         Inputs with the thermal target lowered to the fixture's 61.5 degrees and a boost-off clock 300 MHz above it.
         */
        await using fixture = await makeInputs({
          env: { CPU_WATCH_LIMIT_TEMP: '61', CPU_WATCH_LIMIT_BOOST_OFF_MHZ: '4750', },
          cpuPressure: 0,
        },);
        const { inputs, l, } = fixture;
        await runPasses({ inputs, passes: 3, start: FRESH, },);
        expect(l.messages.warn[0],).toContain('ALERT cpu-thermal-throttle-mhz=300.0 (limit 300)',);
        expect(l.messages.warn[0],).toContain('Tctl 61.5',);
      },
    },),
    it({
      name: 'a power step larger than the burst step turns burst mode on for the following passes',
      fn: async function burstMode() {
        /**
         Inputs for a healthy host.
         */
        await using fixture = await makeInputs({ env: {}, cpuPressure: 0, },);
        const { inputs, root, } = fixture;
        /**
         State after two passes with a flat counter, giving a known 0 W reading.
         */
        const flat = await runPasses({ inputs, passes: 2, start: FRESH, },);
        expect(flat.burstUntil,).toBe(0,);
        // 100 J over the next 2 s interval is 50 W, a step of 50 W from 0 W.
        await writeFile(join(root, 'hwmon', 'hwmon1', 'energy1_input',), '101000000\n',);
        /**
         State after the pass that sees the step.
         */
        const stepped = await runPass({ state: flat, inputs, now: 3 * INTERVAL_MS, },);
        expect(stepped.burstUntil,).toBeGreaterThan(3 * INTERVAL_MS,);
        expect(stepped.previousPassWatts,).toBe(50,);
      },
    },),
    it({
      name: 'a pass that stalls beyond the interval raises scheduler lag',
      fn: async function lagAlert() {
        /**
         Inputs with the lag limit at 100 ms.
         */
        await using fixture = await makeInputs({ env: { CPU_WATCH_LIMIT_LAG: '100', }, cpuPressure: 0, },);
        const { inputs, l, } = fixture;
        /**
         State after the first pass.
         */
        const first = await runPass({ state: FRESH, inputs, now: INTERVAL_MS, },);
        /**
         State after three passes that each arrive 500 ms late.
         */
        const lateTicks = [1, 2, 3,].reduce(async function late(previous: Promise<WatchState>, pass,): Promise<WatchState> {
          return runPass({ state: await previous, inputs, now: INTERVAL_MS + (pass * (INTERVAL_MS + 500)), },);
        }, Promise.resolve(first,),);
        await lateTicks;
        expect(l.messages.warn[0],).toContain('ALERT scheduler-lag-ms=500.0',);
      },
    },),
  ],
},);
