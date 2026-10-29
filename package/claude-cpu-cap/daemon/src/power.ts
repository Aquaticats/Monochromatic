import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  ABSENT,
  CORES_PER_CHIPLET,
  EXPECTED_CORES,
  MICRO_PER_UNIT,
  MS_PER_SECOND,
  NOT_MEASURED,
} from './constants.ts';
import type {
  EnergyReading,
  PowerSample,
  Sensors,
} from './types.ts';

/**
 Line prefix of a per-thread clock in `/proc/cpuinfo`.
 */
const CPU_MHZ_PREFIX = 'cpu MHz';

/**
 Energy counter reading used before the first pass; its NaN time makes every comparison with it false.
 */
export const NO_READING: EnergyReading = {
  socketMicrojoules: NOT_MEASURED,
  coreMicrojoules: [],
  at: NOT_MEASURED,
};

/**
 Converts a counter difference into watts.

 
 @param fromMicrojoules - earlier counter value
 
 @param toMicrojoules - later counter value
 
 @param fromAt - earlier read time in milliseconds
 
 @param toAt - later read time in milliseconds

 @returns watts, or {@link NOT_MEASURED} when the counter went backwards or a value is missing

 @example
 ```ts
 wattsBetween({ fromMicrojoules: 0, toMicrojoules: 100_000_000, fromAt: 0, toAt: 2_000, },); // 50
 ```
 */
export function wattsBetween({
  fromMicrojoules,
  toMicrojoules,
  fromAt,
  toAt,
}: Readonly<{
  fromMicrojoules: number;
  toMicrojoules: number;
  fromAt: number;
  toAt: number;
}>,): number {
  if (!(toMicrojoules >= fromMicrojoules)) return NOT_MEASURED;
  return (toMicrojoules - fromMicrojoules) / (((toAt - fromAt) / MS_PER_SECOND) * MICRO_PER_UNIT);
}

/**
 Averages the `cpu MHz` lines of a `/proc/cpuinfo` text.

 @param cpuinfo - contents of `/proc/cpuinfo`

 @returns the rounded average clock, or {@link NOT_MEASURED} when no line has a clock

 @example
 ```ts
 parseAverageMhz('cpu MHz\t\t: 4000.0\ncpu MHz\t\t: 5000.0\n',); // 4500
 ```
 */
export function parseAverageMhz(cpuinfo: string,): number {
  /**
   Per-thread clocks in MHz.
   */
  const clocks = cpuinfo.split('\n',)
    .filter(function isClockLine(line,) {
      return line.startsWith(CPU_MHZ_PREFIX,);
    },)
    .map(function clockOf(line,) {
      return Number(line.slice(line.indexOf(':',) + 1,),);
    },);
  if (clocks.length === 0) return NOT_MEASURED;
  return Math.round(clocks.reduce(
    function sum(
      total,
      mhz,
    ) {
      return total + mhz;
    },
    0,
  ) / clocks.length,);
}

/**
 Sums the per-core counter growth of a core range into watts.

 
 @param cores - core numbers of the chiplet
 
 @param previous - counters from the previous pass
 
 @param current - counters from this pass

 @returns chiplet watts, or {@link NOT_MEASURED} when any counter went backwards

 @example
 ```ts
 chipletWatts({ cores: [0, 1,], previous, current, },); // 12.5
 ```
 */
export function chipletWatts({
  cores,
  previous,
  current,
}: Readonly<{
  cores: readonly number[];
  previous: EnergyReading;
  current: EnergyReading;
}>,): number {
  if (previous.coreMicrojoules
    .length
    !== EXPECTED_CORES) return NOT_MEASURED;
  /**
   Watts of each core in the chiplet.
   */
  const parts = cores.map(function coreWatts(core,) {
    return wattsBetween({
      fromMicrojoules: previous.coreMicrojoules[core] ?? NOT_MEASURED,
      toMicrojoules: current.coreMicrojoules[core] ?? NOT_MEASURED,
      fromAt: previous.at,
      toAt: current.at,
    },);
  },);
  return parts.some(function isMissing(watts,) {
    return Number.isNaN(watts,);
  },) ? NOT_MEASURED : parts.reduce(
    function sum(
      total,
      watts,
    ) {
      return total + watts;
    },
    0,
  );
}

/**
 Reads the energy counters and clocks and derives watts against the previous reading.

 
 @param sensors - discovered sensor files
 
 @param procRoot - proc filesystem directory
 
 @param previous - counters from the previous pass
 
 @param at - `performance.now()` for this read

 @returns this pass's power sample and the counters to keep

 @example
 ```ts
 await samplePower({ sensors, procRoot: '/proc', previous: NO_READING, at: performance.now(), },);
 ```
 */
export async function samplePower({
  sensors,
  procRoot,
  previous,
  at,
}: Readonly<{
  sensors: Sensors;
  procRoot: string;
  previous: EnergyReading;
  at: number;
}>,): Promise<PowerSample> {
  /**
   Socket energy counter in microjoules, not measured without the sensor.
   */
  const socketMicrojoules = sensors.socketEnergyFile === ABSENT
    ? NOT_MEASURED
    : Number(await readFile(
      sensors.socketEnergyFile,
      'utf8',
    ),);
  /**
   Per-core energy counters in microjoules, empty unless every core has one.
   */
  const coreMicrojoules = sensors.coreEnergyFiles
    .length
    === EXPECTED_CORES
    ? await Promise.all(sensors.coreEnergyFiles
      .map(async function readCore(file,) {
        return Number(await readFile(
          file,
          'utf8',
        ),);
      },),)
    : [];
  /**
   Counters and read time to keep for the next pass.
   */
  const reading: EnergyReading = {
    socketMicrojoules,
    coreMicrojoules,
    at,
  };
  /**
   Core numbers 0 to 7.
   */
  const firstChiplet = Array.from(
    { length: CORES_PER_CHIPLET, },
    function coreNumber(
      _unused,
      index,
    ) {
      return index;
    },
  );
  /**
   Core numbers 8 to 15.
   */
  const secondChiplet = firstChiplet.map(function shifted(core,) {
    return core + CORES_PER_CHIPLET;
  },);
  return {
    reading,
    watts: wattsBetween({
      fromMicrojoules: previous.socketMicrojoules,
      toMicrojoules: socketMicrojoules,
      fromAt: previous.at,
      toAt: at,
    },),
    chiplet1Watts: chipletWatts({
      cores: firstChiplet,
      previous,
      current: reading,
    },),
    chiplet2Watts: chipletWatts({
      cores: secondChiplet,
      previous,
      current: reading,
    },),
    mhz: parseAverageMhz(await readFile(
      join(
        procRoot,
        'cpuinfo',
      ),
      'utf8',
    ),),
  };
}
