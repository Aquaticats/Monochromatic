/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule ST8 requires a function to be declared before it is called in
 source order, even though hoisting makes the reverse order run, and no linter
 flags the reverse order, so every product module's text is read here and checked
 directly. Comment content is dropped first, because a name quoted in a TSDoc
 `@example` is prose rather than a reference, and the scan reads characters
 instead of matching patterns.

 The modules below still place a helper after the code that calls it. They are
 recorded rather than repaired together so the pattern cannot spread: a module
 outside the record may not carry it, and a recorded module must still carry it,
 so reordering one forces its removal from the record and the record only
 shrinks. `#1183` tracks the reordering of the recorded modules.

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
 Line number reported for a function this module never calls, which is no
 ordering violation because a declaration with no call in its own file is fine.
 */
const UNCALLED = 0;

/**
 Product modules that still call a function declared below the call.
 */
const MODULES_AWAITING_DECLARATION_ORDER: readonly string[] = [
  'client-http.ts',
  'client.ts',
  'config-file.ts',
  'config-schema.ts',
  'config.ts',
  'deepseek-client.ts',
  'deepseek-date.ts',
  'deepseek-filter.ts',
  'deepseek-http.ts',
  'deepseek-query.ts',
  'exa-client.ts',
  'exa-http.ts',
  'gh-client.ts',
  'gh-invocation-plan.ts',
  'gh-process.ts',
  'github-content-plan.ts',
  'github-section-plan.ts',
  'github-url-plan.ts',
  'github-url-validation.ts',
  'markdown-data-image-filter.ts',
  'markdown-data-image-parser.ts',
  'migrate-bashrc.ts',
  'migrate-files.ts',
  'migrate.ts',
  'provider-fallback.ts',
  'response-retention.ts',
  'search-fetch-fetch.ts',
  'tool-exec.ts',
  'tool-output-format.ts',
  'tool-output.ts',
  'valibot-to-typebox.ts',
  'whole-call-deadline.ts',
];

//endregion Fixtures

//region Helpers

/**
 Read the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none

 @example
 ```ts
 identifierPrefix('isDirectlyExecuted(): Promise<boolean>');
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
  return (end === (-1)) ? text : text.slice(
    0,
    end,
  );
}

/**
 Name a line declares, when it opens a function declaration.

 @param line - one line of module text with comments removed

 @returns declared function name, empty for any other line

 @example
 ```ts
 declaredFunctionName('export async function serve(options) {');
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
  return identifierPrefix(asynchronous.slice(FUNCTION_KEYWORD.length,),);
}

/**
 Whether a line calls a function by name.

 @param line - one line of module text with comments removed

 @param name - function name to look for

 @returns true when the name appears as a whole identifier followed by `(`

 @example
 ```ts
 callsFunction('const launch = await isDirectlyExecuted();', 'isDirectlyExecuted');
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
    lines: [...state.lines, line.slice(
      0,
      blockOpener,
    ),],
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

 @returns sorted file names scanned by the guards below

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
 Calls inside one module that precede the declaration of the function they call.

 @param file - file name inside the source directory

 @returns one report per out-of-order call, empty when the module is in order

 @example
 ```ts
 await callsBeforeDeclarations('domain-policy.ts');
 ```
 */
async function callsBeforeDeclarations(file: string,): Promise<readonly string[]> {
  /**
   Module text with comment content removed.
   */
  const lines = await moduleLines(file,);
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

/**
 Modules that call a function before declaring it, with one report line per call.

 @returns report lines of every out-of-order call, by module

 @example
 ```ts
 await outOfOrderCalls();
 ```
 */
async function outOfOrderCalls(): Promise<readonly string[]> {
  /**
   One report list per module, gathered in parallel.
   */
  const perModule = await Promise.all((await productModules()).map(function scanModule(file,) {
    return callsBeforeDeclarations(file,);
  },),);
  return perModule.flat();
}

/**
 File name a report line belongs to.

 @param report - one report line, which opens with the file name and a colon

 @returns the file name before the first colon

 @example
 ```ts
 reportFileName('index.ts:12 calls serve, declared at 40');
 ```
 */
function reportFileName(report: string,): string {
  return report.slice(
    0,
    report.indexOf(':',),
  );
}

//endregion Helpers

/**
 Every out-of-order call in this package, scanned once for both guards.
 */
const OUT_OF_ORDER_CALLS: readonly string[] = await outOfOrderCalls();

await describe({
  name: '',
  children: [
    describe({
      name: 'declaration order',
      children: [
        it({
          name: 'calls no function before the module declares it',
          fn: async () => {
            /**
             Report lines whose module is not recorded as awaiting the reorder.
             */
            const unrecorded = OUT_OF_ORDER_CALLS.filter(function isUnrecorded(report,) {
              /**
               File name a report line belongs to.
               */
              const file = reportFileName(report,);
              return !(MODULES_AWAITING_DECLARATION_ORDER.includes(file,));
            },);

            expect(unrecorded,).toEqual([],);
          },
        },),
        it({
          name: 'records only modules that still call a function before declaring it',
          fn: async () => {
            /**
             Files that carry an out-of-order call.
             */
            const violating = new Set(
              OUT_OF_ORDER_CALLS.map(function toFile(report,) {
                return reportFileName(report,);
              },),
            );
            /**
             Recorded modules the scan found in order, which the record must drop.
             */
            const repaired = MODULES_AWAITING_DECLARATION_ORDER.filter(function isRepaired(file,) {
              return !(violating.has(file,));
            },);

            expect(repaired,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
