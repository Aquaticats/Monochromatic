import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readPressure, } from './pressure.ts';

/**
 A pressure file and the directory holding it, which removes itself when the
 caller's scope ends.
 */
type PressureFileFixture = {
  readonly file: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Writes a pressure file with the given text into a fresh directory.

 @param text - file contents

 @returns the file path, and an async disposer that removes the directory
 */
async function pressureFile(text: string,): Promise<PressureFileFixture> {
  /**
   Fresh directory so cases cannot share files.
   */
  const directory = await mkdtempDisposable(join(scratchRoot(), 'cpu-cap-pressure-',),);
  await writeFile(join(directory.path, 'cpu',), text,);
  return {
    file: join(directory.path, 'cpu',),
    [Symbol.asyncDispose]: async function removeFixtureDirectory(): Promise<void> {
      await directory.remove();
    },
  };
}

await describe({
  name: readPressure.name,
  children: [
    it({
      name: 'reads the some and full avg10 values from their own lines',
      fn: async function readsBothLines() {
        /**
         A pressure file whose two lines differ.
         */
        await using fixture = await pressureFile('some avg10=12.50 avg60=1.00 avg300=0.50 total=9\nfull avg10=3.25 avg60=0.00 avg300=0.00 total=1\n',);
        const { file, } = fixture;
        expect(await readPressure({ file, kind: 'some', },),).toBe(12.5,);
        expect(await readPressure({ file, kind: 'full', },),).toBe(3.25,);
      },
    },),
    it({
      name: 'throws when the requested kind has no avg10 value',
      fn: async function throwsWithoutValue() {
        /**
         A pressure file with no recognizable line.
         */
        await using fixture = await pressureFile('nothing here\n',);
        const { file, } = fixture;
        await expect(readPressure({ file, kind: 'some', },),).rejects.toThrow('no some avg10',);
      },
    },),
    it({
      name: 'throws when avg10 is not a number',
      fn: async function throwsOnNaN() {
        /**
         A pressure file with a malformed value.
         */
        await using fixture = await pressureFile('some avg10=abc avg60=0 avg300=0 total=0\n',);
        const { file, } = fixture;
        await expect(readPressure({ file, kind: 'some', },),).rejects.toThrow('no some avg10',);
      },
    },),
  ],
},);
