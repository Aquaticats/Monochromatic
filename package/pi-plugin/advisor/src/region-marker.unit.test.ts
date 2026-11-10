/**
 Guard rule ST2's region markers for this package's substantial source files.

 Rule ST2 marks logical sections with `//region` / `//endregion` pairs whose label
 states the section's purpose. A source file long enough to hold several
 responsibility groups must carry at least one marked region, so its sections stay
 navigable and foldable.

 @module
 */

import type { Dirent, } from 'node:fs';
import { readdir, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** Directory holding every source file this guard scans. */
const SOURCE_DIRECTORY = import.meta.dirname;

/** Source line count from which a file is substantial enough to need section markers. */
const SUBSTANTIAL_LINE_COUNT = 100;

/** Suffix identifying co-located unit test files, which this guard skips. */
const UNIT_TEST_FILE_SUFFIX = '.unit.test.ts';

/** Extension of every source file this guard scans. */
const SOURCE_FILE_SUFFIX = '.ts';

/** Opening marker rule ST2 requires in a substantial source file. */
const REGION_MARKER = '//region';

/**
 Count physical lines the way `wc --lines` does for a newline-terminated file.

 @param text - file contents

 @returns number of terminated lines
 */
function countLines(text: string,): number {
  const lines = text.split('\n',);
  return lines.length - (text.endsWith('\n',) ? 1 : 0);
}

/**
 Decide whether a directory entry is a non-test TypeScript source file.

 @param entry - entry from the source directory

 @returns whether this guard scans the entry
 */
function isScannableSourceFile(entry: Dirent,): boolean {
  if (!entry.isFile())
    return false;
  if (!entry.name.endsWith(SOURCE_FILE_SUFFIX,))
    return false;
  return !entry.name.endsWith(UNIT_TEST_FILE_SUFFIX,);
}

/**
 Name a substantial non-test source file that carries no region marker.

 @param path - source file whose length and markers are inspected

 @returns one `path` entry naming the line count when the file is substantial and unmarked, otherwise an empty list
 */
async function findUnmarkedSubstantialFile(path: string,): Promise<string[]> {
  const text = await readFile(path, 'utf8',);
  const lineCount = countLines(text,);
  if (lineCount < SUBSTANTIAL_LINE_COUNT)
    return [];
  if (text.split('\n',).some(line => line.trimStart().startsWith(REGION_MARKER,)))
    return [];
  return [`${path} (${String(lineCount,)} lines)`];
}

await describe({
  name: 'region markers',
  children: [
    it({
      name: 'marks every non-test source file of at least the substantial line count',
      fn: async () => {
        const entries = await readdir(SOURCE_DIRECTORY, { withFileTypes: true, },);
        const paths = entries
          .filter(isScannableSourceFile,)
          .map(entry => join(SOURCE_DIRECTORY, entry.name,));
        const unmarked = (await Promise.all(
          paths.map(path => findUnmarkedSubstantialFile(path,),),
        )).flat();
        expect(unmarked,).toEqual([],);
      },
    },),
  ],
},);
