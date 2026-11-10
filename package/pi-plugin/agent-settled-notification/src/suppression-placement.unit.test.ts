/**
 Guard rule LN6's suppression placement for this package's source files.

 Rule LN6 suppresses a documented declaration with a disable directive, then the
 TSDoc block, then the declaration, then an enable directive on the very next line.
 A `disable-next-line` directive wedged between a TSDoc block and its declaration
 reads as part of the documentation instead of as a suppression, so this guard
 rejects that shape wherever it appears in this package.

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

/** Extension of every source file this guard scans. */
const SOURCE_FILE_SUFFIX = '.ts';

/** Opening token of the next-line suppression directive rule LN6 forbids above TSDoc. */
const SUPPRESSION_DIRECTIVE = '// oxlint-disable-next-line';

/** Terminating token of any block comment. */
const BLOCK_COMMENT_END = '*/';

/** Opening token of a block comment that carries documentation. */
const TSDOC_OPENER = '/**';

/**
 Decide whether a directory entry is a TypeScript source file.

 @param entry - entry from the source directory

 @returns whether this guard scans the entry
 */
function isScannedSourceFile(entry: Dirent,): boolean {
  return entry.isFile() && entry.name.endsWith(SOURCE_FILE_SUFFIX,);
}

/**
 Decide whether the block comment ending on a given line is a TSDoc block.

 @param lines - file lines

 @param endIndex - index of the line carrying the block comment terminator

 @returns whether the enclosing block opener is a TSDoc opener
 */
function endsTsdocBlock(lines: readonly string[], endIndex: number,): boolean {
  for (let index = endIndex; index >= 0; index -= 1) {
    const line = lines[index] ?? '';
    if (line.includes('/*',))
      return line.trimStart().startsWith(TSDOC_OPENER,);
  }
  return false;
}

/**
 Name every suppression this file wedges between a TSDoc block and its declaration.

 @param text - file contents

 @returns one `line: directive` entry per wedged suppression
 */
function findWedgedSuppressions(text: string,): readonly string[] {
  const lines = text.split('\n',);
  const wedged: string[] = [];

  lines.forEach(function inspect(line: string, index: number,): void {
    if (!line.trimStart().startsWith(SUPPRESSION_DIRECTIVE,))
      return;
    const aboveIndex = index - 1;
    const above = lines[aboveIndex]?.trim() ?? '';
    if (!above.endsWith(BLOCK_COMMENT_END,))
      return;
    if (!endsTsdocBlock(lines, aboveIndex,))
      return;
    wedged.push(`${String(index + 1,)}: ${line.trim()}`,);
  },);

  return wedged;
}

/**
 Name every wedged suppression in one source file.

 @param path - source file whose suppressions are inspected

 @returns one `path: line: directive` entry per wedged suppression
 */
async function describeWedgedSuppressions(path: string,): Promise<readonly string[]> {
  const text = await readFile(path, 'utf8',);
  return findWedgedSuppressions(text,)
    .map(wedged => `${path}: ${wedged}`,);
}

await describe({
  name: 'suppression placement',
  children: [
    it({
      name: 'keeps every next-line suppression clear of the TSDoc block above it',
      fn: async () => {
        const entries = await readdir(SOURCE_DIRECTORY, { withFileTypes: true, },);
        const paths = entries
          .filter(isScannedSourceFile,)
          .map(entry => join(SOURCE_DIRECTORY, entry.name,));
        const wedged = (await Promise.all(
          paths.map(path => describeWedgedSuppressions(path,),),
        )).flat();
        expect(wedged,).toEqual([],);
      },
    },),
  ],
},);
