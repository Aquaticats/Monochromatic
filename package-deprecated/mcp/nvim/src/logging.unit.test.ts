/**
 Guards on how this package logs, which no linter enforces.

 `AGENTS.md` rule TLG requires production code to log only through tagged
 loggers from `\@monochromatic-dev/logger-core` and rule LG1 forbids a
 hand-written tag prefix inside a message string. Issue 1634 recorded the server
 logging through raw `console.error` with a literal `[mcp-nvim]` prefix at every
 site, which put a second spelling of the logging policy in the package and made
 tag filtering impossible.

 The scan reads this package's own module text, with comments dropped first
 because a name quoted in a TSDoc block is prose rather than a call.

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
 Folder holding this package's modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Suffix every unit test file carries, so the scan skips test text.
 */
const TEST_FILE_SUFFIX = '.unit.test.ts';

/**
 Suffix of a module file the shipped server is built from.
 */
const MODULE_FILE_SUFFIX = '.ts';

/**
 Module that builds the server's root logger.
 */
const ROOT_LOGGER_MODULE = 'log.ts';

/**
 Namespace of the raw console this package must not log through.
 */
const CONSOLE_NAMESPACE = 'console';

/**
 Methods of the console namespace that write a log line.
 */
const CONSOLE_METHODS: ReadonlySet<string> = new Set([
  'debug',
  'error',
  'info',
  'log',
  'trace',
  'warn',
]);

/**
 Characters that continue an identifier, so a name is never matched inside a longer one.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Characters that separate tokens on a line.
 */
const WHITESPACE = ' \t';

/**
 Bracket prefix the issue recorded being written into message strings by hand.
 */
const HAND_WRITTEN_PREFIX = '[mcp-nvim]';

/**
 Construction call of a tagged logger.
 */
const TAGGED_CONSTRUCTION = 'tagged({';

/**
 Name the root logger is imported under, so a module logger chains to it.
 */
const ROOT_LOGGER_IMPORT = 'rootLogger';

/**
 Sink factory the server's logger is built over, the console alone.
 */
const CONSOLE_SINK_FACTORY = 'createConsoleSink(';

/**
 Package root of the logger, whose singleton also carries a file sink that writes
 into whichever project directory the embedding host runs from.
 */
const LOGGER_PACKAGE_ROOT_IMPORTS: readonly string[] = [
  "from '@monochromatic-dev/logger-core'",
  "from '@monochromatic-dev/logger-core/ts'",
];

/**
 One module file: its name and its text with comments removed.
 */
type Module = {
  /**
   File name inside {@link SOURCE_DIRECTORY}.
   */
  readonly name: string;
  /**
   Comment-free text, so a name inside prose is never read as a call.
   */
  readonly text: string;
};

//endregion Fixtures

//region Helpers

/**
 Removes comment content so prose about logging is never read as a call.

 @param text - module text as written

 @returns the same text with every comment replaced by nothing

 @example
 ```ts
 stripComments('*\/ console.error *\/ l.error("failed");');
 ```
 */
function stripComments(text: string,): string {
  /**
   Text accumulated outside comments.
   */
  let plain = '';
  /**
   Whether a block comment is open.
   */
  let insideComment = false;
  for (let index = 0; index < text.length; index += 1) {
    if ((!insideComment) && text.startsWith('/*', index,)) {
      insideComment = true;
      index += 1;
      continue;
    }
    if (insideComment) {
      if (text.startsWith('*/', index,)) {
        insideComment = false;
        index += 1;
      }
      continue;
    }
    plain += text[index];
  }
  return plain;
}

/**
 Reads one module, dropping its comments.

 @param name - file name inside {@link SOURCE_DIRECTORY}

 @returns the module with its comment-free text

 @example
 ```ts
 await readModule('log.ts');
 ```
 */
async function readModule(name: string,): Promise<Module> {
  /**
   Module text as written.
   */
  const text = await readFile(
    join(
      SOURCE_DIRECTORY,
      name,
    ),
    'utf8',
  );
  return {
    name,
    text: stripComments(text,),
  };
}

/**
 Reads the identifier at the start of a text.

 @param text - text whose leading identifier characters are read

 @returns leading identifier characters, empty when the text opens with none

 @example
 ```ts
 identifierPrefix('error(`failed`)');
 ```
 */
function identifierPrefix(text: string,): string {
  /**
   Column of the first character that cannot continue an identifier, or -1 when
   every character can.
   */
  const end = text.split('',)
    .findIndex(function endsIdentifier(character,) {
      return !IDENTIFIER_CHARACTERS.includes(character,);
    },);
  return (end === (-1)) ? text : text.slice(0, end,);
}

/**
 Whether a line writes through a raw console method.

 @param line - one line of module text with comments removed

 @returns true when `console` is followed by `.` and a log-writing method name

 @example
 ```ts
 writesThroughRawConsole(`console.error('failed');`);
 ```
 */
function writesThroughRawConsole(line: string,): boolean {
  for (let at = line.indexOf(CONSOLE_NAMESPACE,); at !== (-1); at = line.indexOf(CONSOLE_NAMESPACE, at + CONSOLE_NAMESPACE.length,)) {
    /**
     Character before the namespace, absent at the start of the line.
     */
    const before = line[at - 1];
    if ((before !== undefined) && IDENTIFIER_CHARACTERS.includes(before,))
      continue;
    /**
     Column just after the namespace, before any spacing.
     */
    const namespaceEnd = at + CONSOLE_NAMESPACE.length;
    /**
     Column of the dot that opens a member access, spacing skipped.
     */
    const dot = afterSpacing(line, namespaceEnd,);
    if (line[dot] !== '.')
      continue;
    /**
     Column of the method name, spacing skipped.
     */
    const method = afterSpacing(line, dot + 1,);
    /**
     Method name as written after the dot.
     */
    const methodName = identifierPrefix(line.slice(method,),);
    if (CONSOLE_METHODS.has(methodName,))
      return true;
  }
  return false;
}

/**
 Column after any spacing that starts at a column.

 @param text - line being scanned

 @param from - column the scan starts at

 @returns column of the first character that is not spacing

 @example
 ```ts
 afterSpacing('console .error(', 7);
 ```
 */
function afterSpacing(text: string, from: number,): number {
  /**
   Column of the first character that is not spacing, or -1 when the rest is spacing.
   */
  const spacing = text.slice(from,)
    .split('',)
    .findIndex(function endsSpacing(character,) {
      return !WHITESPACE.includes(character,);
    },);
  return (spacing === (-1)) ? text.length : from + spacing;
}

/**
 Names the modules whose lines a predicate reports, with the matching lines.

 @param modules - modules to search

 @param matches - predicate a violating line satisfies

 @returns one `file: line` entry per matching line, empty when none match

 @example
 ```ts
 matchingLines(modules, writesThroughRawConsole);
 ```
 */
function matchingLines(
  modules: readonly Module[],
  matches: (line: string) => boolean,
): readonly string[] {
  return modules.flatMap(function linesOf(module,) {
    return module.text.split('\n',)
      .filter(function reported(line,) {
        return matches(line,);
      },)
      .map(function describeLine(line,) {
        return `${module.name}: ${line.trim()}`;
      },);
  },);
}

//endregion Helpers

//region Tests

/**
 Every module file the shipped server is built from, tests excluded.
 */
const modules = await Promise.all(
  (await readdir(SOURCE_DIRECTORY,))
    .filter(function isModule(name,) {
      return name.endsWith(MODULE_FILE_SUFFIX,)
        && (!name.endsWith(TEST_FILE_SUFFIX,));
    },)
    .toSorted()
    .map(async function read(name,) {
      return readModule(name,);
    },),
);

/**
 Modules other than the root logger's that build a tagged logger.
 */
const moduleLoggerBuilders = modules.filter(function buildsLogger(module,) {
  return (module.name !== ROOT_LOGGER_MODULE) && module.text.includes(TAGGED_CONSTRUCTION,);
},);

await describe({
  name: '',
  children: [
    describe({
      name: 'server logging',
      children: [
        it({
          name: 'reads the root logger module and the modules that log through it',
          fn: async () => {
            expect(modules.map(function nameOf(module,) {
              return module.name;
            },),)
              .toContain(ROOT_LOGGER_MODULE,);
            expect(moduleLoggerBuilders.length,)
              .toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'writes no log through a raw console call',
          fn: async () => {
            expect(matchingLines(modules, function writesRawConsole(line,) {
              return writesThroughRawConsole(line,);
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'writes no hand-written tag prefix into a message string',
          fn: async () => {
            expect(matchingLines(modules, function namesPrefix(line,) {
              return line.includes(HAND_WRITTEN_PREFIX,);
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'chains every module logger to the server root logger',
          fn: async () => {
            /**
             Modules building a tagged logger without importing the root logger.
             */
            const unchained = moduleLoggerBuilders
              .filter(function withoutRoot(module,) {
                return !module.text.includes(ROOT_LOGGER_IMPORT,);
              },)
              .map(function nameOf(module,) {
                return module.name;
              },);
            expect(unchained,)
              .toEqual([],);
          },
        },),
        it({
          name: 'builds the root logger over the console sink alone',
          fn: async () => {
            /**
             Root logger module text; the subject of both checks below.
             */
            const rootLoggerModule = modules.find(function named(module,) {
              return module.name === ROOT_LOGGER_MODULE;
            },);
            expect(rootLoggerModule === undefined,)
              .toBe(false,);
            expect(rootLoggerModule?.text.includes(CONSOLE_SINK_FACTORY,) ?? false,)
              .toBe(true,);
            expect(LOGGER_PACKAGE_ROOT_IMPORTS.filter(function imported(entry,) {
              return rootLoggerModule?.text.includes(entry,) ?? false;
            },),)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Tests
