/**
 Guards LG2 in this package: a catch block that turns the caught failure into
 a sentinel logs the caught value first, so a resolved candidate that was
 skipped leaves a reason in the log.

 Two checks: a source scan covering every catch block, and one run of
 `parseConfigFiles` showing the record that reaches the log file.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { logger, } from '@monochromatic-dev/logger-core/ts';

import { parseConfigFiles, } from '../dist/final/node/launch.mjs';

/**
 Directory whose TypeScript sources are scanned.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Suffix of the module test files, whose fixtures are not production code.
 */
const TEST_FILE_SUFFIX = '.unit.test.ts';

/**
 Suffix of one log file written by the logger's file sink.
 */
const LOG_FILE_SUFFIX = '.log.jsonl';

/**
 One catch block that swallows its caught value without logging it.
 */
type SilentCatch = {
  /**
   One-based line number of the `catch` keyword.
   */
  readonly line: number;
  /**
   File name inside {@link SOURCE_DIRECTORY}.
   */
  readonly path: string;
};

/**
 One record read back from the logger's file sink.
 */
type LogRecordShape = {
  readonly level: string;
  readonly message: string;
};

/**
 Counts line breaks before one offset, giving the one-based line of a match.

 @param text - Text the offset lives in.

 @param offset - Zero-based character offset.

 @returns One-based line number at that offset.
 */
function lineAt({
  text,
  offset,
}: {
  readonly text: string;
  readonly offset: number;
},): number {
  return text.slice(0, offset,).split('\n',).length;
}

/**
 Finds catch blocks that swallow a caught error without logging it.

 A block that logs with `l.debug(` counts as handled. A block that only
 rethrows (no `return` and no `continue`) also counts, because the caller then
 sees the error. Everything else is reported.

 @param text - Source text to scan.

 @param path - File name reported for each finding.

 @returns Every silent catch block, in line order.
 */
function silentCatchesIn({
  text,
  path,
}: {
  readonly text: string;
  readonly path: string;
},): readonly SilentCatch[] {
  /**
   Findings so far, in line order.
   */
  const silent: SilentCatch[] = [];
  /**
   Keyword search cursor; each hit resumes after its block's closing brace.
   */
  let cursor = 0;

  while (cursor < text.length) {
    /**
     Position of the next catch keyword.
     */
    const catchAt = text.indexOf('catch (', cursor,);
    if (catchAt === (-1))
      break;

    /**
     Position of the block's opening brace, which follows the binding.
     */
    const openAt = text.indexOf('{', catchAt,);
    if (openAt === (-1))
      break;

    /**
     Brace depth, starting inside the opening brace.
     */
    let depth = 1;
    /**
     Scan cursor for the matching closing brace.
     */
    let scan = openAt + 1;
    while ((scan < text.length) && (depth > 0)) {
      if (text[scan] === '{')
        depth++;
      else if (text[scan] === '}')
        depth--;
      scan++;
    }

    /**
     Block body between its braces.
     */
    const body = text.slice(
      openAt + 1,
      scan - 1,
    );
    /**
     Whether the body hides the caught value from the operator.
     */
    const swallows = body.includes('return ',)
      || body.includes('continue;',);
    if (swallows && !body.includes('l.debug(',))
      silent.push({
        line: lineAt({
          text,
          offset: catchAt,
        },),
        path,
      },);

    cursor = scan;
  }

  return silent;
}

/**
 Reads the debug messages the logger's file sink recorded in this run.

 The harness redirects the file sink into per-run scratch through
 `MONOCHROMATIC_LOG_DIRECTORY`, so a test reads back exactly what its own
 process logged.

 @returns Message text of every debug record flushed so far.
 */
async function debugMessages(): Promise<readonly string[]> {
  /**
   Destination the harness chose for this run.
   */
  const directory = process.env['MONOCHROMATIC_LOG_DIRECTORY'];
  if (directory === undefined)
    throw new Error('MONOCHROMATIC_LOG_DIRECTORY is unset: the test harness must redirect the logger file sink',);

  /**
   Day directories the file sink created under that destination.
   */
  const dayDirectories = (await readdir(
    directory,
    { withFileTypes: true, },
  )).filter(function isDirectory(entry,) {
    return entry.isDirectory();
  },);
  /**
   Absolute path of every log file in those day directories.
   */
  const files = (await Promise.all(dayDirectories.map(async function logFilesIn(entry,) {
    /**
     Day directory being listed.
     */
    const dayPath = join(
      directory,
      entry.name,
    );
    return (await readdir(dayPath,)).filter(function isLogFile(name,) {
      return name.endsWith(LOG_FILE_SUFFIX,);
    },).map(function toPath(name,) {
      return join(
        dayPath,
        name,
      );
    },);
  },),)).flat();

  /**
   Text of every log file, read concurrently.
   */
  const texts = await Promise.all(files.map(async function readLog(file,) {
    return await readFile(
      file,
      'utf8',
    );
  },),);

  return texts.flatMap(function toLines(text,) {
    return text.split('\n',);
  },).filter(function isRecord(line,) {
    return line.length > 0;
  },).map(function toRecord(line,) {
    return JSON.parse(line,) as LogRecordShape;
  },).filter(function isDebug(record,) {
    return record.level === 'debug';
  },).map(function toMessage(record,) {
    return record.message;
  },);
}

await describe({
  name: 'caught error logging',
  children: [
    it({
      name: 'logs every caught error a production catch block swallows',
      fn: async () => {
        /**
         Every TypeScript file the package ships, excluding its tests.
         */
        const files = (await readdir(SOURCE_DIRECTORY,)).filter(function isProduction(file,) {
          return file.endsWith('.ts',) && !file.endsWith(TEST_FILE_SUFFIX,);
        },);
        /**
         File name paired with its source text, read concurrently.
         */
        const sources = await Promise.all(files.map(async function readSource(file,) {
          return {
            file,
            text: await readFile(
              join(
                SOURCE_DIRECTORY,
                file,
              ),
              'utf8',
            ),
          };
        },),);

        expect(sources.flatMap(function findSilent({
          file,
          text,
        },) {
          return silentCatchesIn({
            text,
            path: file,
          },);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'logs the reason when a config file cannot be read',
      fn: async () => {
        /**
         Fresh scratch directory holding the unreadable candidate.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'terminal-exec-unreadable-config-',),);
        /**
         Directory path passed where a config file is expected, so the read
         fails with EISDIR.
         */
        const unreadable = join(
          directory,
          'config-as-directory',
        );
        await mkdir(unreadable,);

        /**
         Parsed config; the unreadable path contributes nothing.
         */
        const config = await parseConfigFiles({ paths: [unreadable,], },);
        expect(config.entryIds,).toEqual([],);

        await logger.flush();

        /**
         Debug messages naming the unreadable path.
         */
        const matching = (await debugMessages()).filter(function namesPath(message,) {
          return message.includes(unreadable,);
        },);
        expect(matching,).toHaveLength(1,);
        expect(matching[0],).toContain('EISDIR',);
      },
    },),
  ],
},);
