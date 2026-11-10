/**
 No library source in this package writes through the raw console.

 The repository logs production output through tagged loggers from
 `\@monochromatic-dev/logger-core`, and reserves the raw console for exact terminal output such as a CLI's own
 stdout. A library that writes to the host console cannot be silenced, redirected, tagged or correlated by its
 caller, which is what the container and host backends did with a successful run's stderr (issue #1418).

 Comments are blanked before the scan, so a TSDoc line that names a console method is not read as a call.

 @module
 */

import { stripSourceComments, } from '@monochromatic-dev/container-limit-core/ts/source-text-without-comments.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { librarySourceTexts, } from '../dist/final/node/index.mjs';

/**
 The text a raw console call starts with.
 */
const CONSOLE_CALL = 'console.';

/**
 Lists the lines of one source text that call the raw console.

 @param text - source text, comments already blanked

 @returns 1-based line of each console call, in source order

 @example
 ```ts
 rawConsoleSites('l.info("built",);\nconsole.error("failed",);');
 // [2]
 ```
 */
function rawConsoleSites(text: string,): readonly number[] {
  /**
   Lines carrying a console call, in source order.
   */
  const lines: number[] = [];

  for (let index = 0, line = 1; index < text.length;) {
    if (text[index] === '\n') {
      line += 1;
      index += 1;
      continue;
    }
    if (!text.startsWith(CONSOLE_CALL, index,)) {
      index += 1;
      continue;
    }

    lines.push(line,);
    index += CONSOLE_CALL.length;
  }

  return lines;
}

await describe({
  name: 'raw console in library sources',
  children: [
    it({
      name: 'reports the line of a console call',
      fn: async () => {
        expect(rawConsoleSites('l.info("built",);\nconsole.error("failed",);\n',),)
          .toEqual([2,],);
      },
    },),

    it({
      name: 'ignores a console method named in a comment',
      fn: async () => {
        /**
         A TSDoc line that names the method without calling it.
         */
        const commented = stripSourceComments({
          text: '/**\n Writes through console.error today.\n */\n',
          language: 'typescript',
        },);

        expect(rawConsoleSites(commented,),)
          .toEqual([],);
      },
    },),

    it({
      name: 'writes no library source of this package through the raw console',
      fn: async () => {
        /**
         Library sources this scan reads.
         */
        const sources = await librarySourceTexts();

        expect(sources.length,).toBeGreaterThan(0,);

        /**
         Every console call, named by file and line.
         */
        const reported = sources.flatMap(({ path, text, },) => rawConsoleSites(text,)
          .map((line,) => `${path}:${String(line,)}`,),);

        expect(reported,).toEqual([],);
      },
    },),
  ],
},);
