/**
 Tests that `warnLinesDuring` keeps the work's warnings apart from the
 logger's own report of a sink it could not verify.

 IN A CHILD PROCESS, because the logger is built once a process, on its first
 line: only a process whose first line is logged inside the divert shows what
 a case in the suite meets when its file is the first to log. The child is
 told to log into a directory where a regular file stands
 (`MONOCHROMATIC_LOG_DIRECTORY`, which the logger keeps to and never trades
 for a fallback), so that sink's verify fails at once and the logger reports
 it on `console.warn` itself, untagged. A file sink slow to verify under load
 reports the same way five seconds later (ledger B140); the regular file
 stands in for the load without waiting on it.

 TWO LOGGERS SHARE A TEST PROCESS: the one the built package bundles, which
 the code under test writes through, and module-logger's default logger,
 which the test framework writes through. Each reports its own sinks.

 Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runKeyless, } from './child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Console warn lines tests

/**
 What the logger writes before its own report of a failure inside it.
 */
const LOGGER_REPORT_PREFIX = 'logger internal error: ';

/**
 Source the child runs: one warning logged through the built package inside
 `warnLinesDuring`, as the first line of the child's process, and the warnings
 kept printed as JSON.

 @returns Module source for `node --input-type=module --eval`

 @example
 ```ts
 await runKeyless({
   file: process.execPath,
   args: ['--input-type=module', '--eval', childSource(),],
 },);
 ```
 */
function childSource(): string {
  /**
   The fixture under test, by absolute URL so the child's working directory
   can be anywhere.
   */
  const fixture = import.meta.resolve('@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts',);
  /**
   The built package the fixture reads its logger from.
   */
  const built = import.meta.resolve('@monochromatic-dev/translation-repair-typography',);
  return [
    `const { warnLinesDuring } = await import(${JSON.stringify(fixture,)});`,
    `const { contextRoot } = await import(${JSON.stringify(built,)});`,
    'const { warned } = await warnLinesDuring({',
    '  run: async () => { contextRoot({ tag: "whiskers" }).warn("purr"); return "napped"; },',
    '});',
    'process.stdout.write(JSON.stringify(warned));',
  ].join('\n',);
}

await describe({
  name: 'warnLinesDuring',
  children: [
    it({
      name: 'KEEPS the work\'s warning and SENDS each logger\'s own report of a sink it could not verify to the '
        + 'console the divert stood in for, where the first line of the process starts the logger inside the divert '
        + '(ledger B140)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'console-warn-lines-', },);
        /**
         The log directory the child's loggers are told to use.
         */
        const logDirectory = join(
          scratch.path,
          'logs',
        );
        // A FILE WHERE THE LOG DIRECTORY GOES: the file sink's verify makes the
        // directory and fails on the file standing there.
        await writeFile(
          logDirectory,
          'a cat sleeps here, not a log\n',
        );
        /**
         The child as it finished, whatever it exited with.
         */
        const child = await runKeyless({
          file: process.execPath,
          args: [
            '--input-type=module',
            '--eval',
            childSource(),
          ],
          cwd: scratch.path,
          extra: { MONOCHROMATIC_LOG_DIRECTORY: logDirectory, },
        },);
        /**
         What a logger reports for the file standing where its log directory
         goes.
         */
        const report =
          `${LOGGER_REPORT_PREFIX}file sink verification failed: EEXIST: file already exists, mkdir '${logDirectory}'`;
        // TWO REPORTS, ONE FROM EACH LOGGER: the fixture starts the built package's
        // logger and the test framework's before it diverts, and each has its
        // own file sink to verify.
        expect({
          status: child.code,
          stdout: child.stdout,
          reports: child.stderr
            .split('\n',)
            .filter(function isLoggerReport(line,): boolean {
              return line.startsWith(LOGGER_REPORT_PREFIX,);
            },),
        },).toStrictEqual({
          status: 0,
          stdout: JSON.stringify(['[whiskers] purr',],),
          reports: [
            report,
            report,
          ],
        },);
      },
    },),
  ],
},);

//endregion Console warn lines tests
