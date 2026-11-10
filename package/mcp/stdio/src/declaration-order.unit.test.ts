/**
 Guard on this package's declaration order, which no lint rule enforces.

 `AGENTS.md` rule ST8 requires a function to be declared before it is called in
 source order, so a reader can follow the call graph top-down. No linter has that
 rule, so each product module's text is read here and checked directly. Comment
 content is dropped first, because a name quoted in a TSDoc `@example` is prose
 rather than a reference, and the scan reads characters instead of matching
 patterns.

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
 Characters that continue an identifier, so a name is never matched inside a longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Keyword opening a function declaration, generator or plain.
 */
const FUNCTION_KEYWORD = 'function';

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

//endregion Fixtures

//region Helpers

/**
 Read the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none

 @example
 ```ts
 identifierPrefix('serializeResponse({ response, id }: Params)');
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
  return (end === (-1)) ? text : text.slice(0, end,);
}

/**
 Name a line declares, when it opens a function declaration.

 @param line - one line of module text with comments removed

 @returns declared function name, empty for any other line

 @example
 ```ts
 declaredFunctionName('export async function* readLines(chunks) {');
 ```
 */
function declaredFunctionName(line: string,): string {
  /**
   Line without its leading indentation.
   */
  const indented = line.trimStart();
  /**
   Line without the `export` keyword.
   */
  const exported = indented.startsWith(EXPORT_KEYWORD,)
    ? indented.slice(EXPORT_KEYWORD.length,)
    : indented;
  /**
   Line without the `async` keyword.
   */
  const asynchronous = exported.startsWith(ASYNC_KEYWORD,)
    ? exported.slice(ASYNC_KEYWORD.length,)
    : exported;
  if (!(asynchronous.startsWith(FUNCTION_KEYWORD,)))
    return '';
  /**
   Signature between the `function` keyword and the declared name: `*` for a
   generator, and the separating space in either form.
   */
  const signature = asynchronous
    .slice(FUNCTION_KEYWORD.length,)
    .replace(/^\*/u, '',);
  if (!(signature.startsWith(' ',)))
    return '';
  return identifierPrefix(signature.trimStart(),);
}

/**
 Whether a line calls a function by name.

 @param line - one line of module text with comments removed

 @param name - function name to look for

 @returns true when the name appears as a whole identifier followed by `(`

 @example
 ```ts
 callsFunction('queueFrame({ queue, serialized });', 'queueFrame');
 ```
 */
function callsFunction(line: string, name: string,): boolean {
  for (let from = 0; from < line.length; from += 1) {
    if (!(line.startsWith(name, from,)))
      continue;
    /**
     Character immediately before the name, absent at the start of the line.
     */
    const before = line[from - 1];
    if ((before !== undefined) && (IDENTIFIER_CHARACTERS.includes(before,)))
      continue;
    if (line.slice(from + name.length,).trimStart().startsWith('(',))
      return true;
  }
  return false;
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
 Read one product module and drop its comments, keeping every line number so a
 reported line matches the file a reader opens.

 @param file - file name inside the source directory

 @returns module text split into lines, with comment content removed

 @example
 ```ts
 await moduleLines('transport.ts');
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
      return (name.endsWith('.ts',)) && (!(name.endsWith('.test.ts',)));
    },)
    .toSorted();
}

//endregion Helpers

/**
 Every product module with its comment-free lines, read once so the scan below
 proves something about the whole package.
 */
const modules = await Promise.all((await productModules()).map(
  async function read(file,) {
    return { file, lines: await moduleLines(file,), };
  },
),);

await describe({
  name: '',
  children: [
    describe({
      name: 'declaration order',
      children: [
        it({
          name: 'reads a function declaration out of this package',
          fn: () => {
            /**
             Declarations the scan recognises across every product module.
             */
            const declared = modules.flatMap(function names(module,) {
              return module.lines
                .map(function toName(line,) {
                  return declaredFunctionName(line,);
                },)
                .filter(function isDeclared(name,) {
                  return name !== '';
                },);
            },);

            // A scan that found no declaration anywhere proves nothing about the
            // modules that do declare functions.
            expect(declared.length,).toBeGreaterThan(0,);
          },
        },),
        ...modules.map(function mapModule(module,) {
        /**
         Module text with comment content removed.
         */
        const { file, lines, } = module;
        return it({
          name: `declares every ${file} function before calling it`,
          fn: () => {
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

            /**
             One report line per call that precedes its declaration, in
             declaration order.
             */
            const callsBeforeDeclaration = [...declarations,]
              .map(function toFirstCall([name, declaredAt,],) {
                /**
                 Line of the first call, 1-based, or 0 when never called.
                 */
                const firstCall = lines.findIndex(function mentions(line,) {
                  return callsFunction(line, name,);
                },) + 1;
                return { name, declaredAt, firstCall, };
              },)
              .filter(function isOutOfOrder(entry,) {
                // A zero line means the module never calls the function itself:
                // an export its consumers call is not a forward reference.
                return (entry.firstCall > 0) && (entry.firstCall < entry.declaredAt);
              },)
              .map(function toReport(entry,) {
                return [
                  `${file}:${String(entry.firstCall,)}`,
                  `calls ${entry.name},`,
                  `declared at ${String(entry.declaredAt,)}`,
                ].join(' ',);
              },);

            expect(callsBeforeDeclaration.join('\n',),).toBe('',);
          },
        },);
      },),
      ],
    },),
  ],
},);
