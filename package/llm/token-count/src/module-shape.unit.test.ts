/**
 Guards this package's module shape where no lint rule reaches.

 `AGENTS.md` and `doc/todo/code-quality.md` ban a single-letter variable outside a
 math formula, and `id-length` is set to `allow` in
 `package/config/oxlint/src/rule/style.ts`, so nothing enforces the ban here.
 Every product module's comment-free text is read and each `function` parameter
 list is checked directly.

 The scan covers `function` declarations and function expressions. An arrow
 function's parameter list is not read, and a string literal is read as written,
 so a quoted `function (` would be scanned as code.

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
 Product module holding the CLI's aggregate line, scanned as a positive control.
 */
const CLI_MODULE = 'cli.ts';

/**
 Module text whose second parameter names its element with one character, so the
 scan is shown to report an offender.
 */
const OFFENDING_FIXTURE = [
  'function sumTokens(',
  '  sum,',
  '  r,',
  ') {',
  '  return sum + r;',
  '}',
].join('\n',);

//endregion Fixtures

//region Types

/**
 One `function` parameter list the scan read.

 @example
 ```ts
 const list: ParameterList = { declaredAt: 97, names: ['sum', 'fileResult'] };
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
 skipWhitespace('  (sum', 0); // 2
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
 bracketEnd('(sum,)', 0); // 5
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
 topLevelSeparatorColumns('sum: number, fileResult: Result,'); // [11]
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
 parameterTexts('sum: number, fileResult: Result,'); // ['sum: number', 'fileResult: Result']
 ```
 */
function parameterTexts(text: string,): readonly string[] {
  /**
   Columns of the separators between parameters.
   */
  const separators = topLevelSeparatorColumns(text,);
  /**
   Column each parameter starts at.
   */
  const starts = [0, ...separators.map(function afterSeparator(column,) {
    return column + 1;
  },),];
  /**
   Column just past each parameter.
   */
  const ends = [...separators, text.length,];
  return starts
    .map(function toParameter(start, index,) {
      return text.slice(start, ends[index] ?? text.length,).trim();
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
 parameterName('...fileResult: Result'); // 'fileResult'
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
 declaredParameterLists({ text: 'function sumTokens(sum, r) {}' });
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
 await moduleText('cli.ts');
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
 File names of this package's product modules: every TypeScript file beside this
 test that is not itself a test.

 @returns sorted file names scanned by the guard below

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
 One report line per parameter whose name carries no meaning of its own.

 @param file - file name the lists were read from

 @param lists - parameter lists read from that file

 @returns report lines, empty when every parameter is named with a word

 @example
 ```ts
 meaninglessParameterReports({ file: 'cli.ts', lists });
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
  name: 'token-count module shape',
  children: [
    //region Parameter names: a name that carries no meaning of its own hides the type it holds

    describe({
      name: 'parameter names',
      children: [
        it({
          name: 'reports a one-character parameter name the scan reads',
          fn: async () => {
            /**
             Parameter lists the scan reads from the offending fixture.
             */
            const lists = declaredParameterLists({ text: OFFENDING_FIXTURE, },);
            expect(lists,).toHaveLength(1,);
            expect(lists[0]?.names,).toEqual(['sum', 'r',],);
            expect(
              meaninglessParameterReports({ file: 'fixture.ts', lists, },),
            ).toEqual(['fixture.ts:1 names a parameter r',],);
          },
        },),
        it({
          name: 'names every product module parameter with a word',
          fn: async () => {
            /**
             Every product module with the parameter lists it declares.
             */
            const modules = await Promise.all(
              (await productModules()).map(async function read(file,) {
                return { file, lists: declaredParameterLists({ text: await moduleText(file,), },), };
              },),
            );
            /**
             Names of the modules the scan read, which must include the guarded one.
             */
            const fileNames = modules.map(function toFile(module,) {
              return module.file;
            },);

            // A scan that never reached the module the guard exists for proves nothing.
            expect(fileNames,).toContain(CLI_MODULE,);

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
