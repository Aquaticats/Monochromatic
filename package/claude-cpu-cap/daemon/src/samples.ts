import {
  appendFile,
  mkdir,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { readPressure, } from './pressure.ts';
import {
  ABSENT,
  BOARD_SENSOR_INDEXES,
  LEGACY_CORSAIR_COLUMN_COUNT,
  MB_FAN_INPUTS,
  MB_TEMPERATURE_INPUTS,
  MS_PER_DAY,
  SAMPLE_HEADER,
  SAMPLE_RETENTION_DAYS,
} from './constants.ts';
import {
  readMicroWatts,
  readMilliCelsius,
  readRpm,
  readTextOrEmpty,
} from './files.ts';
import type {
  Paths,
  PowerSample,
  Sensors,
} from './types.ts';

/**
 Suffix of a daily sample file name (`2026-10-13.csv`).
 */
const CSV_SUFFIX = '.csv';

/**
 Length of the `YYYY-MM-DD` part of a sample file name.
 */
const DAY_LENGTH = 10;

/**
 Formats a number with fixed decimals, or an empty field when it was not measured.

 
 @param value - number to format, possibly {@link NOT_MEASURED}
 
 @param digits - decimals

 @returns the CSV field

 @example
 ```ts
 fixedField({ value: Number.NaN, digits: 1, },); // ''
 ```
 */
export function fixedField({
  value,
  digits,
}: Readonly<{
  value: number;
  digits: number
}>,): string {
  return Number.isNaN(value,) ? ABSENT : value.toFixed(digits,);
}

/**
 Removes sample files older than the retention window.

 @param sampleDir - directory of the daily CSV files
 
 @param now - current time in epoch milliseconds

 @example
 ```ts
 await pruneSamples({ sampleDir, now: Date.now(), },);
 ```
 */
export async function pruneSamples({
  sampleDir,
  now,
}: Readonly<{
  sampleDir: string;
  now: number
}>,): Promise<void> {
  /**
   Epoch milliseconds before which a file is expired.
   */
  const cutoff = now - (SAMPLE_RETENTION_DAYS
    * MS_PER_DAY);
  /**
   File names in the sample directory.
   */
  const names = await readdir(sampleDir,);
  /**
   Names of the expired sample files.
   */
  const expired = names.filter(function isExpired(name,) {
    return name.endsWith(CSV_SUFFIX,) && (Date.parse(`${name.slice(
      0,
      DAY_LENGTH,
    )}T00:00:00`,) < cutoff);
  },);
  await Promise.all(expired.map(async function removeExpired(name,) {
    await rm(join(
      sampleDir,
      name,
    ),);
  },),);
}

/**
 Makes sure the day's file starts with the current header; a file written before columns were added
 keeps its rows (older rows stay shorter) and only the header line is brought up to date.

 @param file - the day's CSV path

 @example
 ```ts
 await ensureHeader('/tmp/samples/2026-10-13.csv',);
 ```
 */
export async function ensureHeader(file: string,): Promise<void> {
  /**
   Current contents of the day's file, empty when it does not exist.
   */
  const text = await readTextOrEmpty(file,);
  if (text === ABSENT) {
    await appendFile(
      file,
      SAMPLE_HEADER,
    );
    return;
  }
  /**
   Lines of the day's file.
   */
  const lines = text.split('\n',);
  if (`${lines[0]}\n` !== SAMPLE_HEADER) {
    await writeFile(
      file,
      [
        SAMPLE_HEADER.trimEnd(),
        ...lines.slice(1,),
      ].join('\n',),
    );
  }
}

/**
 Everything one sample row is built from.
 */
export type SampleInput = Readonly<{
  /**
   ISO UTC time of the row.
   */
  time: string;
  /**
   Thread cap in force.
   */
  cap: number;
  /**
   Power and clock measurements of this pass.
   */
  power: PowerSample;
  /**
   Whether burst mode is on.
   */
  burst: boolean;
  /**
   CPU pressure `some avg10`.
   */
  cpuPressure: number;
  /**
   One-minute load average text.
   */
  load1: string;
  /**
   Sensor locations.
   */
  sensors: Sensors;
}>;

/**
 Builds the CSV fields of one sample row in the order of {@link SAMPLE_HEADER}.
 The seven Corsair columns are written empty, because the controller is no longer read.
 The unlabeled motherboard sensors are logged raw; which of them tracks case or socket temperature is for the data.

 @param input - the row's inputs

 @returns the fields

 @example
 ```ts
 (await buildSampleFields(input,)).length; // 42
 ```
 */
export async function buildSampleFields(input: SampleInput,): Promise<readonly string[]> {
  /**
   Sensor locations and power sample taken from the input.
   */
  const {
    sensors,
    power,
  } = input;
  /**
   Directory of the CPU temperature driver.
   */
  const cpuDir = sensors.temperatureFile === ABSENT ? ABSENT : dirname(sensors.temperatureFile,);
  /**
   Motherboard temperature fields.
   */
  const board = await Promise.all(BOARD_SENSOR_INDEXES.map(function boardTemp(index,) {
    return readMilliCelsius({
      dir: sensors.boardDir,
      name: `temp${index}_input`,
    },);
  },),);
  return [
    input.time,
    await readMilliCelsius({
      dir: cpuDir,
      name: 'temp1_input',
    },),
    await readMilliCelsius({
      dir: cpuDir,
      name: 'temp3_input',
    },),
    await readMilliCelsius({
      dir: cpuDir,
      name: 'temp4_input',
    },),
    fixedField({
      value: power.watts,
      digits: 1,
    },),
    fixedField({
      value: power.mhz,
      digits: 0,
    },),
    input.cpuPressure
      .toFixed(1,),
    input.load1,
    String(input.cap,),
    ...Array.from(
      { length: LEGACY_CORSAIR_COLUMN_COUNT, },
      function emptyCorsairField(): string {
        return ABSENT;
      },
    ),
    fixedField({
      value: power.chiplet1Watts,
      digits: 1,
    },),
    fixedField({
      value: power.chiplet2Watts,
      digits: 1,
    },),
    ...board,
    await readMilliCelsius({
      dir: sensors.acpiDir,
      name: 'temp1_input',
    },),
    await readMilliCelsius({
      dir: sensors.nvmeDir,
      name: 'temp1_input',
    },),
    input.burst ? '1' : '0',
    await readMilliCelsius({
      dir: sensors.gpuDir,
      name: 'temp1_input',
    },),
    await readMicroWatts({
      dir: sensors.gpuDir,
      name: 'power1_average',
    },),
    await readMilliCelsius({
      dir: sensors.igpuDir,
      name: 'temp1_input',
    },),
    await readMicroWatts({
      dir: sensors.igpuDir,
      name: 'power1_input',
    },),
    ...await Promise.all(MB_FAN_INPUTS.map(function boardFan(index,) {
      return readRpm({
        dir: sensors.superIoDir,
        name: `fan${index}_input`,
      },);
    },),),
    ...await Promise.all(MB_TEMPERATURE_INPUTS.map(function boardTemperature(index,) {
      return readMilliCelsius({
        dir: sensors.superIoDir,
        name: `temp${index}_input`,
      },);
    },),),
  ];
}

/**
 Appends one sample row to today's file, pruning old files once per day.

 
 @param paths - watcher paths
 
 @param sensors - sensor locations
 
 @param power - this pass's power sample
 
 @param cap - thread cap in force
 
 @param burst - whether burst mode is on
 
 @param sampleDay - local date of the last write, to prune once per day

 @returns the local date written, to pass back as `sampleDay`

 @example
 ```ts
 sampleDay = await writeSample({ paths, sensors, power, cap: 28, burst: false, sampleDay, },);
 ```
 */
export async function writeSample({
  paths,
  sensors,
  power,
  cap,
  burst,
  sampleDay,
}: Readonly<{
  paths: Paths;
  sensors: Sensors;
  power: PowerSample;
  cap: number;
  burst: boolean;
  sampleDay: string;
}>,): Promise<string> {
  /**
   Time of this sample.
   */
  const now = new Date();
  /**
   Local date of this sample.
   */
  const today = now.toLocaleDateString('sv',);
  await mkdir(
    paths.sampleDir,
    { recursive: true, },
  );
  if (today !== sampleDay) await pruneSamples({
    sampleDir: paths.sampleDir,
    now: now.getTime(),
  },);
  /**
   Path of the day's sample file.
   */
  const file = join(
    paths.sampleDir,
    `${today}${CSV_SUFFIX}`,
  );
  await ensureHeader(file,);
  /**
   The row's CSV fields.
   */
  const fields = await buildSampleFields({
    time: now.toISOString(),
    cap,
    power,
    burst,
    cpuPressure: await readPressure({
      file: join(
        paths.procRoot,
        'pressure',
        'cpu',
      ),
      kind: 'some',
    },),
    load1: (await readFile(
      join(
        paths.procRoot,
        'loadavg',
      ),
      'utf8',
    )).split(' ',)[0] ?? ABSENT,
    sensors,
  },);
  await appendFile(
    file,
    `${fields.join(',',)}\n`,
  );
  return today;
}
