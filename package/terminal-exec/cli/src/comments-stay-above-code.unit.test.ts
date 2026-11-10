/**
 Guards this package's comment placement: an explanatory comment sits on its
 own line above the statement it explains, never after it (rule TD4).

 TSDoc `@example` blocks are exempt. A trailing `// -> 'result'` there shows
 what a call returns, which the signature alone cannot express.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Directory whose TypeScript sources are scanned.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 One comment that trails code instead of sitting above it.
 */
type TrailingComment = {
  /**
   One-based line number inside the scanned file.
   */
  readonly line: number;
  /**
   File name inside {@link SOURCE_DIRECTORY}.
   */
  readonly path: string;
};

/**
 Finds comments that trail code in one source text.

 TSDoc blocks are skipped line by line, including the one-line form, so the
 result-annotated examples inside them stay allowed. A marker counts only
 after whitespace, so the slash runs inside string literals such as `'///foo'`
 stay ordinary data.

 @param text - Source text to scan.

 @param path - File name reported for each finding.

 @returns Every trailing comment, in line order.
 */
function trailingCommentsIn({
  text,
  path,
}: {
  readonly text: string;
  readonly path: string;
},): readonly TrailingComment[] {
  /**
   Findings so far, in line order.
   */
  const violations: TrailingComment[] = [];
  /**
   Whether the scan is inside a TSDoc block comment.
   */
  let inTsdoc = false;

  for (const [index, line,] of text
    .split('\n',)
    .entries()) {
    if (inTsdoc) {
      if (line.includes('*/',))
        inTsdoc = false;
      continue;
    }

    if (line
      .trimStart()
      .startsWith('/**',)) {
      if (!line.includes('*/',))
        inTsdoc = true;
      continue;
    }

    /**
     Column where a line comment starts, or -1 when the line has none.
     */
    const marker = line.indexOf('//',);
    if (marker === (-1))
      continue;

    /**
     Character just before the marker; whitespace for a comment that trails code.
     */
    const preceding = line[marker - 1];
    if ((preceding === undefined) || (preceding.trim().length > 0))
      continue;

    /**
     Code ahead of the marker; blank means the marker opens the line's own comment.
     */
    const codeBefore = line.slice(
      0,
      marker,
    );
    if (codeBefore.trim().length
      === 0)
      continue;

    violations.push({
      line: index + 1,
      path,
    },);
  }

  return violations;
}

await describe({
  name: 'comment placement',
  children: [
    it({
      name: 'puts every source comment on its own line above the code',
      fn: async () => {
        /**
         Every TypeScript file the package ships.
         */
        const files = (await readdir(SOURCE_DIRECTORY,)).filter(function isTypeScript(file,) {
          return file.endsWith('.ts',);
        },);
        /**
         File name paired with its source text, read concurrently.
         */
        const sources = await Promise.all(files.map(async function readSource(file,) {
          return {
            file,
            text: await readFile(
              join(
                SOURCE_DIRECTORY,
                file,
              ),
              'utf8',
            ),
          };
        },),);

        expect(sources.flatMap(function findTrailing({
          file,
          text,
        },) {
          return trailingCommentsIn({
            text,
            path: file,
          },);
        },),).toEqual([],);
      },
    },),
  ],
},);
