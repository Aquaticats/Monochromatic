/**
 Tests for the procedure that prints a run folder's records the way the console showed them (ledger B1118).

 WHAT A PERSON MUST BE ABLE TO TRUST: the output of this command never carries a control sequence a message held, and
 never fails on a record a JSON tool refuses. The runbooks used to read the records with `jq --raw-output '.message'`,
 which prints a message raw (an escape sequence reaches the terminal) and exits with an error on a record holding a lone
 surrogate.

 Every folder is written under a scratch directory. Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { printRunRecords, } from '../../dist/final/node/index.mjs';
import { captureReport, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';

/**
 Record lines of a run: an escape sequence, a lone surrogate, a debug record and a cut line.
 */
const RECORD_LINES = [
  JSON.stringify({ level: 'info', message: 'START \u001B]0;title\u0007 tip', timestamp: 0, },),
  JSON.stringify({ level: 'warn', message: 'cut \uD800 here', timestamp: 1, },),
  JSON.stringify({ level: 'debug', message: 'a debug thought', timestamp: 2, },),
  'a foreign line \u001B[2J of the file',
];

/**
 Writes a run folder holding the records.

 @param dir - scratch directory

 @returns Path of the run folder

 @example
 ```ts
 const runDir = await writtenRun({ dir: scratch.path, },);
 ```
 */
async function writtenRun({ dir, }: { readonly dir: string; },): Promise<string> {
  /**
   Day directory of the run.
   */
  const day = join(
    dir,
    'corpus-pass-tabby',
    'log',
    '2026-10-31',
  );
  await mkdir(
    day,
    { recursive: true, },
  );
  await writeFile(
    join(
      day,
      '2026-10-31T00-00-00.000Z.00000000-0000-4000-8000-000000000001.log.jsonl',
    ),
    `${RECORD_LINES.join('\n',)}\n`,
  );
  return join(
    dir,
    'corpus-pass-tabby',
  );
}

/**
 Prints the named folders with the console diverted.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param typed - arguments after the command

 @returns The lines the command printed

 @example
 ```ts
 const lines = await printedBy({ sinon: ctx.sinon, typed: [runDir,], },);
 ```
 */
async function printedBy(
  {
    sinon,
    typed,
  }: {
    readonly sinon: DisposableSandbox;
    readonly typed: readonly string[];
  },
): Promise<readonly string[]> {
  return (await captureReport({
    sinon,
    run: async function printsRecords(): Promise<void> {
      await printRunRecords({
        line: lineOf({
          command: 'run-records',
          typed,
        },),
      },);
    },
  },)).lines;
}

await describe({
  name: printRunRecords.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS A LINE FOR EACH RECORD as the console showed it, the control characters written out, a lone '
        + 'surrogate and a foreign line not failing it, and no debug record',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-records-run-', },);
        expect(await printedBy({
          sinon: ctx.sinon,
          typed: [await writtenRun({ dir: scratch.path, },),],
        },),).toEqual([
          '[info] [1970-01-01T00:00:00.000Z] START \\u001B]0;title\\u0007 tip',
          '[warn] [1970-01-01T00:00:00.001Z] cut \uD800 here',
          'a foreign line \\u001B[2J of the file',
        ],);
      },
    },),

    it({
      name: 'PRINTS THE DEBUG RECORDS TOO under --verbose',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-records-run-', },);
        expect(await printedBy({
          sinon: ctx.sinon,
          typed: [
            '--verbose',
            await writtenRun({ dir: scratch.path, },),
          ],
        },),).toContain('[debug] [1970-01-01T00:00:00.002Z] a debug thought',);
      },
    },),

    it({
      name: 'REFUSES A RUN FOLDER NAMED TWICE by two spellings, and a folder that holds no record, before it prints '
        + 'a line',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-records-run-', },);
        /**
         The run folder.
         */
        const runDir = await writtenRun({ dir: scratch.path, },);
        expect(await rejectionOf(async function namesTwice(): Promise<unknown> {
          return await printedBy({
            sinon: ctx.sinon,
            typed: [
              runDir,
              join(
                runDir,
                'log',
              ),
            ],
          },);
        },),).toBeInstanceOf(StatedRefusalError,);
        expect(await rejectionOf(async function namesNothing(): Promise<unknown> {
          return await printedBy({
            sinon: ctx.sinon,
            typed: [scratch.path,],
          },);
        },),).toBeInstanceOf(StatedRefusalError,);
      },
    },),
  ],
},);
