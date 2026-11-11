/**
 Tests for how a failed check's first error is found: the output is read as it streams past, so the error that stands
 after a long noisy start is the one the fix-forward report shows.

 The fixture is the output recorded from the run that opened issue 6284: 257 warnings from the walk cargo makes before
 it starts compiling, then the compiler's error at character 183,066 of 184,067.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { Readable, } from 'node:stream';

import {
  type FirstError,
  firstErrorOf,
  firstErrorScan,
  keepFirstError,
  MAXIMUM_LINE_CHARACTERS,
  NO_ERROR_IN_OUTPUT,
} from '../dist/final/node/testing.mjs';

//region Fixtures: the output of the recorded run

/**
 One of the warnings cargo prints for every symlink loop its walk meets, in the shape and length of the recorded lines.
 */
const LOOP_WARNING_LINE: string = [
  'warning: File system loop found: ',
  '/r/package/forbidden-strings/cli/node_modules/@monochromatic-dev/or-throw-core',
  '/node_modules/@monochromatic-dev/test-harness-core',
  '/node_modules/@monochromatic-dev/test-harness-core-sandbox',
  '/node_modules/@monochromatic-dev/logger-core',
  '/node_modules/@monochromatic-dev/test-harness-core',
  ' points to an ancestor ',
  '/r/package/forbidden-strings/cli/node_modules/@monochromatic-dev/or-throw-core',
  '/node_modules/@monochromatic-dev/test-harness-core',
].join('',);

/**
 How many of those warnings the recorded run printed before its first line of compiler output.
 */
const LOOP_WARNING_LINES = 257;

/**
 The error that run printed after them, with the location lines and the help the report shows, ending in the blank line
 that closes it.
 */
const RECORDED_ERROR_LINES: readonly string[] = [
  'error: manual implementation of `Option::map`',
  '   --> src/native/worktree_copying/worktree_copy_prune.rs:258:41',
  '    |',
  '258 |               let parent: Option<&[u8]> = match index.checked_sub(1) {',
  '    |  _________________________________________^',
  '    | |_____________^ help: try: `index.checked_sub(1).map(|before| components[before])`',
  '    |',
  '    = help: for further information visit https://clippy.rust-lang.org/main/index.html#manual_map',
  '    = note: `-D clippy::manual-map` implied by `-D warnings`',
  '',
];

/**
 The lines the recorded run printed once its compiler started, in order.
 */
const RECORDED_OUTPUT_LINES: readonly string[] = [
  ...Array.from(
    { length: LOOP_WARNING_LINES, },
    function warning() {
      return LOOP_WARNING_LINE;
    },
  ),
  ...RECORDED_ERROR_LINES,
];

/**
 The recorded output as one text.
 */
const RECORDED_OUTPUT: string = RECORDED_OUTPUT_LINES.join(
  '\n',
);

/**
 The excerpt the recorded error should be shown as: its own lines, up to the blank line that closes it.
 */
const RECORDED_EXCERPT: string = RECORDED_ERROR_LINES.slice(
  0,
  (-1),
).join('\n',);

/**
 Characters one piece of the recorded output carries when it is read in pieces. Seven divides no line of that output
 evenly, so the pieces end inside lines as well as between them.
 */
const PIECE_CHARACTERS = 7;

//endregion Fixtures: the output of the recorded run

//region Helpers

/**
 A stream of the chunks one run of output would arrive in.

 @param chunks - the output in the pieces the stream delivers them

 @returns a readable stream carrying them, ending after the last

 @example
 ```ts
 const stream = chunksOf(['error: x\n',],);
 ```
 */
function chunksOf(chunks: readonly string[],): Readable {
  return Readable.from(chunks,);
}

/**
 The error a scan found, failing the test when it found none.

 @param answered - what the scan answered

 @returns the error

 @example
 ```ts
 const error = errorIn(scan.error(),);
 ```
 */
function errorIn(answered: FirstError | typeof NO_ERROR_IN_OUTPUT,): FirstError {
  if (answered === NO_ERROR_IN_OUTPUT)
    throw new Error('the output has no error',);
  return answered;
}

//endregion Helpers

await describe({
  name: '',
  children: [
    //region Reading the output as it streams

    describe({
      name: keepFirstError.name,
      children: [
        it({
          name: 'finds the error that stands after the warnings the recorded run printed first',
          fn: async () => {
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [chunksOf(RECORDED_OUTPUT_LINES.map(function lineEnded(line,) {
                return `${line}\n`;
              },),),],
              scan,
            },);
            const error = errorIn(scan.error(),);
            expect(error.code,).toBe('',);
            expect(error.message,).toBe('manual implementation of `Option::map`',);
            expect(error.excerpt,).toBe(RECORDED_EXCERPT,);
          },
        },),
        it({
          name: 'reads a line split across two chunks as the one line it is',
          fn: async () => {
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [chunksOf([
                'er',
                'ror: could not find `x` in `y`',
                '\n   --> src/a.rs:2:5\n',
              ],),],
              scan,
            },);
            const error = errorIn(scan.error(),);
            expect(error.message,).toBe('could not find `x` in `y`',);
            expect(error.excerpt,).toContain('src/a.rs:2:5',);
          },
        },),
        it({
          name: 'reads both streams, so an error on the error stream is found beside the other one',
          fn: async () => {
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [
                chunksOf(['   Compiling a v0.1.0\n',],),
                chunksOf(['error: linker not found\n',],),
              ],
              scan,
            },);
            expect(errorIn(scan.error(),).message,).toBe('linker not found',);
          },
        },),
        it({
          name: 'finds the start of an error in a line longer than it keeps, and keeps no more of it',
          fn: async () => {
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [chunksOf([`error: ${'x'.repeat(MAXIMUM_LINE_CHARACTERS * 2,)}\n`],),],
              scan,
            },);
            const error = errorIn(scan.error(),);
            expect(error.message.length,).toBe(MAXIMUM_LINE_CHARACTERS - 'error: '.length,);
          },
        },),
        it({
          name: 'does not read the rest of a line it stopped keeping as a line of its own',
          fn: async () => {
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [chunksOf([
                'y'.repeat(MAXIMUM_LINE_CHARACTERS + 1,),
                'error: not a line start\n',
              ],),],
              scan,
            },);
            expect(scan.error(),).toBe(NO_ERROR_IN_OUTPUT,);
          },
        },),
      ],
    },),

    //endregion Reading the output as it streams

    //region Reading one text

    describe({
      name: firstErrorOf.name,
      children: [
        it({
          name: 'ranks every error start by the order it stands in the output',
          fn: async () => {
            const error = firstErrorOf(RECORDED_OUTPUT,);
            if (error === NO_ERROR_IN_OUTPUT)
              throw new Error('the recorded output has no error',);
            expect(error.message,).toBe('manual implementation of `Option::map`',);
            expect(error.excerpt,).toBe(RECORDED_EXCERPT,);
          },
        },),
        it({
          name: 'reads one text the same way whether it arrives whole or in pieces',
          fn: async () => {
            const whole = firstErrorOf(RECORDED_OUTPUT,);
            const scan = firstErrorScan();
            await keepFirstError({
              streams: [chunksOf(Array.from(
                { length: Math.ceil(RECORDED_OUTPUT.length / PIECE_CHARACTERS,), },
                function pieceAt(_, index,) {
                  return RECORDED_OUTPUT.slice(index * PIECE_CHARACTERS, (index + 1) * PIECE_CHARACTERS,);
                },
              ),),],
              scan,
            },);
            expect(scan.error(),).toEqual(whole,);
          },
        },),
      ],
    },),

    //endregion Reading one text
  ],
},);
