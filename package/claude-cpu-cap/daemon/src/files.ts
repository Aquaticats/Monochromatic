import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  ABSENT,
  MICRO_PER_UNIT,
  MILLI_PER_UNIT,
} from './constants.ts';

/**
 Whether an error is a missing-file error, the one failure sensors are expected to produce.

 @param error - caught value

 @returns true for `ENOENT`

 @example
 ```ts
 isMissingFile(Object.assign(new Error('x'), { code: 'ENOENT', },),); // true
 ```
 */
export function isMissingFile(error: unknown,): boolean {
  return (Error.isError(error,)) && ('code' in error)
    && (error.code === 'ENOENT');
}

/**
 Reads a text file, answering an empty string when the file does not exist so absent sensors yield empty fields.

 @param file - path to read

 @returns the file text, or an empty string when missing

 @throws on any read failure other than a missing file

 @example
 ```ts
 await readTextOrEmpty('/sys/class/hwmon/hwmon9/name',); // '' when the file is missing
 ```
 */
export async function readTextOrEmpty(file: string,): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  } catch (error) {
    if (isMissingFile(error,)) return ABSENT;
    throw error;
  }
}

/**
 Reads a hwmon temperature input and formats it in degrees Celsius with one decimal.

 
 @param dir - hwmon directory, or {@link ABSENT}
 
 @param name - input file name such as `temp1_input`

 @returns degrees Celsius, or an empty string when the sensor is absent

 @example
 ```ts
 await readMilliCelsius({ dir: '/sys/class/hwmon/hwmon4', name: 'temp1_input', },); // '61.5'
 ```
 */
export async function readMilliCelsius({
  dir,
  name,
}: Readonly<{
  dir: string;
  name: string
}>,): Promise<string> {
  if (dir === ABSENT) return ABSENT;
  /**
   Contents of the sensor file, empty when it does not exist.
   */
  const text = await readTextOrEmpty(join(
    dir,
    name,
  ),);
  return text === ABSENT ? ABSENT : (Number(text,) / MILLI_PER_UNIT).toFixed(1,);
}

/**
 Reads a hwmon fan input and formats it as whole revolutions per minute.

 @param dir - hwmon directory, or {@link ABSENT}

 @param name - input file name such as `fan1_input`

 @returns revolutions per minute, or an empty string when the sensor is absent

 @example
 ```ts
 await readRpm({ dir: '/sys/class/hwmon/hwmon1', name: 'fan1_input', },); // '1509'
 ```
 */
export async function readRpm({
  dir,
  name,
}: Readonly<{
  dir: string;
  name: string
}>,): Promise<string> {
  if (dir === ABSENT) return ABSENT;
  /**
   Contents of the sensor file, empty when it does not exist.
   */
  const text = await readTextOrEmpty(join(
    dir,
    name,
  ),);
  return text === ABSENT ? ABSENT : String(Number(text,),);
}

/**
 Reads a hwmon power input and formats it in watts with one decimal.

 
 @param dir - hwmon directory, or {@link ABSENT}
 
 @param name - input file name such as `power1_average`

 @returns watts, or an empty string when the sensor is absent

 @example
 ```ts
 await readMicroWatts({ dir: '/sys/class/hwmon/hwmon2', name: 'power1_average', },); // '12.0'
 ```
 */
export async function readMicroWatts({
  dir,
  name,
}: Readonly<{
  dir: string;
  name: string
}>,): Promise<string> {
  if (dir === ABSENT) return ABSENT;
  /**
   Contents of the sensor file, empty when it does not exist.
   */
  const text = await readTextOrEmpty(join(
    dir,
    name,
  ),);
  return text === ABSENT ? ABSENT : (Number(text,) / MICRO_PER_UNIT).toFixed(1,);
}
