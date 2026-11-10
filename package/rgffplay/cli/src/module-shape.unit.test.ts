/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule PP1 requires `async`/`await` only and forbids a promise chain,
 and the ban on single-letter names for values that are not mathematical has no
 linter behind it either, so the product module's text is read here and both are
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

/**
 Keyword that opens a function declaration or expression, with its trailing
 space so a member named like the keyword does not match.
 */
const FUNCTION_KEYWORD = 'function ';

/**
 Characters that continue an identifier, so a name is never read inside a
 longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Parameter name the positive control declares, which the guard below must
 report.
 */
const SINGLE_LETTER_PARAMETER_NAME = 'w';

/**
 Line declaring one parameter named by {@link SINGLE_LETTER_PARAMETER_NAME},
 which no product module may hold.
 */
const SINGLE_LETTER_PARAMETER_LINE = `function toBracket(${SINGLE_LETTER_PARAMETER_NAME},) {`;

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

/**
 Text before the first occurrence of a separator.

 @param text - text to cut

 @param separator - character the cut happens at

 @returns text before that character, whole text when it holds none

 @example
 ```ts
 beforeFirst({ text: 'word: string', separator: ':' });
 ```
 */
function beforeFirst({
  text,
  separator,
}: {
  readonly text: string;
  readonly separator: string;
},): string {
  /**
   Column of the first separator, -1 when the text holds none.
   */
  const at = text.indexOf(separator,);
  return at === (-1) ? text : text.slice(0, at,);
}

/**
 Whether a text names an identifier and nothing else.

 A destructured parameter, a rest parameter and a type annotation all leave
 characters that fail this test, so only plain names are read.

 @param text - candidate name

 @returns true when every character continues an identifier and the text is not empty

 @example
 ```ts
 isIdentifier('word') // true
 isIdentifier('{ glob') // false
 ```
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
 Parameter names a line declares, one per parameter of every function opened on
 that line.

 A parameter list spanning several lines contributes only what stands on the
 line that opens it, which is where a function taking one parameter keeps it.

 @param line - one line of module code, comment lines already dropped

 @returns parameter names, with type annotations and default values dropped

 @example
 ```ts
 parameterNames('function toBracket(word,) {') // ['word']
 ```
 */
function parameterNames(line: string,): readonly string[] {
  return line
    .split(FUNCTION_KEYWORD,)
    .slice(1,)
    .flatMap(function parametersOpenedHere(fragment,) {
      /**
       Column of the parenthesis opening the parameter list, -1 when this
       fragment opens none.
       */
      const parametersAt = fragment.indexOf('(',);
      if (parametersAt === (-1))
        return [];
      /**
       Column of the parenthesis closing the parameter list, -1 when the list
       runs past this line.
       */
      const listEnd = fragment.indexOf(')', parametersAt,);
      /**
       Parameter list as written on this line.
       */
      const list = (listEnd === (-1))
        ? fragment.slice(parametersAt + 1,)
        : fragment.slice(parametersAt + 1, listEnd,);
      return list
        .split(',',)
        .map(function nameOf(parameter,) {
          /**
           Parameter text before any type annotation and default value.
           */
          const annotated = beforeFirst({ text: parameter, separator: ':', },);
          return beforeFirst({ text: annotated, separator: '=', },)
            .trim();
        },)
        .filter(isIdentifier,);
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
    describe({
      name: 'parameter names',
      children: [
        it({
          name: 'reports a single-letter parameter in a line it is handed',
          fn: async () => {
            expect(parameterNames(SINGLE_LETTER_PARAMETER_LINE,),)
              .toEqual([SINGLE_LETTER_PARAMETER_NAME,],);
          },
        },),
        it({
          name: 'names every function parameter after the value it holds',
          fn: async () => {
            /**
             Module lines with comment lines blanked.
             */
            const lines = await moduleCodeLines(PRODUCT_MODULE,);
            /**
             Parameter names the scan read, so it can prove the module declares any at all.
             */
            const readNames = lines.flatMap(function reportDeclarations(line,) {
              return parameterNames(line,);
            },);
            /**
             One report line per single-letter parameter, naming the line it sits on.
             */
            const reports = lines
              .flatMap(function reportLine(line, index,) {
                return parameterNames(line,)
                  .filter(function isSingleLetter(name,) {
                    return name.length
                      === 1;
                  },)
                  .map(function toReport(name,) {
                    return [
                      `${PRODUCT_MODULE}:${String(index + 1,)}`,
                      `declares parameter "${name}";`,
                      'name it after the value it holds',
                    ].join(' ',);
                  },);
              },);

            // A scan that read no parameter proves nothing about the module.
            expect(readNames.length,).toBeGreaterThan(0,);
            expect(reports.join('\n',),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
