/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule PP1 requires `async`/`await` only and forbids a promise chain,
 and the ban on single-letter names for values that are not mathematical has no
 linter behind it either, so every product module's text is read here and both
 are checked directly.

 Comment content is blanked before either scan, because a name quoted in a TSDoc
 example is prose rather than code. The parameter-name scan reads what a line
 writes between the parentheses of a `function` and before an arrow, which is
 where this package declares its parameters; a name inside a destructured
 pattern, and a name a type annotation declares, are outside its reach.

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

//region Fixtures

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

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

/**
 Promise-chain calls `PP1` forbids, each written as the leading dot and the
 opening parenthesis so a mention of the method's name alone does not match.
 */
const PROMISE_CHAIN_CALLS: readonly string[] = [
  '.then(',
  '.catch(',
  '.finally(',
];

/**
 Keyword that opens a function declaration or expression, with its trailing
 space so a member named like the keyword does not match.
 */
const FUNCTION_KEYWORD = 'function ';

/**
 Operator that ends an arrow function's parameter list.
 */
const ARROW = '=>';

/**
 Characters that continue an identifier, so a name is never read inside a longer
 one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Parameter name the positive controls declare, which the guard must report.
 */
const SINGLE_LETTER_PARAMETER_NAME = 'w';

/**
 Line declaring one function parameter named by
 {@link SINGLE_LETTER_PARAMETER_NAME}.
 */
const SINGLE_LETTER_FUNCTION_LINE = `function toBracket(${SINGLE_LETTER_PARAMETER_NAME},) {`;

/**
 Line declaring one arrow parameter named by
 {@link SINGLE_LETTER_PARAMETER_NAME}.
 */
const SINGLE_LETTER_ARROW_LINE = `[1, 2,].map((${SINGLE_LETTER_PARAMETER_NAME},) => ${SINGLE_LETTER_PARAMETER_NAME},);`;

//endregion Fixtures

//region Helpers

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
 Append one line to the comment-free accumulator.

 This repository's TSDoc bodies carry no leading asterisk, so a block comment is
 tracked by its markers rather than by each line's first character.

 @param state - lines collected so far and whether a block comment is still open

 @param line - next line of module text as written

 @returns the accumulator with this line appended

 @example
 ```ts
 lines.reduce(stripComments, { insideBlockComment: false, lines: [], });
 ```
 */
function stripComments(state: StripState, line: string,): StripState {
  if (state.insideBlockComment)
    return {
      lines: [...state.lines, '',],
      insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER,)),
    };
  /**
   Column where a block comment opens on this line, or -1.
   */
  const blockOpener = line.indexOf(BLOCK_COMMENT_OPENER,);
  if (blockOpener === (-1)) {
    return {
      lines: [...state.lines, line.trimStart()
        .startsWith(LINE_COMMENT_OPENER,) ? '' : line,],
      insideBlockComment: false,
    };
  }
  return {
    lines: [...state.lines, line.slice(0, blockOpener,),],
    insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER, blockOpener,)),
  };
}

/**
 Read one product module and blank its comments, keeping every line number so a
 reported line matches the file a reader opens.

 @param file - file name inside the source directory

 @returns module text split into lines, with comment content removed

 @example
 ```ts
 await moduleLines('index.ts');
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
      return stripComments(state, line,);
    },
    { lines: [], insideBlockComment: false, },
    ).lines;
}

/**
 File names of this package's product modules: every TypeScript file beside this
 test that is not itself a test.

 @returns sorted file names both guards scan

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
      return name.endsWith('.ts',) && (!(name.endsWith('.test.ts',)));
    },)
    .toSorted();
}

/**
 Lines that still hold code, so a guard can prove its scan read something.

 @param lines - module lines with comment content removed

 @returns the non-blank lines
 */
function codeLines(lines: readonly string[],): readonly string[] {
  return lines.filter(function holdsCode(line,) {
    return line.trim() !== '';
  },);
}

/**
 Whether a text names an identifier and nothing else.

 A destructured pattern, a rest parameter and a type annotation all leave
 characters that fail this test, so only plain names are read.

 @param text - candidate name

 @returns true when every character continues an identifier and the text is not empty
 */
function isIdentifier(text: string,): boolean {
  return (text.length
    > 0)
    && text
      .split('',)
      .every(function continuesIdentifier(character,) {
        return IDENTIFIER_CHARACTERS.includes(character,);
      },);
}

/**
 Name one fragment of a parameter list declares, with its type annotation and
 default value dropped.

 @param parameter - one fragment between two commas of a parameter list

 @returns the declared name, or the fragment itself when it declares none
 */
function parameterName(parameter: string,): string {
  /**
   Column where a type annotation opens, or -1.
   */
  const annotationAt = parameter.indexOf(':',);
  /**
   Parameter text before its type annotation.
   */
  const annotated = (annotationAt === (-1)) ? parameter : parameter.slice(0, annotationAt,);
  /**
   Column where a default value opens, or -1.
   */
  const defaultAt = annotated.indexOf('=',);
  return ((defaultAt === (-1)) ? annotated : annotated.slice(0, defaultAt,)).trim();
}

/**
 Names a parameter-list text declares.

 @param list - parameter list as written, without its parentheses

 @returns the declared names, one per parameter
 */
function readParameterNames(list: string,): readonly string[] {
  return list
    .split(',',)
    .map(parameterName,)
    .filter(isIdentifier,);
}

/**
 Parameter lists a line writes after a function keyword.

 @param line - one line of module code, comments already blanked

 @returns each parameter list as written, without its parentheses
 */
function functionParameterLists(line: string,): readonly string[] {
  return line
    .split(FUNCTION_KEYWORD,)
    .slice(1,)
    .flatMap(function listOpenedHere(fragment,) {
      /**
       Column of the parenthesis opening the list, or -1 when this fragment
       opens none.
       */
      const openedAt = fragment.indexOf('(',);
      if (openedAt === (-1))
        return [];
      /**
       Column of the parenthesis closing the list, or -1 when the list runs past
       this line.
       */
      const closedAt = fragment.indexOf(')', openedAt,);
      return [(closedAt === (-1))
        ? fragment.slice(openedAt + 1,)
        : fragment.slice(openedAt + 1, closedAt,),];
    },);
}

/**
 Parameter list a line writes before an arrow.

 @param line - one line of module code, comments already blanked

 @returns each parameter list as written, without its parentheses
 */
function arrowParameterLists(line: string,): readonly string[] {
  /**
   Text before each arrow on the line, which holds that arrow's parameters.
   */
  const beforeArrows = line
    .split(ARROW,)
    .slice(0, -1,);
  return beforeArrows.map(function listBefore(text,) {
    /**
     Text before the arrow with its trailing space dropped.
     */
    const trimmed = text.trimEnd();
    if (!(trimmed.endsWith(')',)))
      return trimmed;
    /**
     Column of the parenthesis opening the list, or -1 when none opens on this
     line.
     */
    const openedAt = trimmed.lastIndexOf('(',);
    return (openedAt === (-1)) ? '' : trimmed.slice(openedAt + 1, -1,);
  },);
}

/**
 Parameter names a line declares, one per parameter of every function and every
 arrow opened on that line.

 @param line - one line of module code, comments already blanked

 @returns parameter names, with type annotations and default values dropped

 @example
 ```ts
 parameterNames('function toBracket(word,) {') // ['word']
 ```
 */
function parameterNames(line: string,): readonly string[] {
  return [
    ...functionParameterLists(line,),
    ...arrowParameterLists(line,),
  ].flatMap(readParameterNames,);
}

//endregion Helpers

/**
 Product modules both guards scan, read once so the two guards cover the same
 files.
 */
const PRODUCT_MODULES = await productModules();

await describe({
  name: '',
  children: [
    describe({
      name: 'product modules',
      children: [
        it({
          name: 'finds the TypeScript files beside this test',
          fn: async () => {
            expect(PRODUCT_MODULES.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'promise chaining',
      children: PRODUCT_MODULES.map(function promiseChainCase(file,) {
        return it({
          name: `awaits every promise in ${file} instead of chaining a handler`,
          fn: async () => {
            /**
             Module lines with comment content removed.
             */
            const lines = await moduleLines(file,);
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
                      `${file}:${String(index + 1,)}`,
                      `chains ${call}`,
                      '-- await the promise and catch what it rejects with',
                    ].join(' ',);
                  },);
              },);

            // A scan that read no code proves nothing about the module.
            expect(codeLines(lines,).length,).toBeGreaterThan(0,);
            expect(reports.join('\n',),).toBe('',);
          },
        },);
      },),
    },),
    describe({
      name: 'parameter names',
      children: [
        it({
          name: 'reports a single-letter parameter of a function it is handed',
          fn: async () => {
            expect(parameterNames(SINGLE_LETTER_FUNCTION_LINE,),)
              .toEqual([SINGLE_LETTER_PARAMETER_NAME,],);
          },
        },),
        it({
          name: 'reports a single-letter parameter of an arrow it is handed',
          fn: async () => {
            expect(parameterNames(SINGLE_LETTER_ARROW_LINE,),)
              .toEqual([SINGLE_LETTER_PARAMETER_NAME,],);
          },
        },),
        ...PRODUCT_MODULES.map(function parameterNameCase(file,) {
          return it({
            name: `names every parameter in ${file} after the value it holds`,
            fn: async () => {
              /**
               Module lines with comment content removed.
               */
              const lines = await moduleLines(file,);
              /**
               Every parameter name the scan read, with the line it sits on.
               */
              const read = lines
                .flatMap(function namesOn(line, index,) {
                  return parameterNames(line,)
                    .map(function withLine(name,) {
                      return {
                        name,
                        line: index + 1,
                      };
                    },);
                },);
              /**
               One report line per single-letter parameter, naming the line it
               sits on.
               */
              const reports = read
                .filter(function isSingleLetter(entry,) {
                  return entry.name.length
                    === 1;
                },)
                .map(function toReport(entry,) {
                  return [
                    `${file}:${String(entry.line,)}`,
                    `declares parameter "${entry.name}";`,
                    'name it after the value it holds',
                  ].join(' ',);
                },);

              // A scan that read no parameter proves nothing about the module.
              expect(read.length,).toBeGreaterThan(0,);
              expect(reports.join('\n',),).toBe('',);
            },
          },);
        },),
      ],
    },),
  ],
},);
