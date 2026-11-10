/**
 Guards on this package's module shape, which no lint rule enforces.

 `.agents/skills/project-code-review/SKILL.md` requires `for...of` when iteration is unavoidable, a functional pattern
 otherwise, and never a classic `for` loop, and no linter behind that requirement reads this package. Every product
 module's text is read here and each line is checked directly, so the shape the review found thirteen times cannot come
 back unnoticed.

 Comment content is blanked before the scan, because a loop quoted in a TSDoc example is prose rather than code, and
 every line keeps its number so a report names the line a reader opens. Spaces are dropped before a header is matched,
 so a header is recognised however it is spaced; the scan sees a header that declares its counter with `let` or `var`,
 which is how every classic loop in this package was written. A header that declares nothing (`for (;;)`), a header
 split across lines, and a header spaced with tabs are outside its reach.

 `#1468` converted the thirteen classic loops the 2026-10-17 review found, and a module written the next day carried a
 fourteenth, so the shape is guarded here rather than swept for again.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Characters that continue an identifier, so the keyword of a loop is never matched inside a longer word.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Openers of a classic loop header with its spaces removed, one per keyword that declares a counter.
 */
const CLASSIC_FOR_HEADERS: readonly string[] = ['for(let', 'for(var',];

/**
 Opening of a block comment.
 */
const BLOCK_COMMENT_OPENER = '/*';

/**
 Closing of a block comment.
 */
const BLOCK_COMMENT_CLOSER = '*/';

/**
 Opening of a line comment.
 */
const LINE_COMMENT_OPENER = '//';

//endregion Fixtures

//region Helpers

/**
 Whether a line opens a classic `for` loop, which is the shape the standard sends to `for...of` or a counted range.

 @param line - one line of module text with its comments removed

 @returns true when the line carries a header that declares its counter

 @example
 ```ts
 opensClassicForLoop('  for (let index = 0; index < 3; index += 1) {'); // true
 ```
 */
function opensClassicForLoop(line: string,): boolean {
  /**
   Line without its spaces, so a header is matched however the formatter spaced it.
   */
  const compact = line.replaceAll(' ', '',);
  return CLASSIC_FOR_HEADERS.some(function standsAlone(header,) {
    /**
     Column the header starts at, or -1 when the line carries none.
     */
    const start = compact.indexOf(header,);
    if (start === (-1))
      return false;
    /**
     Character touching the keyword's first letter, absent when the header opens the line.
     */
    const before = compact[start - 1];
    return (before === undefined) || (!(IDENTIFIER_CHARACTERS.includes(before,)));
  },);
}

/**
 Accumulated comment-free lines and whether a block comment is still open.
 */
type StripState = {
  /**
   Lines collected so far, with comment content removed.
   */
  readonly lines: readonly string[];
  /**
   Whether the previous line left a block comment open.
   */
  readonly insideBlockComment: boolean;
};

/**
 Blank the comment content of one line, keeping the line so a reported number matches the file a reader opens.

 @param state - lines collected so far and whether a block comment is open

 @param line - next line of module text as written

 @returns the accumulator with this line blanked or appended

 @example
 ```ts
 lines.reduce(blankComments, { lines: [], insideBlockComment: false, });
 ```
 */
function blankComments(state: StripState, line: string,): StripState {
  if (state.insideBlockComment)
    return {
      lines: [...state.lines, '',],
      insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER,)),
    };
  /**
   Column where a block comment opens on this line, or -1 when it opens none.
   */
  const blockOpener = line.indexOf(BLOCK_COMMENT_OPENER,);
  if (blockOpener === (-1))
    return {
      lines: [...state.lines, line.trimStart().startsWith(LINE_COMMENT_OPENER,) ? '' : line,],
      insideBlockComment: false,
    };
  return {
    lines: [...state.lines, line.slice(
      0,
      blockOpener,
    ),],
    insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER, blockOpener,)),
  };
}

/**
 Read one product module and blank its comments, keeping every line number.

 @param file - file name inside the source directory

 @returns module text split into lines, with comment content removed

 @example
 ```ts
 await moduleLines('poll.ts');
 ```
 */
async function moduleLines(file: string,): Promise<readonly string[]> {
  /**
   Module text as written.
   */
  const text = await readFile(
    join(SOURCE_DIRECTORY, file,),
    'utf8',
  );
  return text
    .split('\n',)
    .reduce<StripState>(
    function step(state, line,) {
      return blankComments(state, line,);
    },
    { lines: [], insideBlockComment: false, },
    ).lines;
}

/**
 File names of this package's product modules: every TypeScript file beside this test that is not itself a test.

 @returns sorted file names the guard scans

 @example
 ```ts
 await productModules();
 ```
 */
async function productModules(): Promise<readonly string[]> {
  /**
   Directory entries beside this test.
   */
  const entries = await readdir(SOURCE_DIRECTORY,);
  return entries
    .filter(function isProductModule(name,) {
      return (name.endsWith('.ts',)) && (!(name.endsWith('.test.ts',)));
    },)
    .toSorted();
}

/**
 Classic loop headers in one module's comment-free lines, one report line each.

 @param file - file name inside the source directory, which a report names

 @param lines - that module's text with comment content removed

 @returns one report per classic loop, empty when the module opens none

 @example
 ```ts
 classicForLoops({ file: 'wide-playback.ts', lines, });
 ```
 */
function classicForLoops(
  {
    file,
    lines,
  }: {
    readonly file: string;
    readonly lines: readonly string[];
  },
): readonly string[] {
  return lines.flatMap(function reportLine(line, index,) {
    if (!(opensClassicForLoop(line,)))
      return [];
    return [
      [
        `${file}:${String(index + 1,)}`,
        'opens a classic for loop;',
        'iterate with for...of, or count a fixed number of steps with numbersBelow',
      ].join(' ',),
    ];
  },);
}

//endregion Helpers

/**
 Product modules the guard scans, read once.
 */
const PRODUCT_MODULES = await productModules();

await describe({
  name: '',
  children: [
    describe({
      name: 'the scan',
      children: [
        it({
          name: 'reports a header the formatter writes, with its counter declared by let',
          fn: async ({ expect, }) => {
            expect(opensClassicForLoop('  for (let index = 0; index < 3; index += 1) {',),).toBe(true,);
          },
        },),
        it({
          name: 'reports a header whose counter is declared by var',
          fn: async ({ expect, }) => {
            expect(opensClassicForLoop('  for (var step = 0; step < 2; step += 1) {',),).toBe(true,);
          },
        },),
        it({
          name: 'reports a header written without a space after the keyword',
          fn: async ({ expect, }) => {
            expect(opensClassicForLoop('  for(let step = 0; step < 2; step += 1) {',),).toBe(true,);
          },
        },),
        it({
          name: 'leaves a for...of walk, a for...in walk, and a name that ends in for alone',
          fn: async ({ expect, }) => {
            expect(opensClassicForLoop('  for (const rowOffset of numbersBelow(height,)) {',),).toBe(false,);
            expect(opensClassicForLoop('  for (const [index, line,] of lines.entries()) {',),).toBe(false,);
            expect(opensClassicForLoop('  for (const key in record) {',),).toBe(false,);
            expect(opensClassicForLoop('  const counted = usefor(letCount,);',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'product modules',
      children: [
        it({
          name: 'finds the TypeScript files beside this test',
          fn: async ({ expect, }) => {
            expect(PRODUCT_MODULES.length,).toBeGreaterThan(0,);
          },
        },),
        ...PRODUCT_MODULES.map(function classicForLoopCase(file,) {
          return it({
            name: `opens no classic for loop in ${file}`,
            fn: async ({ expect, }) => {
              /**
               Module text with comment content removed.
               */
              const lines = await moduleLines(file,);
              /**
               One report line per classic loop header.
               */
              const reports = classicForLoops({
                file,
                lines,
              },);

              // A scan that read no code proves nothing about the module.
              expect(lines.length,).toBeGreaterThan(0,);
              expect(reports.join('\n',),).toBe('',);
            },
          },);
        },),
      ],
    },),
  ],
},);
