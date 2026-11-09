import {
  mkdtempDisposable,
  type DisposableTempDir,
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
  pathExists,
  readMicroWatts,
  readMilliCelsius,
  readRpm,
  readTextOrEmpty,
} from './files.ts';

/**
 Creates a directory holding one sensor file, removed when the caller's scope
 ends.

 @param name - file name
 @param text - file contents

 @returns the directory, which owns its own removal
 */
async function sensorDir({ name, text, }: Readonly<{ name: string; text: string; }>,): Promise<DisposableTempDir> {
  /**
   Fresh directory so cases cannot share files.
   */
  const directory = await mkdtempDisposable(join(scratchRoot(), 'cpu-cap-files-',),);
  await writeFile(join(directory.path, name,), text,);
  return directory;
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
            await using directory = await sensorDir({ name: 'a', text: 'hello', },);
            const dir = directory.path;
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
            await using directory = await sensorDir({ name: 'a', text: 'x', },);
            const dir = directory.path;
            await expect(readTextOrEmpty(dir,),).rejects.toThrow('EISDIR',);
          },
        },),
      ],
    },),
    describe({
      name: pathExists.name,
      children: [
        it({
          name: 'is true for an existing file or directory and false for a missing one',
          fn: async function existsOrNot() {
            /**
             Directory with one file.
             */
            await using directory = await sensorDir({ name: 'a', text: 'x', },);
            const dir = directory.path;
            /**
             Path of the existing file.
             */
            const existing = join(dir, 'a',);
            /**
             Path of a file that was never written.
             */
            const missing = join(dir, 'b',);
            expect(await pathExists(dir,),).toBe(true,);
            expect(await pathExists(existing,),).toBe(true,);
            expect(await pathExists(missing,),).toBe(false,);
          },
        },),
        it({
          name: 'rethrows failures other than a missing path',
          fn: async function rethrowsOther() {
            /**
             A path that continues past a regular file, which fails with ENOTDIR instead of ENOENT.
             */
            await using directory = await sensorDir({ name: 'a', text: 'x', },);
            const dir = directory.path;
            /**
             Path that continues past the file.
             */
            const belowFile = join(dir, 'a', 'b',);
            await expect(pathExists(belowFile,),).rejects.toThrow('ENOTDIR',);
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
            await using directory = await sensorDir({ name: 'temp1_input', text: '61500\n', },);
            const dir = directory.path;
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
            await using directory = await sensorDir({ name: 'fan1_input', text: '1509\n', },);
            const dir = directory.path;
            /**
             Directory with a fan input of a header with no fan plugged in.
             */
            await using stoppedDirectory = await sensorDir({ name: 'fan2_input', text: '0\n', },);
            const stopped = stoppedDirectory.path;
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
            await using directory = await sensorDir({ name: 'power1_average', text: '12340000\n', },);
            const dir = directory.path;
            expect(await readMicroWatts({ dir, name: 'power1_average', },),).toBe('12.3',);
            expect(await readMicroWatts({ dir, name: 'power9_average', },),).toBe(ABSENT,);
            expect(await readMicroWatts({ dir: ABSENT, name: 'power1_average', },),).toBe(ABSENT,);
          },
        },),
      ],
    },),
  ],
},);
