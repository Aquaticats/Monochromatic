/**
 Guards on paper2vn's build script logging, which no linter enforces.

 `AGENTS.md` rule TLG requires production code to log only through tagged
 loggers and rule LG1 forbids a hand-written tag prefix in a message string.
 Issue 1634 recorded `src/build.ts` writing `'[paper2vn] building...'` and
 `'[paper2vn] wrote ...'` through raw `console.error`, which is the same idiom
 the package's other modules copy.

 The scan reads module text with comments dropped first, because a name quoted
 in a TSDoc block is prose rather than a call.

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
 Package source folder holding every module this test reads.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Suffix every unit test file carries, so the scan skips test text.
 */
const TEST_FILE_SUFFIX = '.unit.test.ts';

/**
 Suffix of a module file.
 */
const MODULE_FILE_SUFFIX = '.ts';

/**
 Folder holding the browser bundle, which the build-script guard does not cover.
 */
const CLIENT_FOLDER_PREFIX = 'client/';

/**
 Build script whose logging this guard pins.
 */
const BUILD_SCRIPT = 'build.ts';

/**
 Bracket prefix the issue recorded being written into message strings by hand.
 */
const HAND_WRITTEN_PREFIX = '[paper2vn]';

/**
 Namespace of the raw console a Node-side module must not log through.
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
 Construction call of a tagged logger.
 */
const TAGGED_CONSTRUCTION = 'tagged({';

/**
 Root logger module the build script tags below.
 */
const ROOT_LOGGER_MODULE = './log.ts';

/**
 One module file: its path relative to {@link SOURCE_DIRECTORY} and its text.
 */
type Module = {
  /**
   Path relative to the source folder, used in failure messages.
   */
  readonly path: string;
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
 stripComments('*\/ console.error *\/ l.info("building");');
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
 Lists the module files under one folder, recursively, tests excluded.

 @param relativePath - folder path relative to {@link SOURCE_DIRECTORY}

 @returns module paths relative to {@link SOURCE_DIRECTORY}

 @example
 ```ts
 await listModules('');
 ```
 */
async function listModules(relativePath: string,): Promise<string[]> {
  /**
   Directory entries of this folder.
   */
  const entries = await readdir(
    join(
      SOURCE_DIRECTORY,
      relativePath,
    ),
    { withFileTypes: true, },
  );
  /**
   Modules of every entry, one list per entry, walked concurrently.
   */
  const perEntry = await Promise.all(entries.map(async function modulesOf(entry,) {
    /**
     Path of this entry relative to {@link SOURCE_DIRECTORY}.
     */
    const entryPath = (relativePath === '') ? entry.name : `${relativePath}/${entry.name}`;
    if (entry.isDirectory())
      return listModules(entryPath,);
    if ((!entry.name.endsWith(MODULE_FILE_SUFFIX,)) || entry.name.endsWith(TEST_FILE_SUFFIX,))
      return [];
    return [entryPath,];
  },),);
  return perEntry.flat();
}

/**
 Reads one module, dropping its comments.

 @param path - module path relative to {@link SOURCE_DIRECTORY}

 @returns the module with its comment-free text

 @example
 ```ts
 await readModule('build.ts');
 ```
 */
async function readModule(path: string,): Promise<Module> {
  /**
   Module text as written.
   */
  const text = await readFile(
    join(
      SOURCE_DIRECTORY,
      path,
    ),
    'utf8',
  );
  return {
    path,
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

 @returns one `path: line` entry per matching line, empty when none match

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
        return `${module.path}: ${line.trim()}`;
      },);
  },);
}

//endregion Helpers

//region Tests

/**
 Every module this guard reads: the Node scripts and the browser bundle alike.
 */
const modules = await Promise.all(
  (await listModules('',))
    .toSorted()
    .map(async function read(path,) {
      return readModule(path,);
    },),
);

/**
 Build script module, the subject of the logging checks below.
 */
const buildScript = modules.find(function named(module,) {
  return module.path === BUILD_SCRIPT;
},);

await describe({
  name: '',
  children: [
    describe({
      name: 'build script logging',
      children: [
        it({
          name: 'reads the build script and the browser bundle',
          fn: async () => {
            expect(buildScript === undefined,)
              .toBe(false,);
            expect(modules.some(function inClient(module,) {
              return module.path.startsWith(CLIENT_FOLDER_PREFIX,);
            },),)
              .toBe(true,);
          },
        },),
        it({
          name: 'writes no hand-written package tag into a message string',
          fn: async () => {
            expect(matchingLines(modules, function namesPrefix(line,) {
              return line.includes(HAND_WRITTEN_PREFIX,);
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'logs the build script through a tagged logger',
          fn: async () => {
            expect(matchingLines(
              (buildScript === undefined) ? [] : [buildScript,],
              function writesRawConsole(line,) {
                return writesThroughRawConsole(line,);
              },
            ),)
              .toEqual([],);
            expect(buildScript?.text.includes(TAGGED_CONSTRUCTION,) ?? false,)
              .toBe(true,);
            expect(buildScript?.text.includes(ROOT_LOGGER_MODULE,) ?? false,)
              .toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Tests
