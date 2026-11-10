/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule PP1 requires `async`/`await` only and forbids a promise chain,
 and no linter carries that rule, so the product module's text is read here and
 checked directly. Lines that open with a comment marker are dropped first: a
 name quoted in a TSDoc example is prose rather than code.

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
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 This package's only product module.
 */
const PRODUCT_MODULE = 'index.ts';

/**
 Markers that open a comment line, whose text is prose rather than code.
 */
const COMMENT_LINE_OPENERS: readonly string[] = [
  '//',
  '/*',
  '*',
];

/**
 Promise-chain calls `PP1` forbids, each written as the leading dot and the
 opening parenthesis so a mention of the method's name alone does not match.
 */
const PROMISE_CHAIN_CALLS: readonly string[] = [
  '.then(',
  '.catch(',
  '.finally(',
];

//endregion Fixtures

//region Helpers

/**
 Read one product module as lines of code, with comment lines blanked.

 @param file - file name inside the source directory

 @returns module text split into lines, so a reported line number matches the file a reader opens

 @example
 ```ts
 await moduleCodeLines('index.ts');
 ```
 */
async function moduleCodeLines(file: string,): Promise<readonly string[]> {
  /**
   Module text as written.
   */
  const text = await readFile(
    join(SOURCE_DIRECTORY, file,),
    'utf8',
  );
  return text
    .split('\n',)
    .map(function withoutCommentLines(line,) {
      /**
       Whether this line opens with a comment marker.
       */
      const opensComment = COMMENT_LINE_OPENERS.some(function startsWith(opener,) {
        return line.trimStart()
          .startsWith(opener,);
      },);
      return opensComment ? '' : line;
    },);
}

/**
 Code lines of one product module, for the guards to count.

 @param lines - module lines with comment lines blanked

 @returns lines that still hold code, so a guard can prove its scan read something

 @example
 ```ts
 codeLines(await moduleCodeLines('index.ts'));
 ```
 */
function codeLines(lines: readonly string[],): readonly string[] {
  return lines.filter(function holdsCode(line,) {
    return line.trim() !== '';
  },);
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'promise chaining',
      children: [
        it({
          name: 'awaits every promise instead of chaining a fulfillment or rejection handler',
          fn: async () => {
            /**
             Module lines with comment lines blanked.
             */
            const lines = await moduleCodeLines(PRODUCT_MODULE,);
            /**
             One report line per promise-chain call, naming the line it sits on.
             */
            const reports = lines
              .flatMap(function reportChains(line, index,) {
                return PROMISE_CHAIN_CALLS
                  .filter(function chains(call,) {
                    return line.includes(call,);
                  },)
                  .map(function toReport(call,) {
                    return [
                      `${PRODUCT_MODULE}:${String(index + 1,)}`,
                      `chains ${call}`,
                      '-- await the promise and catch what it rejects with',
                    ].join(' ',);
                  },);
              },);

            // A scan that read no code proves nothing about the module.
            expect(codeLines(lines,).length,).toBeGreaterThan(0,);
            expect(reports.join('\n',),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
