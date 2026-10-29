import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { NOT_MEASURED, } from './constants.ts';
import {
  chipletWatts,
  NO_READING,
  parseAverageMhz,
  samplePower,
  wattsBetween,
} from './power.ts';
import { makeHost, } from './test-fixture.ts';
import { discoverSensors, } from './hwmon.ts';
import type { EnergyReading, } from './types.ts';

/**
 Sixteen cores at the given counter value.
 */
function allCores(microjoules: number,): readonly number[] {
  return Array.from({ length: 16, }, function sameValue(): number {
    return microjoules;
  },);
}

await describe({
  name: 'power',
  children: [
    describe({
      name: wattsBetween.name,
      children: [
        it({
          name: 'converts a counter step over time to watts',
          fn: async function convertsStep() {
            expect(wattsBetween({ fromMicrojoules: 0, toMicrojoules: 100_000_000, fromAt: 0, toAt: 2_000, },),).toBe(50,);
          },
        },),
        it({
          name: 'answers NOT_MEASURED when the counter went backwards or has no earlier value',
          fn: async function notMeasured() {
            expect(wattsBetween({ fromMicrojoules: 10, toMicrojoules: 5, fromAt: 0, toAt: 2_000, },),).toBeNaN();
            expect(wattsBetween({ fromMicrojoules: NOT_MEASURED, toMicrojoules: 5, fromAt: 0, toAt: 2_000, },),).toBeNaN();
          },
        },),
      ],
    },),
    describe({
      name: parseAverageMhz.name,
      children: [
        it({
          name: 'averages the cpu MHz lines and rounds',
          fn: async function averages() {
            expect(parseAverageMhz('processor\t: 0\ncpu MHz\t\t: 4000.0\ncpu MHz\t\t: 4001.0\n',),).toBe(4_001,);
          },
        },),
        it({
          name: 'answers NOT_MEASURED when no line has a clock',
          fn: async function noClock() {
            expect(parseAverageMhz('model name\t: x\n',),).toBeNaN();
          },
        },),
      ],
    },),
    describe({
      name: chipletWatts.name,
      children: [
        it({
          name: 'sums the core growth of the requested cores',
          fn: async function sumsCores() {
            /**
             Counters at time 0.
             */
            const previous: EnergyReading = { socketMicrojoules: 0, coreMicrojoules: allCores(0,), at: 0, };
            /**
             Counters two seconds later, 2 J per core.
             */
            const current: EnergyReading = { socketMicrojoules: 0, coreMicrojoules: allCores(2_000_000,), at: 2_000, };
            expect(chipletWatts({ cores: [0, 1, 2, 3,], previous, current, },),).toBe(4,);
          },
        },),
        it({
          name: 'answers NOT_MEASURED without 16 earlier core counters',
          fn: async function noEarlierCores() {
            expect(chipletWatts({ cores: [0,], previous: NO_READING, current: NO_READING, },),).toBeNaN();
          },
        },),
        it({
          name: 'answers NOT_MEASURED when any requested core counter went backwards',
          fn: async function counterBackwards() {
            /**
             Earlier counters.
             */
            const previous: EnergyReading = { socketMicrojoules: 0, coreMicrojoules: allCores(5,), at: 0, };
            /**
             Counters with core 1 reset.
             */
            const current: EnergyReading = {
              socketMicrojoules: 0,
              coreMicrojoules: allCores(5,).map(function reset(value, core,): number {
                return core === 1 ? 0 : value + 1;
              },),
              at: 2_000,
            };
            expect(chipletWatts({ cores: [0, 1,], previous, current, },),).toBeNaN();
          },
        },),
      ],
    },),
    describe({
      name: samplePower.name,
      children: [
        it({
          name: 'reads watts from fixture counters on the second pass and nothing on the first',
          fn: async function secondPassWatts() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            /**
             Sensors discovered in the fixture.
             */
            const sensors = await discoverSensors(paths.hwmonRoot,);
            /**
             First pass has no earlier counters.
             */
            const first = await samplePower({ sensors, procRoot: paths.procRoot, previous: NO_READING, at: 0, },);
            expect(first.watts,).toBeNaN();
            expect(first.mhz,).toBe(4_450,);
            /**
             Second pass, two seconds later with unchanged counters.
             */
            const second = await samplePower({ sensors, procRoot: paths.procRoot, previous: first.reading, at: 2_000, },);
            expect(second.watts,).toBe(0,);
            expect(second.chiplet1Watts,).toBe(0,);
            expect(second.chiplet2Watts,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
