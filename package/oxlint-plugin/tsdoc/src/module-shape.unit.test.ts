/**
 Guards this package's function parameter names, which no lint rule enforces.

 `AGENTS.md` and the repository review checklist ban a single-letter variable
 name outside a math formula, and `id-length` is set to `allow` in
 `package/config/oxlint/src/rule/style.ts`, so a helper taking `s: string` passes
 every configured rule. That is how the sweep behind `#1478` found nine such
 helpers across the tsdoc and stylistic packages: nothing read the names, so the
 abbreviation came back. Every product module's comment-free text is read here
 and each `function` parameter list is checked directly.

 This package keeps its modules in two folders, `src` beside this test and the
 rule modules under `src/rule`, so the walk is recursive; the guards in flat
 packages (`package/oxlint-plugin/stylistic/src/module-shape.unit.test.ts`,
 `package/llm/token-count/src/module-shape.unit.test.ts`) read only their own
 folder. `*.test.ts` files are left out: a test's fixtures legitimately quote the
 single-letter parameter the guard exists to find.

 The scan reads `function` declarations and named function expressions, and the
 first identifier of each comma-separated parameter. It does not read an arrow
 function's parameter list, an object-literal or class method's shorthand list,
 nor a destructured parameter's property names. A string literal is read as a
 literal, so a quoted `function (` is scanned as code, while a regular
 expression holding an unbalanced bracket inside a default value can end a
 parameter list early and leave the rest of that list unread.

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

//region Fixtures

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Folder name holding installed packages, which the walk never enters.
 */
const DEPENDENCY_DIRECTORY = 'node_modules';

/**
 Separator between the segments of a path relative to {@link SOURCE_DIRECTORY}.
 */
const PATH_SEPARATOR = '/';

/**
 Suffix of every file the walk keeps.
 */
const MODULE_FILE_SUFFIX = '.ts';

/**
 Suffix marking a file as a test, which the walk skips.
 */
const TEST_FILE_SUFFIX = '.test.ts';

/**
 Characters that continue an identifier, so a name is never matched inside a
 longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Keyword opening a function declaration or a named function expression.
 */
const FUNCTION_KEYWORD = 'function';

/**
 Characters that open a bracket pair whose contents never split a parameter list.
 */
const BRACKET_OPENERS = '([{';

/**
 Characters that close a bracket pair whose contents never split a parameter list.
 */
const BRACKET_CLOSERS = ')]}';

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
 Prefix marking a rest parameter.
 */
const SPREAD_PREFIX = '...';

/**
 Quote characters that open a string literal.
 */
const QUOTE_CHARACTERS = '\'"`';

/**
 Character escaping the next one inside a string literal.
 */
const ESCAPE_CHARACTER = '\\';

/**
 Character separating one line of module text from the next.
 */
const LINE_SEPARATOR = '\n';

/**
 Opening of a type parameter list.
 */
const TYPE_PARAMETER_OPENER = '<';

/**
 Closing of a type parameter list.
 */
const TYPE_PARAMETER_CLOSER = '>';

/**
 Character an arrow's closing greater-than sign follows, which closes no type
 parameter list.
 */
const ASSIGNMENT_CHARACTER = '=';

/**
 Opening of a parameter list.
 */
const PARAMETER_LIST_OPENER = '(';

/**
 Character separating one parameter from the next.
 */
const PARAMETER_SEPARATOR = ',';

/**
 Length of a parameter name that carries no meaning of its own.
 */
const MEANINGLESS_NAME_LENGTH = 1;

/**
 Product module beside this test, scanned as a positive control.
 */
const SURFACE_MODULE = 'comment-text.ts';

/**
 Product module one folder down, scanned to show the walk reaches `src/rule`.
 */
const RULE_MODULE = 'rule/tag-escaping.ts';

/**
 Module text whose second parameter names its element with one character, so the
 scan is shown to report an offender.
 */
const OFFENDING_FIXTURE = [
  'function joinLines(',
  '  lines,',
  '  n,',
  ') {',
  '  return lines.slice(0, n);',
  '}',
].join('\n',);

//endregion Fixtures

//region Types

/**
 One `function` parameter list the scan read.

 @example
 ```ts
 const list: ParameterList = { declaredAt: 97, names: ['line', 'commentText'] };
 ```
 */
type ParameterList = {
  /**
   Line the parameter list's `function` keyword sits on, 1-based.
   */
  readonly declaredAt: number;
  /**
   Parameter names as declared, without type annotations or defaults.
   */
  readonly names: readonly string[];
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

//region Helpers

/**
 Whether a character continues an identifier.

 @param character - one character of module text

 @returns true when the character can appear inside a name

 @example
 ```ts
 isIdentifierCharacter('_'); // true
 ```
 */
function isIdentifierCharacter(character: string,): boolean {
  return (character.length === 1) && IDENTIFIER_CHARACTERS.includes(character,);
}

/**
 Read the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none

 @example
 ```ts
 identifierPrefix('filePath: string,');
 ```
 */
function identifierPrefix(text: string,): string {
  for (let end = 0; end < text.length; end += 1) {
    if (!(isIdentifierCharacter(text[end] ?? '',)))
      return text.slice(0, end,);
  }
  return text;
}

/**
 Column after the run of whitespace starting at a column.

 @param text - text whose whitespace run is skipped

 @param from - column to start skipping at

 @returns column of the first character that is not whitespace

 @example
 ```ts
 skipWhitespace('  (line', 0); // 2
 ```
 */
function skipWhitespace(text: string, from: number,): number {
  for (let cursor = from; cursor < text.length; cursor += 1) {
    if ((text[cursor] ?? '').trim() !== '')
      return cursor;
  }
  return text.length;
}

/**
 Column of the string literal starting at a quote, after its closing quote.

 @param text - text holding the quoted literal

 @param from - column of the opening quote

 @returns column after the closing quote, or the text's length when unterminated

 @example
 ```ts
 stringLiteralEnd("\"a)\", ", 0); // 5
 ```
 */
function stringLiteralEnd(text: string, from: number,): number {
  /**
   Quote character that opened the literal.
   */
  const quote = text[from] ?? '';
  for (let cursor = from + 1; cursor < text.length; cursor += 1) {
    /**
     Character at the examined column.
     */
    const character = text[cursor] ?? '';
    if (character === ESCAPE_CHARACTER) {
      cursor += 1;
      continue;
    }
    if (character === quote)
      return cursor + 1;
  }
  return text.length;
}

/**
 Column of the bracket closing the one opened at a column, ignoring brackets
 inside string literals and type annotations.

 @param text - text holding the bracketed run

 @param from - column of the opening bracket

 @returns column of the closing bracket, or the text's length when unbalanced

 @example
 ```ts
 bracketEnd('(line,)', 0); // 5
 ```
 */
function bracketEnd(text: string, from: number,): number {
  for (let cursor = from, depth = 0; cursor < text.length;) {
    /**
     Character at the examined column.
     */
    const character = text[cursor] ?? '';
    if (QUOTE_CHARACTERS.includes(character,)) {
      cursor = stringLiteralEnd(text, cursor,);
      continue;
    }
    if (BRACKET_CLOSERS.includes(character,)) {
      depth -= 1;
      if (depth === 0)
        return cursor;
    }
    else if (BRACKET_OPENERS.includes(character,))
      depth += 1;
    cursor += 1;
  }
  return text.length;
}

/**
 Column after a function's type parameter list.

 @param text - text whose type parameter list is skipped

 @param from - column the list may open at

 @returns column of the first character after the list, or `from` when the text
 does not open a type parameter list there

 @example
 ```ts
 columnAfterTypeParameters('<Value,>(value', 0); // 9
 ```
 */
function columnAfterTypeParameters(text: string, from: number,): number {
  if ((text[from] ?? '') !== TYPE_PARAMETER_OPENER)
    return from;
  for (let cursor = from, depth = 0; cursor < text.length; cursor += 1) {
    /**
     Character at the examined column.
     */
    const character = text[cursor] ?? '';
    if (character === TYPE_PARAMETER_OPENER)
      depth += 1;
    else if (
      (character === TYPE_PARAMETER_CLOSER)
        && ((text[cursor - 1] ?? '') !== ASSIGNMENT_CHARACTER)
    ) {
      depth -= 1;
      if (depth === 0)
        return cursor + 1;
    }
  }
  return text.length;
}

/**
 Columns of the separators that split a parameter list into its top-level parts.

 @param text - text between a parameter list's brackets

 @returns separator columns, in source order

 @example
 ```ts
 topLevelSeparatorColumns('line: string, commentText: Text,'); // [11]
 ```
 */
function topLevelSeparatorColumns(text: string,): readonly number[] {
  /**
   Separator columns found so far.
   */
  const separators: number[] = [];
  for (let cursor = 0, depth = 0; cursor < text.length;) {
    /**
     Character at the examined column.
     */
    const character = text[cursor] ?? '';
    if (QUOTE_CHARACTERS.includes(character,)) {
      cursor = stringLiteralEnd(text, cursor,);
      continue;
    }
    if (character === TYPE_PARAMETER_OPENER) {
      cursor = columnAfterTypeParameters(text, cursor,);
      continue;
    }
    if (BRACKET_OPENERS.includes(character,))
      depth += 1;
    else if (BRACKET_CLOSERS.includes(character,))
      depth -= 1;
    else if ((character === PARAMETER_SEPARATOR) && (depth === 0))
      separators.push(cursor,);
    cursor += 1;
  }
  return separators;
}

/**
 Split a parameter list into its top-level parts.

 @param text - text between a parameter list's brackets

 @returns parameter texts as written, without surrounding whitespace

 @example
 ```ts
 parameterTexts('line: string, commentText: Text,'); // ['line: string', 'commentText: Text']
 ```
 */
function parameterTexts(text: string,): readonly string[] {
  /**
   Column each parameter starts at, in source order.
   */
  const boundaries = [0, ...topLevelSeparatorColumns(text,).map(function afterSeparator(column,) {
    return column + 1;
  },),];
  return boundaries
    .map(function toParameter(start, index,) {
      return text
        .slice(start, boundaries[index + 1] ?? text.length,)
        .trim();
    },)
    .filter(function notEmpty(parameter,) {
      return parameter !== '';
    },);
}

/**
 Name a parameter declares, with any rest prefix and type annotation removed.

 @param parameter - one parameter text as written

 @returns declared identifier, empty for a destructured parameter

 @example
 ```ts
 parameterName('...commentText: Text'); // 'commentText'
 ```
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
 Line a column sits on.

 @param text - text whose lines are counted

 @param at - column to locate

 @returns 1-based line number of the column

 @example
 ```ts
 lineAt({ text: 'a\nb', at: 2 }); // 2
 ```
 */
function lineAt({ text, at, }: { readonly text: string; readonly at: number; },): number {
  return text.slice(0, at,).split(LINE_SEPARATOR,).length;
}

/**
 Every `function` parameter list in a module text.

 @param text - module text with comments removed

 @returns one entry per parameter list, in source order

 @example
 ```ts
 declaredParameterLists({ text: 'function joinLines(lines, n) {}' });
 ```
 */
function declaredParameterLists({ text, }: { readonly text: string; },): readonly ParameterList[] {
  /**
   Parameter lists found so far.
   */
  const lists: ParameterList[] = [];
  for (let from = 0; from < text.length;) {
    /**
     Column the keyword starts at, or -1 when none is left.
     */
    const keyword = text.indexOf(FUNCTION_KEYWORD, from,);
    if (keyword === (-1))
      break;
    from = keyword + FUNCTION_KEYWORD.length;
    if (isIdentifierCharacter(text[keyword - 1] ?? '',))
      continue;
    if (isIdentifierCharacter(text[from] ?? '',))
      continue;
    /**
     Column after the keyword, its optional generator star, and its optional name.
     */
    let cursor = skipWhitespace(text, from,);
    if ((text[cursor] ?? '') === '*')
      cursor = skipWhitespace(text, cursor + 1,);
    cursor = skipWhitespace(text, cursor + identifierPrefix(text.slice(cursor,),).length,);
    cursor = skipWhitespace(text, columnAfterTypeParameters(text, cursor,),);
    if ((text[cursor] ?? '') !== PARAMETER_LIST_OPENER)
      continue;
    /**
     Column of the bracket closing the parameter list.
     */
    const close = bracketEnd(text, cursor,);
    lists.push({
      declaredAt: lineAt({ text, at: keyword, },),
      names: parameterTexts(text.slice(cursor + 1, close,),)
        .map(function toName(parameter,) {
          return parameterName(parameter,);
        },)
        .filter(function notEmpty(name,) {
          return name !== '';
        },),
    },);
    from = close + 1;
  }
  return lists;
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
  if (state.insideBlockComment)
    return {
      lines: [...state.lines, '',],
      insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER,)),
    };
  /**
   Column where a block comment opens on this line, or -1.
   */
  const blockOpener = line.indexOf(BLOCK_COMMENT_OPENER,);
  if (blockOpener === (-1))
    return {
      lines: [...state.lines, line.trimStart().startsWith(LINE_COMMENT_OPENER,) ? '' : line,],
      insideBlockComment: false,
    };
  return {
    lines: [...state.lines, line.slice(0, blockOpener,),],
    insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER, blockOpener,)),
  };
}

/**
 Read one product module and drop its comments, keeping every line so a reported
 line matches the file a reader opens.

 @param file - file name inside the source directory

 @returns module text with comment content removed

 @example
 ```ts
 moduleText('tsdoc-blocks.ts');
 ```
 */
async function moduleText(file: string,): Promise<string> {
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
    ).lines
    .join('\n',);
}

/**
 File names of this package's product modules: every TypeScript file under this
 test's folder, nested folders included, that is not itself a test.

 @param directory - folder to read, relative to {@link SOURCE_DIRECTORY}, empty
 for the top folder

 @returns product module file names, relative to {@link SOURCE_DIRECTORY}

 @example
 ```ts
 await productModules('');
 ```
 */
async function productModules(directory: string,): Promise<readonly string[]> {
  /**
   Entries of the read folder, with their kind so nested folders are walked.
   */
  const entries = await readdir(
    join(
      SOURCE_DIRECTORY,
      directory,
    ),
    { withFileTypes: true, },
  );
  /**
   Module files each entry contributes: a nested folder's own modules, or the
   entry's name when it is a product module file, gathered in parallel.
   */
  const perEntry = await Promise.all(entries.map(async function namesIn(
    entry: Dirent,
  ): Promise<readonly string[]> {
    /**
     Path of the entry relative to {@link SOURCE_DIRECTORY}.
     */
    const relativePath = (directory === '')
      ? entry.name
      : `${directory}${PATH_SEPARATOR}${entry.name}`;
    if (entry.isDirectory()) {
      return (entry.name === DEPENDENCY_DIRECTORY)
        ? []
        : await productModules(relativePath,);
    }
    if (entry.name.endsWith(MODULE_FILE_SUFFIX,) && (!(entry.name.endsWith(TEST_FILE_SUFFIX,))))
      return [relativePath,];
    return [];
  },),);
  return perEntry.flat();
}

/**
 One report line per parameter whose name carries no meaning of its own.

 @param file - file name the lists were read from

 @param lists - parameter lists read from that file

 @returns report lines, empty when every parameter is named with a word

 @example
 ```ts
 meaninglessParameterReports({ file: 'comment-text.ts', lists });
 ```
 */
function meaninglessParameterReports({
  file,
  lists,
}: { readonly file: string; readonly lists: readonly ParameterList[]; },): readonly string[] {
  return lists
    .flatMap(function toReports(list,) {
      return list.names
        .filter(function isMeaninglessName(name,) {
          return name.length === MEANINGLESS_NAME_LENGTH;
        },)
        .map(function toReport(name,) {
          return `${file}:${String(list.declaredAt,)} names a parameter ${name}`;
        },);
    },);
}

//endregion Helpers

await describe({
  name: 'tsdoc module shape',
  children: [
    //region Parameter names: a name that carries no meaning of its own hides the type it holds

    describe({
      name: 'parameter names',
      children: [
        it({
          name: 'reports a one-character parameter name in a fixture',
          fn: async () => {
            /**
             Parameter lists the scan reads from the offending fixture.
             */
            const lists = declaredParameterLists({ text: OFFENDING_FIXTURE, },);
            expect(lists,).toHaveLength(1,);
            expect(lists[0]?.names,).toEqual(['lines', 'n',],);
            expect(
              meaninglessParameterReports({ file: 'fixture.ts', lists, },),
            ).toEqual(['fixture.ts:1 names a parameter n',],);
          },
        },),
        it({
          name: 'reports no one-character parameter name in a product module',
          fn: async () => {
            /**
             Every product module with the parameter lists it declares.
             */
            const modules = await Promise.all(
              (await productModules('',)).toSorted().map(async function read(file,) {
                return { file, lists: declaredParameterLists({ text: await moduleText(file,), },), };
              },),
            );
            /**
             Modules the scan read, which must include the guarded ones: one
             beside this test and one inside `src/rule`.
             */
            const fileNames = modules.map(function toFile(module,) {
              return module.file;
            },);

            // A scan that never reached the modules the guard exists for proves nothing.
            expect(fileNames,).toContain(SURFACE_MODULE,);
            expect(fileNames,).toContain(RULE_MODULE,);

            /**
             One report line per meaningless parameter name across all modules.
             */
            const reports = modules.flatMap(function toReports(module,) {
              return meaninglessParameterReports(module,);
            },);

            expect(reports.join('\n',),).toBe('',);
          },
        },),
      ],
    },),

    //endregion Parameter names
  ],
},);
