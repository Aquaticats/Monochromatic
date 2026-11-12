/**
 The end of Pi's standard error: the lines a launcher prints, and the reason among them that a batch's one line carries.

 The fixture is what Pi printed on 2026-11-11 when an extension could not be loaded, which is the failure these lines exist for.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  PI_STANDARD_ERROR_TAIL_BYTES,
  PI_STANDARD_ERROR_TAIL_LINES,
  lastOfPiStandardError,
  reasonInFailedStart,
} from '../dist/final/node/index.mjs';

/**
 The error Pi printed when the settled-notification extension could not be loaded, as `stderr.log` held it.
 */
const EXTENSION_ERROR = 'Error: Failed to load extension '
  + '"/repo/package/pi-plugin/agent-settled-notification/dist/final/node/index.mjs": '
  + "Failed to load extension: Cannot find module '@monochromatic-dev/async-time-core/ts'";

/**
 The advice Pi prints after an extension it could not load.
 */
const ADVICE = 'Hint: Start without extensions using "pi -ne".';

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-standard-error-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

await describe({
  name: lastOfPiStandardError.name,
  children: [
    it({
      name: 'reads the error Pi printed and the advice after it, and names the error as the reason rather than the advice',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The file Pi's standard error was kept in, holding the failure and the stack under it.
         */
        const file = join(
          folder.path,
          'stderr.log',
        );
        await writeFile(file, `${EXTENSION_ERROR}\n    at loadOne (file:///repo/a.ts:12:9)\n    at async main (file:///repo/b.ts:3:1)\n${ADVICE}\n`,);
        const tail = await lastOfPiStandardError({ file, },);
        expect(tail.lines,).toEqual([
          EXTENSION_ERROR,
          '    at loadOne (file:///repo/a.ts:12:9)',
          '    at async main (file:///repo/b.ts:3:1)',
          ADVICE,
        ],);
        expect(tail.reason,).toBe(EXTENSION_ERROR,);
        // The reason is what the batch's one line carries, so the same answer must come out of the line rule the batch uses.
        expect(reasonInFailedStart({ output: `the unit ended with status 1\nMemory peak: 12.0M\n${tail.lines.join('\n',)}\n`, },).reason,)
          .toBe(ADVICE,);
      },
    },),
    it({
      name: 'reads only the chosen number of lines, so a long stack cannot bury the error above what a reader needs',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The file Pi's standard error was kept in, holding more lines than a tail carries.
         */
        const file = join(
          folder.path,
          'stderr.log',
        );
        /**
         More lines than the tail keeps, each saying where it is.
         */
        const numbered = Array.from(
          { length: PI_STANDARD_ERROR_TAIL_LINES + 5, },
          function lineAt(_unused, index,) {
            return `frame ${String(index,)}`;
          },
        );
        await writeFile(file, `${numbered.join('\n',)}\n`,);
        const tail = await lastOfPiStandardError({ file, },);
        expect(tail.lines,).toHaveLength(PI_STANDARD_ERROR_TAIL_LINES,);
        expect(tail.lines.at(0,),).toBe(`frame ${String(5,)}`,);
        expect(tail.lines.at(-1,),).toBe(`frame ${String(PI_STANDARD_ERROR_TAIL_LINES + 4,)}`,);
      },
    },),
    it({
      name: 'answers an empty tail for a run whose Pi wrote nothing, and for a file that is not there',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A file a Pi wrote nothing to.
         */
        const empty = join(
          folder.path,
          'empty.log',
        );
        await writeFile(empty, '',);
        expect(await lastOfPiStandardError({ file: empty, },),).toEqual({ lines: [], },);
        expect(await lastOfPiStandardError({ file: join(
          folder.path,
          'never-written.log',
        ), },),).toEqual({ lines: [], },);
      },
    },),
    it({
      name: 'reads the end of a file far larger than the bytes it keeps, and names the error there without its advice',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A file whose beginning is far larger than the end a launcher reads, and whose end holds the failure.
         */
        const long = join(
          folder.path,
          'long.log',
        );
        await writeFile(long, `${'a warning nobody prints'.repeat(PI_STANDARD_ERROR_TAIL_BYTES + 1,)}\n${EXTENSION_ERROR}\n${ADVICE}\n`,);
        const tail = await lastOfPiStandardError({ file: long, },);
        expect(tail.lines.at(-2,),).toBe(EXTENSION_ERROR,);
        expect(tail.lines.at(-1,),).toBe(ADVICE,);
        expect(tail.reason,).toBe(EXTENSION_ERROR,);
      },
    },),
    it({
      name: 'names the last line that stands when Pi printed no error of its own, never its advice',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         A file holding a warning and Pi's advice, and no error.
         */
        const advised = join(
          folder.path,
          'advised.log',
        );
        await writeFile(advised, `an earlier warning\n${ADVICE}\n`,);
        const tail = await lastOfPiStandardError({ file: advised, },);
        expect(tail.lines,).toEqual(['an earlier warning', ADVICE,],);
        expect(tail.reason,).toBe('an earlier warning',);
      },
    },),
  ],
},);
