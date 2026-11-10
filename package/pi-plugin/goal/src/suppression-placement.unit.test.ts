/**
 Guard the LN6 suppression shape for documented declarations in this package.

 Rule LN6 writes a suppression for a documented declaration as a block
 `oxlint-disable`, then the TSDoc block, then the declaration, then a block
 `oxlint-enable` on the very next line. A `disable-next-line` directive wedged
 between a TSDoc block and its declaration breaks that shape and lets the
 directive read as part of the doc.

 @module
 */

import { readdir, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** Directory holding every source file this guard scans. */
const SOURCE_DIRECTORY = import.meta.dirname;

/** Line suppression directive that rule LN6 keeps off the line above a declaration. */
const DISABLE_NEXT_LINE_DIRECTIVE = '// oxlint-disable-next-line';

/** Closing delimiter of a TSDoc block. */
const TSDOC_CLOSING_DELIMITER = '*/';

/**
 Find line suppressions wedged between a TSDoc block and the declaration below it.

 @param path - source file whose lines are scanned

 @returns `path:line` for every wedged suppression, in source order
 */
async function findWedgedSuppressions(path: string,): Promise<string[]> {
  const lines = (await readFile(path, 'utf8',)).split('\n',);
  /** Offending suppressions, named by the line that carries the directive. */
  const wedged: string[] = [];
  for (const [index, line,] of lines.entries()) {
    if (!line.trimStart().startsWith(DISABLE_NEXT_LINE_DIRECTIVE,))
      continue;
    for (let above = index - 1; above >= 0; above -= 1) {
      const candidate = (lines[above] ?? '').trimEnd();
      if (candidate.trim() === '')
        continue;
      if (candidate.endsWith(TSDOC_CLOSING_DELIMITER,))
        wedged.push(`${path}:${String(index + 1,)}`,);
      break;
    }
  }
  return wedged;
}

await describe({
  name: 'suppression placement',
  children: [
    it({
      name: 'keeps every disable-next-line off the line between TSDoc and its declaration',
      fn: async () => {
        const entries = await readdir(SOURCE_DIRECTORY, { withFileTypes: true, },);
        const paths = entries
          .filter(entry => entry.isFile() && entry.name.endsWith('.ts',))
          .map(entry => join(SOURCE_DIRECTORY, entry.name,));
        const wedged = (await Promise.all(
          paths.map(path => findWedgedSuppressions(path,),),
        )).flat();
        expect(wedged,).toEqual([],);
      },
    },),
  ],
},);
