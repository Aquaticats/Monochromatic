/**
 Guards on this package's declaration order, which no lint rule enforces.

 `AGENTS.md` rule ST8 requires a function to be declared before it is called in
 source order, even though hoisting makes the reverse order run, and no linter
 flags the reverse order, so every product module's text is read here and checked
 directly. Comment content is dropped first, because a name quoted in a TSDoc
 `@example` is prose rather than a reference.

 The scan sees a call written as a bare `name` followed by an opening parenthesis,
 and reads only a `function` declaration written at column zero, which is how this
 package declares its callables: a callable bound by a `const`, and a named
 function expression passed as a callback, are outside its reach. The two controls
 below run the scan on sample module text, so a change that quietly stops it
 reporting a real violation fails here rather than letting `duplicateElement.ts`
 regress silently.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, relative, } from 'node:path';

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
 Characters that continue an identifier, so a name is never matched inside a longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Keyword opening a function declaration.
 */
const FUNCTION_KEYWORD = 'function ';

/**
 Keyword opening an exported declaration.
 */
const EXPORT_KEYWORD = 'export ';

/**
 Keyword opening an async declaration.
 */
const ASYNC_KEYWORD = 'async ';

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
 Suffix naming a test module, which declares no product callable and is not scanned.
 */
const TEST_FILE_SUFFIX = '.test.ts';

/**
 Suffix naming a type-only module, which holds no callable statement.
 */
const DECLARATION_FILE_SUFFIX = '.d.ts';

/**
 Suffix naming a module rather than a folder during discovery.
 */
const MODULE_FILE_SUFFIX = '.ts';

/**
 Folder holding installed packages, which is never product source.
 */
const INSTALLED_PACKAGES_DIRECTORY = 'node_modules';

/**
 Line number reported for a function this module never calls, which is no
 ordering violation because a declaration with no call in its own file is fine.
 */
const UNCALLED = 0;

/**
 Sample module text calling a helper before declaring it, which the scan must report.
 */
const OUT_OF_ORDER_SAMPLE = [
  'export function caller(): void {',
  '  helper();',
  '}',
  '',
  'function helper(): void {',
  '}',
].join('\n',);

/**
 Sample module text declaring every helper before the code that calls it.
 */
const IN_ORDER_SAMPLE = [
  'function helper(): void {',
  '}',
  '',
  'export function caller(): void {',
  '  helper();',
  '}',
].join('\n',);

/**
 Sample module text quoting an out-of-order call in comments and in a string,
 which runs no such call and must not be reported.
 */
const QUOTED_SAMPLE = [
  '// caller() runs before helper is declared',
  '/**',
  ' * @example',
  ' * ```ts',
  ' * caller();',
  ' * ```',
  ' */',
  'function helper(): void {',
  '}',
  '',
  'const note = \'caller()\';',
].join('\n',);

/**
 Sample module text whose only out-of-order name belongs to an indented named
 function expression, which is a callback rather than a module declaration.
 */
const CALLBACK_EXPRESSION_SAMPLE = [
  'const values = [1,].reduce(',
  '  function step(accumulator: number,) {',
  '    return step(accumulator,) + accumulator;',
  '  },',
  '  0,',
  ');',
].join('\n',);

//endregion Fixtures

//region Comment removal

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

 @param state - lines collected so far and whether a block comment is open

 @param line - next line of module text as written

 @returns the accumulator with this line appended
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
    lines: [...state.lines, line.slice(
      0,
      blockOpener,
    ),],
    insideBlockComment: !(line.includes(BLOCK_COMMENT_CLOSER, blockOpener,)),
  };
}

/**
 Module text with every comment removed, keeping one line per source line so a
 reported line number matches the file a reader opens.

 @param text - module text as written

 @returns module text split into lines, with comment content removed
 */
function commentedOut(text: string,): readonly string[] {
  return text
    .split('\n',)
    .reduce<StripState>(
    function step(state, line,) {
      return stripComments(state, line,);
    },
    { lines: [], insideBlockComment: false, },
    ).lines;
}

//endregion Comment removal

//region Scan

/**
 Read the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none
 */
function identifierPrefix(text: string,): string {
  /**
   Column of the first character that cannot continue an identifier, or -1 when
   every character can.
   */
  const end = text.split('',).findIndex(function endsIdentifier(character,) {
    return !(IDENTIFIER_CHARACTERS.includes(character,));
  },);
  return (end === (-1)) ? text : text.slice(
    0,
    end,
  );
}

/**
 Name a line declares, when the line opens a column-zero function declaration.

 @param line - one line of module text with comments removed

 @returns declared function name, empty for any other line
 */
function declaredFunctionName(line: string,): string {
  // An indented `function` keyword opens a callback expression, which is not a
  // module declaration and carries no ordering requirement.
  if (line.trimStart().length !== line.length)
    return '';
  /**
   Line without the `export` keyword.
   */
  const exported = line.startsWith(EXPORT_KEYWORD,)
    ? line.slice(EXPORT_KEYWORD.length,)
    : line;
  /**
   Line without the `async` keyword.
   */
  const asynchronous = exported.startsWith(ASYNC_KEYWORD,)
    ? exported.slice(ASYNC_KEYWORD.length,)
    : exported;
  if (!(asynchronous.startsWith(FUNCTION_KEYWORD,)))
    return '';
  return identifierPrefix(asynchronous.slice(FUNCTION_KEYWORD.length,),);
}

/**
 Whether a line calls a function by name.

 @param line - one line of module text with comments removed

 @param name - function name to look for

 @returns true when the name appears as a whole identifier followed by `(`
 */
function callsFunction(line: string, name: string,): boolean {
  for (const [at,] of line.split('',).entries()) {
    if (!(line.startsWith(name, at,)))
      continue;
    /**
     Character immediately before the name, absent at the start of the line.
     */
    const before = line[at - 1];
    if ((before !== undefined) && (IDENTIFIER_CHARACTERS.includes(before,)))
      continue;
    if (line.slice(at + name.length,).trimStart().startsWith('(',))
      return true;
  }
  return false;
}

/**
 Report every call in one module that precedes the declaration of the function it
 calls.

 @param text - one module's text as written

 @param file - file name to report, relative to the source directory

 @returns one report line per out-of-order call, empty when the module is in order
 */
function callsBeforeDeclarations({ text, file, }: {
  readonly text: string;
  readonly file: string;
},): readonly string[] {
  /**
   Module text with comment content removed.
   */
  const lines = commentedOut(text,);
  /**
   Line each function is declared on, by name.
   */
  const declarations = new Map<string, number>();
  for (const [index, line,] of lines.entries()) {
    /**
     Function name this line declares, empty for any other line.
     */
    const name = declaredFunctionName(line,);
    if (name !== '')
      declarations.set(name, index + 1,);
  }
  return [...declarations,]
    .map(function toFirstCall([name, declaredAt,],) {
      /**
       Line of the first call, 1-based, or {@link UNCALLED} when never called.
       */
      const firstCall = lines.findIndex(function mentions(line,) {
        return callsFunction(line, name,);
      },) + 1;
      return { name, declaredAt, firstCall, };
    },)
    .filter(function isOutOfOrder(entry,) {
      return (entry.firstCall !== UNCALLED) && (entry.firstCall < entry.declaredAt);
    },)
    .map(function toReport(entry,) {
      return [
        `${file}:${String(entry.firstCall,)}`,
        `calls ${entry.name},`,
        `declared at ${String(entry.declaredAt,)}`,
      ].join(' ',);
    },);
}

//endregion Scan

//region Module discovery

/**
 File names of this package's product modules, relative to the source directory,
 found by walking it and its sub-folders.

 @param directory - folder to walk

 @returns sorted relative paths of every non-test TypeScript file
 */
async function productModules(directory: string,): Promise<readonly string[]> {
  /**
   Directory entries of the folder being walked.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  /**
   Reports gathered from this folder's entries, one list per entry.
   */
  const perEntry = await Promise.all(entries.map(async function visit(entry,) {
    /**
     Path of the entry, relative to the source directory.
     */
    const path = relative(
      SOURCE_DIRECTORY,
      join(
        directory,
        entry.name,
      ),
    );
    if (entry.isDirectory())
      return entry.name === INSTALLED_PACKAGES_DIRECTORY ? [] : await productModules(join(
        directory,
        entry.name,
      ),);
    if (!(entry.name.endsWith(MODULE_FILE_SUFFIX,)))
      return [];
    if (entry.name.endsWith(TEST_FILE_SUFFIX,) || entry.name.endsWith(DECLARATION_FILE_SUFFIX,))
      return [];
    return [path,];
  },),);
  return perEntry.flat()
    .toSorted();
}

/**
 Every out-of-order call in this package, with one report line per call.

 @returns report lines of every out-of-order call, by module
 */
async function outOfOrderCalls(): Promise<readonly string[]> {
  /**
   One report list per module, gathered in parallel.
   */
  const perModule = await Promise.all((await productModules(SOURCE_DIRECTORY,)).map(
    async function scanModule(file,) {
      return callsBeforeDeclarations({
        text: await readFile(
          join(SOURCE_DIRECTORY, file,),
          'utf8',
        ),
        file,
      },);
    },
  ),);
  return perModule.flat();
}

//endregion Module discovery

/**
 Every out-of-order call in this package, scanned once for the guard below.
 */
const OUT_OF_ORDER_CALLS: readonly string[] = await outOfOrderCalls();

await describe({
  name: 'declaration order',
  children: [
    it({
      name: 'reports a call that precedes the declaration it reaches',
      fn: async () => {
        expect(callsBeforeDeclarations({
          text: OUT_OF_ORDER_SAMPLE,
          file: 'sample.ts',
        },),).toEqual(['sample.ts:2 calls helper, declared at 5',],);
      },
    },),
    it({
      name: 'reports nothing when a module declares every helper before its calls',
      fn: async () => {
        expect(callsBeforeDeclarations({
          text: IN_ORDER_SAMPLE,
          file: 'sample.ts',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'reports nothing for a call quoted in a comment or a string',
      fn: async () => {
        expect(callsBeforeDeclarations({
          text: QUOTED_SAMPLE,
          file: 'sample.ts',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'reports nothing for an indented callback expression',
      fn: async () => {
        expect(callsBeforeDeclarations({
          text: CALLBACK_EXPRESSION_SAMPLE,
          file: 'sample.ts',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'calls no function before the module declares it',
      fn: async () => {
        expect(OUT_OF_ORDER_CALLS,).toEqual([],);
      },
    },),
  ],
},);
