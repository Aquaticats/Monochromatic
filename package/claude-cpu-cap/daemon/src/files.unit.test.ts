import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ABSENT, } from './constants.ts';
import {
  isMissingFile,
  readMicroWatts,
  readMilliCelsius,
  readRpm,
  readTextOrEmpty,
} from './files.ts';

/**
 Creates a directory holding one sensor file.

 @param name - file name
 @param text - file contents

 @returns the directory
 */
async function sensorDir({ name, text, }: Readonly<{ name: string; text: string; }>,): Promise<string> {
  /**
   Fresh directory so cases cannot share files.
   */
  const dir = await mkdtemp(join(scratchRoot(), 'cpu-cap-files-',),);
  await writeFile(join(dir, name,), text,);
  return dir;
}

await describe({
  name: 'files',
  children: [
    describe({
      name: isMissingFile.name,
      children: [
        it({
          name: 'is true only for an error with code ENOENT',
          fn: async function onlyEnoent() {
            expect(
              isMissingFile(Object.assign(new Error('x',), { code: 'ENOENT', },),),
            ).toBe(true,);
            expect(
              isMissingFile(Object.assign(new Error('x',), { code: 'EACCES', },),),
            ).toBe(false,);
            expect(
              isMissingFile(new Error('x',),),
            ).toBe(false,);
            expect(isMissingFile('ENOENT',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: readTextOrEmpty.name,
      children: [
        it({
          name: 'reads an existing file and answers an empty string for a missing one',
          fn: async function readsOrEmpty() {
            /**
             Directory with one file.
             */
            const dir = await sensorDir({ name: 'a', text: 'hello', },);
            expect(
              await readTextOrEmpty(join(dir, 'a',),),
            ).toBe('hello',);
            expect(
              await readTextOrEmpty(join(dir, 'b',),),
            ).toBe(ABSENT,);
          },
        },),
        it({
          name: 'rethrows failures other than a missing file',
          fn: async function rethrows() {
            /**
             A directory path, which cannot be read as a file.
             */
            const dir = await sensorDir({ name: 'a', text: 'x', },);
            await expect(readTextOrEmpty(dir,),).rejects.toThrow('EISDIR',);
          },
        },),
      ],
    },),
    describe({
      name: 'unit conversion',
      children: [
        it({
          name: 'readMilliCelsius converts millidegrees to one-decimal degrees and is empty without a sensor',
          fn: async function milliCelsius() {
            /**
             Directory with a temperature input.
             */
            const dir = await sensorDir({ name: 'temp1_input', text: '61500\n', },);
            expect(await readMilliCelsius({ dir, name: 'temp1_input', },),).toBe('61.5',);
            expect(await readMilliCelsius({ dir, name: 'temp9_input', },),).toBe(ABSENT,);
            expect(await readMilliCelsius({ dir: ABSENT, name: 'temp1_input', },),).toBe(ABSENT,);
          },
        },),
        it({
          name: 'readRpm formats whole revolutions per minute, keeps a stopped fan at 0 and is empty without a sensor',
          fn: async function revolutions() {
            /**
             Directory with a fan input.
             */
            const dir = await sensorDir({ name: 'fan1_input', text: '1509\n', },);
            /**
             Directory with a fan input of a header with no fan plugged in.
             */
            const stopped = await sensorDir({ name: 'fan2_input', text: '0\n', },);
            expect(await readRpm({ dir, name: 'fan1_input', },),).toBe('1509',);
            expect(await readRpm({ dir: stopped, name: 'fan2_input', },),).toBe('0',);
            expect(await readRpm({ dir, name: 'fan9_input', },),).toBe(ABSENT,);
            expect(await readRpm({ dir: ABSENT, name: 'fan1_input', },),).toBe(ABSENT,);
          },
        },),
        it({
          name: 'readMicroWatts converts microwatts to one-decimal watts and is empty without a sensor',
          fn: async function microWatts() {
            /**
             Directory with a power input.
             */
            const dir = await sensorDir({ name: 'power1_average', text: '12340000\n', },);
            expect(await readMicroWatts({ dir, name: 'power1_average', },),).toBe('12.3',);
            expect(await readMicroWatts({ dir, name: 'power9_average', },),).toBe(ABSENT,);
            expect(await readMicroWatts({ dir: ABSENT, name: 'power1_average', },),).toBe(ABSENT,);
          },
        },),
      ],
    },),
  ],
},);
