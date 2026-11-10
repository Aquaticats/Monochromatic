/**
 Guards the benchmark input builders in `parse.bench.ts`: each entry line comes
 from an `Array.from` mapper, never a classic `for` loop appending to a mutable
 accumulator, the shape the css-edit benchmark already uses.

 The builders are private to the benchmark module, which starts its timed loop
 as soon as it is imported, so this guard reads the source text instead of
 calling them.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Benchmark module whose input builders are guarded.
 */
const BENCH_SOURCE_PATH = join(import.meta.dirname, 'parse.bench.ts',);

/**
 Names of the input builders, in source order.
 */
const BUILDER_NAMES = ['buildCleanInput', 'buildCommentedInput',] as const;

/**
 Fragment closing every input builder, which proves the scan found the
 builder rather than empty text.
 */
const RETURN_MARKER = 'return `{';

/**
 Classic counted loop header the builders must not use.
 */
const LOOP_HEADER = 'for (';

/**
 Append call that marks a mutable accumulator.
 */
const APPEND_CALL = '.push(';

/**
 Source text of one top-level function declaration, up to the next one.

 @param text - Benchmark source text.

 @param name - Function name to extract.

 @returns Declaration text, empty when the source declares no such function.
 */
function functionSource({
  text,
  name,
}: {
  readonly text: string;
  readonly name: string;
},): string {
  const declaration = text
    .split('\nfunction ',)
    .find(function declarationOf(candidate,) {
      return candidate.startsWith(`${name}(`,);
    },);

  return declaration ?? '';
}

/**
 Benchmark source text, read once for every guarded builder.
 */
const benchSource = await readFile(BENCH_SOURCE_PATH, 'utf8',);

//endregion Fixtures

await describe({
  name: '',
  children: BUILDER_NAMES.map(function builderGuard(name,) {
    return describe({
      name,
      children: [
        it({
          name: 'builds entry lines without a loop accumulator',
          fn: async () => {
            const source = functionSource({
              text: benchSource,
              name,
            },);

            expect(source,).toContain(RETURN_MARKER,);
            expect(source,).not.toContain(LOOP_HEADER,);
            expect(source,).not.toContain(APPEND_CALL,);
          },
        },),
      ],
    },);
  },),
},);
