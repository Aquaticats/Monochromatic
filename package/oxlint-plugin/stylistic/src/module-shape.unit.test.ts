/**
 Guards on this package's product-module naming that no lint rule enforces.

 `AGENTS.md` and the repository review checklist ban a single-letter variable
 name outside a math formula, and `id-length` is set to `allow` in
 `package/config/oxlint/src/rule/style.ts`, so a helper taking `s: string` passes
 every configured rule. Each product module's text is read here and its function
 parameter lists checked directly. Comment content is dropped first, because the
 TSDoc `@example` blocks in this package legitimately show `(a, b)` pairs.

 The scan covers `function` declarations and function expressions; a parameter
 list of an arrow function is not read. It has no math exemption either, because
 no text tells a formula variable from an abbreviation, so a helper that really
 computes with one takes a deliberate change to this guard, never a silent pass.

 @module
 */

import type { Dirent, } from 'node:fs';

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

//region Types

/**
 One parameter whose name says nothing about what it carries.
 */
type SingleCharacterParameter = {
  /**
   The single-character name as declared.
   */
  readonly name: string;
  /**
   Line the `function` keyword sits on, 1-based.
   */
  readonly line: number;
};

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

//endregion Types

//region Fixtures

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Extension of every file the scan reads.
 */
const MODULE_FILE_SUFFIX = '.ts';

/**
 Infix marking a file as a test rather than a product module.
 */
const TEST_FILE_INFIX = '.test.';

/**
 Characters that continue an identifier, so a name is never read inside a longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Keyword opening a function declaration, generator or plain.
 */
const FUNCTION_KEYWORD = 'function';

/**
 Marker turning a function into a generator.
 */
const GENERATOR_MARKER = '*';

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
 Opening bracket of a parameter list.
 */
const PARAMETER_LIST_OPENER = '(';

/**
 Closing bracket of a parameter list.
 */
const PARAMETER_LIST_CLOSER = ')';

/**
 Separator between two parameters of one list.
 */
const PARAMETER_SEPARATOR = ',';

/**
 Opening bracket of a function's type parameter list.
 */
const TYPE_PARAMETER_LIST_OPENER = '<';

/**
 Closing bracket of a function's type parameter list.
 */
const TYPE_PARAMETER_LIST_CLOSER = '>';

/**
 Character an arrow's greater-than sign follows, which closes no type parameter list.
 */
const ARROW_EQUALS = '=';

/**
 Prefix marking a rest parameter.
 */
const SPREAD_PREFIX = '...';

/**
 Quote characters opening a string, a template, or a quoted literal.
 */
const STRING_QUOTES: ReadonlySet<string> = new Set([
  '\'',
  '"',
  '`',
],);

/**
 Character escaping the next one inside a string literal.
 */
const ESCAPE_CHARACTER = '\\';

/**
 Bracket pairs whose contents never split a parameter list.
 */
const NESTED_BRACKETS: ReadonlyMap<string, string> = new Map([
  [PARAMETER_LIST_OPENER, PARAMETER_LIST_CLOSER,],
  ['[', ']',],
  ['{', '}',],
  [TYPE_PARAMETER_LIST_OPENER, TYPE_PARAMETER_LIST_CLOSER,],
],);

/**
 Length of a parameter name that carries no meaning of its own.
 */
const MEANINGLESS_NAME_LENGTH = 1;

/**
 Column no scan reaches, reported when a bracket pair never closes, a search
 finds nothing, or a text ends first.
 */
const NOT_FOUND_COLUMN = -1;

/**
 Module text holding a single-character parameter, a comment that mentions one,
 a generic function, and a literal that reads like a parameter list, so the scan
 is shown to tell them apart.
 */
const SCANNER_FIXTURE = [
  '/**',
  ' Takes the tag off a line.',
  '',
  ' @param s - line text',
  ' */',
  'export function untagged(line: string,): string {',
  '  return line;',
  '}',
  '',
  'export function broken(s: string,): string {',
  '  return s;',
  '}',
  '',
  'export function generic<Value,>(value: Value,): Value {',
  '  return value;',
  '}',
  '',
  '// function commented(s: string,) {}',
  'const message = \'function quoted(s: string,)\';',
  '',
].join('\n',);

/**
 Line the fixture's single-character parameter is declared on, found in the
 fixture text so the assertion follows the fixture rather than a count.
 */
const SCANNER_FIXTURE_OFFENDER_LINE = SCANNER_FIXTURE
  .split('\n',)
  .findIndex(function declaresOffender(line,) {
    return line.startsWith(`export ${FUNCTION_KEYWORD} broken(`,);
  },) + 1;

//endregion Fixtures

//region Helpers

/**
 Read the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none

 @example
 ```ts
 identifierPrefix('untagged(line)');
 ```
 */
function identifierPrefix(text: string,): string {
  /**
   Column of the first character that cannot continue an identifier, or -1 when
   every character can.
   */
  const end = text.split('',).findIndex(function endsIdentifier(character,) {
    return !(IDENTIFIER_CHARACTERS.includes(character,));
  },);
  return (end === NOT_FOUND_COLUMN) ? text : text.slice(0, end,);
}

/**
 Skip whitespace from a column.

 @param text - module text being scanned

 @param from - column to start at

 @returns column of the first non-whitespace character, or `text.length`
 */
function skipWhitespace(text: string, from: number,): number {
  /**
   Column walked forward over whitespace.
   */
  let cursor = from;
  while ((cursor < text.length) && (text[cursor]?.trim() === ''))
    cursor += 1;
  return cursor;
}

/**
 Skip a string, template, or quoted literal opened at a column.

 @param text - module text being scanned

 @param from - column of the opening quote

 @returns column just past the closing quote, or `text.length` when unclosed
 */
function skipStringLiteral(text: string, from: number,): number {
  /**
   Quote character that closes the literal.
   */
  const quote = text[from];
  for (let cursor = from + 1; cursor < text.length; cursor += 1) {
    if (text[cursor] === ESCAPE_CHARACTER) {
      cursor += 1;
      continue;
    }
    if (text[cursor] === quote)
      return cursor + 1;
  }
  return text.length;
}

/**
 Split a parameter list into its top-level parts, ignoring separators nested in
 a bracket pair or written inside a literal.

 @param text - text between the parameter list brackets

 @returns parameter texts as written, without surrounding whitespace

 @example
 ```ts
 splitParameters('value: Value, options?: Options');
 ```
 */
function splitParameters(text: string,): readonly string[] {
  /**
   Parameter texts collected so far.
   */
  const parameters: string[] = [];
  /**
   Columns of brackets still open, innermost last.
   */
  const openBrackets: string[] = [];
  /**
   Column the current parameter starts at.
   */
  let start = 0;
  /**
   Column walked forward through the list.
   */
  let cursor = 0;
  while (cursor < text.length) {
    /**
     Character at the cursor.
     */
    const character = text[cursor] ?? '';
    if (STRING_QUOTES.has(character,)) {
      cursor = skipStringLiteral(text, cursor,);
      continue;
    }
    /**
     Bracket closing the innermost open one, when the cursor sits on it.
     */
    const innermost = openBrackets.at(-1,);
    if ((innermost !== undefined) && (NESTED_BRACKETS.get(innermost,) === character)) {
      openBrackets.pop();
      cursor += 1;
      continue;
    }
    if (NESTED_BRACKETS.has(character,)) {
      openBrackets.push(character,);
      cursor += 1;
      continue;
    }
    if ((character === PARAMETER_SEPARATOR) && (openBrackets.length === 0)) {
      parameters.push(text.slice(start, cursor,).trim(),);
      start = cursor + 1;
    }
    cursor += 1;
  }
  /**
   Text after the last separator, which is a parameter unless the list is empty.
   */
  const tail = text.slice(start,).trim();
  if (tail !== '')
    parameters.push(tail,);
  return parameters;
}

/**
 Column of the parenthesis closing the one opened at a column.

 @param text - module text being scanned

 @param from - column of the opening parenthesis

 @returns column of the matching closing parenthesis, or `NOT_FOUND_COLUMN`
 */
function matchingParenthesisColumn(text: string, from: number,): number {
  for (let cursor = from, depth = 0; cursor < text.length; cursor += 1) {
    /**
     Character at the cursor.
     */
    const character = text[cursor] ?? '';
    if (STRING_QUOTES.has(character,)) {
      cursor = skipStringLiteral(text, cursor,) - 1;
      continue;
    }
    if (character === PARAMETER_LIST_OPENER)
      depth += 1;
    if (character === PARAMETER_LIST_CLOSER) {
      depth -= 1;
      if (depth === 0)
        return cursor;
    }
  }
  return NOT_FOUND_COLUMN;
}

/**
 Column of the angle bracket closing the type parameter list opened at a column.

 An arrow's `=>` closes nothing, so a `>` that follows `=` is skipped.

 @param text - module text being scanned

 @param from - column of the opening angle bracket

 @returns column of the matching closing angle bracket, or `NOT_FOUND_COLUMN`
 */
function matchingAngleColumn(text: string, from: number,): number {
  for (let cursor = from, depth = 0; cursor < text.length; cursor += 1) {
    /**
     Character at the cursor.
     */
    const character = text[cursor] ?? '';
    if (STRING_QUOTES.has(character,)) {
      cursor = skipStringLiteral(text, cursor,) - 1;
      continue;
    }
    if (character === TYPE_PARAMETER_LIST_OPENER)
      depth += 1;
    if ((character === TYPE_PARAMETER_LIST_CLOSER) && (text[cursor - 1] !== ARROW_EQUALS)) {
      depth -= 1;
      if (depth === 0)
        return cursor;
    }
  }
  return NOT_FOUND_COLUMN;
}

/**
 Column just past a function's type parameter list, which starts at a column.

 @param text - module text being scanned

 @param from - column the type parameter list may open at

 @returns column past the list, `from` when no list opens, or `NOT_FOUND_COLUMN`
 */
function columnAfterTypeParameters(text: string, from: number,): number {
  if (text[from] !== TYPE_PARAMETER_LIST_OPENER)
    return from;
  /**
   Column of the closing angle bracket, or `NOT_FOUND_COLUMN`.
   */
  const closeAngle = matchingAngleColumn(text, from,);
  return (closeAngle === NOT_FOUND_COLUMN) ? NOT_FOUND_COLUMN : closeAngle + 1;
}

/**
 Name a parameter declares, with any rest prefix removed.

 @param parameter - one parameter text as written

 @returns declared identifier, empty for a destructured parameter
 */
function parameterName(parameter: string,): string {
  /**
   Parameter text without its rest prefix.
   */
  const rest = parameter.startsWith(SPREAD_PREFIX,)
    ? parameter.slice(SPREAD_PREFIX.length,)
    : parameter;
  return identifierPrefix(rest.trimStart(),);
}

/**
 Whether a parameter name says nothing about what the parameter carries.

 @param name - name as declared, empty for a destructured parameter

 @returns true when the name is one character long
 */
function isMeaninglessName(name: string,): boolean {
  return name.length === MEANINGLESS_NAME_LENGTH;
}

/**
 Single-character parameter names a `function` keyword introduces.

 @param text - comment-free module text

 @param from - column just past the `function` keyword

 @returns offending names in declaration order, empty when the keyword opens no list

 @example
 ```ts
 meaninglessParameterNamesAfter('function broken(s: string,) {', 8);
 ```
 */
function meaninglessParameterNamesAfter(text: string, from: number,): readonly string[] {
  /**
   Column of the first non-whitespace character after the keyword.
   */
  const keywordEnd = skipWhitespace(text, from,);
  /**
   Column after the generator marker, when one follows the keyword.
   */
  const markerEnd = (text[keywordEnd] === GENERATOR_MARKER)
    ? skipWhitespace(text, keywordEnd + 1,)
    : keywordEnd;
  /**
   Column after the function name, which an anonymous function leaves in place.
   */
  const nameEnd = markerEnd + identifierPrefix(text.slice(markerEnd,),).length;
  /**
   Column just past the type parameter list, or `NOT_FOUND_COLUMN`.
   */
  const typeParameterEnd = columnAfterTypeParameters(text, skipWhitespace(text, nameEnd,),);
  if (typeParameterEnd === NOT_FOUND_COLUMN)
    return [];
  /**
   Column the parameter list opens at.
   */
  const listOpener = skipWhitespace(text, typeParameterEnd,);
  if (text[listOpener] !== PARAMETER_LIST_OPENER)
    return [];
  /**
   Column the parameter list closes at, or `NOT_FOUND_COLUMN`.
   */
  const listCloser = matchingParenthesisColumn(text, listOpener,);
  if (listCloser === NOT_FOUND_COLUMN)
    return [];
  return splitParameters(text.slice(listOpener + 1, listCloser,),)
    .map(parameterName,)
    .filter(isMeaninglessName,);
}

/**
 Single-character parameter names a comment-free module text declares.

 @param text - module text with comment content removed

 @returns one record per offending parameter, in source order

 @example
 ```ts
 singleCharacterParameterNames('export function broken(s: string,) {}');
 ```
 */
function singleCharacterParameterNames(text: string,): readonly SingleCharacterParameter[] {
  /**
   Offending parameters found so far.
   */
  const offenders: SingleCharacterParameter[] = [];
  /**
   Column walked forward through the module text.
   */
  let cursor = 0;
  while (cursor < text.length) {
    /**
     Character at the cursor.
     */
    const character = text[cursor] ?? '';
    if (STRING_QUOTES.has(character,)) {
      cursor = skipStringLiteral(text, cursor,);
      continue;
    }
    if (!(IDENTIFIER_CHARACTERS.includes(character,))) {
      cursor += 1;
      continue;
    }
    /**
     Whole identifier starting at the cursor.
     */
    const word = identifierPrefix(text.slice(cursor,),);
    if (word === FUNCTION_KEYWORD) {
      for (const name of meaninglessParameterNamesAfter(text, cursor + word.length,)) {
        offenders.push({
          name,
          line: text.slice(0, cursor,).split('\n',).length,
        },);
      }
    }
    cursor += word.length;
  }
  return offenders;
}

/**
 Append one line to the comment-free accumulator.

 @param state - lines collected so far and whether a block comment is open

 @param line - next line of module text as written

 @returns the accumulator with this line appended

 @example
 ```ts
 lines.reduce(stripComments, { lines: [], insideBlockComment: false });
 ```
 */
function stripComments(state: StripState, line: string,): StripState {
  if (state.insideBlockComment) {
    return {
      lines: [...state.lines, '',],
      insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER,)),
    };
  }
  /**
   Column where a block comment opens on this line, or -1.
   */
  const blockOpener = line.indexOf(BLOCK_COMMENT_OPENER,);
  if (blockOpener === NOT_FOUND_COLUMN) {
    return {
      lines: [...state.lines, line.trimStart().startsWith(LINE_COMMENT_OPENER,) ? '' : line,],
      insideBlockComment: false,
    };
  }
  return {
    lines: [...state.lines, line.slice(0, blockOpener,),],
    insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER, blockOpener,)),
  };
}

/**
 Drop every comment from a module text, keeping every line so a reported line
 matches the file a reader opens.

 @param text - module text as written

 @returns module text with comment content removed

 @example
 ```ts
 withoutComments('/** takes no tag *\/ const untagged = 1;');
 ```
 */
function withoutComments(text: string,): string {
  return text
    .split('\n',)
    .reduce<StripState>(
    function step(state, line,) {
      return stripComments(state, line,);
    },
    { lines: [], insideBlockComment: false, },
    ).lines
    .join('\n',);
}

/**
 Read one module and drop its comments, keeping every line so a reported line
 matches the file a reader opens.

 @param path - absolute path of the module

 @returns module text with comment content removed

 @example
 ```ts
 await commentFreeText('/repo/package/oxlint-plugin/stylistic/src/index.ts');
 ```
 */
async function commentFreeText(path: string,): Promise<string> {
  return withoutComments(await readFile(path, 'utf8',),);
}

/**
 Whether a directory entry is a product module of this package.

 @param entry - entry of a folder below the source directory

 @returns true for a TypeScript file that is not a test
 */
function isProductModule(entry: Dirent,): boolean {
  if (!(entry.isFile()))
    return false;
  if (!(entry.name.endsWith(MODULE_FILE_SUFFIX,)))
    return false;
  return !(entry.name.includes(TEST_FILE_INFIX,));
}

/**
 Whether a directory entry is a folder holding more product modules.

 @param entry - entry of a folder below the source directory

 @returns true for a folder
 */
function isSubfolder(entry: Dirent,): boolean {
  return entry.isDirectory();
}

/**
 Absolute paths of this package's product modules: every TypeScript file below
 the source directory that is not itself a test.

 @param directory - folder to walk

 @returns sorted absolute paths scanned by the guard below

 @example
 ```ts
 await productModules(SOURCE_DIRECTORY);
 ```
 */
async function productModules(directory: string,): Promise<readonly string[]> {
  /**
   Entries of the folder being walked.
   */
  const entries = await readdir(directory, { withFileTypes: true, },);
  /**
   Product modules in this folder, beside the subfolders walked below.
   */
  const found = entries
    .filter(isProductModule,)
    .map(function toPath(entry,) {
      return join(directory, entry.name,);
    },);
  /**
   Subfolders of this folder, which hold the rest of the package.
   */
  const subfolders = entries
    .filter(isSubfolder,)
    .map(function toPath(entry,) {
      return join(directory, entry.name,);
    },);
  /**
   Product modules found in each subfolder.
   */
  const nested = await Promise.all(subfolders.map(productModules,),);
  return [...found, ...nested.flat(),].toSorted();
}

//endregion Helpers

await describe({
  name: '',
  children: [
    //region scan self-checks

    describe({
      name: singleCharacterParameterNames.name,
      children: [
        it({
          name: 'reports a declared single-character parameter and ignores comment prose',
          fn: async () => {
            expect(
              singleCharacterParameterNames(withoutComments(SCANNER_FIXTURE,),),
            ).toStrictEqual([
              { name: 's', line: SCANNER_FIXTURE_OFFENDER_LINE, },
            ],);
          },
        },),
        it({
          name: 'reports nothing for a module whose parameters all carry role names',
          fn: async () => {
            /**
             Module text whose every parameter is named for its role.
             */
            const text = [
              'export function untagged(line: string,): string {',
              '  return line;',
              '}',
            ].join('\n',);
            /**
             Offending names the scan finds in that module.
             */
            const names = singleCharacterParameterNames(withoutComments(text,),);
            expect(names,).toStrictEqual([],);
          },
        },),
      ],
    },),

    //endregion scan self-checks

    //region product modules

    describe({
      name: 'product module parameter names',
      children: (await productModules(SOURCE_DIRECTORY,)).map(function mapModule(path,) {
        /**
         Module name relative to the source directory, as a reader would write it.
         */
        const name = path.slice(SOURCE_DIRECTORY.length + 1,);
        return it({
          name: `names every ${name} parameter with a role word`,
          fn: async () => {
            /**
             Module text with comment content removed.
             */
            const text = await commentFreeText(path,);
            expect(
              singleCharacterParameterNames(text,).map(function toReport(offender,) {
                return `${name}:${String(offender.line,)} names a parameter ${offender.name}`;
              },).join('\n',),
            ).toBe('',);
          },
        },);
      },),
    },),

    //endregion product modules
  ],
},);
